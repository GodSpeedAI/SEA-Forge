# B — Gap and convergence ledger

One row per gap. Disposition vocabulary: `KEEP | REUSE | EXTEND | EXTRACT | CONSOLIDATE | REPLACE | DELETE | NEW`.
Every `NEW` row states why the other seven were insufficient.

## B1. Execution and recursion

### G1 — No recursive-execution contract anywhere in the ecosystem
* **Current**: absent. Searches over `cognate/packages`, `cognate/profiles`, `cep`, `sxr`
  return no RLM/recursion contract. Gauntlet's recursion is application-local
  (`gauntlet/crates/gauntlet-ports/src/invocation_graph.rs:140-166`).
* **Required**: one provider-independent contract that Cognate, Workbench, Horizon and
  Gauntlet can consume without naming a vendor.
* **Reusable**: Cognate `startRun`/`getRun`/`cancelRun`, continuations, `RunView`, projections.
* **Disposition**: **NEW** (contract types only, in the existing `runtime-api` package, not a
  new package). Justification: no existing port describes provider selection, invocation
  correlation, terminal status or result provenance. REUSE was tried first — `subagent.start`
  (`src/capabilities/subagentStart.ts:43-57`) has no provider, status, cancellation or result
  contract, only fire-and-forget startup; `ProcessPort` is buffered and cannot host a live
  harness (`execution/src/types.ts:64-66`).
* **Files affected**: `cognate/packages/runtime-api/src/rlm.ts` (new), `types.ts` (export),
  `cognate/packages/runtime-bun/src/rlm.ts` (new component + registry),
  `cognate/packages/runtime-bun/src/index.ts`.
* **Removable**: none.
* **Risk**: a new public capability set invites a second scheduler. Mitigation: the contract
  returns run ids and never owns state; `C` shows one run model.
* **Verification**: unit tests that `rlm.invoke` starts exactly one run; that `rlm.inspect`
  is terminal-stable; that `rlm.cancel` is idempotent.

### G2 — No Cognate capability can hold a long-lived stdio process
* **Current**: `ProcessPort.exec` returns after exit (`execution/src/types.ts:64-66`); the only
  streaming implementation is Workbench-local, on an unmerged branch
  (`cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/capabilities/execSpawn.ts:1-20`,
  `src/mcp/client.ts` `StdioChannel`).
* **Required**: spawn, write, read lines, close and kill-process-group behind a port a provider
  adapter can use without importing Workbench.
* **Reusable**: the wrapper's policy layer (containment matrix `worldMatrix`, `decideContainment`,
  posture record, intent/spawned/exited ledger, long-lease slot) and the app-owned stores it
  needs (`aggregateContainment.ts`, `executionPosture.ts`, `authorizeCapabilityOp`).
* **Disposition**: **EXTRACT the interface, not the implementation.** `runtime-api` declares a
  `StdioSpawnPort`; the host application implements it. Workbench's wrapper and its six injected
  dependencies (`getPosture`, `ledger`, `recordRefusal`, `slots`, `localHostSpawnAllowed`,
  `beforeIntent`) stay where they are. Moving them into Cognate would mean inventing four stores
  and an authorizer there — that is what revision 1 wrongly called "extraction".
* **Files affected**: `cognate/packages/runtime-api/src/types.ts` (port declaration);
  `cognate-workbench@feat/capability-runtime-t4a:src/capabilities/execSpawn.ts` (implements it);
  `cognate-workbench@feat/capability-runtime-t4a:src/mcp/client.ts` (second consumer).
* **Removable**: the ad-hoc stdio plumbing inside `mcp/client.ts`.
* **Risk**: the port is only useful where the host implements it; a standalone Cognate host has
  no providers. That is stated in `rlm.providers` rather than hidden.
* **Verification**: the containment-matrix tests unchanged; a contained posture refuses
  (`posture-contained`); a non-local world refuses `stream-port-unavailable`; win32 refuses
  `no-process-groups`; a spawn intent without a terminal record is reported unresolved on boot.

