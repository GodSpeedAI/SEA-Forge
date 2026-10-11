# T09 unit5B safe trace port: test-first assignment

HOLD until unit5A cancellation is independently approved and root verifies its
source identities and exact gate captures. This assignment is preparation only.

## Binding specification and original instructions

Read `observation-safe-trace-port-root-proposal.md`, every independent review
and clarification, and the repaired `observation-safe-trace-port-root-proposal-v3.md`
(SHA-256 e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6).
V3 is the repaired semantic specification; the original and rejection history
remain part of the independent review envelope. Its proposal approval is
`observation-safe-trace-port-independent-review-v3-oct05.md` (84b9f657).
Apply every v3 projection, ownership, refusal, count and test-matrix requirement.
Neither this phase nor port completion settles pollers, SSE, UI or full T09.

## Bounded builder ownership

Own only these new Go files under `apps/godspeed-casework-go/internal/`:

- `ports/run_trace.go`: separate semantic RunTracePort, RunTraceSnapshot and
  RunTraceFrame declarations. Snapshot fields are RunID, CaseID, PlanItemID,
  Execution, Settlement, Frames and TotalFrameCount. Frame fields are EventID,
  Kind, Timestamp, optional ExecutionStatus pointer and optional ExitCode int64
  pointer. Semantic types have no wire JSON tags and do not import raw DTOs.
- `adapters/sfwp/run_trace.go`: the minimal compiling Authority.ReadRunTrace
  signature returning typed unavailable, explicitly temporary for assertion RED.
  No decoder, transport request or safe projection implementation in phase 1.
- `adapters/sfwp/run_trace_test.go`: complete focused matrix, using bounded owned
  SFWP peers/listeners, synchronized actual request/response evidence and cleanup.

Read nearby ports and adapter tests before writing. Native apply_patch only.
Do not edit client.go, cancellation tests, artifact DTO/authorization, existing
tests, contracts, schemas, Rust, Store, CaseFacts, server, UI or dependencies.

## Required proof details beyond the referenced matrix

Tests must distinguish exactly one logical run_get from invented fallback verbs;
positive tests verify the real requested run ID and exact expected ownership.
Blank expected identities must eventually fail invalid before dialing. Refusal
classes remain unchanged. Stub failure must reach a behavior assertion, never
a compile error, leaked peer or cleanup timeout. Verify request counts when the
real implementation exists; avoid awaiting a request before observing the stub's
immediate error. Every blocked worker has owned release/close and bounded join.

Unknown string kinds are omitted before validating their event IDs, timestamps
or payload fields; include malformed ignored fields and a selected/unknown ID
collision. Duplicate selected IDs outside the newest1024 remain invalid. Test
all selected kinds and standings, full returned-array counts, newest source
order, optional execution metadata, exact safe integers and overflow, trace
presence metadata, zero returned rows with positive bytes, non-atomic byte/row
combinations, wrong/null/missing shapes, and absence of public synthetic raw-field
markers in semantic serialization. Never use real credentials or private payloads.
Semantic serialization is not the outgoing canonical wire representation.

## Builder and independent critic loop

Phase 1 builder writes fixtures/declarations only, checks formatting without
compiling, records exact source hashes/diff and material deviations, then stops.
No compiler token is granted automatically. An independent critic receives these
ORIGINAL instructions, all referenced originals and the actual frozen result.
It reviews source first, then runs the intended compiling assertion RED only
after root transfers the sole compiler token. No evidence means no approval.
On rejection a fresh builder repairs findings without weakening assertions.

Production phase 2 is a fresh assignment after accepted RED, confined to
`adapters/sfwp/run_trace.go`. It must implement v3 completely. Independent
focused/full SFWP/canonical/full Go verification remains required. Every compile
gets actual HOST RAM/process preflight and serialized ownership. Go limits:
GOMEMLIMIT256MiB, GOGC50, GOMAXPROCS2, GOFLAGS-p1, GOLDEN_UPDATE0, race/count1/
parallel1. Save all command output, exits and preflights under /tmp, copy exact
bytes dynamically with native apply_patch to NEW immutable evidence files and
byte-compare; retain failures and disclose missing evidence. Never overwrite
evidence. No staging, commit, push, status or debt edits by this builder.
