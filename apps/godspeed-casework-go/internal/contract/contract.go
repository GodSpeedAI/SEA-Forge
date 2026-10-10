// Package contract mirrors the canonical UI<->gateway wire contract (operator decision D-1,
// 2026-09-23): the spec-04 CognitiveWorldSnapshot types from
// .agents/reports/interface-contracts/typescript/types.ts.
//
// The JSON tags are the contract: every tag matches the TypeScript field name exactly
// (camelCase, snake_case where the contract says so). The golden fixtures under
// .agents/reports/interface-contracts/golden/ pin both sides: TestGoldenRoundTrip fails on any
// byte-level drift between these structs and the fixtures, and the exhaustive-kind tests fail
// when an intent, refusal or stream-event kind exists on only one side of the contract.
//
// This package is deliberately dependency-free (standard library only). The Go projection and
// coordinator vocabulary (kebab-case intent kinds, the fixture world shape) moves to these names
// in T06; until then those packages keep their fixture shapes and must not be treated as the
// contract.
package contract

// ActionDescriptor is one role-filtered action offered on an object (spec-04 §3).
type ActionDescriptor struct {
	ID                    string  `json:"id"`
	Label                 string  `json:"label"`
	Intent                string  `json:"intent"`
	Variant               *string `json:"variant,omitempty"`
	Consequential         bool    `json:"consequential"`
	RequiresJustification *bool   `json:"requires_justification,omitempty"`
}

// SpatialLayout is an object's optional placement in the 2.5D grammar.
type SpatialLayout struct {
	X      float64  `json:"x"`
	Y      float64  `json:"y"`
	Z      float64  `json:"z"`
	Radius *float64 `json:"radius,omitempty"`
}

// CognitiveObject is one object in the world projection (spec-04 §3).
type CognitiveObject struct {
	ID            string             `json:"id"`
	Kind          string             `json:"kind"`
	Name          string             `json:"name"`
	Status        string             `json:"status"`
	Badge         string             `json:"badge"`
	Explanation   *string            `json:"explanation,omitempty"`
	Salience      float64            `json:"salience"`
	ParentID      *string            `json:"parent_id,omitempty"`
	DependsOn     []string           `json:"depends_on,omitempty"`
	SpatialLayout *SpatialLayout     `json:"spatial_layout,omitempty"`
	Actions       []ActionDescriptor `json:"actions"`
	// X carries the optional spec-04 object extensions this gateway populates (artifact bindings).
	X *ObjectExtensions `json:"x,omitempty"`
}

// CognitiveArtifact is an artifact bound to an object (spec-04 §7); its bytes come from
// GET /api/artifacts/{digest}, never from the snapshot.
type CognitiveArtifact struct {
	Ref              string `json:"ref"`
	Kind             string `json:"kind"`
	Title            string `json:"title"`
	BoundObject      string `json:"boundObject"`
	CurrentLevel     string `json:"currentLevel"`
	MediaType        string `json:"mediaType"`
	SourceProvenance string `json:"sourceProvenance"`
}

// ObjectExtensions is the subset of the UI's object extensions (src/ports/contract.ts
// ObjectExtensions) the gateway projects.
type ObjectExtensions struct {
	Artifacts []CognitiveArtifact `json:"artifacts,omitempty"`
}

// ActorPerspective is the snapshot's acting view point.
type ActorPerspective struct {
	ActorID     string  `json:"actor_id"`
	Role        string  `json:"role"`
	DisplayName *string `json:"display_name,omitempty"`
}

// WorldSummary is the snapshot's headline block.
type WorldSummary struct {
	Headline        string   `json:"headline"`
	Phase           string   `json:"phase"`
	StatusPhrase    string   `json:"status_phrase"`
	ProgressPercent *float64 `json:"progress_percent,omitempty"`
}

// AttentionFocus is the projection's attention anchor.
type AttentionFocus struct {
	PrimaryObjectID string   `json:"primary_object_id"`
	SalienceRank    []string `json:"salience_rank,omitempty"`
	Narration       *string  `json:"narration,omitempty"`
}

