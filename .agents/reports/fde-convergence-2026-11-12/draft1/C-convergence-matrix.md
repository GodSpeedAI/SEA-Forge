# C. Functional convergence matrix and fidelity definition

Sources: `Mission.md` (M§n, evidence contract ME, execution MX), `sop/GS-FDE-SOP-001-v0.4.md` (S§n), and the
operator capabilities in the mission brief (U). Work packages (`WP-nn`) are in deliverable F; acceptance and
negative tests (`AT-nn`, `NT-nn`) in deliverable G; journeys (`J-n`) in deliverable B; decisions (`AD-n`) and
invariants (`I-n`) in deliverable E.

## 1. Fidelity assessment (fixed before implementation)

- **Unit:** one row below = one individually testable requirement.
- **Status:** `PROVEN` (working behaviour, reproducible evidence, test executed in the integrated gate),
  `PARTIAL`, `BLOCKED`, `NOT TESTED`. Only `PROVEN` counts.
- **Fidelity** = PROVEN rows / applicable rows. A row is "not applicable" only with a recorded rationale approved
  in the EGM record. Target: at least 99 %.
- **Critical invariants** (column `C` = ●) must be 100 % PROVEN, each with a negative test that fails for the
  intended reason when the protected property is broken (mutation check). One unproven critical row makes the
  overall verdict NOT MET whatever the percentage.
- **Evidence levels stay distinct** (ME8): a passing row proves the bounded interaction. Reliability and transfer
  are separate rows (R-71 to R-73) and are expected to finish `PARTIAL` after one case.
- A test that skipped a prerequisite counts as `NOT TESTED`.

Today column: `✓` exists and evidenced, `◐` exists partly or unverified, `✗` absent.

## 2. Matrix

### Scenario and declared model

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-01 | Source evidence for declared intent (the SOP), implemented behaviour (code, schemas), actual executions | M§1 | | sea-rs | ◐ | WP-02, WP-51 | J2 | AT-01 |
| R-02 | Scenario contains conforming, contradicting, legitimate-undeclared and missing-observation behaviour, from real execution paths with deterministic inputs | M§1 | ● | sea-rs | ✗ | WP-50, WP-51 | J6 | AT-07 |
| R-03 | Contextually different meanings of one term are preserved (e.g. "settlement", "projection" across practice contexts) | M§1, S§2.2 | | DomainForge | ◐ semantic packs | WP-02 | J2 | AT-01 |
| R-04 | Valid `.sea` IDM derived by the existing IDM method | M§2, S§6 | | DomainForge / sea-rs | ◐ method exists | WP-02 | J2 | AT-01 |
| R-05 | IDM covers purpose, actors, entities, states, operations, preconditions, affordances, policies, authority, expected outcomes, evidence and settlement obligations | M§2 | | sea-rs | ✗ | WP-02 | J2 | AT-02 |
| R-06 | Every substantive assertion classed declared / observed / inferred / proposed / unknown with provenance | M§2, S§4.2 | | sea-rs | ◐ pattern in Horizon IDM | WP-02, WP-03 | J1, J2 | AT-02 |
| R-07 | Validated by the real compiler; source hash, `world_ref`, compiler version, validation output recorded | M§2 | ● | DomainForge | ✓ CLI | WP-02 | case strip | AT-01 |
| R-08 | Declared model established before evaluated executions | M§2 | ● | RealityTrace | ✓ predecessor rules | WP-51 | J3 | AT-04, NT-01 |

