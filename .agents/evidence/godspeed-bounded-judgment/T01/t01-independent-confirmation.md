# T01 Independent Adversarial Confirmation Record

- Date: 2026-09-17
- Verifier: fresh independent context (subagent), no builder narrative in packet
- **Verdict: CONFIRM** (required: CONFIRM)

## Packet given to the verifier

- Frozen preregistration + expected sha256 (`c44dcef8…`), exact corpus manifest
  (`6738d5a8…`), object-hash sidecar (`b4c863f6…`, 632 files), harness scripts
  and the correction trail (D-T01-01..07 only), raw round-1 evidence.
- Requirement ids settled (normative text read from the spec directly):
  REQ-VERIFY-004, REQ-SETTLE-003, REQ-OUT-006, REQ-OUT-007, REQ-JUDG-023,
  REQ-VERIFY-012, CLAIM-003, CLAIM-006, VAR-001, VAR-002, VAR-003.
- The exact confirmation question (attacks fail as declared; numbers recompute;
  conclusions surface-scoped).
- Exclusions enforced: the builder's outcome-classification decision entry
  (D-T01-08 and later), both status files, T00/round-2 — none read by the
  verifier.

## What the verifier independently established

- All artifact hashes match; the frozen round-1 harness hash equals the
  preserved `run_pilot.round1-crashed.py` byte-for-byte; correction-trail
  hashes match.
- Fresh reconstruction from the hash-verified databases is **byte-identical**
  to the persisted `pilot-dataset.json` (17 rows, 6 runs, 0 chain gaps, 17/17
  temporal ok, 17/17 unattributed).
- Every spot-checked metric recomputes to full precision: abstention rates
  16/17, 12/17, 11/17; capable-vs-capable disagreement 4/17 (0.235294…);
  capable-vs-scripted 6/17; effective diversity 289/49; schema counts.
- Attacks: label-shuffle reproduces `VACUOUS_NO_BASELINE_VARIANCE` and the
  verifier independently proved the vacuity premise (baseline `{PASS: 17}`,
  outside the frozen domain, 0 pairable rows — shuffle provably a no-op);
  label-leakage verified end-to-end (allowlist + frozen domain + no outcome
  value in any provider raw output); degenerate-domain rejection holds
  structurally; extended scope-widening search found **zero** scope
  violations; `verify_disclosure.py` PASS.
- `../gauntlet` unmodified (3 manifest db hashes re-verified; no frozen source
  among the repo's pre-existing dirty paths).

## Defect found by the verifier (repaired)

- Custody defect: `--teeth-suffix` was honored only by the shuffle handler, so
  the round-2 shuffle run overwrote `teeth/shuffle-labels.yml`; D-T01-07's
  preservation claim had become inaccurate. Repaired as D-T01-10: original
  failed-round bytes restored verbatim to
  `teeth/shuffle-labels.round1-false.yml` (sha256 `78f07787…`), round-7
  harness preserved, round-8 fix makes every tooth output suffix-honoring and
  never-overwriting (verified by a custody-fix run that created exactly one
  new file and touched nothing else).

## Residual weaknesses (disclosed, judged non-disqualifying by the verifier)

- `late-domain` / `degenerate-domain` teeth demonstrate rather than execute
  their rejections; the real enforcement is structural (frozen domain
  constant + prereg hash + input allowlist) and was independently verified.
- Four harness corrections (rounds 2–5) were schema-informed pre-provider,
  pre-authorized by the frozen adaptive-join analysis algorithm; no provider
  or outcome data was observed before the last correction.
- The verifier's own hazard note (rerunning teeth could overwrite evidence)
  is closed by the round-8 never-overwrite fix.
