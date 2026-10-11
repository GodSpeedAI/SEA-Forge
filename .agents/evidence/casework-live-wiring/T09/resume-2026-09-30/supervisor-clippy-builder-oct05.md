# Supervisor test Clippy cleanup builder record — 2026-10-05

## Scope and authority

This is a source-only builder record for the narrowly authorized repair in
`crates/sea-forge-server/tests/sfwp_supervisor.rs`, following
`supervisor-clippy-root-assignment-oct05.md`,
`delegated-identity-clippy-independent-approval-oct05.md`, and its full
workspace Clippy raw/exit evidence. The assigned diagnostics were
`clippy::needless_update` at the `enabled_supervisor` fixture and
`clippy::needless_borrows_for_generic_args` at the two event-file reads in
`supervisor_enabled_auto_advances_ready_sandboxed_task_without_a_client`.

The applicable repository instructions and crate instructions were read, along
with the Graft skill, nearby supervisor tests, and the `SupervisorConfig`
definition/default. `SupervisorConfig` has exactly four fields:
`enabled`, `poll_interval_secs`, `max_concurrent_cases`, and `actor`.
`enabled_supervisor` supplies all four explicitly, so the struct update with
`SupervisorConfig::default()` was redundant.

## Change

The complete diff from baseline is exactly three line removals in the one
authorized test file:

* Removed `..SupervisorConfig::default()` from `enabled_supervisor`; its four
  explicit field values are unchanged.
* Removed the two needless borrows around `case_events_path(...)` passed to
  `fs::read` in the named test.

No assertions, expected values, actor/authority identity, timing, setup,
configuration values, or test control flow changed. No other tracked source,
test, configuration, dependency, status, debt, or Git state was edited by this
builder task.

## Source identity and verification

Baseline at `HEAD`:

`crates/sea-forge-server/tests/sfwp_supervisor.rs`
SHA-256 `cb4e760df11116391f2b7ffc588981cb73ff1a2bbd9a18945b187bca5a07bff5`

Builder result:

`crates/sea-forge-server/tests/sfwp_supervisor.rs`
SHA-256 `75404483a0c166191f8b896f4523461c1b7fd489f20cd1b6357bfebb27be8bf3`

`rustfmt --edition 2021 --check crates/sea-forge-server/tests/sfwp_supervisor.rs`
completed with exit code 0. No compiler, Cargo command, or test was run; this
record is not an independent review or runtime/test approval. The file is
frozen for independent review and the separately authorized verifier gate.

## Material deviations

None from the bounded task. The only behavioral source differences are the
three lint-only syntax cleanups listed above; all test semantics and runtime
configuration remain as before.
