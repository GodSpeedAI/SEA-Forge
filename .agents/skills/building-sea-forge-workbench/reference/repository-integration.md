# Repository integration — actual substrate (verified 2026-07-25, branch `frontend`)

Everything here was verified against the working tree. If reality has moved,
update this file in the same change that depends on the new reality. Claims
that could not be verified are marked `UNKNOWN — repository evidence not found`.

## Workspace topology

- Rust workspace, 22 crates under `crates/` (root `Cargo.toml`, resolver 2,
  edition 2021, `rust-version 1.92.0`, `unsafe_code = "deny"`).
- Workbench exists at `workbench/`: Bun 1.4.0 workspace
  (`workbench/package.json:6-7`) with a React/Vite renderer at
  `apps/desktop/`, a standalone Tauri 2 host workspace at
  `apps/desktop/src-tauri/`, and token/theme/contracts/component packages
  under `packages/`.
- Task runner: `justfile` via devbox. Gates: `devbox run -- just check`,
  `just test`, `just context-check`, `just fmt-check`, `just lint`,
  `just no-async-kernel`, `just ci`, `just proof`.
- Plans live in `.agents/plans/` named `YYYY-MM-DD-<topic>.md`; specs in
  `.agents/specs/`; ADRs in `docs/decisions/` (ADR-001 domainforge boundary,
  ADR-002 dependency approvals, ADR-003 additive contracts).
- Frontend gates: `bun run check`, `bun run build`, `bun run test`, and
  `bunx playwright test` from `workbench/`; `just workbench-check` is the
  root convenience gate. The Tauri host remains outside the root Cargo
  workspace by design (ADR-004).

## Server and transport (the wire the Workbench must speak)

| Fact | Location |
|---|---|
| Unix-socket NDJSON server, tokio | `crates/sea-forge-server/src/lib.rs:647` (`UnixListener::bind`, 0600 permissions, line-delimited JSON) |
| Request envelope: `#[serde(tag = "verb", rename_all = "snake_case")] pub enum Request` | `crates/sea-forge-server/src/lib.rs:468` |
| Existing verbs | Legacy verbs plus additive SFWP `system_*`, `request_get_status`, `events_*`, catalog/describe, and `readiness_get` (`lib.rs:468-617`) |
| Readiness view | `crates/sea-forge-server/src/sfwp/readiness.rs:166`; dispatched from `lib.rs:1116` |
| Server config (`server.yaml`, `socket_path`) | `crates/sea-forge-server/src/config.rs` |
| Binary entry | `crates/sea-forge-server/src/main.rs` (39 lines) |
| Delegation + transcript evidence + SWE_SEED declaration ingress | `crates/sea-forge-server/src/delegation.rs` (1851 lines) |
| Case dispatch | `crates/sea-forge-server/src/case_dispatch.rs` |
| Agent probe | `crates/sea-forge-server/src/agent_probe.rs` |
| Transcript sealing | `crates/sea-forge-server/src/transcript_seal.rs` |
| SWE_SEED reconciliation (claim-manifest hashes) | `crates/sea-forge-server/src/swe_seed_reconciliation.rs` |

**SFWP substrate now present:** negotiation, request correlation/status
recovery, event subscription/range replay, method catalog/describe, and
`readiness.get` are implemented additively under
`crates/sea-forge-server/src/sfwp/` and dispatched from `lib.rs`. Later
vertical slices still need their own view-shaped query/protected-command
verbs; the catalog is not proof that roadmap methods exist.

## Canonical types and owners

