# Implementation Brief — GodSpeed Casework Cognitive Environment

- **Authority:** non-normative execution context (plan `source.implementation_brief`).
- **Governing spec:** `.agents/specs/godspeed.casework-cognitive-environment-spec.yaml`,
  id `godspeed.casework-cognitive-environment`, version `0.2.1`, status `authoritative`,
  sha256 `6312453fc571ffdd14f14c47a9a7d3d8670fafe76e6dd80d004e9d50852d6641`.
- **Bound plan:** `.agents/plans/godspeed-casework-cognitive-environment.plan.yaml` v0.2.2 (tasks T00–T14).
- **Conflict rule:** the normative spec governs. A conflict is surfaced as a blocked
  decision; it is never silently resolved here.
- **Producer:** T00 (`.agents/evidence/godspeed-casework-cognitive-environment/T00/`).
  Frozen revision of every fact below: `6ce518fcd9b01bc5a7037f80f5d8986a33cd2924`.

## 1. Frozen authority and repository topology

| Item | Value |
|---|---|
| SEA Forge worktree for this plan | `/home/sprime01/projects/sea-rs/.worktrees/godspeed-casework-cognitive-environment` |
| SEA Forge branch | `godspeed/casework-cognitive-environment` (base `6ce518f`) |
| Operator's active SEA Forge checkout (excluded, never written) | `/home/sprime01/projects/sea-rs` (branch `ultracode/sea-forge-completion`) |
| Gauntlet worktree for this plan (`$GAUNTLET_WORKTREE`) | `/home/sprime01/projects/gauntlet-godspeed-casework` |
| Gauntlet branch / revision | `godspeed/casework-environment` @ `fe75108d62129ffb179b6773b67f3ee92213e44d` |
| Operator's active Gauntlet checkout (excluded, never written) | `/home/sprime01/projects/gauntlet` (branch `task26-dcritic-apparatus-repair`) |
| Gate validator | `.agents/plans/validate-godspeed-casework-cognitive-environment.py` → PASS in the worktree |

`GATE_SPEC_TRACE` and the frozen hash reproduce in the worktree exactly as declared by
the plan. The spec/plan/validator/ADR are all committed at `6ce518f`, so no
out-of-band artifact transfer was needed; nothing was fetched, reset, cleaned, or merged.

**Revision-bound facts a cold agent must not misread:**

1. The plan's removal target *Gauntlet TUI* exists **only on the branch containing
   `fe75108`**. `git ls-tree main` contains no `workbench/**` TUI. The dedicated
   Gauntlet worktree is therefore pinned to `fe75108`, not to `main` (`e4ca393`).
2. `apps/godspeed-casework-go` and `apps/godspeed-cognitive-ui` do **not** exist. The
   Cargo workspace members are exclusively `crates/*` (**22 members**, matching
   `Cargo.toml`; `crates/` also holds an `AGENTS.md` that is not a member). There is no Go
   module, no `go.mod`, and **no Go toolchain on this host** (`go` is not on `PATH`;
   `mise` resolves `go@1.27.1` as *not installed*).

## 2. Toolchain and machine facts

| Tool | Version | Note |
|---|---|---|
| rustc / cargo | 1.92.0 | `rust-toolchain.toml` pinned; workspace `rust-version = "1.92.0"` |
| bun | 1.4.0 | `workbench/package.json` `packageManager: bun@1.4.0`; Gauntlet TUI requires `bun >= 1.4.0` |
| node | 26.8.2 | present |
| python3 | 3.14.4 | validator + gate scripts |
| cargo-deny | 0.20.2 | `just deny` |
| gitleaks | 8.30.1 | `just security` — see §9 red baseline |
| mise | 2026.8.2 | manages `aqua:go-task/task`, `docker-cli`, `ffmpeg`; **no Go install** |
| **go** | **ABSENT** | hard prerequisite for `GATE_GO` and every T01-dependent task |
| nproc / MemTotal | 6 / 7.76 GiB (8132716 kB) | no cgroup memory limit (`/sys/fs/cgroup/memory.max` absent → use host `MemAvailable`) |

Build-resource rule that governs every gate run in this plan: effective headroom is
`min(MemAvailable, cgroup limit − usage)`; with no finite cgroup limit it is
`MemAvailable`. At T00 the observed `MemAvailable` range across gates was
3.24–3.65 GiB (3395144–3831780 kB) with swap free 7.14–7.47 GiB (7486428–7833572 kB) of 12 GiB and low memory pressure
(`/proc/pressure/memory` `some avg300` sampled at 0.84 early in T00 and 2.45 later — both inline
observations, now backed in form by `raw-logs/preflight-readings.log`; see the round-6 correction in
`resource-preflight.md`). Peak RSS observed: **0.99 GiB** (`just test`,
1036928 kB) and 0.53 GiB (`just proof`, 556952 kB). Run at most one heavy gate at a time and use
`CARGO_BUILD_JOBS=1`; a gate that is not run is `RESOURCE_DEFERRED`, never passed. Every memory
figure in this brief converts with 1 GiB = 1048576 kB; an earlier draft mislabelled the same
numbers as GiB when they were GB (found by independent verification round 3).

## 3. SEA Forge interfaces: SFWP v1 (exact, source-of-truth)

