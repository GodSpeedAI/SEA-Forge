package server

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"strings"
	"unicode/utf8"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const maxAskBodyBytes = 8 << 10

type askRequestBody struct {
	Kind    json.RawMessage `json:"kind"`
	Subject json.RawMessage `json:"subject"`
	Purpose json.RawMessage `json:"purpose"`
	Case    json.RawMessage `json:"case"`
}

func (s *Server) handleAsk(w http.ResponseWriter, r *http.Request) {
	if s.opts.Ask == nil {
		writeTypedError(w, http.StatusServiceUnavailable, "unavailable", "the governed Ask service is not configured")
		return
	}
	if !requireJSON(w, r) {
		return
	}
	question, ok := decodeAskBody(w, r)
	if !ok {
		return
	}
	actor := sessionIdentityOf(r).Claim()
	if actor.ActorID == "" || actor.Role == "" {
		writeTypedError(w, http.StatusUnauthorized, "unauthorized", "a verified session identity is required")
		return
	}
	if !s.verifySessionPerspective(w, r, actor) {
		return
	}
	answer, err := s.opts.Ask.Ask(r.Context(), actor, question)
	if err != nil {
		writeAskError(w, err)
		return
	}
	view, err := thothAnswerViewOf(answer)
	if err != nil {
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the governed Ask service returned an incomplete or unsupported disclosure")
		return
	}
	writeJSON(w, http.StatusOK, view)
}

func decodeAskBody(w http.ResponseWriter, r *http.Request) (ports.AskQuestion, bool) {
	raw, err := io.ReadAll(io.LimitReader(r.Body, maxAskBodyBytes+1))
	if err != nil {
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body could not be read")
		return ports.AskQuestion{}, false
	}
	if len(raw) > maxAskBodyBytes {
		writeTypedError(w, http.StatusRequestEntityTooLarge, "invalid", "request body exceeds 8 KiB")
		return ports.AskQuestion{}, false
	}
	if !utf8.Valid(raw) {
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body must contain valid UTF-8")
		return ports.AskQuestion{}, false
	}
	trimmed := bytes.TrimSpace(raw)
	if len(trimmed) == 0 || trimmed[0] != '{' {
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body must be one JSON object")
		return ports.AskQuestion{}, false
	}
	dec := json.NewDecoder(bytes.NewReader(raw))
	dec.DisallowUnknownFields()
	var body askRequestBody
	if err := dec.Decode(&body); err != nil {
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body is not a valid Ask object")
		return ports.AskQuestion{}, false
	}
	var extra json.RawMessage
	if err := dec.Decode(&extra); err != io.EOF {
		writeTypedError(w, http.StatusBadRequest, "invalid", "request body must contain one JSON object and optional trailing whitespace")
		return ports.AskQuestion{}, false
	}
	kind, ok := askString(body.Kind)
	if !ok || !oneOfString(kind, contract.AllThothQuestionKinds) {
		writeTypedError(w, http.StatusBadRequest, "invalid", "kind must be one of the supported Thoth question kinds")
		return ports.AskQuestion{}, false
	}
	subject, ok := askString(body.Subject)
	if !ok || strings.TrimSpace(subject) == "" {
		writeTypedError(w, http.StatusBadRequest, "invalid", "subject must be a non-empty string")
		return ports.AskQuestion{}, false
	}
	purpose := "planning"
	if len(body.Purpose) != 0 {
		purpose, ok = askString(body.Purpose)
		if !ok {
			writeTypedError(w, http.StatusBadRequest, "invalid", "purpose must be a string when supplied")
			return ports.AskQuestion{}, false
		}
	}
	if !utf8.ValidString(purpose) || len(purpose) > 500 {
		writeTypedError(w, http.StatusBadRequest, "invalid", "purpose must not exceed 500 UTF-8 bytes")
		return ports.AskQuestion{}, false
	}
	var caseRef *ports.CaseRef
	if len(body.Case) != 0 {
		caseID, ok := askString(body.Case)
		if !ok || strings.TrimSpace(caseID) == "" {
			writeTypedError(w, http.StatusBadRequest, "invalid", "case must be a non-empty string when supplied")
			return ports.AskQuestion{}, false
		}
		ref := ports.CaseRef(caseID)
		caseRef = &ref
	}
	return ports.AskQuestion{Kind: kind, Subject: subject, Purpose: purpose, Case: caseRef}, true
}

