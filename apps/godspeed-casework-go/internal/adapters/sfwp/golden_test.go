// Table-driven codec tests against RECORDED real-server frames (testdata/frames.json, captured by
// golden_capture_test.go against the real kernel). Each case rebuilds the request through this
// package's builders and asserts (a) the encoded line is JSON-equal to the recorded request, and
// (b) the recorded response decodes through this package's decoder into the expected typed view.
//
// JSON-equality (not byte equality) is the right pin: the server parses objects, and Rust's serde
// and Go's encoding/json order keys differently by construction.
package sfwp

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// goldenFrame is one recorded request/response line pair (plus metadata), as written by the
// live-tagged capture test into testdata/frames.json.
type goldenFrame struct {
	Name     string `json:"name"`
	Request  string `json:"request"`
	Response string `json:"response"`
	Event    bool   `json:"event,omitempty"`
	Note     string `json:"note,omitempty"`
}

type goldenFile []goldenFrame

func loadGolden(t *testing.T) goldenFile {
	t.Helper()
	raw, err := os.ReadFile(filepath.Join("testdata", "frames.json"))
	if err != nil {
		t.Fatalf("golden frames missing (record them with GOLDEN_CAPTURE=testdata go test -tags live -run TestGoldenCapture): %v", err)
	}
	var frames goldenFile
	if err := json.Unmarshal(raw, &frames); err != nil {
		t.Fatalf("golden frames unreadable: %v", err)
	}
	if len(frames) == 0 {
		t.Fatal("golden frames file is empty")
	}
	return frames
}

func byName(t *testing.T, frames goldenFile, name string) goldenFrame {
	t.Helper()
	for _, f := range frames {
		if f.Name == name {
			return f
		}
	}
	t.Fatalf("no golden frame named %q", name)
	return goldenFrame{}
}

// jsonEqual compares two JSON documents by value.
func jsonEqual(t *testing.T, a, b string) bool {
	t.Helper()
	var va, vb any
	if err := json.Unmarshal([]byte(a), &va); err != nil {
		t.Fatalf("left document is not JSON: %s (%v)", a, err)
	}
	if err := json.Unmarshal([]byte(b), &vb); err != nil {
		t.Fatalf("right document is not JSON: %s (%v)", b, err)
	}
	na, _ := json.Marshal(va)
	nb, _ := json.Marshal(vb)
	return string(na) == string(nb)
}

func operatorGolden() Governance {
	return Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}}
}

// TestGoldenRequestEncoding pins every builder's wire form to the recorded real-server request.
func TestGoldenRequestEncoding(t *testing.T) {
	frames := loadGolden(t)

	cases := []struct {
		name  string
		build func() ([]byte, error)
	}{
		{"system.hello", func() ([]byte, error) { return EncodeRequest(NewSystemHello("golden-capture")) }},
		{"readiness.get", func() ([]byte, error) { return EncodeRequest(NewReadinessGet()) }},
		{"identity.get", func() ([]byte, error) { return EncodeRequest(NewIdentityGet(nil)) }},
		{"request.get_status.unknown", func() ([]byte, error) { return EncodeRequest(NewRequestGetStatus("req-never-issued")) }},
		{"case.entry_options", func() ([]byte, error) { return EncodeRequest(NewCaseEntryOptions()) }},
		{"case.preflight", func() ([]byte, error) { return EncodeRequest(NewCasePreflight(sentryChainRef, sentryChainParams())) }},
		{"case.list", func() ([]byte, error) { return EncodeRequest(NewCaseList()) }},
		{"approval.list", func() ([]byte, error) { return EncodeRequest(NewApprovalList("")) }},
		{"events.get_range", func() ([]byte, error) { return EncodeRequest(NewEventsGetRange("", "", 5)) }},
		{"events.subscribe.ack", func() ([]byte, error) { return EncodeRequest(NewEventsSubscribe("")) }},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			golden := byName(t, frames, tc.name)
			got, err := tc.build()
			if err != nil {
				t.Fatal(err)
			}
			if !jsonEqual(t, string(got), golden.Request) {
				t.Fatalf("encoded request drifted from the recorded frame:\n got: %s\nwant: %s", got, golden.Request)
			}
		})
	}

	// Frames whose request ids / case ids are baked into the recording are rebuilt with the
	// recorded values so the comparison stays meaningful.
	pinned := []struct {
		name  string
		build func(golden goldenFrame) ([]byte, error)
	}{
		{"case.commit", func(g goldenFrame) ([]byte, error) {
			return goldenPinnedCommit(t, g)
		}},
		{"case.commit.signoff", func(g goldenFrame) ([]byte, error) {
			return goldenPinnedCommit(t, g)
		}},
		{"case.commit.third", func(g goldenFrame) ([]byte, error) {
			return goldenPinnedCommit(t, g)
		}},
		{"case.commit.identity_not_bound", func(g goldenFrame) ([]byte, error) {
			return goldenPinnedCommit(t, g)
		}},
		{"case.commit.identity_delegation_refused", func(g goldenFrame) ([]byte, error) {
			return goldenPinnedCommit(t, g)
		}},
	}
	for _, tc := range pinned {
		t.Run(tc.name, func(t *testing.T) {
			golden := byName(t, frames, tc.name)
			got, err := tc.build(golden)
			if err != nil {
				t.Fatal(err)
			}
			if !jsonEqual(t, string(got), golden.Request) {
				t.Fatalf("encoded request drifted from the recorded frame:\n got: %s\nwant: %s", got, golden.Request)
			}
		})
	}
}

