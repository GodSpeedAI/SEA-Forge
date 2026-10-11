# T25 IN-FLIGHT — Matched negative + payment evaluation against the manual baseline

- Builder: B1
- Plan: `.agents/plans/godspeed-bounded-judgment-plan.yaml` v1.5.0, task T25
- Contract authority (frozen, read-only; imported/read, NEVER modified):
  - `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml`
    (sha256 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5)
    + addenda 1-3 (frozen sha256s; addendum-1 matched_negative_mutation:
    payment capacity only, r* -> r*-1, no other field changes; the negative
    must actually fail to settle; its label is read from the run_terminal,
    never assigned)
- Dependencies: T21/T22/T23 SETTLED; T24 COMPLETE (accepted generated case
  t24-case-001 = e2e-happy@4, independent replay settled with 3
  settlement_committed events; T24 gate PASS 154 checks, exit 0).

## Scope of this task

Author run_negative.py: take the ACCEPTED T24 case and mutate EXACTLY ONE
factor — payment capacity: max_rounds 4 -> 3 (the discovered r*-1) — run the
REAL mechanism on a fresh copy with a fresh seed, require the actual
non-settlement consequence, record the matched pair (positive = the T24
replay run reference; negative = the new run with its own pinned DB).
Author verify_payment.py (gate): recompute the negative's label from its
pinned DB (fail if it settled or errors); verify the single-factor mutation
by diffing the case configs; assemble payment-comparison.yaml (this branch's
RECORDED counters beside the manual T15/T16 baseline as recorded; unobservable
quantities n/a, no estimates) and novelty.yaml (frozen fingerprint metric
against the T15/T16 grids as recorded, incl. verifying the T16 expansion never
probed fixture-target budget levels 5,6,7); execute tooth T8 (regenerate a
structurally equivalent route record; the novelty computation must DETECT the
structural duplicate; measured, not auto-rejected); fail-closed manifest hash
checks. Exit 0 only if all pass.

## State

- [x] Authoritative docs read in order (plan T25, prereg + addenda, T24
      records incl. accepted-case.yaml + replay-record.yaml, T22/T23 payment
      counters, T15/T16 corpus manifests/records, decisions.yml).
- [x] IN-FLIGHT.md + b1-progress.md authored.
- [x] run_negative.py authored, then executed: single-factor matched negative
      (max_rounds 4 -> 3, everything else identical) through the real
      mechanism (fresh target copy, budget sed 4->3 verifiable in the
      durable copy, fresh seed t25-negative-r1-seed with 0 collisions
      against 14 recorded prior seeds) -> typed terminal budget_exhausted
      with 2 settlement_committed events (label READ from the run_terminal,
      provenance-validated, DB pinned path+sha256); matched-pair.yaml
      written with the pinned positive (T24 replay DB) and negative blocks
      plus the recorded single-factor config diff.
- [x] run_tooth.py executed (tooth T8): structurally-equivalent synthetic
      route record through the SAME frozen novelty metric -> classification
      structural duplicate DETECTED and recorded; measured, no
      auto-rejection; 0 gauntlet executions, 0 provider calls;
      matched-pair.yaml byte-identical before/after.
- [x] verify_payment.py authored (trial run before the manifest freeze
      failed ONLY on the then-missing manifest, by design; gate not
      modified between trial and official run); evidence-manifest.yml
      frozen (8 files); FINAL GATE: PASS (137 checks), exit 0;
      payment-comparison.yaml + novelty.yaml assembled from recorded
      counters and byte-verified on re-runs.
- [x] b1-result.json written (status complete).

CLOSED — no further work in this task directory. Next move (orchestrator):
T26 independent adversarial confirmation of the scoped claims
(T23/T24/T25 evidence).

## Hard rules honored here

- Work ONLY under T25/ + durable probe data under
  `$HOME/.local/share/godspeed-route-discovery/T25/`. No other file modified
  (no preregs, no plan/status, no T21/T22/T23/T24 files - import/read only;
  T24 artifacts hash-verified through T24's own frozen manifest).
- No cargo/just/build. PYTHONDONTWRITEBYTECODE=1 everywhere.
- The negative is the REAL mechanism on a fresh copy with a fresh recorded
  seed; the negative label is READ from the run_terminal, never assigned.
  Payment/novelty numbers are counted from recorded evidence files only;
  unobservable quantities are marked n/a (no estimates).
- Scripts never overwritten after execution; corrections use round-2 names.
  Evidence manifest frozen before the FINAL gate run.
