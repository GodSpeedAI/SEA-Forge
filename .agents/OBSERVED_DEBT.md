# Observed Debt

Record concrete debt, gaps, risks, or defects found while working but outside
the current task. Each entry needs evidence, impact, and a next move. Remove an
entry when resolved; do not use this file as a backlog of ideas.

## Open: SEA Forge SemanticEnvelope diverges from CEP-0008 flat profile
- Observed: 2026-07-16
- Evidence: `crates/sea-forge-cli/tests/task17_closeout.rs` validates a produced
  `semantic-envelope.json` against the copied CEP fixture. Required CEP fields
  absent are `schema_version`, `event_id`, `source_agent`, `occurred_at`,
  `payload`, and `provenance`; SEA Forge also emits 14 top-level fields rejected
  by the profile's `additionalProperties: false`.
- Impact: SEA Forge envelopes cannot currently be claimed CEP-0008 flat-profile
  conformant or sent directly to consumers of that schema.
- Next move: define a versioned CEP projection/adapter or approve a persisted
  envelope version bump; do not mutate the v0.1/v0.2 envelope in place.
- Scope: Task 17 requires recording, not silently changing, this schema divergence.

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
