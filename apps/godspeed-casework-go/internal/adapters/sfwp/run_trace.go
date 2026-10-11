package sfwp

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"strconv"
	"strings"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

var _ ports.RunTracePort = (*Authority)(nil)

const runTraceRetention = 1024

func hasASCIIDigits(value string) bool {
	for i := range value {
		if value[i] < '0' || value[i] > '9' {
			return false
		}
	}
	return true
}

func isRFC3339NanoSyntax(value string) bool {
	if len(value) < len("2006-01-02T15:04:05Z") ||
		value[4] != '-' || value[7] != '-' || value[10] != 'T' ||
		value[13] != ':' || value[16] != ':' ||
		!hasASCIIDigits(value[:4]) || !hasASCIIDigits(value[5:7]) ||
		!hasASCIIDigits(value[8:10]) || !hasASCIIDigits(value[11:13]) ||
		!hasASCIIDigits(value[14:16]) || !hasASCIIDigits(value[17:19]) {
		return false
	}

	zoneStart := 19
	if zoneStart < len(value) && value[zoneStart] == '.' {
		fractionStart := zoneStart + 1
		zoneStart = fractionStart
		for zoneStart < len(value) && value[zoneStart] >= '0' && value[zoneStart] <= '9' {
			zoneStart++
		}
		if zoneStart == fractionStart {
			return false
		}
	}

	if zoneStart == len(value)-1 && value[zoneStart] == 'Z' {
		return true
	}
	if len(value)-zoneStart != 6 || (value[zoneStart] != '+' && value[zoneStart] != '-') ||
		value[zoneStart+3] != ':' || !hasASCIIDigits(value[zoneStart+1:zoneStart+3]) ||
		!hasASCIIDigits(value[zoneStart+4:zoneStart+6]) {
		return false
	}
	offsetHour := int(value[zoneStart+1]-'0')*10 + int(value[zoneStart+2]-'0')
	offsetMinute := int(value[zoneStart+4]-'0')*10 + int(value[zoneStart+5]-'0')
	return offsetHour < 24 && offsetMinute < 60
}

