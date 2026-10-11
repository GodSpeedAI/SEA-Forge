# T26 Confirmation — native route discovery: fresh independent adversarial verification

- Verifier: fresh independent adversarial agent; no involvement in T21..T25.
- Date: 2026-09-19.
- Subject: T26 of `.agents/plans/godspeed-bounded-judgment-plan.yaml` (v1.5.0) — final
  confirmation of the native route-discovery developmental branch (T21..T25).
- Entry point: `.agents/evidence/godspeed-bounded-judgment/T23/round-3/confirmation-packet.yaml`.
- Frozen question: did the native implementation genuinely construct a governed route from
  real candidate probes and independently validate a generated case, or did any
  model-produced representation get mistaken for authority, execution, evidence, or
  settlement?
- Method: recompute everything, adversarially. My gate
  `T26/verify_discovery.py` (authored FIRST, sha256 frozen in `T26/evidence-manifest.yml`
  before each version's first execution, never modified after execution) shares no code
  with the builder's gates and re-derives every consequence from raw records: state DBs
  opened `mode=ro&immutable=1`, artifact objects re-hashed at their content-addressed
  paths, configs diffed tree-by-tree, counters re-counted from sources. Builder
  narratives (`b*-progress.md`, `b*-result.json`, `IN-FLIGHT.md`) were NOT opened.
  Writes were limited to the four T26 files; the forgery experiments ran on /tmp copies.
- Gate result: v1 (sha256 `acfb31c0…474e82`) FAILED on first execution with 17 errors —
  ALL defects of the verifier itself (a data-root prefix bug, a wrong manifest-size
  threshold, an attestation field read from the wrong file, a tuple-arity bug, and a
  grid-classifier bug that UNDER-counted the T15/T16 reference grid, i.e. erred in the
  anti-evidence direction). Each defect is classified in `T26/evidence-manifest.yml`.
  v2 (sha256 `b6378f01…b94d15`, frozen before its first execution) PASSES: 892 checks,
  exit 0. No evidence divergence was found at any point.

## 1. What was independently recomputed (gate sections)

1. **S1 Frozen rule authority** — base prereg + addenda 1-3 re-hashed against the
   packet pins AND pinned in the decision log (`D-2026-09-18-T21-01/-03`,
   `D-2026-09-19-T23-01/-02`).
2. **S2 Fail-closed manifests across T21..T25** — T21 (5 files), T22 (62), T23 round-1
   (87), T23 round-2 (46), T23 round-3 (4), T24 (14), T25 (8): every pinned entry
   re-hashed, zero mismatches; route-record identity pins (round-1 `9345c43e…`,
   round-2 `147f12e0…`, round-3 `c6b9ee9c…`) verified.
3. **S3 The 12 route probes from their pinned DBs** — for each: DB re-hashed at the
   durable data root, exactly one `run_terminal` event, terminal class mapped
   independently, `settlement_committed` counted, DB-side `budget.limits.rounds` joined
   against the recorded max_rounds, packet DB list joined bijectively, and every
   distinct `evidence_produced` ref re-hashed and parsed for `failures`. 12/12 exact.
4. **S4 Addendum-3 mechanical labels + symmetry** — c1/c2/c3 re-derived for all 12:
   the rule fires on exactly the 4 e2e-lying probes (failing artifact `08c3dff5…`,
   content `{"tests":1,"failures":1}`, re-hashed at all 4 paths) and on none of the 8
   e2e-happy probes. Round-1 judgment labels joined from the preserved records;
   round-2 corrected labels joined from the preserved round-2 record.
5. **S5 Goal + fingerprint** — from DB facts alone: baseline (happy@4 settled) true;
   settled set {4,5,6,7,8}, exhausted {1,2,3} so r\*=4 / r\*-1=3, boundary true;
   discriminator = separation true; goal_met true. Ledger fields byte-equal across
   rounds 1→3; fingerprint recomputes to
   `e2e-happy@3:budget_exhausted;e2e-happy@6:settled;e2e-happy@2:budget_exhausted`;
   every retained move is backed by a real probed ledger entry whose terminal
   reproduces from a pinned DB. The ORIGINAL base-prereg baseline leg
   (e2e-happy@8 settles) also holds on the final ledger (probe-i3-cand-2) — checked
   explicitly for the correction-history adjudication below.
6. **S6 T24** — candidate freeze pin == live candidate hash == replay-record's
   `candidate_seen_sha256`; freeze attests no replay record existed; live mtimes match
   the recorded run-time mtime floats exactly; replay DB settled with 3 settlements and
   `budget.limits.rounds=4`; `accepted-case.yaml` joins the same DB hash; the replay
   target is byte-identical to `tests/fixtures/target` (recursive diff: zero
   differences); replay seed absent from all T22/T23 records; the FROZEN candidate
   records `accepted: false` / `replay_status: pending` / `replayed: false` —
   acceptance could only come from the replay.
7. **S7 T25** — negative DB budget_exhausted with 2 settlements and rounds=3; recorded
   label equals the raw terminal consequence; recursive target diff vs the fixture
   shows EXACTLY one differing file (GAUNTLET.md) with exactly one changed line
   (`max_rounds: 4` → `3`); negative seed fresh; the pair's positive leg joins T24's
   replay DB.
8. **S8 Novelty** — grid re-derived from the T15/T16 frozen records (T15 prereg case
   table + corpus manifest 9 runs + T16 expanded manifest 25 runs + expansion addendum
   mechanism mix): exactly the 5 recorded grid fingerprints
   (`e2e-forged-report@4:budget_exhausted`, `e2e-happy@1:budget_exhausted`,
   `e2e-happy@2:budget_exhausted`, `e2e-happy@4:settled`,
   `e2e-lying@4:budget_exhausted`); grid budget levels {1,2,4} — 5/6/7 never probed;
   route levels {2,3,6} with 3 and 6 outside the grid, matching the recorded
   `route_levels_outside_the_grid: [3,6]`; classification `novel` is correct under the
   frozen metric. T8's synthetic duplicate recomputes to the identical fingerprint and
   is recorded `structural-duplicate`, measured not auto-rejected, per the frozen
   policy.
9. **S9 Payment counters** — decision log re-counted: exactly 7 branch entries
   (T21-01/02/03, T22-01, T23-01/02/03) and exactly 21 baseline entries (T15-01..03 +
   T16-01..18). Provider calls re-counted from each stage file's by_identity lists:
   4 + 4 + 16 (4 generation + 12 judgment) + 12 (re-judgment; independently re-counted
   as 12 attempts in rejudgment-results.json) + 0 + 1 + 0 = 37; generation calls 7;
   probe-judgment calls 18; tool executions 3+3+12+1+2+1 = 22; failed executions 3
   (the preserved T22 round-1 run_error trio); recorded wall 48.852+56.456+275.997+
   23.737+0.505+0.204+0.303 = 406.054 s; tokens/cost honestly n/a on both sides;
   baseline quotes match the T15/T16 manifests (9 then 25 runs; class counts as
   recorded).
10. **S10 Model-free settlement** — round-3 has zero provider calls and no
    provider-raw directory; the goal recomputation (S5) uses only DB facts and yet
    reproduces `goal_met=true` while 3 of 4 recorded round-2 provider labels disagree
    with the mechanical rule — proof the settlement does not rest on model output;
    selection records carry `selection-policy-v1` only, no provider/judge fields;
    `selection_policy.py` has no provider or network interface; T21 `admission.py`
    is deny-first deterministic (authority violations → denied, no execution path).
11. **S11 Claim boundaries** — regex scan over the 16 developmental records: the only
    hit is the evidence-kind token `work_product_promoted` in the packet's explanatory
    note (not a claim, not a rule input). `current_status.yml` lists T06 as
    SETTLED-BLOCKED and T19 NOT AUTHORIZED — the branch unblocked nothing.

## 2. Adversarial attacks (full records in `T26/attacks.yaml`)

Nine attacks executed; zero falsified a claim. Highlights:

- **A1 (mandated a) model output as authority/execution/settlement** — no consumption
  path found. The sharpest fact: the round-2 provider labels disagree with the
  mechanical rule on 3 of 4 lying probes, and the settlement stands anyway on raw DB
  facts. Also: all 12 round-1 provider judgment labels agree with the DB facts
  (settled_accepted↔settled, budget_exhausted↔budget_exhausted), so no model label
  ever made the route look better than its raw consequences. T24 acceptance flipped
  only after the replay DB showed settled; the frozen candidate still recorded
  `accepted: false` / `replay_status: pending`.
- **A2 (mandated b) irreproducible terminal** — none; 16/16 run DBs reproduce their
  recorded terminals, settlement counts, and budget levels. Scenario family self-ID
  (`records.role_invocation.session_ref`, e.g. `mock-e2e-lying`) matches 11/12; the
  12th (happy@1, zero dispatches) cannot self-identify and rests on its record pin +
  DB rounds — disclosed, non-material.
- **A3 (mandated c, novel) cross-run artifact laundering** — I forged, on /tmp copies,
  an `evidence_produced` event referencing the lying family's failing artifact inside
  an honest happy@1 DB copy and copied the object into its content-addressed path. The
  mechanical rule alone DOES fire on the forgery (c1+c2+c3 all true) — the honest
  finding is that the rule is record-level forgeable. It is caught twice: the DB
  sha256 pin breaks (multi-era pins: round-1 observations, packet, mechanical-labels,
  fail-closed manifests), and the forged run cannot claim the e2e-lying family
  (no `session_ref`; substitution into the happy family breaks the frozen symmetry
  check 0/8). Escalating to a full re-pin means rewriting files frozen by three
  different builder eras that cross-pin each other (round-3 record pins round-1/2
  records; candidate pins round-3; matched-pair pins T24's DB). Honest limit: the
  evidence tree is untracked worktree state by recorded convention, so an attacker who
  can rewrite EVERYTHING including the raw DBs could forge the entire web; the anchors
  are the raw DB set, the multi-era cross-pins, and the decision log. Recorded as fog,
  not as evidence of forgery.
- **A4 (novel) freeze-mtime forgery** — touch can forge mtimes, but the pinned
  replay-record stored the exact run-time mtime floats and the live files still match
  them exactly; a post-hoc touch would desynchronize live-vs-recorded, a record rewrite
  is a manifest-hash break, and the replay DB content hash independently pins the
  post-freeze run.
- **A5 (mandated d) claim boundaries** — no record claims promotion, production
  readiness, capability-class generalization, or T06/T18 unblocking; the status file
  agrees.
- **A6/A7 (plan T26 teeth)** — tooth 1 (confirm without recomputing a DB terminal):
  REJECTED as non-independent — the confirmation recomputes all 15 run DBs from raw
  event logs with its own frozen code. Tooth 2 (treat judgment/generator output as
  authority/execution/evidence/settlement): no such treatment exists in the final
  evidence; the historical near-miss (addendum-2's judgment-mediated discriminator) was
  superseded precisely because it would have violated this tooth, and its re-judgment
  failure (1/4) is preserved first-class.
- **A8/A9 (self-directed)** — the prior confirmation's "scenario strings in DB
  payloads" claim is correct but lives in the `records` table, not `event_log` (my
  first scan missed it; deeper inspection confirms the provenance). CLAIM C's two
  presentational scope choices (judgment_calls 18-vs-30 decomposition; operator-entry
  range ending at T23-03 vs 9 with the later settlement entries) are source-true,
  disclosed, and non-material.

## 3. Per-claim verdicts

### CLAIM A (route search works) — CONFIRM

The loop constructed a real governed multi-step route (t23-route-001: baseline
e2e-happy@4 settled with 3 settlements; boundary r\*=4 / r\*-1=3 established by this
route's own 12 sequential probes; discriminator separated 4/4 lying vs 0/8 honest under
the frozen addendum-3 mechanical rule over hash-verified artifacts) that genuinely
settles — and the settlement is reproducible from raw DB facts alone, with zero round-3
provider calls, a deterministic deny-first admission layer, and a selection policy with
no provider interface. Candidates were real provider-generated K=3 batches (raw
prompt+output preserved, 16 round-1 calls re-counted), every admitted candidate was
probed, and the 12-probe budget was spent honestly (`probe_budget_exhausted` at depth
12, goal recomputed after addendum-3). Scope limits stated and respected: single
fixture surface (`tests/fixtures/target`), mock replay runner only, N=1 route, 24-op
catalog, bounded budgets. No LLM controlled authority or settlement at any point in the
final evidence.

### CLAIM B (case generation works) — CONFIRM

The settled route was inverted into a fresh case (t24-case-001, e2e-happy@4, derived by
the primary provider from the route's own ledger; prompt+raw preserved) whose FROZEN
record still said `accepted: false` / `replay_status: pending`, and which was accepted
only after an independent replay — fresh disposable target byte-identical to the
fixture, fresh seed absent from all prior records, deterministic-clock env — genuinely
settled through gauntlet's own settlement path (1 `run_terminal` settled, 3
`settlement_committed`, DB pinned and re-hashed). Freeze strictly preceded replay
(three corroborations: recorded run-time mtimes == live mtimes, freeze attestation,
replay start timestamp). The impostor was rejected by the same acceptance path: the T7
lookalike (e2e-happy@2 with the same legitimacy claim) replayed honestly to
budget_exhausted (1 settlement, rounds=2 read from its own DB) and was recorded invalid,
preserved only under `teeth/`, with the developmental record byte-unchanged.

### CLAIM C (data usefulness MEASURED) — CONFIRM (measurement honest and within its recorded scope)

The payment comparison re-counts cleanly from real counters on both sides: branch side
7 decision entries, 37 provider calls, 22 gauntlet executions, 3 failed, 406.054 s
recorded wall, tokens/cost honestly n/a; manual baseline quoted only from the recorded
T15/T16 evidence (9→25 runs, 21 decision entries, class counts, provider answer
reports, wall n/a). The novelty classification `novel` is correct under the frozen
metric against the correctly re-derived grid, with duplication measured-and-recorded
(T8) and no auto-rejection, per the frozen policy. CONFIRM here means exactly what the
plan scopes: the MEASUREMENT is honest, source-true, and within its recorded surface —
the two sides measure DIFFERENT things (loop-mechanism payment vs manual corpus
construction) and the record itself says "no normalized cost claim". Usefulness-at-scale
is explicitly future work and is NOT proven by this branch.

## 4. Correction-history adjudication: principled append-only correction, NOT answer-shopping

The branch amended its frozen prereg three times. My ruling, from the records:

**Freeze ordering (verified by mtimes + recorded timestamps + in-record hash pins).**
Base prereg frozen 2026-09-18 20:38 EDT before any execution. Addendum-1: 21:56:23 EDT
— AFTER the preserved T22 failed rounds (21:36/21:46 EDT) but BEFORE the first T23
probe (22:24:36 EDT). Addendum-2: 22:49:52 EDT — AFTER T23 round-1 but BEFORE the
first round-2 re-judgment call (23:03:28 EDT). Addendum-3: 23:23:26 EDT — AFTER the
round-2 results but BEFORE the round-3 evaluation (23:29:05). Every consuming record
pins the addendum hash it used; the decision log records each freeze; nothing was
edited in place (all manifest chains hash-verify).

**(1) Was any addendum adopted after seeing the data it governs?**
- Addendum-1 governs the T23 fixture mechanism; the data that motivated it (T22's
  run_errors, then all-budget_exhausted on demo-calculator copies with the instrument
  recording failures=1 — baseline unreachable by construction) was preserved failed-
  round evidence, and the measurement the addendum governs (T23) had not run. This is
  a pilot-failure protocol correction, frozen before the real measurement: principled.
- Addendum-2 governs the re-judgment of ALREADY-COLLECTED round-1 observations — the
  data existed before the rule. But it was frozen before the re-judgment executed, it
  specified its own honest-failure branch in advance ("If the labels do not separate,
  the discriminator is honestly NOT met and the route stops with that result
  preserved"), and that branch FIRED (only 1/4 lying probes labeled
  verification_failed_nonsettlement) and was preserved. A rule that honestly predicts
  its own failure mode and then records the failure is not shopping.
- Addendum-3 governs the round-3 goal evaluation of the SAME preserved data, adopted
  after addendum-2's rule FAILED on that data. This is the sharp case — the classic
  answer-shopping shape — and it deserves the strictest scrutiny, which I applied
  (below). Honest disclosure: addendum-3 was written with full knowledge of the data
  it classifies (it says so: "Verified against all 12 preserved probes before
  freezing"). In a strict pre-registration sense that is post-hoc. The question is
  whether it was principled.

**(2) The addendum-3 test.** Four record-based facts decide it:
1. **The plan's own guardrails required exactly this correction.** The plan freezes
   "provider/model output NEVER widens authority… never defaults to permissive" and
   the T26 tooth declares "treat a judgment or generator output as … settlement" a
   confirmation failure. Addendum-2's separation_requirement made ROUTE SETTLEMENT
   depend on provider labels — a rule that could not have survived this very
   confirmation even if it had PASSED (a 4/4 model separation would have been a worse
   violation, not a better outcome). Addendum-3 was therefore required for plan
   compliance, not to convert a failure into a pass.
2. **It returns to the base prereg's own semantics instead of inventing a new
   standard.** The base wording — "an e2e-lying probe on this route honestly fails
   verification (verification_failed_nonsettlement) — the instrument discriminates on
   this surface" — is a consequence fact about the surface, with the answer-domain
   label in parentheses. Addendum-3 codifies exactly that consequence fact as a
   deterministic, symmetric rule (bounded non-settlement AND 0 settlements AND a
   hash-verified failing artifact). No new goal was invented that the data happens to
   pass; the original goal was made evaluable the way the plan's architecture requires
   (deterministic goal evaluation; learned judgment stays evidence for
   selection/claims/payment, never a settlement gate).
3. **The adverse intermediate result is preserved, not erased.** The 1/4
   non-separation remains manifest-pinned (`separation-verdict.json`,
   `rejudgment-results.json`, 24 raw provider files), is carried as a first-class
   finding into T25's payment/usefulness evaluation and this adjudication, and is
   reported beside the mechanical result with an explicit honesty note. Shopping that
   leaves its losing roll on the board is not shopping.
4. **The rule is symmetric and falsifiable, and the symmetry could have failed.** The
   honest family check (0/8 must not trigger) was verified against all 12 probes
   before freezing and re-verified independently by me: happy@1's zero-artifact run,
   happy@2/@3's partial settlements, and happy@4..@8's settled terminals each break a
   different clause. Had any honest run produced a failing artifact with zero
   settlements, the rule would have misfired and the symmetry property would have
   been falsified on adoption.

**(3) Would the FINAL route settlement have been reachable under the ORIGINAL
base-prereg wording alone?** Largely yes — with one necessary exception:
- Under the base fixture mechanism: NO — the frozen recipe was unfollowable (T22
  round-1 run_errors on missing controls; round-2 demo-calculator copies cannot settle
  at any budget). Addendum-1 was strictly necessary for the measurement to exist at
  all; its defect was proven empirically by preserved rounds, not assumed.
- On the corrected fixture, the base wording's own legs are satisfied by the final
  ledger: the base baseline (e2e-happy@8 settles) holds — probe-i3-cand-2 settled at 8
  (my gate checks this explicitly); the boundary (r\*/r\*-1 from this route's own
  probes) holds (4/3); and the discriminator under the base wording's consequence
  reading ("the instrument discriminates") holds mechanically — 4/4 lying runs produce
  hash-verified failing verification artifacts with zero settlements while 0/8 honest
  runs do. The only thing the addenda 2/3 actually changed was WHICH LAYER evaluates
  the discriminator term; addendum-3's answer (deterministic consequence evaluation)
  is the one the plan's frozen guardrails demand. So the branch did not need a
  favorable reinterpretation of its data to settle — it needed a fixture repair
  (proven broken, preserved) and a rule-location repair (required by the plan,
  returning to base semantics).
- Residual fog, recorded honestly: the base wording's parenthetical label was
  genuinely ambiguous (terminal-enum vs judgment label), and addendum-2's
  judgment-mediated reading was a defensible misreading that cost one honest failed
  re-judgment before being corrected. The chain is append-only, hash-pinned, freeze-
  ordered, and adverse-result-preserving at every link. VERDICT ON THE CORRECTION
  HISTORY: principled append-only correction, NOT answer-shopping — with the
  post-hoc construction of addendum-3 (rule written knowing the data, justified by
  guardrail + base wording, symmetry pre-checked) explicitly recorded as the weakest
  epistemic link of the branch, mitigated by the preservation of the failed 1/4
  finding and by the fact that the settlement also passes under the base wording it
  restored.

## 5. Unproven list (explicit, per the plan's final_acceptance)

- **Usefulness-at-scale / usefulness-at-transfer of the generated data** — future
  work; CLAIM C proves the measurement, not the usefulness.
- **Model-quality comparability and judge calibration** — actively weakened by this
  branch's own preserved finding: on the corrected enriched surface the primary
  provider separated only 1/4 lying probes at K=1 (bounded-judgment label
  reliability).
- **Fleet behavior and long-horizon reliability** — not exercised by an N=1 route,
  single fixture surface, mock replay runner, 6 iterations.
- **Durability/promotion (claim D)** — out of scope by freezing; no promotion claim
  exists anywhere in the branch.
- **T06 remains SETTLED-BLOCKED; T19 remains NOT AUTHORIZED** — this branch unblocked
  neither.
- **Normalized economics** — the two sides of the payment comparison measure different
  surfaces; no cost-per-case claim is licensed.
- **Novelty** is measured only against the recorded T15/T16 grids; other references
  were not surveyed.
- **Evidence-tree immutability** rests on the untracked-worktree convention (multi-era
  cross-pins + raw data anchors + decision log), not on an external cryptographic
  anchor (attack A3's honest limit).

## 6. Next affordable move (recorded without promotion claim)

T26 closes the developmental branch. The next affordable move is the operator's
decision on the branch's recorded open question, informed by exactly what was earned:
a working governed route constructor and settlement-first case generator on one
surface at recorded payment, with judge-label reliability (1/4 separation) as the
named defect and usefulness-at-transfer as the named future work. No promotion, no
production claim, no capability-class claim is licensed by this confirmation.

## 7. Verdict

- CLAIM A (route search works, scoped): **CONFIRM**
- CLAIM B (case generation works): **CONFIRM**
- CLAIM C (usefulness MEASURED, measurement-scope): **CONFIRM**
- Correction history: principled append-only correction, **NOT answer-shopping**
- Claim D (durability/promotion): not adjudicated here, per the plan
- Gate: `verify_discovery.py` v2 (sha256 `b6378f01…b94d15`, frozen in
  `evidence-manifest.yml`) exit 0, 892 checks; 9 attacks recorded in `attacks.yaml`,
  0 falsified

VERDICT: CONFIRM
