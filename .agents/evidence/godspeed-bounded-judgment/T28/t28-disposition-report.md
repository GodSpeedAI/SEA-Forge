# T28 Disposition Report — Independent Confirmation of T27

Date: 2026-09-19 · Auditor: orchestrator session (with fresh independent verifier per plan) · Disposition: **CONFIRMED WITH QUALIFICATION** (fresh verifier's verdict per plan contract: **CONFIRM**)

## 1. Exact T27 evidence inspected

| artifact | sha256 (verified this audit) |
|---|---|
| `.agents/preregistrations/godspeed-bounded-judgment-T27.prereg.yaml` | `41af1921285bdbea33684d06b226fc8add83a59ed71378dbed464ab5b4b74343` |
| `.agents/preregistrations/godspeed-bounded-judgment-T27.prereg.addendum-1.yaml` | `a7fb31e295dfcca4c1fe6aba32f9345857d22187038b7c29f918b28681e07609` |
| `.agents/evidence/godspeed-bounded-judgment/T27/run_claim_obs.py` | `cc21e4652933a98fc8f4545aa58e9926141e73c1c2078145587fbe2a0ac81541` |
| `.agents/evidence/godspeed-bounded-judgment/T27/observations.jsonl` (140 records) | `979fd9e9833b9213f21dbc4e49f21041f1e9af78f51dc47b0b5d7d8c8d53fcb1` |
| `.agents/evidence/godspeed-bounded-judgment/T27/teeth-results.yml` | `e17f773fe054cecef5a66a98424fb7bdf695c511042132b0e08a2cc59d40cb7d` |
| `.agents/evidence/godspeed-bounded-judgment/T27/t27-paired-report.yml` | `cc533fed9aa6627c066d80d01b536068c665539cda79c82f03be68f24f4f650f` |
| `.agents/evidence/godspeed-bounded-judgment/T27/t27-settlement-report.md` | read as builder narrative for the orchestrator audit; withheld from the fresh verifier per the plan's `verifier_should_not_receive` |
| 14 corpus DBs under `~/.local/share/godspeed-t16-corpus/runs/` | all match the prereg's per-DB pins (14/14) |
| plan T28 definition; decisions D-2026-09-19-T27-01 … -05 | read |

## 2. Independent integrity checks (all pass; defects recorded)

- Freeze ordering: prereg mtime 11:38:04 < addendum 11:41:48 < harness last write 11:47:34 < **first persisted call ts 11:49:19** (local; UTC equivalently 15:38/15:41/15:47/15:49). Decision-log entries D-2026-09-19-T27-01/-02 (freezes) precede -03 (harness+teeth) precede acquisition.
- Addendum scope: changes exactly the discordant-pair definitions and their inputs to steps 2–4; corpus, conditions, renderers, provider, repetitions, readout, alpha, teeth, stop conditions untouched; base prereg bytes still hash to the recorded value. On this dataset the addendum was **anti-self-serving** (it counts accepted-A4's decided→abstain as a regression, blocking the cleaner C3 label).
- Corpus membership: the frozen selection rule replayed independently from the repeatability partition reproduces the selected 14 exactly (frontier = 1 stable-wrong + 9 unstable; controls = lexicographically-smallest stable-correct per family, filled to 4).
- Condition A identity: **70/70 stored A prompt hashes reproduced byte-exactly** from the prereg's frozen template + blob construction alone (both my audit and the verifier independently).
- Condition B integrity: all 5 reps per case share one prompt hash; the verifier reconstructed B from the prereg `renderer_spec` and matched 14/14 case hashes; atom-set equality with A holds both directions on all 14 cases; all 28 relation annotations are "supports" (uniform, zero discriminating signal); no correctness token in any role/relation label; no terminal/settlement token in any prompt body. B's lower accuracy and higher abstention is the opposite of what leakage would produce.
- Call independence/persistence: 140 unique keys, complete 14×2×5 cells, ts ordering exactly matches the frozen interleave, every record persisted with fsync before any scoring ran (scoring is a separate phase reading the store).
- Teeth: all four recorded PASS including simulated-attack rejections; the verifier re-verified teeth substance itself. Process defect recorded (§qualification c).
- Gate: `run_claim_obs.py --verify-only` exit 0, reproducing the reported result (run during this audit).

## 3. Independent recomputation method

Two recomputations, neither reusing T27's scoring code path for settlement-critical numbers:
1. **Orchestrator**: standalone `.agents/evidence/godspeed-bounded-judgment/T28/t28-independent-recompute.py` (imports nothing from the harness; truth re-derived from DB `run_terminal` events, cross-checked against the T16 manifest with 0 disagreements; own modal/binomial implementations). Output: `t28-independent-recompute.json`.
2. **Fresh verifier** (shielded from all builder narrative): own code over the raw observations and DBs, plus its own renderers reconstructed from the prereg text.

## 4. Independently derived paired table

Identical in both recomputations and to T27's report (see the verifier record §2 for the full table): 8 unchanged-correct, 5 unchanged-wrong, 0 corrections, 1 regression (accepted-A4, decided→abstain), target `budget-C3` unchanged-wrong; accA 9/14 (0.6429), accB 8/14 (0.5714).

## 5. Independently derived McNemar result

Binary-correctness pairs (addendum-1): b=0, c=1 → exact two-sided p = 1.0 (one-sided improvement 0.5, one-sided harm 0.5). Confirmed by both independent implementations.

## 6. Regression-guard verification

c_ctrl = 0 (accepted-A4 is a frontier case, not a control). Hard line (≥2 → C4) not triggered; soft line (=1 blocks C1/C2) not triggered. Guard passes; with zero corrections it supports no improvement claim.

## 7. C1–C5 rule traversal (frozen order, both auditors)

Validity gate PASS → C4 no (c_ctrl=0; c>b but p_harm=0.5) → C1 no (p=1.0; net=−1) → C2 no (T=0: target not corrected) → C3 no (**c_f=1 ≠ 0**) → **C5 (boundary)**. Mechanically required under the actual frozen rules (base + addendum-1).

## 8. Prereg/addendum timing audit

§2 above; both freezes precede the first observation at filesystem and store-timestamp level, and the store's key chain binds all 140 records to the prereg's file-bytes hash — the prereg that existed before the calls is byte-identical to the prereg on disk now.

## 9. Harness-hash audit

On-disk `run_claim_obs.py` hashes to `cc21e465…`, matching the corrected record (D-2026-09-19-T27-04). Stronger: the `--verify-only` gate re-derives every store key from the current file's renderers and passes 140/140 — proving the file that executed acquisition is byte-identical to the file on disk now. The stale-hash defect itself was already corrected append-only in the T27 log.

## 10. Abstention-shift verification — Claim A vs Claim B

- **Verified from raw records**: per-call abstentions A = 18/70, B = 32/70 (both recomputations). Mechanism visible in the persisted reasons: under B the provider repeatedly declines to decide because "no terminal settlement or budget-exhaustion record is present" — the typed structure made the pre-terminal status of the records salient.
- **Claim A (the representation changed provider behavior): SUPPORTED** — a real, measured behavioral shift on this provider/surface. It was not a preregistered endpoint; it emerges from the frozen per-call records and is reported as a confirmed observation, not a licensed generalization.
- **Claim B (the representation improved judgment quality): NOT SUPPORTED** — zero corrections, accuracy −1 case, p = 1.0.
- The two claims are kept separate. "Increased epistemic caution" is noted as an exploratory hypothesis, not a settled finding.

## 11. Stable-wrong case analysis (`budget-C3`)

Source evidence: the DB holds exactly one admitted builder invocation (iteration 1), one verification_record whose claim is locally verified PASS by the instrument, one correction selection, and a `budget_exhausted` terminal (terminal never in any prompt). Accepted-family cases show the complete 3-iteration invocation+correction pattern; C3's record set is truncated, with a correction chosen and no follow-up invocation.

- **Evidence-supported observations**: (i) C3 stayed wrong under both conditions (A 4/5 "settled"; B 4/5 "settled", 1 abstain); (ii) under B the provider *acknowledged* the typing — "All declared claims are supported…", and could see "no further invocations are recorded" — yet still answered "settled" off the positive session label; (iii) the provider is *capable* of sufficiency-style reasoning (A rep3: "Only a single… invocation… and no settlement record, so the run likely exhausts budget") but applies it unreliably; (iv) the same "no settlement record" observation is true of accepted-A* too (all inputs are pre-terminal by the leakage cut), so the discriminating skill cannot be "demand terminal records".
- **Inference (moderately supported, from the reasons across 10 calls + corpus structure)**: the persistent error concerns **evidence sufficiency and completeness for whole-case settlement** — whether the observed record pattern establishes closure — not provenance visibility. The provider pattern-matches local positive valence (session label, supported claims, a PASS-shaped observation) instead of judging whether the record set is complete relative to what settlement requires.
- **Hypothesis requiring a future experiment** (not tested by T27, not authorized now): explicitly representing required vs observed vs missing evidence coverage (settlement-sufficiency structure) would improve judgment on such cases. A secondary unresolved tension worth naming: when the provider applied strict "no terminal evidence → abstain" logic, it abstained broadly (the B abstention shift), so the needed competence is calibrating sufficiency *within* pre-terminal evidence — a coverage/completeness model, not stricter skepticism.

## 12. Confirmed claims

1. T27 was executed according to its frozen contract (integrity checks §2, timing §8, hash chain §9).
2. Its reported statistics are exactly reproducible from primary evidence by two independent implementations (§4–§6, §10).
3. C5 is the mechanically correct disposition under the frozen rules; the substantive pattern is additionally consistent with no useful treatment effect (under the superseded decided-only rule the label would be C3 — equally non-supporting).
4. The treatment changed provider behavior (abstentions 18→32 per-call) without improving case-level judgment (Claim A supported, Claim B not).
5. On this surface/provider/corpus/treatment, making claim-vs-observation provenance structurally explicit is not sufficient to improve this bounded-judgment failure.

## 13. Unsupported claims (explicitly not licensed)

That representation or provenance never matters; any transfer to unseen cases/surfaces/providers; that the stable-wrong behavior has a single cause; that DeepSeek owns truth or settlement; that JEVO/model judgment should replace deterministic case logic; that `.sea` grammar should change; that the UI should expose internal epistemic terminology. Any future representational change remains a projection/composition concern absent proof that a genuinely new semantic distinction requires canonical representation.

## 14. Developmental consequence

**Branch pruned**: input-representation provenance typing (claim/observation structuring of the *same facts*) is removed as a sufficient fix for this judgment failure. With T18's independent negative (single-call epistemic-role tagging) and T27's negative (R=5 typed provenance with the frontier targeted), the representation branch is closed on this surface by two differently-designed, both-valid experiments. **Smallest newly visible unresolved distinction (evidence-supported candidate): local positive evidence vs sufficiency of the whole-case record for settlement** — with the pre-terminal-inference calibration nuance recorded in §11.

## 15. Exact next affordable move

The plan authorizes nothing beyond T28: after this settlement every task is settled (T00–T05, T15–T18, T21–T27, T29) or blocked by evidence/physics (T06–T14 escalated; T19/T20 not authorized). Therefore **no further experiment is executed or authorized here**; the next move is an operator decision. Evidence-affordable candidates, in the order the evidence pays for them:
1. **Zero-provider-cost inspection**: study the deterministic settlement surface of `budget-C3` vs the accepted pattern (what a completeness/coverage criterion would have to check) before spending any calls.
2. If and only if the operator later authorizes a new developmental experiment: a settlement-sufficiency representation probe (required vs observed vs missing evidence coverage), preregistered against the same frontier — the one distinction this evidence actually localizes. Absent that authorization: *no further representational intervention is yet affordable; inspect the stable-wrong case and deterministic settlement surface more deeply first.*

## 16. Final T28 disposition

**CONFIRMED WITH QUALIFICATION.** T27 was validly executed, its numbers reproduce exactly, and C5 follows mechanically; the qualification set (none verdict-changing, all recorded append-only here):
- (a) the frozen prereg's constant-settled baseline fraction "4/14" is arithmetically wrong (actual 6/14 = 0.4286; the settlement report's decimal 0.4286 is correct, its "4/14" label is not); it gates nothing;
- (b) the frozen prereg file fails strict YAML parsing in `relation_rules` (block-scalar indentation); the contract is intact as frozen text/hash and was transcribed faithfully into the harness, whose behavior is independently verified;
- (c) `teeth-results.yml` on disk is a post-verification rewrite artifact (the verify path rewrites the log); the pre-acquisition teeth pass is evidenced by decision entry D-2026-09-19-T27-03, the harness's fail-closed ordering, and two independent post-hoc re-verifications;
- (d) C5 vs C3: C5 is mechanically required under the frozen rules; substantively the pattern is consistent with no useful treatment effect — the formal disposition remains inconclusive and is reported as such.

Per the plan, the required_verdict "CONFIRM" is returned by the fresh verifier; the orchestrator's disposition label adds the qualifications above rather than hiding them.
