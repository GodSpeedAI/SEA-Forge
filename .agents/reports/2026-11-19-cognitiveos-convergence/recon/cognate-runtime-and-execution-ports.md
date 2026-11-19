# Cognate: runtime & execution ports (recon)

Repo `/home/sprime01/projects/cognate` @ `29dcd1e`. All paths relative to that root.

## 1. packages/execution — the execution-world port

- **Process port (one method)**: `exec(spec: ExecSpec, options?: ExecOptions): Promise<ExecResult>` — `packages/execution/src/types.ts:64-66`.
- **Filesystem port**: `resolve/stat/exists/readText/list/writeText` + optional `realPath`, `version` — `packages/execution/src/types.ts:48-62`. `resolve` is the containment boundary (`PathEscapeError`, `types.ts:97-103`).
- **Input/output**: `ExecSpec { argv, cwd, env?, timeoutMs? }` `types.ts:25-30`; `ExecResult { stdout, stderr, exitCode, timedOut }` `types.ts:32-37`; `ExecOptions { signal? }` `types.ts:39-42`.
- **World provider**: `worldId`, `kind`, `metadata`, `fileSystem`, `process`, `dispose()` — `types.ts:72-80`; secret-free `WorldDescriptor` `types.ts:83-87`.
- **Registry**: `register/unregister/has/get/list/dispose`, `get` throws `WorldUnavailableError` and never falls back — `packages/execution/src/registry.ts:6-17, 38-42`.
- **Worlds present**: `local` (`local.ts:101`), `memory` (`memory.ts:119`), `ssh` (`packages/execution-ssh/src/index.ts:243`), `wsl` (`packages/execution-wsl/src/index.ts:268`), contained/`linux-bubblewrap` (`packages/execution/src/bubblewrap.ts:120`, metadata `network:"denied"`). `WorldKind` is an open string (`types.ts:5-6`).
- **Streaming / stdin / cancellation**: no streaming (single buffered `ExecResult`), **no stdin field anywhere in `ExecSpec`**; cancellation is `AbortSignal` only (`types.ts:40-41`), plus `timeoutMs` → `timedOut`.
- Capability ids for the ports: `FILESYSTEM_CAPABILITIES`/`PROCESS_CAPABILITY = "process.exec"` `packages/execution/src/capabilities.ts:12-25`; `ExecInput` `capabilities.ts:54`.

## 2. Where external commands run; authority at the kernel boundary

- **No built-in `bash` tool exists**: `rg` for bash tool ids hits only test fixtures (`packages/execution/test/local.test.ts:17-19`).
- **Surface A — `program.run`**: `PROGRAM_CAPABILITY` `packages/execution/src/program.ts:23`; `programExecutorComponent` resolves the world then runs through `world.process` — `program.ts:211-215`; header: "capability policy and the runtime's governance decide whether it runs at all. Nothing here authorizes itself" `program.ts:5`.
- **Surface B — harness `sandbox.exec`**: `Bun.spawn(["bun","-e",code])` with scrubbed env/timeout — `profiles/harness/src/index.ts:134-148`.
- **Kernel boundary enforcement**: `Invoker.policy.authorize` → `governor.governCapability(request, …)` — `packages/runtime-bun/src/invoker.ts:51-72` (call at `:60`); `Governor.govern` — `packages/runtime-bun/src/governed-operation.ts:203-220`; assignment actions go through `core.governor.govern({kind:"action",…})` — `packages/runtime-bun/src/assignment.ts:109-112`.

## 3. packages/runtime-api — RuntimeService

`interface RuntimeService` `packages/runtime-api/src/types.ts:257-322`:
- `startRun` `:264`; `getRun` `:265`; `cancelRun` `:266`; `events` (AsyncIterable, follow+signal) `:271`; `listContinuations` `:272`; `resolveContinuation` `:273-276`; `invoke` `:277`; `getArtifact` `:278`; `readProjection` `:279`; `listRuns` `:291`; `updateState` `:292-295`; `readSharedState` `:297`; `updateSharedState` `:302-305`; `observe` `:315`; `publishRemoteOffer` `:317`; `listRemoteOffers` `:319`; `revokeRemoteOffer` `:321`.
- Companion `AssignmentService` `:344-358` (`getAssignment`/`interruptAssignment`/`resumeAssignment`/`recordSpend`); `CaseContext` `:14-34`; `WaitSpec` `:36-44`; `Caller` `:183-186`.

## 4. runtime-bun assignment.ts + executor.ts