Anchor: `crates/sea-forge-server/src/sfwp/mod.rs` (`SFWP_PROTOCOL_VERSION = "1"`,
`IMPLEMENTED_METHODS`), `crates/sea-forge-server/src/lib.rs` (`Request` enum dispatch),
`crates/sea-forge-server/src/identity.rs`, `crates/sea-forge-server/src/sfwp/events.rs`,
`crates/sea-forge-server/src/sfwp/correlation.rs`.

**Transport.** NDJSON over a **local Unix domain socket** (`<root>/server.sock`, mode
`0600`). One JSON object per line. Request `{id, method, params}`; success `{id, result}`;
error `{id, error:{code,message,data}}`; server notifications `{type:"event", event:{...}}`.
Byte caps: `MAX_RECORD_BYTES = 4 MiB`, `MAX_JOURNAL_BYTES = 64 MiB` (readers treat an
oversized record as unreadable rather than allocating). Admission is bounded
(`max_concurrent_runs` permits + eight-request waiting room) and overflow returns
`server_busy` **before** durable state. Consequence for this plan: the Go front end is
**co-located** with the cell; there is no remote transport, no TLS, no bearer auth.

**Identity/authorization.** Peer credentials (`SO_PEERCRED` uid) + an actor claim line
+ role bindings (`IdentityBindings::resolve`), with a protected-request gate
(`identity::is_protected`) and SoD enforcement (`sod_violation`). `identity.get`
reports the active actor, roles, and available actors; `readiness.get` evaluates cell
readiness. There is no token authority: a forged *frontend* identity cannot be minted
beyond the local uid, so REQ-SEC-002 is enforced server-side.

**Method catalog — 23 methods actually implemented** (class in parentheses):
`system.hello`/`system.describe`/`system.get_schema`/`request.get_status` (Inspect),
`readiness.get`/`identity.get` (Inspect), `thoth.ask` (Command),
`case.entry_options`/`case.preflight` (Inspect), `case.commit` (Command),
`case.list`/`case.get_overview`/`case.get_horizon` (Inspect),
`approval.list` (Inspect), `approval.decide` (Command),
`run.list`/`run.get` (Inspect), `asset.list` (Inspect),
`delegation.preview`/`delegation.list` (Inspect),
`events.subscribe` (Subscribe), `events.unsubscribe` (Command), `events.get_range` (Inspect).

> **Doc drift (recorded, not repaired outside T00 scope):**
> `docs/reference/sfwp-protocol-reference.md` §3 documents **18** methods and omits
> `request.get_status`, `events.get_range`, `case.entry_options`, `delegation.preview`,
> `delegation.list`. A cold agent must treat `sfwp/mod.rs::IMPLEMENTED_METHODS` as
> authoritative and the reference doc as stale. Harmonising the doc is a documentation
> fix for the T04/T05 window, not a contract change.

**Event history / cursors.** `events.subscribe` (with optional `from_cursor`),
`events.get_range(from_cursor, to_cursor, limit)`, and `events.unsubscribe`. The cursor
is the events-ledger `entry_ulid` — an *opaque durable* cursor; an unknown provided
cursor is a typed `events_unknown_cursor` error (never silent truncation). Replay is
capped at `EVENTS_REPLAY_CAP = 500` frames per call and replay is strictly *after*
`from_cursor` (exclusive). `case.get_horizon` folds `plan.json` items over the
append-only `case-events.jsonl`.

**Idempotency / correlation.** Mutations carry a caller `request_id` that acts as an
idempotency key; `sfwp::correlation` returns dedupe verdicts `Replay` / `Pending` /
`Reused` / `Proceed`, with `request.get_status` for pending correlation. **This is the
mechanism T05's duplicate-execution teeth must use** — Go must not invent a second
idempotency authority.

**Settlement visibility.** `case.get_overview` exposes `RunSettlement {run_id, status,
basis, review_required, settled_at}` (settlement standing + cited evidence basis);
`run.get` returns the full six-record episode metadata. `asset.list` is Inspect-only and
covers exactly three families — plan templates, agent endpoints, extensions — and
invents nothing.

**Generated contracts.** `crates/sea-forge-server/src/bin/gen_sfwp_schema.rs` emits one
`<TypeName>.schema.json` per SFWP contract type into
`workbench/packages/contracts/schema/` (Rust-authored intermediate consumed by the
TypeScript generator), with a drift test that fails on unregenerated contract changes.
**This target directory is inside the Workbench package that T14 removes** — per
ADR-006 the schema and its drift assertion must move to the replacement client contract
location *before* Workbench contract files are deleted.

## 4. Capability coverage matrix (spec capability → exact anchor → verdict)

Verdict vocabulary: **PRESENT** (reproducible anchor exists), **PARTIAL** (some of the
required distinction exists but a required part does not), **ABSENT** (no anchor; must
not be fabricated). No capability is marked PRESENT without an anchor.

