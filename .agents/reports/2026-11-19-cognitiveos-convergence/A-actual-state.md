# A — Actual-state architecture (verified)

Baselines: see [`README.md`](README.md). Citations marked **‡** were taken from a recon report
and not re-read line-by-line by the design author.

## A1. Who owns what today

| Product | Owns | Does not own |
|---|---|---|
| **Cognate** | Kernel + capability contracts, event log, runs, continuations, shared state, projections, execution worlds, providers, identity principals, governed operations, Connect protocol | Case semantics, settlement authority, agent-identity policy, recursive execution |
| **Cognate Workbench** | Experts/Teams/Assignments, local case+world lineage, verification, local acceptance, capability bundles, SEA-Forge governance client, governed settlement for `agent.coding` | Authority decisions, external settlement, cross-agent transport, RLM |
| **Horizon** | Case navigation, operator questions, decisions, plan items, expected-vs-observed desk | Execution of any kind; it owns `run.start`/`run.cancel` authorization only |
| **Gauntlet** | Frozen reference, controlled execution loop, independent verification, recovery, settlement decision for its own invocations | SEA-Forge authority, CEP emission, Workbench tasks |
| **DomainForge** | `.sea` semantic model, canonical serialization, CEP envelope emission (`domainforge envelope --emit cep`) | Worlds, cases, execution |
| **SEA-Forge (`sea-rs`)** | Authority requests/decisions, world registry, evidence acceptance, settlement, containment | Agent orchestration, workbench UX |
| **RealityTrace (`sxr`)** | Observation ledger, hash-chained evidence, expected-vs-observed verification | Authority, settlement |

## A2. The five execution paths that exist

**1. Ordinary coding session.** `agent.coding` (`src/agents/agentCoding.ts`) with tools
`read/edit/bash/run_code/subagent/capability_invoke/skill_load` (`:38-39`). A `subagent` call
starts a fire-and-forget child run through `subagent.start`
(`src/capabilities/subagentStart.ts:43-57`), correlated `subagent:<parentRunId>:<id>` and
running under the parent's agent principal (never a wider one).

**2. Expert solo run / Team run.** `expert.converse` / `team.converse` create an Assignment
(`src/experts/service.ts:400-423`) and a queued durable record. `ExpertCompletionBridge`
dispatches it as a run `agent.expert` under principal `wb-agent:<expertId>`, idempotency
`expert-run:<assignment>:<attempt>` (`src/experts/bridge.ts:43-50`). Completion lands in
`reconciliation-required`, and the operator (or review) settles it
(`src/experts/bridge.ts:86`).

**3. Orchestrator → Specialist delegation.** `team.delegate` is accepted only from an
authorized root Orchestrator in a shared Team conversation
(`src/experts/service.ts:424-430`). The child Assignment's authority is the intersection of
Expert tools, the requested set, Team membership and Team ceiling, with `team_delegate`
removed below the orchestrator (`:195-199`). Depth is bounded by the Team's `maxDepth`
(`:190`). The parent suspends on `waitFor("expert-assignment:<id>")`
(`src/agents/agentCoding.ts:806-816`), and the bridge resolves the continuation once the child
is terminal (`src/experts/bridge.ts:93-103`).

**4. External harness session.** `session.harness.bind` / `harness.turn.advance` drive an
installed CLI through a host-process runner (`src/capabilities/harness.ts:151-182`). Local
worlds only; remote/contained/SSH/WSL refused (`:143`). **Under SEA-Forge governance the whole
family is refused**: "an external CLI's tool calls cannot be governed or settled"
(`:89`).

**5. Governed settlement for coding runs.** Only `tool.edit`, `tool.bash`, `tool.run_code` are
consequential (`src/settlement/orchestrator.ts:22`). A proof must be declared *before* the run
(`:100-109`); RealityTrace verifies under Workbench's bubblewrap launcher (`:129-137`); then
`seaForge.submitEvidence` + `seaForge.settle` (`:153-164`), and the recorded status is
SEA-Forge's.

