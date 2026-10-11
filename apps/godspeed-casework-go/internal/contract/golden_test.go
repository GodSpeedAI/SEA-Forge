package contract

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"testing"
)

// The golden fixtures are the shared source of truth between the TypeScript contract
// (.agents/reports/interface-contracts/typescript/types.ts, exercised by the UI's
// src/ports/wireContract.test.ts) and these Go structs. Every file under the golden directory
// must be exercised here: an unrecognized file fails the walk, and a byte-level round-trip
// mismatch fails the test, so a json-tag rename on either side cannot drift silently.
//
// Setting GOLDEN_UPDATE=1 rewrites the fixtures in this package's canonical encoding instead of
// comparing (standard golden-test affordance; use it only after an intentional contract change
// that the TypeScript side has already agreed to).
//
// Cache caveat: `go test` caches on package sources, NOT on files read at runtime. After a
// fixture-only change, run this package with -count=1 or the cached result may mask the change
// (the T01 gates and teeth transcripts use -count=1 for this reason).

// goldenDir resolves .agents/reports/interface-contracts/golden relative to the module root
// (the directory holding go.mod, found by walking up from this test's working directory).
func goldenDir(t *testing.T) string {
	t.Helper()
	wd, err := os.Getwd()
	if err != nil {
		t.Fatalf("getwd: %v", err)
	}
	root := ""
	for dir := wd; ; dir = filepath.Dir(dir) {
		if _, err := os.Stat(filepath.Join(dir, "go.mod")); err == nil {
			root = dir
			break
		}
		if parent := filepath.Dir(dir); parent == dir {
			t.Fatalf("go.mod not found above %s", wd)
		}
	}
	return filepath.Join(root, "..", "..", ".agents", "reports", "interface-contracts", "golden")
}

// canonicalIndent encodes v the way the .json fixtures are written (two-space indent, trailing
// newline, no HTML escaping).
func canonicalIndent(v any) ([]byte, error) {
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	enc.SetIndent("", "  ")
	if err := enc.Encode(v); err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// canonicalCompact encodes v as one jsonl line (no newline; the caller owns line endings).
func canonicalCompact(v any) (string, error) {
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(v); err != nil {
		return "", err
	}
	return strings.TrimRight(buf.String(), "\n"), nil
}

func goldenUpdate() bool { return os.Getenv("GOLDEN_UPDATE") == "1" }

func rewriteIfChanged(t *testing.T, path string, want []byte) {
	t.Helper()
	if err := os.WriteFile(path, want, 0o644); err != nil {
		t.Fatalf("GOLDEN_UPDATE: rewrite %s: %v", path, err)
	}
}

// requiredFixtures must all exist; deleting one fails the walk instead of silently narrowing
// coverage.
var requiredFixtures = []string{
	"world-snapshot.json",
	"templates-entry-options.json",
	"template-preflight-pass.json",
	"template-preflight-fail.json",
	"ask-request.json",
	"ask-answer-answered.json",
	"ask-answer-partial.json",
	"ask-answer-denied.json",
	"sse-events.jsonl",
}

func TestGoldenRoundTrip(t *testing.T) {
	dir := goldenDir(t)
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("read golden dir: %v", err)
	}
	seen := map[string]bool{}
	for _, e := range entries {
		name := e.Name()
		if e.IsDir() {
			t.Errorf("unexpected directory in golden/: %s", name)
			continue
		}
		seen[name] = true
		path := filepath.Join(dir, name)
		switch {
		case name == "world-snapshot.json":
			roundTrip(t, path, &CognitiveWorldSnapshot{})
		case strings.HasPrefix(name, "intent-") && strings.HasSuffix(name, ".json"):
			var fx IntentFixture
			if roundTrip(t, path, &fx) {
				checkIntentFixture(t, name, &fx)
			}
		case name == "templates-entry-options.json":
			roundTrip(t, path, &[]TemplateEntryOption{})
		case strings.HasPrefix(name, "template-preflight-") && strings.HasSuffix(name, ".json"):
			roundTrip(t, path, &TemplatePreflightResult{})
		case name == "ask-request.json":
			var request ThothAskRequest
			if roundTrip(t, path, &request) {
				checkThothAskRequest(t, name, &request)
			}
		case strings.HasPrefix(name, "ask-answer-") && strings.HasSuffix(name, ".json"):
			var answer ThothAnswerView
			if roundTrip(t, path, &answer) {
				checkThothAnswer(t, name, &answer)
			}
		case name == "sse-events.jsonl":
			roundTripLines(t, path)
		default:
			t.Errorf("unrecognized golden fixture %q: every file under golden/ must map to a contract struct", name)
		}
	}
	for _, req := range requiredFixtures {
		if !seen[req] {
			t.Errorf("required golden fixture missing: %s", req)
		}
	}
}

func checkThothAskRequest(t *testing.T, name string, req *ThothAskRequest) {
	t.Helper()
	if !contains(AllThothQuestionKinds, string(req.Kind)) {
		t.Errorf("%s: kind %q is not a canonical Thoth question kind", name, req.Kind)
	}
	if strings.TrimSpace(req.Subject) == "" {
		t.Errorf("%s: subject is empty", name)
	}
}