| Owner | Location |
|---|---|
| Canonical record types (cases, plans, criteria, settlement, approvals, authority, trace) | `crates/sea-forge-core/src/types.rs` (~1500 lines; serde snake_case) |
| Typed errors with machine classes (`ForgeError`, `.class()`) | `crates/sea-forge-core/src/errors.rs` |
| Authority fabric (default deny, one engine) | `crates/sea-forge-authority/` |
| Append-only ledger, ULID entries, MMR proofs | `crates/sea-forge-ledger/src/types.rs:206` (`entry_ulid`), `:916` (`prove_entry`) — the natural durable event cursor |
| Settlement evaluation + declarations | `crates/sea-forge-settlement/` |
| Capability promotion | `crates/sea-forge-capability/` |
| Evidence | `crates/sea-forge-evidence/` |
| Planner / case engine / criteria provenance | `crates/sea-forge-planner/` |
| Self-model + projections | `crates/sea-forge-self-model/` |
| Thoth disclosure engine | `crates/sea-forge-thoth/src/engine.rs`; service seam `service.rs:258` (`pub fn ask`), `service.rs:60` (`LedgerSnapshotView::open`) |
| Thoth protocol types (`ThothQuestion:151`, `ThothAnswer:223`, `QuestionKind:120`, `ClaimClass:41`, `GroundedClaim:91`, high-risk classes `:79`) | `crates/sea-forge-thoth/src/protocol.rs` |
| ACP sessions + permission mediation (`AcpPermissionRequest:63`, `PermissionDecision:49`, `AcpPermissionMediator:77`, `AcpTermination:129`, `AcpSession:166`) | `crates/sea-forge-agent/src/acp.rs` |
| CLI adapters (the pattern SFWP handlers should mirror: thin adapter over shared service) | `crates/sea-forge-cli/src/commands/` (`ask.rs`, `approve.rs`, `case.rs`, `run.rs`, `self_model.rs`, …) |

## Identity, sponsor, approvals

- Actor identity today is a CLI/server argument defaulting to
  `operator_local` (18 sites in `crates/sea-forge-cli/src/main.rs`); approvals
  check standing and separation-of-duties server-side
  (`crates/sea-forge-settlement/src/declaration.rs:541-560`,
  `sea_forge_core::types::validate_claim_authorship_sod` at
  `crates/sea-forge-core/src/types.rs:1492`).
- SFWP `context.actor_id`/`role` propagation to these checks is additive work.
  Renderer never resolves identity; the Tauri host passes it through.

## Generated contracts

- Rust SFWP types derive schemas through `schemars`
  (`crates/sea-forge-server/Cargo.toml:26`); `gen_sfwp_schema` emits committed
  schema files under `workbench/packages/contracts/schema/`.
- `workbench/packages/contracts/scripts/generate.ts` generates TypeScript and
  AJV validators into `packages/contracts/generated/`; for example
  `ReadinessView.ts` and `ReadinessView.validator.ts`. These zones are
  generated and must never be hand-edited (ADR-005).

## Tests and fixtures

- Per-crate `tests/` conformance suites (e.g.
  `crates/sea-forge-server/tests/conformance_m12.rs`, `_m13`, `_m16`;
  `sea-forge-cli/tests/ask_cli.rs`). Mirror these for new verbs.
- End-to-end proofs: `.agents/specs/spec-minimum.md` §12.2 (P1–P4b) and
  `spec-full.md` §12/§17 milestone gates — must stay green after Workbench
  server changes.

## Packaging / release

- `just release-check`, `scripts/`, devbox environment. Desktop packaging
  (Tauri bundling, signing, icons): `UNKNOWN — repository evidence not found`
  — greenfield in the packaging milestone.

## Missing seams (summary for planners)

| Seam | Status |
|---|---|
| Frontend workspace (Bun/Vite/React) | EXISTS (`workbench/`) |
| Tauri host crate | EXISTS (`workbench/apps/desktop/src-tauri/`) |
| SFWP envelope/negotiation/request-status | EXISTS |
| Event subscription + cursor + gap recovery | EXISTS |
| JSON Schema / TS generation | EXISTS |
| Read-model queries shaped for views | PARTIAL (`readiness.get` exists; later route families remain milestone-scoped) |
| Thoth AG-UI stream | MISSING (`ask` is synchronous request/response today) |
| ACP permission surfacing to an interactive client | PARTIAL (`AcpPermissionMediator` trait exists; only `DenyAllMediator` shipped) |