// CognitiveWorldSnapshot is the canonical world projection served by the gateway (spec-04 §3).
type CognitiveWorldSnapshot struct {
	WorldID          string             `json:"world_id"`
	CaseID           string             `json:"case_id"`
	Cursor           string             `json:"cursor"`    // "<epoch>.<seq>" kernel cursor
	Timestamp        string             `json:"timestamp"` // ISO RFC3339
	Perspective      ActorPerspective   `json:"perspective"`
	Summary          WorldSummary       `json:"summary"`
	VisibleObjects   []CognitiveObject  `json:"visible_objects"`
	AvailableActions []ActionDescriptor `json:"available_actions"`
	AttentionFocus   AttentionFocus     `json:"attention_focus"`
}

// IntentActor is the acting principal carried by an intent.
type IntentActor struct {
	ActorID string `json:"actor_id"`
	Role    string `json:"role"`
}

// InteractionIntent is the consequential intent envelope (spec-04 §5). Typed payloads ride in
// Parameters under the action names listed by PayloadCarryingKinds.
type InteractionIntent struct {
	IntentID       string         `json:"intent_id"`
	Kind           string         `json:"kind"` // REACT_LOCAL | BACKEND_INFORMATION | CONSEQUENTIAL_CASE
	ActionName     string         `json:"action_name"`
	TargetObjectID string         `json:"target_object_id"`
	CaseID         string         `json:"case_id"`
	ClientCursor   string         `json:"client_cursor"`
	Actor          IntentActor    `json:"actor"`
	Parameters     map[string]any `json:"parameters,omitempty"`
	Justification  *string        `json:"justification,omitempty"`
}

// IntentRefusal is the typed refusal envelope on a refused intent (T01).
type IntentRefusal struct {
	RefusalKind   string  `json:"refusal_kind"`
	Message       string  `json:"message"`
	CurrentCursor *string `json:"current_cursor,omitempty"`
}

// IntentResponse answers an intent; a refusal is an outcome, not a transport error.
type IntentResponse struct {
	IntentID        string           `json:"intent_id"`
	Success         bool             `json:"success"`
	NewCursor       *string          `json:"new_cursor,omitempty"`
	Refusal         *IntentRefusal   `json:"refusal,omitempty"`
	ErrorCode       *string          `json:"error_code,omitempty"`    // legacy stringly field, kept additively
	ErrorMessage    *string          `json:"error_message,omitempty"` // legacy stringly field, kept additively
	ResultingObject *CognitiveObject `json:"resulting_object,omitempty"`
}

// IntentFixture is the golden-file pairing of one intent request with its exercised responses
// (first the success answer, then each meaningful refusal).
type IntentFixture struct {
	Request   InteractionIntent `json:"request"`
	Responses []IntentResponse  `json:"responses"`
}

// TemplateParameter is one declaratively typed template parameter.
type TemplateParameter struct {
	Name         string   `json:"name"`
	Title        *string  `json:"title,omitempty"`
	Description  *string  `json:"description,omitempty"`
	Type         string   `json:"type"` // string | number | boolean | enum
	Required     *bool    `json:"required,omitempty"`
	DefaultValue any      `json:"default_value,omitempty"`
	Options      []string `json:"options,omitempty"`
}

// TemplateEntryOption is one entry option returned by case.entry_options.
type TemplateEntryOption struct {
	TemplateRef string              `json:"template_ref"`
	Title       string              `json:"title"`
	Description *string             `json:"description,omitempty"`
	Parameters  []TemplateParameter `json:"parameters"`
}

// TemplatePreflightResult is the preflight verdict for a filled template.
type TemplatePreflightResult struct {
	TemplateRef string         `json:"template_ref"`
	Params      map[string]any `json:"params"`
	Passed      bool           `json:"passed"`
	Reasons     []string       `json:"reasons"`
	Digest      *string        `json:"digest,omitempty"`
}

// StreamEvent is the generic SSE envelope for existing event payloads. Cursor is the real case
// cursor; revision events may use it for Last-Event-ID.
type StreamEvent struct {
	EventType string `json:"event_type"`
	Cursor    string `json:"cursor"`
	Timestamp string `json:"timestamp"`
	Payload   any    `json:"payload"`
}

// RunTraceObservationEvent is the named informational side-channel event. Cursor is the latest
// real case cursor; the event does not create a trace cursor or advance case history.
type RunTraceObservationEvent struct {
	EventType string              `json:"event_type"`
	Cursor    string              `json:"cursor"`
	Timestamp string              `json:"timestamp"`
	Payload   RunTraceObservation `json:"payload"`
}

