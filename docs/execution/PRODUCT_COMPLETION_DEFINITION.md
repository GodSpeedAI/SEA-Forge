# Product Completion Definition

SEA Forge is complete when a representative user can install the supported
local distribution and finish the governed Workbench journey against real
records. Existing test success is necessary but not sufficient.

## Completion Claim Levels

| Level | Observable claim |
|---|---|
| Component verified | A crate, package, or adapter passes its focused checks. |
| Slice integrated | One vertical path crosses its real boundaries and leaves valid records. |
| Journey usable | The representative journey works in a developer-run real stack. |
| Distribution verified | A clean supported host installs and runs the packaged journey. |
| Product complete | Linux distribution is verified, macOS support is either verified or explicitly excluded, operational/security/release gates pass, and no required journey uses a preview, mock, or silent fallback. |

Higher levels include lower levels. Pass 1 establishes component verification
and the minimum CLI journey; it does not establish distribution verification.

## Observable Completion Contract

### Installation And Upgrade

- A clean Linux host follows documented prerequisites and installs or unpacks a
  versioned Workbench, server, and CLI artifact set without a repository checkout.
- The documented first-run path selects an existing cell root or initializes a
  new one after checking for history.
- Desktop, server, CLI, record-schema, and SFWP versions are visible and checked
  before protected work.
- Upgrading preserves record IDs and history and runs explicit compatible
  migrations. Unsupported record/schema combinations fail visibly.

Evidence: clean-host setup log, artifact hashes, version negotiation output,
migration tests over prior fixtures, and retained-cell restart proof.

### Deterministic Build And Generated Artifacts

- Pinned Rust/Cargo and Bun inputs produce the same source-derived contracts and
  token projection on repeated runs.
- Rust JSON Schema, TypeScript/AJV, UI tokens, and release realization are
  regenerated from their canonical sources and produce zero diff.
- Build output contains no Bun runtime and no untracked generated source.
- The standalone Tauri Cargo workspace boundary remains intact.

Evidence: CI regeneration-to-temp comparisons, `cargo metadata` boundary check,
bundle inventory, and repeat-build checksums where platform tooling permits.

### Required Services And Startup

- The owner-selected service topology starts or reconnects the Workbench to its
  matching local server through one documented procedure.
- Desktop, server, and CLI resolve one absolute cell root and one socket path.
- The socket is owner-only (`0600`) and a second conflicting server fails safely.
- Invalid server configuration, unavailable required sandbox, invalid policy,
  integrity failure, DomainForge mismatch, or unresolved identity blocks
  protected work with a typed corrective action. No invalid configuration falls
  back to defaults.
- A valid config reload affects only future dispatches; invalid reload retains
  the last-known-good snapshot and emits a visible event.

Evidence: startup matrix tests, process/socket inspection, and UI readiness states
backed by server records.

### Frontend And Real Backend Communication

- SFWP hello/version negotiation succeeds through the real Tauri host.
- The renderer accesses no socket, `.sea-forge/`, database, or generic backend
  invocation.
- Every received payload is validated against the generated contract before use.
- Events resume from a durable cursor after disconnect and server restart.
- A 10-second ordinary request timeout does not lose request status or fabricate
  cancellation.

Evidence: packaged E2E with Tauri command, socket, server, reconnect, and cursor
traces. Mocked IPC tests remain component tests only.

### Primary User Journey

The packaged product proves all of these against one temporary cell:

1. Open or initialize the cell without overwriting existing history.
2. Display resolved actor, role, sponsor where required, policy digest, semantic
   model, integrity assurance, sandbox availability, and server readiness.
3. Ask what the installation can currently do and receive a source-linked,
   disclosure-controlled answer that separates declared from demonstrated.
4. Select a real domain/template/environment/executor and inspect immutable
   criteria and provenance.
5. Preflight then commit a case with one stable idempotency key.
6. Exercise an allow path and a deny/escalate path; denial leaves no unauthorized
   side effect and remains inspectable.
7. Complete at least one command episode and one human approval path. Agent work
   is required only for an artifact claiming agent support.
8. Observe execution and settlement as separate states, including exit-zero
   false success settling rejected.
9. Disconnect during a mutation, recover by request ID, and prove exactly one
   mutation occurred.
