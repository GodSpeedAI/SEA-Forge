# Observed Debt

Record concrete debt, gaps, risks, or defects found while working but outside
the current task. Each entry needs evidence, impact, and a next move. Remove an
entry when resolved; do not use this file as a backlog of ideas.

## Open: settlement_id is not globally unique
- Observed: 2026-07-16
- Evidence: `crates/sea-forge-settlement/src/lib.rs:120` hardcodes `settlement_id: "set_01"`.
  `validate_ledger_references` does an unscoped global `require_record`/`find`
  (`crates/sea-forge-artifact-ip/src/lib.rs:2783`, `2120`), so chained CLI
  transitions (synthesize→productize→capitalize) collide on the same settlement.
- Impact: blocks `conformance_m8_cli_capitalize_commits_one_pending_and_approval_then_exits_5`
  (cr.md finding 15's test) from passing green; flaky productize failures.
- Next move: make `settlement_id` unique (ULID/counter), or scope the lookup by run/case.
- Scope: out of cr.md review fixes; product-code change requiring care.

## Open: approve resolution hardcodes sequence, colliding with the escalate decision_id
- Observed: 2026-07-16
- Evidence: `crates/sea-forge-cli/src/commands/approve.rs:295` hardcodes `sequence: 1`.
  `decision_id = format!("auth_{sequence:02}")` (`crates/sea-forge-authority/src/lib.rs:1937`),
  so the approve-resolution decision reuses `auth_01` and duplicates the capitalize
  escalate decision. `resume`'s `unique_entry` (`resume.rs:640`/`992`) then sees two
  `auth_01` records → `ledger_integrity_error: expected one authority_decision record for auth_01`.
- Impact: root cause of the pre-existing line-740 capitalize-test failure; blocks cr.md
  finding 15 green.
- Next move: derive the approve sequence from the existing case-ledger decision count.
- Scope: out of cr.md review fixes; touches decision_id derivation.

## Open: M8 artifact-ip capitalize/escalation tests fail (consumption dedup)
- Observed: 2026-07-16
- Evidence: `cargo test -p sea-forge-artifact-ip` → 9 failures, all at
  `tests/conformance_m8.rs:724` (`grant_after_approval` →
  "approval resolution was already granted"). The crate was non-compiling
  (`ReviewStatus::Pending/Rejected` mismatch at `src/lib.rs:2008`, fixed this session)
  so there was no prior green baseline. Failures are in the capitalize/escalation
  path, upstream of and unrelated to the cr.md review fixes applied.
- Impact: cannot run a fully green artifact-ip suite; the capitalize approval-grant
  consumption dedup interacts with the shared `case-governance` test stream.
- Next move: reconcile `ledger_governance` + `authorized_transition` helpers with the
  ledger-backed approval-grant consumption (finding 8's feature) so a single escalate
  grant does not self-conflict.
- Scope: pre-existing in untracked M8 test infra; not a cr.md finding.

## Open: SodRule cannot scope a transition separation-of-duty rule to one transition_kind
- Observed: 2026-07-16
- Evidence: `SodRule` (`crates/sea-forge-authority/src/lib.rs:429`) matches only on
  `operation_kind`; all three transitions share `operation_kind: "transition_artifact_stage"`
  (`crates/sea-forge-artifact-ip/src/lib.rs:1763`). Escalation is
  `requires_approval || sod_requires_approval` (`authority/src/lib.rs:1780`), so an
  unscoped transition SOD rule forces synthesize/productize to escalate too.
- Impact: cr.md finding 18's transition SOD rule cannot be added without regressing the
  non-capitalize transitions; only its identity_bindings + R-SO resolve-approval parts
  were applied.
- Next move: add a `transition_kind`/`to_stage` field to `SodRule` (and matching logic)
  so a SOD rule can target capitalize only.
- Scope: schema change to the authority bundle; out of cr.md review fixes.


<!-- Entry format:
## Open: Short problem statement
- Observed: YYYY-MM-DD
- Evidence: file, line, command, or failing proof
- Impact: concrete cost or risk
- Next move: smallest credible resolution
- Scope: why it was not fixed in the discovering task
-->