### Field Case, navigation and control

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-10 | Field Case exists as an authoritative, versioned case with the §4.2 content groups | S§1.1, §4.2 | | SEA-Forge | ◐ cases, no field-case document | WP-03 | J1 | AT-03 |
| R-11 | Case opened or retrieved in Horizon and confirmed against the authoritative record | S§4.1 | | Horizon | ◐ case keeper | WP-30, WP-31 | J1 | AT-03, AT-14 |
| R-12 | Reopened case is continuable without the previous worker; journal never substitutes for records | S§4.3 | ● | SEA-Forge / Horizon | ◐ handoff exists | WP-03, WP-31 | J1, J9 | AT-15 |
| R-13 | Strategy decision recorded before substantive work and revisable | S§4.4 | | SEA-Forge | ✗ | WP-03 | J1 | AT-03 |
| R-14 | Activities are repeatable, omittable, unordered | S§6 | | SEA-Forge | ◐ discretionary items | WP-03 | Work | AT-03 |
| R-15 | Candidate moves compared on seven dimensions; blocked and unknown shown as such | S§1.2, §7.2, U | | Horizon (reads all) | ◐ `reach` ladder | WP-30 | J3 | AT-10 |
| R-16 | Evaluated episode's reference fixed and hashed before execution | S§7.2, M§3 | ● | SEA-Forge + RealityTrace | ◐ question/threshold exist | WP-03 (I-1, I-3) | J3 | AT-04, NT-01 |
| R-17 | Exploratory observation labelled as such at the time, never relabelled | S§7.2 | ● | SEA-Forge | ✗ | WP-03 | J3 | NT-13 |
| R-18 | Assignment bound to case, world, criterion, authority, budget, with the §7.4 brief | S§2.1, §7.3-7.4 | | SEA-Forge / Cognate | ◐ delegation | WP-03, WP-05, WP-06 | J4 | AT-17 |
| R-19 | SEA-Forge authorization at the action boundary for consequential actions | S§7.3 | ● | SEA-Forge | ✓ CEP loop | WP-04 | J4 | AT-05, NT-05 |
| R-20 | Execution, cost, external effects, denials, interruptions captured | S§7.3, §5.3 | | Cognate / Workbench | ◐ no cost ledger in Cognate | WP-05, WP-06 | Work, Cost | AT-17 |
| R-21 | Changing criteria, meaning or policy starts a new authorized evaluation version | S§7.3 | ● | SEA-Forge | ✗ | WP-03 (I-3) | J3 | NT-01 |
| R-22 | Result classed S / F / U / C; reliability and independence recorded separately | S§7.5 | | RealityTrace / SEA-Forge | ◐ settlement kinds | WP-03 | J5 | AT-09 |
| R-23 | Executor cannot rewrite frozen criteria or attest its own acceptance | S§3.2 | ● | SEA-Forge | ◐ SoD on `proposed_by` | WP-03 (I-2) | J5 | NT-10 |
| R-24 | Six dispositions, chosen by the authorized actor | S§7.5 | ● | SEA-Forge | ✗ | WP-03, WP-21 | J5 | AT-10 |
| R-25 | Four status axes distinct and never collapsed | S§7.6 | | SEA-Forge / Horizon | ✗ | WP-03, WP-31 | all | AT-10 |
| R-26 | Tool completion is not settlement; settlement is not closure or capability | S§7.5 | ● | all | ◐ SEA-Forge principle | WP-03 | J5 | AT-10 |
| R-27 | Nine gates with states met / blocked / unassessed / not applicable; no automatic advance; reassess on evidence change | S§8 | | SEA-Forge | ◐ milestones | WP-03 | J8 | AT-11 |
| R-28 | Not-applicable never waives a hard governance gate | S§8.1 | ● | SEA-Forge | ✗ | WP-03 | J8 | NT-11 |
| R-29 | Payment priced and measured in separate currencies; unmeasured is `unknown` | S§5 | | Workbench / Horizon | ◐ spend ceilings | WP-06, WP-31 | Cost | AT-17 |
| R-30 | Reliability dimensions T/A/I/K/D/X/N applied only where thresholds are set | S§7.7 | | RealityTrace | ◐ | WP-12 | J5 | AT-09 |

### Semantic exhaust

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-31 | Workflow executed through the real runtime path | M§3 | ● | Cognate / SEA-Forge | ◐ live loop skips without prerequisites | WP-05, WP-51 | Work | AT-05 |
| R-32 | Events carry model and element identity, actor, operation, resource, context | M§3 | ● | all emitters | ◐ | WP-01, WP-04, WP-05, WP-06 | inspector | AT-05, NT-02, NT-15 |
| R-33 | Events carry correlation, causation, authority decision | M§3 | | all emitters | ◐ | WP-01, WP-04, WP-05 | timeline | AT-05 |
| R-34 | Events link expected outcome (pre-action reference) and actual outcome, evidence, settlement | M§3 | ● | RealityTrace | ◐ `evidence.recorded` | WP-01, WP-11 | J5 | AT-05 |
| R-35 | Original source records preserved; canonical contracts and adapters used | M§3 | | all | ✓ CAS | WP-01 | inspector | AT-05 |
| R-36 | Independent verifier can establish the expectation pre-existed the execution | M§3 | ● | RealityTrace | ✓ hash chain | WP-51 | replay | NT-01 |
| R-37 | Instrumentation begins as soon as an episode needs evidence; coverage gaps reported | S§6.1 | | RealityTrace | ✗ coverage | WP-11 | J6 | AT-06 |

