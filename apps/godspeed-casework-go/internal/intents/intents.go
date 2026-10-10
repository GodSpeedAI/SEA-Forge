// Package intents translates the canonical journey intents (T01's consequential set) onto exactly
// one governed SFWP verb call each, through the T05 CaseAuthorityPort, with the effective actor
// carried as on_behalf_of per operator decision D-2 (T02): the gateway principal is the request's
// actor, the end user rides beside it, and the kernel - never the gateway - verifies both and
// ledger-pronounces the pair.
//
// Refusal contract (T01): a refusal is an OUTCOME (HTTP 200), never a transport error, and every
// refusal carries the typed kind plus a plain-language message. The mapping:
//
//	UNAUTHORIZED_ROLE      the projection's own role filter would not have offered the action
//	                       to this role (a gateway-level guard, enforced BEFORE any kernel call);
//	                       also the kernel's identity_role_not_held, passed through.
//	SOD_VIOLATION          the kernel refused on separation of duty (it compares actors from its
//	                       ledgers; the gateway never guesses at this).
//	AUTHORITY_DENIED       any other kernel identity/authority refusal, class preserved verbatim.
//	STALE_PROJECTION       the intent's cursor no longer matches the target's kernel cursor, or
//	                       the kernel rejected the commit because the pinned preflight digest no
//	                       longer matches the template (its own precondition_failed verdict).
//	JUSTIFICATION_REQUIRED a mandatory justification field was not carried.
//	UNAVAILABLE            the kernel is unreachable, or the intent is honestly unroutable in this
//	                       deployment (ESCALATE_OR_OVERRIDE: no kernel verb routes it before T13).
//	INVALID                malformed shape, unknown payload fields, missing required parameters,
//	                       unknown case/target, or a kernel malformed-proposal class
//	                       (input_error, plan_schema_error; class preserved).
//
// Idempotency is two-layered: the same intent id with a byte-identical canonical body replays the
// recorded outcome in-process; and every mutation is correlated to the kernel with request_id =
// intent_id, so the kernel's own durable correlation store deduplicates and recovers outcomes
// across gateway restarts (request_id_reused on payload drift, recorded outcomes on replay).
package intents

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"time"
	"unicode/utf8"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
)

// CursorSource is the projection's per-case view of the kernel event cursor, maintained by the
// SSE relay from the frames it actually received (the T05 subscription channel). The staleness
// guard compares an intent's client_cursor against it.
type CursorSource interface {
	// CursorForCase reports the newest kernel cursor observed for the case, and whether any has
	// been observed at all.
	CursorForCase(caseID string) (cursor string, ok bool)
	// WaitForCaseAdvance blocks until the case's observed cursor advances strictly past `before`
	// or ctx is done, then returns the newest observed cursor. It exists so a successful
	// mutation's response can carry the post-mutation cursor once the kernel's frame arrives.
	WaitForCaseAdvance(ctx context.Context, caseID, before string) string
}

// CaseLister is the minimal existence check the staleness guard needs for cases the relay has not
// observed yet.
type CaseLister interface {
	ListCases(ctx context.Context) ([]ports.CaseRecord, error)
}

// Options configure the handler.
type Options struct {
	// Gateway is the gateway principal's own claim (the request's actor block). It must match the
	// kernel's configured gateway principal; the kernel refuses delegation otherwise.
	Gateway ports.ActorClaim
	// PolicyRef is the cell-relative authority policy the governed verbs authorize against.
	PolicyRef string
	// ExecutionTimeoutSec bounds the kernel-side execution of advance/execute verbs.
	ExecutionTimeoutSec int
}

// Handler translates intents. Safe for concurrent use.
type Handler struct {
	auth    ports.CaseAuthorityPort
	cursors CursorSource
	lister  CaseLister
	opts    Options

	mu       sync.Mutex
	outcomes map[string]replayRecord
	order    []string // insertion order of outcomes, oldest first (bounded FIFO, T11 review F7)
}

const (
	maxIntentIDLen   = 128
	maxReasonBytes   = 2000
	maxReplayRecords = 20000
)

func validIntentID(id string) bool {
	if id == "" || len(id) > maxIntentIDLen {
		return false
	}
	for i := 0; i < len(id); i++ {
		if id[i] < 0x21 || id[i] > 0x7e {
			return false
		}
	}
	return true
}

