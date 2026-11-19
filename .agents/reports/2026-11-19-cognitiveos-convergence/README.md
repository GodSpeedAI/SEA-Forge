# CognitiveOS Convergence — EGM design package (2026-11-19)

Status: **Draft for review.** Design only. No runtime code was changed.

This package answers one question: what is the smallest coherent change set that makes
Cognate's RLM capability first-class and provider-independent, lets Cognate Workbench
Experts/Teams use it without a second agent hierarchy, keeps Workbench usable alone, and
preserves CEP identity and settlement authority across Horizon, Gauntlet and SEA-Forge?

## Reading order

| File | What it is |
|---|---|
| [`concepts_brief.md`](concepts_brief.md) | EGM brainstorm output: the problem, what survived criticism, alternatives refused |
| [`design.md`](design.md) | The EGM design document (problem, goals, CUJs, architecture, alternatives, NFRs, plan) |
| [`A-actual-state.md`](A-actual-state.md) | Verified current state, with citations, split into implemented / partial / specified-not-built / absent |
| [`B-gap-and-convergence-ledger.md`](B-gap-and-convergence-ledger.md) | Every gap with disposition KEEP/REUSE/EXTEND/EXTRACT/CONSOLIDATE/REPLACE/DELETE/NEW |
| [`C-target-architecture.md`](C-target-architecture.md) | Seven Mermaid diagrams of the target system |
| [`D-minimal-contracts.md`](D-minimal-contracts.md) | The only public contracts that must exist, plus refusal and recovery semantics |
| [`E-acceptance-journeys.md`](E-acceptance-journeys.md) | 14 end-to-end journeys with observable pass/fail and negative tests |
| [`F-implementation-sequence.md`](F-implementation-sequence.md) | Dependency-ordered increments, smallest first |
| [`G-residual-uncertainty.md`](G-residual-uncertainty.md) | Unverified claims, owner decisions still needed, design risks |
| [`H-transformation-brief.md`](H-transformation-brief.md) | Handoff for the Matt Pocock implementation workflow |
| [`I-closure-register.md`](I-closure-register.md) | Every open question closed: port-governance gaps, three admission routes, eight decisions, five decision trees, accepted residuals |
| [`DECISION-LOG.md`](DECISION-LOG.md) | 15 decisions: alternatives, evidence, trade-off, and what would overturn each |
| [`egm-review/adversarial-review.md`](egm-review/adversarial-review.md) | The 12 required attacks, each answered with evidence, plus what stayed open |
| [`egm-review/review-record.md`](egm-review/review-record.md) | Goldfish lens findings and how each was reconciled |
| [`egm-review/goldfish_review_report.md`](egm-review/goldfish_review_report.md) | `egm-resolve` pass over the residual-uncertainty register: every item categorized, with the evidence that settled it |
| [`recon/`](recon/) | Raw evidence inventories produced by read-only recon agents |

## Design baseline (exact commits)

| Repo (folder) | Branch | HEAD | Notes |
|---|---|---|---|
| Cognate (`cognate`) | `main` | `29dcd1e` | packages at 0.10.0 |
| Cognate Workbench (`cognate-workbench`) | `main` | `f60acb7` | plus in-flight `feat/capability-runtime-t3` `d84b73b`, `t4a` `bbe6c73`, `t4b` `efdcc1c`, `feat/wp23-assignment-binding` `668f686` |
| Horizon | `main` | `0e34a34` | also `fde/integration` `4242d31` (detached worktree) |
| Gauntlet (`gauntlet`) | `main` | `6a7fe6a` | also `fde/integration` `0e185aa` |
| DomainForge (`domainforge`) | `main` | `35c4ed1` | `fix/replace-chrono` `0226b50` in a worktree |
| SEA-Forge (`sea-rs`) | `main` | `2f35ad27` | also `fde/integration` `537a50b7` (2026-11-19, newer than main) |
| CEP (`cep`) | `main` | `5111ec3` | `fde/integration` `8b6fc90` |
| RealityTrace (`sxr`) | `main` | `42f38ca` | folder name is `sxr`; the product is RealityTrace |
| SWE_SEED | `main` | `737f0b3` | used only as a proof source |
| Context_Kernel | `main` | `b6387cd` | inspected for cross-product contracts only |

Two baselines matter and are not the same: `sea-rs` `fde/integration` is one day newer than
`main`, and Workbench's capability-runtime work (T3/T4a/T4b, WP-23) is unmerged. The design
names exactly which baseline each increment assumes (see [`F`](F-implementation-sequence.md)).

## Revision status

**Revision 5 (closure).** Every remaining item is decided in `I-closure-register.md`: the port's
limitations each get a mechanism and a guard (grant→fs mode, proxied egress with credential
substitution, systemd pid/memory ceilings, manifest-diff evidence), the adapter speaks **ACP**
(verified against `prime-agent` 0.10.2) rather than extracting Gauntlet's host, I3b is in scope, and
the five execution-dependent checks are converted to pass/fail decision trees. No owner decision
remains open.

**Revision 4 (port governance).** The owner observed that a provider can be governed by governing
the *port*, not only its tool list. Verified on this host with bubblewrap 0.13.1: a contained
world holds a live stdio process under `--unshare-all`, denies the network, and reaches a
bind-mounted unix socket. The design now has two admission routes (`D8`), a contained streaming
increment (`I3b`), and a precise refusal where the port cannot enforce the parent's grant. This
supersedes revision 3's "contained providers are impossible" conclusion.

**Revision 3 (resolve pass).** The three questions carried into the handoff, and every row of `G`,
were run through `egm-resolve`. Outcomes: Prime 0.10.2 cannot be restricted to bridged tools, so
its adapter is `unattested`; a contained provider is blocked by two independent facts, not by
policy; `pluginRegistry` has four live consumers and is no longer a deletion candidate; the `sea-rs`
settlement diff is material, so the design baseline stays `main`; seven CEP profiles plus one record
schema is the correct count. `H-transformation-brief.md` is written.

**Revision 2 (post-review).** Revision 1 was reviewed by an EGM design-critic Goldfish. It found
three blockers that were real: a fictional `native` RLM runner, a spawn extraction that would have
moved six app-owned stores, and authority/depth/budget rules with no enforcement point in Cognate.
Revision 2 removed the first, replaced the second with a declared `StdioSpawnPort` implemented by
the host, and restated the third as caller-attested and re-checked. It also reversed the claim that
providers can be admitted under governance. The full trail is in [`egm-review/`](egm-review/).

**Branch-qualified citations.** Files that exist only on an unmerged branch are cited as
`<repo>@<branch> (<commit>):<path>`. At the baseline those paths do not exist on `main`.

## Verification status of this package

- Every architectural claim cites a path and line, a commit, or a named test.
- Citations marked **‡** come from the recon reports and were not re-read line-by-line by
  the design author; everything else was read directly at the baseline commit.
- No test suite, build, or live cross-process run was executed. Live behaviour claims are
  therefore source-level. See [`G`](G-residual-uncertainty.md) §G6 for what only execution can settle.