func checkThothAnswer(t *testing.T, name string, answer *ThothAnswerView) {
	t.Helper()
	if !contains(AllThothDispositions, string(answer.Disposition)) {
		t.Errorf("%s: disposition %q is not canonical", name, answer.Disposition)
	}
	if !contains(AllThothFreshnessValues, string(answer.Freshness)) {
		t.Errorf("%s: freshness %q is not canonical", name, answer.Freshness)
	}
	if answer.AnswerID == "" || answer.QuestionID == "" || answer.SnapshotRef == "" || answer.AnsweredAt == "" {
		t.Errorf("%s: answer is missing identity, snapshot, or answered_at", name)
	}
	if answer.Assurance == "" || answer.AuthorityNotice == "" {
		t.Errorf("%s: assurance and authority_notice remain required strings", name)
	}
	if answer.Claims == nil || answer.OmittedClaimClasses == nil || answer.Limitations == nil {
		t.Errorf("%s: claims, omitted_claim_classes, and limitations must be arrays", name)
	}
	for _, class := range answer.OmittedClaimClasses {
		if !contains(AllThothClaimClasses, string(class)) {
			t.Errorf("%s: omitted claim class %q is not canonical", name, class)
		}
	}
	for i, claim := range answer.Claims {
		if claim.ClaimID == "" || claim.Subject == "" || claim.Statement == "" || claim.SnapshotRef == "" {
			t.Errorf("%s: claim %d is missing required disclosure fields", name, i)
		}
		if !contains(AllThothClaimClasses, string(claim.ClaimClass)) {
			t.Errorf("%s: claim %d class %q is not canonical", name, i, claim.ClaimClass)
		}
		if !contains(AllThothClaimStatuses, string(claim.Status)) {
			t.Errorf("%s: claim %d status %q is not canonical", name, i, claim.Status)
		}
		if claim.EvidenceRefs == nil || claim.SettlementRefs == nil {
			t.Errorf("%s: claim %d evidence_refs and settlement_refs must be arrays", name, i)
		}
	}
}

func checkRunTraceObservationEvent(t *testing.T, path string, line int, ev *RunTraceObservationEvent) {
	t.Helper()
	if ev.EventType != RunTraceObservationEventType {
		t.Errorf("%s line %d: event_type %q, want %q", path, line, ev.EventType, RunTraceObservationEventType)
	}
	if ev.Cursor == "" || ev.Timestamp == "" {
		t.Errorf("%s line %d: observation event is missing its informational case cursor or timestamp", path, line)
	}
	obs := ev.Payload
	if obs.CaseID == "" || obs.ObservedAt == "" {
		t.Errorf("%s line %d: observation payload is missing case_id or observed_at", path, line)
	}
	if !contains(AllRunTraceListStates, string(obs.RunListState)) || !contains(AllRunTraceObservationStates, string(obs.ObservationState)) {
		t.Errorf("%s line %d: unrecognized run-list or cohort observation state", path, line)
	}
	if obs.HydrationReadBudget.Limit != 8 || obs.HydrationReadBudget.ReadsAttempted < 0 || obs.HydrationReadBudget.ReadsAttempted > 8 {
		t.Errorf("%s line %d: invalid hydration read budget %+v", path, line, obs.HydrationReadBudget)
	}
	if len(obs.Runs) > 8 {
		t.Errorf("%s line %d: cohort carries %d runs, maximum is 8", path, line, len(obs.Runs))
	}
	if obs.Runs == nil {
		t.Errorf("%s line %d: runs must be an array", path, line)
	}
	counts := []*int{obs.ListedRunCount, obs.SelectedRunCount, obs.ValidatedRunCount, obs.UnreadableRunCount, obs.UnavailableRunCount, obs.OmittedRunCount}
	if obs.RunListState == "unavailable" {
		for _, count := range counts {
			if count != nil {
				t.Errorf("%s line %d: unavailable run.list must leave every optional count absent", path, line)
				break
			}
		}
		if len(obs.Runs) != 0 || obs.ObservationState != "unavailable" {
			t.Errorf("%s line %d: failed run.list must not report a hydrated cohort", path, line)
		}
	} else {
		for _, count := range counts {
			if count == nil {
				t.Errorf("%s line %d: completed run.list must report every cohort count (including zero)", path, line)
				break
			}
		}
	}
	for i, run := range obs.Runs {
		ctx := fmt.Sprintf("%s line %d run %d", path, line, i)
		if run.RunID == "" || run.ObservedAt == "" || run.PlanItemID == "" {
			t.Errorf("%s: missing run identity/parent metadata", ctx)
		}
		if !contains(AllRunExecutionStandings, string(run.Execution)) || !contains(AllRunSettlementStandings, string(run.Settlement)) {
			t.Errorf("%s: execution and settlement standings must use their separate vocabularies", ctx)
		}
		if !contains(AllRunTraceRunObservationStates, string(run.ObservationState)) {
			t.Errorf("%s: unrecognized nested observation state %q", ctx, run.ObservationState)
		}
		if run.TotalFrameCount < 0 || run.RetainedFrameCount != len(run.Frames) || run.OmittedFrameCount < 0 || run.TotalFrameCount != run.RetainedFrameCount+run.OmittedFrameCount || run.Truncated != (run.OmittedFrameCount > 0) {
			t.Errorf("%s: frame counts/truncated flag do not describe the retained frames", ctx)
		}
		if run.Frames == nil {
			t.Errorf("%s: frames must be an array", ctx)
		}
		if len(run.Frames) > 1024 {
			t.Errorf("%s: carries %d frames, maximum is 1024", ctx, len(run.Frames))
		}
		for j, frame := range run.Frames {
			if frame.EventID == "" || frame.Timestamp == "" || !contains(AllRunTraceFrameKinds, string(frame.Kind)) {
				t.Errorf("%s frame %d: missing identity/time or unrecognized safe kind", ctx, j)
			}
			if frame.ExecutionStatus != nil && !contains(AllRunTraceCommandExecutionStatuses, string(*frame.ExecutionStatus)) {
				t.Errorf("%s frame %d: unrecognized command execution status %q", ctx, j, *frame.ExecutionStatus)
			}
			if frame.Kind != "command_finished" && (frame.ExecutionStatus != nil || frame.ExitCode != nil) {
				t.Errorf("%s frame %d: command metadata is allowed only on command_finished", ctx, j)
			}
		}
	}
}