const RunTraceObservationEventType = "execution_observation"

type RunTraceFrameKind string

type RunTraceCommandExecutionStatus string

type RunExecutionStanding string

type RunSettlementStanding string

type RunTraceRunObservationState string

type RunTraceListState string

type RunTraceObservationState string

// RunTraceFrame is the safe allowlist projection of one real trace row. Command metadata is
// present only when it was recorded on an actual command_finished row.
type RunTraceFrame struct {
	EventID         string                          `json:"event_id"`
	Kind            RunTraceFrameKind               `json:"kind"`
	Timestamp       string                          `json:"timestamp"`
	ExecutionStatus *RunTraceCommandExecutionStatus `json:"execution_status,omitempty"`
	ExitCode        *int64                          `json:"exit_code,omitempty"`
}

// RunTraceRunObservation is one successfully owned run, distinct from invocation outcomes.
type RunTraceRunObservation struct {
	RunID              string                      `json:"run_id"`
	ObservedAt         string                      `json:"observed_at"`
	PlanItemID         string                      `json:"plan_item_id"`
	Execution          RunExecutionStanding        `json:"execution"`
	Settlement         RunSettlementStanding       `json:"settlement"`
	ObservationState   RunTraceRunObservationState `json:"observation_state"`
	Frames             []RunTraceFrame             `json:"frames"`
	TotalFrameCount    int                         `json:"total_frame_count"`
	RetainedFrameCount int                         `json:"retained_frame_count"`
	OmittedFrameCount  int                         `json:"omitted_frame_count"`
	Truncated          bool                        `json:"truncated"`
}

type RunTraceHydrationReadBudget struct {
	Limit          int  `json:"limit"`
	ReadsAttempted int  `json:"reads_attempted"`
	Exhausted      bool `json:"exhausted"`
}

// RunTraceObservation is one bounded hydration cohort. Counts remain nil until run.list
// completed and decoded, preserving absent versus present zero values on unavailable reads.
type RunTraceObservation struct {
	CaseID              string                      `json:"case_id"`
	ObservedAt          string                      `json:"observed_at"`
	RunListState        RunTraceListState           `json:"run_list_state"`
	ObservationState    RunTraceObservationState    `json:"observation_state"`
	ListedRunCount      *int                        `json:"listed_run_count,omitempty"`
	SelectedRunCount    *int                        `json:"selected_run_count,omitempty"`
	ValidatedRunCount   *int                        `json:"validated_run_count,omitempty"`
	UnreadableRunCount  *int                        `json:"unreadable_run_count,omitempty"`
	UnavailableRunCount *int                        `json:"unavailable_run_count,omitempty"`
	OmittedRunCount     *int                        `json:"omitted_run_count,omitempty"`
	HydrationReadBudget RunTraceHydrationReadBudget `json:"hydration_read_budget"`
	Runs                []RunTraceRunObservation    `json:"runs"`
}

type ThothQuestionKind string

type ThothClaimClass string

type ThothClaimStatus string

type ThothDisposition string

type ThothFreshness string

// ThothAskRequest is the authenticated route input; identity comes from the verified session.
type ThothAskRequest struct {
	Kind    ThothQuestionKind `json:"kind"`
	Subject string            `json:"subject"`
	Purpose *string           `json:"purpose,omitempty"`
	Case    *string           `json:"case,omitempty"`
}

type ThothClaimView struct {
	ClaimID             string           `json:"claim_id"`
	ClaimClass          ThothClaimClass  `json:"claim_class"`
	Subject             string           `json:"subject"`
	Status              ThothClaimStatus `json:"status"`
	Statement           string           `json:"statement"`
	SnapshotRef         string           `json:"snapshot_ref"`
	EvidenceRefs        []string         `json:"evidence_refs"`
	SettlementRefs      []string         `json:"settlement_refs"`
	CapabilityRecordRef *string          `json:"capability_record_ref,omitempty"`
}

