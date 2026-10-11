# Explanation: Native Route Discovery

> **How a route can be discovered by consequence instead of authored by a plan: candidate generation → governed probe → consequence → discrimination → retained route, with the retained route invertible into a fresh, independently re-settleable case.**
>
> Earned by the settled developmental branch `godspeed-bounded-judgment` tasks T21–T26 (plan v1.5.0); fresh independent adversarial confirmation returned `CONFIRM`. Scope: one fixture surface, deterministic mock replay runner, one route — not a capability-class claim.

---

## 1. The Durable Structure

Native route discovery realizes the loop slot that [`goodspeed-loop.md`](../explanations-and-references/goodspeed-loop.md) calls *changed navigation next cycle* without a hand-authored plan:

```text
current bounded case/route state
  → represented candidate neighborhood (bounded, never the universe)
  → K typed candidate moves
  → authority/admission classification (per existing rules)
  → bounded real probes through the execution substrate
  → typed consequence observations (evidence-backed)
  → discrimination over observations
  → deterministic selection policy (versioned, frozen before evaluation)
  → retained route move (provenance-joined)
  → repeat to a frozen settlement criterion or an honest stop
```

The load-bearing separation, each piece owned where it already lives:

| Responsibility | Owner |
|---|---|
| What candidate moves exist and what they mean | Frozen, hashed operations catalog (representation data), derived from real fixture mechanisms |
| Whether a candidate may execute | The existing admission/authority rules — model output can widen nothing |
| Producing the consequence | **Gauntlet** (the execution/probe substrate): headless `gauntlet run` over disposable target copies; its own verify/settlement path emits the typed terminal and settlement events |
| Preserving what happened | Gauntlet's state database + content-addressed artifact store, pinned by path + sha256 in append-only manifests |
| Authority, evidence, settlement responsibility | **SEA-Forge** retains the authority/evidence/settlement boundaries; the discovery loop is a governed *consumer* of consequences, never a second authority |
| Retaining a move | A frozen, versioned deterministic selection policy — never a model vote |
| Deciding what survived | Settlement (Gauntlet's own `run_terminal` / `settlement_committed` records) |

**Architectural inference (earned, scoped):** the sequence above is realizable with *no new service, no new ledger, and no product-code change* — as task-local harness machinery over existing seams. Nothing in the stable architecture needs to know how the candidates were proposed.

## 2. Evidence-Backed Results (branch T21–T26; scoped to the proven surface)

* A multi-step route was discovered and retained by the loop and **genuinely settled**: the baseline probe settled through Gauntlet's own verification (3 `settlement_committed` events); the payment boundary was **experimentally observed** (the route's required budget settled; the matched one-notch-lower budget honestly exhausted payment); the adversarial family was separated from the honest family by the frozen mechanical rule (4/4 vs 0/8).
* The settled route was **inverted into a new case** whose independent replay (fresh target copy, fresh seed) **re-settled**; a lookalike case presented as route-solvable was **rejected** by real replay (tooth T7).
* Authority teeth: out-of-authority candidates denied before execution; judgment-preferred-but-ineligible candidates never retained; evidence swaps fail correlation joins; fabricated success refused by mechanism provenance; genuine failures preserved as typed failures; budget exhaustion stops the route honestly.
* Payment was measured against the recorded manual T15/T16 corpus construction (decision entries, provider calls, executions, wall time; unobservable quantities marked n/a — the comparison is useful but **partial**: the historical manual path lacks comparable wall-time/token evidence).
* The mechanism **transferred to a materially different second family** (task T29, adversarially confirmed): applied to the fraud family — whose runs can never settle — the same machinery (catalog → admission → real probes → mechanical rule → frozen route goal → one-factor matched negative) completed in characterize-and-stop mode and measured the family's detection boundary d\* = 2 by real probes. The two adversarial families are partitioned by two complementary mechanical rules (unsubstantiated-pass vs failing-artifact). What did **not** transfer: a settleable route (no honest-completion scenario exists in the catalog), so the two-settleable-families breadth claim remains open.