func TestCanonicalMirrorFieldSets(t *testing.T) {
	assertFields := func(name string, typ reflect.Type, want, optional []string) {
		t.Helper()
		var got, gotOptional []string
		for i := 0; i < typ.NumField(); i++ {
			parts := strings.Split(typ.Field(i).Tag.Get("json"), ",")
			got = append(got, parts[0])
			if len(parts) > 1 && contains(parts[1:], "omitempty") {
				gotOptional = append(gotOptional, parts[0])
			}
		}
		sort.Strings(got)
		sort.Strings(gotOptional)
		sort.Strings(want)
		sort.Strings(optional)
		if !reflect.DeepEqual(got, want) {
			t.Errorf("%s JSON fields = %v, want exactly %v", name, got, want)
		}
		if !reflect.DeepEqual(gotOptional, optional) {
			t.Errorf("%s optional JSON fields = %v, want exactly %v", name, gotOptional, optional)
		}
	}
	assertFields("ThothAskRequest", reflect.TypeOf(ThothAskRequest{}), []string{"kind", "subject", "purpose", "case"}, []string{"purpose", "case"})
	assertFields("ThothClaimView", reflect.TypeOf(ThothClaimView{}), []string{"claim_id", "claim_class", "subject", "status", "statement", "snapshot_ref", "evidence_refs", "settlement_refs", "capability_record_ref"}, []string{"capability_record_ref"})
	assertFields("ThothAnswerView", reflect.TypeOf(ThothAnswerView{}), []string{"answer_id", "question_id", "disposition", "claims", "omitted_claim_classes", "snapshot_ref", "freshness", "assurance", "limitations", "authority_notice", "answered_at"}, nil)
	assertFields("RunTraceObservationEvent", reflect.TypeOf(RunTraceObservationEvent{}), []string{"event_type", "cursor", "timestamp", "payload"}, nil)
	assertFields("RunTraceFrame", reflect.TypeOf(RunTraceFrame{}), []string{"event_id", "kind", "timestamp", "execution_status", "exit_code"}, []string{"execution_status", "exit_code"})
	assertFields("RunTraceRunObservation", reflect.TypeOf(RunTraceRunObservation{}), []string{"run_id", "observed_at", "plan_item_id", "execution", "settlement", "observation_state", "frames", "total_frame_count", "retained_frame_count", "omitted_frame_count", "truncated"}, nil)
	assertFields("RunTraceHydrationReadBudget", reflect.TypeOf(RunTraceHydrationReadBudget{}), []string{"limit", "reads_attempted", "exhausted"}, nil)
	assertFields("RunTraceObservation", reflect.TypeOf(RunTraceObservation{}), []string{"case_id", "observed_at", "run_list_state", "observation_state", "listed_run_count", "selected_run_count", "validated_run_count", "unreadable_run_count", "unavailable_run_count", "omitted_run_count", "hydration_read_budget", "runs"}, []string{"listed_run_count", "selected_run_count", "validated_run_count", "unreadable_run_count", "unavailable_run_count", "omitted_run_count"})
}

