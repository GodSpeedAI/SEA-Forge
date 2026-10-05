# Stage 9 settlement — remaining boundaries

Date: 2026-10-05. Spec: `specs/cep-world-ref-migration.spec.md` (Stage 9, with an "as built" section). Plan: `plans/2026-10-05-stage9-remaining-boundaries.md` (written before coding).

## What changed, by repo

| Repo | Branch | Commit | What |
|---|---|---|---|
| Context_Kernel | `migration/cep-world-ref` (new, from `harden`) | 30e8ce1 | `ContextRequired` requires a well-formed `world_ref`; the packet echoes it and states `retrieval_completeness` (`complete`/`partial`/`none`) with `omissions`. Hitting `max_results` with a further match, or truncating a citation, is `partial`. |
| godspeed_agent | `migration/cep-world-ref` (new, from `audit-corrections/neatcode-2026-07-29`) | baf6005 | `DesiredDirection`/`WorkRequested` require `world_ref`; consequential loop events refuse to emit without a pinned world; `SettlementRecorded` is labelled `agent_local`; inbound evidence is checked against the deployment pin. |
| SWE_SEED | `migration/cep-world-ref` (worktree `~/projects/SWE_SEED-cep`, from `deploy-prep`) | a1a3c3d | `world_ref` required at E1, sent at E2, checked on the E3 packet, carried by E4, bound at E6 adjudication, carried by E7. `ContextRequest.require_complete` refuses partial context. |
| sea-rs | `migration/cep-world-ref` | 05b723b | SEA-Forge's legacy E4→E5A→E5B→E6 library chain pins the world at intake, keeps it on the ledger record, and emits it on the E6 settlement. `GovernedWorkIntent::verify_world` hooks the registry check. |
| gauntlet | `migration/domainforge-0.18.2` | 752cea6 | Confirmation tests only. |
| cognate | `cognate/harness` | 6118ac2 | Stage 8 status revision 36 and evidence file (carried over from Stage 8). Cognate is otherwise unchanged. |

## What is and is not claimed
- Syntax and equality are checked in Context Kernel, GodSpeed-Agent and SWE_SEED. None of them verifies a digest. SEA-Forge verifies on the CEP authority path; the legacy library chain has an opt-in hook (M-37).
- Partial context is surfaced, not coerced: a packet that does not state `complete` reads as not complete, and a consumer can refuse it.
- GodSpeed-Agent's settlement is its own cycle outcome, not SEA-Forge's.
- A request carries no authority; the tests assert the `WorkRequested` payload has no authority or decision fields.

## Verification (every suite run with the cross-repo roots pointed at the migration worktrees)
- Context Kernel: `cargo test --workspace` 230 passed, 0 failed; clippy `-D warnings` clean. Live: SWE_SEED's `context_kernel_client` test against the real CK binary passes (world echoed, `complete`, no omissions).
- godspeed_agent: pytest 1131 passed, 2 skipped, 0 failed (1099 before; +32 new).
- SWE_SEED: `cargo test --workspace` 440 passed, 0 failed; clippy back to its 45-warning baseline.
- sea-rs: `just test` 1180 passed, 0 failed (1170 before; +10); `just check` green.
- Gauntlet: `bun test` 1032 pass, 28 skip, 0 fail; `tsc` clean; biome 0 errors.
- Cross-language fixtures regenerated from real producers: GSA `WorkRequested` → SWE_SEED E1 ingress; SWE_SEED `GovernedWorkRequest` → SEA-Forge E4 intake; SEA-Forge `OperationalSettlement` → SWE_SEED adjudication. `sxr`'s 12 T07 ingestion tests pass against SWE_SEED's new `ProofCompleted` (fixture not committed there).

## Debt
M-37 to M-44 in `.agents/DEBT.md`: the legacy chain's digest is never verified in the library path; RealityTrace's legacy wire has no world; some producers and mirrors are not pinned; fixture tests dirty sibling checkouts by default; Gauntlet is still not a verifier; pre-existing lint failures; `mise trust` friction; the new branches have no PR.
