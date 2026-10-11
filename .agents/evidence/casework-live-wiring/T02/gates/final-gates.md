# T02 final gates — 2026-09-23, branch casework/live-wiring (uncommitted tree)

Every log in this directory ends with its own `EXIT=<code>` line captured
after the command itself (`{ cmd; echo EXIT=$?; } > log 2>&1`; no pipe in
front of the capture). Earlier numbered logs (`*-1..4.log`,
`unit-identity*.log`) are the TDD history of the previous iterations; the
`*-final.log` files below are the gates on the final tree. The three
delegation-bearing logs were rerun once more after a print-only addition to
the SoD-negative test (verbatim-refusal evidence line); both runs exited 0
and the recorded logs are from the final tree.

| gate | command | exit | log |
|---|---|---|---|
| plan gate 1 | `cargo test -p sea-forge-server identity` | 0 | `unit-identity-final.log` (22 unit tests in `identity` + identity-named tests elsewhere, all ok) |
| plan gate 2 | `cargo test -p sea-forge-server --test sfwp_delegated_identity` | 0 | `delegated-identity-final.log` (9 passed; 6 carried over + 3 added) |
| plan gate 2 (glob form) | `cargo test -p sea-forge-server --test '*delegat*'` | 0 | `delegat-glob-final.log` (conformance_delegation_preview 14 + conformance_delegations 12 + sfwp_delegated_identity 9) |
| prereg global gate | `cargo test -p sea-forge-cli -p sea-forge-case-runner -p sea-forge-server` | 0 | `workspace-cli-runner-server-final.log` (58 test binaries ok) |
| formatting | `cargo fmt --all -- --check` | 0 | `fmt-check-final.log` |
| dependency boundary | `just no-async-kernel` | 0 | `no-async-kernel-final.log` ("ok: no async runtime or HTTP client in 19 kernel crates") |
| T04 regression | `cargo test -p sea-forge-server --test sfwp_case_mutations` | 0 | `case-mutations-regression-final.log` (10 passed) |
| unit-C regression | `cargo test -p sea-forge-server --test sfwp_supervisor` | 0 | `supervisor-regression-final.log` (4 passed) |

## Interlude during this run

The first execution of the three-crate gate exited 101:
`conformance_sfwp::generated_schemas_are_committed_and_current` failed with
"schema drift in IdentityView.schema.json" — the prior T02 iteration added
`effective_actor` (and `EffectiveActorView`/`DelegatedByView`) to the
`JsonSchema`-deriving `IdentityView` without regenerating the committed
schemas. Repaired by running the generator the failure message names
(`cargo run --locked -p sea-forge-server --bin gen_sfwp_schema`, the Rust
half of `just workbench-contracts-generate`): exactly one file changed,
`workbench/packages/contracts/schema/IdentityView.schema.json`, +52/−0,
purely the additive T02 field and its two new `$defs`. The gate was then
rerun from scratch and is the EXIT=0 recorded above.

The TS/AJV projection half of `workbench-contracts-generate` (`bun run
generate:contracts`) is a workbench gate and was NOT run: workbench is out
of T02's scope for this task. Flagged for the workbench gate owner.

## Skipped gates, with reasons

- `sea-forge-planner` tests (part of the plan's first global gate list):
  not in this task's gate list; no planner source changed (verified:
  `git status` shows no planner files modified).
- Go / bun global gates (`apps/godspeed-casework-go`, `apps/godspeed-cognitive-ui`):
  out of scope for this task and untouched (`apps/` unmodified); T02 must
  not change the SFWP wire shape and did not (additive optional
  `on_behalf_of` sibling parsed off the raw line, per ADR-003).
- `just proof`, `just ci`, workbench gates: not in this task's gate list.
