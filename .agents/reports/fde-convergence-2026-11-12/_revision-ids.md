# Draft 2 identifier map (binding for every revised deliverable)

Authority order: `H-egm-review-record.md` revision decisions (RD-01…RD-19) > this map > draft-1 text.
Draft-1 files are preserved under `draft1/` for reference. Keep every decision ID (`AD-n`), invariant (`I-n`),
journey (`J-n`) and existing test ID stable unless H changes its meaning.

## Slices and work packages (replaces draft-1 phases P0–P9 and WP numbering)

| Slice | WP | Title | Repo lane |
|---|---|---|---|
| S0 baseline | WP-00a | Measured baseline (fetch done; serial gate runs per repo) | all |
| | WP-00b | Backup tags on every branch head; push unpushed owner branches as backups | all |
| | WP-00c | Integration branches `fde/integration` per repo, built in new worktrees from copies | each repo |
| S1 walking skeleton | WP-09 | Pre-registration v1 (S1 acceptance tests, falsifiers, program-executor inputs), externally anchored | SEA-Forge |
| | WP-10 | Spike: DomainForge library evaluation over generated facts on a ten-element model | DomainForge / RealityTrace |
| | WP-11 | Contracts package S1 (interaction-event extension, CEP-0008 hash rule, frozen reference as verification contract, comparison profile, observed-representation schema, residual shape, anchor format) | CEP + consumers |
| | WP-12 | DomainForge `evaluate` command (per-policy tristate, bindings, operation-bound policies) | DomainForge |
| | WP-13 | SOP v0.4.1 promoted; IDM `.sea/interaction/fde-sop/` authored and validated | SEA-Forge |
| | WP-14 | Case keeper: `case.record.append` with `episode.frozen` / `episode.settled`, two-phase freeze, I-1, I-3, signed interaction-event emission, external anchor | SEA-Forge |
| | WP-15 | Runtime: interaction-event extension on the CEP outbox; deterministic program-executor harness | Cognate |
| | WP-16 | RealityTrace: CEP-envelope ingest of the extension with source resolution; `sxr-observed` derive | RealityTrace |
| | WP-17 | RealityTrace: `observed compare`, five classes, residual records, comparison profile | RealityTrace |
| | WP-18 | Minimal governed revision: proposal, regression matrix, world pin / promote / re-pin over the existing approval-gated transition, subsequent episode | RealityTrace + SEA-Forge |
| | WP-19 | Horizon read adapters and parity test; `scripts/fde-proof.sh`; S1 gate on a clean cell | Horizon + SEA-Forge |
| S2 Field Case records | WP-20 | Contracts package S2 (field-case document, remaining record kinds, continuation, reserved-operation interface, assignment payload) | SEA-Forge + consumers |
| | WP-21 | Field-case document, strategy, gates, dispositions, lifecycle function, continuation, template checked against model, I-2 | SEA-Forge |
| | WP-22 | Promotion saga, reconciler, world-stamped classifications, key mapping, held-out rule | SEA-Forge + RealityTrace |
| | WP-23 | Assignment binding, per-currency cost, interruption events | Cognate + Workbench |
| S3 identity and authority | WP-30 | Identity package: evidence-verified issuance (D2), scope claims, operator recording | Cognate |
| | WP-31 | Authority bridge: enrolment decision API (D1), controller classes (D4/D6), reserved-operation check validating A3 evidence | SEA-Forge |
| | WP-32 | Horizon: route scope (D3), saga and reconciliation (D5), real P3 authority client, first-controller bootstrap, A3 confirmation on the paired phone, PR split | Horizon |
| | WP-33 | Emitter signatures with workload identities; RealityTrace verification; independent-emitter rule | Cognate + SEA-Forge + RealityTrace |
| | WP-34 | Workbench consumes the identity package; Gauntlet binding record | Workbench + Gauntlet |
| S4 operator experience | WP-40 | Horizon debugger sources and operation extensions | Horizon |
| | WP-41 | Horizon journeys J1–J10 (five proof-critical first) | Horizon |
| | WP-42 | Workbench seam (status strip, awaiting-decision, back-link) | Workbench |
| | WP-43 | Gauntlet / SWE_SEED evidence bridge | Gauntlet + SWE_SEED + RealityTrace |
| | WP-44 | Context Kernel bundle on the assignment brief | Context Kernel + Workbench |
| S5 proof, transfer, operate, release | WP-50 | Pre-registration v2 (full FC-001) | SEA-Forge |
| | WP-51 | FC-001 full run | all |
| | WP-52 | Negative suite collection and verifier-reapplied mutations | all |
| | WP-53 | Evidence bundle, claim ledger, proof command, owner proof ledgers | SEA-Forge |
| | WP-54 | Independent verification (different model CLI, own observations) | independent |
| | WP-55 | Operate package (topology, config, migration, observability, retention/backup) | all + godspeed-readiness |
| | WP-56 | Tax pre-filing transfer case (PARTIAL, fixture-based, with same-gap variant) | SEA-Forge (model, template, scenarios only) |
| | WP-57 | FDE gates added to `local_operator`; release; clean-cell install | godspeed-readiness + all |

## Tests
- Keep AT-01…AT-20 and NT-01…NT-15 (meanings may be tightened per H).
- New negative tests start at NT-16. Required additions: NT-16 backdated freeze / regenerated chain; NT-17
  reference derived from the result; NT-18 model registered after execution; NT-19 unsigned or forged emitter
  event cannot contribute to a classification; NT-20 executor-only evidence cannot yield `confirmed`;
  NT-21 regression matrix (lost capability and unlisted weakening both refuse promotion); NT-22 suspended case
  refuses freeze and delegation; NT-23 case cannot close without a close disposition; NT-24 a reserved decision
  confirmed on the desk page (even with a passkey assertion) is refused, only the A3 surface is accepted.
  More may be added (NT-25+) so that every critical requirement row has at least one negative test.
- Requirement IDs: existing `R-nn` IDs keep their meaning; rows split from a merged row take suffixes
  (`R-71a`…`R-71i`); new rows start at `R-82`.
- Evidence root (single): `sea-rs/.agents/evidence/fde-field-case/<run-id>/`; baseline evidence stays under
  `.agents/reports/fde-convergence-2026-11-12/evidence/baseline/`.