// validReason bounds the free-text reopen/terminate reason that is written to the kernel's
// event ledger and trace: bounded size, valid UTF-8, no control characters other than newline/tab.
func validReason(reason string) bool {
	if len(reason) > maxReasonBytes || !utf8.ValidString(reason) {
		return false
	}
	for _, r := range reason {
		if r < 0x20 && r != '\n' && r != '\t' || r == 0x7f {
			return false
		}
	}
	return true
}

type replayRecord struct {
	bodyHash [sha256.Size]byte
	response contract.IntentResponse
}

// NewHandler builds the translator.
func NewHandler(auth ports.CaseAuthorityPort, cursors CursorSource, lister CaseLister, opts Options) *Handler {
	return &Handler{auth: auth, cursors: cursors, lister: lister, opts: opts, outcomes: map[string]replayRecord{}}
}

// Typed refusal kinds (the T01 vocabulary; pinned by contract.AllRefusalKinds and the goldens).
const (
	RefAuthorityDenied     = "AUTHORITY_DENIED"
	RefUnauthorizedRole    = "UNAUTHORIZED_ROLE"
	RefSODViolation        = "SOD_VIOLATION"
	RefStaleProjection     = "STALE_PROJECTION"
	RefJustificationNeeded = "JUSTIFICATION_REQUIRED"
	RefUnavailable         = "UNAVAILABLE"
	RefInvalid             = "INVALID"
)

// templateRefPrefix is the kernel's record-ref spelling for template precondition pins
// (verified live in T06: case.preflight reports precondition.ref "template:<ref>").
const templateRefPrefix = "template:"

// Handle validates, guards, translates and executes one intent. It always returns a response;
// refusals are outcomes.
func (h *Handler) Handle(ctx context.Context, in contract.InteractionIntent) contract.IntentResponse {
	// 1. Envelope shape and idempotency key.
	if in.IntentID == "" {
		return refuse(in, RefInvalid, "intent_id is required")
	}
	// The intent id is the SFWP request_id, the response header and the log correlation id, and
	// the replay-cache key: bound it and keep it free of control bytes (T11 review F7).
	if !validIntentID(in.IntentID) {
		return refuse(contract.InteractionIntent{}, RefInvalid,
			fmt.Sprintf("intent_id must be 1-%d printable ASCII characters", maxIntentIDLen))
	}
	if in.Kind != "CONSEQUENTIAL_CASE" {
		return refuse(in, RefInvalid,
			fmt.Sprintf("intent kind %q is not CONSEQUENTIAL_CASE: only consequential intents cross this gateway", in.Kind))
	}
	if !isConsequentialAction(in.ActionName) {
		return refuse(in, RefInvalid, fmt.Sprintf("action_name %q is not one of the consequential journey intents", in.ActionName))
	}
	if in.Actor.ActorID == "" || in.Actor.Role == "" {
		return refuse(in, RefInvalid, "the intent must carry the acting user (actor_id and role); identity is never guessed here")
	}

	hash := bodyHash(in)
	if rec, ok := h.replay(in.IntentID, hash); ok {
		return rec
	}

	// 2. Projection-level role guard: the same predicates that filter the snapshot's action
	// descriptors. The kernel remains the authority - this guard only keeps the gateway honest
	// about what it offered, and it fires BEFORE any kernel call.
	if !roleWouldOffer(in.ActionName, in.Actor.Role) {
		return h.record(in.IntentID, hash, refuse(in, RefUnauthorizedRole,
			fmt.Sprintf("the action list for role %s does not offer %s on this projection; the gateway will not route it to the kernel", in.Actor.Role, in.ActionName)))
	}

	// 3. Typed payload decode and per-kind mandatory fields. mutates reports whether the mapped
	// verb writes to the kernel (reads and unroutable kinds do not).
	mutates, err := h.validate(in)
	if err != nil {
		return h.record(in.IntentID, hash, refusalFrom(in, err))
	}

	// 4. Staleness guard: a mutating intent carries the projection cursor it acted on; the kernel
	// having moved past it for the target case refuses before any kernel write. PROPOSE_CASE is
	// exempt: its target case does not exist yet, and its freshness is bound by the kernel's own
	// preflight-digest precondition instead.
	caseID := targetCase(in)
	if mutates && caseID != "" && in.ActionName != "PROPOSE_CASE" {
		if refusal, ok := h.stalenessRefusal(ctx, in, caseID); ok {
			return h.record(in.IntentID, hash, refusal)
		}
	}

	// 5. Translate to exactly one governed verb call.
	newCursor, resulting, err := h.execute(ctx, in)
	if err != nil {
		return h.record(in.IntentID, hash, refusalFrom(in, err))
	}
	resp := contract.IntentResponse{IntentID: in.IntentID, Success: true}
	if newCursor != "" {
		resp.NewCursor = &newCursor
	}
	resp.ResultingObject = resulting
	return h.record(in.IntentID, hash, resp)
}

