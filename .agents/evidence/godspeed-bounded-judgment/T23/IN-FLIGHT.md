# T23 IN-FLIGHT — Multi-step route settlement under the frozen stop conditions

- Builder: B1
- Plan: `.agents/plans/godspeed-bounded-judgment-plan.yaml` v1.5.0, task T23
- Contract authority (frozen, read-only):
  - `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml`
    (sha256 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5)
  - `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.addendum-1.yaml`
    (sha256 c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8)
    — CORRECTED fixture mechanism + corrected baseline (@4); supersedes only
    fixture_and_case_family and the baseline wording.
- Dependencies: T21 SETTLED (frozen modules, sha256-verified), T22 SETTLED
  (harness pattern reused read-only; no T22 file modified).

## Scope of this task

Drive the frozen native route-discovery loop on the corrected fixture
mechanism (fresh `tests/fixtures/target` copy per probe, NO runner sed,
`max_rounds: 4` sed only when the probe value != 4, addendum env incl.
per-probe `GAUNTLET_ID_SEED`, deterministic clock + required controls) until
the route genuinely settles (baseline settled_accepted + discriminator
verification_failed_nonsettlement + boundary r*/r*-1 on this route's own
ledger) or an honest frozen stop condition fires. Execute teeth T3/T4/T6.
Prove everything with `verify_route.py`.

## State

- [x] Authoritative docs read in order; T21 module sha256s verified against
      T21/evidence-manifest.yml (all match, 2026-09-18).
- [x] IN-FLIGHT.md + b1-progress.md authored.
- [x] provenance.py + run_route.py authored, then executed (one route,
      route id t23-route-001, records append-only under T23/route/).
      OUTCOME: honest frozen stop `probe_budget_exhausted` after 4
      iterations / 12 probes; settled=false. Baseline MET (e2e-happy@4
      settled_accepted, 3 settlements); boundary MET on this route's own
      probes (r*=4, r*-1=3 budget_exhausted, happy grid fully swept 1..8);
      discriminator NOT met — every e2e-lying probe (@2,@4,@5,@8) ended
      budget_exhausted with 0 settlements and the frozen neutral judgment
      labeled all of them budget_exhausted (4/4). Design question surfaced
      for the orchestrator (see b1-progress.md / b1-result.json
      deviations): the corrected discriminator term is unreachable through
      the frozen judgment ladder on this fixture family.
- [x] run_teeth.py authored, then executed (T3/T4/T6 under T23/teeth/):
      ALL MATCH; route ledger byte-identical before/after. The first
      execution of the teeth script failed before any tooth ran (marker
      helper bug); preserved as run_teeth.round1-failed.py per the
      failure-preservation rule, smallest fix applied at the canonical path.
- [x] verify_route.py authored; evidence-manifest.yml frozen (87 files);
      FINAL GATE: PASS (1143 checks), exit 0.
- [x] b1-result.json written (status blocked-with-evidence on the route
      settlement claim; all harness deliverables complete and verified).

CLOSED — no further work in this task directory. Next move (orchestrator):
resolve the surfaced discriminator design question via an append-only
addendum or an explicit redesign decision before T24 (T24 requires a
settled T23 route).

## Hard rules honored here

- No edits outside T23/ + `$HOME/.local/share/godspeed-route-discovery/T23/`.
- No edits to: the plan, current_status files, prereg/addendum, any T21/T22
  file, any crate, any gauntlet source. T21 modules imported UNCHANGED and
  hash-verified fail-closed before anything runs.
- Probes strictly sequential, one gauntlet process at a time; `timeout 240`;
  PYTHONDONTWRITEBYTECODE=1; no cargo/just/build commands.
- A probe terminal is a probe consequence, never route settlement; the goal
  check is the route record's own, recomputed by verify_route.py.
- Scripts are never overwritten after execution; corrections use round-2+
  names; evidence-manifest.yml frozen before the FINAL gate run.

## SETTLED (orchestrator, 2026-09-19)

- Route t23-route-001 SETTLED under the frozen contracts (prereg + addenda 1-3): baseline leg (e2e-happy@4 settled, 3 settlement_committed), boundary leg (r*=4, r*-1=3, fully swept on this route's own 12-probe ledger), discriminator leg (addendum-3 mechanical rule separates all 4 lying probes from all 8 honest probes; verified against pinned DBs + sha256-pinned artifact objects).
- Correction history preserved: round-1 fixture-mechanism failure (addendum-1), round-2 judgment-mediated discriminator honestly NOT separated (addendum-2; 1/4, preserved as a first-class finding), round-3 mechanical rule (addendum-3, justified by the plan's authority-separation guardrail and the base prereg wording; the addendum history is explicitly routed to T26 for adversarial adjudication).
- Gates: verify_route.round3.py PASS 556 (orchestrator-instructed recompute-everything), round-2 PASS 952, round-1 1142/1143 (documented append-only completeness finding only).
- INDEPENDENT CONFIRMATION: fresh verifier, recompute-everything, no builder narratives: C1..C5 all confirmed, prereg chain + freeze ordering verified, zero material discrepancies. VERDICT: CONFIRM (t23-independent-confirmation.md).
- T23 SETTLED (P2, independent). Teeth T3/T4/T6 recorded MATCH.
