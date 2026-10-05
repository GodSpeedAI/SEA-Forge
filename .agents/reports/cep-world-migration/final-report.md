# CEP-0008 world_ref migration — final report

2026-10-05. Stages 0–11 complete. Stage 12 is this report. Nothing below has been merged except what is marked
"merged"; the open PRs are listed with their checks.

## Architecture

One immutable thing identifies a semantic world, and one component owns each kind of authority:

| Concern | Owner | Mechanism |
|---|---|---|
| Meaning and identity | DomainForge | `world_ref` = `world:<name>@sha256:<digest>`, digest = `DomainModelIdentity::canonical_digest()`. Changes with any source edit, even a comment, and with every release; `semantic_closure_hash` is the stable part. |
| Governance and settlement | SEA-Forge | Recomputes every world it governs from source (never trusts a sender's digest); decides each consequential transformation; derives state from a ledger; an allowed move between worlds is a recorded, approved `world_transition`. |
| Operation | Cognate | One `GovernedOperation` choke point; speaks CEP envelopes to SEA-Forge. |
| Observation | RealityTrace (`sxr`) | Observes; never decides. |
| Interchange | cep (CEP-0008) | `godspeed.*` profiles v1.0.0 (+ `context_bundle`), shared corpus, Python/Rust/TypeScript validators with identical verdicts. |

The legacy `sea.agent.event.v1` loop (E1 WorkRequested … E8 EvidenceRecorded) carries `world_ref` next to the legacy
`domain_model_hash` and never derives one from the other. Nothing is collapsed: partial context is not complete,
agent-local settlement is not SEA-Forge settlement, a request is not authority, a trace is not evidence is not
settlement. Everything fails closed.

Two doors, deliberately: CEP verbs are the only way into SEA-Forge's decisions. The legacy chain is a contract
library (with a verified entry point), so one decision never has two paths.

## Versions

DomainForge 0.19.0 (crates.io, PyPI, npm, GitHub release v0.19.0); every consumer is on it: SEA-Forge
(`sea-forge-domainforge =0.19.0`), Cognate (CLI and `BINDING`), RealityTrace 0.3.0 (pinned CLI), Gauntlet, legacy SEA.
cep profiles 1.0.0; CEP-0008 envelope 1.0.

## Repositories

Merged earlier (stages 0–10): cep #1 #2, DomainForge #132 #133 #134, Context-Kernel #1, GodSpeed-Agent #1, swe_seed
#2, Gauntlet #1, Cognate #1 #2, SEA-Forge #7 #8 (squashed). Stage 11, all OPEN, all pushed, local == remote:

| Repo | PR (base) | Branch @ SHA | Checks |
|---|---|---|---|
| SEA-Forge | #9 (`casework/live-wiring`) | `migration/cep-legacy-chain-verified` @ the PR head | lint, test, package pass; workbench and macOS fail on unrelated base issues (M-53) |
| RealityTrace | #5 (`main`) | `migration/cep-world-evidence` @ 0461b50 | pass; last commit is test-only |
| GodSpeed-Agent | #2 (`audit-corrections/neatcode-2026-07-29`) | `migration/cep-world-required-evidence` @ 4bfed12 | none (base has no CI); 1132/0 local |
| GodSpeed-Agent | #3 (same base) | `ci/pytest` @ 47e3c37 | pass |
| Context-Kernel | #2 (`harden`) | `migration/cep-context-bundle` @ 270e911 | pass |
| swe_seed | #3 (`deploy-prep`) | `e2e/world-loop` @ 86ef9d3 | pass |
| cep | #3 (`main`) | `profiles/context-bundle` @ d7591c1 | pass |
| Cognate | #3 (`main`) | `ci/verify` @ 17846d9 | pass |

Merge order that keeps every base green: RealityTrace #5, then GodSpeed-Agent #3 then #2, cep #3, Context-Kernel #2,
swe_seed #3, Cognate #3, SEA-Forge #9 (squash only). Legacy SEA `migration/domainforge-0.18.2` still has no PR (M-01).

## Verification

See `stage11-settlement.md`. In short: SEA-Forge 1197/0; Cognate verify 78/78 and live 33/0; Context Kernel 237/0;
SWE_SEED 443/0; RealityTrace 201/0; GodSpeed-Agent 1132/0; cep 149/0, 67/0, 10/0; `just e2e-world-loop` OK.
Proven end to end, not per hop: one world from E1 to the evidence the agent records, SEA-Forge agreeing independently
with DomainForge's world, and the refusals for the right reasons.

## What is not production-ready yet, and why

Blocking nothing, but real: E5B execution is simulated in the loop and the loop is not in CI (M-48: needs five
checkouts); a transition moves nothing (M-46: decision below); transition compatibility is a sender's claim unless the
closures are equal (M-45: needs a DomainForge world diff); Gauntlet is not a verifier (M-41); `.githooks/*` are not
executable so no local hook runs (M-50: enabling them changes behaviour); the full open list is
`.agents/DEBT.md` (kept untracked in the main checkout).

## Decisions that are the operator's

1. Merge the eight open PRs (each squash-only where the repo requires it).
2. M-46. My recommendation: an approved transition becomes the only way a request may cite a lineage from another
   world, checked in SEA-Forge; in-flight work stays pinned to its world and is never re-pinned.
3. Whether to enable `.githooks/*`.