func TestCanonicalMirrorFieldTypes(t *testing.T) {
	type fieldTypePin struct {
		name string
		typ  reflect.Type
	}
	assertTypes := func(name string, typ reflect.Type, fields []fieldTypePin) {
		t.Helper()
		for _, field := range fields {
			got, ok := typ.FieldByName(field.name)
			if !ok {
				t.Errorf("%s is missing Go field %s", name, field.name)
				continue
			}
			if got.Type != field.typ {
				t.Errorf("%s.%s Go type = %v, want %v", name, field.name, got.Type, field.typ)
			}
		}
	}

	stringType := reflect.TypeOf("")
	stringSliceType := reflect.TypeOf([]string{})
	intType := reflect.TypeOf(0)
	int64PointerType := reflect.TypeOf((*int64)(nil))
	stringPointerType := reflect.TypeOf((*string)(nil))
	intPointerType := reflect.TypeOf((*int)(nil))

	assertTypes("ThothAskRequest", reflect.TypeOf(ThothAskRequest{}), []fieldTypePin{
		{"Kind", reflect.TypeOf(ThothQuestionKind(""))}, {"Subject", stringType},
		{"Purpose", stringPointerType}, {"Case", stringPointerType},
	})
	assertTypes("ThothClaimView", reflect.TypeOf(ThothClaimView{}), []fieldTypePin{
		{"ClaimID", stringType}, {"ClaimClass", reflect.TypeOf(ThothClaimClass(""))},
		{"Subject", stringType}, {"Status", reflect.TypeOf(ThothClaimStatus(""))},
		{"Statement", stringType}, {"SnapshotRef", stringType},
		{"EvidenceRefs", stringSliceType}, {"SettlementRefs", stringSliceType},
		{"CapabilityRecordRef", stringPointerType},
	})
	assertTypes("ThothAnswerView", reflect.TypeOf(ThothAnswerView{}), []fieldTypePin{
		{"AnswerID", stringType}, {"QuestionID", stringType},
		{"Disposition", reflect.TypeOf(ThothDisposition(""))},
		{"Claims", reflect.TypeOf([]ThothClaimView{})},
		{"OmittedClaimClasses", reflect.TypeOf([]ThothClaimClass{})},
		{"SnapshotRef", stringType}, {"Freshness", reflect.TypeOf(ThothFreshness(""))},
		{"Assurance", stringType}, {"Limitations", stringSliceType},
		{"AuthorityNotice", stringType}, {"AnsweredAt", stringType},
	})
	assertTypes("RunTraceObservationEvent", reflect.TypeOf(RunTraceObservationEvent{}), []fieldTypePin{
		{"EventType", stringType}, {"Cursor", stringType}, {"Timestamp", stringType},
		{"Payload", reflect.TypeOf(RunTraceObservation{})},
	})
	assertTypes("RunTraceObservation", reflect.TypeOf(RunTraceObservation{}), []fieldTypePin{
		{"CaseID", stringType}, {"ObservedAt", stringType},
		{"RunListState", reflect.TypeOf(RunTraceListState(""))},
		{"ObservationState", reflect.TypeOf(RunTraceObservationState(""))},
		{"ListedRunCount", intPointerType}, {"SelectedRunCount", intPointerType},
		{"ValidatedRunCount", intPointerType}, {"UnreadableRunCount", intPointerType},
		{"UnavailableRunCount", intPointerType}, {"OmittedRunCount", intPointerType},
		{"HydrationReadBudget", reflect.TypeOf(RunTraceHydrationReadBudget{})},
		{"Runs", reflect.TypeOf([]RunTraceRunObservation{})},
	})
	assertTypes("RunTraceRunObservation", reflect.TypeOf(RunTraceRunObservation{}), []fieldTypePin{
		{"RunID", stringType}, {"ObservedAt", stringType}, {"PlanItemID", stringType},
		{"Execution", reflect.TypeOf(RunExecutionStanding(""))},
		{"Settlement", reflect.TypeOf(RunSettlementStanding(""))},
		{"ObservationState", reflect.TypeOf(RunTraceRunObservationState(""))},
		{"Frames", reflect.TypeOf([]RunTraceFrame{})},
		{"TotalFrameCount", intType}, {"RetainedFrameCount", intType},
		{"OmittedFrameCount", intType}, {"Truncated", reflect.TypeOf(false)},
	})
	assertTypes("RunTraceHydrationReadBudget", reflect.TypeOf(RunTraceHydrationReadBudget{}), []fieldTypePin{
		{"Limit", intType}, {"ReadsAttempted", intType}, {"Exhausted", reflect.TypeOf(false)},
	})
	assertTypes("RunTraceFrame", reflect.TypeOf(RunTraceFrame{}), []fieldTypePin{
		{"EventID", stringType}, {"Kind", reflect.TypeOf(RunTraceFrameKind(""))},
		{"Timestamp", stringType},
		{"ExecutionStatus", reflect.TypeOf((*RunTraceCommandExecutionStatus)(nil))},
		{"ExitCode", int64PointerType},
	})
}

func TestCanonicalMirrorVocabularies(t *testing.T) {
	assertVocabulary := func(name string, got, want []string) {
		t.Helper()
		if !reflect.DeepEqual(got, want) {
			t.Errorf("%s = %v, want canonical vocabulary %v", name, got, want)
		}
		seen := map[string]bool{}
		for _, value := range got {
			if seen[value] {
				t.Errorf("%s contains duplicate %q", name, value)
			}
			seen[value] = true
		}
	}
	assertVocabulary("AllStreamEventKinds", AllStreamEventKinds, []string{"snapshot", "patch", "execution_progress", "settlement_recorded", "lease_expired", "resync_required", "interrupted", "error", "heartbeat", "execution_observation"})
	assertVocabulary("AllRunTraceFrameKinds", AllRunTraceFrameKinds, []string{"run_started", "item_activated", "command_started", "command_finished", "item_completed", "item_failed", "item_terminated", "human_task_completed", "run_halted", "run_finished"})
	assertVocabulary("AllRunTraceCommandExecutionStatuses", AllRunTraceCommandExecutionStatuses, []string{"completed", "spawn_failed", "timed_out", "sandbox_violation", "suspected_sandbox_violation"})
	assertVocabulary("AllRunExecutionStandings", AllRunExecutionStandings, []string{"pending", "enabled", "active", "completed", "failed", "terminated"})
	assertVocabulary("AllRunSettlementStandings", AllRunSettlementStandings, []string{"unsettled", "accepted", "rejected", "escalated"})
	assertVocabulary("AllRunTraceRunObservationStates", AllRunTraceRunObservationStates, []string{"validated", "unavailable"})
	assertVocabulary("AllRunTraceListStates", AllRunTraceListStates, []string{"complete", "unavailable"})
	assertVocabulary("AllRunTraceObservationStates", AllRunTraceObservationStates, []string{"complete", "no_runs", "capacity_limited", "unavailable"})
	assertVocabulary("AllThothQuestionKinds", AllThothQuestionKinds, []string{"ask_capability", "ask_operation_requirements", "ask_authority_requirements", "ask_projection_support", "ask_environment_status", "ask_failure_explanation", "ask_evidence_for_claim", "ask_available_affordances", "ask_why_denied"})
	assertVocabulary("AllThothClaimClasses", AllThothClaimClasses, []string{"identity", "architecture", "declared_capability", "installed_capability", "demonstrated_capability", "authority_requirements", "environment_status", "failure_condition", "security_implementation", "customer_private", "credential_bearing", "policy_thresholds"})
	assertVocabulary("AllThothClaimStatuses", AllThothClaimStatuses, []string{"unknown", "unsupported", "declared", "installed", "available", "validated", "demonstrated"})
	assertVocabulary("AllThothDispositions", AllThothDispositions, []string{"answered", "partial", "denied"})
	assertVocabulary("AllThothFreshnessValues", AllThothFreshnessValues, []string{"current", "stale"})
}

