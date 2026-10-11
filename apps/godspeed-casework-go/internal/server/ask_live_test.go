//go:build live

package server_test

import (
	"bufio"
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
)

const askLivePolicy = `version: "0.1"
rules: []
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: live-ask-grant
        actor_role: operator_local
        claim_classes:
          - declared_capability
          - installed_capability
          - demonstrated_capability
`

type askLedgerEntry struct {
	EntryULID      string                     `json:"entry_ulid"`
	RecordKind     string                     `json:"record_kind"`
	SubjectRefs    []string                   `json:"subject_refs"`
	AuthorityRefs  []string                   `json:"authority_refs"`
	Payload        map[string]json.RawMessage `json:"payload"`
	WriterIdentity string                     `json:"writer_identity_ref"`
}

type persistedClaimProjection struct {
	ClaimID             string          `json:"claim_id"`
	ClaimClass          string          `json:"claim_class"`
	Subject             string          `json:"subject"`
	Status              string          `json:"status"`
	Statement           string          `json:"statement"`
	SnapshotRef         string          `json:"snapshot_ref"`
	EvidenceRefs        []string        `json:"evidence_refs"`
	SettlementRefs      []string        `json:"settlement_refs"`
	CapabilityRecordRef *string         `json:"capability_record_ref"`
	Flags               json.RawMessage `json:"flags"`
	Limitations         json.RawMessage `json:"limitations"`
	AuthoredBy          json.RawMessage `json:"authored_by"`
}

type persistedAnswerProjection struct {
	AnswerID            string                     `json:"answer_id"`
	QuestionID          string                     `json:"question_id"`
	Disposition         string                     `json:"disposition"`
	Claims              []persistedClaimProjection `json:"claims"`
	OmittedClaimClasses []string                   `json:"omitted_claim_classes"`
	SnapshotRef         string                     `json:"snapshot_ref"`
	Freshness           string                     `json:"freshness"`
	Assurance           string                     `json:"assurance"`
	Limitations         []string                   `json:"limitations"`
	AuthorityNotice     string                     `json:"authority_notice"`
	AnsweredAt          string                     `json:"answered_at"`
}

