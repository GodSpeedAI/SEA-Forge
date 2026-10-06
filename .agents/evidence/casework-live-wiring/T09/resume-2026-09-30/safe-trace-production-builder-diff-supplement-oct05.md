# Safe trace builder exact diff supplement

This immutable supplement preserves the exact full source diff for the frozen
production builder record. The initial `safe-trace-production-builder-oct05.md`
describes the source changes but its fenced diff was abbreviated and is not a
byte-for-byte full diff; this supplement corrects that evidence limitation
without changing the original record or source.

The authorized scope remains the single `run_trace.go` file, implementing
`safe-trace-production-root-assignment-oct05.md` under approved V3 and the
empty-slice erratum. Before hash:
`944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`.
Frozen after hash:
`6abc9572e4ae89570812a0395444aab56b1aab8a36809392a842f8dcf259523b`.
There were no material deviations from the instructions. No compilation,
tests, scanner, Git, hooks, status, or debt command was run.

```diff
--- a/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
+++ b/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
@@ -1,15 +1,232 @@
 package sfwp
 
 import (
+	"bytes"
 	"context"
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
+
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
+
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
+			// Unknown string kinds carry no disclosed fields and cannot invalidate a selected row.
+			if ok {
+				continue
+			}
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
+
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
+
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
+
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
+	return runTraceResponseView{
+		runID: runID, caseID: caseID, planItemID: itemID,
+		execution: execution, settlement: settlement, trace: trace, records: records,
+	}, nil
+}
+
+func hasPresentTraceRecord(raw any) bool {
+	records, ok := raw.([]any)
+	if !ok {
+		return false
+	}
+	count := 0
+	for _, value := range records {
+		entry, ok := value.(map[string]any)
+		if !ok {
+			return false
+		}
+		name, ok := entry["record"].(string)
+		if !ok {
+			return false
+		}
+		if name != "trace.jsonl" {
+			continue
+		}
+		count++
+		present, ok := entry["present"].(bool)
+		if !ok || !present {
+			return false
+		}
+		bytesValue, ok := entry["bytes"].(json.Number)
+		if !ok {
+			return false
+		}
+		if _, err := strconv.ParseUint(string(bytesValue), 10, 64); err != nil {
+			return false
+		}
+	}
+	return count == 1
+}
+
+func commandTraceMetadata(row map[string]any) (*string, *int64, bool) {
+	payload, ok := row["payload"].(map[string]any)
+	if !ok {
+		return nil, nil, true
+	}
+	executionRaw, exists := payload["execution"]
+	if !exists || executionRaw == nil {
+		return nil, nil, true
+	}
+	execution, ok := executionRaw.(map[string]any)
+	if !ok {
+		return nil, nil, false
+	}
+	var status *string
+	if value, exists := execution["status"]; exists && value != nil {
+		text, ok := value.(string)
+		if !ok || !containsRunTraceValue(contract.AllRunTraceCommandExecutionStatuses, text) {
+			return nil, nil, false
+		}
+		status = &text
+	}
+	var exitCode *int64
+	if value, exists := execution["exit_code"]; exists && value != nil {
+		number, ok := value.(json.Number)
+		if !ok {
+			return nil, nil, false
+		}
+		parsed, err := strconv.ParseInt(string(number), 10, 64)
+		if err != nil || parsed < -maxSafeJSONInteger || parsed > maxSafeJSONInteger {
+			return nil, nil, false
+		}
+		exitCode = &parsed
+	}
+	return status, exitCode, true
+}
+
+const maxSafeJSONInteger int64 = 9007199254740991
+
+func containsRunTraceValue(values []string, value string) bool {
+	for _, candidate := range values {
+		if candidate == value {
+			return true
+		}
+	}
+	return false
+}
+
+func runTraceUnavailable() error {
+	return apperr.New(apperr.KindUnavailable, "", "run_trace", "the authority returned an invalid safe run trace projection")
 }
```