### Observed behaviour and comparison

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-40 | Observed representation computed from records, not hand-authored | M§4 | ● | RealityTrace | ✗ | WP-10, WP-11 | J6 | AT-06 |
| R-41 | Preserves occurrence identity, frequency, ordering, context, provenance, uncertainty; aggregates traceable to occurrences | M§4, S§9.1 | | RealityTrace | ✗ | WP-11 | J6 | AT-06 |
| R-42 | Compared with the versioned declared model through semantic identities and explicit mappings | M§4 | | RealityTrace + DomainForge | ✗ | WP-12 | J6 | AT-07 |
| R-43 | Five classes distinguished | M§4 | ● | RealityTrace | ✗ | WP-12 | J6 | AT-07 |
| R-44 | Each difference names evidence, context, affected elements and classification basis | M§4 | | RealityTrace | ◐ residual kind | WP-12 | `why` | AT-07 |
| R-45 | Changing inputs changes the observed model and comparison with no manual edit | M§4 | ● | RealityTrace | ✗ | WP-11, WP-12 | replay | AT-08, NT-03 |
| R-46 | Episode comparison and structural comparison never merged | S§9.1 | | RealityTrace | ✗ | WP-12 | J5 vs J6 | AT-07 |
| R-47 | Divergence is not treated as noncompliance; six diagnostic causes kept open until evidence attributes | S§1.3, §9.2 | | RealityTrace | ◐ diagnostic kinds | WP-13 | J6 | AT-12 |

### Governed revision and development

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-50 | Investigation concludes: local correction, maintain difference, observe further, or structural candidate | M§5 | | RealityTrace | ◐ | WP-13 | J6 | AT-12 |
| R-51 | Original divergence and diagnosis recorded | M§5.1 | | RealityTrace | ✓ kinds | WP-13 | J6 | AT-12 |
| R-52 | Versioned candidate `.sea` with stated changed distinctions | M§5.2-3 | | DomainForge | ◐ `ConceptChange` | WP-13 | J7 | AT-12 |
| R-53 | Retained invariants and affected capabilities identified; candidate and semantic diff validated | M§5.4-5 | | DomainForge | ✓ diff | WP-13 | J7 | AT-12 |
| R-54 | Held-out executions and retained-behaviour regression; failing regression blocks success | M§5.6 | ● | RealityTrace | ✗ | WP-13 | J7 | AT-12, NT-08 |
| R-55 | Independent authorization before promotion; denied or unknown cannot promote | M§5.7 | ● | SEA-Forge | ◐ gated world transitions | WP-13, WP-21 | J7 | NT-05 |
| R-56 | Previous version, migration and recovery path preserved; old evidence stays attached to its version | M§5.8, S§9.4 | ● | SEA-Forge / RealityTrace | ◐ registry in memory | WP-04, WP-13 | version switcher | NT-07 |
| R-57 | Subsequent bounded episode under the approved representation | M§5.9 | | all | ✗ | WP-51 | J7 | AT-13 |
| R-58 | The new representation makes a distinction or affordance available that was absent before | M§5 | ● | all | ✗ | WP-51 | J7 | AT-13 |
| R-59 | Unsuccessful and inconclusive experiments preserved | M§5 | | RealityTrace | ✓ append-only | WP-51 | timeline | AT-12 |
| R-60 | Undeclared occurrence stays distinguishable from an authorized model change | M§7 | ● | RealityTrace | ✗ | WP-12 | J6 | NT-04 |
| R-61 | Developmental memory states what the next observer can now do; asset is candidate until variation and transfer | S§9.5, §6 | | SEA-Forge / Horizon | ◐ lessons | WP-03, WP-31 | J10 | AT-13 |

