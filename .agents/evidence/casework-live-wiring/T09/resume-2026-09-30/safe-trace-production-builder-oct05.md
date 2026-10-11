# Safe trace production builder record

This is a new immutable record for the bounded implementation release. It does
not claim compilation, test, scanner, or independent review approval.

## Original assignment and governing inputs

Root released `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go`
only after accepting the independent behavioral RED in
`safe-trace-fixture-independent-red-e8393465-oct05.md`. The bounded production
instructions are in `safe-trace-production-root-assignment-oct05.md`; the
mandatory empty-slice correction is in
`safe-trace-production-assignment-empty-slice-erratum-oct05.md`. The semantic
requirements come from approved V3
`observation-safe-trace-port-root-proposal-v3.md`, whose independent source
review is `observation-safe-trace-port-independent-review-v3-oct05.md`.

Input hashes:

| Input | SHA-256 |
|---|---|
| Production assignment | `31faeb43d160ffc2bf32f463e771fc17afc4693aff735c3ea6360853c00f0681` |
| Empty-slice erratum | `3e672aaf4c06c166823b394622cb1a8946506439020863080943b26835a87b9d` |
| Approved V3 proposal | `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6` |
| V3 source review | `84b9f657bd51be0215de31dc41cec8c07745a6fe200344bcb80b1881318a04b3` |
| Accepted frozen fixture | `e839346535b3b804326da3f03e2f99df909739528eea591046cbb2f3f5fd522e` |
| Semantic port | `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5` |

## Source identity and result

Only `run_trace.go` was edited. Its pre-edit stub hash was
`944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`; its
frozen implementation hash is
`6abc9572e4ae89570812a0395444aab56b1aab8a36809392a842f8dcf259523b`.
The final fixture hash remains the accepted `e839...522e`; it was not edited.

The implementation rejects blank expected identities before transport,
performs one `Client.Do(NewRunGet(runID))`, preserves returned transport and
authority errors, and converts malformed projections to a fixed privacy-safe
unavailable error with an exactly zero snapshot. It validates response shape,
exact ownership, standings, required single present `trace.jsonl` metadata and
an exact nonnegative `u64` byte value. It omits unknown string kinds before
validating their other fields; validates selected IDs, timestamps, and
duplicates across the full returned array; projects only the ten approved
kinds and command metadata; preserves source order and timestamp text; counts
before retaining at most 1024 newest returned frames. Empty successful Frames
remain nil per the required erratum. No source completeness is inferred.

## Full source diff