// ReadRunTrace returns only allowlisted metadata from one authority run.get response.
func (a *Authority) ReadRunTrace(ctx context.Context, caseID, runID, planItemID string) (ports.RunTraceSnapshot, error) {
	if strings.TrimSpace(caseID) == "" || strings.TrimSpace(runID) == "" || strings.TrimSpace(planItemID) == "" {
		return ports.RunTraceSnapshot{}, apperr.New(apperr.KindInvalid, "", "run_trace", "case, run, and plan item identities must not be blank")
	}

	resp, err := a.client.Do(ctx, NewRunGet(runID))
	if err != nil {
		var refusal *Refusal
		if errors.As(err, &refusal) {
			return ports.RunTraceSnapshot{}, err
		}
		var appError *apperr.Error
		if errors.As(err, &appError) && appError.Op == "decode" {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		return ports.RunTraceSnapshot{}, err
	}
	view, err := decodeRunTraceResponse(resp.Raw)
	if err != nil {
		return ports.RunTraceSnapshot{}, runTraceUnavailable()
	}
	if view.runID != runID || view.caseID != caseID || view.planItemID != planItemID {
		return ports.RunTraceSnapshot{}, runTraceUnavailable()
	}
	if !containsRunTraceValue(contract.AllRunExecutionStandings, view.execution) ||
		!containsRunTraceValue(contract.AllRunSettlementStandings, view.settlement) {
		return ports.RunTraceSnapshot{}, runTraceUnavailable()
	}
	if !hasPresentTraceRecord(view.records) {
		return ports.RunTraceSnapshot{}, runTraceUnavailable()
	}

	rows, ok := view.trace.([]any)
	if !ok {
		return ports.RunTraceSnapshot{}, runTraceUnavailable()
	}
	var selected []ports.RunTraceFrame
	if len(rows) > 0 {
		capacity := len(rows)
		if capacity > runTraceRetention {
			capacity = runTraceRetention
		}
		selected = make([]ports.RunTraceFrame, 0, capacity)
	}
	retainedStart := 0
	total := 0
	seen := make(map[string]struct{}, len(rows))
	for _, value := range rows {
		row, ok := value.(map[string]any)
		if !ok {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		kind, ok := row["kind"].(string)
		if !ok || !containsRunTraceValue(contract.AllRunTraceFrameKinds, kind) {
			// Unknown string kinds carry no disclosed fields and cannot invalidate a selected row.
			if ok {
				continue
			}
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}

		eventID, ok := row["event_id"].(string)
		if !ok || strings.TrimSpace(eventID) == "" {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		timestamp, ok := row["timestamp"].(string)
		if !ok || strings.TrimSpace(timestamp) == "" {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		// Go's RFC3339 parser falls back to a more permissive layout parser.
		if !isRFC3339NanoSyntax(timestamp) {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		if _, err := time.Parse(time.RFC3339Nano, timestamp); err != nil {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		if _, exists := seen[eventID]; exists {
			return ports.RunTraceSnapshot{}, runTraceUnavailable()
		}
		seen[eventID] = struct{}{}

		frame := ports.RunTraceFrame{EventID: eventID, Kind: kind, Timestamp: timestamp}
		if kind == "command_finished" {
			status, exitCode, valid := commandTraceMetadata(row)
			if !valid {
				return ports.RunTraceSnapshot{}, runTraceUnavailable()
			}
			frame.ExecutionStatus = status
			frame.ExitCode = exitCode
		}
		if len(selected) < runTraceRetention {
			selected = append(selected, frame)
		} else {
			selected[retainedStart] = frame
			retainedStart = (retainedStart + 1) % runTraceRetention
		}
		total++
	}

	if total > runTraceRetention {
		retained := make([]ports.RunTraceFrame, runTraceRetention)
		copy(retained, selected[retainedStart:])
		copy(retained[len(selected)-retainedStart:], selected[:retainedStart])
		selected = retained
	}
	return ports.RunTraceSnapshot{
		RunID: runID, CaseID: caseID, PlanItemID: planItemID,
		Execution: view.execution, Settlement: view.settlement,
		Frames: selected, TotalFrameCount: total,
	}, nil
}

type runTraceResponseView struct {
	runID, caseID, planItemID string
	execution, settlement     string
	trace, records            any
}

func decodeRunTraceResponse(raw json.RawMessage) (runTraceResponseView, error) {
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return runTraceResponseView{}, err
	}
	var extra any
	if err := decoder.Decode(&extra); err != io.EOF {
		return runTraceResponseView{}, strconv.ErrSyntax
	}
	root, ok := value.(map[string]any)
	if !ok {
		return runTraceResponseView{}, strconv.ErrSyntax
	}
	runID, runOK := root["run_id"].(string)
	caseID, caseOK := root["case_id"].(string)
	itemID, itemOK := root["plan_item_id"].(string)
	execution, executionOK := root["execution"].(string)
	settlement, settlementOK := root["settlement"].(string)
	if !runOK || !caseOK || !itemOK || !executionOK || !settlementOK ||
		strings.TrimSpace(runID) == "" || strings.TrimSpace(caseID) == "" || strings.TrimSpace(itemID) == "" {
		return runTraceResponseView{}, strconv.ErrSyntax
	}
	trace, traceOK := root["trace"]
	records, recordsOK := root["records"]
	if !traceOK || !recordsOK {
		return runTraceResponseView{}, strconv.ErrSyntax
	}
	return runTraceResponseView{
		runID: runID, caseID: caseID, planItemID: itemID,
		execution: execution, settlement: settlement, trace: trace, records: records,
	}, nil
}

func hasPresentTraceRecord(raw any) bool {
	records, ok := raw.([]any)
	if !ok {
		return false
	}
	count := 0
	for _, value := range records {
		entry, ok := value.(map[string]any)
		if !ok {
			return false
		}
		name, ok := entry["record"].(string)
		if !ok {
			return false
		}
		if name != "trace.jsonl" {
			continue
		}
		count++
		present, ok := entry["present"].(bool)
		if !ok || !present {
			return false
		}
		bytesValue, ok := entry["bytes"].(json.Number)
		if !ok {
			return false
		}
		if _, err := strconv.ParseUint(string(bytesValue), 10, 64); err != nil {
			return false
		}
	}
	return count == 1
}

func commandTraceMetadata(row map[string]any) (*string, *int64, bool) {
	payload, ok := row["payload"].(map[string]any)
	if !ok {
		return nil, nil, true
	}
	executionRaw, exists := payload["execution"]
	if !exists || executionRaw == nil {
		return nil, nil, true
	}
	execution, ok := executionRaw.(map[string]any)
	if !ok {
		return nil, nil, false
	}
	var status *string
	if value, exists := execution["status"]; exists && value != nil {
		text, ok := value.(string)
		if !ok || !containsRunTraceValue(contract.AllRunTraceCommandExecutionStatuses, text) {
			return nil, nil, false
		}
		status = &text
	}
	var exitCode *int64
	if value, exists := execution["exit_code"]; exists && value != nil {
		number, ok := value.(json.Number)
		if !ok {
			return nil, nil, false
		}
		parsed, err := strconv.ParseInt(string(number), 10, 64)
		if err != nil || parsed < -maxSafeJSONInteger || parsed > maxSafeJSONInteger {
			return nil, nil, false
		}
		exitCode = &parsed
	}
	return status, exitCode, true
}

const maxSafeJSONInteger int64 = 9007199254740991

func containsRunTraceValue(values []string, value string) bool {
	for _, candidate := range values {
		if candidate == value {
			return true
		}
	}
	return false
}

func runTraceUnavailable() error {
	return apperr.New(apperr.KindUnavailable, "", "run_trace", "the authority returned an invalid safe run trace projection")
}