// validate decodes the typed payload and checks the mandatory fields per kind. mutates reports
// whether the mapped kernel verb writes (open reads and unroutable kinds do not).
func (h *Handler) validate(in contract.InteractionIntent) (bool, error) {
	switch in.ActionName {
	case "PROPOSE_CASE":
		var p contract.ProposeCasePayload
		if err := decodePayload(in, &p); err != nil {
			return false, err
		}
		if p.TemplateRef == "" {
			return false, invalid("PROPOSE_CASE requires parameters.template_ref")
		}
		if p.PreflightDigest == "" {
			return false, invalid("PROPOSE_CASE requires parameters.preflight_digest from a passing preflight; run POST /api/templates/preflight first")
		}
		if p.Params == nil {
			return false, invalid("PROPOSE_CASE requires parameters.params (an object, possibly empty)")
		}
		return true, nil
	case "ADD_DISCRETIONARY_WORK":
		var p contract.AddDiscretionaryWorkPayload
		if err := decodePayload(in, &p); err != nil {
			return false, err
		}
		// stage_id is the optional DAG anchor (T06 fix F2): when given, the proposal depends on
		// that plan item so the new work lands after it; when absent the item has no entry
		// sentries. kind defaults to sandboxed_task, justification is never optional.
		if p.CaseID == "" || p.Title == "" {
			return false, invalid("ADD_DISCRETIONARY_WORK requires parameters.case_id and title")
		}
		if p.Justification == "" {
			return false, &refusalError{kind: RefJustificationNeeded, message: "Discretionary work is consequential: state why this work is needed before it can be proposed."}
		}
		return true, nil
	case "EXECUTE_ITEM":
		var p contract.ExecuteItemPayload
		if err := decodePayload(in, &p); err != nil {
			return false, err
		}
		if p.ItemID == "" {
			return false, invalid("EXECUTE_ITEM requires parameters.item_id")
		}
		return true, nil
	case "COMPLETE_HUMAN_TASK":
		var p contract.CompleteHumanTaskPayload
		if err := decodePayload(in, &p); err != nil {
			return false, err
		}
		if p.ItemID == "" {
			return false, invalid("COMPLETE_HUMAN_TASK requires parameters.item_id")
		}
		if p.Justification == "" {
			return false, &refusalError{kind: RefJustificationNeeded, message: "A human-task completion is a governed judgment: a recorded justification is mandatory."}
		}
		return true, nil
	case "APPROVE_HUMAN_TASK", "REJECT_HUMAN_TASK":
		if in.TargetObjectID == "" {
			return false, invalid(in.ActionName + " requires target_object_id (the item the approval gates)")
		}
		if in.Justification == nil || *in.Justification == "" {
			return false, &refusalError{kind: RefJustificationNeeded, message: "An approval decision must record why: the justification is mandatory."}
		}
		return true, nil
	case "REOPEN_CASE", "TERMINATE_CASE":
		var p contract.CaseLifecyclePayload
		if err := decodePayload(in, &p); err != nil {
			return false, err
		}
		if p.CaseID == "" {
			return false, invalid(in.ActionName + " requires parameters.case_id")
		}
		// The T01 golden pins a blank termination reason as INVALID.
		if p.Reason == "" {
			return false, invalid("A " + lowerKind(in.ActionName) + " reason is required and must not be blank.")
		}
		if !validReason(p.Reason) {
			return false, invalid(fmt.Sprintf("A %s reason must be at most %d bytes of text without control characters.", lowerKind(in.ActionName), maxReasonBytes))
		}
		return true, nil
	case "OPEN_ARTIFACT":
		if _, ok := in.Parameters["digest"]; !ok {
			return false, invalid("OPEN_ARTIFACT requires parameters.digest (a content-addressed artifact digest); this kernel's view verbs do not attach artifacts to objects by id")
		}
		return false, nil
	case "ESCALATE_OR_OVERRIDE":
		// Honest unavailability: no kernel verb routes escalation or override before T13 (the
		// plan's extended-journeys task). The gateway fabricates nothing.
		return false, &refusalError{kind: RefUnavailable, message: "No escalation target is configured in this deployment; the request cannot be routed."}
	}
	return false, invalid("unroutable intent " + in.ActionName)
}