```diff
--- a/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
+++ b/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
@@
 import (
 	"context"
+	"bytes"
+	"encoding/json"
+	"io"
+	"strconv"
+	"strings"
+	"time"

 	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
+	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
 	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
 )

 var _ ports.RunTracePort = (*Authority)(nil)

-// ReadRunTrace is a temporary typed-unavailable stub for the test-first assertion phase.
-func (a *Authority) ReadRunTrace(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
-	return ports.RunTraceSnapshot{}, apperr.New(apperr.KindUnavailable, "", "run_trace", "the governed run trace projection is not implemented")
+const runTraceRetention = 1024
+
+// ReadRunTrace returns only allowlisted metadata from one authority run.get response.
+func (a *Authority) ReadRunTrace(ctx context.Context, caseID, runID, planItemID string) (ports.RunTraceSnapshot, error) {
+	if strings.TrimSpace(caseID) == "" || strings.TrimSpace(runID) == "" || strings.TrimSpace(planItemID) == "" {
+		return ports.RunTraceSnapshot{}, apperr.New(apperr.KindInvalid, "", "run_trace", "case, run, and plan item identities must not be blank")
+	}
+	resp, err := a.client.Do(ctx, NewRunGet(runID))
+	if err != nil {
+		return ports.RunTraceSnapshot{}, err
+	}
+	view, err := decodeRunTraceResponse(resp.Raw)
+	if err != nil {
+		return ports.RunTraceSnapshot{}, runTraceUnavailable()
+	}
+	if view.runID != runID || view.caseID != caseID || view.planItemID != planItemID {
+		return ports.RunTraceSnapshot{}, runTraceUnavailable()
+	}
+	if !containsRunTraceValue(contract.AllRunExecutionStandings, view.execution) ||
+		!containsRunTraceValue(contract.AllRunSettlementStandings, view.settlement) {
+		return ports.RunTraceSnapshot{}, runTraceUnavailable()
+	}
+	if !hasPresentTraceRecord(view.records) {
+		return ports.RunTraceSnapshot{}, runTraceUnavailable()
+	}
+	rows, ok := view.trace.([]any)
+	if !ok {
+		return ports.RunTraceSnapshot{}, runTraceUnavailable()
+	}
+	var selected []ports.RunTraceFrame
+	if len(rows) > 0 {
+		selected = make([]ports.RunTraceFrame, 0, len(rows))
+	}
+	seen := make(map[string]struct{}, len(rows))
+	for _, value := range rows {
+		row, ok := value.(map[string]any)
+		if !ok {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+		kind, ok := row["kind"].(string)
+		if !ok || !containsRunTraceValue(contract.AllRunTraceFrameKinds, kind) {
+			if ok {
+				continue
+			}
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+		eventID, ok := row["event_id"].(string)
+		if !ok || strings.TrimSpace(eventID) == "" {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+		timestamp, ok := row["timestamp"].(string)
+		if !ok || strings.TrimSpace(timestamp) == "" {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+		if _, err := time.Parse(time.RFC3339Nano, timestamp); err != nil {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+		if _, exists := seen[eventID]; exists {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+		seen[eventID] = struct{}{}
+		frame := ports.RunTraceFrame{EventID: eventID, Kind: kind, Timestamp: timestamp}
+		if kind == "command_finished" {
+			status, exitCode, valid := commandTraceMetadata(row)
+			if !valid {
+				return ports.RunTraceSnapshot{}, runTraceUnavailable()
+			}
+			frame.ExecutionStatus = status
+			frame.ExitCode = exitCode
+		}
+		selected = append(selected, frame)
+	}
+	total := len(selected)
+	if total > runTraceRetention {
+		selected = selected[total-runTraceRetention:]
+	}
+	return ports.RunTraceSnapshot{
+		RunID: runID, CaseID: caseID, PlanItemID: planItemID,
+		Execution: view.execution, Settlement: view.settlement,
+		Frames: selected, TotalFrameCount: total,
+	}, nil
+}
+
+type runTraceResponseView struct {
+	runID, caseID, planItemID string
+	execution, settlement     string
+	trace, records            any
+}
+
+func decodeRunTraceResponse(raw json.RawMessage) (runTraceResponseView, error) {
+	decoder := json.NewDecoder(bytes.NewReader(raw))
+	decoder.UseNumber()
+	var value any
+	if err := decoder.Decode(&value); err != nil {
+		return runTraceResponseView{}, err
+	}
+	var extra any
+	if err := decoder.Decode(&extra); err != io.EOF {
+		return runTraceResponseView{}, strconv.ErrSyntax
+	}
+	root, ok := value.(map[string]any)
+	if !ok {
+		return runTraceResponseView{}, strconv.ErrSyntax
+	}
+	runID, runOK := root["run_id"].(string)
+	caseID, caseOK := root["case_id"].(string)
+	itemID, itemOK := root["plan_item_id"].(string)
+	execution, executionOK := root["execution"].(string)
+	settlement, settlementOK := root["settlement"].(string)
+	if !runOK || !caseOK || !itemOK || !executionOK || !settlementOK ||
+		strings.TrimSpace(runID) == "" || strings.TrimSpace(caseID) == "" || strings.TrimSpace(itemID) == "" {
+		return runTraceResponseView{}, strconv.ErrSyntax
+	}
+	trace, traceOK := root["trace"]
+	records, recordsOK := root["records"]
+	if !traceOK || !recordsOK {
+		return runTraceResponseView{}, strconv.ErrSyntax
+	}
+	return runTraceResponseView{runID: runID, caseID: caseID, planItemID: itemID, execution: execution, settlement: settlement, trace: trace, records: records}, nil
+}
+
+func hasPresentTraceRecord(raw any) bool {
+	records, ok := raw.([]any)
+	if !ok { return false }
+	count := 0
+	for _, value := range records {
+		entry, ok := value.(map[string]any)
+		if !ok { return false }
+		name, ok := entry["record"].(string)
+		if !ok { return false }
+		if name != "trace.jsonl" { continue }
+		count++
+		present, ok := entry["present"].(bool)
+		if !ok || !present { return false }
+		bytesValue, ok := entry["bytes"].(json.Number)
+		if !ok { return false }
+		if _, err := strconv.ParseUint(string(bytesValue), 10, 64); err != nil { return false }
+	}
+	return count == 1
+}
+
+func commandTraceMetadata(row map[string]any) (*string, *int64, bool) {
+	payload, ok := row["payload"].(map[string]any)
+	if !ok { return nil, nil, true }
+	executionRaw, exists := payload["execution"]
+	if !exists || executionRaw == nil { return nil, nil, true }
+	execution, ok := executionRaw.(map[string]any)
+	if !ok { return nil, nil, false }
+	var status *string
+	if value, exists := execution["status"]; exists && value != nil {
+		text, ok := value.(string)
+		if !ok || !containsRunTraceValue(contract.AllRunTraceCommandExecutionStatuses, text) { return nil, nil, false }
+		status = &text
+	}
+	var exitCode *int64
+	if value, exists := execution["exit_code"]; exists && value != nil {
+		number, ok := value.(json.Number)
+		if !ok { return nil, nil, false }
+		parsed, err := strconv.ParseInt(string(number), 10, 64)
+		if err != nil || parsed < -maxSafeJSONInteger || parsed > maxSafeJSONInteger { return nil, nil, false }
+		exitCode = &parsed
+	}
+	return status, exitCode, true
+}
+
+const maxSafeJSONInteger int64 = 9007199254740991
+
+func containsRunTraceValue(values []string, value string) bool {
+	for _, candidate := range values { if candidate == value { return true } }
+	return false
+}
+
+func runTraceUnavailable() error {
+	return apperr.New(apperr.KindUnavailable, "", "run_trace", "the authority returned an invalid safe run trace projection")
 }
```

## Deviations and limits

No material deviation from the released V3 assignment or erratum is intended.
The implementation intentionally keeps empty successful `Frames` nil. It does
not add tests or alter declarations, artifacts, contracts, server behavior, UI,
or other files. No compiler, test, scanner, Git, hook, status, or debt command
was run. The frozen result awaits the assigned independent source critic.