func TestRunTraceObservationCountOmission(t *testing.T) {
	observationType := reflect.TypeOf(RunTraceObservation{})
	for _, field := range []string{"ListedRunCount", "SelectedRunCount", "ValidatedRunCount", "UnreadableRunCount", "UnavailableRunCount", "OmittedRunCount"} {
		fieldType, ok := observationType.FieldByName(field)
		if !ok || fieldType.Type.Kind() != reflect.Pointer || fieldType.Type.Elem().Kind() != reflect.Int {
			t.Errorf("%s must be *int to preserve absent versus present zero", field)
		}
	}
	var observation RunTraceObservation
	raw, err := json.Marshal(observation)
	if err != nil {
		t.Fatal(err)
	}
	var absent map[string]any
	if err := json.Unmarshal(raw, &absent); err != nil {
		t.Fatal(err)
	}
	countFields := []string{"listed_run_count", "selected_run_count", "validated_run_count", "unreadable_run_count", "unavailable_run_count", "omitted_run_count"}
	for _, field := range countFields {
		if _, exists := absent[field]; exists {
			t.Errorf("nil %s must remain absent", field)
		}
	}
	zero := 0
	observation.ListedRunCount = &zero
	observation.SelectedRunCount = &zero
	observation.ValidatedRunCount = &zero
	observation.UnreadableRunCount = &zero
	observation.UnavailableRunCount = &zero
	observation.OmittedRunCount = &zero
	raw, err = json.Marshal(observation)
	if err != nil {
		t.Fatal(err)
	}
	var present map[string]any
	if err := json.Unmarshal(raw, &present); err != nil {
		t.Fatal(err)
	}
	for _, field := range countFields {
		if value, exists := present[field]; !exists || value != float64(0) {
			t.Errorf("present zero %s must serialize as zero; got %#v", field, value)
		}
	}
}

func TestThothAskRequestOptionalPresence(t *testing.T) {
	request := ThothAskRequest{Kind: "ask_capability", Subject: "shell"}
	raw, err := json.Marshal(request)
	if err != nil {
		t.Fatal(err)
	}
	var absent map[string]any
	if err := json.Unmarshal(raw, &absent); err != nil {
		t.Fatal(err)
	}
	for _, field := range []string{"purpose", "case"} {
		if _, exists := absent[field]; exists {
			t.Errorf("omitted request %s must remain absent", field)
		}
	}
	empty := ""
	request.Purpose = &empty
	request.Case = &empty
	raw, err = json.Marshal(request)
	if err != nil {
		t.Fatal(err)
	}
	var present map[string]any
	if err := json.Unmarshal(raw, &present); err != nil {
		t.Fatal(err)
	}
	for _, field := range []string{"purpose", "case"} {
		if value, exists := present[field]; !exists || value != "" {
			t.Errorf("present empty %s must remain distinguishable from omission", field)
		}
	}
}

func TestRunTraceFrameCommandMetadataPresence(t *testing.T) {
	frame := RunTraceFrame{EventID: "evt-1", Kind: "command_finished", Timestamp: "2026-09-30T00:00:00Z"}
	raw, err := json.Marshal(frame)
	if err != nil {
		t.Fatal(err)
	}
	var absent map[string]any
	if err := json.Unmarshal(raw, &absent); err != nil {
		t.Fatal(err)
	}
	if _, exists := absent["execution_status"]; exists {
		t.Error("execution_status must remain absent when not recorded")
	}
	if _, exists := absent["exit_code"]; exists {
		t.Error("exit_code must remain absent when not recorded")
	}
	status := RunTraceCommandExecutionStatus("completed")
	exitCode := int64(0)
	frame.ExecutionStatus = &status
	frame.ExitCode = &exitCode
	raw, err = json.Marshal(frame)
	if err != nil {
		t.Fatal(err)
	}
	var present map[string]any
	if err := json.Unmarshal(raw, &present); err != nil {
		t.Fatal(err)
	}
	if value, exists := present["execution_status"]; !exists || value != "completed" {
		t.Errorf("recorded execution_status must be serialized exactly; got %#v", value)
	}
	if value, exists := present["exit_code"]; !exists || value != float64(0) {
		t.Errorf("recorded zero exit_code must not collapse to absence; got %#v", value)
	}
}