## A3. What is implemented, partial, specified-but-unbuilt, and absent

### Implemented and tested

| Mechanism | Evidence |
|---|---|
| One Orchestrator per active Team, enforced in the mutation path | `src/experts/state.ts:39-47`; `assertTeamInvariant` called on every mutation `:162` |
| Five seeded Experts + `cognate-team`, PM as sole Orchestrator | `src/experts/state.ts:49-76` |
| Authority narrowing on delegation (intersection, no widening) | `src/experts/service.ts:195-199`, `:424-430` |

### Partially implemented

| Mechanism | What exists | What is missing |
|---|---|---|
| Case-anchored runs | Cognate `CaseContext` (`cognate/packages/runtime-api/src/types.ts:14-38`) and `StartRunRequest.caseContext`; WP-23 `startAssignedRun` binds idempotency/correlation to an assignment (`cognate-workbench@feat/wp23-assignment-binding (668f686):src/assignments/assignmentBinding.ts:38-42`) | Not merged to `main`; `expert.converse` does not accept or forward case context; Horizon never sends it |
| Cognitive move selection | Pure `resolveAffordances` + `CaseProjection` (`src/cognitive/case.ts:92`) | No capability exposes the resulting field; operations are supplied by the caller; no move catalogue covering investigate/model/escalate |
| Governed spawn of long-lived processes | `execSpawn.ts` + `mcp/client.ts` `StdioChannel` on `feat/capability-runtime-t4a` (`bbe6c73`) | Unmerged; app-local; the RLM adapter cannot use it from Cognate |
| Safe mode | Derived fold over fault events, operator-only clear (`cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/workbench/safeMode.ts:1-7`, `:40-62`) | Unmerged; excludes extensions only; restores no data |
| Local vs external settlement labels | Two event families with different producers (`cognitive.ts:267`; `orchestrator.ts:160-164`) | Both feed the same SXR observation stream (`src/observations.ts:28-37`) with no local/global tag |
| Capability bundles | Manifest + store + dispatcher on `main` (`capabilityRuntime.ts:145-146` refuses executable handlers); executables only on T3 | No governed route for a bundle to run a process until T3 merges |

### Specified but not implemented

* **RLM / recursive execution.** No contract, port, runner, or provider adapter exists in
  Cognate, CEP, DomainForge, SEA-Forge or RealityTrace. `rg -i 'rlm|recursive language model'`
  over `cognate/packages`, `cognate/profiles`, `cep`, `sxr` returns nothing relevant (the only
  `recursi*` hits are directory walkers and parser depth caps). Gauntlet is the sole exception,
  and its recursion lives in *its* application layer, not in a shared contract.
* **Provider-independent runner port in Cognate.** Cognate's `ProcessPort` is buffered
  (`cognate/packages/execution/src/types.ts:64-66`), so no existing port can carry a live
  stdin/stdout harness.
* **Restore of the built-in Expert baseline.** `seedExperts` returns immediately when
  `state.receipts["seed:v1"]` exists (`src/experts/state.ts:57-59`); no command clears it.
* **CEP identity in Workbench.** No `world_ref`, no profile, no semantic snapshot anywhere in
  Workbench `src/` (search recorded in `recon/workbench-capabilities-and-safemode.md` §11).
  Workbench's `worldHash` is a hash of its own manifest (`src/capabilities/cognitive.ts:161-164`).

### Absent by design (and must stay absent)

* A second authority engine in Workbench: settlement status comes from SEA-Forge
  (`src/settlement/orchestrator.ts:153-164`).
* A second scheduler: the Expert bridge is explicitly "not a second work scheduler"
  (`src/experts/bridge.ts:1-3`).
* A general-purpose MCP *client* in Cognate: `packages/mcp` is server-only ‡
  (`cognate/packages/mcp/src/index.ts:74-100`).

## A4. Cross-repo contracts that actually exist

