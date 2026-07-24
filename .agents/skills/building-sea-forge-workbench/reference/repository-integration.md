# Repository integration — actual substrate (verified 2026-07-24, branch `full-spec`)

Everything here was verified against the working tree. If reality has moved,
update this file in the same change that depends on the new reality. Claims
that could not be verified are marked `UNKNOWN — repository evidence not found`.

## Workspace topology

- Rust workspace, 22 crates under `crates/` (root `Cargo.toml`, resolver 2,
  edition 2021, `rust-version 1.92.0`, `unsafe_code = "deny"`).
- **No JavaScript/TypeScript workspace exists.** No `package.json`, no
  `bun.lock`, no `node_modules` outside `.opencode/` (editor tooling, not
  product). No Tauri, no Vite, no frontend directory of any kind.
- Task runner: `justfile` via devbox. Gates: `devbox run -- just check`,
  `just test`, `just context-check`, `just fmt-check`, `just lint`,
  `just no-async-kernel`, `just ci`, `just proof`.
- Plans live in `.agents/plans/` named `YYYY-MM-DD-<topic>.md`; specs in
  `.agents/specs/`; ADRs in `docs/decisions/` (ADR-001 domainforge boundary,
  ADR-002 dependency approvals, ADR-003 additive contracts).
- Frontend workspace location: **to be created** — plan Milestone 2 proposes
  `workbench/` at repo root (Bun workspace + Tauri host), kept out of the
  Cargo workspace except the Tauri host crate. Until it exists:
  `UNKNOWN — repository evidence not found` for any frontend path.

## Server and transport (the wire the Workbench must speak)

| Fact | Location |
|---|---|
| Unix-socket NDJSON server, tokio | `crates/sea-forge-server/src/lib.rs:515-535` (`UnixListener::bind(socket_path)`, 0600 perms, line-delimited JSON responses) |
| Request envelope: `#[serde(tag = "verb", rename_all = "snake_case")] pub enum Request` | `crates/sea-forge-server/src/lib.rs:396-401` |
| Existing verbs | `submit`, `status`, `approve`, `reject`, `agent_list`, `agent_probe`, `delegate`, `cancel_delegation`, `ask` (`lib.rs:401-…`) |
| Responses are ad-hoc `serde_json::json!` values, one line each | `lib.rs:563-568` |
| Server config (`server.yaml`, `socket_path`) | `crates/sea-forge-server/src/config.rs` |
| Binary entry | `crates/sea-forge-server/src/main.rs` (39 lines) |
| Delegation + transcript evidence + SWE_SEED declaration ingress | `crates/sea-forge-server/src/delegation.rs` (1851 lines) |
| Case dispatch | `crates/sea-forge-server/src/case_dispatch.rs` |
| Agent probe | `crates/sea-forge-server/src/agent_probe.rs` |
| Transcript sealing | `crates/sea-forge-server/src/transcript_seal.rs` |
| SWE_SEED reconciliation (claim-manifest hashes) | `crates/sea-forge-server/src/swe_seed_reconciliation.rs` |

**Missing vs SFWP** (all additive, ADR-003 pattern — old servers reject
unknown verbs cleanly): request envelopes with `request_id`/`protocol_version`,
`system.hello` negotiation, `request.get_status` recovery, `events.subscribe`
/ `events.get_range`, preconditions/expected digests on protected verbs,
structured error frames. Idempotency: `UNKNOWN — repository evidence not
found` for any server-side request-correlation store.

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

- **No schema/TS generation exists.** `schemars`, `ts-rs`, `typeshare` appear
  in no `Cargo.toml`. The `canonical Rust types → JSON Schema → TypeScript →
  AJV` pipeline is greenfield (plan Milestone 3); adding the generator crate
  requires an ADR-002 entry.

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
| Frontend workspace (Bun/Vite/React) | MISSING |
| Tauri host crate | MISSING |
| SFWP envelope/negotiation/request-status | MISSING (additive on existing socket) |
| Event subscription + cursor + gap recovery | MISSING (ledger `entry_ulid` is the substrate) |
| JSON Schema / TS generation | MISSING (needs ADR) |
| Read-model queries shaped for views (`readiness.get`, `case.get_horizon`, …) | MISSING to PARTIAL (records exist; view shaping absent) |
| Thoth AG-UI stream | MISSING (`ask` is synchronous request/response today) |
| ACP permission surfacing to an interactive client | PARTIAL (`AcpPermissionMediator` trait exists; only `DenyAllMediator` shipped) |
