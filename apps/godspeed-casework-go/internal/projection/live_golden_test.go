//go:build live

// The projection builder's live golden test (plan T06): the builder is driven over a REAL
// sea-forge-server on a temp cell through the full live assembly, and the spec-04 snapshot for
// the T03 sentry-chain template case is pinned under testdata/. The pinned assertions are the
// plan's teeth: the operator sees EXECUTE_ITEM on the ready item, and the dependent item shows
// its sentry reason and offers no EXECUTE descriptor.
//
// GOLDEN_UPDATE=1 rewrites the pinned snapshot after an intentional two-sided change.
package projection_test

import (
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestMain(m *testing.M) {
	livetest.EnsureServerBinary()
	os.Exit(m.Run())
}

// normalized returns the snapshot with per-run noise masked so the pinned golden carries exactly
// the shape claims: objects, standings, explanations, actions. Every occurrence of the case id
// (ids, world id, action ids) is masked along with the cursor and timestamp.
func normalized(t *testing.T, raw []byte) []byte {
	t.Helper()
	var snap map[string]any
	if err := json.Unmarshal(raw, &snap); err != nil {
		t.Fatal(err)
	}
	caseID, _ := snap["case_id"].(string)
	snap["case_id"] = "<CASE_ID>"
	snap["world_id"] = "<WORLD_ID>"
	if id, _ := snap["cursor"].(string); id != "" {
		snap["cursor"] = "<KERNEL_CURSOR>"
	}
	snap["timestamp"] = "<TIMESTAMP>"
	maskCaseID(snap, caseID)
	out, err := json.MarshalIndent(snap, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	return out
}

// maskCaseID replaces every occurrence of the run's case id anywhere in the structure.
func maskCaseID(v any, caseID string) {
	switch t := v.(type) {
	case map[string]any:
		for k, val := range t {
			t[k] = maskedValue(val, caseID)
		}
	case []any:
		for i, val := range t {
			t[i] = maskedValue(val, caseID)
		}
	}
}

func maskedValue(v any, caseID string) any {
	switch t := v.(type) {
	case string:
		if caseID != "" && strings.Contains(t, caseID) {
			return strings.ReplaceAll(t, caseID, "<CASE_ID>")
		}
		return t
	case map[string]any:
		maskCaseID(t, caseID)
		return t
	case []any:
		maskCaseID(t, caseID)
		return t
	default:
		return v
	}
}

func TestLiveGoldenSentryChainSnapshot(t *testing.T) {
	cell := livestack.NewCell(t)
	stack := livestack.AssembleStack(t, cell)

	caseID := stack.CommitSentryChain(t, "gw-t06-golden-commit-1")
	stack.WaitRevision(t, caseID, "")

	snap, err := stack.Source.Snapshot(context.Background(), caseID,
		ports.ActorClaim{ActorID: "operator_local", Role: "operator"}, cursorForCase(t, stack, caseID))
	if err != nil {
		t.Fatalf("live snapshot: %v", err)
	}

	raw, err := json.MarshalIndent(snap, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	normalizedRaw := normalized(t, raw)
	goldenPath := filepath.Join("testdata", "livesnapshot-sentry-chain.json")
	if os.Getenv("GOLDEN_UPDATE") != "" {
		if err := os.MkdirAll("testdata", 0o755); err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(goldenPath, normalizedRaw, 0o644); err != nil {
			t.Fatal(err)
		}
		t.Logf("golden rewritten at %s", goldenPath)
	}
	pinned, err := os.ReadFile(goldenPath)
	if err != nil {
		t.Fatalf("the pinned live golden is missing (run with GOLDEN_UPDATE=1 once): %v", err)
	}
	if string(normalizedRaw) != string(pinned) {
		t.Fatalf("the live snapshot drifted from the pinned golden; diff:\n%s", diffFor(t, pinned, normalizedRaw))
	}

	// The plan's teeth, asserted against the live build directly.
	findObj := func(id string) (contract.CognitiveObject, bool) {
		for _, o := range snap.VisibleObjects {
			if o.ID == id {
				return o, true
			}
		}
		return contract.CognitiveObject{}, false
	}
	ready, ok := findObj("task_prepare")
	if !ok {
		t.Fatal("task_prepare missing from the live snapshot")
	}
	if ready.Status != "READY_TO_BEGIN" || !objectOffers(ready, "EXECUTE_ITEM") {
		t.Fatalf("the operator must see EXECUTE_ITEM on the ready item; got %+v", ready)
	}
	dependent, ok := findObj("task_publish")
	if !ok {
		t.Fatal("task_publish missing from the live snapshot")
	}
	if objectOffers(dependent, "EXECUTE_ITEM") {
		t.Fatal("the dependent item must not offer EXECUTE while its sentry has not fired")
	}
	if dependent.Explanation == nil {
		t.Fatal("the dependent item must show its sentry reason")
	}

	// Kernel truth: the horizon's standing came from the durable case ledger.
	if events := cell.CaseEvents(caseID); len(events) < 2 {
		t.Fatalf("the committed case must carry its durable trace events, got %d", len(events))
	}
}

func objectOffers(o contract.CognitiveObject, intent string) bool {
	for _, a := range o.Actions {
		if a.Intent == intent {
			return true
		}
	}
	return false
}

func cursorForCase(t *testing.T, stack *livestack.Stack, caseID string) string {
	t.Helper()
	cursor, ok := stack.Relay.CursorForCase(caseID)
	if !ok {
		t.Fatal("the relay must have observed the commit's kernel frame")
	}
	return cursor
}

func diffFor(t *testing.T, a, b []byte) string {
	t.Helper()
	la := splitLinesFor(string(a))
	lb := splitLinesFor(string(b))
	out := ""
	for i := 0; i < len(la) || i < len(lb); i++ {
		var x, y string
		if i < len(la) {
			x = la[i]
		}
		if i < len(lb) {
			y = lb[i]
		}
		if x != y {
			out += "- " + x + "\n+ " + y + "\n"
		}
	}
	return out
}

func splitLinesFor(s string) []string {
	var out []string
	start := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '\n' {
			out = append(out, s[start:i])
			start = i + 1
		}
	}
	return append(out, s[start:])
}