### Horizon inspection and operator experience

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-62 | Operator can inspect workflow and pinned world; execution and evidence; declared-observed difference; why classified; history and provenance; candidate change and consequences; authorization, settlement, resulting version; what the next episode can do | M§6 | | Horizon | ◐ debugger ops, no sources | WP-30, WP-31 | J2, J5-J7 | AT-14 |
| R-63 | Horizon shows the same result as the programmatic path | M§7, S§11.7 | ● | Horizon | ✗ | WP-30 | all | AT-14, NT-09 |
| R-64 | Separation of world versions, checkpoints, hypotheticals, authority decisions, observed consequences, developmental claims | M§6 | | Horizon | ✓ value labels | WP-31 | rule 2-3 | AT-14 |
| R-65 | Explore world and source-grounded relationships; discover affordances and blocked possibilities; investigate evidence; compose, delegate, monitor, interrupt, resume; inspect plans, changes, costs, approvals, outcomes; decide at boundaries; compare candidates; preserve work | U | | Horizon + Workbench | ◐ | WP-31, WP-32 | J1-J10 | AT-19 |
| R-66 | One coherent environment across Horizon and Workbench | U | | Horizon + Workbench | ◐ connect on a branch | WP-06, WP-32 | seam | AT-19 |
| R-67 | Human-reserved decisions cannot be made by an agent, including one driving the operator's browser | S§3.2, Horizon critic 4 | ● | SEA-Forge / Cognate identity / Horizon | ✗ | WP-20 to WP-23 | J5, J7 | NT-12 |
| R-68 | Case continuation and receiver-specific handoff from source-owned records | S§10.1-10.4 | | SEA-Forge / Horizon | ◐ | WP-03, WP-31 | J9 | AT-15 |
| R-69 | Closure only by authorized disposition with economic account, liabilities, recoverability | S§10.3 | ● | SEA-Forge | ✗ | WP-03 | J9 | AT-10 |
| R-70 | Retrieved context is cited reference material, never canonical | S§3.4 | | Context Kernel | ✓ | WP-41 | label `untrusted_reference` | AT-17 |

### Verification, evidence and transfer

| ID | Requirement | Src | C | Owner | Today | Change | UX | Proof |
|---|---|---|---|---|---|---|---|---|
| R-71 | Nine negative proofs of M§7, each failing for the intended reason under mutation | M§7 | ● | all | ✗ | WP-52 | — | NT-01 to NT-09 |
| R-72 | Clean conforming control and deliberate fault injections | M§7 | | all | ✗ | WP-52 | — | AT-18 |
| R-73 | Fresh independent verifier confirms from frozen criteria and artifacts | M§7, S§3.5 | ● | independent | ✗ | WP-54 | — | AT-18 |
| R-74 | Evidence bundle: run manifest, semantic, operational, verification evidence | ME1-4 | | sea-rs | ◐ conventions | WP-53 | — | AT-18 |
| R-75 | Claim-to-evidence ledger over the eight claim areas with PROVEN / PARTIAL / PLANNED / NOT TESTED | ME5 | | sea-rs | ✗ | WP-53 | — | AT-18 |
| R-76 | One command (or short sequence) regenerates the proof | ME6 | | sea-rs | ◐ `e2e-world-loop.sh` | WP-53 | — | AT-18 |
| R-77 | Owner repositories' proof ledgers updated | ME7 | | each repo | — | WP-53 | — | AT-18 |
| R-78 | Acceptance tests and falsifiers registered before results exist | MX | ● | sea-rs | ✓ preregistration convention | WP-50 | — | AT-18 |
| R-79 | Templates and instrumentation transfer to a second bounded situation without rewriting the evidence mechanism | S§11.8 | | all | ✗ | WP-60 | J1 | AT-16 |
| R-80 | Gauntlet / SWE_SEED verification against frozen references feeds evidence | S§3.4, §7.3 | | Gauntlet / SWE_SEED | ◐ E7 path; Gauntlet unlinked | WP-40 | J5 | AT-17 |
| R-81 | Integrated release gate; installable from release artifacts; skipped gate is NO-GO | completion mandate | ● | godspeed-readiness | ◐ `local_operator` | WP-70 | — | AT-20, NT-14 |

Total: 76 rows, 31 critical.

## 3. Coverage check

- Every `Mission.md` task section 1–7 and evidence-contract item 1–5 maps to at least one row (R-01 to R-08,
  R-31 to R-64, R-71 to R-78).
- Every numbered SOP section 1–12 maps to at least one row; §13 (conflict table) is non-normative.
- Every operator capability in the brief maps to R-65 and a journey.
- Requirements with no existing implementation at all (✗ today): 27. These are the new construction, concentrated
  in Casework field-case records (WP-03) and RealityTrace observed/compare (WP-10 to WP-13).