// roundTrip unmarshals the file into a fresh value of the pointed-to type and requires that the
// canonical re-marshal reproduces the file byte for byte. Reports pass/fail (the fixture is only
// parsed further when the round-trip is intact).
func roundTrip(t *testing.T, path string, v any) bool {
	t.Helper()
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Errorf("read %s: %v", path, err)
		return false
	}
	if err := json.Unmarshal(raw, v); err != nil {
		t.Errorf("unmarshal %s into %T: %v", path, v, err)
		return false
	}
	canonical, err := canonicalIndent(v)
	if err != nil {
		t.Errorf("re-marshal %s: %v", path, err)
		return false
	}
	if !bytes.Equal(raw, canonical) {
		if goldenUpdate() {
			rewriteIfChanged(t, path, canonical)
			return true
		}
		t.Errorf("%s: round-trip is not byte-stable; the Go json tags and the fixture disagree.\n--- fixture ---\n%s\n--- re-marshaled ---\n%s",
			filepath.Base(path), raw, canonical)
		return false
	}
	return true
}

// roundTripLines does the per-line equivalent for the SSE jsonl fixture. Observation events are
// decoded into their complete typed shape, then object key order is normalized for comparison.
// Under GOLDEN_UPDATE it rewrites the file in canonical line encoding instead of comparing.
func roundTripLines(t *testing.T, path string) {
	t.Helper()
	raw, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read %s: %v", path, err)
	}
	lines := strings.Split(strings.TrimRight(string(raw), "\n"), "\n")
	canonicalLines := make([]string, 0, len(lines))
	for i, line := range lines {
		if strings.TrimSpace(line) == "" {
			t.Fatalf("%s: blank line at %d", path, i+1)
		}
		var header struct {
			EventType string `json:"event_type"`
		}
		if err := json.Unmarshal([]byte(line), &header); err != nil {
			t.Errorf("%s line %d: read event type: %v", path, i+1, err)
			continue
		}
		if header.EventType == RunTraceObservationEventType {
			var ev RunTraceObservationEvent
			dec := json.NewDecoder(strings.NewReader(line))
			dec.DisallowUnknownFields()
			if err := dec.Decode(&ev); err != nil {
				t.Errorf("%s line %d: observation does not match its typed event: %v", path, i+1, err)
				continue
			}
			var trailing any
			if err := dec.Decode(&trailing); err != io.EOF {
				t.Errorf("%s line %d: observation line has trailing JSON", path, i+1)
				continue
			}
			canonical, err := canonicalCompact(&ev)
			if err != nil {
				t.Errorf("%s line %d: re-marshal typed observation: %v", path, i+1, err)
				continue
			}
			if goldenUpdate() {
				canonicalLines = append(canonicalLines, canonical)
			} else if !sameCanonicalJSON([]byte(line), []byte(canonical)) {
				t.Errorf("%s line %d: typed observation differs from the fixture after JSON object-key normalization", path, i+1)
			}
			checkRunTraceObservationEvent(t, path, i+1, &ev)
			continue
		}
		var ev StreamEvent
		if err := json.Unmarshal([]byte(line), &ev); err != nil {
			t.Errorf("%s line %d: unmarshal: %v", path, i+1, err)
			continue
		}
		canonical, err := canonicalCompact(&ev)
		if err != nil {
			t.Errorf("%s line %d: re-marshal: %v", path, i+1, err)
			continue
		}
		if goldenUpdate() {
			canonicalLines = append(canonicalLines, canonical)
		} else if canonical != line {
			t.Errorf("%s line %d: round-trip is not byte-stable.\n--- fixture ---\n%s\n--- re-marshaled ---\n%s",
				path, i+1, line, canonical)
		}
		checkStreamEventPayload(t, path, i+1, &ev)
	}
	if goldenUpdate() && len(canonicalLines) == len(lines) {
		rewriteIfChanged(t, path, []byte(strings.Join(canonicalLines, "\n")+"\n"))
	}
}

// sameCanonicalJSON ignores object-key order while retaining array order and scalar values. The
// shared JSONL fixture is authored in canonical interface order; Go's map encoder sorts object
// keys, so raw byte order is not a contract field.
func sameCanonicalJSON(left, right []byte) bool {
	canonical := func(raw []byte) ([]byte, error) {
		var value any
		dec := json.NewDecoder(bytes.NewReader(raw))
		dec.UseNumber()
		if err := dec.Decode(&value); err != nil {
			return nil, err
		}
		return json.Marshal(value)
	}
	a, err := canonical(left)
	if err != nil {
		return false
	}
	b, err := canonical(right)
	return err == nil && bytes.Equal(a, b)
}

