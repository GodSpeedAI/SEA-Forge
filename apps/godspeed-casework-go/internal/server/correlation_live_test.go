//go:build live

// T11 log correlation, proven across all three layers against the REAL kernel: one
// correlation_id (the intent id the browser sent) appears in
//
//	(1) the gateway's logfmt request line and the X-Correlation-Id response header,
//	(2) the kernel's durable request record (<cell>/requests/<request_id>.json), and
//	(3) the kernel's delegation-audit ledger entry (request_id + both principals).
package server_test

import (
	"bytes"
	"log"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/metrics"
)

type syncBuf struct {
	mu sync.Mutex
	b  bytes.Buffer
}

func (s *syncBuf) Write(p []byte) (int, error) { s.mu.Lock(); defer s.mu.Unlock(); return s.b.Write(p) }
func (s *syncBuf) String() string              { s.mu.Lock(); defer s.mu.Unlock(); return s.b.String() }

func TestLiveCorrelationIDTracesBrowserGatewayKernelRequestAndLedger(t *testing.T) {
	cell := livetest.NewCell(t)
	logs := &syncBuf{}
	reg := metrics.New()
	stack := livestack.AssembleStackObserved(t, cell, log.New(logs, "", 0), reg)
	ts := stack.HTTPServer(t)
	op := stack.Login(t, ts.URL, "operator")

	caseID := stack.CommitSentryChain(t, "gw-t11-corr-commit")
	stack.WaitRevision(t, caseID, "")
	cursor, _ := stack.Relay.CursorForCase(caseID)

	const corr = "gw-t11-corr-trace-1"
	resp, err := op.PostJSON("/api/intents", contract.InteractionIntent{
		IntentID:       corr,
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     "EXECUTE_ITEM",
		TargetObjectID: "task_prepare",
		CaseID:         caseID,
		ClientCursor:   cursor,
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters:     map[string]any{"item_id": "task_prepare"},
	})
	if err != nil {
		t.Fatal(err)
	}
	resp.Body.Close()
	if got := resp.Header.Get("X-Correlation-Id"); got != corr {
		t.Fatalf("layer 1 (response header): X-Correlation-Id = %q, want %q", got, corr)
	}
	// (1) gateway log line.
	if !strings.Contains(logs.String(), `correlation_id="`+corr+`" method="POST" path="/api/intents" status=200`) {
		t.Fatalf("layer 1 (gateway log) lacks the correlation line:\n%s", logs.String())
	}
	// (2) kernel request record; (3) kernel delegation-audit ledger.
	livetest.WaitUntil(t, 15*time.Second, "kernel request record and delegation-audit entry", func() bool {
		return cell.RequestRecord(corr) != nil && cell.FindDelegationRecord(corr) != nil
	})
	audit := cell.FindDelegationRecord(corr)
	if audit["request_id"] != corr || audit["gateway_actor_id"] != "gateway" || audit["effective_actor_id"] != "operator_local" {
		t.Fatalf("layer 3 (delegation audit) = %v", audit)
	}
	t.Logf("correlation_id %s traced: gateway log + header, requests/%s.json, delegation-audit (verb %v)", corr, corr, audit["verb"])

	// The same traffic is visible in the metrics, with no identifying labels.
	var sb strings.Builder
	reg.Write(&sb)
	rec := sb.String()
	if !strings.Contains(rec, `route="POST /api/intents",status_class="2xx"`) {
		t.Fatalf("metrics missing the intent request:\n%s", rec)
	}
	if strings.Contains(rec, corr) || strings.Contains(rec, "operator_local") {
		t.Fatalf("metrics must not leak ids or actors:\n%s", rec)
	}
}
