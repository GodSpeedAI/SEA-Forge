// Package ports declares the application's own interfaces and value types.
//
// REQ-ARCH-001: Go defines application-owned ports so that provider, transport, rendering and
// integration technologies remain adapters. SEA Forge SFWP view structs, Gauntlet executor types,
// and repository-provider payloads are translated at the adapter boundary (REQ-ARCH-002) and never
// appear here. internal/boundary enforces this mechanically.
//
// Nothing in this package imports a transport, a provider SDK, or an adapter: the value types below
// are the stable application model that the adapters translate *into*.
package ports

import (
	"context"
	"time"
)

// CaseRef identifies a governed case in application terms. It carries no provider identity: an
// adapter maps it to whatever the provider needs.
type CaseRef string

// RunRef identifies one execution episode in application terms.
type RunRef string

// ArtifactRef is a content-addressed reference to a stored artifact.
type ArtifactRef string

// Standing is the authoritative standing of a run's settlement, in application terms. It is a
// vocabulary the application owns; adapters map provider spellings onto it and must refuse to guess
// when the provider reports something the application does not model.
type Standing string

const (
	StandingAccepted       Standing = "accepted"
	StandingReviewRequired Standing = "review_required"
	StandingPending        Standing = "pending"
)

// Health is the liveness contract every port exposes. Preflight uses it to decide blast radius
// without knowing what a provider considers healthy.
type Health interface {
	Health(ctx context.Context) error
}

// CaseSummary is a bounded, non-authoritative view of a case for projection building.
type CaseSummary struct {
	Ref       CaseRef
	Title     string
	State     string
	UpdatedAt time.Time
}

// SettlementObservation is what the authority recorded for a run, including the evidence basis it
// cited. A run is not settled because a process exited; it is settled because the authority says so.
type SettlementObservation struct {
	Run            RunRef
	Standing       Standing
	Basis          []string
	ReviewRequired bool
	SettledAt      time.Time
}

// ExecutionRequest asks an executor to perform one bounded unit of work.
type ExecutionRequest struct {
	Run     RunRef
	Intent  string
	Timeout time.Duration
}

// ExecutionObservation is what the executor observed. It is an observation, never a settlement.
type ExecutionObservation struct {
	Run            RunRef
	StartedAt      time.Time
	FinishedAt     time.Time
	Outcome        string
	EvidenceDigest string
}

// HistoryPoint is one position in a case's history, expressed in application terms rather than in a
// provider's record shape.
type HistoryPoint struct {
	At      time.Time
	Summary string
	Version string
}

// HistoryWindow is a bounded page of history. The bound is explicit because a provider's history
// query may be capped (the SEA Forge event log caps a replay at 500 frames per call).
type HistoryWindow struct {
	Case   CaseRef
	Points []HistoryPoint
}

// ChangeProposal is a repository-side change in application terms.
type ChangeProposal struct {
	ID     string
	Title  string
	URL    string
	Merged bool
}

// AuthorityPort is the application's view of governed case authority: case meaning, settlement, and
// history. It grants no authority itself; it reports what the authority decided.
type AuthorityPort interface {
	Health
	ListCases(ctx context.Context) ([]CaseSummary, error)
	Settlement(ctx context.Context, run RunRef) (SettlementObservation, error)
	History(ctx context.Context, ref CaseRef, since time.Time, limit int) (HistoryWindow, error)
}

// ExecutionPort is the application's view of an executor. It returns observations only: nothing here
// can settle, approve, or promote anything.
type ExecutionPort interface {
	Health
	Execute(ctx context.Context, req ExecutionRequest) (ExecutionObservation, error)
}

// RepositoryPort is the application's view of repository facts for workflows that use a repository.
type RepositoryPort interface {
	Health
	Proposal(ctx context.Context, id string) (ChangeProposal, error)
}

// ArtifactStore is an OPTIONAL capability: artifact persistence for the cognitive environment. It is
// declared as a port so a missing or failing implementation degrades rather than blocks.
type ArtifactStore interface {
	Health
	Put(ctx context.Context, name string, content []byte) (ArtifactRef, error)
	Get(ctx context.Context, ref ArtifactRef) ([]byte, error)
}

// ---------------------------------------------------------------------------
// Live case authority (plan casework-live-wiring-production T05)
// ---------------------------------------------------------------------------

// ActorClaim names the principal behind a governed operation and the role it claims. It is
// application vocabulary: adapters translate it into whatever identity block the authority's wire
// requires, and the authority - never the application - decides whether the claim resolves.
type ActorClaim struct {
	ActorID string
	Role    string
}

// Governance carries who a protected operation runs for: the claim the authority verifies, and
// optionally the end user a gateway principal speaks for (operator decision D-2: the ledger records
// both principals and separation of duty compares end users). A nil OnBehalfOf means the actor acts
// directly.
type Governance struct {
	Actor      ActorClaim
	OnBehalfOf *ActorClaim
}

