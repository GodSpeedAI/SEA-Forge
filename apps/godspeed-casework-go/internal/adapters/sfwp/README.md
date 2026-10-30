# sfwp

## Overview
The live adapter for the governed SEA Forge kernel: an NDJSON-over-Unix-socket client for the
kernel's SFWP wire protocol (`ProtocolVersion "1"`) plus the `Authority`/`AskAdapter` implementations
of the application-owned `internal/ports` interfaces. This is the only place kernel view structs,
refusal classes, and transport behavior appear; everything crossing back into the application is
translated into `ports` types. The client multiplexes with one in-flight request per pooled
connection (the kernel pairs NDJSON replies with requests positionally; see `crates/sea-forge-server/src/lib.rs`).

## Interface
### Client / transport
- `Client` — pooled, deadline-aware SFWP client; `New(cfg Config)`, `Do(ctx, *Request) (*Response, error)`, `Close()` [client.go:Client]
- `Config` — socket path, response-line cap (default 32 MiB), admission limiter, error observer [client.go:Config]
- `Request` constructors — `NewSystemHello`, `NewSystemDescribe`, `NewRequestGetStatus`, `NewReadinessGet`, `NewIdentityGet`, `NewEventsSubscribe`, `NewEventsUnsubscribe`, `NewEventsGetRange`, `NewCaseEntryOptions`, `NewCasePreflight`, `NewCaseCommit`, `NewApprovalList`, `NewApprovalDecide`, `NewCaseList`, `NewRunList`, `NewRunListForCase`, `NewRunGet`, `NewCaseGetOverview`, `NewCaseGetHorizon`, `NewCaseAddItem`, `NewCaseReopen`, `NewCaseTerminate`, `NewCaseAdvance`, `NewItemExecute`, `NewHumanTaskComplete`, `NewArtifactGet` [frame.go]
- `Response`, `DecodeResponse`, `EncodeRequest`, `(*Response).Into` — frame codec [frame.go:Response]
- `Refusal` — kernel refusal with class; maps onto `apperr.Kind` [frame.go:Refusal]
- Wire views (`HelloView`, `CaseOverviewView`, `RunTraceView`-adjacent structs, etc.) — kernel-shaped JSON views translated by `Authority` [frame.go]
- `Subscription`, `(*Client).Subscribe`, `(*Client).EventsGetRange` — event-stream subscription with resumable cursor [subscribe.go]
- `RunGetLimiter`, `RunGetAdmission`, `RunGetPermit` — per-run `run.get` admission control (1 s cooldown per run) [run_get_admission.go:RunGetLimiter] [run_get_admission.go:runGetAdmissionCooldown]
- `ProtocolVersion` — `"1"` [frame.go:ProtocolVersion]

### Ports implementations
- `Authority` — implements `ports.CaseAuthorityPort`, `ports.RunTracePort`, `ports.RunArtifactLister`, `ports.Health`: `Health`, `ListCases`, `CaseOverview`, `CaseHorizon`, `RunsList`, `RunsListForCase`, `PendingApprovals`, `EntryOptions`, `PreflightCase`, `CommitCase`, `DecideApproval`, `AddCaseItem`, `ReopenCase`, `TerminateCase`, `AdvanceCase`, `ExecuteItem`, `CompleteHumanTask`, `GetArtifact`, `RunArtifacts`, `ResolveIdentity`, `Readiness`, `RequestStatus`, `ReadRunTrace` [authority.go:Authority] [run_trace.go:Authority] [artifact_provenance.go:RunArtifacts]
- `AskAdapter` — implements `ports.AskPort`: `Ask` (Thoth governed ask) [ask.go:AskAdapter]
- `Governance` — actor/governance frame block carried on requests (actor, role, on_behalf_of, policy) [frame.go:Governance]

## Invariants
- Everything crossing into the application is translated into `internal/ports` types; kernel views stay here [authority.go:Authority]
- One in-flight request per pooled connection; replies pair with requests positionally [client.go:package doc]
- Response lines are size-bounded (`readBoundedLine`, default 32 MiB) [client.go:readBoundedLine]
- `run.get` admission: at most one in-flight read per run, 1 s cooldown between reads [run_get_admission.go:AcquireRunGet]
- Mutation verbs are the closed set in `mutationVerbs`; mutations carry request ids and are transport-retry-safe only when declared [frame.go:mutationVerbs] [frame.go:IsTransportRetrySafe]
- Run-trace decoding bounds: retention window 1024, safe-JSON-integer cap `maxSafeJSONInteger` [run_trace.go:runTraceRetention] [run_trace.go:maxSafeJSONInteger]

## Side Effects
- Opens Unix-domain sockets to the kernel and exchanges NDJSON frames (`dial`, `call`) [client.go:dial]
- Spawns subscription reader goroutines; buffered event delivery to `Subscription.Done`/events channel [subscribe.go:runSubscription]
- Writes SFWP error observations through `Config.OnError` (wired to the metrics registry by main) [client.go:Config]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
