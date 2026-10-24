# Current status

**Status revision:** 159

**Stage:** docs-reconciliation-and-interactive-explainer

**Summary:** Documentation refresh published on top of current main. Documentation reconciliation is applied to the stale policy, tutorial, troubleshooting, approval-cycle, persisted-record, and network-boundary pages. The policy how-to matches the actual PolicyRule schema, the first governed-run tutorial uses a working policy bundle and the exact demo intent, approval-cycle and troubleshooting references use the real field names, persisted-record examples match the current JSON, and network-boundary claims state that the 0.1 schema reserves but does not expose a TCP-port dimension. A new standalone docs/explainers directory adds an index and a governed-run explorer with seven scenarios and source links. Verification used actual CLI runs, a relative-link checker, and a headless Chrome render. The earlier Windows-link repair (254 file:/// links across 30 files) is folded in, and the new upgrade-domainforge-producer row on main uses a relative link too.

**Verified:**

- Reconciled `docs/how-to/write-authority-policies.md`, `docs/tutorials/01-first-governed-run.md`, `docs/operations/troubleshooting.md`, `docs/workflows/human-approval-cycle.md`, `docs/how-to/troubleshoot-cell-failures.md`, `docs/explanation/landlock-jail-security-model.md`, `docs/subsystems/sandbox-runtime.md`, `docs/reference/terminology.md`, and `docs/reference/persisted-record-schemas.md` against current source.
- Corrected stale policy fields (`rule_id`, `action_pattern`, `default_disposition`) to the actual `PolicyRule` schema, and replaced the incorrect network-boundary example with the reserved-path reality.
- Corrected the tutorial to create a policy bundle, use the exact `"generate a .sea model"` intent, show the real CLI output, and demonstrate `sea-forge recall`.
- Updated `AuthorityDecision` and `CasePlan` examples to the current persisted JSON shapes.
- Added `docs/explainers/index.md` and `docs/explainers/governed-run-explorer.html`, and linked them from `docs/index.md`, `docs/documentation-map.md`, `docs/workflows/cli-run-lifecycle.md`, and `docs/subsystems/settlement-evidence.md`.
- Executed real CLI runs for accepted, denied, escalated, false-success, nonzero, timeout, and protected-zone scenarios and recorded their exact settlement basis and exit codes.
- A relative-link checker over 108 Markdown and HTML files found zero broken links.
- Headless Chrome rendered `governed-run-explorer.html` with the expected DOM and no JavaScript errors.
- No Rust source, tests, configuration, dependencies, or CI were changed.
- Replayed onto origin/main 061db77: lines main added to docs since the merge base are still present; no file:/// links remain under docs/, README.md or .agents/*.md.

**Limits:**

- No docs-specific automated gate exists; verification is source tracing, real CLI runs, the link checker, and a headless Chrome render.
- `just check`, `just test`, `just proof`, and Workbench gates were not run because this is a docs-only change.
- The HTML explainer was validated headlessly; it has not received a manual graphical-browser review.
- The network-boundary schema gap is recorded as open debt in `.agents/OBSERVED_DEBT.md`.
- Local main's unpushed commit 6e92969 (2026-10-21 branch-reconciliation record and round-4 handoff prompt) is not included; the operator decides whether to publish it.

**Next move:** Review and merge the docs PR; SEA-Forge #48 renumbers its status revision after this lands.

**Evidence:**

- docs/how-to/write-authority-policies.md
- docs/tutorials/01-first-governed-run.md
- docs/reference/persisted-record-schemas.md
- docs/explainers/index.md
- docs/explainers/governed-run-explorer.html
- docs/documentation-map.md
- docs/index.md
- crates/sea-forge-authority/src/lib.rs
- crates/sea-forge-settlement/src/lib.rs
- crates/sea-forge-cli/src/pipeline.rs
- .agents/OBSERVED_DEBT.md

**Spec:** docs/architecture.md

**Ledger:** .agents/OBSERVED_DEBT.md
