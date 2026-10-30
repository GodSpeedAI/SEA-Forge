# ports

## Overview
The application's own interfaces and value types (REQ-ARCH-001). Go defines application-owned ports
so that provider, transport, rendering and integration technologies remain adapters; SEA Forge SFWP
view structs, Gauntlet executor types and repository-provider payloads are translated at the adapter
boundary (REQ-ARCH-002) and never appear here. `internal/boundary` enforces this mechanically.

## Interface
- Core port interfaces: `Health`, `AuthorityPort`, `ExecutionPort`, `RepositoryPort`, `ArtifactStore` [ports.go:AuthorityPort]
- Live case authority: `CaseAuthorityPort` (list cases, overviews, horizons, runs, approvals, templates, preflight, commit, decisions, items, lifecycle, artifacts, identity, readiness, status) [ports.go:CaseAuthorityPort]
- Run observation: `RunTracePort`, `RunTraceSnapshot`, `RunTraceFrame` [run_trace.go:RunTracePort]
- Governed Ask: `AskPort`, `AskQuestion`, `AskClaim`, `AskAnswer` [ask.go:AskPort]
- Artifact viewing: `RunArtifactLister`, `RunArtifactRef`, `ArtifactContent`, `ErrArtifactIntegrity` [ports.go:RunArtifactLister] [ports.go:ErrArtifactIntegrity]
- Governance value types: `ActorClaim`, `Governance`, `GovernedOptions` [ports.go:Governance]
- Case/run value types: `CaseRef`, `RunRef`, `ArtifactRef`, `Standing`, `CaseSummary`, `SettlementObservation`, `CaseRecord`, `CaseOverview`, `CaseHorizon`, `RunSummary`, `RunListResult`, `ApprovalRecord`, `HorizonItem` (via `CaseHorizon`), `CaseItemProposal`, `AdvanceReport`, `EpisodeReport` [ports.go:CaseRef]
- Template/commit types: `TemplateParameter`, `TemplateOption`, `CaseDraft`, `PreconditionDigest`, `PreflightReport`, `CommitReceipt` [ports.go:CaseDraft]
- Identity/readiness: `IdentityReport`, `IdentityRefusal`, `AvailableActor`, `ReadinessReport`, `ReadinessCondition`, `RequestOutcome`, `OperationResult`, `ClassRefusal` [ports.go:IdentityReport]

## Invariants
- Nothing in this package imports a transport, a provider SDK, or an adapter [ports.go:package doc]
- Provider payloads must be named by their own types — no untyped carriers that could hide provider shape [ports.go:package doc]
- `ErrArtifactIntegrity`: stored artifact content that does not match its digest is an integrity failure [ports.go:ErrArtifactIntegrity]

## Side Effects
- None (interface and value-type declarations only)

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
