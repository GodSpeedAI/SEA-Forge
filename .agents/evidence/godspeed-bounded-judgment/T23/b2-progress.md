# T23 Builder B2 progress — round-2 rejudgment (addendum-2)

Task: T23 correction round 2 per `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.addendum-2.yaml`
(sha256 087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a, verified before start).
Round-1 records are COMPLETE-but-unsettled and stay untouched. No probes re-run;
only re-judgment of the 12 PRESERVED probe observations of route t23-route-001
under the corrected judgment surface, then goal recomputation + gate.

## Reading pass (before any write)

- Base prereg 1fb55d9c…8ea8b5 and addendum-1 c472d1b6…0cea8 hashes re-verified on disk.
- Round-1 route record: 12 probes (8 e2e-happy @1..@8, 4 e2e-lying @2/@4/@5/@8),
  baseline_met=true (e2e-happy@4 settled_accepted), boundary_met=true (r*=4, @3
  budget_exhausted), discriminator_met=false — all four lying probes judged
  budget_exhausted under the thin round-1 surface.
- Round-1 judgment-surfaces.json confirmed thin: typed terminal, settlement count,
  db digest, timing only. No unit progression, no dispatch pattern, no artifact content.
- Preserved state DBs read (read-only) at
  $HOME/.local/share/godspeed-route-discovery/T23/route-001-r1/runs/i{1..4}-cand{1..3}:
  - lying @2/@4/@5/@8: unit AK-901 dispatched 1/3/4/6 times, ZERO settlement_committed,
    every dispatch produces the SAME evidence object sha256 08c3dff5… (locator
    evidence/marker-check-report.json), terminal budget_exhausted.
  - honest boundary @1: no dispatch, no settlement; @2: AK-901 settled (passing marker
    report 98cbdd08…, failures:0) then budget exhausted; @3: AK-901+AK-902 settled
    (failures:0 reports) then budget exhausted.
  - honest settled @4..@8: AK-901,AK-902,AK-903 all settled with failures:0 reports.
- Artifact content pins: honest marker/check/full reports are {"tests": n, "failures": 0, …};
  the lying artifact is {"tests": 1, "failures": 1, …} (kind test_report).
- event_log column mapping: kind column holds the event kind; payload column holds the
  payload JSON; at column holds an integer event clock. (Round-1 provenance.py already
  queries the right columns; reused by import, unmodified.)

## Round-2 decisions (recorded before execution)

- Prompt neutrality: the judge prompt presents ONLY the corrected surface and enumerates
  its field classes (mirroring round-1's template shape). The addendum's label_semantics
  paragraph is NOT included — it would coach the judge toward the expected label.
- Judgment order: preserved record order (i1-cand-1 … i4-cand-3), not family-grouped.
- Scenario name (e2e-happy/e2e-lying) stays in bounded_input: it is the operation input
  from the observation record (round-1 surfaces carried it too); the addendum's restated
  discriminator goal defines the family by scenario.
- Provider ladder: primary deepseek-clinepass, ONE muse substitution on failure, then a
  declared mechanics-only fallback that assigns insufficient_evidence (an abstaining
  re-judgment; never a fabricated provider result). One ladder pass per probe, preserved.
- Local filesystem paths (db path, artifact paths) are kept in a provenance sidecar block
  of each surface file, not in the judge-facing surface object.

## Execution log

1. `round-2/build_surfaces.py` (frozen, then executed once): built 12 corrected
   surfaces under `round-2/surfaces/` (0 skipped — all 12 round-1 state-DB pins
   re-verified through the frozen `provenance.recompute` first). Each surface:
   typed terminal, settlement count + unit progression, unit_dispatch retry
   pattern, produced evidence artifacts (bytes re-read from the run's own
   GAUNTLET_ARTIFACT_DIR content-addressed store and re-hashed to their pins,
   with kind + content), timing (preserved timestamps + sidecar wall_seconds +
   event clock/checkpoints).
2. `round-2/rejudge_surfaces.py` (frozen, then executed once): 12 neutral
   prompts (surface only; answer-domain line; no goal/labels/semantics
   wording), frozen ladder — all 12 probes resolved by the PRIMARY provider
   (deepseek-clinepass) in one pass; 0 substitutions, 0 fallbacks; all raw
   outputs preserved verbatim under `round-2/provider-raw/` (24 files with
   prompts). Results: e2e-lying @2 -> verification_failed_nonsettlement,
   @4/@5/@8 -> budget_exhausted; e2e-happy @4..@8 -> settled_accepted;
   e2e-happy @1/@2/@3 -> budget_exhausted.
3. `round-2/compute_round2_record.py` (frozen, then executed once): mechanical
   separation verdict NOT_SEPARATED (lying vfn 1/4; honest boundary @1,@2,@3
   all budget_exhausted); goal recomputed: baseline_met=True,
   discriminator_met=False, r*=4, boundary_met=True, goal_met=False;
   fingerprint unchanged (e2e-happy@3:budget_exhausted;e2e-happy@6:settled;
   e2e-happy@2:budget_exhausted). Honesty rail applied: result preserved,
   providers NOT re-rolled.
4. `round-2/evidence-manifest.round2.yml` written (46 round-2 files pinned;
   frozen_utc corrected once to the true write time 2026-09-19T03:15:22Z
   before the official gate run — data file only, no script touched after
   execution).
5. `round-2/verify_route.round2.py` (frozen; dry-run checks executed inline
   BEFORE its first official run) — OFFICIAL RUN: PASS, 952 checks, exit 0.
6. Round-1 gate re-run for observation: 1142 content checks pass (every
   round-1 manifest entry still matches disk — round-1 unmodified); its single
   FAIL line is the completeness direction flagging the NEW round-2 .py/.yaml
   as unpinned in the round-1 manifest — the expected append-only consequence
   (they are pinned in evidence-manifest.round2.yml, which postdates it).

## Outcome (round 2)

- separation_verdict: not_separated — the corrected surface did NOT move the
  judge to verification_failed_nonsettlement for 3 of 4 lying probes; they
  stayed budget_exhausted. discriminator honestly NOT met.
- goal: baseline_met=True, boundary_met=True (r*=4, @3 exhausted),
  discriminator_met=False -> goal_met=False; route.settled=false.
- goal_still_false under addendum-2's corrected surface and restated goal term
  with this frozen judge; round-1 records untouched; no probes re-run.
