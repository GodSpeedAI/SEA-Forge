package sfwp

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// AskAdapter translates the application's semantic Ask port to the protected SFWP ask verb.
// The gateway identity is configured at construction; the effective end-user identity is supplied
// by the authenticated request and is never taken from AskQuestion.
type AskAdapter struct {
	client  *Client
	gateway ports.ActorClaim
}

func NewAskAdapter(client *Client, gateway ports.ActorClaim) *AskAdapter {
	return &AskAdapter{client: client, gateway: gateway}
}

type thothClaimWire struct {
	ClaimID             string          `json:"claim_id"`
	ClaimClass          string          `json:"claim_class"`
	Subject             string          `json:"subject"`
	Status              string          `json:"status"`
	Statement           string          `json:"statement"`
	SnapshotRef         string          `json:"snapshot_ref"`
	EvidenceRefs        []string        `json:"evidence_refs"`
	SettlementRefs      []string        `json:"settlement_refs"`
	CapabilityRecordRef json.RawMessage `json:"capability_record_ref"`
}

type thothAnswerWire struct {
	AnswerID            string           `json:"answer_id"`
	QuestionID          string           `json:"question_id"`
	Disposition         string           `json:"disposition"`
	Claims              []thothClaimWire `json:"claims"`
	OmittedClaimClasses []string         `json:"omitted_claim_classes"`
	SnapshotRef         string           `json:"snapshot_ref"`
	Freshness           string           `json:"freshness"`
	Assurance           string           `json:"assurance"`
	Limitations         []string         `json:"limitations"`
	AuthorityNotice     string           `json:"authority_notice"`
	AnsweredAt          string           `json:"answered_at"`
}

func (a *AskAdapter) Ask(ctx context.Context, effective ports.ActorClaim, question ports.AskQuestion) (ports.AskAnswer, error) {
	if a == nil || a.client == nil || strings.TrimSpace(a.gateway.ActorID) == "" || strings.TrimSpace(a.gateway.Role) == "" {
		return ports.AskAnswer{}, apperr.New(apperr.KindUnavailable, "", "ask", "the governed Ask adapter is not configured")
	}
	if strings.TrimSpace(effective.ActorID) == "" || strings.TrimSpace(effective.Role) == "" {
		return ports.AskAnswer{}, apperr.New(apperr.KindInvalid, "", "ask", "the effective Ask actor is missing")
	}
	if strings.TrimSpace(question.Kind) == "" || strings.TrimSpace(question.Subject) == "" {
		return ports.AskAnswer{}, apperr.New(apperr.KindInvalid, "", "ask", "the Ask question is incomplete")
	}

	req := newRequest("ask")
	req.body["kind"] = question.Kind
	req.body["subject"] = question.Subject
	req.body["purpose"] = question.Purpose
	if question.Case != nil {
		req.body["case"] = string(*question.Case)
	}
	req.body["actor_id"] = effective.ActorID
	Governance{
		Actor:      Actor{ActorID: a.gateway.ActorID, Role: a.gateway.Role},
		OnBehalfOf: &Actor{ActorID: effective.ActorID, Role: effective.Role},
	}.apply(req.body)

	resp, err := a.client.Do(ctx, req)
	if err != nil {
		return ports.AskAnswer{}, err
	}
	var wire thothAnswerWire
	dec := json.NewDecoder(bytes.NewReader(resp.Raw))
	dec.DisallowUnknownFields()
	if err := dec.Decode(&wire); err != nil {
		return ports.AskAnswer{}, apperr.Wrap(apperr.KindUnavailable, "", "ask", "the authority returned an invalid Ask answer", err)
	}
	var extra json.RawMessage
	if err := dec.Decode(&extra); err != io.EOF {
		return ports.AskAnswer{}, apperr.New(apperr.KindUnavailable, "", "ask", "the authority returned more than one Ask answer")
	}
	if err := validateThothAnswer(wire); err != nil {
		return ports.AskAnswer{}, apperr.Wrap(apperr.KindUnavailable, "", "ask", "the authority returned an unsupported Ask disclosure", err)
	}

	answer := ports.AskAnswer{
		AnswerID: wire.AnswerID, QuestionID: wire.QuestionID, Disposition: wire.Disposition,
		Claims: make([]ports.AskClaim, len(wire.Claims)), OmittedClaimClasses: append([]string{}, wire.OmittedClaimClasses...),
		SnapshotRef: wire.SnapshotRef, Freshness: wire.Freshness, Assurance: wire.Assurance,
		Limitations: append([]string{}, wire.Limitations...), AuthorityNotice: wire.AuthorityNotice,
		AnsweredAt: wire.AnsweredAt,
	}
	for i, claim := range wire.Claims {
		capabilityRef, err := capabilityRecordRef(claim.CapabilityRecordRef)
		if err != nil {
			return ports.AskAnswer{}, apperr.Wrap(apperr.KindUnavailable, "", "ask", "the authority returned an invalid capability reference", err)
		}
		answer.Claims[i] = ports.AskClaim{
			ClaimID: claim.ClaimID, ClaimClass: claim.ClaimClass, Subject: claim.Subject,
			Status: claim.Status, Statement: claim.Statement, SnapshotRef: claim.SnapshotRef,
			EvidenceRefs: append([]string{}, claim.EvidenceRefs...), SettlementRefs: append([]string{}, claim.SettlementRefs...),
			CapabilityRecordRef: capabilityRef,
		}
	}
	return answer, nil
}