### G3 — Two independent Prime integrations already exist
* **Current**: Rust `PrimeAgentRunner` (`gauntlet/.../runner.rs:639`) and TypeScript
  `RealPrimeHost` over `Bun.spawn` (`gauntlet/workbench/src/real-prime-host.ts:1-40`), the
  latter with its own recursion policy (`prime-recursion.ts:1-30`).
* **Required**: one protocol implementation the Cognate adapter can reuse without adopting
  Gauntlet's controller.
* **Reusable**: the TS host's protocol knowledge (`{type:"session"}`,
  `{type:"message_end"}`, result contract after exit) and spawn-path validation.
* **Disposition**: **EXTRACT the protocol and the process driver** into
  `@godspeedai/cognate-rlm-prime` (new optional package) and **CONSOLIDATE** Gauntlet's workbench
  onto it. Scope correction from the resolve pass: the import closure of the candidate files reaches
  `./focus`, `./horizon`, `./affordance`, `./operation-gateway`, `./prime-topology`, so only
  `prime-host` and `real-prime-host` plus the `terminal-session` validation functions cross the
  boundary. Gauntlet keeps its policy, topology and envelope modules. The Rust adapter stays
  untouched and Gauntlet-internal.
* **Files affected**: `gauntlet/workbench/src/{prime-host,real-prime-host}.ts` +
  `terminal-session` validation; new `cognate/packages/rlm-prime/*`.
* **Removable**: Gauntlet's duplicate TS copy after it imports the package.
* **Risk**: cross-repo TS dependency and release ordering. Fallback: the package is still the
  single new implementation and Gauntlet's copy is marked deprecated.
* **Verification**: one protocol fixture suite run against the extracted package and against
  Gauntlet's adapter; no `prime` token in `runtime-api` or capability ids.


### G4 — An Expert Specialist cannot recurse at all
* **Current**: `subagent` is removed from Expert tool eligibility
  (`src/agents/agentCoding.ts:374-377`); only a root Orchestrator may `team.delegate`
  (`src/experts/service.ts:424-430`); depth ≤ Team `maxDepth` (default 2).
* **Required**: a Specialist may investigate/decompose inside its own Assignment without
  becoming an Orchestrator and without a new identity.
* **Reusable**: the `agent.expert` loop, the Assignment snapshot, the continuation wait.
* **Disposition**: **EXTEND** `ExpertExecution` with an optional `runner`
  (`{kind:"none"} | {kind:"provider", providerId}`); the Expert loop delegates to `rlm.invoke`
  only when a provider runner is frozen in. `team_delegate` semantics do not change.
* **Files affected**: `src/experts/types.ts:58-68`, `src/experts/service.ts:181-216`,
  `src/agents/agentCoding.ts` (expert branch), `src/capabilities/expertModel.ts`.
* **Removable**: nothing.
* **Risk**: two ways to spawn work inside one Expert run. Mitigation: one decision point
  (`runner:none` ⇒ the existing loop) plus a negative test that both cannot run.
* **Verification**: journeys 3 and 4; a test that a Specialist with an external runner still
  cannot create sibling Assignments.

### G5 — Three different things are called "Assignment"
* **Current**: Workbench `Assignment` (expert delegation, `src/experts/types.ts:69-83`);
  Cognate runtime `Assignment` bound to `caseContext`
  (`cognate/packages/runtime-api/src/assignment.ts`, `types.ts:264`); Gauntlet
  `RoleInvocation`/`InvocationGraph`. Horizon uses `assign:<caseId>:<stepId>` ids.
* **Required**: one explicit mapping so work cannot be attributed to the wrong case, agent or
  attempt.
* **Reusable**: Cognate's `AssignmentService` views (`types.ts:327-358`), WP-23's binding.
* **Disposition**: **CONSOLIDATE** by naming: Workbench's record stays `Assignment` (agent
  delegation); Cognate's is referred to as `CaseAssignment`; the link field on the Workbench
  Assignment is `caseAssignmentId`. No wire ids are made interchangeable.
* **Files affected**: `src/experts/types.ts`, `src/experts/service.ts`,
  `src/projections/experts.ts`, Workbench UI labels,
  `cognate/packages/runtime-api/src/assignment.ts` (doc/alias only).