// ThothAnswerView preserves the complete governed disclosure, including denied/partial metadata.
type ThothAnswerView struct {
	AnswerID            string            `json:"answer_id"`
	QuestionID          string            `json:"question_id"`
	Disposition         ThothDisposition  `json:"disposition"`
	Claims              []ThothClaimView  `json:"claims"`
	OmittedClaimClasses []ThothClaimClass `json:"omitted_claim_classes"`
	SnapshotRef         string            `json:"snapshot_ref"`
	Freshness           ThothFreshness    `json:"freshness"`
	Assurance           string            `json:"assurance"`
	Limitations         []string          `json:"limitations"`
	AuthorityNotice     string            `json:"authority_notice"`
	AnsweredAt          string            `json:"answered_at"`
}

// ExecutionProgressPayload is the payload of an execution_progress event.
type ExecutionProgressPayload struct {
	RunID           string  `json:"run_id"`
	Phase           string  `json:"phase"` // orchestrator | builder | critic | verifier | settling
	ProgressPercent float64 `json:"progress_percent"`
	LogLine         string  `json:"log_line"`
}

// ResyncRequiredPayload is the payload of a resync_required event.
type ResyncRequiredPayload struct {
	RequestedCursor       string `json:"requested_cursor"`
	OldestAvailableCursor string `json:"oldest_available_cursor"`
	Reason                string `json:"reason"`
}

// ErrorPayload is the payload of an error stream event (the stream itself failed).
type ErrorPayload struct {
	ErrorCode string `json:"error_code"`
	Message   string `json:"message"`
}

// InterruptedPayload is the payload of an interrupted event (connection signal).
type InterruptedPayload struct {
	Reason     string  `json:"reason"`
	LastCursor *string `json:"last_cursor,omitempty"`
}

// ExecutionLease is the typed payload of a lease_expired stream event.
type ExecutionLease struct {
	LeaseID       string `json:"lease_id"`
	OpportunityID string `json:"opportunity_id"`
	WorkerID      string `json:"worker_id"`
	Role          string `json:"role"`
	ExpiresAt     string `json:"expires_at"`
	Nonce         string `json:"nonce"`
}

// OperationalSettlement is the typed payload of a settlement_recorded stream event.
type OperationalSettlement struct {
	SettlementID       string  `json:"settlement_id"`
	CaseID             string  `json:"case_id"`
	PlanItemID         string  `json:"plan_item_id"`
	InvocationID       string  `json:"invocation_id"`
	Decision           string  `json:"decision"` // ACCEPTED | REJECTED | INCONCLUSIVE
	EvidenceID         *string `json:"evidence_id,omitempty"`
	SettledAt          string  `json:"settled_at"`
	ConsequenceSummary string  `json:"consequence_summary"`
}

// Typed intent request payloads (T01), each riding in InteractionIntent.Parameters.

// ProposeCasePayload is the PROPOSE_CASE parameters payload.
type ProposeCasePayload struct {
	TemplateRef     string         `json:"template_ref"`
	Params          map[string]any `json:"params"`
	PreflightDigest string         `json:"preflight_digest"`
}

// AddDiscretionaryWorkPayload is the ADD_DISCRETIONARY_WORK parameters payload.
type AddDiscretionaryWorkPayload struct {
	CaseID        string  `json:"case_id"`
	StageID       string  `json:"stage_id"`
	AnchorItemID  *string `json:"anchor_item_id,omitempty"`
	Kind          string  `json:"kind"`
	Title         string  `json:"title"`
	Summary       *string `json:"summary,omitempty"`
	Justification string  `json:"justification"`
}

// ExecuteItemPayload is the EXECUTE_ITEM parameters payload.
type ExecuteItemPayload struct {
	ItemID string `json:"item_id"`
}

// CompleteHumanTaskPayload is the COMPLETE_HUMAN_TASK parameters payload.
type CompleteHumanTaskPayload struct {
	ItemID        string         `json:"item_id"`
	Result        map[string]any `json:"result"`
	Justification string         `json:"justification"`
}

// CaseLifecyclePayload is the REOPEN_CASE and TERMINATE_CASE parameters payload.
type CaseLifecyclePayload struct {
	CaseID string `json:"case_id"`
	Reason string `json:"reason"`
}