// execute performs the mapped verb call and, on success, waits briefly for the relay to observe
// the kernel's frame so the response can carry the post-mutation cursor.
func (h *Handler) execute(ctx context.Context, in contract.InteractionIntent) (string, *contract.CognitiveObject, error) {
	gov := ports.Governance{
		Actor:      h.opts.Gateway,
		OnBehalfOf: &ports.ActorClaim{ActorID: in.Actor.ActorID, Role: in.Actor.Role},
	}
	opts := ports.GovernedOptions{
		Governance: gov,
		Policy:     h.opts.PolicyRef,
		RequestID:  in.IntentID, // the durable locator: the kernel correlates and deduplicates by it
		TimeoutSec: h.opts.ExecutionTimeoutSec,
	}
	caseID := targetCase(in)

	switch in.ActionName {
	case "PROPOSE_CASE":
		var p contract.ProposeCasePayload
		_ = decodePayload(in, &p) // validate() already decoded it successfully
		params := map[string]string{}
		for k, v := range p.Params {
			s, err := asString(v)
			if err != nil {
				return "", nil, invalid(fmt.Sprintf("PROPOSE_CASE parameter %q must be a scalar string: %v", k, err))
			}
			params[k] = s
		}
		receipt, err := h.auth.CommitCase(ctx, ports.CaseDraft{TemplateRef: p.TemplateRef, Params: params},
			ports.PreconditionDigest{Present: true, TemplateRef: templateRefPrefix + p.TemplateRef, ExpectedDigest: p.PreflightDigest}, opts)
		if err != nil {
			return "", nil, err
		}
		// The case did not exist before the commit, so there is no pre-call cursor: the wait is
		// for the new case's FIRST observed frame, which is the commit's own.
		cursor := h.postMutationCursor(ctx, receipt.CaseID, "")
		// T09's case-design journey focuses the new case, so the response must name it
		// (the object id is the case id; kind/name follow the contract's case object).
		obj := &contract.CognitiveObject{
			ID:     receipt.CaseID,
			Kind:   "case",
			Name:   p.TemplateRef,
			Status: "ACTIVE",
			Badge:  "Committed",
		}
		return cursor, obj, nil

	case "ADD_DISCRETIONARY_WORK":
		var p contract.AddDiscretionaryWorkPayload
		_ = decodePayload(in, &p)
		kind, err := kernelItemKind(p.Kind)
		if err != nil {
			return "", nil, err
		}
		// The gateway constructs a minimal VALID plan item (T06 fix F2): the kernel refuses an
		// operation-less sandboxed task with plan_schema_error, so the proposal carries one
		// governed write_file operation whose path is derived from the item name and whose
		// content_hint is the payload's summary/justification body; the settlement criteria
		// require that same path as an artifact, so executing the item settles accepted on the
		// materialized file. The kernel overwrites proposed_by with the verified actor, so the
		// proposer-side separation of duty applies naturally. stage_id anchors the item in the
		// DAG (depends_on, not parent_stage - the anchor may not be a stage-kind item).
		proposal := ports.CaseItemProposal{
			ItemID:       newItemID(),
			Name:         p.Title,
			Kind:         kind,
			SandboxClass: "local",
		}
		if p.StageID != "" {
			proposal.DependsOn = []string{p.StageID}
		}
		if kind == "sandboxed_task" {
			// The kernel forbids operations on non-sandboxed items (stage/milestone), so the
			// governed write only exists where the plan schema allows one.
			path := discretionaryWritePath(p.Title)
			proposal.Operations = []ports.ItemOperation{{
				Kind:        "write_file",
				Path:        path,
				ContentHint: discretionaryContentHint(p),
			}}
			proposal.RequiredArtifacts = []string{path}
		}
		before, _ := h.cursors.CursorForCase(p.CaseID)
		itemID, err := h.auth.AddCaseItem(ctx, ports.CaseRef(p.CaseID), proposal, opts)
		if err != nil {
			return "", nil, err
		}
		obj := proposedObject(itemID, p.Title, kind, p.StageID, in.Actor.ActorID)
		return h.postMutationCursor(ctx, p.CaseID, before), &obj, nil

	case "EXECUTE_ITEM":
		var p contract.ExecuteItemPayload
		_ = decodePayload(in, &p)
		before, _ := h.cursors.CursorForCase(caseID)
		if _, err := h.auth.ExecuteItem(ctx, ports.CaseRef(caseID), p.ItemID, opts); err != nil {
			return "", nil, err
		}
		return h.postMutationCursor(ctx, caseID, before), nil, nil

	case "COMPLETE_HUMAN_TASK":
		var p contract.CompleteHumanTaskPayload
		_ = decodePayload(in, &p)
		before, _ := h.cursors.CursorForCase(caseID)
		if err := h.auth.CompleteHumanTask(ctx, ports.CaseRef(caseID), p.ItemID, p.Justification, opts); err != nil {
			return "", nil, err
		}
		return h.postMutationCursor(ctx, caseID, before), nil, nil

	case "APPROVE_HUMAN_TASK", "REJECT_HUMAN_TASK":
		approvalID, err := h.resolveApproval(ctx, caseID, in.TargetObjectID)
		if err != nil {
			return "", nil, err
		}
		before, _ := h.cursors.CursorForCase(caseID)
		if err := h.auth.DecideApproval(ctx, approvalID, in.ActionName == "APPROVE_HUMAN_TASK", *in.Justification, opts); err != nil {
			return "", nil, err
		}
		return h.postMutationCursor(ctx, caseID, before), nil, nil

	case "REOPEN_CASE":
		var p contract.CaseLifecyclePayload
		_ = decodePayload(in, &p)
		before, _ := h.cursors.CursorForCase(p.CaseID)
		if err := h.auth.ReopenCase(ctx, ports.CaseRef(p.CaseID), p.Reason, opts); err != nil {
			return "", nil, err
		}
		return h.postMutationCursor(ctx, p.CaseID, before), nil, nil

	case "TERMINATE_CASE":
		var p contract.CaseLifecyclePayload
		_ = decodePayload(in, &p)
		before, _ := h.cursors.CursorForCase(p.CaseID)
		if err := h.auth.TerminateCase(ctx, ports.CaseRef(p.CaseID), p.Reason, opts); err != nil {
			return "", nil, err
		}
		return h.postMutationCursor(ctx, p.CaseID, before), nil, nil

	case "OPEN_ARTIFACT":
		digest, _ := in.Parameters["digest"].(string)
		if digest == "" {
			return "", nil, invalid("OPEN_ARTIFACT parameters.digest must be a non-empty artifact digest")
		}
		// The artifact content is fetched to prove reachability and integrity through the
		// governed path; the intent response carries no payload (T08's resolveArtifact serves it).
		if _, err := h.auth.GetArtifact(ctx, digest); err != nil {
			return "", nil, err
		}
		return "", nil, nil
	}
	return "", nil, invalid("unroutable intent " + in.ActionName)
}