// goldenPinnedCommit rebuilds a recorded commit request from its own recorded fields (ids differ
// per capture run), through the real builder - including the precondition pin the capture carried
// from its preflight, so the rebuild covers the whole recorded request.
func goldenPinnedCommit(t *testing.T, golden goldenFrame) ([]byte, error) {
	t.Helper()
	var rec struct {
		Verb        string            `json:"verb"`
		TemplateRef string            `json:"template_ref"`
		Params      map[string]string `json:"params"`
		Policy      string            `json:"policy"`
		Entity      string            `json:"entity"`
		Process     string            `json:"process"`
		Timeout     int               `json:"timeout"`
		RequestID   string            `json:"request_id"`
		Actor       *struct {
			ActorID string `json:"actor_id"`
			Role    string `json:"role"`
		} `json:"actor"`
		OnBehalfOf *struct {
			ActorID string `json:"actor_id"`
			Role    string `json:"role"`
		} `json:"on_behalf_of"`
		Preconditions *struct {
			Records []struct {
				Ref            string `json:"ref"`
				ExpectedDigest string `json:"expected_digest"`
			} `json:"records"`
		} `json:"preconditions"`
	}
	if err := json.Unmarshal([]byte(golden.Request), &rec); err != nil {
		t.Fatal(err)
	}
	g := Governance{}
	if rec.Actor != nil {
		g.Actor = Actor{ActorID: rec.Actor.ActorID, Role: rec.Actor.Role}
	}
	if rec.OnBehalfOf != nil {
		g.OnBehalfOf = &Actor{ActorID: rec.OnBehalfOf.ActorID, Role: rec.OnBehalfOf.Role}
	}
	var pin *PreconditionWire
	if rec.Preconditions != nil {
		wire := &PreconditionWire{}
		for _, r := range rec.Preconditions.Records {
			wire.Records = append(wire.Records, RecordDigestWire{Ref: r.Ref, ExpectedDigest: r.ExpectedDigest})
		}
		pin = wire
	}
	req, err := NewCaseCommit(rec.TemplateRef, rec.Params, rec.Policy, rec.Process, rec.Timeout, rec.RequestID, pin, g)
	if err != nil {
		t.Fatal(err)
	}
	return EncodeRequest(req)
}