// checkStreamEventPayload validates the typed payload of each SSE line against the struct that
// mirrors its TypeScript payload type.
func checkStreamEventPayload(t *testing.T, path string, line int, ev *StreamEvent) {
	t.Helper()
	payload, err := json.Marshal(ev.Payload)
	if err != nil {
		t.Errorf("%s line %d: re-encode payload: %v", path, line, err)
		return
	}
	decode := func(v any) {
		if err := json.Unmarshal(payload, v); err != nil {
			t.Errorf("%s line %d: payload does not match %T: %v", path, line, v, err)
		}
	}
	switch ev.EventType {
	case "snapshot":
		var snap CognitiveWorldSnapshot
		decode(&snap)
		if snap.WorldID == "" || len(snap.VisibleObjects) == 0 {
			t.Errorf("%s line %d: snapshot payload is missing world_id or visible_objects", path, line)
		}
	case "patch":
		var patch map[string]any
		decode(&patch)
		if len(patch) == 0 {
			t.Errorf("%s line %d: patch payload must be a non-empty object", path, line)
		}
	case "execution_progress":
		var p ExecutionProgressPayload
		decode(&p)
		if p.RunID == "" || p.Phase == "" {
			t.Errorf("%s line %d: execution_progress payload missing run_id or phase", path, line)
		}
	case "settlement_recorded":
		var s OperationalSettlement
		decode(&s)
		if s.SettlementID == "" || s.Decision == "" {
			t.Errorf("%s line %d: settlement_recorded payload missing settlement_id or decision", path, line)
		}
	case "lease_expired":
		var l ExecutionLease
		decode(&l)
		if l.LeaseID == "" {
			t.Errorf("%s line %d: lease_expired payload missing lease_id", path, line)
		}
	case "resync_required":
		var p ResyncRequiredPayload
		decode(&p)
		if p.RequestedCursor == "" || p.OldestAvailableCursor == "" || p.Reason == "" {
			t.Errorf("%s line %d: resync_required payload missing cursor fields or reason", path, line)
		}
	case "interrupted":
		var p InterruptedPayload
		decode(&p)
		if p.Reason == "" {
			t.Errorf("%s line %d: interrupted payload missing reason", path, line)
		}
	case "error":
		var p ErrorPayload
		decode(&p)
		if p.ErrorCode == "" || p.Message == "" {
			t.Errorf("%s line %d: error payload missing error_code or message", path, line)
		}
	case RunTraceObservationEventType:
		var p RunTraceObservation
		decode(&p)
		if p.CaseID == "" || p.ObservedAt == "" {
			t.Errorf("%s line %d: execution_observation payload missing case_id or observed_at", path, line)
		}
	case "heartbeat":
		var h map[string]any
		decode(&h)
		if len(h) == 0 {
			t.Errorf("%s line %d: heartbeat payload must be a non-empty object", path, line)
		}
	default:
		t.Errorf("%s line %d: event_type %q is not in AllStreamEventKinds", path, line, ev.EventType)
	}
}

// checkIntentFixture enforces the well-formedness rules of an intent golden: envelope fields,
// typed payloads for payload-carrying kinds, envelope-only parameters otherwise, and responses
// that are either successes or typed refusals.
func checkIntentFixture(t *testing.T, name string, fx *IntentFixture) {
	t.Helper()
	req := fx.Request
	if req.IntentID == "" || req.ActionName == "" || req.CaseID == "" || req.ClientCursor == "" {
		t.Errorf("%s: request missing envelope fields (intent_id, action_name, case_id, client_cursor)", name)
	}
	if req.Kind != "CONSEQUENTIAL_CASE" {
		t.Errorf("%s: request kind is %q, want CONSEQUENTIAL_CASE", name, req.Kind)
	}
	if req.Actor.ActorID == "" || req.Actor.Role == "" {
		t.Errorf("%s: request actor missing actor_id or role", name)
	}
	if len(fx.Responses) == 0 {
		t.Errorf("%s: fixture carries no responses", name)
	}
	for i, resp := range fx.Responses {
		if resp.IntentID != req.IntentID {
			t.Errorf("%s: response %d intent_id does not echo the request", name, i)
		}
		if resp.Success {
			if resp.Refusal != nil {
				t.Errorf("%s: response %d is a success and must not carry a refusal", name, i)
			}
			// new_cursor is present when the intent moved the world; non-mutating outcomes
			// (e.g. OPEN_ARTIFACT) legitimately answer without one.
			continue
		}
		if resp.Refusal == nil {
			t.Errorf("%s: response %d is a refusal and must carry the typed refusal envelope", name, i)
			continue
		}
		if !contains(AllRefusalKinds, resp.Refusal.RefusalKind) {
			t.Errorf("%s: response %d refusal_kind %q is not in AllRefusalKinds", name, i, resp.Refusal.RefusalKind)
		}
		if resp.Refusal.Message == "" {
			t.Errorf("%s: response %d refusal message is empty", name, i)
		}
		if resp.NewCursor != nil {
			t.Errorf("%s: response %d is a refusal and must not carry new_cursor", name, i)
		}
	}

	proto := payloadPrototype(req.ActionName)
	if proto == nil {
		if len(req.Parameters) != 0 {
			t.Errorf("%s: action %q carries parameters but has no typed payload in the contract", name, req.ActionName)
		}
		return
	}
	if len(req.Parameters) == 0 {
		t.Errorf("%s: action %q must carry its typed payload in parameters", name, req.ActionName)
		return
	}
	raw, err := json.Marshal(req.Parameters)
	if err != nil {
		t.Errorf("%s: re-encode parameters: %v", name, err)
		return
	}
	if err := json.Unmarshal(raw, proto); err != nil {
		t.Errorf("%s: parameters do not decode into %T: %v", name, proto, err)
		return
	}
	switch p := proto.(type) {
	case *ProposeCasePayload:
		if p.TemplateRef == "" || p.Params == nil || p.PreflightDigest == "" {
			t.Errorf("%s: ProposeCasePayload missing template_ref, params or preflight_digest", name)
		}
	case *AddDiscretionaryWorkPayload:
		if p.CaseID == "" || p.StageID == "" || p.Kind == "" || p.Title == "" || p.Justification == "" {
			t.Errorf("%s: AddDiscretionaryWorkPayload missing case_id, stage_id, kind, title or justification", name)
		}
	case *ExecuteItemPayload:
		if p.ItemID == "" {
			t.Errorf("%s: ExecuteItemPayload missing item_id", name)
		}
	case *CompleteHumanTaskPayload:
		if p.ItemID == "" || p.Result == nil || p.Justification == "" {
			t.Errorf("%s: CompleteHumanTaskPayload missing item_id, result or justification", name)
		}
	case *CaseLifecyclePayload:
		if p.CaseID == "" || p.Reason == "" {
			t.Errorf("%s: CaseLifecyclePayload missing case_id or reason", name)
		}
	default:
		t.Errorf("%s: unhandled payload prototype %T", name, proto)
	}
}