func TestLiveAskLedger(t *testing.T) {
	repoRoot := livetest.RepoRoot(t)
	serverBin := filepath.Join(repoRoot, "target", "debug", "sea-forge-server")
	cliBin := filepath.Join(repoRoot, "target", "debug", "sea-forge")
	for _, bin := range []string{serverBin, cliBin} {
		if _, err := os.Stat(bin); err != nil {
			t.Fatalf("live Ask proof requires the existing binary %s (it must not trigger a build): %v", bin, err)
		}
	}
	cell := livetest.NewCell(t)
	stack := livestack.AssembleStack(t, cell)
	seedLiveAskModel(t, cliBin, cell.Root())
	concept := firstBundledConcept(t, cliBin, cell.Root())
	writeLiveAskPolicy(t, cell.Root(), "version: \"0.1\"\nrules: []\npolicy_surfaces:\n  self_disclosure:\n    mode: deny-by-default\n")

	ts := stack.HTTPServer(t)
	op := stack.Login(t, ts.URL, "operator")
	beforeEntries := readAskLedger(t, cell.Root())
	beforeDelegations := askDelegations(cell.DelegationAuditRecords())
	if len(beforeEntries) != 0 || len(beforeDelegations) != 0 {
		t.Fatalf("fresh fixture cell unexpectedly has prior Ask state: entries=%d delegations=%d", len(beforeEntries), len(beforeDelegations))
	}

	unauthenticated := postAnonymousAsk(t, ts.URL, `{"kind":"ask_capability","subject":"`+concept+`"}`)
	if unauthenticated.StatusCode != http.StatusUnauthorized {
		body, _ := io.ReadAll(unauthenticated.Body)
		unauthenticated.Body.Close()
		t.Fatalf("Ask without a session should be refused before the kernel, got %d: %s", unauthenticated.StatusCode, body)
	}
	unauthenticated.Body.Close()
	invalid := askPostRaw(t, ts.URL, op, `{"kind":"ask_capability","subject":"  "}`, true)
	if invalid.StatusCode != http.StatusBadRequest {
		body, _ := io.ReadAll(invalid.Body)
		invalid.Body.Close()
		t.Fatalf("invalid Ask should be refused before the kernel, got %d: %s", invalid.StatusCode, body)
	}
	invalid.Body.Close()
	noCSRF := askPostRaw(t, ts.URL, op, `{"kind":"ask_capability","subject":"`+concept+`"}`, false)
	if noCSRF.StatusCode != http.StatusForbidden {
		body, _ := io.ReadAll(noCSRF.Body)
		noCSRF.Body.Close()
		t.Fatalf("Ask without CSRF should be refused before the kernel, got %d: %s", noCSRF.StatusCode, body)
	}
	noCSRF.Body.Close()
	if got := readAskLedger(t, cell.Root()); len(got) != 0 {
		t.Fatalf("unauthenticated, invalid, or CSRF-refused Ask wrote %d disclosure records", len(got))
	}
	if got := askDelegations(cell.DelegationAuditRecords()); len(got) != 0 {
		t.Fatalf("unauthenticated, invalid, or CSRF-refused Ask wrote %d delegated request records", len(got))
	}

	opDenied := stack.Login(t, ts.URL, "operator")
	denied := postLiveAsk(t, opDenied, concept)
	if denied.Disposition != "denied" {
		t.Fatalf("real kernel should return a governed denied answer under absent disclosure grants, got %q", denied.Disposition)
	}
	writeLiveAskPolicy(t, cell.Root(), askLivePolicy)
	opAnswered := stack.Login(t, ts.URL, "operator")
	answered := postLiveAsk(t, opAnswered, concept)
	if answered.Disposition != "answered" || len(answered.Claims) == 0 {
		t.Fatalf("real bundled-model subject should produce granted disclosures, got disposition=%q claims=%d", answered.Disposition, len(answered.Claims))
	}
	for _, claim := range answered.Claims {
		if claim.Subject != concept || strings.TrimSpace(claim.Statement) == "" {
			t.Fatalf("granted claim is not grounded in the selected bundled concept: %+v", claim)
		}
	}

	entries := readAskLedger(t, cell.Root())
	if len(entries) != 8 {
		t.Fatalf("two Ask calls should commit two four-entry chains, got %d entries", len(entries))
	}
	verifyAskChain(t, entries[:4], denied, concept)
	verifyAskChain(t, entries[4:], answered, concept)
	for _, entry := range entries {
		if entry.WriterIdentity != "operator_local" {
			t.Errorf("Ask ledger writer=%q, want effective session actor operator_local", entry.WriterIdentity)
		}
	}
	delegations := askDelegations(cell.DelegationAuditRecords())
	if len(delegations) != 2 {
		t.Fatalf("two admitted Ask calls should each have a durable delegation audit record, got %d", len(delegations))
	}
	for _, record := range delegations {
		if record["verb"] != "ask" || record["gateway_actor_id"] != "gateway" ||
			record["effective_actor_id"] != "operator_local" || record["effective_role"] != "operator" || record["request_id"] != nil {
			t.Errorf("delegation ledger did not preserve the gateway/effective-user pair for uncorrelated Ask: %+v", record)
		}
	}
	verifyAskLedgerCLI(t, cliBin, cell.Root())
}

func postAnonymousAsk(t *testing.T, base, body string) *http.Response {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, base+"/api/ask", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	return resp
}

func askPostRaw(t *testing.T, base string, session *livestack.Session, body string, withCSRF bool) *http.Response {
	t.Helper()
	req, err := http.NewRequest(http.MethodPost, base+"/api/ask", strings.NewReader(body))
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set("Content-Type", "application/json")
	if withCSRF {
		req.Header.Set("X-CSRF-Token", session.CSRF)
	}
	resp, err := session.Client.Do(req)
	if err != nil {
		t.Fatal(err)
	}
	return resp
}

func postLiveAsk(t *testing.T, session *livestack.Session, subject string) contract.ThothAnswerView {
	t.Helper()
	resp, err := session.PostJSON("/api/ask", contract.ThothAskRequest{Kind: "ask_capability", Subject: subject})
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()
	raw, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Ask returned %d: %s", resp.StatusCode, raw)
	}
	var answer contract.ThothAnswerView
	if err := json.Unmarshal(raw, &answer); err != nil {
		t.Fatalf("decode protected Ask answer: %v: %s", err, raw)
	}
	return answer
}

