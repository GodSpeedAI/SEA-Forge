# Source evidence and checkout baseline

The authority order is CognitiveOS theory for navigation ontology, DomainForge and CEP for declared semantics and exchange, accepted decisions for their stated scope, code and tests for implemented behavior, and glossaries or inference as explanatory sources. The workspace baselines below are local Git refs. PR metadata was checked through GitHub. A source-confirmed assertion describes what the inspected code and tests express.

## Checkout inventory

| Repository | Inspected checkout | Relevant additional ref or constraint |
| --- | --- | --- |
| godspeed-growth | `main a1012f0` | The existing `new_world` documentation set declares vocabulary ownership. Theory v17 is unfrozen, and mathematical v2.3 is a draft. |
| Horizon | `main 0e34a34` | Remote-tracking `fde/integration efee391`. Frozen HORIZON-001 v1.2.0 controls the mainline spec. |
| Cognate | `main 29dcd1e` | WP-23 verified assignment start `3aae0d8` adds deadline admission. Main already has the assignment contract. |
| Cognate Workbench | `main f60acb7` | One commit behind local `origin/main efdcc1c`. The inspected vocabulary paths have no diff against that ref. WP-23 binding ref `668f686` is separate. |
| Gauntlet | `main 6a7fe6a` | Remote-tracking `fde/integration 846830d` diverges. |
| DomainForge | `main 35c4ed1` | Application contract and locally resolved semantics control identity. |
| CEP | `main 5111ec3` | `fde/integration 8b6fc90` is distinct. Profile assertions here refer to main unless stated. |
| SEA-Forge | `main 2f2ef65c` | Local main matches PR #107 head and is one commit ahead of its base `4fd6e9f7`. Local `fde/integration 99631343` carries the FDE contract source. Pre-existing working changes were present in status, FDE reports, a skill and Workbench outputs. |
| RealityTrace / sxr | `main 42f38ca` | RealityTrace is the product. `sxr` is the CLI and checkout name. |
| SWE_SEED | `main 737f0b3` | Proof and seed contracts are distinct from operation authority. |
| Context_Kernel | `main b6387cd` | Checkout name contains an underscore. Product language uses Context Kernel. |
| GodSpeed-Agent | `main d6b4580` | The product checkout is separate from older `godspeed` at `f8e7dab`. The older checkout had existing changes and was excluded as an authority. |
| godspeed-compass | `main d52f39d` | Portable v13.1 doctrine and glossary are historical projections. |

The primary checkouts other than SEA-Forge were confirmed clean before editing with targeted `git status` checks. Root and scoped `AGENTS.md`, corresponding Copilot/Claude instructions, EGM where present, and named normative contracts supplied each repository’s boundaries. Recovered Cline session `1795139123148_ftdn7` located sources. Where session claims differed, this report follows the inspected source revisions.

## T1 CognitiveOS