* **Removable**: the ambiguity itself.
* **Risk**: renaming touches many strings; one mechanical commit, no behaviour change.
* **Verification**: a type-level test that a Workbench Assignment is not accepted where a
  CaseAssignment is expected; journey 8 asserts both ids in evidence.

### G6 — Provider-internal children have no authority bound
* **Current**: the Prime adapter registers an MCP bridge with a fixed caller and a depth
  binding (`gauntlet/.../mcp_bridge.rs:1-30`), but no tool allowlist is visible in the adapter
  and the spawned harness keeps its own builtin tools. Depth is reported, not enforced by the
  adapter (`runner.rs:1001-1014`); enforcement lives in Gauntlet's app layer.
* **Required**: a recursive child cannot exceed its parent's grant, and the system can say
  truthfully whether it enforced that.
* **Reusable**: the bridge pattern; Workbench's authority intersection; containment posture.
* **Disposition**: **EXTEND** the adapter contract with an attested `toolPolicy`
  (`"bridge-only" | "contained" | "unattested"`). `unattested` is refused under governance and
  under any run whose authority is narrower than `full-access`.
* **Files affected**: `cognate/packages/runtime-api/src/rlm.ts` (capability facts), adapter.
* **Removable**: nothing.
* **Risk (resolved)**: Prime 0.10.2 does **not** offer a way to disable builtin tools (no CLI
  flags, no settings key, zero binary string hits — G1 row 1). `bridge-only` is therefore
  unattainable for it, the adapter declares `unattested`, and the governed path refuses — the
  designed outcome, not a workaround.
* **Verification**: journey 5; a test that an adapter declaring `unattested` is refused with a
  typed code.


## B2. Identity, authority and settlement

### G7 — External harnesses are refused wholesale under governance
* **Current**: one refusal covers the whole harness family
  (`src/capabilities/harness.ts:89`), keyed on `Boolean(externalAuthority)`
  (`src/server.ts:785`).
* **Required**: the refusal must be about *unintercepted effects*, not about "external".
* **Reusable**: the refusal text and its governance wiring; the bridge pattern from G6.
* **Disposition**: **EXTEND** with port governance (D8). Harness sessions stay refused (their
  effects are not intercepted and they cannot be given a declared reach). An RLM provider is
  admitted under governance through a **contained streaming port**: a filesystem mode mapped from
  the parent's grant, no network except one bind-mounted egress socket, a process group, and an
  fs-diff record. Verified feasible with bubblewrap 0.13.1 (D8.3). Without that world, the provider
  is refused `rlm-port-unavailable` — the old behaviour, now named precisely.
* **Files affected**: `src/capabilities/harness.ts:57-89`, `src/server.ts:785`.
* **Removable**: the implicit assumption that all external processes are equivalent.
* **Risk**: an adapter could lie about `toolPolicy`. Mitigation: the declaration is recorded as an
  operator-visible capability fact with the executable's version and hash, and the provider id is
  operator-authorized. Attribution, not proof. The pre-existing self-approval hazard on
  `uncontained-local` hosts (`src/server.ts:508-511`) is unchanged by this design and is raised as
  owner decision O2.
* **Verification**: journeys 12 and 13.

### G8 — No Workbench→Gauntlet delegation seam
* **Current**: none. `gauntlet run` needs a project with `.wayfinder/reference.yaml`,
  `.wayfinder/answer-key.md` and `.gauntlet/reference.lock`; `fabric serve` is a bounded
  agent-delegation surface that does not freeze, verify or settle
  (`gauntlet/crates/gauntlet-cli/src/fabric_cmd.rs:19-21,77-78`; `run_cmd.rs:1-11`) ‡.
* **Required**: an explicit, inspectable escalation with a frozen reference and no second
  settlement authority.
* **Reusable**: `criterion.freeze` + `world.freeze` for frozen criteria; governed spawn (G2);
  Gauntlet's report JSON (`run_cmd.rs:430-442,464-476`).