// postMutationWait bounds the post-mutation cursor wait (T06 fix F4). The kernel's frame
// normally arrives within milliseconds of the mutation's response; a bound keeps a lost frame
// from hanging the HTTP request past its own lifetime.
const postMutationWait = 5 * time.Second

// postMutationCursor returns the cursor the response should carry as new_cursor: the tracked
// per-case cursor AFTER the mutation, waited for with a strict-advance requirement against the
// pre-call cursor. Passing `before` (the tracked cursor observed before the kernel call) is what
// makes the wait real: without it the relay answers with the current observed cursor, which for
// an already-observed case is the PRE-mutation cursor (T06 fix F4).
func (h *Handler) postMutationCursor(ctx context.Context, caseID, before string) string {
	waitCtx, cancel := context.WithTimeout(ctx, postMutationWait)
	defer cancel()
	return h.cursors.WaitForCaseAdvance(waitCtx, caseID, before)
}

// stalenessRefusal applies the cursor rule. It is a refusal exactly when the kernel has moved
// past the intent's projection cursor in a way that affects the target (any newer kernel frame
// for that case), or when the case cannot be confirmed to exist.
func (h *Handler) stalenessRefusal(ctx context.Context, in contract.InteractionIntent, caseID string) (contract.IntentResponse, bool) {
	if in.ClientCursor == "" {
		return refuse(in, RefInvalid, "a mutating intent must carry client_cursor (the projection cursor it acted on); refetch /api/world and retry"), true
	}
	cursor, ok := h.cursors.CursorForCase(caseID)
	if ok && cursor != in.ClientCursor {
		resp := refuse(in, RefStaleProjection,
			"The kernel has moved past this projection for "+caseID+"; refetch the world and retry with the fresh cursor.")
		resp.Refusal.CurrentCursor = &cursor
		return resp, true
	}
	if !ok {
		// The relay has observed no frame for this case: either the case does not exist or the
		// projection is still catching up. Ask the authority - never assume.
		cases, err := h.lister.ListCases(ctx)
		if err != nil {
			return refuse(in, RefUnavailable, "the kernel view could not be reached to confirm "+caseID+": "+err.Error()), true
		}
		exists := false
		for _, c := range cases {
			if string(c.Ref) == caseID {
				exists = true
				break
			}
		}
		if !exists {
			return refuse(in, RefInvalid, "no case "+caseID+" is known to the kernel; refetch the world"), true
		}
		return refuse(in, RefUnavailable,
			"The projection for "+caseID+" is still syncing; refetch the world and retry."), true
	}
	return contract.IntentResponse{}, false
}