| Spec capability (domain_contract / REQ) | Anchor in this revision | Verdict |
|---|---|---|
| Case authority, admission, activation (`case.commit`, `case.preflight`, `case.entry_options`) | `sfwp/case.rs`, `lib.rs` dispatch; idempotent via `request_id` | PRESENT |
| Case/plan projection for a world view (REQ-PROJ-*) | `case.list`, `case.get_overview`, `case.get_horizon`, `case_views.rs` | PRESENT |
| Settlement + evidence basis observation (REQ-VERIFY-001/003) | `RunSettlement` in `case_views.rs`; `run.get` six-record episode | PRESENT |
| Human approval authority (REQ-GOAL-006) | `approval.list` / `approval.decide`; SoD refusal | PRESENT |
| Execution observation (REQ-VERIFY-004, `execution_lease`) | `run.list` / `run.get` / `delegation.list` / `delegation.preview` observability | PARTIAL |
| **Work opportunity + execution lease as a governed claim** | No lease/claim/reserve method in the 23; no lease concept in `sea-forge-core` (`lease` matches only `ReleaseGateProof`) | **ABSENT** |
| Event stream + gap recovery (`RECOV-002`) | `events.subscribe(from_cursor)`, `events.get_range`, typed `events_unknown_cursor`, durable `entry_ulid` cursor | PRESENT |
| Temporal context: history/version comparison (`temporal_context`, REQ-TIME-*) | Event-level history exists (bounded 500/call). No typed *object-version* history or supersession query | PARTIAL |
| Cognitive artifacts: bounded disclosure (REQ-ART-001/002) | Artifact records exist inside episode evidence; `asset.list` is Inspect-only and not an artifact API | PARTIAL |
| **Cognitive artifact persistence as durable case material** (REQ-ART-003/004, REQ-SEC-004) | No artifact create/persist method; `asset.list` creates nothing | **ABSENT** |
| Agent grounding (`REQ-AGENT-001/002/003`) | `thoth.ask` (Command) returns claim views with declared standing, freshness, limitations, and "knowledge confers no execution authority" | PRESENT |
| Multi-beat interruptible narration (REQ-NAR-*) | No narration/beat method; a stream would have to ride `events.subscribe` | PARTIAL |
| Repository facts (GitHub) | **PARTIAL — corrected by the T00 tooth run.** The *authority* half exists: `AuthorityAction::GithubPr { has_required_evidence }` (`sea-forge-core/src/types.rs:223`), policy surface `GithubPrSurface` with default `escalate` (`sea-forge-authority/src/lib.rs:541/608`), action-kind `("github_pr","pull_request")`, surfaced through `governed_execution_boundary.rs:416` and the SFWP approvals projection as `github_pr` (`sfwp/approvals.rs:147`). The *execution* half is ABSENT: no SFWP method for repository facts, and no GitHub REST/webhook client in `sea-forge-server`. Repository-fact execution and webhook ingestion (RECOV-002) must therefore be either a reviewed Rust/SFWP addition or an explicitly out-of-scope capability named in the projection as unavailable | PARTIAL |
| Ports/adapters replacement boundary (REQ-ARCH-*) | Not implemented (no Go, no new React app) | **ABSENT** (plan scope) |
| Go operational front end | No `apps/`, no `go.mod`, no Go toolchain | **ABSENT** (plan scope) |
| React cognitive environment (REQ-GOAL-002, REQ-UI-*) | No `apps/godspeed-cognitive-ui`; existing Workbench is the removal target, not the successor | **ABSENT** (plan scope) |
| Donor mechanics (REQ-DONOR-*) | Neither Open MCT nor OpenMontage is present locally (no checkout under `/home/sprime01/projects`) | **ABSENT** (T02 decides) |
| Scene renderer donor | No R3F/Three.js dependency in a new app (existing Workbench has its own renderer stack) | ABSENT (plan scope) |

**Consequences that bind later tasks (not optional):**

1. **The lease is Go-owned coordination only.** SEA Forge has no unclaimed-work queue, no
   atomic claim, and no lease record. `source_truth.operational_coordination` states
   leases never grant semantic authority, so T05 must represent a lease as a Go-side
   coordination record keyed to a *governed* case/run identity and must take its
   duplicate-execution safety from the server's own `request_id` correlation and the
   case/horizon read projection — **not** from a new authority.
2. **T07 (temporal) cannot invent object history.** Version comparison must be built from
   `events.get_range` / `case-events.jsonl` + episode records, or it needs a reviewed
   Rust contract addition. `EVENTS_REPLAY_CAP = 500` bounds each replay window.