10. Restart during or after work, then inspect the case, run, evidence, and
    recovery/orphan state.
11. Inspect authority, trace, evidence, artifacts, declarations, and settlement
    from source-linked views.
12. Reuse an accepted result through memory/capability/projection/artifact while
    showing only the assurance the evidence supports.

### Persistence And Data Integrity

- Every case-linked run ID resolves to one complete canonical run before and
  after restart.
- Every sandboxed episode contains authority echo, trace, evidence, criteria,
  settlement, and required semantic/capability references.
- Append-only streams verify chain/MMR commitments and survive a killed writer as
  a valid prefix.
- SQLite and UI/query projections rebuild from canonical records without changing
  meaning.
- Existing flat minimum-run fixtures remain readable through the declared
  compatibility/migration rule.

Evidence: generated real episodes, restart/kill tests, migration fixtures,
cross-reference verification, and rebuild equivalence.

### Authority, Security, And Safety

- Every protected ingress reaches the one authority mediator before any effect,
  including workspace creation.
- Missing identity, policy, unsupported action, unavailable sandbox, semantic
  drift, and integrity failure deny or escalate without effects.
- The exact action grant is consumed by the matching sandbox/runtime and cannot
  be reused with changed context.
- Approver identity and separation-of-duty are server-resolved and recorded.
- Child processes use argv, minimal explicit environment, bounded output and
  time, and no network unless granted.
- Secret sentinels never appear in source records, logs, errors, transcripts,
  views, fixtures, or bundles.
- Renderer CSP and Tauri capability scopes permit only required local actions.

Evidence: negative conformance tests that assert absence of side effects,
sentinel scans, Tauri permission audit, and real sandbox tests.

### Errors And Recovery

- Input, configuration, authority, integrity, semantic, transport, execution,
  settlement, compatibility, and recovery failures retain machine-readable
  classes across server, bridge, and UI.
- Every blocked/failed state identifies evidence, affected capability, and the
  next lawful action.
- Unknown protocol versions, verbs, enum variants, and event kinds fail cleanly
  or mark views stale; none render as success.
- Notification failure is visible in logs but never changes settlement.
- Orphaned/in-flight work after restart is explicit and recoverable or safely
  terminable.

### Test Architecture

- Unit tests cover pure reducers, validation, canonicalization, and components.
- Contract tests verify Rust/schema/TS compatibility and old-reader behavior.
- Integration tests run real server, socket, authority, sandbox, persistence,
  restart, and generated records.
- Packaged E2E runs the representative journey through Tauri and the real server.
- Linux Landlock is exercised. macOS Seatbelt is exercised before macOS is
  advertised as a verified package. Optional external integrations use separate
  configured release gates.
- All skips state the platform/infrastructure reason; no inferred pass is allowed.

### Packaging, CI, And Release

- Required CI aggregates kernel, Workbench, Tauri host, generated drift,
  packaging, and real E2E results.
- A Linux package and CLI archive are built, checksummed, installed, smoke-tested,
  and retained as release artifacts.
- macOS artifacts receive the same treatment before being called supported.
- The release does not claim `cargo install sea-forge-cli` until every registry
  dependency is published and an install test passes.
- Release notes name record/schema migrations, supported platforms, optional
  integrations, and known bounded limitations.

## Permitted Deferrals

The following do not block the first complete local product when they are
explicitly marked unsupported and no required journey depends on them:

- Unmapped SFWP catalog methods outside the representative journey.
- Windows/mobile, remote HTTP, multi-user auth, containers, Kubernetes, hosted
  control plane, MicroVM, NATS, and marketplace features.
- Real ACP, SWE_SEED, witness, or IFL tests for packages that do not advertise
  those integrations.
- Crates.io publication of internal libraries.
- Optional CopilotKit adapter work.

## Completion Blockers At This Commit

- Case sandbox episodes bypass the full lifecycle and canonical run view.
- Workbench lacks resolved identity and safe idempotent mutation recovery.
- Server startup/config reload and request timeout violate the full spec.
- No owner-selected, implemented, and verified desktop/server lifecycle exists;
  default socket paths also disagree.
- Packaged real-server E2E and supported release artifacts do not exist.
- Workbench/generated/package gates are absent from remote CI.

No completion claim may hide these blockers behind preview labels or passing
component tests.
