// Unit tests over a fake Unix-socket listener that speaks the same NDJSON discipline as the real
// server: one request line in, exactly one response line out, per connection. These tests pin the
// client's failure discipline without a kernel; the recorded-frame tests in golden_test.go pin the
// wire shapes against the real server.
package sfwp

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
	"path/filepath"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// fakeServer is a scriptable stand-in for the authority's socket.
type fakeServer struct {
	t      *testing.T
	ln     net.Listener
	socket string

	mu       sync.Mutex
	requests []string // every request line received, in order
	// handler answers one request line; returning write "" and close true severs the connection
	// without responding (the transport-failure path).
	handler func(line string) (write string, close bool)

	conns  atomic.Int64
	closed chan struct{}
}

func newFakeServer(t *testing.T, handler func(string) (string, bool)) *fakeServer {
	t.Helper()
	dir, err := os.MkdirTemp("", "sfwp-fake-*")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { os.RemoveAll(dir) })
	socket := filepath.Join(dir, "fake.sock")
	ln, err := net.Listen("unix", socket)
	if err != nil {
		t.Fatal(err)
	}
	fs := &fakeServer{t: t, ln: ln, socket: socket, handler: handler, closed: make(chan struct{})}
	go fs.acceptLoop()
	t.Cleanup(func() { ln.Close() })
	return fs
}

func (fs *fakeServer) acceptLoop() {
	for {
		nc, err := fs.ln.Accept()
		if err != nil {
			close(fs.closed)
			return
		}
		fs.conns.Add(1)
		go fs.serve(nc)
	}
}

func (fs *fakeServer) serve(nc net.Conn) {
	defer nc.Close()
	br := bufio.NewReader(nc)
	for {
		nc.SetReadDeadline(time.Now().Add(10 * time.Second))
		line, err := br.ReadString('\n')
		if err != nil {
			return
		}
		fs.mu.Lock()
		fs.requests = append(fs.requests, line)
		handler := fs.handler
		fs.mu.Unlock()
		if handler == nil {
			return
		}
		reply, sever := handler(line)
		if sever {
			return // drop the connection without answering
		}
		nc.SetWriteDeadline(time.Now().Add(5 * time.Second))
		if _, err := io.WriteString(nc, reply+"\n"); err != nil {
			return
		}
	}
}

func (fs *fakeServer) requestCount(verb string) int {
	fs.mu.Lock()
	defer fs.mu.Unlock()
	n := 0
	for _, r := range fs.requests {
		var probe struct {
			Verb string `json:"verb"`
		}
		if json.Unmarshal([]byte(r), &probe) == nil && probe.Verb == verb {
			n++
		}
	}
	return n
}

func testConfig(socket string) Config {
	return Config{
		SocketPath:     socket,
		MaxConns:       2,
		RequestTimeout: 2 * time.Second,
		ConnectTimeout: 1 * time.Second,
		RecoveryBudget: 3 * time.Second,
		BackoffBase:    20 * time.Millisecond,
		BackoffMax:     100 * time.Millisecond,
		SubscribeIdle:  5 * time.Second,
	}
}

