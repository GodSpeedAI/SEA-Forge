# T05 Independent Adversarial Confirmation Record
- Date: 2026-09-17
- Verifier: fresh independent context; builder narrative excluded
- **Verdict: CONFIRM**

Independent work (all in /tmp):
- Minted its own two-run cell with its own policy/intents; inspected the ledgers
  directly: settlement_event.authority_refs now resolve to real authority_decision
  ULIDs in the same ledger; each decision cites its request by ULID; subjects
  cross-check; payloads/views untouched exactly as the prereg declares; entries
  hash-chained.
- From-scratch joiner attacks: cross-run citation rejected mechanically on
  committed subject_refs (merged-index ULID resolves, join still refused);
  missing hop named exactly, and the RUNTIME ITSELF detects a tampered ledger
  (`ledger_integrity_error: ordinal gap at 13` on the next mint); rename attack
  (all run subjects renamed to dissimilar strings) leaves every join identical —
  no join depends on string values; inconsistent rename rejected as mismatched
  pair; VAR-008 append-only immutability holds with no rewrite path existing.
- No false join achieved in any attack.
- Gates re-run: core 0, joinability 0 (8/8), cli 101 with EXACTLY the one
  pre-existing t13_1 failure name matching the recorded attribution.

Residuals (recorded, not falsifiers):
  1. plan_pipeline.rs:802 (`run --plan` settlement) still cites decision_id
     labels — the same defect class T05 fixed, in a sibling pipeline outside
     the preregistered delta. Filed in OBSERVED_DEBT.md.
  2. Views remain mutually string-joinable (projections); a joiner ignoring
     the ledger would not see a ledger defect — the ledger join fails loudly
     and the runtime verify path flags tampering.
  3. The earliest hops (request -> intent/plan) bind by committed subject_refs
     exact-match rather than ULID citation — deterministic and heuristic-free,
     but the prereg's "already carries a causal chain" phrasing is looser than
     the actual mechanism there.
