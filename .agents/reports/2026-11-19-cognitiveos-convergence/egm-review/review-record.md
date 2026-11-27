# EGM review record

## Process actually run

| Stage | What was done | Output |
|---|---|---|
| Brainstorm | Problem framing, premise stress-test, alternatives refused | [`concepts_brief.md`](../concepts_brief.md) |
| Codebase recon (before design) | Six read-only recon agents over Cognate, Workbench, Horizon, Gauntlet, CEP/DomainForge, SEA-Forge/RealityTrace, each with file:line evidence and an explicit verified/unverified split | [`../recon/`](../recon/) |
| Draft | Design document and appendices A–H, decisions 1–15 | this package |
| Critic review | Fresh-context design-critic Goldfish, adversarial, spot-checking 8–12 load-bearing citations | [`design-critic.md`](design-critic.md) |
| Remediation | Every blocker and major reconciled in place; revision 2 | `D`, `F`, `B`, `C`, `E`, `DECISION-LOG`, `design.md` |
| Implementation readiness | Fresh-context readiness review of revision 2 | [`readiness.md`](readiness.md) |
| Adversarial review | The twelve required attacks, applied to revision 2 | [`adversarial-review.md`](adversarial-review.md) |
| Closure review | Brainstorm + external research (ACP spec, Pi security/containerization docs) + 12 probes; every residual item turned into a decision with a guard | [`../I-closure-register.md`](../I-closure-register.md), revision 5 |
| Port-governance review | Owner question: can a provider be governed by governing the port? Smoke-tested bubblewrap 0.13.1 (stdio under `--unshare-all`, network denial, bind-mounted socket) and revised the admission rule, contracts (D8), journeys (J13) and sequence (I3b) | revision 4 of `D`, `F`, `G`, `E`, `H` |
| Resolve | `egm-resolve` over the residual register and the three open questions, with new evidence from the installed `prime-agent`, live import graphs, branch diffs and the CEP directory | [`goldfish_review_report.md`](goldfish_review_report.md), revision 3 of `G`, and `H` |

## What the critic found, and what happened to it

| Finding | Severity | Disposition |
|---|---|---|
| B1 `native` runner incoherent (one run vs in-process loop) | blocker | **Removed.** No `native` provider; `runner` is `none` or a provider id; the child run is `agent.rlm` for the Assignment's execution. |
| B2 the "extracted" spawn is refused in every configuration it is needed for | blocker | **Accepted as a limitation.** Providers are refused under governance; the admission table is part of `rlm.providers`; G7 changed from "admit" to "keep the refusal, make it precise". |
| B3 authority/depth/budget have no enforcement point in Cognate | blocker | **Withdrawn claim.** They are caller-attested and re-checked at the seam; the real grant stays Workbench's. |
| M1 governed settlement never reaches the observation stream | major | **Corrected premise and scope.** G12 rewritten; I5 now adds the governed family to the stream and budgets the extra SXR ingest. |
| M2 G1 labelled NEW although Gauntlet's port exists | major | **Relabelled** NEW code / REUSE vocabulary, with `isolation` and `resultDelivery` adopted and a parity test. |
| M3 invented Gauntlet CLI flags and status words | major | **Replaced** with `--config`, exit codes 0/1/2, six terminal states, `runners.default` as an operator precondition. |
| M4 `requestedBy` can never match | major | **Demoted** to `assertedBy`, recorded, never a refusal. |
| M5 `caseContext` asserted, not corroborated | major | **Added** `worldCorroboration: verified | asserted`. |
| M6 `_wt/...` citations do not exist at the baseline | major | **Replaced** with branch-qualified citations. |
| M7 "EXTRACT" hid six app-owned dependencies | major | **Scoped down** to a declared `StdioSpawnPort` implemented by the host. |
| M8 `experts.restore` clauses contradicted each other | major | **Rewritten** with create/reactivate/keep semantics. |
| 8 minors (naming, copy, citation precision) | minor | Applied where they affected meaning; the rest are recorded in `G`. |

## Second review (implementation readiness)

A fresh-context readiness reviewer produced 12 must-fix items against revision 2, all at document
seams rather than architecture: the journeys had not been regenerated, no increment owned the
Cognate *release*, `caseContext` did not match the published type, the parent had no way to resume
after `rlm.invoke`, invocation state had no home, `rlm.reconcile` duplicated the existing ambiguity
machinery, one failure code was unreachable, authority was specified two ways, Gauntlet was invoked
with flags it does not have, four items had no owning increment, one deletion contradicted its own
increment, and retry keys disagreed across three files. All twelve were fixed in place; the mapping
is [`../G-residual-uncertainty.md`](../G-residual-uncertainty.md) §G5. The reviewer's report is kept
unchanged in [`readiness.md`](readiness.md) so the trail shows what was found, including items that
were fixed rather than argued away.

## What the review did not resolve

* Whether Prime can be restricted to bridged tools. No evidence on either side; it now blocks I4
  explicitly rather than being assumed.
* Whether a `contained` provider could ever be authorised. Owner decision O2.
* Whether `pluginRegistry` still has a live consumer. Deletion stays gated on an inventory.
* Any runtime behaviour. No suite, build or live service was executed in this phase.

## Honesty notes about this package

* Citations marked **‡** were taken from recon reports and not re-read line-by-line by the design
  author. The critic re-checked several of the unmarked ones and found them accurate; it also
  found the marked-adjacent `_wt` paths unusable, which is fixed.
* The design's own earlier claim that both settlement families feed the observation stream was
  false, and the critic caught it. It is now stated as the opposite defect, with a fix.
* Revision 2 is weaker in one respect than revision 1 claimed: it no longer promises a working RLM
  capability in Cognate before a provider is installed. That is the correct claim.