* **Disposition**: **EXTEND** Workbench with one capability `case.escalate` that projects a
  frozen Assignment into a Gauntlet reference project and spawns `gauntlet run`.
* **Files affected**: new `src/capabilities/escalate.ts`; `src/cognitive/criterion.ts`
  (projection); `src/settlement/` (record only; no settle path).
* **Removable**: nothing (the seam is new but small).
* **Risk**: Gauntlet's reference format changes. Mitigation: the projector pins the Gauntlet
  version string and refuses on mismatch.
* **Verification**: journey 7; an escalation cannot mark a case settled.

### G9 — Workbench has no CEP world identity
* **Current**: local `worldHash` is a hash of Workbench's own manifest
  (`src/capabilities/cognitive.ts:161-164`); no CEP profile, no `world_ref` anywhere in
  Workbench `src/`. Horizon's handoff carries neither (`Horizon/src/capabilities/handoff.ts:77-86`).
* **Required**: a run that addresses a semantic world must carry `world_ref` + semantic
  revision, and must refuse when it claims to be evaluated without one.
* **Reusable**: Cognate `CaseContext` already has exactly those fields
  (`cognate/packages/runtime-api/src/types.ts:14-38`); CEP `scope.world_ref` grammar ‡.
* **Disposition**: **EXTEND** `expert.converse`/`team.converse` with optional `caseContext`,
  pass it to `startRun`, and record the mapping `localWorldHash ↔ worldRef` in the Assignment's
  evidence. No new envelope. The context carries `worldCorroboration: "verified" | "asserted"`: a
  registry check is performed only when DomainForge/SEA-Forge is configured, and its absence is
  recorded rather than implied.
* **Files affected**: `src/experts/service.ts:400-423`, `src/experts/types.ts:58-68`,
  `src/capabilities/connectHorizon.ts`.
* **Removable**: nothing.
* **Risk**: a local-only case has no CEP world. Rule: local cases keep `worldHash` and are
  marked `scope:"local"`; they must never be presented to SEA-Forge as CEP worlds.
* **Verification**: journey 8; an evaluated case without `worldRef` is refused.

### G10 — Horizon's handoff is authority-free
* **Current**: `handoffItem` carries five keys; `settlement_criteria` is always empty; no
  expectation, uncertainty or citation travels (`Horizon/src/capabilities/handoff.ts:77-86`,
  `:133-161`).
* **Required**: preserve case identity, frozen criteria, source world, uncertainty and
  evidence references across the handoff.
* **Reusable**: the existing item shape and case-keeper verbs; CEP
  `godspeed.context_bundle` / `semantic_snapshot` fields ‡.
* **Disposition**: **EXTEND** the item with `world_ref`, `semantic_refs[]`, `acceptance[]`,
  `uncertainties[]`, and `cep: {profile, version}` — fields that already exist in the CEP
  profiles, not a new schema.
* **Files affected**: `Horizon/src/capabilities/handoff.ts`,
  `Horizon/src/adapters/sea-forge-case.ts`.
* **Removable**: the empty `settlement_criteria` default.
* **Risk**: the case keeper may reject unknown item fields. Mitigation: the keeper's preflight
  is the gate; send only after a successful `case_preflight` dry run.
* **Verification**: journey 8 and journey 14.

### G11 — Horizon's identity stops at a bearer key
* **Current**: Horizon presents only `authorization: Bearer <session key>`; its `Caller` is
  discarded by the token supplier (`Horizon/src/adapters/workshop.ts:68`). No
  operator→Workbench-actor map exists.
* **Required**: the actor an operation is attributed to must be the operator or agent that
  actually requested it.
* **Reusable**: Workbench's session/identity ledger (`src/auth/identity.ts:177`), its
  `identity.session.issue` capability (`src/server.ts:268`), and `identity.inspect`.
* **Disposition**: **EXTEND** `expert.converse` with an optional `assertedBy` that is *recorded*,
  never used as a refusal. Horizon always presents the operator's own session
  (`src/capabilities/connectHorizon.ts:41-49`; `src/auth/identity.ts:188-196`), so a mismatch test
  would be unreachable. The run records the session actor as `onBehalfOf` and `assertedBy` as an
  unverified claim. No second auth mechanism, and no implied operator mapping.
