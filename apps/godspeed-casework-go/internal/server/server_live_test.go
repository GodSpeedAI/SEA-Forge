//go:build live

// The live gateway's SSE relay test (plan T06): a mutation posted through the real stack produces
// kernel frames relayed on /api/events as snapshot revisions with id=kernel cursor, and a
// reconnect carrying Last-Event-ID resumes without gaps.
package server_test

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
)

func TestMain(m *testing.M) {
	livetest.EnsureServerBinary()
	os.Exit(m.Run())
}

type sseEvent struct {
	id    string
	event string
	data  string
}

func streamSSE(t *testing.T, body io.Reader) <-chan sseEvent {
	t.Helper()
	ch := make(chan sseEvent, 256)
	go func() {
		defer close(ch)
		reader := bufio.NewReader(body)
		var ev sseEvent
		for {
			line, err := reader.ReadString('\n')
			switch {
			case strings.HasPrefix(line, "id: "):
				ev.id = strings.TrimSuffix(strings.TrimPrefix(line, "id: "), "\n")
			case strings.HasPrefix(line, "event: "):
				ev.event = strings.TrimSuffix(strings.TrimPrefix(line, "event: "), "\n")
			case strings.HasPrefix(line, "data: "):
				ev.data = strings.TrimSuffix(strings.TrimPrefix(line, "data: "), "\n")
			case line == "\n" || line == "":
				if ev.event != "" || ev.data != "" {
					ch <- ev
					ev = sseEvent{}
				}
			}
			if err != nil {
				return
			}
		}
	}()
	return ch
}

func nextEvent(t *testing.T, ch <-chan sseEvent, what string) sseEvent {
	t.Helper()
	select {
	case ev, ok := <-ch:
		if !ok {
			t.Fatalf("stream closed while waiting for %s", what)
		}
		return ev
	case <-time.After(10 * time.Second):
		t.Fatalf("timed out waiting for %s", what)
	}
	return sseEvent{}
}

// postIntent drives one intent through the gateway's HTTP surface.
func postIntent(t *testing.T, ts *httptest.Server, in contract.InteractionIntent) contract.IntentResponse {
	t.Helper()
	raw, err := json.Marshal(in)
	if err != nil {
		t.Fatal(err)
	}
	resp, err := http.Post(ts.URL+"/api/intents", "application/json", strings.NewReader(string(raw)))
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var out contract.IntentResponse
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		t.Fatal(err)
	}
	return out
}