// TOOTH (a): the server does not know the verb (an older kernel). Expected: a typed unavailable
// refusal, no panic, and no retry storm - the fake counts every request line it receives.
func TestUnknownVerbIsTypedUnavailableWithoutRetryStorm(t *testing.T) {
	fs := newFakeServer(t, func(line string) (string, bool) {
		// Byte-for-byte the shape serde's unknown-variant failure produces on the real server:
		// an "error" string with NO error_class.
		return `{"error":"bad request: unknown variant ` + "`case_add_item`" + `, expected one of submit, status, approve, reject, agent_list, agent_probe, delegate, cancel_delegation, ask, system_hello, system_describe, system_get_schema, request_get_status, events_subscribe, events_unsubscribe, events_get_range, readiness_get, identity_get, case_entry_options, case_preflight, case_commit, case_list, case_get_overview, case_get_horizon, approval_list, approval_decide, run_list, run_get, asset_list, delegation_preview, delegation_list"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	req, err := NewCaseAddItem("case_x", map[string]any{"plan_item_id": "i1", "name": "n", "item_kind": "sandboxed_task", "settlement_criteria": map[string]any{}},
		"", "req-unknown-verb-1", Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	resp, err := client.Do(context.Background(), req)

	if err == nil {
		t.Fatalf("expected a typed error, got a response: %v", resp)
	}
	if got := apperr.KindOf(err); got != apperr.KindUnavailable {
		t.Fatalf("unknown verb must be typed unavailable, got %s (%v)", got, err)
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) {
		t.Fatalf("the refusal must stay in the error chain for its class, got %v", err)
	}
	if !refusal.UnknownVerb {
		t.Fatalf("the refusal must be marked UnknownVerb, got %+v", refusal)
	}
	if got := fs.requestCount("case_add_item"); got != 1 {
		t.Fatalf("retry storm: the unknown verb was sent %d times, want exactly 1", got)
	}
}

// A typed identity refusal keeps its class and maps to authority_denied.
func TestIdentityRefusalKeepsClass(t *testing.T) {
	fs := newFakeServer(t, func(line string) (string, bool) {
		return `{"error":"actor ` + "`intruder`" + ` is not bound to uid 1000 in this cell","error_class":"identity_not_bound","no_side_effect":true,"next_lawful_action":"Use an actor bound to this operating-system user"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	req, err := NewCaseCommit("e2e-sentry-chain@0.1.0", nil, "", "", 0, "req-identity-1", nil,
		Governance{Actor: Actor{ActorID: "intruder", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	_, err = client.Do(context.Background(), req)
	if err == nil {
		t.Fatal("expected the identity refusal to surface as an error")
	}
	if got := apperr.KindOf(err); got != apperr.KindAuthorityDenied {
		t.Fatalf("identity refusal must map to authority_denied, got %s", got)
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) || refusal.Class != "identity_not_bound" {
		t.Fatalf("class identity_not_bound must survive translation, got %+v", refusal)
	}
}

// A mutation whose connection is severed mid-flight is never re-sent: the client resolves the
// outcome through request.get_status, and reports honestly when the authority has no record.
func TestSeveredMutationIsRecoveredNotResent(t *testing.T) {
	var phase atomic.Int32 // 0: sever the commit, 1: answer get_status "unknown"
	fs := newFakeServer(t, func(line string) (string, bool) {
		var probe struct {
			Verb string `json:"verb"`
		}
		if err := json.Unmarshal([]byte(line), &probe); err != nil {
			return `{"error":"bad json"}`, false
		}
		switch probe.Verb {
		case "case_commit":
			if phase.CompareAndSwap(0, 1) {
				return "", true // sever: no response, connection dropped
			}
			return "", true // any re-sent commit is also severed (and counted)
		case "request_get_status":
			return `{"request_id":"req-sever-1","status":"unknown"}`, false
		}
		return `{"error":"unhandled"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	req, err := NewCaseCommit("e2e-sentry-chain@0.1.0", map[string]string{"dataset_name": "d"}, "", "", 0, "req-sever-1", nil,
		Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	_, err = client.Do(context.Background(), req)
	if err == nil {
		t.Fatal("expected the unresolved outcome to surface as an error")
	}
	if got := apperr.KindOf(err); got != apperr.KindUnavailable {
		t.Fatalf("an unrecorded outcome must be typed unavailable/unresolved, got %s (%v)", got, err)
	}
	if got := fs.requestCount("case_commit"); got != 1 {
		t.Fatalf("the mutation was sent %d times; a severed mutation must never be re-sent", got)
	}
	if got := fs.requestCount("request_get_status"); got == 0 {
		t.Fatal("the client must resolve the outcome through request.get_status")
	}
}

// A mutation whose outcome the authority DID record returns that recorded outcome verbatim.
func TestSeveredMutationReturnsRecordedOutcome(t *testing.T) {
	fs := newFakeServer(t, func(line string) (string, bool) {
		var probe struct {
			Verb string `json:"verb"`
		}
		if err := json.Unmarshal([]byte(line), &probe); err != nil {
			return `{"error":"bad json"}`, false
		}
		switch probe.Verb {
		case "case_commit":
			return "", true // sever every commit
		case "request_get_status":
			return `{"request_id":"req-recorded-1","status":"completed","method":"case.commit","outcome":{"case_id":"case_0177","state":"awaiting_activation","exit_code":0}}`, false
		}
		return `{"error":"unhandled"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	req, err := NewCaseCommit("e2e-sentry-chain@0.1.0", nil, "", "", 0, "req-recorded-1", nil,
		Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	resp, err := client.Do(context.Background(), req)
	if err != nil {
		t.Fatalf("the recorded outcome must be returned as the response, got %v", err)
	}
	var commit CommitView
	if err := resp.Into(&commit); err != nil {
		t.Fatal(err)
	}
	if commit.CaseID != "case_0177" {
		t.Fatalf("recorded outcome not surfaced verbatim: %+v", commit)
	}
}

// Inspect verbs may safely retry once on a transport failure; a second failure surfaces.
func TestInspectRetriesOnceOnTransportFailure(t *testing.T) {
	var caseList atomic.Int32
	var down atomic.Bool
	fs := newFakeServer(t, func(line string) (string, bool) {
		var probe struct {
			Verb string `json:"verb"`
		}
		if err := json.Unmarshal([]byte(line), &probe); err != nil {
			return `{"error":"bad json"}`, false
		}
		if probe.Verb == "case_list" {
			n := caseList.Add(1)
			if n == 1 {
				return "", true // first attempt severed
			}
			if down.Load() {
				return "", true
			}
			return `{"cases":[{"case_id":"case_1","case_state":"active","summary":"s","created_at":"2026-09-23T00:00:00Z","run_count":0}],"unreadable":[]}`, false
		}
		return `{"error":"unhandled"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	// First attempt severed, retry succeeds.
	if _, err := client.Do(context.Background(), NewCaseList()); err != nil {
		t.Fatalf("inspect retry after transport failure should succeed: %v", err)
	}
	if got := caseList.Load(); got != 2 {
		t.Fatalf("expected exactly one retry (2 sends), got %d", got)
	}

	// Server fully down afterwards: typed unavailable, no storm.
	down.Store(true)
	caseList.Store(0)
	_, err = client.Do(context.Background(), NewCaseList())
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("expected typed unavailable, got %v", err)
	}
	if got := caseList.Load(); got > 2 {
		t.Fatalf("retry storm: %d sends", got)
	}
}

// Mutations without a request id are refused client-side: the authority requires the durable
// locator before admission, and an uncorrelated mutation would be unrecoverable.
func TestMutationRequiresRequestID(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) { return `{"error":"unhandled"}`, false })
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	g := Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}}
	for name, build := range map[string]func() (*Request, error){
		"case_commit": func() (*Request, error) { return NewCaseCommit("t@1", nil, "", "", 0, "", nil, g) },
		"approval_decide": func() (*Request, error) {
			return NewApprovalDecide("c", "a", "approve", "", "", g)
		},
		"case_add_item":  func() (*Request, error) { return NewCaseAddItem("c", nil, "", "", g) },
		"case_reopen":    func() (*Request, error) { return NewCaseReopen("c", "", "", g) },
		"case_terminate": func() (*Request, error) { return NewCaseTerminate("c", "r", "", "", g) },
		"case_advance":   func() (*Request, error) { return NewCaseAdvance("c", "", 0, "", g) },
		"item_execute": func() (*Request, error) {
			return NewItemExecute("c", "i", "", 0, "", g)
		},
		"human_task_complete": func() (*Request, error) {
			return NewHumanTaskComplete("c", "i", "because", "", "", g)
		},
	} {
		t.Run(name, func(t *testing.T) {
			_, err := build()
			if err == nil {
				t.Fatalf("%s must refuse an empty request_id before sending", name)
			}
			if apperr.KindOf(err) != apperr.KindInvalid {
				t.Fatalf("%s refusal must be typed invalid, got %v", name, err)
			}
		})
	}
}

// The subscription resubscribes from its last delivered cursor after the connection drops, and
// never delivers a duplicate or non-monotonic cursor.
func TestSubscriptionResumesFromLastCursor(t *testing.T) {
	dir, err := os.MkdirTemp("", "sfwp-sub-*")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { os.RemoveAll(dir) })
	socket := filepath.Join(dir, "sub.sock")

	var mu sync.Mutex
	var fromCursors []string
	ln, err := net.Listen("unix", socket)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { ln.Close() })

	// Each connection: read the events.subscribe line, ack it, push one event frame whose cursor
	// is strictly after the requested from_cursor, then drop the connection (forcing a resume).
	go func() {
		for {
			nc, err := ln.Accept()
			if err != nil {
				return
			}
			go func(nc net.Conn) {
				defer nc.Close()
				br := bufio.NewReader(nc)
				line, err := br.ReadString('\n')
				if err != nil {
					return
				}
				var probe struct {
					Verb       string `json:"verb"`
					FromCursor string `json:"from_cursor"`
				}
				if json.Unmarshal([]byte(line), &probe) != nil || probe.Verb != "events_subscribe" {
					return
				}
				mu.Lock()
				fromCursors = append(fromCursors, probe.FromCursor)
				n := len(fromCursors)
				mu.Unlock()
				io.WriteString(nc, `{"ok":true,"subscribed":true,"replayed":0}`+"\n")
				frame := map[string]any{
					"type": "event",
					"event": map[string]any{
						"cursor":       fmt.Sprintf("CUR_%04d", n),
						"kind":         "case.submitted",
						"detail":       map[string]any{},
						"committed_at": "2026-09-23T00:00:00Z",
					},
				}
				raw, _ := json.Marshal(frame)
				io.WriteString(nc, string(raw)+"\n")
				time.Sleep(50 * time.Millisecond) // let the client deliver it
			}(nc)
		}
	}()

	client, err := New(testConfig(socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	sub, err := client.Subscribe(ctx)
	if err != nil {
		t.Fatal(err)
	}

	var cursors []string
	for range 3 {
		select {
		case ev, ok := <-sub.Events:
			if !ok {
				t.Fatalf("stream closed early after %d frames", len(cursors))
			}
			cursors = append(cursors, ev.Cursor)
		case <-time.After(5 * time.Second):
			t.Fatalf("timed out after %d frames", len(cursors))
		}
	}
	if len(cursors) != 3 || cursors[0] != "CUR_0001" || cursors[1] != "CUR_0002" || cursors[2] != "CUR_0003" {
		t.Fatalf("expected strictly monotonic cursors CUR_0001..3, got %v", cursors)
	}
	mu.Lock()
	defer mu.Unlock()
	if len(fromCursors) != 3 {
		t.Fatalf("expected 3 subscribe passes, got %d (%v)", len(fromCursors), fromCursors)
	}
	if fromCursors[1] != "CUR_0001" || fromCursors[2] != "CUR_0002" {
		t.Fatalf("each resume must continue from the last delivered cursor, got %v", fromCursors)
	}
}
