// Package sfwp is the live adapter for the governed case authority: an NDJSON client for the
// kernel's Unix-socket protocol (plan casework-live-wiring-production T05, resolves SEAM-4).
//
// It mirrors the reference client (workbench/apps/desktop/src-tauri/src/{bridge,socket}.rs):
// verb-tagged snake_case request objects, an `actor` block (and optional `on_behalf_of` sibling)
// on protected verbs, request_id correlation, request.get_status recovery after a reconnect, and
// the authority's error classes preserved verbatim rather than flattened into strings. It does not
// invent a second dialect: the golden-frame tests pin this codec to frames recorded from the real
// server.
//
// Wire facts this package relies on (verified against crates/sea-forge-server/src/lib.rs):
//   - A connection is a line loop: the server reads one request line, dispatches it, awaits the
//     handler, and writes exactly one response line before reading the next request. Responses
//     carry no sequence id; pairing is positional.
//   - Unsolicited `{"type":"event","event":{...}}` lines are pushed only on connections that
//     issued events.subscribe, interleaved between responses.
//   - Errors are objects with an "error" string and (usually) an "error_class"; identity refusals
//     additionally carry "no_side_effect" and "next_lawful_action".
//   - A verb an older server does not know fails serde deserialization and comes back as
//     `{"error":"bad request: unknown variant ..."}` with NO error_class; this codec maps that to
//     a typed unavailable error rather than retrying.
package sfwp

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// ProtocolVersion is the protocol major this client negotiates through system.hello.
const ProtocolVersion = "1"

// Actor is the identity block a protected request carries: the claim the authority verifies
// (never trusted - the authority derives the OS principal from the socket itself).
type Actor struct {
	ActorID string
	Role    string
}

func (a Actor) wire() map[string]any {
	return map[string]any{"actor_id": a.ActorID, "role": a.Role}
}

// Governance is who a protected operation runs for: the gateway's own claim, plus optionally the
// end user it speaks for (on_behalf_of, an optional SIBLING of actor on the raw request line -
// the delegation is never expressed by reshaping the actor block).
type Governance struct {
	Actor      Actor
	OnBehalfOf *Actor
}

func (g Governance) apply(body map[string]any) {
	body["actor"] = g.Actor.wire()
	if g.OnBehalfOf != nil {
		body["on_behalf_of"] = g.OnBehalfOf.wire()
	}
}

// Request is one outbound protocol request. The body is a plain map so encoding is deterministic
// (encoding/json sorts map keys) and additive: a new verb is a new builder, never a reshaped old
// one.
type Request struct {
	verb string
	body map[string]any
}

func newRequest(verb string) *Request {
	return &Request{verb: verb, body: map[string]any{}}
}

// Verb is the wire verb tag (e.g. "case_commit"); the dotted logical name is "case.commit".
func (r *Request) Verb() string { return r.verb }

// RequestID reports the caller correlation id, or "" when the request carries none.
func (r *Request) RequestID() string {
	if r == nil {
		return ""
	}
	id, _ := r.body["request_id"].(string)
	return id
}

// IsMutation reports whether this verb participates in request-ID correlation, deduplication, and
// request.get_status recovery. Ask is protected and record-writing but deliberately uncorrelated.
func (r *Request) IsMutation() bool {
	_, ok := mutationVerbs[r.verb]
	return ok
}

// IsTransportRetrySafe reports whether an ambiguous transport failure may repeat this request.
// Ask writes a durable disclosure question but has no request_id, so it is neither safe to resend
// nor recoverable through request.get_status. An explicit server_busy refusal remains retryable
// in Client.Do because it proves the request was not admitted.
func (r *Request) IsTransportRetrySafe() bool {
	return !r.IsMutation() && r.verb != "ask"
}