// GovernedOptions are the knobs every protected (side-effecting) case operation accepts. RequestID
// is the caller's correlation id: an adapter may not leave it empty on a mutation, because it is
// the only handle a reconnect can recover an interrupted outcome through. Policy is the
// cell-relative authority policy; empty means the authority's own default.
type GovernedOptions struct {
	Governance Governance
	Policy     string
	RequestID  string
	TimeoutSec int
}

// TemplateParameter is one parameter a case template declares.
type TemplateParameter struct {
	Name       string
	Type       string
	Required   bool
	Default    string
	HasDefault bool
}

// TemplateOption is one template an operator may instantiate a case from.
type TemplateOption struct {
	TemplateRef string
	Description string
	Parameters  []TemplateParameter
}

// CaseDraft is an un-committed case: a template plus the operator's parameter choices. Nothing
// about a draft is authoritative; it becomes case truth only through CommitCase.
type CaseDraft struct {
	TemplateRef string
	Params      map[string]string
}

// PreconditionDigest pins the template bytes a preflight validated, so the commit that follows can
// refuse rather than run against a changed template. Opaque to the application: carry it from
// PreflightCase into CommitCase unchanged.
type PreconditionDigest struct {
	Present        bool
	TemplateRef    string
	ExpectedDigest string
}

// PlanItemSummary is a non-authoritative summary of one instantiated plan item.
type PlanItemSummary struct {
	ItemID string
	Name   string
	Kind   string
}

// PreflightReport is the audit-only dry run of a draft: what the case would contain, what would
// have to change, and the precondition pin a commit should carry.
type PreflightReport struct {
	OK           bool
	TemplateRef  string
	Errors       []string
	Items        []PlanItemSummary
	Precondition PreconditionDigest
}

// CommitReceipt is what the authority recorded when a case was committed.
type CommitReceipt struct {
	CaseID   string
	State    string
	ExitCode int
}

// CaseRecord is one row of the case list: a bounded, non-authoritative view.
type CaseRecord struct {
	Ref         CaseRef
	State       string
	Summary     string
	CreatedAt   time.Time
	ClosedAt    time.Time
	RunCount    int
	CloseReason string
}

// SettlementNote is a governed settlement record linked to one run. Execution standing and
// settlement standing stay disjoint: a run executes and a settlement records - never the same fact.
type SettlementNote struct {
	Run            RunRef
	Status         string
	Basis          []string
	ReviewRequired bool
	SettledAt      time.Time
}

// CaseOverview is the full view of one committed case.
type CaseOverview struct {
	Ref         CaseRef
	State       string
	Summary     string
	CreatedAt   time.Time
	TemplateRef string
	ItemCount   int
	Stages      []string
	Settlements []SettlementNote
	RunIDs      []RunRef
}

// HorizonItem is one plan item with its separately-derived execution and settlement standing.
type HorizonItem struct {
	ItemID      string
	Name        string
	Kind        string
	Execution   string
	Settlement  string
	ParentStage string
	DependsOn   []string
	LastEventID string
	RunIDs      []RunRef
}

// CaseHorizon is the derived per-case work horizon. LastEventID names the newest trace event
// folded into it: a caller holding a later event knows this view is behind and must refetch.
type CaseHorizon struct {
	Ref          CaseRef
	State        string
	Items        []HorizonItem
	LastEventID  string
	EventsFolded int
}

// RunSummary is one row of the governed run list: an episode with its separately-derived
// execution and settlement standing (never collapsed into one fact).
type RunSummary struct {
	RunID         string
	CaseID        string
	PlanItemID    string
	Execution     string
	Settlement    string
	StartedAt     time.Time
	HasStarted    bool
	FinishedAt    time.Time
	HasFinished   bool
	EvidenceCount int
}

// RunListResult is the scoped authority view of readable runs and run IDs the authority could
// not read. Unreadable IDs are factual reports, not inferred ownership.
type RunListResult struct {
	Runs          []RunSummary
	UnreadableIDs []string
}

// ApprovalRecord is one decision awaiting a resolver.
type ApprovalRecord struct {
	ApprovalID  string
	CaseID      string
	RunID       string
	ItemID      string
	DecisionID  string
	RequestedAt time.Time
	ExpiresAt   time.Time
	Expired     bool
	CriteriaRef string
}

// ItemOperation is one governed operation a proposed item performs.
type ItemOperation struct {
	Kind        string
	Path        string
	ContentHint string
}

// ItemCriterion is one sentry watch a proposed item declares: an event kind (and status, when the
// kind is a settlement status) on a source item that unlocks this one.
type ItemCriterion struct {
	Source string
	Event  string
	Kind   string
	Status string
}

// CaseItemProposal is a discretionary work item an operator proposes mid-case. The authority
// validates it (including dependency-cycle checks), overwrites the proposer with the verified
// actor, and refuses rather than guesses when the shape is not governable.
type CaseItemProposal struct {
	ItemID            string
	Name              string
	Kind              string
	SandboxClass      string
	ParentStage       string
	DependsOn         []string
	EntryCriteriaMode string
	EntryCriteria     []ItemCriterion
	Operations        []ItemOperation
	RequiredArtifacts []string
	RequireApproval   bool
	MaxInstances      int
}