// resolveApproval finds the pending approval record gating an item (APPROVE/REJECT target the
// item object; the kernel's approval id comes from its own inbox view).
func (h *Handler) resolveApproval(ctx context.Context, caseID, itemID string) (string, error) {
	approvals, err := h.auth.PendingApprovals(ctx, ports.CaseRef(caseID))
	if err != nil {
		return "", err
	}
	for _, ap := range approvals {
		if ap.ItemID == itemID && !ap.Expired {
			return ap.ApprovalID, nil
		}
	}
	return "", invalid("nothing is awaiting approval on item " + itemID + " in case " + caseID)
}

// refusalFrom maps an error (a gateway refusalError, or the kernel's verdict through the adapter's
// typed chain) onto the typed refusal vocabulary.
func refusalFrom(in contract.InteractionIntent, err error) contract.IntentResponse {
	var r *refusalError
	if errors.As(err, &r) {
		return refuse(in, r.kind, r.message)
	}
	var typed *apperr.Error
	if errors.As(err, &typed) {
		class := ""
		var cr ports.ClassRefusal
		if errors.As(err, &cr) {
			class = cr.RefusalClass()
		}
		kind, message := mapKernelRefusal(typed, class)
		resp := refuse(in, kind, message)
		if kind == RefStaleProjection {
			// The kernel's own stale verdict (precondition_failed): no fresher cursor exists to
			// offer; the client reruns preflight.
			resp.Refusal.CurrentCursor = nil
		}
		return resp
	}
	return refuse(in, RefUnavailable, "the governed authority could not be reached: "+err.Error())
}

// mapKernelRefusal translates the kernel's verdict (its apperr kind plus verbatim error class)
// onto the typed refusal vocabulary. Unrecognized classes default to AUTHORITY_DENIED with the
// kernel's message preserved: the kernel refused, and its word is carried honestly.
func mapKernelRefusal(typed *apperr.Error, class string) (kind, message string) {
	message = typed.Message
	if class != "" {
		message = class + ": " + message
	}
	switch class {
	case "identity_role_not_held":
		return RefUnauthorizedRole, typed.Message
	case "separation_of_duty", "separation_of_duty_unverifiable", "self_approval":
		return RefSODViolation, typed.Message
	case "precondition_failed":
		return RefStaleProjection,
			"The kernel rejected this commit as stale: the preflight digest no longer matches the template. Run preflight again and commit the fresh digest."
	case "identity_required", "identity_unverifiable", "identity_unconfigured", "identity_not_bound",
		"identity_entity_mismatch", "identity_delegation_refused":
		return RefAuthorityDenied, message
	case "input_error", "invalid_decision", "unsafe_path_error", "durable_locator_required",
		"request_id_reused", "not_found", "record_unreadable", "missing_config_error",
		"plan_schema_error":
		// plan_schema_error is a malformed proposal (T06 fix F5): it belongs with INVALID
		// (malformed shape), not with the authority refusals.
		return RefInvalid, message
	case "unsupported_version", "server_busy", "request_cancelled", "request_interrupted", "events_unknown_cursor":
		return RefUnavailable, message
	}
	switch typed.Kind {
	case apperr.KindInvalid:
		return RefInvalid, message
	case apperr.KindUnavailable:
		return RefUnavailable, message
	case apperr.KindAuthorityDenied:
		return RefAuthorityDenied, message
	}
	return RefAuthorityDenied, message
}