// mutationVerbs is the correlation-tracked durable-locator set for the verbs this client speaks
// (the server's `requires_durable_locator` match in crates/sea-forge-server/src/lib.rs).
var mutationVerbs = map[string]bool{
	"submit":              true,
	"approve":             true,
	"reject":              true,
	"agent_probe":         true,
	"delegate":            true,
	"cancel_delegation":   true,
	"case_commit":         true,
	"approval_decide":     true,
	"case_add_item":       true,
	"case_reopen":         true,
	"case_terminate":      true,
	"case_advance":        true,
	"item_execute":        true,
	"human_task_complete": true,
}

// EncodeRequest renders the request as its NDJSON line (without the trailing newline).
func EncodeRequest(r *Request) ([]byte, error) {
	body := make(map[string]any, len(r.body)+1)
	for k, v := range r.body {
		body[k] = v
	}
	body["verb"] = r.verb
	line, err := json.Marshal(body)
	if err != nil {
		return nil, apperr.Wrap(apperr.KindInternal, "", "encode", "cannot serialize "+r.verb+" request", err)
	}
	return line, nil
}

// ---------------------------------------------------------------------------
// Request builders (one per verb; field names mirror the server's Request enum)
// ---------------------------------------------------------------------------

// NewSystemHello negotiates the protocol version and lists implemented methods.
func NewSystemHello(client string) *Request {
	r := newRequest("system_hello")
	r.body["protocol_version"] = ProtocolVersion
	if client != "" {
		r.body["client"] = client
	}
	return r
}

// NewSystemDescribe asks for the method catalog with interaction classes.
func NewSystemDescribe() *Request { return newRequest("system_describe") }

// NewRequestGetStatus recovers the outcome of a prior correlated request.
func NewRequestGetStatus(requestID string) *Request {
	r := newRequest("request_get_status")
	r.body["request_id"] = requestID
	return r
}

// NewReadinessGet asks for the authority's readiness projection.
func NewReadinessGet() *Request { return newRequest("readiness_get") }

// NewIdentityGet asks which actors this connection may claim. An optional governance block makes
// the inspect report the actor a delegated request would be attributed to (it is still an inspect:
// no actor is required, nothing is decided).
func NewIdentityGet(g *Governance) *Request {
	r := newRequest("identity_get")
	if g != nil {
		g.apply(r.body)
	}
	return r
}

// NewEventsSubscribe subscribes this connection to live events, replaying strictly after fromCursor
// when it is non-empty.
func NewEventsSubscribe(fromCursor string) *Request {
	r := newRequest("events_subscribe")
	if fromCursor != "" {
		r.body["from_cursor"] = fromCursor
	}
	return r
}

// NewEventsUnsubscribe drops this connection's live subscription.
func NewEventsUnsubscribe() *Request { return newRequest("events_unsubscribe") }

// NewEventsGetRange is the deterministic bounded read for gap recovery.
func NewEventsGetRange(fromCursor, toCursor string, limit int) *Request {
	r := newRequest("events_get_range")
	if fromCursor != "" {
		r.body["from_cursor"] = fromCursor
	}
	if toCursor != "" {
		r.body["to_cursor"] = toCursor
	}
	if limit > 0 {
		r.body["limit"] = limit
	}
	return r
}

// NewCaseEntryOptions lists the templates a case may be committed from.
func NewCaseEntryOptions() *Request { return newRequest("case_entry_options") }

// NewCasePreflight is the audit-only dry run of a draft.
func NewCasePreflight(templateRef string, params map[string]string) *Request {
	r := newRequest("case_preflight")
	r.body["template_ref"] = templateRef
	if len(params) > 0 {
		r.body["params"] = params
	}
	return r
}

// PreconditionWire pins record bytes (a template digest) a commit must still match.
type PreconditionWire struct {
	Records []RecordDigestWire
}

type RecordDigestWire struct {
	Ref            string
	ExpectedDigest string
}

func (p *PreconditionWire) wire() map[string]any {
	records := make([]any, 0, len(p.Records))
	for _, rec := range p.Records {
		records = append(records, map[string]any{"ref": rec.Ref, "expected_digest": rec.ExpectedDigest})
	}
	return map[string]any{"records": records}
}