func askString(raw json.RawMessage) (string, bool) {
	if len(raw) == 0 || bytes.Equal(bytes.TrimSpace(raw), []byte("null")) {
		return "", false
	}
	var value string
	if err := json.Unmarshal(raw, &value); err != nil {
		return "", false
	}
	return value, true
}

func oneOfString(value string, allowed []string) bool {
	for _, candidate := range allowed {
		if value == candidate {
			return true
		}
	}
	return false
}

func thothAnswerViewOf(answer ports.AskAnswer) (contract.ThothAnswerView, error) {
	if !nonblankAsk(answer.AnswerID) || !nonblankAsk(answer.QuestionID) || !nonblankAsk(answer.SnapshotRef) ||
		!nonblankAsk(answer.Assurance) || !nonblankAsk(answer.AuthorityNotice) || !nonblankAsk(answer.AnsweredAt) ||
		!oneOfString(answer.Disposition, contract.AllThothDispositions) || !oneOfString(answer.Freshness, contract.AllThothFreshnessValues) ||
		answer.Claims == nil || answer.OmittedClaimClasses == nil || answer.Limitations == nil {
		return contract.ThothAnswerView{}, apperr.New(apperr.KindUnavailable, "", "ask", "Ask answer is missing required fields or contains unsupported enums")
	}
	view := contract.ThothAnswerView{
		AnswerID: answer.AnswerID, QuestionID: answer.QuestionID, Disposition: contract.ThothDisposition(answer.Disposition),
		Claims: make([]contract.ThothClaimView, len(answer.Claims)), OmittedClaimClasses: make([]contract.ThothClaimClass, len(answer.OmittedClaimClasses)),
		SnapshotRef: answer.SnapshotRef, Freshness: contract.ThothFreshness(answer.Freshness), Assurance: answer.Assurance,
		Limitations: append([]string{}, answer.Limitations...), AuthorityNotice: answer.AuthorityNotice, AnsweredAt: answer.AnsweredAt,
	}
	for i, class := range answer.OmittedClaimClasses {
		if !oneOfString(class, contract.AllThothClaimClasses) {
			return contract.ThothAnswerView{}, apperr.New(apperr.KindUnavailable, "", "ask", "Ask answer contains an unsupported omitted claim class")
		}
		view.OmittedClaimClasses[i] = contract.ThothClaimClass(class)
	}
	for i, claim := range answer.Claims {
		if !nonblankAsk(claim.ClaimID) || !nonblankAsk(claim.Subject) || !nonblankAsk(claim.Statement) || !nonblankAsk(claim.SnapshotRef) ||
			claim.EvidenceRefs == nil || claim.SettlementRefs == nil || !oneOfString(claim.ClaimClass, contract.AllThothClaimClasses) ||
			!oneOfString(claim.Status, contract.AllThothClaimStatuses) || claim.CapabilityRecordRef != nil && !nonblankAsk(*claim.CapabilityRecordRef) {
			return contract.ThothAnswerView{}, apperr.New(apperr.KindUnavailable, "", "ask", "Ask answer contains an incomplete or unsupported claim")
		}
		view.Claims[i] = contract.ThothClaimView{
			ClaimID: claim.ClaimID, ClaimClass: contract.ThothClaimClass(claim.ClaimClass), Subject: claim.Subject,
			Status: contract.ThothClaimStatus(claim.Status), Statement: claim.Statement, SnapshotRef: claim.SnapshotRef,
			EvidenceRefs: append([]string{}, claim.EvidenceRefs...), SettlementRefs: append([]string{}, claim.SettlementRefs...),
			CapabilityRecordRef: cloneAskString(claim.CapabilityRecordRef),
		}
	}
	return view, nil
}

func nonblankAsk(value string) bool { return strings.TrimSpace(value) != "" }

func cloneAskString(value *string) *string {
	if value == nil {
		return nil
	}
	copy := *value
	return &copy
}

func writeAskError(w http.ResponseWriter, err error) {
	switch apperr.KindOf(err) {
	case apperr.KindAuthorityDenied:
		writeTypedError(w, http.StatusForbidden, "authority_denied", "the kernel refused this Ask request")
	case apperr.KindInvalid:
		writeTypedError(w, http.StatusBadRequest, "invalid", "the governed Ask request was refused")
	default:
		writeTypedError(w, http.StatusBadGateway, "unavailable", "the governed Ask service could not complete this request")
	}
}