## 3. Learned vs Mechanical Discrimination (preserved finding)

Mechanical, settlement-backed discrimination over the observation records was **strong and cheap**: the frozen rule (bounded non-settlement terminal ∧ zero settlements ∧ a hash-verified failing verification artifact) separates the adversarial family from honest resource exhaustion with no errors on the frozen ledger.

Learned judgment was **materially weaker** on the same records: under the enriched observation surface and neutral prompts, the provider labeled only **1 of 4** adversarial probes correctly (preserved round-2 records, raw outputs retained).

A frozen remeasurement on the **natively generated corpus** (15 consequence-backed rows; prereg `8142c0a5…`, decisions `D-2026-09-19-RM-01/-02/-03/-04`) confirmed and sharpened this:

* **Mechanical consequence discrimination is the authoritative retention signal.** On this surface the learned layer added **no demonstrated useful distinction** over the mechanical consequence layer: predictive accuracy tied the zero-call face-value baseline exactly (0.600 vs 0.600, McNemar p = 1.0), and classification failed precisely where the mechanical rule succeeds (0/4 adversarial rows, 9/9 honest rows).
* **The errors were systematic, not stochastic**: per-row three-sample agreement was 0.974 and the errors were identical under single-sample and majority-of-3 scoring — the provider anchors on the terminal class and ignores the contradicting artifact facts visible in the same surface.
* **Majority sampling therefore did not repair the error**, and the graded-progress probe reduced to copy-fidelity of a surface field.
* **Learned judgment remains available only as an instrumented/advisory experimental capability** here — never a required route-discovery layer, and never a settlement gate.
* This conclusion is **bounded to the tested surface, provider, and corpus** (one fixture family, one free provider, N=15, naturally generated cases); it is not a universal claim about learned judgment, and the full measurement, error patterns, and payment (78 calls, ~3218 s wall, vs zero calls for the mechanical layer) live in the remeasurement evidence, not here.

Route settlement is evaluated deterministically from consequence records; judgment is never a settlement gate.

## 4. Route-to-Case Inversion and Developmental Memory

A retained settled route can be inverted (settlement-first): derive a candidate case for which the route should be a legitimate solution, freeze the candidate, then require **independent replay settlement** through the real mechanism before accepting the case. Rejections are recorded, never retried into acceptance. Accepted cases extend the developmental record with route fingerprint, generator identity, replay reference, and actual settlement; novelty against earlier corpora is measured (structural duplicates are detectable and recorded, not auto-rejected).

## 5. Unresolved Questions

* Route breadth: the mechanism has now survived two materially different families (one settlement family, one detection family); whether a **second settleable** family exists (two-settleable-families breadth) is still open — the current catalog contains no honest-completion scenario beyond family 1.
* Usefulness-at-transfer of the generated cases for judgment evaluation/training.
* The bounded role of learned judgment on native corpora is resolved for the tested surface/provider/corpus (no demonstrated distinction over the mechanical layer; see the remeasurement evidence); any stronger role requires new independent evidence.
* Promotion (CLAIM D) was explicitly out of scope and remains undecided.

## 6. Evidence Map

Experiment artifacts (append-only, under this repository's `.agents` tree; not part of the shipped docs):

* Frozen preregistration and addenda: `.agents/preregistrations/godspeed-bounded-judgment-T21.prereg.yaml` (+ `addendum-1/2/3`)
* Branch evidence roots: `.agents/evidence/godspeed-bounded-judgment/T21/` … `T26/`
* Adversarial confirmation: `.agents/evidence/godspeed-bounded-judgment/T26/t26-confirmation.md`
* Native-corpus judgment remeasurement: `.agents/preregistrations/godspeed-bounded-judgment-native-corpus-remeasure.prereg.yaml`; evidence `.agents/evidence/godspeed-bounded-judgment/remeasure-native-corpus/`
* Decision chronology: `.agents/evidence/godspeed-bounded-judgment/decisions.yml` (entries `D-2026-09-18-T21-01` … `D-2026-09-19-RM-04`)
