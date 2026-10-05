# Stage 11 settlement — production hardening and the end-to-end proof

Date: 2026-10-05. Stages 11 and 12 were never defined in writing; the operator asked for "production ready", and I
defined 11 as hardening plus one end-to-end proof, and 12 as the final report (`final-report.md`).

## What was done

| Item | Change | Debt |
|---|---|---|
| World on RealityTrace's legacy wire | ProofCompleted ingestion requires a pinned `world_ref` equal to the deployment's and carries it; EvidenceRecorded stamps it and refuses on a world mismatch. GodSpeed-Agent refuses E8 evidence with no world when a world is pinned (unpinned deployments still record `unbound`). | M-38 closed |
| Legacy E4–E6 chain | Decision: a contract library, not a server door (a second entry gives one decision two paths). `accept_verified_governed_work_request` recomputes the world against a `WorldRegistry`; the syntax-only entry point is `#[deprecated]`. | M-37 closed |
| Context Kernel at the CEP boundary | cep profile `godspeed.context_bundle` (partial or empty retrieval is never complete; omissions mandatory below complete) with Python, Rust and TypeScript validators on one corpus; `ck-mcp::cep_bundle` projects packets onto it and refuses unexplained incompleteness. | new M-49 |
| CI | Cognate and GodSpeed-Agent had none: added. SEA-Forge CI ran only for PRs into `main`, so stacked work merged untested: now any base. Fixed what that exposed (below). | new M-50 |
| End-to-end proof | `scripts/e2e-world-loop.sh` (`just e2e-world-loop`). | new M-48 |

## The end-to-end proof

GodSpeed-Agent E1 → Context Kernel E3 (and its CEP bundle, validated by cep) → SWE_SEED E4 → SEA-Forge verified
intake, real authority decision, E5A/E5B, E6 → SWE_SEED E7 → RealityTrace E8 → GodSpeed-Agent records it `bound`.
Each hop is a real production surface reading the previous hop's actual output. The world is what the DomainForge CLI
computes for the demo source; SEA-Forge independently recomputes it and must agree. The refusals are asserted by
reason, not just by failure: an E4 in another world than its context packet ("context packet: expected world…"), a
world SEA-Forge never registered ("unknown world…"), a denied request (no invocation, nothing settled), and E8 from
another world or with none. Simulated: E5B execution (real execution is Cognate's live gate).

While building it, the first version of the "refused" check passed vacuously (a tamper script truncated the file it
was about to read, so SEA-Forge failed on a parse error). Caught by requiring the refusal to name its reason.

## Defects found by running this for real

- **SWE_SEED gateway audit log (real bug).** `writeln!` on an unbuffered file is two writes, so concurrent requests
  could fuse two audit records into one line and lose one. Found because CI hit
  `pool_keeps_governance_counts_exact_under_concurrency` (expected 4 allow records, got 3). Fixed with one buffer and
  a lock; the new regression test fails 3/3 on the old code with fused lines and passes now.
- **Two tracked files differing only by case** (`.agents/plans/TODO.md`, `todo.md`) made every macOS checkout show one
  modified. Renamed the lowercase one (different content, no references).
- **gitleaks in CI** reported 14 hits in historical evidence that commit-fingerprint ignores cannot cover on a
  shallow PR checkout; replaced by a path-and-rule allowlist for the same inspected false positives.
- **RealityTrace test race:** two helpers wrote to a child's stdin and panicked on a broken pipe when the child
  rejected its arguments early.
- The workbench's own `Cargo.lock` still pinned `domainforge-core` 0.16.0 while the crates it links require
  `=0.19.0`; regenerated (it only surfaced when CI built the workbench on this base). Two further lint errors that
  only the workbench and macOS jobs reach (needless borrows in `bridge.rs`; an import unused off Linux) are fixed.
- SEA-Forge CI lacked `bun` and the Tauri system libraries for its workbench jobs (never exercised on a stacked base).

## Verification (on the branch heads below)

SEA-Forge workspace 1197/0 (clippy `-D warnings` clean, gitleaks clean); Cognate `just verify` 78/78 and
`just sea-forge-live` 33/0 against the real server, DomainForge CLI, cep validators and RealityTrace; Context Kernel
237/0; SWE_SEED 443/0; RealityTrace 201/0; GodSpeed-Agent 1132 passed/0 failed (5 skipped); cep Python 149/0 (10
skipped), TypeScript 67/0, Rust 10/0; `scripts/e2e-world-loop.sh` OK.

## Not done, on purpose

M-46 (a transition moves nothing) is a design decision, not a defect: nothing breaks while transitions are recorded
and not consulted. Recommendation: make an approved transition the only way a request may name a lineage from another
world, enforced in SEA-Forge. See `final-report.md`.