// NewCaseCommit commits a case from a template. The authority requires a non-empty request_id
// before admission (durable_locator_required otherwise), so an empty one is refused here rather
// than on the wire. `entity` is set to the verified actor id, mirroring the reference bridge: the
// recorded principal may never disagree with the verified one.
func NewCaseCommit(templateRef string, params map[string]string, policy, process string, timeoutSec int, requestID string, pin *PreconditionWire, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "case_commit",
			"a durable case commit requires a non-empty request_id before it can be sent")
	}
	r := newRequest("case_commit")
	r.body["template_ref"] = templateRef
	if len(params) > 0 {
		r.body["params"] = params
	}
	if policy != "" {
		r.body["policy"] = policy
	}
	if process != "" {
		r.body["process"] = process
	}
	if timeoutSec > 0 {
		r.body["timeout"] = timeoutSec
	}
	r.body["request_id"] = requestID
	if pin != nil {
		r.body["preconditions"] = pin.wire()
	}
	// entity must name the verified actor or the authority refuses the mismatch.
	r.body["entity"] = effectiveActorID(g)
	g.apply(r.body)
	return r, nil
}

// NewApprovalList lists decisions awaiting a resolver, scoped to one case when caseID is set.
func NewApprovalList(caseID string) *Request {
	r := newRequest("approval_list")
	if caseID != "" {
		r.body["case_id"] = caseID
	}
	return r
}

// NewApprovalDecide resolves one approval ("approve"/"reject"). Correlated like every mutation.
func NewApprovalDecide(caseID, approvalID, decision, note, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "approval_decide",
			"a durable approval decision requires a non-empty request_id before it can be sent")
	}
	r := newRequest("approval_decide")
	r.body["case_id"] = caseID
	r.body["approval_id"] = approvalID
	r.body["decision"] = decision
	if note != "" {
		r.body["note"] = note
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewCaseList lists committed cases.
func NewCaseList() *Request { return newRequest("case_list") }

// NewRunList lists the governed run records (episodes) the cell holds, newest first. Runs whose
// directory no case claims are reported by the authority rather than dropped.
func NewRunList() *Request { return newRequest("run_list") }

// NewRunListForCase lists the governed run records (episodes) captured for one case.
func NewRunListForCase(caseID string) (*Request, error) {
	if strings.TrimSpace(caseID) == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "run_list", "case id must not be blank")
	}
	r := newRequest("run_list")
	r.body["case_id"] = caseID
	return r, nil
}

// NewRunGet resolves one run's committed records, including its case, plan item, trace, and evidence.
func NewRunGet(runID string) *Request {
	r := newRequest("run_get")
	r.body["run_id"] = runID
	return r
}

// NewCaseGetOverview projects one case's committed records.
func NewCaseGetOverview(caseID string) *Request {
	r := newRequest("case_get_overview")
	r.body["case_id"] = caseID
	return r
}

// NewCaseGetHorizon projects one case's derived work horizon.
func NewCaseGetHorizon(caseID string) *Request {
	r := newRequest("case_get_horizon")
	r.body["case_id"] = caseID
	return r
}

