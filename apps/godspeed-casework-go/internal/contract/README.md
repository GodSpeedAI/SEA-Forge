# contract

## Overview
Go mirror of the canonical UI↔gateway wire contract (operator decision D-1, 2026-09-23): the
spec-04 `CognitiveWorldSnapshot` types from
`.agents/reports/interface-contracts/typescript/types.ts`. JSON tags are the contract — every tag
matches the TypeScript field name exactly. Golden fixtures under
`.agents/reports/interface-contracts/golden/` pin both sides. The package is deliberately
dependency-free (standard library only).

## Interface
- Snapshot types: `CognitiveWorldSnapshot`, `WorldSummary`, `AttentionFocus`, `CognitiveObject`, `CognitiveArtifact`, `SpatialLayout`, `ActionDescriptor`, `ObjectExtensions`, `ActorPerspective` [contract.go:CognitiveWorldSnapshot]
- Intent types: `InteractionIntent`, `IntentActor`, `IntentRefusal`, `IntentResponse`, `IntentFixture` [contract.go:InteractionIntent]
- Template types: `TemplateParameter`, `TemplateEntryOption`, `TemplatePreflightResult` [contract.go:TemplateParameter]
- Stream/observation types: `StreamEvent`, `RunTraceObservationEvent` (`RunTraceObservationEventType` = `"execution_observation"`), `RunTraceFrame`, `RunTraceObservation`, `RunTraceRunObservation`, `RunTraceHydrationReadBudget`, and the frame/status/standing/state string types [contract.go:StreamEvent] [contract.go:RunTraceObservationEventType]
- Thoth Ask types: `ThothAskRequest`, `ThothClaimView`, `ThothAnswerView`, and the question-kind/claim-class/status/disposition/freshness string types [contract.go:ThothAskRequest]
- Lifecycle payload types: `ExecutionProgressPayload`, `ResyncRequiredPayload`, `ErrorPayload`, `InterruptedPayload`, `ExecutionLease`, `OperationalSettlement`, `ProposeCasePayload`, `AddDiscretionaryWorkPayload`, `ExecuteItemPayload`, `CompleteHumanTaskPayload`, `CaseLifecyclePayload` [contract.go:ExecutionLease]
- Closed vocabularies: `AllIntentKinds`, `AllRefusalKinds`, `AllStreamEventKinds`, `AllRunTraceFrameKinds`, `AllRunTraceCommandExecutionStatuses`, `AllRunExecutionStandings`, `AllRunSettlementStandings`, `AllRunTraceRunObservationStates`, `AllRunTraceListStates`, `AllRunTraceObservationStates`, `AllThothQuestionKinds`, `AllThothClaimClasses`, `AllThothClaimStatuses`, `AllThothDispositions`, `AllThothFreshnessValues`, `PayloadCarryingKinds` [contract.go:AllIntentKinds]

## Invariants
- JSON tags are the contract: they must match the TypeScript field names exactly; golden round-trip tests fail on any byte-level drift [contract.go:package doc]
- Exhaustive-kind tests fail when an intent, refusal or stream-event kind exists on only one side of the contract [contract.go:package doc]
- Standard library only — no external dependencies [contract.go:package doc]

## Side Effects
- None (pure type declarations)

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