// roleWouldOffer is the guard side of the projection's role filter (projection.roles.go): would
// the action list for this role ever carry this action?
func roleWouldOffer(action, role string) bool {
	if !projection.RoleOffersConsequentialWork(role) {
		return false
	}
	switch action {
	case "APPROVE_HUMAN_TASK", "REJECT_HUMAN_TASK":
		return projection.MayDecideApproval(role)
	case "EXECUTE_ITEM", "COMPLETE_HUMAN_TASK":
		return projection.MayExecute(role)
	case "PROPOSE_CASE", "ADD_DISCRETIONARY_WORK":
		return projection.MayPropose(role)
	case "REOPEN_CASE", "TERMINATE_CASE":
		return projection.MayDriveLifecycle(role)
	case "OPEN_ARTIFACT", "ESCALATE_OR_OVERRIDE":
		return true
	}
	return false
}

// isConsequentialAction reports whether the action name is in the T01 consequential set.
func isConsequentialAction(name string) bool {
	for _, k := range contract.AllIntentKinds {
		if k == name {
			return true
		}
	}
	return false
}

// targetCase resolves the case the intent targets: the typed payload's case id where the payload
// carries one, else the envelope's.
func targetCase(in contract.InteractionIntent) string {
	switch in.ActionName {
	case "ADD_DISCRETIONARY_WORK":
		var p contract.AddDiscretionaryWorkPayload
		if decodePayload(in, &p) == nil {
			return p.CaseID
		}
	case "REOPEN_CASE", "TERMINATE_CASE":
		var p contract.CaseLifecyclePayload
		if decodePayload(in, &p) == nil {
			return p.CaseID
		}
	}
	return in.CaseID
}

// decodePayload round-trips the intent's free-form parameters through the typed payload struct so
// unknown fields and wrong shapes are refused instead of silently dropped.
func decodePayload(in contract.InteractionIntent, v any) error {
	if in.Parameters == nil {
		return invalid("the intent carries no parameters object")
	}
	raw, err := json.Marshal(in.Parameters)
	if err != nil {
		return invalid("the intent parameters are not valid JSON: " + err.Error())
	}
	dec := json.NewDecoder(bytes.NewReader(raw))
	dec.DisallowUnknownFields()
	if err := dec.Decode(v); err != nil {
		return invalid(fmt.Sprintf("the %s parameters do not match the contract payload: %v", in.ActionName, err))
	}
	return nil
}

func bodyHash(in contract.InteractionIntent) [sha256.Size]byte {
	canonical, err := json.Marshal(in)
	if err != nil {
		canonical = []byte(in.IntentID + "\x00" + in.ActionName)
	}
	return sha256.Sum256(canonical)
}

func (h *Handler) replay(id string, hash [sha256.Size]byte) (contract.IntentResponse, bool) {
	h.mu.Lock()
	defer h.mu.Unlock()
	rec, seen := h.outcomes[id]
	if !seen {
		return contract.IntentResponse{}, false
	}
	if rec.bodyHash != hash {
		return refuse(contract.InteractionIntent{IntentID: id}, RefInvalid,
			"intent id was already used with a different body; mint a new intent id"), true
	}
	return rec.response, true // idempotent replay: the recorded outcome stands
}

func (h *Handler) record(id string, hash [sha256.Size]byte, resp contract.IntentResponse) contract.IntentResponse {
	h.mu.Lock()
	if _, seen := h.outcomes[id]; !seen {
		h.order = append(h.order, id)
		if len(h.order) > maxReplayRecords {
			delete(h.outcomes, h.order[0])
			h.order = h.order[1:]
		}
	}
	h.outcomes[id] = replayRecord{bodyHash: hash, response: resp}
	h.mu.Unlock()
	return resp
}

// refusalError is a gateway-level refusal (typed kind + plain message) surfaced as an error so it
// flows through the same mapping point as kernel verdicts.
type refusalError struct {
	kind    string
	message string
}

func (e *refusalError) Error() string { return e.kind + ": " + e.message }

func invalid(message string) error { return &refusalError{kind: RefInvalid, message: message} }

func refuse(in contract.InteractionIntent, kind, message string) contract.IntentResponse {
	resp := contract.IntentResponse{
		IntentID: in.IntentID,
		Success:  false,
		Refusal:  &contract.IntentRefusal{RefusalKind: kind, Message: message},
	}
	code := kind
	msg := message
	resp.ErrorCode = &code
	resp.ErrorMessage = &msg
	return resp
}