- **assignment.ts**: `foldAssignment` `:36`; `lifecycleOf` `:81`; `lifecycleEvent` `:84`; `acceptLifecycleEvent` `:92`; `createAssignmentService` `:106`. Each mutating op authorizes first (`:109-112`), then commits idempotently with derived event ids `asg_<fingerprint>_<i>` and a replayed request is a no-op (`:120-137`). Key types: `Assignment`, `AssignmentView`, `AssignmentEventName`, `SpendEntry`, `AssignmentLifecycle`.
- **executor.ts**: deterministic replay execution `executeOnce(core, runId, ownership?)` `:29` (`ExecutionResult = "done"|"retry"` `:17`); fenced run-write lease `RunExecutionOwnership { ownerId, generation, ambiguousSteps, renew() }` `:22-27`; cancellation/shutdown via per-run `AbortController` `:30-38`. Process-local admission `ExecutionSlots` `packages/runtime-bun/src/execution-slots.ts:3-36` (not durable ownership).

## 5. packages/continuations — durable waits

- `ContinuationStatus` `"OPEN"|"RESUMED"|"CANCELLED"|"EXPIRED"|"FAILED"|"BLOCKED"` `packages/continuations/src/types.ts:4`; `ContinuationRecord` `:7-28`; `OpenContinuation { type, tenant, runId, scopeId, correlationId, condition, payload, handler{id,version}, expiresAt?, ownerComponentId? }` `:30-45`.
- Resolve semantics: `ContinuationAction { actor, idempotencyKey, expectedVersion? }` `:47-52`; `TransitionOutcome` = `transitioned|duplicate|already_terminal|denied|version_mismatch|not_found` `:54-62`; `ContinuationPolicy.authorize` for `"resume"|"cancel"` `:64-67`; `ResumeHandler.supports/handle` `:69-74`.
- `ContinuationService` `packages/continuations/src/continuations.ts:37`: `open` `:59`, `get` `:113`, `list` `:118`, `resume` `:128`, `cancel` `:176`, `fail` `:218`, `expireDue` `:256`, `dispatchResumes` `:304`, `unblock` `:346`, `redactForRun` `:391`. Durable external commitments: `CommitmentService` `commitments.ts:39`, `CommitmentState` `types.ts:76`.
- The runtime-facing wait is `WaitSpec` (`runtime-api/src/types.ts:36-44`), opened by `ctx.waitFor(...)`.

## 6. profiles/harness/src/index.ts

- Profile components `index.ts:15`: model-providers, tool-registry, agent-loop, sessions, storage, skills, scheduler, sandbox, subagents.
- Capabilities actually registered: `tool.<name>` (`toolComponent` `:45-57`), `tool.<name>` bundles via `skillComponent` `:59`, `model.generate` (`providerComponent` `:72`), `storage.kv` (`storageComponent` `:84`), `subagent.start` (`subagentStartComponent` `:118-133`), `sandbox.exec` (`sandboxComponent` `:134`).
- `spawnSubagent(ctx, id, agent, input)` `:391-395`: `ctx.invoke("subagent.start", { agent, input, key: `${ctx.runId}:${id}` })` then `ctx.waitFor(id, { type:"subagent", condition:{ kind:"run_completion", runId }, payload:{ agent } })`. Idempotency = parent runId + local id; **no `correlationId` is passed** to `startRun`. Child is started as `{actor:{id:"system",kind:"system"}}` in the invoking caller's tenant (`:127-129`). Settlement: `watchSubagentCompletions` `:215-266` (required, else the parent waits forever).

## 7. packages/mcp

- Server-only adapter: `createMcpServer(options)` `packages/mcp/src/index.ts:74-100` builds an SDK `McpServer` (`:2`) whose tools call `RuntimeService.invoke` as a fixed caller (`:19-22`, `:84-90`); `McpToolBinding { capability, name?, description, inputSchema }` `:7-15`.
- **No MCP client**: no `@modelcontextprotocol/sdk/client` import in the repo; `agent-discovery` only *parses* MCP server configs (`packages/agent-discovery/src/mcp.ts:11-66`). So Cognate **cannot currently host an external process as a tool provider** over MCP; the direction is inverse (capabilities exposed as MCP tools). Consumer-of-remote-tools today is the `RemoteCapabilityOffer` path (`runtime-api/src/types.ts:189-206, 317-321`).

## 8. agent-discovery + a2a exports (provider-discovery potential)

- `packages/agent-discovery/src/index.ts`: `AGENT_REGISTRY` `:49`, `findExecutables` `:85`, `inspectAgent` `:190`, `discoverAgents` `:213`, re-exports `discoverMcpServers, probeVersion, probeAllowed` `:17` (+ `parseCodexMcpTables` `mcp.ts:66`). These discover *local CLI agents and their MCP configs* — no capability contract, no TTL, no lease, so not a provider registry as-is.
- `packages/a2a/src/index.ts`: `A2AAdapterOptions` `:17`, `createA2AHandler` `:99` (inbound agent-card + JSON-RPC task serving). Outbound A2A client/provider discovery: none.

## 9. packages/identity