func validateThothAnswer(answer thothAnswerWire) error {
	if !nonblank(answer.AnswerID) || !nonblank(answer.QuestionID) || !nonblank(answer.SnapshotRef) ||
		!nonblank(answer.Assurance) || !nonblank(answer.AuthorityNotice) || !nonblank(answer.AnsweredAt) {
		return apperr.New(apperr.KindUnavailable, "", "ask", "the answer is missing required disclosure metadata")
	}
	if !oneOf(answer.Disposition, "answered", "partial", "denied") || !oneOf(answer.Freshness, "current", "stale") {
		return apperr.New(apperr.KindUnavailable, "", "ask", "the answer contains an unknown disposition or freshness")
	}
	if answer.Claims == nil || answer.OmittedClaimClasses == nil || answer.Limitations == nil {
		return apperr.New(apperr.KindUnavailable, "", "ask", "the answer omitted a required disclosure array")
	}
	for _, class := range answer.OmittedClaimClasses {
		if !validClaimClass(class) {
			return apperr.New(apperr.KindUnavailable, "", "ask", "the answer contains an unknown omitted claim class")
		}
	}
	for _, claim := range answer.Claims {
		if !nonblank(claim.ClaimID) || !nonblank(claim.Subject) || !nonblank(claim.Statement) || !nonblank(claim.SnapshotRef) ||
			claim.EvidenceRefs == nil || claim.SettlementRefs == nil {
			return apperr.New(apperr.KindUnavailable, "", "ask", "a claim is missing required fields or reference arrays")
		}
		if !validClaimClass(claim.ClaimClass) || !oneOf(claim.Status, "unknown", "unsupported", "declared", "installed", "available", "validated", "demonstrated") {
			return apperr.New(apperr.KindUnavailable, "", "ask", "a claim contains an unknown class or status")
		}
		if _, err := capabilityRecordRef(claim.CapabilityRecordRef); err != nil {
			return apperr.Wrap(apperr.KindUnavailable, "", "ask", "a claim contains an invalid capability reference", err)
		}
	}
	return nil
}

func validClaimClass(value string) bool {
	return oneOf(value, "identity", "architecture", "declared_capability", "installed_capability", "demonstrated_capability",
		"authority_requirements", "environment_status", "failure_condition", "security_implementation", "customer_private",
		"credential_bearing", "policy_thresholds")
}

func oneOf(value string, allowed ...string) bool {
	for _, candidate := range allowed {
		if value == candidate {
			return true
		}
	}
	return false
}

func nonblank(value string) bool { return strings.TrimSpace(value) != "" }

func capabilityRecordRef(raw json.RawMessage) (*string, error) {
	if len(raw) == 0 {
		return nil, nil
	}
	if bytes.Equal(bytes.TrimSpace(raw), []byte("null")) {
		return nil, fmt.Errorf("capability_record_ref cannot be null when present")
	}
	var value string
	if err := json.Unmarshal(raw, &value); err != nil {
		return nil, err
	}
	if !nonblank(value) {
		return nil, fmt.Errorf("capability_record_ref cannot be empty")
	}
	return &value, nil
}