// kernelItemKind translates the contract's cognitive object kind onto the kernel's plan-item kind
// vocabulary for a discretionary proposal. An absent kind defaults to sandboxed_task (T06 fix F2):
// it is the one proposed kind that can carry the governed write the proposal constructs.
func kernelItemKind(kind string) (string, error) {
	switch kind {
	case "", "work_item", "discretionary_opportunity", "sandboxed_task":
		return "sandboxed_task", nil
	case "stage":
		return "stage", nil
	case "milestone":
		return "milestone", nil
	default:
		return "", invalid(fmt.Sprintf("kind %q cannot be proposed as discretionary work on this kernel", kind))
	}
}

func asString(v any) (string, error) {
	switch t := v.(type) {
	case string:
		return t, nil
	case float64:
		return json.Number(fmt.Sprintf("%v", t)).String(), nil
	case bool:
		if t {
			return "true", nil
		}
		return "false", nil
	case nil:
		return "", nil
	default:
		return "", fmt.Errorf("unsupported type %T", v)
	}
}

func lowerKind(s string) string {
	out := make([]byte, len(s))
	for i := 0; i < len(s); i++ {
		c := s[i]
		if c >= 'A' && c <= 'Z' {
			c += 'a' - 'A'
		}
		out[i] = c
	}
	return string(out)
}

// newItemID mints a plan-item id for a discretionary proposal. The kernel re-validates the whole
// proposal (cycles included) and overwrites the proposer; the id only has to be unique and
// well-formed (alphanumerics, `_`, `-`).
func newItemID() string {
	var rnd [4]byte
	if _, err := rand.Read(rnd[:]); err != nil {
		return "item-disc-" + fmt.Sprintf("%d", time.Now().UnixNano())
	}
	return "item-disc-" + hex.EncodeToString(rnd[:])
}

// discretionaryWritePath derives the proposal's governed write path from the item name: a
// kebab-case slug under discretionary/, e.g. "Rollback rehearsal!" -> discretionary/rollback-rehearsal.md.
// The slug keeps only ASCII alphanumerics so the path always passes the kernel's relative-path
// validation (no `..`, no absolute, no empty segments).
func discretionaryWritePath(name string) string {
	var b []byte
	lastDash := true // suppress a leading dash
	for i := 0; i < len(name) && len(b) < 80; i++ {
		c := name[i]
		switch {
		case c >= 'a' && c <= 'z', c >= '0' && c <= '9':
			b = append(b, c)
			lastDash = false
		case c >= 'A' && c <= 'Z':
			b = append(b, c+'a'-'A')
			lastDash = false
		default:
			if !lastDash {
				b = append(b, '-')
				lastDash = true
			}
		}
	}
	for len(b) > 0 && b[len(b)-1] == '-' {
		b = b[:len(b)-1]
	}
	if len(b) == 0 {
		b = []byte("item")
	}
	return "discretionary/" + string(b) + ".md"
}

// discretionaryContentHint is the write operation's content hint: the payload's summary when it
// carries one, else the mandatory justification body.
func discretionaryContentHint(p contract.AddDiscretionaryWorkPayload) string {
	if p.Summary != nil && *p.Summary != "" {
		return *p.Summary
	}
	return p.Justification
}

// proposedObject renders the resulting_object an accepted ADD_DISCRETIONARY_WORK returns: the
// proposed item as it exists the moment the kernel accepted it. The object kind mirrors the
// projection builder's mapping (milestones are their own kind, everything else is a work item);
// the stage anchor is a dependency, not a parent, so parent_id stays unset for it.
func proposedObject(itemID, title, kind, anchorID, proposer string) contract.CognitiveObject {
	obj := contract.CognitiveObject{
		ID:       itemID,
		Kind:     "work_item",
		Name:     title,
		Status:   "WAITING",
		Badge:    "Proposed",
		Salience: 0.5,
		Actions:  []contract.ActionDescriptor{},
	}
	if kind == "milestone" {
		obj.Kind = "milestone"
	}
	s := "Proposed by " + proposer + "; the kernel's plan_mutated frame carries it into the world."
	if anchorID != "" {
		s = "Proposed by " + proposer + "; it waits on " + anchorID + "'s entry sentry."
	}
	obj.Explanation = &s
	return obj
}