3. **T08/T11 (artifact durability) require a reviewed SFWP addition** if UI-preserved
   artifacts must become durable governed material (spec `go_boundary`: "Missing methods
   require a separately reviewed Rust contract change with conformance tests"). T08 can
   settle its ephemeral/local scope without it; REQ-ART-004 cannot be settled by a
   fixture.
4. **T09 (narration) has `thoth.ask` grounding but no beat protocol.** Interruption
   semantics must come from the React/Go core, not from a fabricated SFWP method.
5. **T04 may not import SEA Forge ontology into the Go core**; SFWP view structs are
   wire contracts to be translated at the adapter (`sea_forge_server::sfwp::*` names must
   not become the Go domain model).
6. **T11/T12 will run co-located with a cell.** No remote transport exists, so the Go
   HTTP/SSE boundary is a *local* boundary and the "frontend/agent identity" teeth are
   about the Go boundary, not about TLS.
7. **Removal order (ADR-006 + T14).** The SFWP schema generator target
   (`workbench/packages/contracts/schema/`) and its drift assertion must relocate before
   Workbench contract files are removed; `just workbench-*` recipes and
   `scripts/workbench-e2e-*.sh` must be removed with the GUI, not left as dead entry
   points, and the server/executor assertions they protect must be ported first.

## 5. Gauntlet: engine to retain, TUI to remove

Anchor: `$GAUNTLET_WORKTREE` = `/home/sprime01/projects/gauntlet-godspeed-casework` @ `fe75108`.

- **Engine (retained):** 29 Rust workspace crates — `gauntlet-domain`, `gauntlet-ports`
  (ports incl. `evidence_store`, `state_store`, `observer`, `verifier`, `comparator`,
  `verifier`, `producer_authority`, `human_escalation`, `capability_ledger`,
  `revision_ledger`, `deadline`, `clock`, `id_generator`, `operator_surface`,
  `tool_provider`, `workspace_provider`, `reference_provider`, `policy_provider`,
  `sandbox`, `role_result`, `event_log`, `tokens`, `budget_meter`, `observation_digest`),
  `gauntlet-app`, `gauntlet-arch-tests`, and `gauntlet-cli` (binary `gauntlet`, with the
  `surfaces` feature gating `gauntlet serve` / `gauntlet-adapter-surface-json`).
- **TUI (removal target):** `workbench/` in the Gauntlet repo — Bun + TypeScript +
  OpenTUI (`@opentui/core`, `@opentui/react`, `react`, `react-reconciler`), **162 source
  files** under `workbench/src` (implementation + `*.test.ts` suites + `*.bench.ts`),
  plus `workbench/AGENTS.md`, `workbench/biome.json`, `workbench/tsconfig.json`,
  `workbench/TUI-INSPECTION.md`, `workbench/repair/`, and the build output `workbench/dist`
  (present only after a build; absent from the dedicated worktree, which was created fresh from
  `fe75108`).
- **Entry points to remove with it:** root `justfile` recipes `tui-check`, `tui-test`,
  `tui-lint`, `tui-preflight`, `tui-bench`, `tui-build`, `tui-dev-up`, `tui-dev-down`,
  and `scripts/tui-preflight.sh`.
- **Gate coupling to respect:** Gauntlet's root `just ci` = `check test lint deny
  boundaries ruler`; the TUI gates are deliberately *decoupled* from it
  (`workbench/AGENTS.md` §2). Removing the TUI therefore does not silently weaken the
  Rust closure gate, but every `tui-*` recipe must disappear as a supported build entry
  point, and `just boundaries` (261 checks reported at `main`) must still pass.
- **Ids for correlation:** T05's Execution Port must bind a Go execution attempt to a
  Gauntlet run's own identifiers and evidence path; the plan forbids inventing a parallel
  execution authority or modifying Gauntlet's governing objective.

## 6. SEA Forge removal inventory (Tauri/Rust GUI)

| Artifact | Exact location at `6ce518f` | Size / count |
|---|---|---|
| React renderer | `workbench/apps/desktop/src` | 80 files, 656 KB |
| Rust/Tauri host | `workbench/apps/desktop/src-tauri/src` | 7 files, 120 KB (`lib.rs`, `main.rs`, `bridge.rs`, `drafts.rs`, `events.rs`, `socket.rs`, `supervisor.rs`) |
| Host package | `workbench/apps/desktop/src-tauri/Cargo.toml` | package `sea-forge-workbench` 0.1.0, lib target `app_lib` |
| App manifest | `workbench/apps/desktop/src-tauri/tauri.conf.json` | `productName: sea-forge-workbench`, `identifier: dev.seaforge.workbench`, `frontendDist: ../dist` |
| Host tests | `workbench/apps/desktop/src-tauri/tests` | 2 files |
| E2E | `workbench/apps/desktop/e2e`, `workbench/apps/desktop/e2e-agent-browser` | 2 + 1 files |
| Shared packages | `workbench/packages/{contracts,sea-forge-ui-components,sea-forge-astryx-theme,sea-forge-ui-tokens}` | 208 files total across the four packages; the `contracts` package alone holds 174 files, of which **57** are `schema/*.schema.json` and 115 are under `generated/` |
| Workspace root | `workbench/package.json` (`sea-forge-workbench`), `workbench/bunfig.toml`, `workbench/bun.lock` | — |
| Instructions | `workbench/AGENTS.md`; ADR-004 (superseded for the human interface only at T14) | — |
| Root recipes | `workbench-check`, `workbench-contracts-gate`, `workbench-tauri-test`, `workbench-sidecar`, `workbench-tauri-dev`, `workbench-host-build`, `workbench-package`, `workbench-package-inventory`, `workbench-demo`, `workbench-completion-eval-inputs-check`, `workbench-e2e-agent-browser`, `workbench-e2e-case-authoring`, `workbench-e2e-real` | `justfile` lines ~308–600 |
| Scripts | `scripts/workbench-e2e-agent-browser.sh`, `scripts/workbench-e2e-case-authoring.sh`, `scripts/workbench-e2e-real.sh`, `scripts/check-workbench-completion-eval-inputs.sh`, `scripts/workbench-*.sh` | — |
| Generated contracts to **migrate before** removal | `workbench/packages/contracts/schema/*.schema.json` (57 files) produced by `gen_sfwp_schema.rs`; TS output in `workbench/packages/contracts/generated` (115 files) | 172 files |

A hidden/disabled legacy UI does **not** satisfy REQ-MIG-001; the removal diff and the
post-removal gates (`GATE_REMOVAL`, `just ci`, `just proof`, `just workbench-check`
becoming moot) are the evidence.

## 7. Gates: existing vs planned

| Gate | Command | Activation | Status at T00 |
|---|---|---|---|
| `GATE_SPEC_TRACE` | `python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py` | T00 | **PASS** (worktree, exits 0) |
| `GATE_GO` | `just casework-go-check` | T01 | **NOT IMPLEMENTED** — recipe absent; Go toolchain absent |
| `GATE_UI` | `just casework-ui-check` | T03 | **NOT IMPLEMENTED** — recipe and app absent |
| `GATE_SEAFORGE` | `just check && just test && just ci && just proof` | T05 | baseline recorded below |
| `GATE_GAUNTLET` | `cd "$GAUNTLET_WORKTREE" && just check && just test && just ci` | T05 | recorded below |
| `GATE_INTEGRATED` | `just casework-integrated` | T11 | **NOT IMPLEMENTED** |
| `GATE_REMOVAL` | `just casework-removal-check` | T14 | **NOT IMPLEMENTED** |

## 8. Blocking prerequisites discovered by T00

These are recorded, not worked around. None may be satisfied by fabrication.

**B1 — No Go toolchain (blocks T01, T04, and every dependent task; GATE_GO cannot run).**
`go` is absent from `PATH`; `mise` reports `go@1.27.1 not installed`; no Go binary exists
under `/usr/local`, `/usr/lib`, or the home tree. The spec's `target_layout.go_module` and
`GATE_GO: just casework-go-check` both presuppose a Go toolchain. Resolution requires an
operator decision: install the toolchain (e.g. `mise install go@1.27.1`) or defer all
Go-gated work. Writing Go code that cannot be compiled, vetted, or tested would violate
`rules`: "A task is not complete because code exists".

**B2 — `just security` is RED at the frozen revision (GATE_SEAFORGE precondition).**
`gitleaks detect --no-banner --redact` reports **14** `generic-api-key` findings, all in
commit `6ce518f` (the HEAD commit, authored by the sibling Godspeed bounded-judgment
plan), in `.agents/evidence/godspeed-bounded-judgment/**`
(`T18/observations.jsonl` ×6, `T27/observations.jsonl` ×4, `repeatability/observations.jsonl` ×1,
`T00/gate-SEA_CHECK*.{log,txt}` ×3). **Correction (round 2):** an earlier statement here claimed
each matched value was "8 characters, mixed-case alphabetic, no digits or symbols". That was
measured off gitleaks' `--redact` placeholder (`REDACTED`, 8 characters) and is **withdrawn**.
The corrected, non-exposing measurement reads the real bytes at the recorded column offsets:
11 matched spans are 23 characters, 3 are 32 characters; the matched field is named `key`
(observation records) or `idempotency_key` (gate logs); each value is 13–16 characters of
lower-case alnum with hyphens, and **all 14 contain digits**; entropy is 3.55–3.75. The text is therefore
**identifier-shaped**, not prose — but shape alone cannot prove non-secrecy from a redacted
report, so the operator must confirm the field semantics before any allowlist is written.
The repo already carries a reviewed precedent for exactly this class: `.gitleaks.toml`
has a *commit-scoped + path-scoped* allowlist added during "T17 gate-remediation
(2026-09-18)" for "test fixtures and redaction sentinels with simulated credentials",
deliberately commit-scoped so new occurrences still fail.
Repair (narrow: `commits = ["6ce518f…"]` + exact evidence paths + `rules =
["generic-api-key"]`, with the field-semantics verification recorded) is a **change to the
security gate configuration**, which AGENTS.md requires asking about before implementing.
Until that decision exists, `GATE_SEAFORGE` cannot pass, and T00 records the baseline as
red-with-exact-assertion rather than green or deferred.

**B3 — No donor checkouts.** Neither Open MCT nor OpenMontage exists locally; REQ-DONOR-*
defaults to the plan's "native implementations are valid" path (T02 owns the decision).

**B4 — `docs/reference/sfwp-protocol-reference.md` is stale** (18 documented vs 23
implemented). A cold agent following it would under-count the boundary by five methods.

## 9. Gate baselines recorded by T00

SEA Forge, in the dedicated worktree, `CARGO_BUILD_JOBS=1`, one heavy gate at a time,
sharing the operator checkout's warm `target/` (cache-only effect; no tracked file in the
active checkout is written). Raw logs:
`.agents/evidence/godspeed-casework-cognitive-environment/T00/raw-logs/`.

| Gate | Exit | Wall | Peak RSS | Verdict |
|---|---|---|---|---|
| `just lint` (`cargo clippy --workspace --all-targets --all-features -- -D warnings`) | 0 | 52 s | — | PASS |
| `just typecheck` (`cargo check --workspace --all-targets --locked`) | 0 | 36 s | — | PASS |
| `just test` (`cargo test --workspace --all-features --locked`) | 0 | 290 s | 0.99 GiB (1036928 kB) | PASS |
| `just proof` (spec-minimum §12.2 P1–P4b) | 0 | **3 s** (round-2 re-run, backed by `raw-logs/round2-proof.log`; the round-0 observation of "2 s" was unbacked and is retained only as a disclosed defect) | 72784 kB | PASS — *see correction below* |
| `just security` (`cargo deny` + `gitleaks`) | **1** | 52 s | — | **RED** (B2; `deny` green, 14 gitleaks findings) |
| `just build` | 0 | 70 s | — | PASS |

**Preserved correction round (not erased):** the first `just proof` attempt exited **127**
because the measurement harness set `CARGO_TARGET_DIR` to the operator's warm cache while
the recipe reads the relative path `target/debug/sea-forge`. That is a defect of the
measurement setup, not of the gate. The worktree now carries `target →` the warm cache as
a symlink and `just proof` exits 0 with `[proof] P1-P4b passed`. Both rounds are in the
evidence directory. The same path coupling must be remembered by later tasks that run
`just proof` from a worktree: keep the `target` symlink, or let cargo use the worktree's
own target directory.

`just context-check` is expected to be red until this task's handoff files are updated
(a task-local ordering fact, not a gate failure): `scripts/check-agent-context.sh` fails
when project files changed without a corresponding `CURRENT_STATUS.md` update. The
composite `just check` / `just ci` runs are therefore executed **after** the status update
and recorded in the evidence directory.

## 10. Open decisions that require human authority

1. **B1 Go toolchain** — install (unblocks T01+) or defer Go-gated work.
2. **B2 `.gitleaks.toml`** — approve a narrow, commit-scoped, rule-scoped allowlist (with
   the field-semantics verification recorded — see the round-2 correction in §8) or leave
   `just security` red as a recorded
   precondition on `GATE_SEAFORGE`.
3. **Any new SFWP method** needed by REQ-ART-004 / REQ-VERIFY-004 / REQ-TIME-003 is a
   Rust public-contract change and requires separate review + conformance tests before
   implementation; this brief does **not** authorise it.

## 11. Rules a cold implementer must not break

- Fixture/local adapters may prove UI-core isolation; they are never evidence of governed
  integration or settlement.
- No async runtime/HTTP client may enter the 19 kernel crates (`just no-async-kernel`);
  the Go app is a separate application, not a kernel crate.
- Go and React own their ports; SEA Forge SFWP view structs and Gauntlet/vendor objects
  stop at adapters and never become the application model.
- Settlement is SEA Forge's; a Go lease or a UI projection is never case truth, and an
  exit code is never a settlement.
- `.sea` grammar is not to be changed for UI/transport/runtime features.
- Preserve failed rounds and corrections; append rather than rewrite.

## 12. T00 correction history (append-only)

Three independent verification rounds have been run against T00. Every one returned NOT_CONFIRM,
and every one found real defects — all of them in this brief's or the evidence's *measurement and
bookkeeping* layer, none in the load-bearing claim (authority binding, 86/86 traceability, the DAG,
worktree isolation, the missing-seam inventory, the gate baselines, or the teeth). All rounds are
preserved in `T00-verification-round-{1,2,3}-NOT_CONFIRM.md`.

**Round 0 → Round 1: GitHub capability mis-classified as ABSENT.**
The first draft of §4 recorded "Repository facts (GitHub) → ABSENT" on the strength of a
`grep -rqi 'github' crates/sea-forge-server/src/sfwp/` probe that unexpectedly matched.
Investigating the match instead of dismissing it revealed `AuthorityAction::GithubPr` and the
`github_pr` policy surface. Root cause: *representation* — the probe tested for the string in the
wrong place, and the verdict was written before the match was explained. The corrected row is in
§4; the tooth now asserts **both** halves (authority PRESENT, execution ABSENT) so the error cannot
silently return. The failed probe is preserved in `teeth/teeth-run.log` (round 1 shows
`claim survived inventory (TOOTH FAILED): GitHub casework adapter`).

**Round 0 → Round 1: the tooth harness reported vacuous absences.**
The tooth script's first run derived the worktree root one directory too shallow and so tested paths
that did not exist, which trivially "confirmed" every absence. The script was repaired to derive the
root correctly and to **refuse to report teeth at all** when the plan cannot be located from that
root (`exit 2`). A tooth that can pass for the wrong reason is worse than no tooth.

**Round 1 → Round 2: the gitleaks characterization measured the redaction placeholder.**
The round-0 analysis read the `Secret` field of a `gitleaks --redact` JSON report — the literal
string `REDACTED` for every finding — and concluded "8 characters, mixed-case alphabetic" for all
14. That was a measurement of the placeholder, not of the findings, and it contradicted the column
spans the same report recorded (23 and 32 characters). An independent verifier falsified it. The
corrected measurement reads the real bytes at the reported column offsets and prints no matched
value: 11 spans are 23 characters and 3 are 32; the matched fields are named `key` (observation
records) and `idempotency_key` (gate logs); the values are 13 characters (×3) and 16 characters
(×11) of lower-case alphanumeric-with-hyphen shape, every one containing a digit, entropy
3.5465937–3.75 — record idempotency identifiers rather than prose. The earlier "word-like, obviously
not a credential" reasoning is **withdrawn**; the finding is now "identifiers in a named field, and
the operator must confirm the semantics before any allowlist". Round 0's table is preserved verbatim
as withdrawn in `gate-baseline-seafoerge.md` and in the round-1 record.

**Round 1 → Round 2: four inventory counts and one dangling evidence pointer.**
(a) `crates/` has **22** workspace members, not 23 — the 23 came from counting `crates/AGENTS.md`;
(b) `src-tauri/src` holds **7** files, not 9 — the 9 conflated 7 source files with 2 test files;
(c) `contracts/schema` holds **57** `*.schema.json`, not 174 — 174 is the whole `contracts` package
(115 of its files are under `generated/`); (d) the composite gate results were recorded by pointer to
`gate-baseline-composites.md`, which did not exist yet. All four are corrected here and in the
affected evidence files, and the composites are now actually run and recorded.

**Round 2 → Round 3: two round-1 corrections were only partially applied, and the corrections
introduced defects of their own.** Independent verification round 2 confirmed the corrected counts
and the composites but found that (i) the withdrawn gitleaks wording was still *current* text in
this brief's §8 and §10, the settlement report's B2 row, both canonical handoff files, and the
decision log — all six now replaced, and a grep for it across this plan's artifacts returns nothing;
(ii) the unbacked round-0 "2 s" `proof` time was still displayed and cited — the §9 table now
carries the backed round-2 value (exit 0, 3 s, 72784 kB, `raw-logs/round2-proof.log`) and labels
"2 s" as a disclosed defect; (iii) `teeth/remeasure-gitleaks.sh` printed a field-name *fragment*
(`i` / `k`) rather than the field name it claimed, so its cited log did not show
`key` / `idempotency_key`; (iv) that script would have exited 0 with `findings: 0` on a
present-but-empty report; and (v) the preregistration's mtime postdated its declared `frozen_at`
with nothing to reconcile it. Round 3 replaced the wording everywhere, re-backed the proof time,
fixed the field-name function and the empty-report hole, and disclosed the preregistration write.

**Round 3 → Round 4: the corrections were still incomplete, and one destroyed its own evidence.**
Independent verification round 3 (`T00-verification-round-3-NOT_CONFIRM.md`) confirmed Claims 1 and 2
but found that: the prereg disclosure claimed "no command ran before the freeze", which nothing
establishes, and the round-3 edit *overwrote the mtime* that disclosure was about; the two canonical
handoff files named different "current rounds"; the re-measurement script measured the *span* while
the text described the *value* (so its log could not back the claim), reported nonsense on
whitespace-only columns, crashed on an unreadable file, and defaulted to a `/tmp` report path that
made the citation non-durable; the round-2 record's layout was damaged by an earlier edit; and every
memory figure in this brief and three evidence files carried GB values under GiB labels
(`MemTotal` 8.13 → 7.76 GiB, `just test` RSS 1.04 → 0.99 GiB, `just proof` 0.56 → 0.53 GiB,
`MemAvailable` 3.39–3.83 → 3.24–3.65 GiB, swap 7.3–7.8 → 7.14–7.47 GiB, Gauntlet headroom
3.8 → 3.63 GiB (3809564 kB)). **Note added in round 6:** the round-3 correction as first written gave
the swap low end as 7.05 GiB and the Gauntlet headroom as 3.69 GiB — both **wrong**, coming from the
round-3 verifier's own arithmetic rather than from the raw logs; round 5 replaced them
(7486428 kB = 7.14 GiB; 3809564 kB = 3.63 GiB). The wrong figures are quoted here only so the
correction history is complete and are marked as wrong so they are not reused.

Round 4 fixed the *class* rather than the instances: the script now measures and reports the VALUE
(class, digit presence, hyphen presence) from the real bytes and refuses empty reports,
whitespace-only spans, unreadable files and unresolved values with `exit 2`; its default report path
is inside the evidence directory; every memory figure was recomputed from raw kB with
1 GiB = 1048576 kB and the convention is declared; the prereg disclosure was narrowed to what is
establishable and carries an explicit `not_established` list plus the lesson that a freeze needs a
hash pin rather than only a timestamp; the round-2 record's layout was repaired; and both handoff
files were aligned on one current round and one next action.

**Lesson, and the rule that now applies to the rest of this plan.** A correction is not done when
the *claim* is fixed; it is done when no artifact still asserts the falsified thing, when the cited
raw log can actually show the number, and when the handoff files agree. Rounds 1–3 each cost a full
verification cycle because that audit was done by hand and missed something. Every correction round
now ends with a mechanical self-audit — grep for the withdrawn wording, verify every referenced path
exists, re-derive every numeric claim from its raw log, and confirm the handoff files name the same
round — before any re-confirmation is requested.

**Round 4 → Round 5: the self-audit itself was theatre, and one of its checks never affected its
verdict.** Independent verification round 4
(`T00-verification-round-4-NOT_CONFIRM.md`) held the gitleaks re-measurement and the four refusal
guards, then demonstrated that the audit it was supposed to be gated by could print
`SELF-AUDIT: PASS` with a live defect of **every** class it claimed to cover: withdrawn wording in
a file outside its hardcoded list, line-wrapped wording, a broken citation past a magic 145-line
cutoff, rewritten numbers it never re-derived, a stale round line, an unlisted gate name, and a
deleted evidence file. It also found a live unit error of the class round 3 had flagged
(`3809564 kB` labelled 3.69 GiB; swap low end 7.05 instead of 7.14 GiB — both inherited from the
round-3 verifier's own arithmetic), a stale settlement-report history, a truncated §3 sentence, a
malformed table row, an orphan line, and two false statements that `target` is gitignored.

Round 5 fixed all of it and rebuilt the audit so that its checks are real: the file set is derived
by walking this plan's own artifact directories (other plans under `.agents/` are explicitly not its
business); the plan-owned region of `CURRENT_STATUS.md` ends at an explicit marker instead of a line
number; matching is whitespace-collapsed so a wrapped phrase is caught; every `N GiB (M kB)` pair is
re-derived arithmetically; both handoff files must name the same applied correction round; every
`GATE_*`, `casework-*`, `workbench-check` and `no-async-kernel` name is covered with two-line
negation context; and each named handoff file must exist.

Two defects in the audit were found *by the audit's own self-test* and fixed: the exit status of
checks 1–2 was not propagated (the audit printed FAIL and still exited 0 — i.e. the very failure
mode round 4 described), and the self-test's own injections were mis-quoted (literal `\n`, missing
backticks) so three classes were being tested by inputs that never applied. Non-vacuity is now
demonstrated: `self-audit.sh --self-test` builds a mirror, injects one defect per class, and
requires the audit to fail on all seven — currently **SELF-TEST: PASS**, with the plain audit
**SELF-AUDIT: PASS**. That last statement is the only place in this plan where an audit result is
quoted from the audit's own log, and the log is in `self-audit-selftest.log`.

**The rule this leaves behind:** an audit that cannot fail for the right reason is not evidence.
Any future self-check in this plan must ship with an injection test that proves it fails when the
defect is present, not merely pass when it is absent.

**Round 5 → Round 6: the audit certified an empty file, and hardening it produced the opposite
error.** Independent verification round 5 (`T00-verification-round-5-NOT_CONFIRM.md`) found that
`.agents/reports/godspeed-casework-cognitive-environment/decision-log.yaml` was **0 bytes** — while
this brief, both handoff files and the plan all cited it as holding decisions `D-…-T00-01 … -09`,
and the self-audit reported it present because the check tested existence, not content. It then
proved, by injection into a private mirror, that the audit missed thirteen further classes
(punctuation-variant and re-wrapped wording; non-whitelisted citation forms including `.csv` and
extensionless paths; bare `GiB` figures; gate names outside its list; deleted non-`REQUIRED`
evidence and round records; round claims in files other than the two handoff files; a negation word
anywhere on the line suppressing a gate claim; and content placed inside the instrument itself). It
also showed the injection test credited detection without verifying that its injector had modified
anything, and that an environment variable disabled the non-vacuity cross-check in a real run.

Round 6 fixed the substantive defects: the decision log was restored and now carries a **content**
assertion (non-trivial size, YAML parse, the nine expected ids); the audit detects
punctuation-variant and wrapped wording, `.csv`/extensionless citations, `GiB`/`kB` arithmetic
mismatches, absent and stale round claims, deleted raw logs and deleted verification-round records;
injections must now be proven to have changed the mirror before detection is credited; the
skip-variable loophole was replaced by real mirror detection; and the three unbacked preflight rows,
the stale conversions above, and the settlement report's incomplete index were corrected.

**Then it was narrowed on purpose.** Tightening the prose heuristics produced the mirror-image
failure: `mixed-case` alone fired four times on this plan's own disclosure text, and a rule against
bare prose figures flagged legitimate ranges. So the instrument no longer pretends to police natural
language: it checks **existence, content shape, arithmetic on anchored pairs, round agreement and
its own non-vacuity**, and its header states that `SELF-AUDIT: PASS` means *the mechanical and
arithmetic checks pass* — never *every prose claim in this brief is true*. Gate-claim scanning is
best-effort over unrun gates only; the adversarial round records are excluded from it because they
quote attacks verbatim; and those two limits are printed in the audit's own output.

**The durable lesson, stated plainly:** five adversarial rounds each found real defects, and every
one was in the evidence layer rather than the claim. The claim was stable from round 1; what kept
failing was the builder's ability to describe its own evidence accurately. A future task in this
plan should assume the same and budget for it: write the raw log first, state the number once, derive
it from the log, and never let a phrase stand in for a measurement.

**T00 tooth maintenance after the world changed (round 7).** T00's second tooth began failing once the
operator approved B1 and T01 created the Go module, the React contract package and the `GATE_GO`
recipe: it had recorded those things as *absent*, and they no longer were. The tooth was updated to
assert the current truth — naming each change and the task or approval that caused it — while keeping
the seams that are still absent (`just casework-ui-check`, the SFWP repository-fact/lease/artifact
methods, donor checkouts), and it now reads the DAG from the live status file instead of the plan's
frozen `initial_state`. Lesson for later tasks: a frozen inventory must be *maintained*, and its teeth
must say what changed and why; deleting a tooth because reality moved would destroy the only thing that
noticed.
