package sfwp

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

func TestAskAdapterMapsQuestionIdentityAndCompleteAnswer(t *testing.T) {
	answer := `{"answer_id":"answer-1","question_id":"question-1","disposition":"partial","claims":[{"claim_id":"claim-1","claim_class":"demonstrated_capability","subject":"build","status":"demonstrated","statement":"Grounded claim.","snapshot_ref":"snapshot-1","evidence_refs":["evidence-1"],"settlement_refs":["settlement-1"],"capability_record_ref":"capability-1"}],"omitted_claim_classes":["authority_requirements"],"snapshot_ref":"snapshot-1","freshness":"current","assurance":"local_tamper_evident","limitations":["bounded"],"authority_notice":"No authority granted.","answered_at":"2026-10-01T12:00:00Z"}`
	fs := newFakeServer(t, func(string) (string, bool) { return answer, false })
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	adapter := NewAskAdapter(client, ports.ActorClaim{ActorID: "gateway", Role: "service"})
	effective := ports.ActorClaim{ActorID: "operator_local", Role: "operator"}
	caseID := ports.CaseRef("case-1")
	got, err := adapter.Ask(context.Background(), effective, ports.AskQuestion{
		Kind: "ask_capability", Subject: " build ", Purpose: "", Case: &caseID,
	})
	if err != nil {
		t.Fatal(err)
	}
	lines := capturedAskLines(fs)
	if len(lines) != 1 {
		t.Fatalf("Ask request count=%d, want 1", len(lines))
	}
	var request map[string]any
	if err := json.Unmarshal([]byte(lines[0]), &request); err != nil {
		t.Fatal(err)
	}
	if request["verb"] != "ask" || request["kind"] != "ask_capability" || request["subject"] != " build " ||
		request["purpose"] != "" || request["case"] != "case-1" || request["actor_id"] != "operator_local" {
		t.Fatalf("adapter changed semantic input or effective actor: %#v", request)
	}
	if _, exists := request["request_id"]; exists {
		t.Fatalf("Ask must not invent a request_id: %#v", request)
	}
	actor, _ := request["actor"].(map[string]any)
	onBehalfOf, _ := request["on_behalf_of"].(map[string]any)
	if actor["actor_id"] != "gateway" || actor["role"] != "service" ||
		onBehalfOf["actor_id"] != "operator_local" || onBehalfOf["role"] != "operator" {
		t.Fatalf("adapter principal pair does not match configured gateway and effective user: %#v", request)
	}
	if got.AnswerID != "answer-1" || got.QuestionID != "question-1" || got.Disposition != "partial" ||
		got.Freshness != "current" || got.Assurance != "local_tamper_evident" || got.AuthorityNotice != "No authority granted." ||
		got.AnsweredAt != "2026-10-01T12:00:00Z" || got.SnapshotRef != "snapshot-1" ||
		len(got.Claims) != 1 || got.Claims[0].ClaimID != "claim-1" || got.Claims[0].ClaimClass != "demonstrated_capability" ||
		got.Claims[0].Status != "demonstrated" || got.Claims[0].CapabilityRecordRef == nil || *got.Claims[0].CapabilityRecordRef != "capability-1" ||
		len(got.Claims[0].EvidenceRefs) != 1 || got.Claims[0].EvidenceRefs[0] != "evidence-1" ||
		len(got.Claims[0].SettlementRefs) != 1 || got.Claims[0].SettlementRefs[0] != "settlement-1" ||
		len(got.OmittedClaimClasses) != 1 || got.OmittedClaimClasses[0] != "authority_requirements" ||
		len(got.Limitations) != 1 || got.Limitations[0] != "bounded" {
		t.Fatalf("adapter lost fields from the governed answer: %+v", got)
	}
}