* **Files affected**: `src/experts/service.ts:400-423`, `src/auth/identity.ts`.
* **Removable**: nothing.
* **Risk**: a bearer key remains a bearer key. The guarantee is attribution, not proof.
* **Verification**: journey 8; `assertedBy` is recorded verbatim and never used as a refusal;
  attribution still comes from the session actor.


## B3. Epistemics, recovery and packaging

### G12 — Local acceptance is indistinguishable downstream
* **Current**: `world.settled` (local, human promotion) and `workbench.settlement.recorded`
  (SEA-Forge) are different event types, correctly separated
  (`src/capabilities/cognitive.ts:267`; `src/settlement/orchestrator.ts:160-164`). The downstream
  defect is the opposite of "indistinguishable": `OBSERVED_EVENT_TYPES` contains `world.settled`
  but **not** `workbench.settlement.recorded` (`src/observations.ts:28-35`), so a governed
  settlement never reaches RealityTrace at all, while a local promotion does.
* **Required**: a downstream reader must tell local acceptance from governed settlement
  without knowing Workbench's internals.
* **Reusable**: the existing event payloads and projections.
* **Disposition**: **EXTEND** both payloads with `settlementScope: "local" | "governed"` and
  `receiptRef: null` for local; **EXTEND** the observation fold to include the governed family, and
  record the cost (one SXR ingest subprocess per settled operation, `src/observations.ts:26-27`).
* **Files affected**: `src/capabilities/cognitive.ts:253-269`,
  `src/settlement/orchestrator.ts:160-164`, `src/observations.ts:28-37`.
* **Removable**: nothing.
* **Risk**: stored events lack the field; readers treat absence as `local` (conservative) and
  that upgrade rule must be written down.
* **Verification**: journey 9; a governed settlement always has a non-null `receiptRef`.

### G13 — The built-in Expert baseline is not recoverable
* **Current**: `seedExperts` returns when `seed:v1` exists (`src/experts/state.ts:57-59`); no
  capability clears the receipt; deleting `cognate-team` is permanent.
* **Required**: an operator can restore the five Experts and the default Team without losing
  their customized copies.
* **Reusable**: the versioned `mutate` path, receipts/idempotency, the `experts.seed` surface.
* **Disposition**: **EXTEND** with an operator-only `experts.restore`: create absent baseline ids;
  with `includeArchived:true` reactivate archived baseline ids (an explicit reversal of an
  operator choice, never automatic); never modify an active baseline id or any custom id.
  **DELETE** the permanent lock: the boot seed becomes idempotent per id rather than per receipt.
* **Files affected**: `src/experts/state.ts:57-76`, `src/experts/service.ts:345`.
* **Removable**: the `seed:v1` permanent short-circuit.
* **Risk**: two seeds racing at boot; the versioned mutation already serializes them.
* **Verification**: journey 12; restore leaves a renamed custom Expert untouched.

### G14 — CognitiveOS move selection is not reachable as a capability

*Owner: increment I5.*
* **Current**: `resolveAffordances` is pure and tested (`src/cognitive/affordance.ts:51`;
  `test/cognitive-semantics.test.ts:238-252`), consumed by `CaseProjection`
  (`src/cognitive/case.ts:92`). The operation catalogue is supplied by the caller; nothing
  defines the *kinds* of move (clarify, investigate, model, prototype, specify, implement,
  verify, settle, escalate).
* **Required**: the system can choose an appropriate next move instead of always treating a
  request as an implementation task, with inspectable criteria and no learned score.
* **Reusable**: the resolver, the Case projection, `method.*`, `experiment.*`.
* **Disposition**: **EXTEND**: a data-defined `OPERATIONS` catalogue consumed by `case.derive`,
  plus one read capability `affordance.field`. Data plus one read, not a selector service.
* **Files affected**: new `src/cognitive/operations.ts`, `src/capabilities/cognitive.ts:80`
  (one more id), `src/cognitive/case.ts`.
