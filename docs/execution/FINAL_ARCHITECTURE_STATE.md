# Final Architecture State

Status date: 2026-07-30. Branch: `ultracode/sea-forge-completion`.

This describes what the repository *is* after this pass, not what any external
description says it should be.

## Shape

Three build roots, deliberately not one:

| Root | Toolchain | Contents |
|---|---|---|
| `.` | cargo 1.92.0, edition 2021 | 22-crate Rust workspace — the kernel, CLI, and local server |
| `workbench/` | bun 1.3.14 | Bun workspace — desktop app, UI components, UI tokens, generated contracts |
| `workbench/apps/desktop/src-tauri/` | cargo, **standalone workspace** | Tauri host (ADR-004) |

The third root is standalone on purpose. If its `[workspace]` table were
removed, cargo would walk up and adopt the repository root, pulling Tauri's
async dependency tree into the kernel workspace and breaking BUILD-01. That is
now checked rather than documented — see *Gates* below.

## Invariants and where they are enforced

| Invariant | Meaning | Enforced by |
|---|---|---|
| AUTH-01 | Authority precedes every side effect, including workspace creation | `case_dispatch.rs` / `case-runner/src/lib.rs` create `workspace/` and `artifacts/` only inside the Allow branch; `conformance_case_episode::a_denied_episode_creates_no_workspace_and_no_artifacts` |
| BUILD-01 | 19 kernel crates are async-free; Tokio only in `sea-forge-server` and `sea-forge-agent` | `just no-async-kernel`; the Tauri boundary check in `just workbench-contracts-gate` |
| DATA-01/02 | One canonical locator across CLI, case, SFWP, restart, UI | `run_views::run_dir` resolves both layouts; `conformance_run_locator` (6 tests) |
| DOM-01 | Settlement evaluates criteria and evidence, never exit code alone | `sea_forge_settlement::settle` is the only settlement path; `a_zero_exit_without_the_declared_artifact_is_rejected` |
| DOM-02 | Every episode leaves a trace and evidence | `case_dispatch.rs` writes `trace.jsonl` / `evidence.jsonl`; `an_allowed_episode_records_its_trace_and_evidence` |
| DOM-03 | A DomainForge candidate is evaluated before authority | `bundle.load_domainforge_model()` → `DomainForgeCandidate::evaluate` in both episode paths |
| K-04 | Existing flat run history stays readable | `run_dir` probes flat first; `both_layouts_resolve_through_one_run_get` |
| K-06 | The kernel gate acquires no Bun dependency | `workbench-contracts-gate` hangs off `workbench-check`, not off `check` or `ci` |
| GEN-01 / API-01 | Rust types are canonical; schemas and TS are committed projections | `conformance_sfwp::generated_schemas_are_committed_and_current` (Rust→schema); `just workbench-contracts-gate` (schema→TS/AJV, tokens) |

`K-02` appears in SF-002's and SF-003's `architectural_invariants` lists but is
not defined in `docs/execution/ARCHITECTURAL_INVARIANTS.md`. It is a dangling
reference and enforces nothing. Recorded rather than silently dropped.

## The cell contract

One resolution order, shared by server, CLI, and desktop
(`crates/sea-forge-server/src/config.rs`):

```
SEA_FORGE_SOCKET   → absolute socket path, wins outright
SEA_FORGE_ROOT     → cell root
default            → `.sea-forge` relative to CWD (server, CLI)
                     `$HOME/.sea-forge`        (desktop)
```

`server.yaml` lives *inside* the cell it configures, which is why
`ServerState.root` is fixed for the process lifetime and kept out of the
reloadable snapshot: a reload cannot relocate the cell that contains the file
being reloaded.

## Governed episode lifecycle

Both episode paths — the server's `execute_sandbox` and the case-runner's
`run_stage_episode` — now run the same sequence:

```
create run_dir (governance records only)
  → trace.jsonl, evidence.jsonl opened
  → DomainForge candidate evaluated               (DOM-03, pre-authority)
  → authority evaluated                           (AUTH-01)
  → trace AuthorityEvaluated + evidence AuthorityDecision
  → decision committed to the ledger
  → Escalate?  ApprovalRequest committed + approvals.jsonl; nothing runs
  → Deny?      trace RunHalted; nothing runs
  → Allow?     workspace/ + artifacts/ created, grant minted, execute,
               trace CommandStarted/CommandFinished, evidence ExecutionResult
  → sea_forge_settlement::settle over criteria + evidence   (DOM-01)
```

The run directory is created *before* authority because a denial's trace and
evidence have to land somewhere; a denial that recorded nothing would be
indistinguishable from an episode that never happened. The directories the
operation itself would touch stay unborn until a committed Allow.

The CLI path (`cli/src/pipeline.rs`) creates `workspace/` before authority as
part of its run-record scaffold, and `justfile:386-389` pins that by asserting
a denied CLI run still has an empty `workspace/`. The server is the stricter of
the two. This divergence is deliberate and documented in SF-003, not an
oversight.

## Run layouts

Two, both permanent:

- `<root>/runs/<id>` — minimum CLI, and what `just proof` reads.
- `<root>/cases/<case>/runs/<id>` — case episodes (spec-full.md:659).

`run_views::run_dir` probes flat, then case-owned, then falls back to the flat
path so an absent run stays "no such run" rather than becoming "malformed id."
A duplicated id resolves to flat in both `run_dir` and `run_dirs`, so a link
and the list it came from can never open different records.

## Gates

| Command | Covers |
|---|---|
| `just check` / `just ci` | kernel: fmt, clippy, workspace tests, `no-async-kernel` |
| `just proof` | spec-minimum §12.2 P1-P4b, against the real CLI |
| `just workbench-check` | Bun install/check/build/test, and now depends on ↓ |
| `just workbench-contracts-gate` | TS/AJV zero-diff, UI token drift, Tauri workspace boundary |

The Rust→JSON-Schema zero-diff step lives in
`conformance_sfwp.rs:625` and is deliberately not duplicated in `just`: two
copies of the same gate can disagree.

## Relationship to the wider stack

Four external boundary documents describe how SEA Forge sits alongside
Context Kernel, SWE_SEED, and GodSpeed-Agent. They are context, not this
repository's canon, and no code was changed to conform to them. Two
observations worth recording:

1. **The word "settlement" is used in two senses.** In those documents,
   settlement classification belongs to GodSpeed-Agent
   (`SWE_SEED_to_godspeed_agent.md:64-94`). In this repository,
   `sea-forge-settlement` evaluates a run's *declared acceptance criteria*
   against the evidence it produced — which maps to SWE_SEED's "work-contract
   pass or failure," not to GodSpeed's developmental settlement. Same word,
   different layer. Renaming the crate to remove the collision would be a
   large, purely cosmetic change and was not attempted.

2. **SEA Forge does not rank context or select routes**, which is the
   boundary those documents care about most
   (`agentic_loop_boundaries.md:733-743`). Nothing in this pass moved it
   toward doing so. The one place it comes close — `sea-forge-thoth` — was not
   touched.

Whether the naming should change is a question for the owner of those
boundaries, not something to settle inside a completion pass.
