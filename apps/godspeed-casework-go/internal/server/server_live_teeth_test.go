//go:build live

// T07 LIVE teeth: the same attacks the unit teeth pin, run against the REAL kernel through the
// assembled stack, with the durable proof read off the cell's own records:
//
//   - tooth (a): a valid session cookie without a CSRF token is 403 AND the kernel's request
//     store carries no record for the intent id (zero kernel calls, proven off disk).
//   - tooth (d): user A's session cannot act as user B's kernel actor. The rso session forges
//     the operator actor into an executor intent: the SESSION's standing (R-SO) governs the
//     projection-level role guard, so the intent is refused UNAUTHORIZED_ROLE before any kernel
//     call - a forged operator claim would have passed it. The operator session then executes
//     honestly and the kernel's DURABLE case ledger attributes the mutation to the SESSION's
//     actor, kernel truth rather than gateway claims.
package server_test

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
)

// Live tooth (a): valid cookie, no CSRF header -> 403, and the kernel's durable request store
// has NO record for the refused intent id.
func TestLiveTeethAValidCookieWithoutCSRFReachesNoKernel(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	ts := stack.HTTPServer(t)
	op := stack.Login(t, ts.URL, "operator")

	in := contract.InteractionIntent{
		IntentID:     "gw-t07-tooth-a",
		Kind:         "CONSEQUENTIAL_CASE",
		ActionName:   "EXECUTE_ITEM",
		CaseID:       "case_nonexistent",
		ClientCursor: "01AAA",
		Parameters:   map[string]any{"item_id": "task_prepare"},
	}
	raw, err := json.Marshal(in)
	if err != nil {
		t.Fatal(err)
	}
	req, err := http.NewRequest(http.MethodPost, ts.URL+"/api/intents", bytes.NewReader(raw))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	// NO X-CSRF-Token header: the session cookie alone must prove nothing.
	resp, err := op.Client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("a cookie without CSRF must be 403, got %d body %s", resp.StatusCode, body)
	}

	// Give any would-be kernel call time to exist, then read the kernel's own durable record:
	// the correlation store must have nothing under the refused intent's id.
	time.Sleep(200 * time.Millisecond)
	if rec := cell.RequestRecord(in.IntentID); rec != nil {
		t.Fatalf("the kernel must hold NO request record for a CSRF-refused intent, got %v", rec)
	}
}

// Live tooth (d): a session can never act as another user's kernel actor - the forged payload
// actor is overwritten by the session's standing, proven in both directions (refusal where the
// forged standing would have passed; durable attribution where the session acts).
func TestLiveTeethDSessionCannotActAsAnotherActor(t *testing.T) {
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	ts := stack.HTTPServer(t)
	op := stack.Login(t, ts.URL, "operator")
	rso := stack.Login(t, ts.URL, "rso")

	caseID := stack.CommitSentryChain(t, "gw-t07-tooth-d-commit-1")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	// 1. The R-SO session posts an EXECUTE_ITEM whose payload LIES (claims operator_local, who
	//    holds executor standing; R-SO does not). The session's standing must overwrite the
	//    claim, the role guard must fire on R-SO, and the kernel must never be called.
	in := contract.InteractionIntent{
		IntentID:       "gw-t07-tooth-d-forged",
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     "EXECUTE_ITEM",
		TargetObjectID: "task_prepare",
		CaseID:         caseID,
		ClientCursor:   cursor,
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters:     map[string]any{"item_id": "task_prepare"},
	}
	resp, err := rso.PostJSON("/api/intents", in)
	if err != nil {
		t.Fatal(err)
	}
	var refused contract.IntentResponse
	if err := json.NewDecoder(resp.Body).Decode(&refused); err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if refused.Success || refused.Refusal == nil || refused.Refusal.RefusalKind != "UNAUTHORIZED_ROLE" {
		t.Fatalf("the forged intent must be refused on the SESSION's role (R-SO is offered no execution), got %+v", refused)
	}
	time.Sleep(200 * time.Millisecond)
	if rec := cell.RequestRecord(in.IntentID); rec != nil {
		t.Fatalf("a role-refused intent must never reach the kernel, got record %v", rec)
	}

	// 2. The operator session executes honestly; the kernel's DURABLE ledger must attribute the
	//    activation to the SESSION's actor (operator_local).
	honest := in
	honest.IntentID = "gw-t07-tooth-d-honest"
	resp2, err := op.PostJSON("/api/intents", honest)
	if err != nil {
		t.Fatal(err)
	}
	var accepted contract.IntentResponse
	if err := json.NewDecoder(resp2.Body).Decode(&accepted); err != nil {
		t.Fatal(err)
	}
	resp2.Body.Close()
	if !accepted.Success {
		t.Fatalf("the operator session's honest intent must be admitted, refused: %+v", accepted.Refusal)
	}
	livetest.WaitUntil(t, 15*time.Second, "durable item_activated trace", func() bool {
		return cell.CountTraceKinds(caseID, "item_activated") >= 1
	})
	// 2a. The kernel's request store holds the admitted intent (kernel truth for the mutation).
	if rec := cell.RequestRecord(honest.IntentID); rec == nil {
		t.Fatal("the kernel must hold the durable request record for the admitted intent")
	}
	// 2b. The T02 delegation-audit ledger names the pair: the gateway spoke for the SESSION's
	// actor (operator_local), never for the forged claim. This is the kernel's own durable
	// attribution - not a gateway assertion.
	payload := cell.FindDelegationRecord(honest.IntentID)
	if payload == nil {
		t.Fatal("no delegation-audit record for the admitted intent (the durable pair-principal attribution is missing)")
	}
	if actor, _ := payload["effective_actor_id"].(string); actor != "operator_local" {
		t.Fatalf("the durable delegation record must name the session's actor operator_local, got %v", payload)
	}
	if role, _ := payload["effective_role"].(string); role != "operator" {
		t.Fatalf("the durable delegation record must name the session's role, got %v", payload)
	}
	// The FORGED intent (role-refused before any kernel call) left no delegation record either.
	if rec := cell.FindDelegationRecord(in.IntentID); rec != nil {
		t.Fatalf("a refused intent must have no delegation-audit record, got %v", rec)
	}
}
