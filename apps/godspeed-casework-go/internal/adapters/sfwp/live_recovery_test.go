//go:build live

// TOOTH (b), permanent: kill the server during an in-flight case.commit, restart it, and prove
// the client resolves the outcome through request.get_status without double-committing.
package sfwp

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

func TestLiveKillMidCommitRecovery(t *testing.T) {
	cell := newLiveCell(t)
	cl := cell.client()

	// A first, uneventful commit proves the cell works and gives the kill a warm server.
	first := commitCase(context.Background(), t, cl, cl.NewRequestID("case_commit"))
	t.Logf("warm-up case %s committed", first.CaseID)

	requestID := cl.NewRequestID("case_commit")
	type result struct {
		view CommitView
		err  error
	}
	done := make(chan result, 1)
	go func() {
		req, err := NewCaseCommit(sentryChainRef, sentryChainParams(), policyRef, "sfwp_live_tests", 60,
			requestID, nil, operatorGov())
		if err != nil {
			done <- result{err: err}
			return
		}
		resp, err := cl.Do(context.Background(), req)
		if err != nil {
			done <- result{err: err}
			return
		}
		var view CommitView
		if err := resp.Into(&view); err != nil {
			done <- result{err: err}
			return
		}
		done <- result{view: view}
	}()

	// Kill the server the moment the request is durably admitted (its pending correlation record
	// appears under <root>/requests/), i.e. strictly while it is in flight.
	waitSeen(t, 10*time.Second, "the commit's pending correlation record", func() bool {
		return cell.requestRecord(requestID) != nil
	})
	cell.kill()
	t.Log("server killed mid-commit; restarting")

	// Let the client hit the dead socket and start its bounded recovery backoff, then revive.
	time.Sleep(200 * time.Millisecond)
	cell.start()

	select {
	case res := <-done:
		record := cell.requestRecord(requestID)
		if record == nil {
			t.Fatal("the correlation record disappeared")
		}
		switch {
		case res.err == nil:
			// The commit landed before the kill: the outcome is the recorded one.
			if res.view.CaseID == "" {
				t.Fatalf("recovered commit has no case id: %+v", res.view)
			}
			if got := record["status"]; got != "completed" {
				t.Fatalf("client reported success but the record says %v", got)
			}
			assertOutcomeCase(t, record, res.view.CaseID)
			t.Logf("commit completed before the kill; outcome recovered for case %s", res.view.CaseID)
		default:
			// The kill interrupted the admitted work: the client must surface the authority's own
			// interrupted-failure record, not invent an outcome and not hang.
			var refusal *Refusal
			if !errors.As(res.err, &refusal) {
				t.Fatalf("interrupted commit must surface the recorded refusal, got %v", res.err)
			}
			if refusal.Class != "request_interrupted" {
				t.Fatalf("expected the request_interrupted record, got class %q (%v)", refusal.Class, res.err)
			}
			if got := apperr.KindOf(res.err); got == "" {
				t.Fatal("interrupted outcome must be typed")
			}
			if got := record["status"]; got != "failed" {
				t.Fatalf("the durable record should be failed/interrupted, says %v", got)
			}
			t.Logf("commit interrupted by the kill; outcome resolved as request_interrupted")
		}

		// The no-double-commit proof, from durable truth alone:
		// 1. exactly one correlation record file for this request id;
		entries, _ := os.ReadDir(filepath.Join(cell.root, "requests"))
		same := 0
		for _, e := range entries {
			if e.Name() == requestID+".json" {
				same++
			}
		}
		if same != 1 {
			t.Fatalf("expected exactly 1 correlation record for %s, found %d", requestID, same)
		}
		// 2. at most one CaseCreated per committed outcome: this request committed once or not at
		// all - never twice. The warm-up case accounts for exactly one of the cell's creations.
		created := cell.countCaseCreated()
		if created != 1 && created != 2 {
			t.Fatalf("cell shows %d CaseCreated events; the killed commit must add at most one", created)
		}
		completed := record["status"] == "completed"
		if completed && created != 2 {
			t.Fatalf("a completed commit must have appended exactly one CaseCreated (cell has %d)", created)
		}
		if !completed && created != 1 {
			t.Fatalf("an interrupted commit must have appended no CaseCreated (cell has %d)", created)
		}
		t.Logf("durable truth: %d CaseCreated in the cell, record status=%v - no double commit", created, record["status"])

	case <-time.After(60 * time.Second):
		t.Fatal("the client never resolved the interrupted commit's outcome")
	}

	// After recovery the server and client keep working together (the pool discarded its dead
	// connections and re-dialed; any connection that died with the old process resolves through
	// the restart contract - see commitCaseResolvingRestart).
	after, err := commitCaseResolvingRestart(context.Background(), t, cl)
	if err != nil {
		t.Fatalf("post-recovery case.commit failed: %v", err)
	}
	t.Logf("post-recovery case %s committed", after.CaseID)
}

// assertOutcomeCase checks the correlation record's recorded outcome names the recovered case.
func assertOutcomeCase(t *testing.T, record map[string]any, caseID string) {
	t.Helper()
	raw, err := json.Marshal(record["outcome"])
	if err != nil {
		t.Fatal(err)
	}
	var outcome struct {
		CaseID string `json:"case_id"`
	}
	if err := json.Unmarshal(raw, &outcome); err != nil {
		t.Fatal(err)
	}
	if outcome.CaseID != caseID {
		t.Fatalf("recorded outcome names case %q, client recovered %q", outcome.CaseID, caseID)
	}
}