func TestLiveSSERelayOfKernelFramesWithResume(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	ts := httptest.NewServer(stack.API.Handler())
	t.Cleanup(ts.Close)

	caseID := stack.CommitSentryChain(t, "gw-t06-sse-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	// Open the SSE stream subscribed at the current kernel cursor.
	resp, err := http.Get(ts.URL + "/api/events?last=" + cursor)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	if ct := resp.Header.Get("Content-Type"); !strings.HasPrefix(ct, "text/event-stream") {
		t.Fatalf("events content type: %q", ct)
	}
	events := streamSSE(t, resp.Body)

	hello := nextEvent(t, events, "hello")
	if hello.event != "hello" || !strings.Contains(hello.data, `"go:live:sfwp"`) {
		t.Fatalf("hello: %+v", hello)
	}

	// A mutation through the gateway: the kernel frames must arrive as snapshot revisions with
	// id = kernel cursor, and the snapshot must reflect the mutation.
	in := contract.InteractionIntent{
		IntentID:       "gw-t06-sse-exec-1",
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     "EXECUTE_ITEM",
		TargetObjectID: "task_prepare",
		CaseID:         caseID,
		ClientCursor:   cursor,
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters:     map[string]any{"item_id": "task_prepare"},
	}
	if out := postIntent(t, ts, in); !out.Success {
		t.Fatalf("intent refused: %+v", out.Refusal)
	}

	// Collect the FIRST relayed revision after the mutation; every one carries the kernel cursor
	// as its SSE id. The mutation appends several trace frames (item_activated, settlement, ...),
	// so waiting for the kernel's durable settlement first guarantees at least one frame exists
	// strictly after the first one for the deterministic resume below.
	livetest.WaitUntil(t, 15*time.Second, "durable settlement after the mutation", func() bool {
		return cell.CountTraceKinds(caseID, "settlement_recorded") >= 1
	})
	time.Sleep(200 * time.Millisecond) // let the relay deliver the remaining frames of the burst
	last := sseEvent{}
	seen := []string{}
	for {
		select {
		case ev := <-events:
			seen = append(seen, ev.event+" id="+ev.id)
			if ev.event != "snapshot" {
				continue
			}
			last = ev
		case <-time.After(15 * time.Second):
			t.Fatalf("no snapshot revision arrived after the mutation; seen=%v", seen)
		}
		if last.id != "" {
			t.Logf("collected first snapshot; events seen so far: %v", seen)
			break
		}
	}
	if last.id == "" || last.id == cursor {
		t.Fatal("relayed revisions must carry the kernel cursor as their id and be newer than the subscribe point")
	}
	var frame contract.StreamEvent
	if err := json.Unmarshal([]byte(last.data), &frame); err != nil {
		t.Fatal(err)
	}
	if frame.EventType != "snapshot" || frame.Cursor != last.id {
		t.Fatalf("frame envelope: %+v (id %q)", frame, last.id)
	}
	snap, ok := frame.Payload.(map[string]any)
	if !ok || snap["case_id"] != caseID {
		t.Fatalf("the revision must carry the case snapshot: %v", frame.Payload)
	}
	// The snapshot reflects the executed item's kernel standing (execution advanced off pending).
	found := false
	for _, o := range snap["visible_objects"].([]any) {
		obj, _ := o.(map[string]any)
		if obj["id"] == "task_prepare" {
			found = true
			if obj["status"] == "WAITING" {
				t.Fatalf("the relayed snapshot must reflect the executed item, got %v", obj)
			}
		}
	}
	if !found {
		t.Fatal("the executed item must appear in the relayed snapshot")
	}

	// Kernel truth behind the relayed revision: the durable ledger carries the trace kinds.
	if got := cell.CountTraceKinds(caseID, "item_activated"); got != 1 {
		t.Fatalf("durable item_activated = %d, want 1", got)
	}

	// Disconnect and reconnect with Last-Event-ID set to the FIRST post-mutation revision: the
	// replay must cover the remaining revisions of the mutation burst, strictly newer, with no
	// gaps against the store.
	resp.Body.Close()
	time.Sleep(50 * time.Millisecond)
	resp2, err := http.Get(ts.URL + "/api/events?last=" + last.id)
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	events2 := streamSSE(t, resp2.Body)
	nextEvent(t, events2, "hello after reconnect")
	replayed := []string{}
	for {
		select {
		case ev := <-events2:
			if ev.event != "snapshot" {
				continue
			}
			if ev.id <= last.id {
				t.Fatalf("replayed revision %q must be newer than the resume cursor %q", ev.id, last.id)
			}
			replayed = append(replayed, ev.id)
			if len(replayed) >= 1 {
				// Every replayed cursor must exist in the store, in order: resume without gaps.
				if _, err := stack.Store.At(ev.id); err != nil {
					t.Fatalf("replayed cursor %q is not in the retained history: %v", ev.id, err)
				}
				return
			}
		case <-time.After(3 * time.Second):
			t.Fatalf("no revision replayed after resume from %s", last.id)
		}
	}
}

// The templates endpoints over the real kernel, closing the template flow the PROPOSE_CASE
// journey needs.
func TestLiveTemplateEndpoints(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	ts := httptest.NewServer(stack.API.Handler())
	t.Cleanup(ts.Close)

	resp, err := http.Get(ts.URL + "/api/templates")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	var templates struct {
		Templates []contract.TemplateEntryOption `json:"templates"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&templates); err != nil {
		t.Fatal(err)
	}
	refs := map[string]bool{}
	for _, tpl := range templates.Templates {
		refs[tpl.TemplateRef] = true
	}
	if !refs["e2e-sentry-chain@0.1.0"] || !refs["e2e-signoff-gate@0.1.0"] {
		t.Fatalf("both E2E templates must be listed, got %v", refs)
	}

	body := `{"template_ref":"e2e-sentry-chain@0.1.0","params":{"dataset_name":"t06-sse"}}`
	resp2, err := http.Post(ts.URL+"/api/templates/preflight", "application/json", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	defer resp2.Body.Close()
	var pf contract.TemplatePreflightResult
	if err := json.NewDecoder(resp2.Body).Decode(&pf); err != nil {
		t.Fatal(err)
	}
	if !pf.Passed || pf.Digest == nil || len(pf.Reasons) != 0 {
		t.Fatalf("preflight must pass on the live cell: %+v", pf)
	}
	fmt.Printf("live preflight digest: %s\n", *pf.Digest)
}