// TestGoldenCoversEveryIntentKind pins the exhaustive intent set both ways: every golden intent
// fixture kind is a known kind, and every known kind has at least one fixture. A kind added on
// only one side of the contract fails here.
func TestGoldenCoversEveryIntentKind(t *testing.T) {
	dir := goldenDir(t)
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("read golden dir: %v", err)
	}
	got := map[string]bool{}
	for _, e := range entries {
		name := e.Name()
		if !strings.HasPrefix(name, "intent-") || !strings.HasSuffix(name, ".json") {
			continue
		}
		raw, err := os.ReadFile(filepath.Join(dir, name))
		if err != nil {
			t.Fatalf("read %s: %v", name, err)
		}
		var fx IntentFixture
		if err := json.Unmarshal(raw, &fx); err != nil {
			t.Fatalf("unmarshal %s: %v", name, err)
		}
		if !contains(AllIntentKinds, fx.Request.ActionName) {
			t.Errorf("%s: action_name %q is not in the Go AllIntentKinds list; a kind exists in the fixtures (and likely in the TS union) that the Go side does not know", name, fx.Request.ActionName)
		}
		got[fx.Request.ActionName] = true
	}
	for _, kind := range AllIntentKinds {
		if !got[kind] {
			t.Errorf("intent kind %q has no golden fixture under golden/intent-*.json", kind)
		}
	}
}

// TestGoldenCoversEveryRefusalKind pins the exhaustive refusal vocabulary: every refusal
// exercised by the goldens is known, and every known refusal kind is exercised at least once.
func TestGoldenCoversEveryRefusalKind(t *testing.T) {
	dir := goldenDir(t)
	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("read golden dir: %v", err)
	}
	got := map[string]bool{}
	for _, e := range entries {
		name := e.Name()
		if !strings.HasPrefix(name, "intent-") || !strings.HasSuffix(name, ".json") {
			continue
		}
		raw, err := os.ReadFile(filepath.Join(dir, name))
		if err != nil {
			t.Fatalf("read %s: %v", name, err)
		}
		var fx IntentFixture
		if err := json.Unmarshal(raw, &fx); err != nil {
			t.Fatalf("unmarshal %s: %v", name, err)
		}
		for _, resp := range fx.Responses {
			if resp.Refusal != nil {
				if !contains(AllRefusalKinds, resp.Refusal.RefusalKind) {
					t.Errorf("%s: refusal_kind %q is not in the Go AllRefusalKinds list", name, resp.Refusal.RefusalKind)
				}
				got[resp.Refusal.RefusalKind] = true
			}
		}
	}
	for _, kind := range AllRefusalKinds {
		if !got[kind] {
			t.Errorf("refusal kind %q is never exercised by the golden fixtures", kind)
		}
	}
}

// TestStreamEventKindsExhaustive pins the exhaustive SSE vocabulary against the jsonl fixture.
func TestStreamEventKindsExhaustive(t *testing.T) {
	raw, err := os.ReadFile(filepath.Join(goldenDir(t), "sse-events.jsonl"))
	if err != nil {
		t.Fatalf("read sse-events.jsonl: %v", err)
	}
	got := map[string]bool{}
	for i, line := range strings.Split(strings.TrimRight(string(raw), "\n"), "\n") {
		var ev StreamEvent
		if err := json.Unmarshal([]byte(line), &ev); err != nil {
			t.Fatalf("sse-events.jsonl line %d: %v", i+1, err)
		}
		if !contains(AllStreamEventKinds, ev.EventType) {
			t.Errorf("sse-events.jsonl line %d: event_type %q is not in the Go AllStreamEventKinds list", i+1, ev.EventType)
		}
		got[ev.EventType] = true
	}
	for _, kind := range AllStreamEventKinds {
		if !got[kind] {
			t.Errorf("stream event kind %q has no line in sse-events.jsonl", kind)
		}
	}
}

// TestKindListsAreWellFormed guards the exhaustive lists themselves: no duplicates, and payload
// kinds are a subset of intent kinds. Declaration order mirrors types.ts, not alphabetical order.
func TestKindListsAreWellFormed(t *testing.T) {
	for _, tc := range []struct {
		name string
		list []string
	}{
		{"AllIntentKinds", AllIntentKinds},
		{"AllRefusalKinds", AllRefusalKinds},
		{"AllStreamEventKinds", AllStreamEventKinds},
		{"PayloadCarryingKinds", PayloadCarryingKinds},
	} {
		seen := map[string]bool{}
		for _, k := range tc.list {
			if seen[k] {
				t.Errorf("%s: duplicate kind %q", tc.name, k)
			}
			seen[k] = true
		}
	}
	for _, k := range PayloadCarryingKinds {
		if !contains(AllIntentKinds, k) {
			t.Errorf("PayloadCarryingKinds: %q is not in AllIntentKinds", k)
		}
	}
}

func contains(list []string, v string) bool {
	for _, s := range list {
		if s == v {
			return true
		}
	}
	return false
}
