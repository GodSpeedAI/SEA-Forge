package sfwp

import (
	"context"
	"encoding/json"
	"errors"
	"net"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

type scopedRunLister interface {
	RunsListForCase(context.Context, ports.CaseRef) (ports.RunListResult, error)
}

func scopedLister(t *testing.T, auth *Authority) scopedRunLister {
	t.Helper()
	lister, ok := any(auth).(scopedRunLister)
	if !ok {
		t.Fatal("Authority does not implement scoped RunsListForCase")
	}
	return lister
}

func TestScopedRunsListUsesCaseRequestAndMapsEveryField(t *testing.T) {
	started := "2026-09-30T12:13:14Z"
	finished := "2026-09-30T12:14:15Z"
	fs := newFakeServer(t, func(line string) (string, bool) {
		var request struct {
			Verb   string `json:"verb"`
			CaseID string `json:"case_id"`
		}
		if err := json.Unmarshal([]byte(line), &request); err != nil {
			t.Errorf("decode scoped request: %v", err)
		}
		if request.Verb != "run_list" || request.CaseID != "case_actual" {
			t.Errorf("request verb/case_id = %q/%q, want run_list/case_actual", request.Verb, request.CaseID)
		}
		return `{"runs":[{"run_id":"run_actual","case_id":"case_actual","plan_item_id":"item_actual","execution":"completed","settlement":"accepted","started_at":"` + started + `","finished_at":"` + finished + `","evidence_count":7}],"unreadable":["run_unreadable"]}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	result, err := scopedLister(t, NewAuthority(client)).RunsListForCase(context.Background(), ports.CaseRef("case_actual"))
	if err != nil {
		t.Fatal(err)
	}
	if fs.requestCount("run_list") != 1 {
		t.Fatalf("run_list requests = %d, want exactly one", fs.requestCount("run_list"))
	}
	wantStart, _ := time.Parse(time.RFC3339, started)
	wantFinish, _ := time.Parse(time.RFC3339, finished)
	want := ports.RunSummary{RunID: "run_actual", CaseID: "case_actual", PlanItemID: "item_actual", Execution: "completed", Settlement: "accepted", StartedAt: wantStart, HasStarted: true, FinishedAt: wantFinish, HasFinished: true, EvidenceCount: 7}
	if len(result.Runs) != 1 || result.Runs[0] != want {
		t.Errorf("mapped runs = %+v, want %+v", result.Runs, []ports.RunSummary{want})
	}
	if len(result.UnreadableIDs) != 1 || result.UnreadableIDs[0] != "run_unreadable" {
		t.Errorf("unreadable IDs = %v, want [run_unreadable]", result.UnreadableIDs)
	}
}

func TestLegacyRunListRequestRemainsUnscoped(t *testing.T) {
	encoded, err := EncodeRequest(NewRunList())
	if err != nil {
		t.Fatal(err)
	}
	if string(encoded) != `{"verb":"run_list"}` {
		t.Fatalf("legacy request = %s, want exactly an unscoped run_list", encoded)
	}
}

func TestScopedRunsListRejectsBlankCaseBeforeWireContact(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) { return `{"runs":[],"unreadable":[]}`, false })
	var dialAttempts atomic.Int32
	config := testConfig(fs.socket)
	config.Dial = func(context.Context, string) (net.Conn, error) {
		dialAttempts.Add(1)
		return nil, errors.New("unexpected dial for invalid case ref")
	}
	client, err := New(config)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	lister := scopedLister(t, NewAuthority(client))
	for _, caseRef := range []ports.CaseRef{"", " \t\n"} {
		_, err := lister.RunsListForCase(context.Background(), caseRef)
		if err == nil || apperr.KindOf(err) != apperr.KindInvalid {
			t.Fatalf("blank case ref %q error = %v, want typed invalid", caseRef, err)
		}
	}
	if got := fs.requestCount("run_list"); got != 0 {
		t.Fatalf("blank case refs caused %d wire requests, want zero", got)
	}
	if got := dialAttempts.Load(); got != 0 {
		t.Fatalf("blank case refs caused %d dial attempts, want zero", got)
	}
	if got := fs.conns.Load(); got != 0 {
		t.Fatalf("blank case refs caused %d accepted connections, want zero", got)
	}
	fs.mu.Lock()
	requestLines := len(fs.requests)
	fs.mu.Unlock()
	if requestLines != 0 {
		t.Fatalf("blank case refs caused %d request lines, want zero", requestLines)
	}
}

func TestScopedRunsListRejectsMalformedAuthorityRows(t *testing.T) {
	valid := `{"run_id":"run_1","case_id":"case_1","plan_item_id":"item_1","execution":"completed","settlement":"accepted","evidence_count":0}`
	cases := []struct {
		name  string
		body  string
		count int
	}{
		{"missing-runs", `{"unreadable":[]}`, 0},
		{"null-runs", `{"runs":null,"unreadable":[]}`, 0},
		{"missing-unreadable", `{"runs":[]}`, 0},
		{"null-unreadable", `{"runs":[],"unreadable":null}`, 0},
		{"blank-run-id", `{"runs":[{"run_id":" ","case_id":"case_1","plan_item_id":"item_1","execution":"completed","settlement":"accepted"}],"unreadable":[]}`, 0},
		{"missing-case", `{"runs":[{"run_id":"run_1","plan_item_id":"item_1","execution":"completed","settlement":"accepted"}],"unreadable":[]}`, 0},
		{"foreign-case", `{"runs":[{"run_id":"run_1","case_id":"case_other","plan_item_id":"item_1","execution":"completed","settlement":"accepted"}],"unreadable":[]}`, 0},
		{"missing-item", `{"runs":[{"run_id":"run_1","case_id":"case_1","execution":"completed","settlement":"accepted"}],"unreadable":[]}`, 0},
		{"unknown-execution", `{"runs":[{"run_id":"run_1","case_id":"case_1","plan_item_id":"item_1","execution":"mystery","settlement":"accepted"}],"unreadable":[]}`, 0},
		{"unknown-settlement", `{"runs":[{"run_id":"run_1","case_id":"case_1","plan_item_id":"item_1","execution":"completed","settlement":"mystery"}],"unreadable":[]}`, 0},
		{"bad-timestamp", `{"runs":[{"run_id":"run_1","case_id":"case_1","plan_item_id":"item_1","execution":"completed","settlement":"accepted","started_at":"yesterday"}],"unreadable":[]}`, 0},
		{"negative-evidence", `{"runs":[{"run_id":"run_1","case_id":"case_1","plan_item_id":"item_1","execution":"completed","settlement":"accepted","evidence_count":-1}],"unreadable":[]}`, 0},
		{"duplicate-run-id", `{"runs":[` + valid + `,` + valid + `],"unreadable":[]}`, 0},
		{"duplicate-unreadable", `{"runs":[],"unreadable":["run_x","run_x"]}`, 0},
		{"overlap", `{"runs":[` + valid + `],"unreadable":["run_1"]}`, 0},
		{"blank-unreadable", `{"runs":[],"unreadable":[" "]}`, 0},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			fs := newFakeServer(t, func(string) (string, bool) { return tc.body, false })
			client, err := New(testConfig(fs.socket))
			if err != nil {
				t.Fatal(err)
			}
			defer client.Close()
			_, err = scopedLister(t, NewAuthority(client)).RunsListForCase(context.Background(), ports.CaseRef("case_1"))
			if err == nil {
				t.Fatal("malformed or foreign run list was accepted")
			}
			if got := apperr.KindOf(err); got != apperr.KindUnavailable {
				t.Fatalf("error kind = %s, want unavailable: %v", got, err)
			}
			if fs.requestCount("run_list") != 1 {
				t.Fatalf("run_list requests = %d, want one", fs.requestCount("run_list"))
			}
		})
	}
}

func TestScopedRunsListRefusalRemainsTyped(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) {
		return `{"error":"run list refused","error_class":"unavailable"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	_, err = scopedLister(t, NewAuthority(client)).RunsListForCase(context.Background(), ports.CaseRef("case_1"))
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("refusal error = %v, want typed unavailable", err)
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) {
		t.Fatalf("typed refusal was lost: %v", err)
	}
	if got := refusal.RefusalClass(); got != "unavailable" {
		t.Fatalf("preserved refusal class = %q, want unavailable", got)
	}
}

func TestScopedRunsListPreservesOtherRefusalKinds(t *testing.T) {
	cases := []struct {
		name  string
		class string
		want  apperr.Kind
	}{
		{name: "invalid input", class: "input_error", want: apperr.KindInvalid},
		{name: "identity denial", class: "identity_required", want: apperr.KindAuthorityDenied},
		{name: "unsupported version", class: "unsupported_version", want: apperr.KindUnavailable},
		{name: "unknown refusal", class: "unknown_refusal", want: apperr.KindInternal},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			fs := newFakeServer(t, func(string) (string, bool) {
				return `{"error":"run list refused","error_class":"` + tc.class + `"}`, false
			})
			client, err := New(testConfig(fs.socket))
			if err != nil {
				t.Fatal(err)
			}
			defer client.Close()
			_, err = scopedLister(t, NewAuthority(client)).RunsListForCase(context.Background(), ports.CaseRef("case_1"))
			if got := apperr.KindOf(err); got != tc.want {
				t.Fatalf("refusal class %q kind = %s, want %s: %v", tc.class, got, tc.want, err)
			}
			var refusal *Refusal
			if !errors.As(err, &refusal) {
				t.Fatalf("refusal class %q was not preserved through errors.As: %v", tc.class, err)
			}
			if got := refusal.RefusalClass(); got != tc.class {
				t.Fatalf("preserved refusal class = %q, want %q", got, tc.class)
			}
			if fs.requestCount("run_list") != 1 {
				t.Fatalf("run_list requests = %d, want one", fs.requestCount("run_list"))
			}
		})
	}
}