* **Removable**: nothing.
* **Risk**: catalogue drift. Mitigation: every operation names the capability it invokes, and a
  test asserts each named capability exists.
* **Verification**: journey 1 and the "investigate instead of implement" negative test.

### G15 — Installed Cognate versions do not match the manifests
* **Current**: Workbench pins 0.9.2, Horizon 0.10.0, installed tree 0.8.0 in both
  (`A5`). `CaseContext` is absent from the installed 0.8.0 source.
* **Required**: the feature set this design relies on must actually resolve at runtime.
* **Reusable**: the release pipeline already tags components before the release PR.
* **Disposition**: **CONSOLIDATE** in increment I0: one released Cognate version consumed by
  both apps, installed, and asserted by a test that prints the resolved version.
* **Files affected**: `cognate-workbench/package.json`, `cognate-workbench/bun.lock`,
  `Horizon/package.json`, `Horizon/bun.lock`.
* **Removable**: the stale installs.
* **Risk**: 0.10.0 may break Workbench. Mitigation: I0 is separate and reversible, gated by the
  existing suites.
* **Verification**: `bun install` + `bun test` green in both repos; a runtime assertion that
### G16 — Duplicate extension registries and duplicate subagent paths

*Owner: increment I5. **Status changed by the resolve pass: KEEP, not DELETE.***
* **Current**: `src/components/pluginRegistry.ts` is live, with four consumers and a test:
  `src/server.ts:111,638,806` (imported, constructed, injected into `pluginInstallComponent`),
  `src/capabilities/pluginInstall.ts:14`, `src/workbench/bundleSources.ts:22`, and
  `test/workbench-plugins.test.ts:5`. Two subagent starters exist: app-local `subagentStart.ts`
  and `@godspeedai/cognate-profile-harness`'s `spawnSubagent`
  (`cognate/profiles/harness/src/index.ts:391-395`).
* **Required**: one way to register an extension, one way to start a child run.
* **Reusable**: the bundle store and the app-local starter.
* **Disposition**: **KEEP** `pluginRegistry.ts`. A live registry with a capability mounted on it is
  not legacy by self-description. Its successor is a port of `PluginSource` and
  `verifyImplementation` into the bundle store; only after `pluginInstall` moves does deletion
  become possible, with its test moved too. **KEEP** the app-local `subagent.start` and **DELETE**
  the unused `cognate-profile-harness` dependency if nothing imports it.
* **Files affected**: `src/plugins.ts`, `package.json` (for the dependency question only).
* **Removable**: the unused dependency, if the inventory says so.
* **Risk**: the bundle store may not cover every consumer; the port is the prerequisite, not the
  inventory alone.
* **Verification**: the full Workbench suite passes; `pluginInstall` still resolves its registry.

### G17 — Containment is Linux-only and `run_code` is not a sandbox
* **Current**: bubblewrap is the only enforcement (`src/executionPosture.ts:1-35`; verifier
  launcher flags in `src/settlement/launcher.ts:38-46`); `run_code` is `bun -e` in the same
  world as `bash` (`src/capabilities/tools.ts:208-227`); macOS/Windows get a posture refusal,
  not isolation.
* **Required**: recursive execution refuses where containment is unavailable and never implies
  stronger isolation than exists.
* **Reusable**: the posture guard, aggregate quotas, the T3/T4a per-kind matrix.
* **Disposition**: **KEEP** the posture refusal for uncontained effects; **EXTEND** the provider
  contract with the honest per-restriction matrix (D8) and add a contained streaming world (I3b) so
  the matrix has a world that can actually enforce filesystem and network reach.
* **Files affected**: `cognate/packages/runtime-api/src/rlm.ts`,
  `cognate-workbench@feat/capability-runtime-t4a (bbe6c73):src/capabilities/execSpawn.ts` (matrix source).
* **Removable**: UI copy implying `run_code` is sandboxed.
* **Risk**: operators on any platform may lose the external-provider path, because the refusal is
  posture-driven rather than OS-driven; the ordinary Expert loop still works everywhere.