// TestGoldenResponseDecoding pins the decoder against the recorded real-server responses.
func TestGoldenResponseDecoding(t *testing.T) {
	frames := loadGolden(t)

	decode := func(name string, v any) {
		t.Helper()
		golden := byName(t, frames, name)
		resp, err := DecodeResponse([]byte(golden.Response))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		if resp.Err != nil {
			t.Fatalf("%s: recorded happy response decoded as a refusal: %+v", name, resp.Err)
		}
		if err := resp.Into(v); err != nil {
			t.Fatalf("%s: %v", name, err)
		}
	}
	decodeRefusal := func(name, wantClass string, wantKind apperr.Kind, unknownVerb bool) {
		t.Helper()
		golden := byName(t, frames, name)
		resp, err := DecodeResponse([]byte(golden.Response))
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		if resp.Err == nil {
			t.Fatalf("%s: recorded refusal decoded as a success", name)
		}
		if resp.Err.Class != wantClass {
			t.Fatalf("%s: class %q, want %q", name, resp.Err.Class, wantClass)
		}
		if resp.Err.UnknownVerb != unknownVerb {
			t.Fatalf("%s: UnknownVerb=%v, want %v", name, resp.Err.UnknownVerb, unknownVerb)
		}
		typed := resp.Err.appErr(name)
		if typed.Kind != wantKind {
			t.Fatalf("%s: kind %s, want %s", name, typed.Kind, wantKind)
		}
		var back *Refusal
		if !errors.As(error(typed), &back) || back.Class != wantClass {
			t.Fatalf("%s: class did not survive the error chain", name)
		}
	}

	var hello HelloView
	decode("system.hello", &hello)
	if hello.ServerProtocolVersion != "1" {
		t.Fatalf("hello: %+v", hello)
	}
	for _, want := range []string{"case.commit", "case.add_item", "item.execute", "human_task.complete", "artifact.get"} {
		found := false
		for _, m := range hello.ImplementedMethods {
			if m == want {
				found = true
			}
		}
		if !found {
			t.Fatalf("recorded server does not implement %s", want)
		}
	}

	var readiness ReadinessView
	decode("readiness.get", &readiness)
	if readiness.Overall == "" {
		t.Fatal("readiness without an overall verdict")
	}

	var identity IdentityView
	decode("identity.get", &identity)
	if len(identity.Available) != 1 || identity.Available[0].ActorID != "operator_local" {
		t.Fatalf("identity: %+v", identity)
	}

	var unknownStatus RequestStatusView
	decode("request.get_status.unknown", &unknownStatus)
	if unknownStatus.Status != "unknown" {
		t.Fatalf("request.get_status.unknown: %+v", unknownStatus)
	}

	var options EntryOptionsView
	decode("case.entry_options", &options)
	if len(options.Templates) != 2 {
		t.Fatalf("entry options: %+v", options)
	}

	var preflight PreflightView
	decode("case.preflight", &preflight)
	if !preflight.OK || len(preflight.Items) != 3 || preflight.Precondition == nil {
		t.Fatalf("preflight: %+v", preflight)
	}

	var commit CommitView
	decode("case.commit", &commit)
	if commit.CaseID == "" || commit.State == "" {
		t.Fatalf("commit: %+v", commit)
	}

	var doneStatus RequestStatusView
	decode("request.get_status.completed", &doneStatus)
	if doneStatus.Status != "completed" || len(doneStatus.Outcome) == 0 {
		t.Fatalf("completed status: %+v", doneStatus)
	}

	var list CaseListView
	decode("case.list", &list)
	if len(list.Cases) == 0 {
		t.Fatal("case.list recorded no cases")
	}

	var overview CaseOverviewView
	decode("case.get_overview", &overview)
	if overview.TemplateRef == nil || *overview.TemplateRef != sentryChainRef || overview.ItemCount != 3 {
		t.Fatalf("overview: %+v", overview)
	}

	var horizon CaseHorizonView
	decode("case.get_horizon", &horizon)
	if len(horizon.Items) != 3 {
		t.Fatalf("horizon: %+v", horizon)
	}

	var approvals ApprovalListView
	decode("approval.list", &approvals)

	var add AddItemView
	decode("case.add_item", &add)
	if add.PlanItemID != "golden_extra" || add.ProposedBy != "operator_local" {
		t.Fatalf("add_item: %+v", add)
	}

	var exec AdvanceView
	decode("item.execute", &exec)
	if len(exec.Episodes) != 1 || exec.Episodes[0].ItemID != "task_prepare" || exec.Episodes[0].RunID == "" {
		t.Fatalf("item.execute: %+v", exec)
	}
	// The episode's terminal status is the authority's own verdict. While the
	// T05 dispatch finding stands, the recorded reality is `rejected` with the
	// dispatch-finding basis; the accepted-only pin returns automatically once
	// the kernel executes write_file items (the re-captured frame then reads
	// accepted, and the skip disappears).
	switch exec.Episodes[0].SettlementStatus {
	case "accepted":
	case "rejected":
		t.Run("item.execute.accepted_episode_unavailable", func(t *testing.T) {
			t.Skipf("recorded episode settled rejected: %s", executionFindingNote)
		})
	default:
		t.Fatalf("item.execute recorded a non-terminal episode status: %+v", exec)
	}

	// The artifact.get happy frame exists only when the capture could obtain a
	// captured artifact. While the T05 dispatch finding stands (the kernel
	// cannot execute write_file items, so no artifact is ever captured), the
	// frame is a documented tombstone with an empty Response: the decode is
	// skipped LOUDLY with the finding, never silently. Once the kernel is
	// fixed the tombstone is replaced by a real frame and this branch goes
	// back to pinning the decode.
	if artifactFrame := byName(t, frames, "artifact.get"); artifactFrame.Response == "" {
		t.Run("artifact.get.happy_frame_unavailable", func(t *testing.T) {
			t.Skipf("artifact.get happy frame not capturable: %s", artifactFrame.Note)
		})
	} else {
		var artifact ArtifactView
		decode("artifact.get", &artifact)
		data, err := artifact.Bytes()
		if err != nil || len(data) == 0 {
			t.Fatalf("artifact: %v", err)
		}
	}

	var human HumanTaskCompleteView
	decode("human_task.complete", &human)
	if !human.OK {
		t.Fatalf("human task: %+v", human)
	}

	var term TerminateView
	decode("case.terminate", &term)
	if term.CaseState != "terminated" {
		t.Fatalf("terminate: %+v", term)
	}

	var reopened ReopenView
	decode("case.reopen", &reopened)
	if reopened.CaseState != "active" {
		t.Fatalf("reopen: %+v", reopened)
	}

	var ack SubscribeAckView
	decode("events.subscribe.ack", &ack)
	if !ack.Subscribed {
		t.Fatalf("subscribe ack: %+v", ack)
	}

	var rangeView struct {
		Events []Event `json:"events"`
	}
	decode("events.get_range", &rangeView)
	if len(rangeView.Events) == 0 {
		t.Fatal("events.get_range recorded no events")
	}

	// The pushed event line decodes as an event, not as a response.
	pushed := byName(t, frames, "events.subscribe.pushed_event")
	event, isEvent, err := DecodeEvent([]byte(pushed.Response))
	if err != nil || !isEvent {
		t.Fatalf("pushed line did not decode as an event: %v", err)
	}
	if event.Cursor == "" || event.Kind == "" || event.CommittedAt == "" {
		t.Fatalf("event frame incomplete: %+v", event)
	}
	// The replay burst's first line is an event frame that PRECEDES the ack on the wire.
	replayFirst := byName(t, frames, "events.subscribe.replay_burst_first_line")
	if _, isEvent, _ := DecodeEvent([]byte(replayFirst.Response)); !isEvent {
		t.Fatalf("expected the replay burst's first line to be an event frame, got: %s", replayFirst.Response)
	}

	// Refusals keep their classes and map to the application's kinds.
	decodeRefusal("case.commit.identity_not_bound", "identity_not_bound", apperr.KindAuthorityDenied, false)
	decodeRefusal("case.commit.identity_delegation_refused", "identity_delegation_refused", apperr.KindAuthorityDenied, false)
	decodeRefusal("case.add_item.unknown_verb_simulated", "", apperr.KindUnavailable, true)
	// The cycle refusal's class is whatever the kernel recorded (plan_cycle_error in
	// case_engine.rs); assert it is a refusal and non-empty rather than duplicating the enum.
	cycle := byName(t, frames, "case.add_item.cycle_refused")
	resp, err := DecodeResponse([]byte(cycle.Response))
	if err != nil {
		t.Fatal(err)
	}
	if resp.Err == nil || resp.Err.Class == "" {
		t.Fatalf("cycle refusal must carry a class: %+v", resp.Err)
	}
	if got := resp.Err.appErr("cycle").Kind; got != apperr.KindInvalid && got != apperr.KindInternal {
		t.Fatalf("cycle refusal kind: %s", got)
	}
	// The empty-reason terminate is an input refusal.
	decodeRefusal("case.terminate.empty_reason_refused", "input_error", apperr.KindInvalid, false)
	// artifact_not_found is a typed not-found input refusal.
	decodeRefusal("artifact.get.not_found", "input_error", apperr.KindInvalid, false)
}