func seedLiveAskModel(t *testing.T, cli, root string) {
	t.Helper()
	runAskCLI(t, cli, "self-model", "--root", root, "rebuild")
}

func firstBundledConcept(t *testing.T, cli, root string) string {
	t.Helper()
	raw := runAskCLI(t, cli, "self-model", "--root", root, "show", "--json")
	var snapshot struct {
		SystemModelRef struct {
			ConceptRefs []string `json:"concept_refs"`
		} `json:"system_model_ref"`
	}
	if err := json.Unmarshal(raw, &snapshot); err != nil {
		t.Fatalf("decode self-model snapshot: %v: %s", err, raw)
	}
	if len(snapshot.SystemModelRef.ConceptRefs) == 0 || strings.TrimSpace(snapshot.SystemModelRef.ConceptRefs[0]) == "" {
		t.Fatal("real rebuilt bundled self-model exposed no concept subject")
	}
	return snapshot.SystemModelRef.ConceptRefs[0]
}

func writeLiveAskPolicy(t *testing.T, root, policy string) {
	t.Helper()
	path := filepath.Join(root, "authority", "active-policy.json")
	if err := os.WriteFile(path, []byte(policy), 0o600); err != nil {
		t.Fatalf("seed disposable-cell Ask policy: %v", err)
	}
}

func runAskCLI(t *testing.T, cli string, args ...string) []byte {
	t.Helper()
	cmd := exec.Command(cli, args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("existing sea-forge CLI %v failed: %v\n%s", args, err, out)
	}
	return bytes.TrimSpace(out)
}

func readAskLedger(t *testing.T, root string) []askLedgerEntry {
	t.Helper()
	path := filepath.Join(root, "ledgers", "thoth-asks", "entries.jsonl")
	file, err := os.Open(path)
	if os.IsNotExist(err) {
		return nil
	}
	if err != nil {
		t.Fatal(err)
	}
	defer file.Close()
	var entries []askLedgerEntry
	scanner := bufio.NewScanner(file)
	scanner.Buffer(make([]byte, 4096), 4<<20)
	for scanner.Scan() {
		var entry askLedgerEntry
		if err := json.Unmarshal(scanner.Bytes(), &entry); err != nil {
			t.Fatalf("decode actual Thoth ledger entry: %v", err)
		}
		entries = append(entries, entry)
	}
	if err := scanner.Err(); err != nil {
		t.Fatal(err)
	}
	return entries
}

