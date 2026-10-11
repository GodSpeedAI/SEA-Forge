# Ultracode Readiness Report

## Verdict: READY WITH EXPLICIT RISKS

The work package is complete, evidence-backed, and verified against current
source. Two user-decision checkpoints (U-06, U-07) proceed on reversible
provisional defaults and do not block execution. The highest-risk packet
(SF-003) is precisely scoped with a proven reuse target.

## Reasons

1. **Baseline confirmed this pass**: fmt clean, core checks, P1-P4b proofs
   green, workbench check passes. The repository is at the inspected commit
   with no material drift.
2. **All critical claims verified against source**: the case_dispatch
   pre-authority side effect (line 520 before 530), exit-code settlement
   (lines 587-594), run_views flat-only locator (line 366), config fallback
   (main.rs:24-30), discarded reload (lib.rs:1455-1461), fabricated identity
   (router.tsx:38-51), and missing dedupe (correlation.rs:81-93) are all
   confirmed by direct line-level inspection.
3. **The canonical lifecycle to reuse exists**: `cli/src/pipeline.rs:474-599`
   implements per-operation authority with DomainForge candidates, trace,
   evidence, and criteria-based settlement. SF-003 wires the server to reuse
   it, not rebuild it.
4. **No packet is blocked by an unanswered user decision**: U-06 and U-07 have
   reversible provisional defaults that preserve the SFWP/record contract
   either way.
5. **Dependency graph is acyclic and conflict-free**: no two parallel packets
   compete for sole-ownership files; high-conflict files have explicit
   sequencing rules.

## Remaining blockers

None that require user input before starting. The following are scoped risks
within packets:

| Risk | Packet | Mitigation |
|---|---|---|
| SF-003 rewrites the highest-risk code path | SF-003 | Sole file ownership; reuse proven CLI pipeline; P1-P4b gate; denial-no-side-effect test |
| SF-009 merges 5 foundation packets into one slice | SF-009 | Sequential; all dependencies must land first; per-layer tests gate the merge |
| SF-012 requires a clean host and real packaging | SF-012 | Linux primary; macOS marked "not run" until Seatbelt passes |
| CI changes require owner authorization | SF-013 | Package the change; apply only when authorized |

## Baseline results (this pass)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0, clean |
| `cargo check -p sea-forge-core` | exit 0, 44.86s |
| `devbox run -- just proof` | exit 0, P1-P4b passed |
| `cd workbench && bun run check` | exit 0, 1 known warning |
| `git status` | clean except CURRENT_STATUS.md and jolli debug log |

Pass 1 additionally confirmed: 776 Rust + 133 TS tests green, workspace check
clean, live SFWP smoke, Tauri host build. See `VERIFICATION_BASELINE.md`.

## Highest-risk packets

1. **SF-003 (Canonical episode pipeline)** — the central invariant fix. Touches
   the most critical code path. Mitigated by: the correct pattern exists in
   `pipeline.rs`, sole file ownership, and P1-P4b as a regression gate.
2. **SF-009 (Case-to-settled-run slice)** — the integration chokepoint. Depends
   on 5 prior packets. Mitigated by: sequential execution and per-layer test
   gates.
3. **SF-012 (Packaged E2E)** — environmental risk (clean host, packaging).
   Mitigated by: Linux primary, macOS deferred.

## Expected critical path

```
SF-001 → SF-005 → SF-006 → SF-009 → SF-011 → SF-012 → SF-013
```

SF-003 (parallel with SF-001→SF-005) is the highest-risk but does not extend
the critical path length; it gates SF-009 alongside SF-005/SF-006.

## Recommended initial parallelism

After SF-001 lands its cell-root/socket contract, start four lanes:

| Lane | Packet | Rationale |
|---|---|---|
| A | SF-003 | Highest risk; start immediately; sole ownership of case_dispatch.rs |
| B | SF-002 | Server lifecycle; parallel with SF-003 (different files) |
| C | SF-005 → SF-006 | Identity then idempotency; both edit SFWP request layer |
| D | SF-007 | Contract gate; independent; small; unblocks SF-008 |

## Recommended integration cadence

- **After each foundation packet (SF-001 through SF-007)**: run
  `cargo test -p sea-forge-server` + `just proof` to confirm no regression.
- **After each SFWP slice (SF-008 through SF-011)**: run `just workbench-check`
  + the contract generation gate to catch drift.
- **Before SF-012**: run the full gate sequence
  (`just check && just test && just proof && just no-async-kernel && just workbench-check`).
- **After SF-012**: clean-host verification.
- **SF-013**: owner-authorized CI/release changes only.

## Work that should NOT be delegated concurrently

| Work | Reason |
|---|---|
| Any two packets editing `case_dispatch.rs` | SF-003 sole ownership |
| SF-005 and SF-006 simultaneously | Both edit SFWP Request enum and bridge.rs |
| SF-002 and SF-006 simultaneously | Both edit lib.rs dispatch |
| Two SFWP slices adding verbs simultaneously | Both extend IMPLEMENTED_METHODS and generated contracts |
| Any packet and a CI workflow change | CI changes are ask-first; package separately |

## Deferred work (explicitly out of scope)

- Unmapped SFWP catalog methods outside the representative journey.
- Windows/mobile, remote HTTP, multi-user auth, containers, Kubernetes, hosted
  services, MicroVM, NATS, marketplace.
- Real ACP/SWE_SEED/witness/IFL tests for packages not advertising them.
- crates.io publication of internal libraries.
- Optional CopilotKit adapter.
- `working/face/` deletion (owner decision, cosmetic).

None of these conceal incomplete production paths; all are explicitly
permitted deferrals per `PRODUCT_COMPLETION_DEFINITION.md`.