| Contract | Producer → consumer | Verified shape |
|---|---|---|
| `godspeed.semantic_snapshot` etc., 7 profiles at `1.0.0` **plus one `world-transition.v1` record schema** | DomainForge → (CEP validators, SEA-Forge) | `cep/schemas/profiles/godspeed/*` (8 files, 7 are profiles); world pinned by `scope.world_ref` = `world:<name>@sha256:<64hex>` ‡ |
| CEP envelope emission | `domainforge envelope --emit cep --world-name <n>`; `build_cep_envelope` | `cep_version: "0.1.0"`, `envelope_version: "1.0.0"`; profile declared only when a world is supplied ‡ |
| `authority_request` / `authority_decision` | Cognate `SeaForgeAuthority` ⇄ SEA-Forge server | NDJSON over `<root>/server.sock` (0600, atomic rename); Windows named pipe ‡ |
| Settlement | SEA-Forge `cep.rs` → ledger `settlement_packet` | Keyed `cep-settle-final:<op>` / `cep-settle:<op>:<hash>` via `commit_typed_once` ‡ |
| Evidence acceptance | SEA-Forge `evidence_packet` validation | 11 ordered fail-closed checks; 7 `CORROBORATED_FIELDS` must match RealityTrace's own record ‡ |
| RealityTrace ledger | `sxr` CLI `--json`; `sxr:<ledger>:<ordinal>` refs consumed by `sea-forge-server/src/realitytrace_source.rs` ‡ | Hash-chained, idempotent append (`sxr-ledger/src/ledger.rs:285-301`) ‡ |
| Workbench → SEA-Forge settlement | `SeaForgeAuthority.submitEvidence/settle` | Status and `final` originate in SEA-Forge (`orchestrator.ts:9-10,153-164`) |
| Horizon → Workbench | Connect `RuntimeService.Invoke("expert.converse", …)` | Bearer session key only; Horizon's `Caller` is not presented (`workshop.ts:68,174`) |
| Horizon → SEA-Forge cases | NDJSON over Unix socket, verbs `case_preflight/commit/…` | actor + `on_behalf_of` in the envelope ‡ |

**No verified producer/consumer pair exists for a CEP envelope crossing into Workbench,
Gauntlet or Horizon's handoff item.** That is the single largest semantic gap in the ecosystem,
larger than the missing RLM.

| Durable parent/child assignment with continuation + boot reconcile | `src/agents/agentCoding.ts:806-816`; `src/experts/bridge.ts:13-104` |
| Review independence: accept needs an independent Expert's observed checks | `src/experts/service.ts:316-325` |
| Durable pre-dispatch billing; unknown price never zero; ceiling fails closed | `src/experts/billing.ts:1-9`, `:149-164` |
| Two-phase effect boundary with `reconciliation-required` | `src/capabilities/expertEffect.ts:1-45` |
| Local case lifecycle + human-only `world.settle` | `src/capabilities/cognitive.ts:80`, `:253-269` |
| Affordance resolution (`possibility != affordance`, blockers, successor) | `src/cognitive/affordance.ts:1-51`; consumed by `src/cognitive/case.ts:92`; tested `test/cognitive-semantics.test.ts:238-252` |

## A5. Installation and version reality (this bites first)

| Consumer | Manifest pin | Installed in `node_modules` |
|---|---|---|
| Cognate Workbench | `@godspeedai/cognate-*` **0.9.2** (`package.json:21-49`; `bun.lock:148`) | **0.8.0** for runtime-api, execution, capabilities, continuations, kernel-api, client |
| Horizon | `@godspeedai/cognate-*` **0.10.0** (`package.json`) | **0.8.0** for runtime-api, client, protocol-connect |
| Local Cognate checkout | **0.10.0** (`packages/runtime-api/package.json`) | n/a |