func verifyAskChain(t *testing.T, chain []askLedgerEntry, answer contract.ThothAnswerView, subject string) {
	t.Helper()
	wantKinds := []string{"self_disclosure_question", "self_disclosure_plan", "self_disclosure_decision", "self_disclosure_answer"}
	if len(chain) != len(wantKinds) {
		t.Fatalf("Ask chain has %d entries, want 4", len(chain))
	}
	for i, want := range wantKinds {
		if chain[i].RecordKind != want || chain[i].WriterIdentity != "operator_local" {
			t.Fatalf("chain entry %d kind/writer=%q/%q, want %q/operator_local", i, chain[i].RecordKind, chain[i].WriterIdentity, want)
		}
	}
	var question struct {
		QuestionID string `json:"question_id"`
		ActorID    string `json:"actor_id"`
		Kind       string `json:"kind"`
		Subject    string `json:"subject"`
	}
	decodeAskPayload(t, chain[0], &question)
	if question.QuestionID != answer.QuestionID || question.ActorID != "operator_local" || question.Kind != "ask_capability" || question.Subject != subject {
		t.Fatalf("persisted question does not match the effective user and returned answer: %+v / %+v", question, answer)
	}
	var plan struct {
		QuestionID           string `json:"question_id"`
		AuthorityDecisionRef string `json:"authority_decision_ref"`
	}
	decodeAskPayload(t, chain[1], &plan)
	var decision struct {
		AuthorityDecisionRef string `json:"authority_decision_ref"`
	}
	decodeAskPayload(t, chain[2], &decision)
	var persisted persistedAnswerProjection
	decodeAskPayload(t, chain[3], &persisted)
	if plan.QuestionID != answer.QuestionID || plan.AuthorityDecisionRef != answer.AnswerID || decision.AuthorityDecisionRef != answer.AnswerID ||
		persisted.AnswerID != answer.AnswerID || persisted.QuestionID != answer.QuestionID {
		t.Fatalf("question/plan/decision/answer links do not match the returned answer: plan=%+v decision=%+v answer=%+v", plan, decision, persisted)
	}
	if len(chain[1].AuthorityRefs) != 1 || chain[1].AuthorityRefs[0] != chain[0].EntryULID ||
		len(chain[2].AuthorityRefs) != 1 || chain[2].AuthorityRefs[0] != chain[1].EntryULID ||
		len(chain[3].AuthorityRefs) != 1 || chain[3].AuthorityRefs[0] != chain[2].EntryULID {
		t.Fatalf("ledger authority references do not form the committed question→plan→decision→answer chain")
	}
	projected := contract.ThothAnswerView{
		AnswerID: persisted.AnswerID, QuestionID: persisted.QuestionID, Disposition: contract.ThothDisposition(persisted.Disposition),
		Claims: make([]contract.ThothClaimView, len(persisted.Claims)), OmittedClaimClasses: make([]contract.ThothClaimClass, len(persisted.OmittedClaimClasses)),
		SnapshotRef: persisted.SnapshotRef, Freshness: contract.ThothFreshness(persisted.Freshness), Assurance: persisted.Assurance,
		Limitations: persisted.Limitations, AuthorityNotice: persisted.AuthorityNotice, AnsweredAt: persisted.AnsweredAt,
	}
	for i, claim := range persisted.Claims {
		if len(claim.Flags) == 0 || len(claim.Limitations) == 0 || len(claim.AuthoredBy) == 0 {
			t.Fatalf("persisted GroundedClaim lacks its kernel-only flags/limitations fields: %+v", claim)
		}
		var authoredBy string
		if err := json.Unmarshal(claim.AuthoredBy, &authoredBy); err != nil || authoredBy != "thoth" {
			t.Fatalf("persisted GroundedClaim did not retain its kernel author: %s (%v)", claim.AuthoredBy, err)
		}
		projected.Claims[i] = contract.ThothClaimView{
			ClaimID: claim.ClaimID, ClaimClass: contract.ThothClaimClass(claim.ClaimClass), Subject: claim.Subject,
			Status: contract.ThothClaimStatus(claim.Status), Statement: claim.Statement, SnapshotRef: claim.SnapshotRef,
			EvidenceRefs: claim.EvidenceRefs, SettlementRefs: claim.SettlementRefs, CapabilityRecordRef: claim.CapabilityRecordRef,
		}
	}
	for i, class := range persisted.OmittedClaimClasses {
		projected.OmittedClaimClasses[i] = contract.ThothClaimClass(class)
	}
	// ClaimView is the kernel's deliberate projection of GroundedClaim: the API retains its eight
	// required fields and optional capability reference, while persistence additionally carries
	// flags, claim limitations, and authored_by. Compare the eleven top-level answer fields and
	// every field in that actual projection; do not equate structurally different records.
	if !reflect.DeepEqual(projected, answer) {
		got, _ := json.Marshal(projected)
		want, _ := json.Marshal(answer)
		t.Fatalf("API answer differs from the persisted ThothAnswer after the exact ClaimView projection:\nledger projection: %s\nHTTP: %s", got, want)
	}
}

func decodeAskPayload(t *testing.T, entry askLedgerEntry, target any) {
	t.Helper()
	raw, err := json.Marshal(entry.Payload)
	if err != nil {
		t.Fatal(err)
	}
	if err := json.Unmarshal(raw, target); err != nil {
		t.Fatalf("decode %s payload: %v", entry.RecordKind, err)
	}
}

func askDelegations(records []map[string]any) []map[string]any {
	var asks []map[string]any
	for _, entry := range records {
		payload, _ := entry["payload"].(map[string]any)
		if payload != nil && payload["verb"] == "ask" {
			asks = append(asks, payload)
		}
	}
	return asks
}

func verifyAskLedgerCLI(t *testing.T, cli, root string) {
	t.Helper()
	out := runAskCLI(t, cli, "ledger", "--root", root, "verify", "thoth-asks")
	if !strings.Contains(string(out), "ledger thoth-asks: verified") {
		t.Fatalf("existing ledger verification CLI did not confirm the Thoth ledger: %s", out)
	}
}
