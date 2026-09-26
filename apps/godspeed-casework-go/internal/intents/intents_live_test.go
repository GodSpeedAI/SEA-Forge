//go:build live

// The intent translator's live teeth (plan T06) against the REAL kernel on a temp cell:
//
//   - accepted EXECUTE_ITEM -> the kernel's durable case ledger carries the TraceKind;
//   - idempotent replay (same request_id twice) -> no duplicate kernel effect;
//   - an operator-role actor posts APPROVE_HUMAN_TASK -> UNAUTHORIZED_ROLE and the kernel
//     receives NO call (proven against the kernel's own durable correlation store);
//   - an intent with an older cursor -> STALE_PROJECTION and no kernel write (same proof);
//   - a PROPOSE_CASE whose preflight digest does not match the kernel's current preflight ->
//     the kernel refuses honestly (precondition_failed / rejected_as_stale, no case created),
//     surfaced as STALE_PROJECTION per the T01 golden's pinned semantics;
//   - a second handler instance (a gateway restart) replaying the same intent id + body proves
//     the kernel's own request_id dedup as the durable backstop.
package intents_test

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
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

func operatorExecute(id string, itemID, cursor string) contract.InteractionIntent {
	return contract.InteractionIntent{
		IntentID:       id,
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     "EXECUTE_ITEM",
		TargetObjectID: itemID,
		CaseID:         "", // filled by the caller
		ClientCursor:   cursor,
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters:     map[string]any{"item_id": itemID},
	}
}

func refusalKind(t *testing.T, resp contract.IntentResponse) string {
	t.Helper()
	if resp.Success || resp.Refusal == nil {
		t.Fatalf("expected a refusal, got %+v", resp)
	}
	return resp.Refusal.RefusalKind
}