// NewCaseAddItem proposes a discretionary item. The server overwrites proposed_by with the
// verified actor and validates the proposal (dependency cycles included) before any write.
func NewCaseAddItem(caseID string, item map[string]any, policy, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "case_add_item",
			"a durable item proposal requires a non-empty request_id before it can be sent")
	}
	r := newRequest("case_add_item")
	r.body["case_id"] = caseID
	r.body["item"] = item
	if policy != "" {
		r.body["policy"] = policy
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewCaseReopen reopens a terminated or completed case; a non-blank reason is recorded on the
// kernel's CaseReopened event.
func NewCaseReopen(caseID, reason, policy, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "case_reopen",
			"a durable case reopen requires a non-empty request_id before it can be sent")
	}
	r := newRequest("case_reopen")
	r.body["case_id"] = caseID
	if strings.TrimSpace(reason) != "" {
		r.body["reason"] = reason
	}
	if policy != "" {
		r.body["policy"] = policy
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewCaseTerminate terminates a case with a mandatory recorded reason.
func NewCaseTerminate(caseID, reason, policy, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "case_terminate",
			"a durable case termination requires a non-empty request_id before it can be sent")
	}
	r := newRequest("case_terminate")
	r.body["case_id"] = caseID
	r.body["reason"] = reason
	if policy != "" {
		r.body["policy"] = policy
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewCaseAdvance advances a case: computes ready actions and executes enabled sandboxed items.
func NewCaseAdvance(caseID, policy string, timeoutSec int, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "case_advance",
			"a durable case advance requires a non-empty request_id before it can be sent")
	}
	r := newRequest("case_advance")
	r.body["case_id"] = caseID
	if policy != "" {
		r.body["policy"] = policy
	}
	if timeoutSec > 0 {
		r.body["timeout"] = timeoutSec
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewItemExecute scopes the advance engine to exactly one item.
func NewItemExecute(caseID, itemID, policy string, timeoutSec int, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "item_execute",
			"a durable item execution requires a non-empty request_id before it can be sent")
	}
	r := newRequest("item_execute")
	r.body["case_id"] = caseID
	r.body["item_id"] = itemID
	if policy != "" {
		r.body["policy"] = policy
	}
	if timeoutSec > 0 {
		r.body["timeout"] = timeoutSec
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewHumanTaskComplete records a governed human-task completion. A judgment with no recorded
// justification is not a governed completion, so it is mandatory here as it is on the server.
func NewHumanTaskComplete(caseID, itemID, justification, policy, requestID string, g Governance) (*Request, error) {
	if requestID == "" {
		return nil, apperr.New(apperr.KindInvalid, "", "human_task_complete",
			"a durable human-task completion requires a non-empty request_id before it can be sent")
	}
	r := newRequest("human_task_complete")
	r.body["case_id"] = caseID
	r.body["item_id"] = itemID
	r.body["justification"] = justification
	if policy != "" {
		r.body["policy"] = policy
	}
	r.body["request_id"] = requestID
	g.apply(r.body)
	return r, nil
}

// NewArtifactGet fetches one content-addressed artifact by SHA-256 digest (bounded server-side).
func NewArtifactGet(digest string) *Request {
	r := newRequest("artifact_get")
	r.body["digest"] = digest
	return r
}

// effectiveActorID names the principal the ledger records: the delegated end user when the
// gateway speaks for one, else the gateway's own claim.
func effectiveActorID(g Governance) string {
	if g.OnBehalfOf != nil {
		return g.OnBehalfOf.ActorID
	}
	return g.Actor.ActorID
}

// ---------------------------------------------------------------------------
// Response decoding
// ---------------------------------------------------------------------------

// Response is one decoded inbound line. Err carries the authority's refusal (with its class
// preserved) when the frame is an error; otherwise Raw holds the whole result object for the
// verb-specific view to unmarshal.
type Response struct {
	Raw json.RawMessage
	Err *Refusal
}

// Into unmarshals the result body into v. It is an error to call it on an error frame.
func (resp *Response) Into(v any) error {
	if resp.Err != nil {
		return resp.Err.appErr("decode")
	}
	if err := json.Unmarshal(resp.Raw, v); err != nil {
		return apperr.Wrap(apperr.KindInternal, "", "decode", "cannot decode response body", err)
	}
	return nil
}

// Refusal is an authority refusal (or any wire error), with the operator-facing class kept
// verbatim. NoSideEffect states what the authority asserted about side effects; an unknown-verb
// parse failure from an older server synthesizes class "" with UnknownVerb set.
type Refusal struct {
	Class            string
	Message          string
	NoSideEffect     bool
	NextLawfulAction string
	// UnknownVerb marks the "bad request: unknown variant" parse failure an older server returns
	// for a verb it predates: typed as unavailable, never retried.
	UnknownVerb bool
}

// RefusalClass exposes the authority's own error class without string-parsing prose. It lets
// application-layer refusers (internal/intents) map kernel classes onto the typed refusal
// vocabulary through the ports.ClassRefusal interface without importing this adapter.
func (e *Refusal) RefusalClass() string { return e.Class }

func (e *Refusal) Error() string {
	if e.Class == "" {
		return e.Message
	}
	return fmt.Sprintf("%s: %s", e.Class, e.Message)
}

// appErr maps the refusal onto the application's error kinds. The Refusal stays in the chain so a
// caller can errors.As it and read the class without parsing prose.
func (e *Refusal) appErr(op string) *apperr.Error {
	kind := e.kind()
	msg := e.Message
	if e.UnknownVerb {
		msg = "the authority did not recognize this operation (likely an older server): " + e.Message
	}
	wrapped := apperr.New(kind, "", op, msg)
	// The refusal always stays in the chain: a caller switches on Kind for control flow and may
	// still errors.As the class, the no-side-effect assertion, or the next lawful action.
	wrapped.Err = e
	return wrapped
}

func (e *Refusal) kind() apperr.Kind {
	if e.UnknownVerb {
		return apperr.KindUnavailable
	}
	switch e.Class {
	case "identity_required", "identity_unverifiable", "identity_unconfigured", "identity_not_bound",
		"identity_role_not_held", "identity_entity_mismatch", "identity_delegation_refused",
		"separation_of_duty", "separation_of_duty_unverifiable":
		return apperr.KindAuthorityDenied
	case "unsupported_version", "server_busy", "request_cancelled", "request_interrupted":
		return apperr.KindUnavailable
	case "input_error", "invalid_decision", "unsafe_path_error", "durable_locator_required",
		"request_id_reused", "events_unknown_cursor":
		return apperr.KindInvalid
	default:
		return apperr.KindInternal
	}
}

// transportRefusalKind are classes that mean "the operation could not even be attempted"; the
// recovery loop treats them as retryable-with-backoff rather than terminal refusals.
func transportRefusalKind(class string) bool {
	switch class {
	case "server_busy":
		return true
	}
	return false
}

// DecodeResponse decodes one non-event inbound line into a Response.
func DecodeResponse(line []byte) (*Response, error) {
	var probe struct {
		Type  string          `json:"type"`
		Error *string         `json:"error"`
		Class string          `json:"error_class"`
		NoEff bool            `json:"no_side_effect"`
		Next  string          `json:"next_lawful_action"`
		Raw   json.RawMessage `json:"-"`
	}
	if err := json.Unmarshal(line, &probe); err != nil {
		return nil, apperr.Wrap(apperr.KindInternal, "", "decode", "response line is not a JSON object", err)
	}
	if probe.Type == "event" {
		return nil, apperr.New(apperr.KindInternal, "", "decode",
			"an event frame arrived on a request connection; event frames exist only on subscribe connections")
	}
	if probe.Error != nil {
		refusal := &Refusal{
			Class:            probe.Class,
			Message:          *probe.Error,
			NoSideEffect:     probe.NoEff,
			NextLawfulAction: probe.Next,
		}
		// An older server that does not know a verb fails serde deserialization and answers
		// {"error":"bad request: unknown variant ..."} with no class. That is an availability
		// fact about the server, typed as such, never retried.
		if probe.Class == "" && strings.HasPrefix(*probe.Error, "bad request:") {
			refusal.UnknownVerb = true
		}
		return &Response{Raw: json.RawMessage(line), Err: refusal}, nil
	}
	return &Response{Raw: json.RawMessage(line)}, nil
}

// ---------------------------------------------------------------------------
// Typed result views (adapter-local mirrors of the authority's result bodies)
// ---------------------------------------------------------------------------

// HelloView mirrors system.hello's result.
type HelloView struct {
	ProtocolVersion       string   `json:"protocol_version"`
	ServerProtocolVersion string   `json:"server_protocol_version"`
	ImplementedMethods    []string `json:"implemented_methods"`
}

// RequestStatusView mirrors request.get_status's result. Status is pending | completed | failed |
// unknown; Outcome is the recorded terminal response when one exists.
type RequestStatusView struct {
	RequestID string          `json:"request_id"`
	Status    string          `json:"status"`
	Method    string          `json:"method"`
	Outcome   json.RawMessage `json:"outcome"`
}

// SubscribeAckView mirrors events.subscribe's acknowledgement.
type SubscribeAckView struct {
	OK         bool `json:"ok"`
	Subscribed bool `json:"subscribed"`
	Replayed   int  `json:"replayed"`
}

// TemplateParameterView is one declared template parameter.
type TemplateParameterView struct {
	ParamType string  `json:"param_type"`
	Required  bool    `json:"required"`
	Default   *string `json:"default"`
}

// TemplateOptionView is one committable template.
type TemplateOptionView struct {
	TemplateRef string                           `json:"template_ref"`
	Description string                           `json:"description"`
	Parameters  map[string]TemplateParameterView `json:"parameters"`
}

type EntryOptionsView struct {
	Templates []TemplateOptionView `json:"templates"`
}

type PlanItemSummaryView struct {
	PlanItemID string `json:"plan_item_id"`
	Name       string `json:"name"`
	ItemKind   string `json:"item_kind"`
}

type RecordDigestView struct {
	Ref            string `json:"ref"`
	ExpectedDigest string `json:"expected_digest"`
}

// PreflightView mirrors case.preflight's result.
type PreflightView struct {
	OK           bool                  `json:"ok"`
	TemplateRef  string                `json:"template_ref"`
	Errors       []string              `json:"errors"`
	Items        []PlanItemSummaryView `json:"items"`
	Precondition *RecordDigestView     `json:"precondition"`
}

// CommitView mirrors case.commit's result body.
type CommitView struct {
	CaseID   string `json:"case_id"`
	State    string `json:"state"`
	ExitCode int    `json:"exit_code"`
}

// CaseSummaryView mirrors one case.list row.
type CaseSummaryView struct {
	CaseID      string  `json:"case_id"`
	CaseState   string  `json:"case_state"`
	Summary     string  `json:"summary"`
	CreatedAt   string  `json:"created_at"`
	ClosedAt    *string `json:"closed_at"`
	RunCount    int     `json:"run_count"`
	CloseReason *string `json:"close_reason"`
}

type CaseListView struct {
	Cases      []CaseSummaryView `json:"cases"`
	Unreadable []string          `json:"unreadable"`
}

// RunSummaryView mirrors one run.list row.
type RunSummaryView struct {
	RunID         string  `json:"run_id"`
	CaseID        *string `json:"case_id"`
	PlanItemID    *string `json:"plan_item_id"`
	Execution     string  `json:"execution"`
	Settlement    string  `json:"settlement"`
	StartedAt     *string `json:"started_at"`
	FinishedAt    *string `json:"finished_at"`
	EvidenceCount int     `json:"evidence_count"`
}

type RunListView struct {
	Runs       []RunSummaryView `json:"runs"`
	Unreadable []string         `json:"unreadable"`
}

// RunArtifactProvenanceView is the narrow run.get projection needed to bind an artifact to its
// committed owner and to the trace event that captured it.
type RunArtifactProvenanceView struct {
	RunID      string                    `json:"run_id"`
	CaseID     *string                   `json:"case_id"`
	PlanItemID *string                   `json:"plan_item_id"`
	Evidence   []RunArtifactEvidenceView `json:"evidence"`
	Trace      []RunArtifactTraceView    `json:"trace"`
}

type RunArtifactEvidenceView struct {
	EvidenceID    string  `json:"evidence_id"`
	Kind          string  `json:"kind"`
	URI           string  `json:"uri"`
	SHA256        *string `json:"sha256"`
	SourceEventID string  `json:"source_event_id"`
}

type RunArtifactTraceView struct {
	EventID    string  `json:"event_id"`
	Kind       string  `json:"kind"`
	PlanItemID *string `json:"plan_item_id"`
}

// RunSettlementView mirrors one settlement row of case.get_overview.
type RunSettlementView struct {
	RunID          string   `json:"run_id"`
	Status         string   `json:"status"`
	Basis          []string `json:"basis"`
	ReviewRequired bool     `json:"review_required"`
	SettledAt      string   `json:"settled_at"`
}

// CaseOverviewView mirrors case.get_overview's result.
type CaseOverviewView struct {
	CaseID      string              `json:"case_id"`
	CaseState   string              `json:"case_state"`
	Summary     string              `json:"summary"`
	CreatedAt   string              `json:"created_at"`
	ClosedAt    *string             `json:"closed_at"`
	CloseReason *string             `json:"close_reason"`
	PlanRef     *string             `json:"plan_ref"`
	TemplateRef *string             `json:"template_ref"`
	ItemCount   int                 `json:"item_count"`
	Stages      []string            `json:"stages"`
	Settlements []RunSettlementView `json:"settlements"`
	RunIDs      []string            `json:"run_ids"`
}

// HorizonItemView mirrors one case.get_horizon row.
type HorizonItemView struct {
	PlanItemID  string   `json:"plan_item_id"`
	Name        string   `json:"name"`
	ItemKind    string   `json:"item_kind"`
	Execution   string   `json:"execution"`
	Settlement  string   `json:"settlement"`
	ParentStage *string  `json:"parent_stage"`
	DependsOn   []string `json:"depends_on"`
	LastEventAt *string  `json:"last_event_at"`
	RunIDs      []string `json:"run_ids"`
}

type CaseHorizonView struct {
	CaseID       string            `json:"case_id"`
	CaseState    string            `json:"case_state"`
	Items        []HorizonItemView `json:"items"`
	LastEventID  *string           `json:"last_event_id"`
	EventsFolded int               `json:"events_folded"`
}

// PendingApprovalView mirrors one approval.list row.
type PendingApprovalView struct {
	ApprovalID  string  `json:"approval_id"`
	CaseID      string  `json:"case_id"`
	RunID       string  `json:"run_id"`
	PlanItemID  string  `json:"plan_item_id"`
	DecisionID  string  `json:"decision_id"`
	RequestedAt string  `json:"requested_at"`
	ExpiresAt   string  `json:"expires_at"`
	Expired     bool    `json:"expired"`
	CriteriaRef *string `json:"criteria_ref"`
}

type ApprovalListView struct {
	Approvals  []PendingApprovalView `json:"approvals"`
	Unreadable *string               `json:"unreadable"`
}

// AddItemView mirrors case.add_item's result.
type AddItemView struct {
	OK         bool   `json:"ok"`
	CaseID     string `json:"case_id"`
	PlanItemID string `json:"plan_item_id"`
	ProposedBy string `json:"proposed_by"`
}

// ReopenView mirrors case.reopen's result.
type ReopenView struct {
	OK        bool   `json:"ok"`
	CaseID    string `json:"case_id"`
	CaseState string `json:"case_state"`
}

// TerminateView mirrors case.terminate's result.
type TerminateView struct {
	OK          bool   `json:"ok"`
	CaseID      string `json:"case_id"`
	CaseState   string `json:"case_state"`
	CloseReason string `json:"close_reason"`
}

// EpisodeSummaryView mirrors one executed episode of case.advance / item.execute.
type EpisodeSummaryView struct {
	ItemID           string `json:"item_id"`
	RunID            string `json:"run_id"`
	SettlementStatus string `json:"settlement_status"`
}

// AdvanceView mirrors case.advance / item.execute's result.
type AdvanceView struct {
	OK       bool                 `json:"ok"`
	CaseID   string               `json:"case_id"`
	State    string               `json:"state"`
	Episodes []EpisodeSummaryView `json:"episodes"`
}

// HumanTaskCompleteView mirrors human_task.complete's result.
type HumanTaskCompleteView struct {
	OK       bool   `json:"ok"`
	CaseID   string `json:"case_id"`
	ItemID   string `json:"item_id"`
	ExitCode int    `json:"exit_code"`
}

// ArtifactView mirrors artifact.get's result. Content is set for UTF-8 payloads,
// ContentBase64 (standard, with padding) otherwise.
type ArtifactView struct {
	Digest        string  `json:"digest"`
	RunID         string  `json:"run_id"`
	EvidenceID    string  `json:"evidence_id"`
	URI           string  `json:"uri"`
	SizeBytes     uint64  `json:"size_bytes"`
	Content       *string `json:"content"`
	ContentBase64 *string `json:"content_base64"`
}

// Bytes returns the artifact payload.
func (v *ArtifactView) Bytes() ([]byte, error) {
	switch {
	case v.Content != nil:
		return []byte(*v.Content), nil
	case v.ContentBase64 != nil:
		return base64.StdEncoding.DecodeString(*v.ContentBase64)
	default:
		return nil, apperr.New(apperr.KindInternal, "", "artifact_get",
			"the authority returned an artifact view with no content")
	}
}

// AvailableActorView mirrors one identity.get available actor.
type AvailableActorView struct {
	ActorID string   `json:"actor_id"`
	Roles   []string `json:"roles"`
}

// EffectiveActorView mirrors identity.get's delegated-actor projection.
type EffectiveActorView struct {
	ActorID string `json:"actor_id"`
	Role    string `json:"role"`
}

// RefusalView mirrors identity.get's refusal block.
type RefusalView struct {
	ErrorClass       string `json:"error_class"`
	Message          string `json:"message"`
	NoSideEffect     *bool  `json:"no_side_effect"`
	NextLawfulAction string `json:"next_lawful_action"`
}

// IdentityView mirrors identity.get's result.
type IdentityView struct {
	UID            *uint32              `json:"uid"`
	Configured     bool                 `json:"configured"`
	Available      []AvailableActorView `json:"available"`
	EffectiveActor *EffectiveActorView  `json:"effective_actor"`
	Refusal        *RefusalView         `json:"refusal"`
}

// ReadinessItemView mirrors one readiness.get condition.
type ReadinessItemView struct {
	ID               string `json:"id"`
	Name             string `json:"name"`
	Category         string `json:"category"`
	Status           string `json:"status"`
	Reason           string `json:"reason"`
	NextLawfulAction string `json:"next_lawful_action"`
}

// ReadinessView mirrors readiness.get's result.
type ReadinessView struct {
	Overall                 string              `json:"overall"`
	Foundations             []ReadinessItemView `json:"foundations"`
	OperationalCapabilities []ReadinessItemView `json:"operational_capabilities"`
}

// Event is one pushed (or replayed) durable event frame. Cursor is the opaque, monotonic,
// replay-addressable ledger position the client resumes from.
type Event struct {
	Cursor      string          `json:"cursor"`
	Kind        string          `json:"kind"`
	CaseID      string          `json:"case_id"`
	RunID       string          `json:"run_id"`
	Detail      json.RawMessage `json:"detail"`
	CommittedAt string          `json:"committed_at"`
}

// eventEnvelope is the pushed-line shape: {"type":"event","event":{...}}. The single discriminator
// the reference client also uses.
type eventEnvelope struct {
	Type  string `json:"type"`
	Event *Event `json:"event"`
}

// DecodeEvent decodes one pushed event line, reporting ok=false for non-event lines.
func DecodeEvent(line []byte) (*Event, bool, error) {
	var env eventEnvelope
	if err := json.Unmarshal(line, &env); err != nil {
		return nil, false, apperr.Wrap(apperr.KindInternal, "", "decode", "pushed line is not JSON", err)
	}
	if env.Type != "event" || env.Event == nil {
		return nil, false, nil
	}
	return env.Event, true, nil
}
