# T24 IN-FLIGHT — Settlement-first inverse case generation with independent replay

- Builder: B1
- Plan: `.agents/plans/godspeed-bounded-judgment-plan.yaml` v1.5.0, task T24
- Contract authority (frozen, read-only; imported/read, NEVER modified):
  - `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml`
    (sha256 1fb55d9c56d87d94adc4850bc5ca713db3e600297fbdc997accfab54c98ea8b5)
  - addendum-1 c472d1b63248ff5895433df16c545c4e145c8171e9d475a8f4b0732fa1c0cea8
  - addendum-2 087887c27cc50c6c989fcf7bb214e77aca62ea74d7e32a868ef0f7dd211e4c5a
  - addendum-3 1c13889ae0c2a9621b7f0b803b327ee6dd76a16983c9ed72b00ff309f59c6ddb
- Dependencies: T21 SETTLED (frozen modules), T22 SETTLED, T23 SETTLED
  (route t23-route-001 genuinely settled: baseline e2e-happy@4 settled_accepted;
  boundary r*=4 with r*-1=3 budget_exhausted; discriminator separated by the
  frozen addendum-3 mechanical rule). This task derives ONE candidate case
  FROM THE SETTLED ROUTE ONLY (T23/round-3/route-record.round3.yaml, sha256
  c6b9ee9cff7c8f2047dd29e2eef5576f9b113d72d5f8618c6849ec618bb27097).

## Scope of this task

Author generate_case.py (inverse generation from the settled route; frozen
provider ladder: deepseek-clinepass primary, ONE muse substitution, declared
deterministic fallback), freeze the candidate case record BEFORE replay,
replay it independently (fresh tests/fixtures/target copy, same frozen
addendum-1 env, fresh GAUNTLET_ID_SEED) through the real mechanism, accept
ONLY on genuine replay settlement (tooth T7: a companion case at the known
honest non-settlement budget max_rounds=2 presented as if solvable by the
route must be REJECTED by the same acceptance path). Gate: verify_case.py
(run from the sea-rs repo root; stdlib+PyYAML; imports frozen T21
route_contracts + T23 provenance, hash-verified fail-closed).

## State

- [x] Authoritative docs read in order (plan T24/T25, base prereg, addenda
      1-3, T23 round-3 settled route record, T21/T22/T23 manifests and
      payment counters, T15/T16 corpus records).
- [x] IN-FLIGHT.md + b1-progress.md authored.
- [x] generate_case.py authored, then executed. OUTCOME: mechanical
      settlement-first derivation -> case t24-case-001 = e2e-happy@4 (the
      route's own settled baseline case, 3 settlement_committed events on the
      route ledger); provider ladder: primary deepseek-clinepass responded
      with a derivation CONSISTENT with the mechanical case -> generator =
      deepseek-clinepass; prompt + raw output preserved verbatim under
      provider-raw/. Candidate FROZEN (candidate-freeze.json pins the
      candidate-case.yaml sha256 6bf2610c12a609cee98d48ad571baf2fc58d383232eb72bdb9275b245ea25977
      before any replay).
- [x] replay_case.py authored, then executed: independent replay through the
      real mechanism (fresh target copy, fresh seed t24-case-001-replay-r1,
      0 collisions against 12 recorded prior seeds) -> terminal settled with
      3 settlement_committed events, absorb-time provenance recompute clean
      -> accepted-case.yaml written (accepted=true, replay DB pinned
      path+sha256 under $HOME/.local/share/godspeed-route-discovery/T24/).
- [x] run_tooth.py executed (tooth T7): companion case e2e-happy@2 (the
      route's own known honest non-settlement budget, probe-i3-cand-1
      budget_exhausted) presented as if solvable by the settled route; REAL
      replay -> budget_exhausted, 1 settlement, acceptance path REJECTED it
      (rejected record preserved under teeth/ only, never added as valid
      developmental data); accepted-case.yaml + candidate-case.yaml
      byte-identical before/after.
- [x] verify_case.py authored (trial run before the manifest freeze failed
      ONLY on the then-missing manifest, by design; gate not modified
      between trial and official run); evidence-manifest.yml frozen (14
      files); FINAL GATE: PASS (154 checks), exit 0.
- [x] b1-result.json written (status complete; case accepted).

CLOSED — no further work in this task directory. Next: T25 (matched negative
max_rounds 4 -> 3 + payment/novelty evaluation) against the ACCEPTED case.

## Hard rules honored here

- Work ONLY under T24/ + durable probe data under
  `$HOME/.local/share/godspeed-route-discovery/T24/`. No other file modified
  (no preregs, no plan/status, no T21/T22/T23 files - import/read only).
- No cargo/just/build. PYTHONDONTWRITEBYTECODE=1 everywhere.
- Probe runs use exactly the frozen addendum-1 mechanism (fresh
  tests/fixtures/target copy; max_rounds sed only when the case value != 4;
  frozen env incl. unique recorded GAUNTLET_ID_SEED; GAUNTLET_CLOCK=
  deterministic; timeout 240; ground truth = run_terminal +
  settlement_committed from the state DB read-only, path+sha256 pinned).
- The candidate case record was frozen (sha256 pinned) BEFORE the replay; the
  replay re-verified the pin at start. Acceptance decided ONLY by the
  replay's real settlement; generator assertions are never acceptance
  evidence. Scripts never overwritten after execution; corrections use
  round-2 names. Evidence manifest frozen before the FINAL gate run.