func TestLiveAcceptedExecuteAndIdempotentReplay(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	caseID := stack.CommitSentryChain(t, "gw-t06-intent-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	in := operatorExecute("gw-t06-exec-1", "task_prepare", cursor)
	in.CaseID = caseID
	resp := stack.Dispatcher.Handle(context.Background(), in)
	if !resp.Success {
		t.Fatalf("accepted execute refused: %+v", resp.Refusal)
	}
	if resp.NewCursor == nil || *resp.NewCursor == "" {
		t.Fatal("an accepted mutation must answer with the post-mutation kernel cursor")
	}

	// Kernel truth: exactly one durable item_activated for the executed item.
	if got := cell.CountTraceKinds(caseID, "item_activated"); got != 1 {
		t.Fatalf("durable item_activated count = %d, want 1", got)
	}

	// Idempotent replay: same intent id, byte-identical body -> the recorded outcome stands and
	// the kernel effect does not duplicate.
	replay := stack.Dispatcher.Handle(context.Background(), in)
	if replay != resp {
		t.Fatalf("replay must return the recorded outcome verbatim:\n%+v\n%+v", resp, replay)
	}
	if got := cell.CountTraceKinds(caseID, "item_activated"); got != 1 {
		t.Fatalf("replay must not duplicate the kernel effect, item_activated = %d", got)
	}
}

func TestLiveToothOperatorApproveRefusedWithNoKernelCall(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	caseID := stack.CommitSentryChain(t, "gw-t06-tooth-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	in := operatorExecute("gw-t06-tooth-approve", "task_prepare", cursor)
	in.CaseID = caseID
	in.ActionName = "APPROVE_HUMAN_TASK"
	in.TargetObjectID = "task_prepare"
	in.Parameters = nil
	resp := stack.Dispatcher.Handle(context.Background(), in)
	if kind := refusalKind(t, resp); kind != "UNAUTHORIZED_ROLE" {
		t.Fatalf("operator APPROVE_HUMAN_TASK must refuse UNAUTHORIZED_ROLE, got %s (%+v)", kind, resp)
	}
	// The kernel's own durable correlation store proves it received no call at all.
	if rec := cell.RequestRecord(in.IntentID); rec != nil {
		t.Fatalf("the kernel must have no correlation record for the refused intent, got %v", rec)
	}
	if got := cell.CountTraceKinds(caseID, "item_activated"); got != 0 {
		t.Fatalf("no kernel side effect may exist, item_activated = %d", got)
	}
}

func TestLiveToothStaleCursorRefusedWithNoKernelWrite(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	caseID := stack.CommitSentryChain(t, "gw-t06-stale-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	// A real mutation advances the case, making the first cursor stale.
	in := operatorExecute("gw-t06-stale-exec", "task_prepare", cursor)
	in.CaseID = caseID
	if resp := stack.Dispatcher.Handle(context.Background(), in); !resp.Success {
		t.Fatalf("first execute refused: %+v", resp.Refusal)
	}
	newCursor, _ := stack.Relay.CursorForCase(caseID)
	if newCursor == "" || newCursor == cursor {
		t.Fatalf("the relay must have observed the kernel move: %q", newCursor)
	}

	// The plan's tooth: post an intent carrying the OLDER cursor.
	stale := operatorExecute("gw-t06-stale-second", "task_publish", cursor)
	stale.CaseID = caseID
	resp := stack.Dispatcher.Handle(context.Background(), stale)
	if kind := refusalKind(t, resp); kind != "STALE_PROJECTION" {
		t.Fatalf("an intent with an older cursor must refuse STALE_PROJECTION, got %s (%+v)", kind, resp)
	}
	if resp.Refusal.CurrentCursor == nil || *resp.Refusal.CurrentCursor != newCursor {
		t.Fatalf("the stale refusal must carry the fresh cursor: %+v", resp.Refusal)
	}
	if rec := cell.RequestRecord(stale.IntentID); rec != nil {
		t.Fatalf("a stale intent must produce no kernel write, correlation record: %v", rec)
	}
}

func TestLiveToothPreflightDigestVerifiedByTheKernel(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	before := len(cell.CaseDirs())

	// A digest that cannot match any current template state.
	in := contract.InteractionIntent{
		IntentID:     "gw-t06-digest-wrong",
		Kind:         "CONSEQUENTIAL_CASE",
		ActionName:   "PROPOSE_CASE",
		CaseID:       "case-new",
		ClientCursor: "", // no target case yet: the commit's freshness is the kernel's digest check
		Actor:        contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{
			"template_ref":     "e2e-sentry-chain@0.1.0",
			"preflight_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
			"params":           map[string]any{"dataset_name": "t06"},
		},
	}
	resp := stack.Dispatcher.Handle(context.Background(), in)
	if kind := refusalKind(t, resp); kind != "STALE_PROJECTION" {
		t.Fatalf("a stale preflight digest must refuse honestly, got %s (%+v)", kind, resp)
	}
	// The kernel refused the commit: no case directory may exist beyond the baseline.
	if after := len(cell.CaseDirs()); after != before {
		t.Fatalf("the kernel must not create a case from a stale digest, cases went %d -> %d", before, after)
	}

	// The control: a digest from a REAL passing preflight commits.
	preflight, err := stack.Source.Preflight(context.Background(), "e2e-sentry-chain@0.1.0",
		map[string]any{"dataset_name": "t06"})
	if err != nil || !preflight.Passed || preflight.Digest == nil {
		t.Fatalf("live preflight must pass: %+v %v", preflight, err)
	}
	good := in
	good.IntentID = "gw-t06-digest-right"
	good.Parameters = map[string]any{
		"template_ref":     "e2e-sentry-chain@0.1.0",
		"preflight_digest": *preflight.Digest,
		"params":           map[string]any{"dataset_name": "t06"},
	}
	resp = stack.Dispatcher.Handle(context.Background(), good)
	if !resp.Success {
		t.Fatalf("the passing digest must commit: %+v", resp.Refusal)
	}
	if len(cell.CaseDirs()) != before+1 {
		t.Fatalf("the passing digest must create exactly one case: %d -> %d", before, len(cell.CaseDirs()))
	}
}

func TestLiveKernelDedupsSameRequestIDAcrossGatewayRestart(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	caseID := stack.CommitSentryChain(t, "gw-t06-restart-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	in := operatorExecute("gw-t06-restart-exec-1", "task_prepare", cursor)
	in.CaseID = caseID
	if resp := stack.Dispatcher.Handle(context.Background(), in); !resp.Success {
		t.Fatalf("first execute refused: %+v", resp.Refusal)
	}
	if got := cell.CountTraceKinds(caseID, "item_activated"); got != 1 {
		t.Fatalf("first execute must be durable, item_activated = %d", got)
	}

	// A SECOND handler instance stands in for a restarted gateway: its in-process replay map is
	// empty, so the intent goes back to the kernel with the same request id. The kernel's own
	// correlation store must not duplicate the effect. (The replayed intent carries the CURRENT
	// projection cursor: staleness is a projection-level guard, not what this tooth exercises.)
	restarted := livestack.AssembleStack(t, cell)
	freshCursor := restarted.WaitRevision(t, caseID, "")
	in.ClientCursor = freshCursor
	resp := restarted.Dispatcher.Handle(context.Background(), in)
	if got := cell.CountTraceKinds(caseID, "item_activated"); got != 1 {
		t.Fatalf("the kernel must not duplicate a correlated mutation across restarts, item_activated = %d", got)
	}
	// Whatever the kernel answered (replayed outcome or refusal), it answered ONCE durably.
	t.Logf("restarted-gateway replay outcome: success=%t refusal=%v", resp.Success, resp.Refusal)
}

// T06 fix F2, live: ADD_DISCRETIONARY_WORK must produce an ACCEPTED kernel write through the
// served surface. The gateway constructs the governed write_file item (kernel refuses
// operation-less proposals), the plan mutates durably, and executing the new item settles
// accepted on the materialized file (kernel commit 514fc72's write_file path).
func TestLiveDiscretionaryAddProducesAnAcceptedKernelWrite(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	caseID := stack.CommitSentryChain(t, "gw-t06-disc-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	add := contract.InteractionIntent{
		IntentID:     "gw-t06-disc-add-1",
		Kind:         "CONSEQUENTIAL_CASE",
		ActionName:   "ADD_DISCRETIONARY_WORK",
		CaseID:       caseID,
		ClientCursor: cursor,
		Actor:        contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{
			"case_id":       caseID,
			"stage_id":      "task_prepare",
			"kind":          "sandboxed_task",
			"title":         "Augment dataset notes",
			"summary":       "Record why the discretionary augmentation was needed.",
			"justification": "The prepared dataset lacks provenance notes; the reviewer asked for them.",
		},
	}
	resp := stack.Dispatcher.Handle(context.Background(), add)
	if !resp.Success {
		t.Fatalf("discretionary add refused: %+v", resp.Refusal)
	}
	// Kernel truth: the plan mutated durably, exactly once for this intent.
	if got := cell.CountTraceKinds(caseID, "plan_mutated"); got != 1 {
		t.Fatalf("durable plan_mutated count = %d, want 1", got)
	}
	if resp.NewCursor == nil || *resp.NewCursor == "" || *resp.NewCursor == cursor {
		t.Fatalf("accepted add must answer with a NEWER kernel cursor, got %+v", resp.NewCursor)
	}
	if resp.ResultingObject == nil || resp.ResultingObject.ID == "" {
		t.Fatalf("accepted add must surface the proposed object, got %+v", resp.ResultingObject)
	}

	// The next snapshot offers EXECUTE_ITEM for the new item (it depends on task_prepare, which
	// is still pending here, so instead execute task_prepare first and re-check the offer).
	executeFirst := operatorExecute("gw-t06-disc-exec-prepare-1", "task_prepare", *resp.NewCursor)
	executeFirst.CaseID = caseID
	if out := stack.Dispatcher.Handle(context.Background(), executeFirst); !out.Success {
		t.Fatalf("task_prepare execute refused: %+v", out.Refusal)
	}
	stack.WaitRevision(t, caseID, *resp.NewCursor)
	cursorAfterPrepare, _ := stack.Relay.CursorForCase(caseID)

	execNew := contract.InteractionIntent{
		IntentID:       "gw-t06-disc-exec-1",
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     "EXECUTE_ITEM",
		TargetObjectID: resp.ResultingObject.ID,
		CaseID:         caseID,
		ClientCursor:   cursorAfterPrepare,
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters:     map[string]any{"item_id": resp.ResultingObject.ID},
	}
	out := stack.Dispatcher.Handle(context.Background(), execNew)
	if !out.Success {
		t.Fatalf("discretionary item execute refused: %+v", out.Refusal)
	}

	// Kernel truth: the discretionary episode settled ACCEPTED — the item_completed event's
	// own payload records the settlement status — with the honest write_only basis (the
	// governed write materialized and was captured as an artifact; no command ran).
	livetest.WaitUntil(t, 15*time.Second, "durable item_completed for the discretionary item", func() bool {
		for _, ev := range cell.CaseEvents(caseID) {
			kind, _ := ev["kind"].(string)
			item, _ := ev["plan_item_id"].(string)
			if kind == "item_completed" && item == resp.ResultingObject.ID {
				payload, _ := ev["payload"].(map[string]any)
				settlement, _ := payload["settlement"].(string)
				return settlement == "accepted"
			}
		}
		return false
	})
	writeOnlyBasis := false
	basisFiles, _ := filepath.Glob(filepath.Join(cell.Root(), "cases", caseID, "runs", "*", "settlement.json"))
	for _, f := range basisFiles {
		raw, err := os.ReadFile(f)
		if err != nil {
			t.Fatal(err)
		}
		var doc struct {
			Status string   `json:"status"`
			Basis  []string `json:"basis"`
		}
		if err := json.Unmarshal(raw, &doc); err != nil {
			t.Fatal(err)
		}
		if doc.Status == "accepted" && containsString(doc.Basis, "write_only") {
			writeOnlyBasis = true
		}
	}
	if len(basisFiles) == 0 {
		t.Fatal("expected settlement.json under the case runs for every dispatched episode")
	}
	if !writeOnlyBasis {
		t.Fatal("no accepted settlement in the case carries the honest write_only basis")
	}
}

func containsString(list []string, want string) bool {
	for _, v := range list {
		if v == want {
			return true
		}
	}
	return false
}