- `new_world/README.md` §§ What This Is and How These Documents Work explicitly identify the canonical set and glossary. The published summary and old glossary compressed the gates and capability conditions. [Canonical set](https://github.com/GodSpeedAI/godspeed-growth/blob/a1012f0/new_world/README.md#what-this-is) · [Summary affordance](https://github.com/GodSpeedAI/godspeed-growth/blob/a1012f0/new_world/cognitive-os.md#L94-L104)
- V17 §§2–4 and 6–13 cover observer roles, horizon, direction, representation and its operators, affordance, payment, governance, settlement, capability, metabolization and reference authority. The draft math §7 distinguishes hard gates from scores, §8 governs reliability, and §§9/14 concern horizon and capability. [V17](https://github.com/GodSpeedAI/godspeed-growth/blob/a1012f0/world/cognitive_os_manifesto_v17.md#7-affordance) · [Math v2.3](https://github.com/GodSpeedAI/godspeed-growth/blob/a1012f0/world/cognitiveos_math_layer_v2_3.md#7-hard-gates-and-continuous-spendability)
- Purposeful Case Navigation §§9, 12, 15–16 and 19 distinguish observer-assessed admissibility, consequence-bearing evidence, epistemic status and authorized disposition. Its broader purposeful navigation includes epistemic moves that do not traverse the full action-bearing OCS loop. [Purposeful Case Navigation and OCS](https://github.com/GodSpeedAI/godspeed-growth/blob/a1012f0/new_world/Purposeful%20Case%20Navigation%20and%20the%20Observer%E2%80%93Case%E2%80%93Settlement%20Model.md#12-observercasesettlement) · [Compass earlier doctrine](https://github.com/GodSpeedAI/godspeed-compass/blob/d52f39d/1-Portable_Context/02-cognitiveos-canon.md)

## D1 DomainForge

- The SEA application contract §§1, 6 and 14 owns entity/record, exact source set framing, resolved closure, canonical model identity and World Reference. The world reference digest covers the entire eleven-field identity tuple. Semantic equivalence uses the closure digest. [Application contract](https://github.com/GodSpeedAI/DomainForge/blob/35c4ed1/docs/reference/sea-application-contract.md#L931-L957)
- The producer separates canonical representation payload, its own content identity, full model identity and closure hash. Invalid model snapshots can carry diagnostics with no representation or minted world identity. [Envelope producer](https://github.com/GodSpeedAI/DomainForge/blob/35c4ed1/domainforge-core/src/application/envelope.rs#L1456-L1519) · [World reference fixtures](https://github.com/GodSpeedAI/DomainForge/blob/35c4ed1/domainforge-core/tests/world_ref_tests.rs#L87-L143) · [Older workflow explanation](https://github.com/GodSpeedAI/DomainForge/blob/35c4ed1/docs/workflows/application-resolution.md#L65-L70)
- The SEA-Forge model itself distinguishes declared semantics from kernel records. [SEA-Forge .sea application](https://github.com/GodSpeedAI/SEA-Forge/blob/2f2ef65c/models/seaforge-system%400.1.0.sea#L8-L18)

## P1 CEP

- CEP defines the envelope boundary explicitly and keeps observation separate from evidence and evidence separate from settlement. The GodSpeed profile table lists seven profiles, plus a separate world-transition record schema. The invalid snapshot branch carries no authoritative World Reference. [CEP specification index](https://github.com/GodSpeedAI/canonical-evaluation-protocol/tree/5111ec3/spec) · [Settlement model](https://github.com/GodSpeedAI/canonical-evaluation-protocol/blob/5111ec3/spec/CEP-0007-settlement-model.md#3-core-definition) · [Profile contracts](https://github.com/GodSpeedAI/canonical-evaluation-protocol/blob/5111ec3/spec/profiles/GODSPEED-PROFILES-v1.md#L28-L72)

## H1 Horizon

- `loadWorld` explicitly substitutes the closure hash for the loader’s digest before comparing the boot pin. It returns the loader’s World Reference separately as `cognateWorldRef`. The Cognate adapter has already generated and verified that value with DomainForge. [Horizon loader](https://github.com/GodSpeedAI/Horizon/blob/0e34a34/src/adapters/semantic.ts#L39-L62) · [Cognate semantic loader](https://github.com/GodSpeedAI/Cognate/blob/29dcd1e/packages/domainforge/src/index.ts#L91-L116)
- The frozen spec owns tiered semantics. Navigation computes gate verdicts and a case view. The handoff conveys a plan item, and the workshop adapter creates a two-part correlation rather than the FDE Case Assignment attempt. [Frozen spec](https://github.com/GodSpeedAI/Horizon/blob/0e34a34/.agents/specs/horizon.agent.spec.yaml#L5-L20) · [Navigation](https://github.com/GodSpeedAI/Horizon/blob/0e34a34/src/domain/navigation.ts) · [Handoff](https://github.com/GodSpeedAI/Horizon/blob/0e34a34/src/capabilities/handoff.ts#L77-L86) · [Workshop adapter](https://github.com/GodSpeedAI/Horizon/blob/0e34a34/src/adapters/workshop.ts#L151-L180)

## C1 Cognate

- WP-23’s validator states the SEA-Forge origin and defines case, step, iteration, attempt, episode and spend. The Bun run stream folds the binding. `CaseContext` references a world, semantic element, case, plan item and, where applicable, episode and assignment. [Assignment contract](https://github.com/GodSpeedAI/Cognate/blob/29dcd1e/packages/runtime-api/src/assignment.ts#L1-L76) · [CaseContext](https://github.com/GodSpeedAI/Cognate/blob/29dcd1e/packages/runtime-api/src/types.ts#L10-L38) · [Bun fold](https://github.com/GodSpeedAI/Cognate/blob/29dcd1e/packages/runtime-bun/src/assignment.ts) · [Execution world port](https://github.com/GodSpeedAI/Cognate/blob/29dcd1e/packages/execution/src/types.ts#L64-L96)
- Local authority source at `sea-rs@99631343:contracts/fde/assignment.v1.json` and adjacent `frozen-reference.v1.json`. The vendored copy is `cognate@29dcd1e:packages/runtime-api/schemas/assignment.v1.json`. That source is a branch contract, not part of SEA-Forge main at the PR head.

## W1 Cognate Workbench

- Expert and assignment identity, Team constraints, Task provenance and handoff types are explicit in Workbench. Journey definitions preserve independent review and human acceptance. The admission path binds the Expert run principal to its registered assignment. [Expert types](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/src/experts/types.ts#L12-L97) · [.sea journeys](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/.sea/interaction/interaction-model.sea#L1715-L1730) · [Expert admission](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/src/experts/admission.ts#L174-L194) · [Retry and review journey](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/test/J-TEAM-execution.test.ts#L93-L122)
- The manifest digest excludes lineage/status. Promotion requires the operator, current parent and evidence. The observation outbox recognizes `world.settled` but omits the governed settlement record. SEA-Forge supplies the governed determination. [Candidate lifecycle](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/src/cognitive/world.ts#L39-L117) · [Observation filter](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/src/observations.ts#L26-L38) · [Governed producer](https://github.com/GodSpeedAI/Cognate-Workbench/blob/f60acb7/src/settlement/orchestrator.ts#L153-L164)
- WP-23’s `src/assignments/assignmentBinding.ts` and `test/J-WP23-assignment-binding.test.ts` are at `668f686`, separate from the Expert types and mainline behavior. [Branch binding](https://github.com/GodSpeedAI/Cognate-Workbench/blob/668f686/src/assignments/assignmentBinding.ts)

## G1 Gauntlet

- The Core spec names the Orchestrator role, recursive AgentSessions, result contract and mailbox hints. The Run Controller is separate. Proof-based promotion follows a stronger evaluation than a single controlled run. [Core spec](https://github.com/GodSpeedAI/Gauntlet/blob/6a7fe6a/.agents/specs/Core-spec.md#L785-L806) · [Role invocation port](https://github.com/GodSpeedAI/Gauntlet/blob/6a7fe6a/crates/gauntlet-ports/src/agent_runner.rs#L420-L457) · [Run Controller](https://github.com/GodSpeedAI/Gauntlet/blob/6a7fe6a/crates/gauntlet-app/src/control/controller.rs#L745-L790) · [Capability and promotion](https://github.com/GodSpeedAI/Gauntlet/blob/6a7fe6a/README.md#one-settled-run-is-not-durable-capability)

## S1 SEA-Forge

- Minimum spec §§7.3.2, 7.3.5, 7.3.8 and 10.4 define Case, authority, evidence and the kernel-local evaluation. Full spec §7.2.1 defines the separate settlement authority and declaration. [Minimum spec](../../specs/spec-minimum.md#104-settlement-rules) · [Full spec](../../specs/spec-full.md#721-settlementauthority--settlementdeclaration-new-e4m4a)
- SEA-Forge verifies world identity against its registry, accepts registered current scope, and checks recorded transition authority for cross-world lineage. The operation path corroborates evidence, compares criteria, and commits scoped final or non-final outcomes. The E6 return is a separate governed work path. [World binding](../../../crates/sea-forge-domainforge/src/world.rs) · [CEP authority and settlement](../../../crates/sea-forge-authority/src/cep.rs) · [Migration spec](../../specs/cep-world-ref-migration.spec.md) · [E6 settlement return](../../../crates/sea-forge-server/src/governed_settlement_return.rs)

## R1 RealityTrace

- RealityTrace distinguishes expectation, observation, evidence, verification and the ledger. The SEA-Forge bridge states its own E6/E7/E8 ownership split and resolves source observations for corroboration. [RealityTrace model](https://github.com/GodSpeedAI/RealityTrace/blob/42f38ca/sxr-core/src/types.rs#L207) · [Record kinds](https://github.com/GodSpeedAI/RealityTrace/blob/42f38ca/sxr-core/src/record_kinds.csv) · [SEA-Forge bridge](../../../crates/sea-forge-realitytrace/src/lib.rs)

## K1 Context Kernel

- Core governs retrieval entities and access. Reference material, session state and large context artifacts serve different needs. The access decision concerns data disclosure. [Context contract overview](https://github.com/GodSpeedAI/Context-Kernel/blob/b6387cd/README.md#global-invariants) · [Context family](https://github.com/GodSpeedAI/Context-Kernel/blob/b6387cd/README.md#three-kinds-of-context-fail-differently)

## Q1 SWE_SEED

- Native seed and layer contracts own assembly scopes and provenance. Live proof qualification controls capability evidence at the applicable boundary. No standalone SWE_SEED adjudicator was confirmed for the SEA-Forge E6 return. [Seed spec](https://github.com/GodSpeedAI/swe_seed/blob/737f0b3/SWE_SEED_SPEC_v0.2.0.md#L29-L47) · [Layer boundary](https://github.com/GodSpeedAI/swe_seed/blob/737f0b3/docs/specs/0018-layer-boundary-governance.md#L23-L69) · [Proof qualification](https://github.com/GodSpeedAI/swe_seed/blob/737f0b3/docs/specs/0013-eval-and-proof.md#L31-L35)

## A1 GodSpeed-Agent

- Ingest binds model and world references independently. Provisional evidence is inert. Runtime settlement records a developmental interpretation, and promotion requires cited settlement history and distinct declared variation for metabolization. The source checks stop short of showing the full CognitiveOS capability conjunction. [Evidence ingest](https://github.com/GodSpeedAI/GodSpeed-Agent/blob/d6b4580/godspeed_nav/evidence_ingest.py#L331-L404) · [Runtime assessment](https://github.com/GodSpeedAI/GodSpeed-Agent/blob/d6b4580/godspeed_nav/runtime.py#L543-L645) · [Developmental memory](https://github.com/GodSpeedAI/GodSpeed-Agent/blob/d6b4580/godspeed_nav/developmental_memory.py#L185-L297)

## E1 PR 107

[PR #107](https://github.com/GodSpeedAI/SEA-Forge/pull/107) was open with head `2f2ef65c2cc961e3451421d28895e764edeee27b`. The concepts brief, current-state investigation, architecture, minimal contracts, ledger, decisions, closure register and transformation brief were compared at that head. They record a design. The closure register is revision 5 within a single PR commit.

- **Adapter and admission.** The concepts brief, B/G3, decision 3 and parts of H retain extraction wording. I §I2–I4 selects an ACP-speaking optional adapter and port-based governance, with ACP claims for observation and cancellation. Earlier universal refusal or bridge-only language applies to the inspected wrapper or those earlier designs. I3b and acceptance evidence are implementation work. [Concepts brief](../2026-11-19-cognitiveos-convergence/concepts_brief.md) · [Target architecture](../2026-11-19-cognitiveos-convergence/C-target-architecture.md) · [Contracts](../2026-11-19-cognitiveos-convergence/D-minimal-contracts.md) · [Revision 5](../2026-11-19-cognitiveos-convergence/I-closure-register.md)
- **Observation and scope.** A and decision 11 say both settlement families feed RealityTrace. B/G12 and H correct that statement and match the Workbench outbox. I §I4 makes the prohibition on using local hashes as CEP world identity explicit. B/G5 correctly points to a Cognate Assignment contract. Workbench WP-23 is evidence for linking a session start, and supplies no equivalence to the Expert lifecycle. [Actual state](../2026-11-19-cognitiveos-convergence/A-actual-state.md) · [Decision log](../2026-11-19-cognitiveos-convergence/DECISION-LOG.md) · [Gap ledger](../2026-11-19-cognitiveos-convergence/B-gap-and-convergence-ledger.md) · [Transformation brief](../2026-11-19-cognitiveos-convergence/H-transformation-brief.md)