* **Verification**: journey 13.

### G18 — Cognate has no ambiguity story for external invocation
* **Current**: Gauntlet prescribes it (`agent_runner.rs:46-48`) and Workbench has
  `reconciliation-required` for tool effects (`expertEffect.ts:1-45`), but Cognate has no
  generic "external invocation outcome unknown" state.
* **Required**: a crash or lost transport must not cause a blind retry of a consequential
  external call.
* **Reusable**: `AssignmentStatus` already has `reconciliation-required`
  (`src/experts/types.ts:11`); continuations model durable waits.
* **Disposition**: **REUSE** `reconciliation-required` and the existing ambiguous-invocation
  machinery (`run.invocation.ambiguous` + `ambiguous-invocation` continuation +
  `run.invocation.reconcile`, `src/capabilities/ambiguousStep.ts:6-9`), and **EXTEND** the RLM
  status with `unknown` (never `failed`) for lost transport. Revision 2 removed the invented
  `rlm.reconcile` capability: it duplicated a mechanism Cognate already has.
* **Files affected**: `cognate/packages/runtime-api/src/rlm.ts`, `runtime-bun/src/rlm.ts`;
  `src/experts/bridge.ts` (map `unknown` to reconciliation).
* **Removable**: nothing.
* **Risk**: an operator may reconcile wrongly; the evidence ref keeps the claim reviewable.
* **Verification**: journey 11.

## B4. Disposition summary and net footprint

| Disposition | Items |
|---|---|
| **KEEP** | Team/assignment invariants, authority intersection, billing reservations, effect boundary, continuations, local case lifecycle, posture refusal, `expert.converse`, session key transport, `subagent.start` |
| **REUSE** | `startRun`/`getRun`/`cancelRun`, `reconciliation-required`, CEP profiles (7 × 1.0.0), `CaseContext`, WP-23 binding, Gauntlet report JSON, `resolveAffordances`, `method.*` |
| **EXTEND** | `ExpertExecution.runner`; `expert.converse` `caseContext`/`assertedBy`; harness refusal condition; settlement payload scope field; `experts.restore`; `case.escalate`; handoff item fields; RLM status `unknown`; one `affordance.field` capability |
| **EXTRACT** | `StdioSpawnPort` interface (implementation stays in the host app); Gauntlet TS Prime host → `@godspeedai/cognate-rlm-prime` |
| **CONSOLIDATE** | Assignment naming; Cognate versions; Prime protocol (one TS implementation) |
| **REPLACE** | nothing (no existing mechanism is bad enough to justify replacement) |
| **DELETE** | `seed:v1` permanent lock; duplicate Prime TS protocol/driver (I4); ad-hoc stdio plumbing in `mcp/client.ts`; unused `cognate-profile-harness` dependency if unused. **Not** `pluginRegistry.ts` — four live consumers (G1 row 3) |
| **NEW** | `rlm.*` contract module inside `runtime-api` (types only); `agent.rlm` + registry in `runtime-bun`; `cognate-rlm-prime` optional package |

**Expected net footprint.** Added: ~1 contract module, 1 runtime component, 1 optional adapter
package, 1 Workbench capability (`case.escalate`), 1 restore command, 1 operations data file.
Removed: 1 legacy registry and consumers, 1 duplicate Prime host, 1 app-local spawn wrapper,
1 permanent seed lock, stale dependency installs. Line count is likely slightly positive in
Cognate and negative in Workbench and Gauntlet. The claim that matters is not the line count:
four mechanisms stop having two implementations.

**Every `NEW` justified in one line each.**
* `rlm.*` types: no existing port expresses provider choice, terminal status, cancellation and
  result provenance (G1). Placed in an existing package, not a new one.
* RLM component: the capabilities need a mount point; the only alternative is Workbench-local
  code, which G4 shows is the wrong owner.
* `cognate-rlm-prime`: the protocol must live *somewhere* outside canonical semantics; a
  separate optional package is the only placement that keeps Prime out of `runtime-api` and
  out of Workbench's dependency graph.

  `runtime-api` exports `CaseContext`.
