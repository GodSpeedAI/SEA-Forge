# Ultracode Mission

## Mission

Complete SEA Forge as a working, usable integrated local product.

A clean Linux host installs one versioned distribution, starts the Workbench
and its matching local server through one documented procedure, completes the
governed representative journey against real records, restarts without losing
committed truth, and inspects every failure with a lawful next action. No
required journey uses a preview, mock, or silent fallback.

## Desired final product state

- **CLI**: the proven minimum lifecycle remains the compatibility and recovery
  path. Existing commands, IDs, exit codes, and P1-P4b proofs are unchanged.
- **Server**: starts fail-closed; invalid config blocks startup; reload is
  atomic; requests time out at 10s with recoverable status; case episodes flow
  through the full governed lifecycle (authority → trace → evidence →
  settlement); one run locator resolves all records.
- **Workbench**: opens to source-backed readiness with resolved identity;
  protected commands carry actor context; mutations are idempotent and
  recoverable; the 12-scenario journey completes through the real stack.
- **Distribution**: a checksummed Linux package and CLI archive, installable on
  a clean host without source checkout. macOS is either verified or explicitly
  unsupported.
- **CI**: aggregates kernel, frontend, host, generated-drift, package, and
  real E2E. Release notes are honest about platforms, migrations, and
  limitations.

## Completion conditions

1. `devbox run -- just check && devbox run -- just test && just workbench-check`
   pass on the integrated tree.
2. `devbox run -- just proof` (P1-P4b) is green.
3. `just no-async-kernel` confirms 19 kernel crates are async-free.
4. The SF-007 contract gate reports zero drift.
5. The SF-012 packaged E2E passes all 12 representative scenarios on a clean
   Linux host.
6. The SF-013 aggregate CI is green and artifacts are checksummed.
7. No required journey uses a preview, mock, or silent fallback.
8. macOS is either verified or explicitly marked unsupported.

See `PRODUCT_COMPLETION_DEFINITION.md` for the full observable contract and
`VERIFICATION_MATRIX.md` for the evidence map.

## Authoritative evidence hierarchy

1. The current user request.
2. The nearest `AGENTS.md` (`workbench/AGENTS.md` governs `workbench/`).
3. `.github/copilot-instructions.md`.
4. Normative specs: `Shell-SPEC.md`, `spec-minimum.md`, `spec-full.md`.
5. Accepted ADRs (`docs/decisions/`).
6. Executable contracts: current Rust types, tests, and behavior.

Plans, status files, README, architecture summaries, knowledge graphs, and
memory artifacts are **evidence, not authority**. When a higher source is
demonstrably stale (e.g., README describes a 2-crate system; reality is 22),
a lower-ranked source can prove the staleness — but normative rules in
`AGENTS.md` are never discarded.

## Non-negotiable invariants

- **Authority before every side effect**, including workspace creation. Denial
  is complete: governance records are written, no mutation occurs.
- **Settlement from criteria + evidence**, never process completion alone.
  Exit-zero with unmet criteria settles Rejected.
- **One authority fabric**: no renderer, bridge, provider, or sandbox bypass.
- **Append-only truth**: JSONL ledgers and canonical records are truth;
  SQLite, JSON views, TS contracts, and UI caches are rebuildable projections.
- **One run locator**: every run ID resolves across CLI, case, SFWP, restart,
  and UI.
- **Kernel isolation**: 19 kernel crates remain synchronous and free of async
  runtimes and HTTP clients. The Tauri Cargo workspace stays standalone.
- **DomainForge boundary**: exact-pinned, side-effect-free; SEA Forge owns
  final authority.
- **Generated zones**: never hand-edited; source changes first, generators
  run, committed output is drift-checked.
- **Compatibility**: minimum CLI record schemas, IDs, exit codes, flat run
  history, SFWP v1, and P1-P4b proofs remain valid.

## Autonomy boundaries

### Agent-autonomous (proceed without asking)

- Internal code organization within accepted crate boundaries.
- Wiring existing lifecycle services into server paths (SF-003 reuses
  `pipeline.rs` patterns; does not build a second pipeline).
- Adding read-only SFWP inspect verbs (additive, ADR-003).
- Test design and verification scripts.
- Vertical-slice ordering within the DAG.

### Ask first (do not proceed without owner authorization)

- Adding or upgrading dependencies.
- Changing persisted schemas, ID grammar, or policy precedence.
- Changing public SFWP contracts beyond additive ADR-003 verbs.
- Editing CI/deployment configuration (`.github/workflows/`).
- Deleting files.
- Changing the DomainForge exact-pinned version.
- Weakening any conformance test.

### Provisional defaults (reversible; proceed until overridden)

- **U-06**: separately installed local server (not Tauri sidecar). The
  SFWP/cell-root/socket contract is identical either way; only process
  lifecycle wiring differs (SF-012).
- **U-07**: minimal internal actor context (not a public identity API).
  Additive fields; inspect verbs remain identity-optional.

If the owner overrides either before SF-012, adjust only the packaging/lifecycle
wiring; no prior packet's contract changes.

## Initial context to inspect

1. `ULTRACODE_CONTEXT_INDEX.md` — navigation, source locations, traps.
2. `MASTER_EXECUTION_PLAN.md` — stage sequence and critical path.
3. `EXECUTION_DAG.json` — pick the highest-priority unblocked packet.
4. The packet file under `work-packets/`.
5. The packet's `current_state_evidence` lines in source.
6. `ARCHITECTURAL_TRUTH.md` and `ARCHITECTURAL_INVARIANTS.md` for the "why".
7. `AGENTS.md` (root) for commands, conventions, and safety boundaries.

## Final verification requirements

The run is complete only when ALL of the following hold:

```sh
devbox run -- just check                    # exit 0
devbox run -- just test                     # exit 0
devbox run -- just proof                    # P1-P4b passed
just no-async-kernel                        # 19 kernel crates ok
just workbench-check                        # frontend + drift + boundary
cd workbench && bunx playwright test e2e/ --workers=1   # 12 real scenarios
```

Plus: clean-host Linux install proof, checksummed artifacts, honest release
notes, macOS verified or explicitly excluded, and no required journey using a
preview/mock/silent fallback.

If any gate cannot pass without violating a non-negotiable invariant or
requiring an unauthorized contract change, abort and surface to the owner.
