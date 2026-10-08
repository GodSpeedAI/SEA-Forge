# T28 — Fresh Independent Verifier Confirmation Record

Task: T28 (independent adversarial confirmation of T27) · Verifier: fresh general-purpose agent, no prior involvement in T27 · Date: 2026-09-19

Verifier packet (per the plan's `verifier_receives`, and nothing else): the frozen T27 preregistration and addendum-1, the harness containing both renderers and scoring, the 140 raw persisted observations, the teeth log, the frozen T16 expanded-corpus manifest, the repeatability per-case partition, and read-only corpus DBs. The verifier was shielded from the builder narrative (`t27-settlement-report.md`, `t27-paired-report.yml`, `decisions.yml`, status files) and from the preferred C1–C5 outcome. Its final report is transcribed below verbatim; settlement-critical numbers were recomputed by its own code, not by the T27 harness.

---

## 1. VERDICT

**VERDICT: CONFIRM**

I independently recomputed every settlement-critical number from the raw observations and the corpus DBs, and they reproduce exactly: accA=9/14, accB=8/14, b=0, c=1, c_f=1, T=0, p_two=1.0, c_ctrl=0, which under the frozen ordered rules (base prereg + addendum-1, both proven to precede the first provider call) mechanically lands on **C5** — the claimed disposition. The store is fully bound to the frozen contract (140/140 key-chain, 70/70 A-prompt and 14/14 B-case prompt reproductions from my own code), and I could not construct any attack that produces a positive disposition (C1/C2) or reveals leakage. The only defects found are non-load-bearing (frozen baseline arithmetic 4/14 vs actual 6/14; on-disk teeth log is a post-acquisition re-run artifact).

## 2. RECOMPUTED NUMBERS

Per-case readouts (verifier derivation, truth from DB `run_terminal`):

| case | split | truth | A readout | B readout | A ok | B ok |
|---|---|---|---|---|---|---|
| accepted-A1 | ctrl | settled | settled | settled | yes | yes |
| accepted-A2 | ctrl | settled | settled | settled | yes | yes |
| forge-D1 | ctrl | not_settled | not_settled | not_settled | yes | yes |
| lying-B1 | ctrl | not_settled | not_settled | not_settled | yes | yes |
| accepted-A4 | frontier | settled | settled | **insufficient_evidence** | yes | **no** |
| accepted-A10/A11/A12 | frontier | settled | settled | settled | yes | yes |
| forge-D2 | frontier | not_settled | not_settled | not_settled | yes | yes |
| budget-C1/C2/C4/C5 | frontier | not_settled | insufficient_evidence | insufficient_evidence | no | no |
| budget-C3 (target) | frontier | not_settled | settled (wrong) | settled (wrong) | no | no |

- **accA = 9/14, accB = 8/14**; baseline constant-settled = 6/14 (prereg froze "4/14" — see defects)
- **b = 0, c = 1** (binary-correctness per addendum-1: accepted-A4 A-correct → B-abstain); **b_f = 0, c_f = 1; c_ctrl = 0**
- **unchanged-correct = 8, unchanged-wrong = 5** (C1/C2/C4/C5 abstain→abstain; C3 decided-wrong→decided-wrong)
- Decided-only: b=0, c=0; abstention transitions: A-abstain→B-decided = 0, A-decided→B-abstain = 1, abstain→abstain = 4
- **Target budget-C3: does NOT correct** (A=settled wrong, B=settled wrong; T=0)
- **Per-call abstentions: A = 18/70, B = 32/70**
- **McNemar exact: two-sided p = 1.000000; one-sided improvement p = 0.500000; one-sided harm p = 0.500000**
- practical criterion: net = −1, frontier_net = −1 → **not practical**; guard: c_ctrl=0 → **guard passes**
- **C-rule traversal**: validity gate PASS → C4 not fired (c_ctrl=0; c>b but p_harm=0.5) → C1 not fired (p=1.0) → C2 not fired (T=0) → C3 not fired (c_f=1) → **C5**, boundary criteria: c_f=1 (accepted-A4), T=0, b=0.

## 3. INTEGRITY FINDINGS

- **Corpus**: all 14 pinned DB sha256s match the prereg freeze exactly (accepted-A1 `8d419e02…` … lying-B1 `ab6cc686…`). Each DB has exactly one `run_terminal` event; 6 settled / 8 budget_exhausted among the selected. DB-derived truth agrees with per-case.json for all 25 cases; `all_agree`/`aggregate_correct` internally consistent with the recorded per-call answers.
- **Selection replay**: exact match. stable_correct=15, stable_wrong=1 (budget-C3), unstable=9; frontier = those 10; controls by the frozen rule = {accepted-A1, forge-D1, lying-B1, accepted-A2}; selected 14 == prereg's set == case_ids in observations.jsonl (14 cases × 2 conditions × 5 reps, no missing/duplicate cells).
- **Key chain**: 140/140 records satisfy `key = sha256("<prereg_sha256>|<case>|<cond>|<rep>|<prompt_hash>")[:16]` with prereg_sha256 = SHA256 of the actual prereg file bytes computed by the verifier = `41af1921285bdbea33684d06b226fc8add83a59ed71378dbed464ab5b4b74343` (also matches the addendum's recorded base hash and the harness's hardcoded constant). The store is bound to the frozen contract.
- **A-prompt reproduction**: 70/70 stored A `prompt_hash` values match the verifier's reconstruction (prereg header template + `json.dumps({"invocations":[{"result_document":…}], "corrections":k}, default=str)`, truncation at 12000 a no-op for all 14 cases). **B reproduction**: the verifier's independent renderer written from the prereg `renderer_spec` matches all 14 stored B case hashes (14/14), so all 70 B records are bound to frozen-spec prompts built from DB content.
- **B-renderer judgment** (`run_claim_obs.py`): **(i)** no new scalar fact — `_claim_blocks` (lines 134–172) and `render_b` (200–202) draw every atom from the same invocations/corrections that `a_structure` (122–124) feeds A; the verifier's own atom-set equality both directions passes for all 14 cases; the only new propositional text is the trace-completeness clause (line 160), implied by the array extent, containing no truth token. **(ii)** no correctness encoding — vocabularies at lines 82–86 are neutral; `_relation` (175–197) consults only payload fields, never truth; recomputation shows **all 28 relation annotations are "supports"**, uniform across cases and carrying zero case-discriminating signal. **(iii)** no terminal/settlement leak — the terminal event is read only into `truth` (109–111) which never enters a renderer; `phase_acquire` hashes and sends the same prompt object (399–411); `horizon_check` (269–275) + assembler guard (594–598); the verifier's own scan of all reconstructed A and B bodies is clean; no truth token exists in any payload. B's *lower* accuracy and *higher* abstention is the opposite of what leakage would produce.
- **Timing (UTC)**: prereg freeze 15:38:04Z < addendum-1 15:41:48Z < harness last write 15:47:34Z < **first observation ts 15:49:19Z** < last ts 17:23:15Z (= observations.jsonl mtime). ts order across all 140 records exactly equals the frozen acquisition interleaving (case sorted, rep 0..4, A then B). **Harness sha256 = cc21e4652933a98fc8f4545aa58e9926141e73c1c2078145587fbe2a0ac81541 — matches the audit claim.**
- **Addendum audit**: superseded terms are exactly `paired_analysis.discordant_counts` and step_3_C2's b_f input; the preserved list restates the practical formula, guard form, step order, and transitions tabulation unchanged; corpus, conditions, renderers, provider, repetitions, normalization, readout, alpha=0.05, teeth, stop conditions, evidence locations untouched (verified by full-text comparison). `append_only` holds: base file bytes today still hash to the addendum's recorded value, and base mtime < addendum mtime < first call. Notably, on this dataset the addendum is **anti-self-serving**: it counts accepted-A4's decided→abstain as a regression (blocking C3), moving C3→C5.
- **Minor defects (non-load-bearing)**: (1) the prereg's frozen baseline "constant settled predictor = 4/14" is arithmetically wrong — the selected 14 contain 6 settled-truth cases (6/14); nothing consumes it. (2) `teeth-results.yml` mtime is 17:50:54Z, i.e. a post-acquisition `--verify-only` re-run artifact (the log is rewritten in verify mode, lines 600–605); the pre-acquisition teeth pass is inferred from the harness's fail-closed ordering (teeth gate acquisition before `phase_acquire`) — and the verifier verified the teeth substance itself, so no gate was bypassed. (3) The prereg's trace-join pseudo-code (", ".join) differs cosmetically from the implementation ("; " join); same atoms, store bound to the implementation.

## 4. ATTACKS

- **Decided-only (pre-addendum) recomputation**: b=0, c=0, b_f=0, c_f=0, c_ctrl=0, p_two=1.0 → traversal fires **C3** instead of C5. The *label* changes, but no rule-set produces anything positive: both C5 and C3 are non-supporting dispositions, and the addendum's binary rule is the stricter one here. No rescue of the hypothesis.
- **Leakage/anomaly scan**: 140/140 status ok, 0 out-of-domain, 0 schema failures, 0 empty raws, 140 unique keys, all cells complete, no rep-conditioned prompt variation, 138 distinct raw prefixes (one benign duplicate abstention phrasing), plausible confidence ranges (A mean 0.637, B mean 0.658). Nothing implausibly perfect; answers vary across cases and conditions. No evidence of prompt-truth leakage.
- **Key-chain forgery**: would require knowing the prereg file-bytes hash in advance; it matches and binds all 140 records. No failure.
- **Prompt-hash mismatch (condition swap / cross-case contamination)**: would surface as reproduction failure of reconstructed A/B hashes; 70/70 A and 14/14 B case hashes reproduced from DB content per the frozen specs. No failure.

## 5. VERDICT-CHANGING DEFECTS

None. The two reportable defects (frozen baseline 4/14 vs actual 6/14; teeth log being a post-verification re-write) touch no gate, no b/c count, no p-value, and no traversal step — the disposition remains C5 under every frozen reading, and the scientific conclusion (structural claim/observation typing did **not** improve case-level bounded judgment on this frontier; it added one abstention regression and raised abstention 18→32 per-call) is independently confirmed.
