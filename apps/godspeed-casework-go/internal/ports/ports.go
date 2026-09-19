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