// AllIntentKinds is the exhaustive set of consequential journey intent names on the wire
// (T01 set; mirrors ConsequentialIntentName in types.ts). A kind present here but absent from
// the golden fixtures (or vice versa) fails TestGoldenCoversEveryIntentKind.
var AllIntentKinds = []string{
	"PROPOSE_CASE",
	"ADD_DISCRETIONARY_WORK",
	"EXECUTE_ITEM",
	"COMPLETE_HUMAN_TASK",
	"APPROVE_HUMAN_TASK",
	"REJECT_HUMAN_TASK",
	"ESCALATE_OR_OVERRIDE",
	"OPEN_ARTIFACT",
	"REOPEN_CASE",
	"TERMINATE_CASE",
}

// AllRefusalKinds is the exhaustive typed refusal vocabulary (mirrors IntentRefusalKind).
var AllRefusalKinds = []string{
	"AUTHORITY_DENIED",
	"UNAUTHORIZED_ROLE",
	"SOD_VIOLATION",
	"STALE_PROJECTION",
	"JUSTIFICATION_REQUIRED",
	"UNAVAILABLE",
	"INVALID",
}

// AllStreamEventKinds is the exhaustive SSE event vocabulary (mirrors StreamEventType).
var AllStreamEventKinds = []string{
	"snapshot",
	"patch",
	"execution_progress",
	"settlement_recorded",
	"lease_expired",
	"resync_required",
	"interrupted",
	"error",
	"heartbeat",
	RunTraceObservationEventType,
}

var AllRunTraceFrameKinds = []string{
	"run_started",
	"item_activated",
	"command_started",
	"command_finished",
	"item_completed",
	"item_failed",
	"item_terminated",
	"human_task_completed",
	"run_halted",
	"run_finished",
}

var AllRunTraceCommandExecutionStatuses = []string{
	"completed",
	"spawn_failed",
	"timed_out",
	"sandbox_violation",
	"suspected_sandbox_violation",
}

var AllRunExecutionStandings = []string{"pending", "enabled", "active", "completed", "failed", "terminated"}

var AllRunSettlementStandings = []string{"unsettled", "accepted", "rejected", "escalated"}

var AllRunTraceRunObservationStates = []string{"validated", "unavailable"}

var AllRunTraceListStates = []string{"complete", "unavailable"}

var AllRunTraceObservationStates = []string{"complete", "no_runs", "capacity_limited", "unavailable"}

var AllThothQuestionKinds = []string{
	"ask_capability",
	"ask_operation_requirements",
	"ask_authority_requirements",
	"ask_projection_support",
	"ask_environment_status",
	"ask_failure_explanation",
	"ask_evidence_for_claim",
	"ask_available_affordances",
	"ask_why_denied",
}

var AllThothClaimClasses = []string{
	"identity",
	"architecture",
	"declared_capability",
	"installed_capability",
	"demonstrated_capability",
	"authority_requirements",
	"environment_status",
	"failure_condition",
	"security_implementation",
	"customer_private",
	"credential_bearing",
	"policy_thresholds",
}

var AllThothClaimStatuses = []string{"unknown", "unsupported", "declared", "installed", "available", "validated", "demonstrated"}

var AllThothDispositions = []string{"answered", "partial", "denied"}

var AllThothFreshnessValues = []string{"current", "stale"}

// PayloadCarryingKinds lists the intent kinds whose parameters carry a typed payload
// (mirrors PayloadCarryingIntentName in types.ts).
var PayloadCarryingKinds = []string{
	"PROPOSE_CASE",
	"ADD_DISCRETIONARY_WORK",
	"EXECUTE_ITEM",
	"COMPLETE_HUMAN_TASK",
	"REOPEN_CASE",
	"TERMINATE_CASE",
}

// payloadPrototype returns a zero value of the typed payload for kind, or nil when the kind
// carries no typed payload (envelope fields only).
func payloadPrototype(kind string) any {
	switch kind {
	case "PROPOSE_CASE":
		return &ProposeCasePayload{}
	case "ADD_DISCRETIONARY_WORK":
		return &AddDiscretionaryWorkPayload{}
	case "EXECUTE_ITEM":
		return &ExecuteItemPayload{}
	case "COMPLETE_HUMAN_TASK":
		return &CompleteHumanTaskPayload{}
	case "REOPEN_CASE", "TERMINATE_CASE":
		return &CaseLifecyclePayload{}
	}
	return nil
}