// EpisodeReport is one executed episode of an advance pass.
type EpisodeReport struct {
	ItemID     string
	RunID      string
	Settlement string
}

// AdvanceReport is the outcome of an advance or single-item execution pass. State is the pass's
// terminal standing (completed, awaiting_approval, parked_human_task, ...).
type AdvanceReport struct {
	CaseID   string
	State    string
	Episodes []EpisodeReport
}

// ArtifactContent is a content-addressed artifact the authority already committed, fetched back
// with its integrity re-verified by the authority.
type ArtifactContent struct {
	Digest     string
	RunID      string
	CaseID     string
	PlanItemID string
	EvidenceID string
	URI        string
	Size       int64
	Data       []byte
}

// AvailableActor is one principal the authority says the calling connection may act as.
type AvailableActor struct {
	ActorID string
	Roles   []string
}

// IdentityRefusal is the authority refusing an identity claim before any side effect.
type IdentityRefusal struct {
	Class            string
	Message          string
	NextLawfulAction string
}

// IdentityReport is what the authority says about who the calling connection may act as, and - when
// a delegation is presented - which end user a delegated operation would be attributed to.
type IdentityReport struct {
	UID            int64
	Configured     bool
	Available      []AvailableActor
	EffectiveActor *ActorClaim
	Refusal        *IdentityRefusal
}

// ReadinessCondition is one readiness condition the authority derives from committed truth.
type ReadinessCondition struct {
	ID               string
	Name             string
	Category         string
	Status           string
	Reason           string
	NextLawfulAction string
}

// ReadinessReport is the authority's own readiness projection.
type ReadinessReport struct {
	Overall      string
	Foundations  []ReadinessCondition
	Capabilities []ReadinessCondition
}

// OperationResult is the terminal outcome payload of a correlated request, normalized. An error
// outcome carries the authority's error text and class rather than being flattened into a Go error,
// because a caller recovering by request id needs the recorded outcome verbatim.
type OperationResult struct {
	OK         bool
	CaseID     string
	State      string
	ExitCode   int
	Error      string
	ErrorClass string
}

// RequestOutcome is the recovered lifecycle of a correlated request: pending while the authority
// has admitted it but not settled it, terminal once it has, unknown when no record exists.
type RequestOutcome struct {
	RequestID string
	Status    string // pending | completed | failed | unknown
	Method    string
	Result    *OperationResult
}

// ClassRefusal is implemented by adapter refusals that carry the authority's own error class
// verbatim. Application-layer refusers map classes onto the typed refusal vocabulary through
// errors.As without importing any adapter package.
type ClassRefusal interface {
	error
	RefusalClass() string
}

// CaseAuthorityPort is the application's view of the live, governed case authority: the full verb
// surface the cognitive environment needs, in application terms. Like AuthorityPort it grants no
// authority itself; every method reports or requests what the authority decides. Adapters translate
// these calls into the authority's wire protocol; the wire shapes never appear here.
type CaseAuthorityPort interface {
	Health
	// Inspect surface.
	ListCases(ctx context.Context) ([]CaseRecord, error)
	CaseOverview(ctx context.Context, ref CaseRef) (CaseOverview, error)
	CaseHorizon(ctx context.Context, ref CaseRef) (CaseHorizon, error)
	PendingApprovals(ctx context.Context, ref CaseRef) ([]ApprovalRecord, error)
	RunsList(ctx context.Context) ([]RunSummary, error)
	RunsListForCase(ctx context.Context, ref CaseRef) (RunListResult, error)
	EntryOptions(ctx context.Context) ([]TemplateOption, error)
	PreflightCase(ctx context.Context, draft CaseDraft) (PreflightReport, error)
	GetArtifact(ctx context.Context, digest string) (ArtifactContent, error)
	ResolveIdentity(ctx context.Context, g Governance) (IdentityReport, error)
	Readiness(ctx context.Context) (ReadinessReport, error)
	RequestStatus(ctx context.Context, requestID string) (RequestOutcome, error)
	// Governed (side-effecting) surface. Every method is correlated: the adapter must ensure a
	// non-empty RequestID so an interrupted outcome stays recoverable.
	CommitCase(ctx context.Context, draft CaseDraft, pin PreconditionDigest, opts GovernedOptions) (CommitReceipt, error)
	DecideApproval(ctx context.Context, approvalID string, approve bool, note string, opts GovernedOptions) error
	AddCaseItem(ctx context.Context, ref CaseRef, item CaseItemProposal, opts GovernedOptions) (string, error)
	ReopenCase(ctx context.Context, ref CaseRef, opts GovernedOptions) error
	TerminateCase(ctx context.Context, ref CaseRef, reason string, opts GovernedOptions) error
	AdvanceCase(ctx context.Context, ref CaseRef, opts GovernedOptions) (AdvanceReport, error)
	ExecuteItem(ctx context.Context, ref CaseRef, itemID string, opts GovernedOptions) (AdvanceReport, error)
	CompleteHumanTask(ctx context.Context, ref CaseRef, itemID string, justification string, opts GovernedOptions) error
}