func TestAskAdapterKeepsOmittedCaseAbsentAndAcceptsLegitimateEmptyArrays(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) {
		return `{"answer_id":"answer-2","question_id":"question-2","disposition":"denied","claims":[],"omitted_claim_classes":[],"snapshot_ref":"snapshot-2","freshness":"stale","assurance":"local_tamper_evident","limitations":[],"authority_notice":"No authority granted.","answered_at":"2026-10-01T12:00:00Z"}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	adapter := NewAskAdapter(client, ports.ActorClaim{ActorID: "gateway", Role: "service"})
	got, err := adapter.Ask(context.Background(), ports.ActorClaim{ActorID: "operator_local", Role: "operator"}, ports.AskQuestion{
		Kind: "ask_why_denied", Subject: "operation", Purpose: "planning",
	})
	if err != nil {
		t.Fatal(err)
	}
	lines := capturedAskLines(fs)
	if len(lines) != 1 {
		t.Fatalf("Ask request count=%d, want 1", len(lines))
	}
	var request map[string]any
	if err := json.Unmarshal([]byte(lines[0]), &request); err != nil {
		t.Fatal(err)
	}
	if _, exists := request["case"]; exists {
		t.Fatalf("omitted case became present on the wire: %#v", request)
	}
	if got.Claims == nil || got.OmittedClaimClasses == nil || got.Limitations == nil || got.Disposition != "denied" {
		t.Fatalf("valid empty disclosure arrays or denied disposition were changed: %+v", got)
	}
}

func capturedAskLines(fs *fakeServer) []string {
	fs.mu.Lock()
	defer fs.mu.Unlock()
	return append([]string(nil), fs.requests...)
}

func TestAskAdapterRejectsMalformedOrUnsupportedDisclosure(t *testing.T) {
	cases := map[string]string{
		"unknown disposition":                `{"answer_id":"a","question_id":"q","disposition":"future","claims":[],"omitted_claim_classes":[],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"unknown freshness":                  `{"answer_id":"a","question_id":"q","disposition":"answered","claims":[],"omitted_claim_classes":[],"snapshot_ref":"s","freshness":"future","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"missing required claims":            `{"answer_id":"a","question_id":"q","disposition":"answered","omitted_claim_classes":[],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"null required claims":               `{"answer_id":"a","question_id":"q","disposition":"answered","claims":null,"omitted_claim_classes":[],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"unknown omitted class":              `{"answer_id":"a","question_id":"q","disposition":"answered","claims":[],"omitted_claim_classes":["future"],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"unknown claim status":               `{"answer_id":"a","question_id":"q","disposition":"answered","claims":[{"claim_id":"c","claim_class":"identity","subject":"s","status":"future","statement":"x","snapshot_ref":"r","evidence_refs":[],"settlement_refs":[]}],"omitted_claim_classes":[],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"claim missing references":           `{"answer_id":"a","question_id":"q","disposition":"answered","claims":[{"claim_id":"c","claim_class":"identity","subject":"s","status":"unknown","statement":"x","snapshot_ref":"r"}],"omitted_claim_classes":[],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
		"null optional capability reference": `{"answer_id":"a","question_id":"q","disposition":"answered","claims":[{"claim_id":"c","claim_class":"identity","subject":"s","status":"unknown","statement":"x","snapshot_ref":"r","evidence_refs":[],"settlement_refs":[],"capability_record_ref":null}],"omitted_claim_classes":[],"snapshot_ref":"s","freshness":"current","assurance":"x","limitations":[],"authority_notice":"n","answered_at":"t"}`,
	}
	for name, body := range cases {
		t.Run(name, func(t *testing.T) {
			fs := newFakeServer(t, func(string) (string, bool) { return body, false })
			client, err := New(testConfig(fs.socket))
			if err != nil {
				t.Fatal(err)
			}
			defer client.Close()
			adapter := NewAskAdapter(client, ports.ActorClaim{ActorID: "gateway", Role: "service"})
			_, err = adapter.Ask(context.Background(), ports.ActorClaim{ActorID: "operator_local", Role: "operator"}, ports.AskQuestion{
				Kind: "ask_capability", Subject: "real subject", Purpose: "planning",
			})
			if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
				t.Fatalf("malformed upstream disclosure must be unavailable, got %v", err)
			}
		})
	}
}