`CaseContext` and `AssignmentService` are present in the 0.10.0 source and **absent from the
installed 0.8.0 tree**. Whether 0.9.2 carries them is unverified: no tarball was fetched, and the
settling action is `bun install` followed by a source grep (G1 row 10). (`rg 'CaseContext|caseContext'` over the installed `src/*.ts` returns
nothing). Any design that assumes case-anchored runs therefore also assumes a dependency
refresh that has not happened. This is recorded as increment I0, not as a design assumption.

## A6. What previous architecture assumptions are now false

| Assumption | Status | Evidence |
|---|---|---|
| "Workbench has a governance toggle that disables harnesses" | True but narrower than stated: the refusal is in the harness component only, and it keys off the presence of an external authority object | `harness.ts:58,89`; `server.ts:785` (`governanceActive: Boolean(externalAuthority)`) |
| "Safe mode can restore the default Experts" | False on `main`; safe mode is unmerged and extension-scoped | `cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/workbench/safeMode.ts:1-7` |
| "Workbench already speaks CEP" | False: no profile, no `world_ref` in Workbench. Horizon's handoff carries none either | A3, A4 above |
| "Gauntlet delegates to Prime through one adapter" | False: two independent integrations (Rust adapter, TS host) | `gauntlet/crates/adapters/gauntlet-adapter-agent-prime/src/runner.rs:639`; `gauntlet/workbench/src/real-prime-host.ts:1-40` |
| "Gauntlet can be driven as a bounded single run over a small RPC" | Partly false: `fabric serve` is bounded but deliberately does **not** freeze, verify or settle; `gauntlet run` is the whole controller | `gauntlet/crates/gauntlet-cli/src/fabric_cmd.rs:19-21,77-78`; `run_cmd.rs:1-11` ‡ |
| "DomainForge and RealityTrace have a working CLI contract" | True, with a recorded version caveat: DomainForge pins CEP commit `ae9b73b` whose `spec/profiles/` path does not exist; compatibility entries include explicit `incompatible` pairs | `recon/cep-and-domainforge-contracts.md` §5, §7 ‡ |
| "The five Experts and one Team are the default and are stable" | True as seed data, false as recoverable data | `state.ts:57-76` |

## A7. Tests that already constrain the design

| Area | Test evidence |
|---|---|
| One-Orchestrator invariant, team lifecycle | workbench `test/` expert/team suites ‡ (`recon/workbench-capabilities-and-safemode.md` §10) |
| Local vs external settlement separation | `test/E2E-cognitive-console.test.ts:39` ("settlement reachable != settled") ‡ |
| Affordance semantics | `test/cognitive-semantics.test.ts:238-252` |
| Horizon key secrecy and adversarial workshop calls | `test/workshop-key-secrecy.test.ts:26-52`, `test/adversarial-workshop.test.ts` ‡ |
| SEA-Forge evidence corroboration | `sea-rs` `cep_settlement.rs:716-814`, plus `just proof` jq gates over real artifacts ‡ |
| Gauntlet reference integrity and recursion refusal | `crates/gauntlet-cli/tests/p1_reference_integrity.rs`; `agents_tests.rs:731` ‡ |
| Cognate governed operations | `runtime-bun/test/{governed-operation,sea-forge-authority,invoker,run-ownership}.test.ts` ‡ |
| Cognate continuations | `continuations/test/{lifecycle,commitments}.test.ts` ‡ |

These are the regression fences the new work must not weaken; [`F`](F-implementation-sequence.md)
names which of them each increment must still pass.

| Single capability authorization function (kernel-enforced) | `src/workbench/authoritySelection.ts:86-129` ‡ |
| Execution posture refuses uncontained effects by default | `src/executionPosture.ts:1-35` ‡ |
| SEA-Forge authority client (`submitEvidence`, `settle`, approvals) | `src/capabilities/governance.ts` ‡; `cognate/packages/runtime-bun/src/sea-forge-authority.ts:1-10` |
| Horizon → Workbench generic capability call | `Horizon/src/adapters/workshop.ts:174` |
| Session-key provisioning with 0600 enforcement; the key is never logged | `Horizon/src/adapters/workshop.ts:5-7`, `:24-29` |
