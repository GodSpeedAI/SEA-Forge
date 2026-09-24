package contract

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
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

// roundTripLines does the per-line equivalent for the SSE jsonl fixture. Under GOLDEN_UPDATE it
// rewrites the file in canonical line encoding instead of comparing.
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