- Actor kinds: `Principal.kind = "user"|"agent"|"service"|"system"` `packages/kernel-api/src/types.ts:22-25`; `Caller = { actor, tenant }` `packages/runtime-api/src/types.ts:183-186`.
- Principals are built from durable operator records only: `ResolvedSession` `packages/identity/src/service.ts:93-97`; session verification produces `{ tenant, actor:{ id: operatorId, kind:"user" } }` `:284`. Enrolment/creation: `createIdentityService` `:130`, `enrolDevice`/paired enrolment `:289-295`; pairing links authorize passkey enrolment only (`packages/identity/src/pairing.ts:5-17`).

## 10. protocol-connect

- Yes: **`cognate.v1.RuntimeService.Invoke`** — `packages/proto/src/gen/cognate/v1/runtime_pb.ts:510`; router handler `packages/protocol-connect/src/index.ts:447-452`; remote client `:646-648`. Registered with the rest of the service at `:361-363`.

## 11. governed-operation.ts + governance-env.ts

- `GovernedOperation` union `packages/runtime-bun/src/governed-operation.ts:20-22`: `{kind:"action", action, caller, resource}` | `{kind:"capability", request, claims?}`.
- Decision order in `Governor.govern` `:203-220`: local `ActionPolicy`/`Policy` first (`:206`, `:222-225`); `mode==="off"` → allow (`:207`); system reads bypass the authority unless `governSystemReads` (`:208-210`, `:141-143`); then `authority.decide(...)`; a throwing authority yields `disposition:"unknown"`, `allow:false` (`:215-218`). Unrecognized disposition → refuse (`:228`); `constrained_allow|boundary|degraded` are allows only if every constraint has a registered enforcer that returns true, else refuse (`:237-259`).
- **No authority configured**: mode comes from env only — unset/`off` → `{mode:"off"}` (local policy alone), stray `COGNATE_SEA_FORGE_SOCKET`/`COGNATE_WORLD_REF`/`COGNATE_SEA_FORGE_ACTOR` with mode not `sea-forge` throws, `sea-forge` without socket or `world:<name>@sha256:<hex>` throws (`packages/runtime-bun/src/governance-env.ts:16-33`). So: never a silent downgrade, but the *default is local-policy-only*, not fail-closed; fail-closed applies per-operation once `sea-forge` is selected.

## 12. recursive-context / child-context / provider-probing

`rg -n "recursi|child context|probe" packages profiles` → **no recursive-context and no child-context code exists**. Probe hits are: `VersionProbe`/`probeAllowed`/`probeVersion` (`packages/agent-discovery/src/probes.ts:8, 36`), a `probe` *store handle* in Postgres test scaffolding (`packages/store-postgres/test/conformance.test.ts:12-14`, `packages/runtime-bun/test/run-ownership.test.ts:17-19`), and `recursive` in `mkdirSync({recursive:true})` / "keys sorted recursively" (`packages/store-postgres/src/migrate-from-sqlite.ts:65`). No provider-probing or best-provider-selection logic. Child runs exist only as the `subagent.start` + `run_completion` continuation pattern (§6).

## 13. Tests that prove these

- Execution ports: `packages/execution/test/{local,memory,registry,capabilities,bubblewrap,program}.test.ts`; `packages/execution-ssh/test/ssh.test.ts`; `packages/execution-wsl/test/wsl.test.ts`; conformance kit `packages/execution/src/conformance.ts:26`.
- Continuations: `packages/continuations/test/lifecycle.test.ts`, `packages/continuations/test/commitments.test.ts`.
- Subagent spawn: `profiles/harness/test/harness.test.ts:132` (durable await of a child) and `:140` (settler on a hand-composed runtime).
- Governed operations: `packages/runtime-bun/test/{governed-operation,governance-env,governance-escalation,governed-system-reads,authority-claims,sea-forge-authority,invoker}.test.ts`; assignment/lease: `packages/runtime-bun/test/{assignment,run-ownership,concurrency,async-admission}.test.ts`.
- Adapters: `packages/mcp/test/mcp.test.ts`; `packages/a2a/test/a2a.test.ts`; `packages/agent-discovery/test/discovery.test.ts`; identity `packages/identity/test/{service,pairing,posture,webauthn,collections}.test.ts`.

## VERIFIED / UNVERIFIED

**VERIFIED** (read directly): §1, §2, §3, §4, §5, §6, §7, §8, §9, §10, §11, §12, §13 (file paths listed by `ls`; subagent test line numbers read in-file).
**UNVERIFIED**: (a) that no other package mounts `programExecutorComponent` into a shipped runtime (only `rg` on `worldId`/`programExecutorComponent` in cli/service — no hit found, not exhaustively excluded); (b) exact per-world behaviour of the ssh/wsl providers beyond their `kind` line; (c) whether any downstream consumer spawns an MCP *client* outside `packages/`+`profiles/` (searched only those trees).
