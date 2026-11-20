# Recon: cognate-workbench — capabilities, governance, settlement, safe mode

Repo: `/home/sprime01/projects/cognate-workbench` @ `main` `f60acb7 feat(capabilities): capability runtime T2 remainder (#38)`.
Read-only recon; no files changed. All citations are absolute-ish repo-relative `path:line`.

## 0. Branch / worktree inventory (what is NOT on main)

| Branch | Worktree | Tip | Delta vs main |
|---|---|---|---|
| `feat/capability-runtime-t3` | `_wt/cwb-t3` | `d84b73b` (10 fix rounds on MCP) | 34 files, +5137/−28 |
| `feat/capability-runtime-t4a` | `_wt/cwb-t4a` | `bbe6c73 feat(capabilities): T4a presentation slots and safe mode` | 39 files, +5759/−28 |
| `feat/capability-runtime-t4b` | `_wt/cwb-t4b` | `efdcc1c` (= merge of T3 #39; **dirty**: M `src/workbench/authorityIndex.ts`, M `reducer.ts`, ?? `authorityFold.ts`) | 34 files, +5137/−28 |
| `feat/wp23-assignment-binding` | `_wt/wp23-cwb` | `668f686 feat(assignments): bind Workbench sessions to case assignments (WP-23)` | 5 files, +235/−83 (includes `src/server.ts`, `test/J-WP23-assignment-binding.test.ts`) |

T3-only files that matter for this recon: `src/capabilities/execSpawn.ts`, `src/capabilities/mcpManage.ts`, `src/workbench/safeMode.ts` (T4a only).
Shared T3/T4a/T4b changed files: `src/capabilities/{aggregateContainment,authorityGrants,capabilityBundles,capabilityRuntime,execSpawn,mcpManage,skillLoader}.ts`,
`src/workbench/{authorityIndex,authoritySelection,reducer}.ts` (+ `safeMode.ts`, `types.ts` on T4a).

---

## 1. `src/capabilities/authorityGrants.ts` — grants, shape, narrowing, enforcement

**Contracts provided** (`authorityGrants.ts:52`): `authority.grant.issue`, `authority.grant.revoke`,
`authority.grant.list`, `capability.runtime.enable`. Component id `capability-runtime:authority` (`:62`).

**Grant shape.** The record is the event payload, not a separate table:
`{ grantId, principalId, permissions: string[], expiresAt?, seeded? }` (`authorityGrants.ts:88-90`).
Principal space is deliberately tiny — `wb-agent:*` derivations plus the operator root only
(`:81`), grant ids must match `/^[a-zA-Z0-9][a-zA-Z0-9._:-]{0,127}$/` (`:22`), and the permission
vocabulary is a closed set (`:26`):

```ts
const KNOWN_PERMISSIONS: ReadonlySet<string> = new Set(["capability.invoke", "skill.load", "harness.modify", "bundle.create", "bundle.activate", "bundle.deactivate", "*"]);
```

Grants are **immutable**; adjustment is revoke + reissue (header `:1-9`). Revocation commits one
CAS-atomic batch that suspends every activation whose `clampProvenance` names the grant
(`:105-112`), and `sweepExpiredGrants` does the same on wall-clock expiry with reason
`grant-expired` (`:157-175`).

**The one seeded grant**: `grant-operator-root` → principal `wb-operator:root`, permissions `["*"]`
(`:19-20`, `:145-150`). This is the `operatorPermissions` leg of every ceiling.

**How a run's tools are narrowed — two independent mechanisms.**

(a) *Preset catalog* (model-facing, app level): `PRESET_TOOLS` (`src/agents/agentCoding.ts:37-41`)
— `standard` has no `run_code`; `ptc` adds it; `minimal` is `bash` + dispatcher only. Filtering is
`toolsForPreset` (`agentCoding.ts:55-59`), and a call outside the turn catalog fails closed in the
tool loop (`agentCoding.ts:688-693`: `Error: unknown tool "<name>"`). Harness overrides may only
select from `KNOWN_TOOLS` and may not add `run_code` outside `ptc`
(`src/workbench/harnessPatches.ts:34-36`, `:97`).

(b) *Grant ceiling* (authority): an activation's ceiling is
`requirements ∩ actor grant ∩ operator root` — `src/capabilities/capabilityRuntime.ts:375-386`:

```ts
const ceiling = requirements.filter((r) => permissionCovered(grant.permissions, r) && permissionCovered(root.permissions, r));
const clipped = requirements.filter((r) => permissionCovered(grant.permissions, r) && !permissionCovered(root.permissions, r));
if (clipped.length > 0) return { ok: false, code: "beyond-operator", ... };
```

**Where enforcement actually happens: the kernel, not the prompt.** The single decision function is
`authorizeCapabilityOp` (`src/workbench/authoritySelection.ts:86-129`), reached from the dispatcher
on every invoke (`capabilityRuntime.ts:264-265`, `:306-313`) and from grant issue/revoke as the last
step before commit (`authorityGrants.ts:71-74`). The decisive code
(`authoritySelection.ts:118-127`):

```ts
const live = liveGrant(index, principal.principalId, a.nowMs);
if (!live.ok) return live;
if (a.permission && !permissionCovered(live.grant.permissions, a.permission)) return { ok: false, code: "beyond-grant", ... };
const root = liveGrant(index, LOCAL_ROOT_ID, a.nowMs, "operator root");
if (!root.ok) return root;
if (a.permission && !permissionCovered(root.grant.permissions, a.permission)) return { ok: false, code: "beyond-operator", ... };
if (row) { const clamp = liveClamp(index, row, a.nowMs); if (!clamp.ok) return clamp; }
```

Fail-closed preconditions precede it: disabled runtime (`runtime-disabled`), unsupported store
(`deployment-invariant`), missing index (`index-unavailable`), and `system` actors are refused
outright as "anti-laundering" (`authoritySelection.ts:14`). Session scoping is derived from the
**run**, never from call input (`sessionOfRun`, `authoritySelection.ts:131-140`).

The operator guard is a second, independent gate for human-only acts: `requireOperator` refuses
`kind !== "user" || runId !== undefined` and records `authority.decision.recorded`
(`src/workbench/operatorGuard.ts:63-75`).

*Prompt-only / app-level (not the kernel)*: the tool-loop `authority.check` verdict for
`edit`/`bash`/`run_code` and the approval waterfall (`agentCoding.ts:697-727`,
`src/domain/approvalGate.ts:51-58`) are app code that decides *whether to dispatch*, not who may
hold authority. They are structural, not prose, but they are not the capability-runtime gate.

---

## 2. `src/capabilities/governance.ts` — SEA-Forge as external authority

**Correction to the brief:** there is **no symbol `governanceActive` anywhere in this repository**
(verified with `rg -n governanceActive` across the tree, including `node_modules/@godspeedai` for the
runtime package; zero hits). What exists is a *derived* authority object, `ExternalAuthority`
(`governance.ts:21-32`), and the toggle is the env var `COGNATE_GOVERNANCE`.

**What it toggles.** `externalAuthorityFromEnv` (`governance.ts:37-58`) maps
`COGNATE_GOVERNANCE=sea-forge` (+ `COGNATE_SEA_FORGE_OPERATOR`, `COGNATE_SEA_FORGE_SOCKET`,
optional `COGNATE_SEA_FORGE_ACTOR`) to a `SeaForgeAuthority` client + service/operator actors; `off`
gives `undefined`. Its presence is what makes the runtime's Governor escalate every capability
invoke as a CEP `authority_request` (header `:3-7`).

**Producer trace** (all of them):
- `src/capabilities/governance.ts:37` — `externalAuthorityFromEnv(env)`.
- `src/server.ts:204` — server option (undefined ⇒ read from env); `src/server.ts:515` —
  `if (realitytrace && !externalAuthority) throw new Error("governed settlement (COGNATE_REALITYTRACE_*) requires COGNATE_GOVERNANCE=sea-forge")`.
- `src/settlement/orchestrator.ts:67` — builds a second `SeaForgeAuthority` for settlement from the
  same config.
- `src/cli.ts:325` — prints the mode; `src/ui/GovernancePane.tsx:132,167` — displays it.

**Capabilities and refusals.** Three contracts (`governance.ts:11-13`):
`governance.status` (never throws — returns `{mode:"off"}` when unconfigured, `:80-82`),
`governance.approvals.list`, `governance.approval.resolve`. The refusal for the last two is the
shared `require()` (`:72-75`):

```ts
const require = (): ExternalAuthority => {
  if (!authority) throw new Error("No external authority is configured (COGNATE_GOVERNANCE is off)");
  return authority;
};
```

`governance.approval.resolve` adds an operator-only gate — an agent must never approve its own
escalation (`:89-95`): `"Only a direct human operator may resolve a SEA-Forge approval"`, recorded
as `authority.decision.recorded` with `reasonCode:"operator-only"`. Config-time refusals:
`ENV_OPERATOR` set without sea-forge (`:41`), unsupported mode (`:44`), missing/invalid operator
(`:45`), operator == service actor (`:47`, "a requester cannot approve its own escalation").

There is **no other "governanceActive" surface** — no boolean is stored, projected, or gated on it.

---

## 3. `src/capabilities/connectHorizon.ts` — end to end

One capability: **`identity.connectHorizon`** (`connectHorizon.ts:36`), component
`identity:connect-horizon`. It invokes **no other capability ids** — it calls the auth helpers
`ensureIdentity` / `issueSession` from `../auth/identity.ts` directly (`:11`, `:51-52`).

Flow:
1. Operator guard: `ctx.actor?.kind !== "user" || ctx.runId` → `${capability.id}: only a direct human operator may connect Horizon` (`:42`); identity must be exactly the operator tenant/actor (`:43`).
2. Input is `{ttlHours}` only (`:45`); default/max are both 90 days (`:13-14`, `:47-49`).
3. `ensureIdentity(..., {label:"horizon"})` then `issueSession(..., {ttlMs, cause:"horizon"})` (`:51-52`).
4. Persistence: `<secretsDir>/workbench.key` written **mode 0600 from the first byte** via
   tmp+chmod+rename (`writePrivate`, `:28-33`), plus companion `workbench.key.json`
   `{sessionId, expiresAt}` (`:56-65`). Companion first, key second, with rollback of the companion
   if the key write fails (`:59-65`).
5. Reply is `{ sessionId, expiresAt }` only — **the token is never returned to the browser** (`:66`,
   header `:4-5`).

---

## 4. `src/capabilities/cognitive.ts` — cognitive capability surface

**20 ids** (`cognitive.ts:80`), all version `1.0.0`:

```
case.open, criterion.freeze, world.freeze, world.fork, world.attach, world.discard,
verification.run, case.derive, world.settle, risk.assess, case.observe,
ui.focus.publish, ui.context, ui.request, ui.pending,
experiment.round, experiment.status, method.review, method.accept, method.active
```

**What they wrap.** Pure folds and policies in `src/cognitive/*` — `case.ts` (`deriveCase`,
`reality`), `criterion.ts` (`freezeCriterion`), `world.ts` (`worldHash`, `worldManifest`,
`foldLineage`, `checkPromotion`), `mediation.ts`, `sentinel.ts`, `preflight.ts` (`riskPreflight`),
`experiment.ts`, `method.ts`, `focus.ts` (`cognitive.ts:22-37`). Execution and storage are borrowed,
not owned: `requires: { resolve, read, list, exec }` from `filesystem.*` / `process.exec`
(`:108-113`), and state is persisted only as events through a system run of `AGENT_COGNITIVE`
(`record()`, `:96-100`: `startRun(... app: AGENT_COGNITIVE ...)` then `waitDone`). Module header
`:1-5`: "Nothing here keeps its own store: the Case, Reality and lineage are folds of events, so a
restart reconstructs them."

**External runner:** none of its own. It does not spawn RealityTrace, MCP, or any bundle process;
process work goes through the framework `process.exec` capability into an execution world. The only
`Bun.spawn` in the settlement/cognitive neighbourhood is `RealityTraceCli.run`
(`src/settlement/realitytrace.ts:66`), which is not reachable from this component.

Note a hardcoded assumption worth flagging: `recoveryAvailable: true` and `observerAvailable: true`
are literals in every facts object (`cognitive.ts:236`, `:258`, `:271`), so "recovery is available"
is asserted, not observed.

---

## 5. Settlement — two distinct machines, deliberately not merged

### 5a. The *local* case lifecycle (`cognitive.ts`)

| step | capability | what it does | persisted |
|---|---|---|---|
| `case.open` | `cognitive.ts:147-152` | declares desired outcome + optional checks | `case.opened {caseId, desiredOutcome, unknowns, checks}` (`:149`) |
| `criterion.freeze` | `:153-158` | freezes the observer/criterion | `criterion.frozen {caseId, criterion}` (`:155`) |
| `world.freeze` | `:159-166` | baseline/candidate manifest → `worldHash` + lineage | world events (`WORLD_EVENTS`) |
| `world.fork` | `:167-189` | `cp` source root into a temp dir, register a disposable world; `world.discard` `rm`s it (`:186`) | world events |
| `verification.run` | `:190-231` | runs the frozen checks; the *process result* is the fact | `verification.observed` |
| `world.settle` | `:253-269` | **human-only promotion** | `world.settled {worldHash, parentWorldHash, caseId, evidenceRefs, authority}` (`:267`) |

`world.settle` refuses any in-run call outright (`:256`
`"world.settle: settlement is a human act and cannot be invoked from a run"`) and only records when
`checkPromotion` accepts — candidate matches the derived projection, evidence present, and
`settlementReachable` (`:260-266`). Return value is `{ settled: candidateHash }`.

**What "accepted" means locally**: a promoted candidate hash in the lineage, backed by recorded
verification evidence and a human actor id. Nothing external is contacted; no receipt is issued by
any other system. It is a *local* acceptance.

### 5b. Governed settlement (`settlement/orchestrator.ts`, `launcher.ts`, `realitytrace.ts`, `sweSeed.ts`)

Scope: only `agent.coding` runs, only these capabilities (`orchestrator.ts:22`):

```ts
export const CONSEQUENTIAL_CAPABILITIES: ReadonlySet<string> = new Set(["tool.edit", "tool.bash", "tool.run_code"]);
```

Events (`orchestrator.ts:23-25`): `workbench.proof.declared`, `workbench.settlement.evidence`,
`workbench.settlement.recorded`. Statuses (`:48`):
`"settled" | "rejected" | "unsettled" | "refused" | "not-attempted"`.

Lifecycle, all resumable from the record:
1. Proof must have been declared for the workspace **before** the run started — declaring later
   cannot reach back (`:100-109`; if absent → `not-attempted`, reason `"no proof was declared for this workspace before the run started"`).
2. Under `COGNATE_PROOF_SOURCE=swe_seed`, a typed proof is refused (`:114-117`;
   `capabilities/settlement.ts:100`).
3. RealityTrace verifies under Workbench's containment launcher (`orchestrator.ts:129-137`;
   `containedVerifierLauncher`, `launcher.ts:20-46`); evidence is reused on restart (`:121`).
4. Per operation: `seaForge.submitEvidence(packet)` then `seaForge.settle(operationId, worldRef)`
   (`:153-164`) — **the recorded status is SEA-Forge's**, and `final` comes from SEA-Forge.
   Workbench never decides the outcome (header `:9-10`).
5. Records are keyed `settlement:${runId}`; boot `sweep()` re-checks every ended run (`:181-190`).

`realitytrace.ts`: env must be all-three-or-none (`:22-30`), `PROOF_QUESTION` /
`PROOF_THRESHOLD` declared once per repo (`:33-40`, `:90-97`), and the verifier must report
`verifier_id === "command"` or the evidence is rejected (`:115`).
`sweSeed.ts`: `swe-seed eval proof --spec --check --sha256` (`:62-90`); the answer is accepted only if
`schema === "swe_seed.eval_proof.v1"`, the digest matches the pin, and `check_id` matches (`:80-85`).

### 5c. Does anything label local acceptance as external settlement?

No mislabelling found in code: `world.settled` (local, `cognitive.ts:267`) and
`workbench.settlement.recorded` (external, `orchestrator.ts:160-164`) are different event types with
different producers, and `settlement.status` returns `{ active, operations, evidence }` where
`operations` are only the SEA-Forge records (`capabilities/settlement.ts:140-158`). The UI keeps them
apart deliberately — the E2E test name is literally "settlement reachable != settled"
(`test/E2E-cognitive-console.test.ts:39`).

One genuine *conflation risk*, not a mislabel: both families feed the same SXR observation stream —
`OBSERVED_EVENT_TYPES` includes `"world.settled"` alongside `tool.call.*` (`src/observations.ts:28-37`).
A downstream reader of the SXR ledger therefore sees a locally-promoted candidate as an "observation"
of the same class as tool evidence, with nothing in the event type marking it local.

---

## 6. Bundle model — `capabilityRuntime.ts` + `capabilityBundles.ts` + `bundleStore.ts`

**Manifest shape** (ajv schema, `capabilityBundles.ts:24-50`): `{ name, purpose, skills[{name, entry,
isEntryDocument?, trigger?}] (≤8), tools[{id, description, inputSchema}] (≤8), requirements[] (≤16),
conflicts[], complements[], owns[], harness{} }`; required `name`, `purpose`, `skills`;
`additionalProperties: false`. `capabilityId` must match `^capability\.bundle\.[a-z0-9][a-z0-9.-]{0,63}$`.

**Install (register)** — `capability.bundle.register` (`capabilityBundles.ts:101-185`): operator or a
covered grant (`requireOperator` + `bundle.create`, `:104-114`); requirement kinds must be one of
`capability.invoke | skill.load | harness.modify | bundle.create | bundle.activate` (`:54`, `:127-129`);
harness surfaces require the `harness.modify` requirement (`:137-139`); the source may be a local
dir or an execution-world dir staged through its filesystem port (`:145-155`); ceilings and
skill-entry containment are checked (`:156-168`); then `materializeVersion` + prune + record
`capability.bundle.registered` (+ `workbench.bundles.pruned`) (`:171-181`). Every refusal also records
`capability.validation_failed` (`:123-126`).

**Store** (`bundleStore.ts`): `<home>/bundles/<capabilityId>/<versionHash>/` (`:84-90`); payload
copied once and immutable (`:94-103`); ceilings `maxFileBytes 512 KiB`, `maxBundleBytes 5 MiB`,
`maxFiles 200`, `retentionBeyondActive 3` (`:15-20`); load-time integrity re-hashes the directory
(`:106-118`); version identity covers payload **and** whole manifest (`:34-40`). The store sits under
the app data home, which "is refused to contained executables" (header `:2-4`).

**Enable / activate / deactivate.** `capability.bundle.activate` and `.deactivate` live in the
dispatcher (`capabilityRuntime.ts:397+`), serialized on the shared mutex (`:394-395`), with the
ceiling/clamp computed by `authorize` (`:375-386`) and harness/ownership conflicts checked against
live activations (`:359-371`). Enablement itself is the *runtime* marker, not a per-bundle flag:
`capability.runtime.enable` refuses without the remediation marker and on a multi-writer store
(`authorityGrants.ts:127-138`).

**Can a bundle add tools or runners?**
- *Tools*: yes — declared and schema-validated, dispatched under `tool.capability_invoke` with ajv
  argument checking (`capabilityRuntime.ts:137-147`).
- *Runners / executable handlers*: **not on main.** `tool.invoke` terminates at
  `{ ok: false, code: "no_executable_handler", message: "...has no executable handler in this release" }`
  (`capabilityRuntime.ts:145-146`), and `capabilityRuntime.ts:5-7` states "executable handlers (T3)
  are spawned, contained processes". The T3 branch adds exactly that wrapper
  (`_wt/cwb-t3:src/capabilities/execSpawn.ts:1-24` — "the ONLY route by which a bundle's executable
  payload becomes a process"), with a per-provider containment matrix and a record-before-spawn
  ledger. On main a bundle therefore cannot run code — but it *can* widen a preset's toolset through
  the validated `harness` override (`workbench/harnessPatches.ts:34-36`, `:97`).

---

## 7. "Safe mode" / recovery machinery

**There is no safe mode, and no `safeMode`/`recovery`/`reset` restore path, on `main`.** The only
`safeMode` in the tree is `_wt/cwb-t4a:src/workbench/safeMode.ts`, added by
`bbe6c73 feat(capabilities): T4a presentation slots and safe mode` (not merged to main).

What T4a's safe mode is (`safeMode.ts:1-7`, `:40-62`, `:100-150`):
- **Derived, never a stored flag**: `foldSafeMode` folds `capability.presentation.faulted` and
  `workbench.safemode.{boot_attempted,boot_ready,cleared}` (`:13-18`, `:40-57`).
- Two triggers: a `core-violation` fault class, or `FAILED_BOOTS_TO_SUSPEND = 2` consecutive attempts
  with the *same* extension-set hash and no `boot_ready` (`:22`, `:54-56`).
- `SafeModeRead` is brand-typed and mintable only inside the module (`:26-34`); both render
  (`FragmentSlot`) and mount (`mountBundleHandlers`) take it as a parameter, and in safe mode
  `mountBundleHandlers` returns `[]` — nothing extension-provided mounts (`:143-148`).
- Lifting requires `workbench.safemode.cleared` from `actor.role === "operator"`; any other role is
  recorded as an `unauthorized-clear` fault and refused (`:126-135`).

**What it can restore today: nothing.** It excludes extension/presentation fragments and re-derives
the read after an operator clear. It never re-installs a bundle, re-seeds data, or restores state.

**`seedExperts` call sites** (3 + one boot call):
- `src/experts/state.ts:57` — definition.
- `src/experts/service.ts:97` — `seed()` → `mutate(..., "initialize:v1", {kind:"seed:v1"})`.
- `src/experts/service.ts:345` — the `experts.seed` case in `command`.
- `src/server.ts:1031` — `await experts.seed(LOCAL_OPERATOR.tenant)` at boot.

**`seed:v1` receipt semantics** (`src/experts/state.ts:57-76`):

```ts
export function seedExperts(state: ExpertState): void {
  // Seed once, including deliberate user removals/archives. No restart overwrites customization.
  if (state.receipts["seed:v1"]) return;
  ...
  state.receipts["seed:v1"] = {fingerprint:"seed:v1",result:{ok:true}};
```

Seeds five profiles (`product-manager` as orchestrator, plus `builder`, `critic`, `verifier`,
`reviewer`) and team `cognate-team` with
`limits:{maxDepth:2,maxAssignments:12,maxUnread:20,maxRounds:8,maxConcurrentRuns:2,admissionBatch:1}`
(`:50-55`, `:70-74`). Because the receipt short-circuits re-seeding, **there is no "restore built-in
experts and team" path**: an operator who deletes `cognate-team` or a profile cannot get it back via
`experts.seed` (it is a no-op), and no capability clears the receipt. Restoring today would require
editing the durable experts state out of band.

Other recovery machinery that does exist (unrelated to safe mode): interrupted-owner takeover at CLI
start (`src/cli.ts:67-74`, `:223`), expert billing crash-recovery generations
(`src/experts/billing.ts:172-211`), and SSH/WSL world restoration at boot (`src/README.md:97`).

---

## 8. `src/observations.ts` and `src/components/pluginRegistry.ts`

**`observations.ts`** — an I/O + wiring module, not a registry:
(a) `observationOutboxFor`, the runtime `outboxFor` hook that commits an observation intent atomically
with each observed event, over a fixed `OBSERVED_EVENT_TYPES` set (`:28-37`, `:50-53`); model
requests only for `MODEL_CAPABILITIES = {"model.generate","expert.model.generate"}` (`:39`);
(b) `defaultSxrBin` (pinned `.tmp/realitytrace/bin/realitytrace` fallback);
(c) `wireObservations` — SXR prerequisite workflow plus an at-least-once `ObservationDelivery` drain,
gated on the durable RealityTrace plugin state;
(d) `observationCapabilityComponent` (id `observation:read`, provides `observation.status` and
`observation.record`, **unrecorded** — receipts live in the DurableStore `observations` collection,
not the event log; header `:8-14`).
Consumers: `src/server.ts:112` (import) and `src/server.ts:737` (`outboxFor: observationOutboxFor`);
`test/J-RT-realitytrace.test.ts:16`.

**`components/pluginRegistry.ts`** — **legacy, explicitly superseded.** Its own header says so
(`:17-21`): "It is superseded as the Workbench's extension path by the capability-bundle system…
Directory sources for bundles arrive through the capability runtime (T2), not through this registry."
It is nevertheless still live and wired: `class PluginRegistry` (`:53`) is constructed once at
`src/server.ts:638` and handed to capability components at `src/server.ts:806`. It resolves
`PluginSource`s (built-in catalog) and verifies that a declared implementation already exists in this
build — no downloaded code is ever evaluated (`:17-19`) — and performs no durable side effects.
Consumers: `src/server.ts:111,638,806`; `src/capabilities/pluginInstall.ts:14,37` (+ its capability);
`src/workbench/bundleSources.ts:22` (reuses only the `PluginSource` type); tests `test/J-PLG-*.test.ts`.

---

## 9. Containment for tool execution — what is enforced vs prompt-only

**Where the processes actually start.** `src/capabilities/tools.ts` owns `tool.read|edit|bash|run_code`
(`:168`) and does **no** direct `node:fs` / `Bun.spawn` (header `:1-14`). It requires
`{ filesystem.resolve/readText/list/writeText, exec: { capability: "process.exec" } }` (`:170-174`) and
delegates each call to the execution world named by `executionWorldId` (`:208-227`). Concretely:

```ts
// tool.bash
{ worldId: world(input), argv: ["bash", "-lc", input.command], cwd: input.workspaceRoot, ... }
// tool.run_code
{ worldId: world(input), argv: ["bun", "-e", RUN_CODE_HARNESS(input.code)], cwd: input.workspaceRoot, ... }
```

Both are the *same* `process.exec` path in the *same* world (`tools.ts:208-227`). `run_code` is not a
stronger sandbox — it is `bun -e` with a small `tools.bash` harness (`:125-131`); the only difference is
that it is `ptc`-preset-only and it is the tool that can request `sandbox_permissions`, which is what
triggers approval (`src/domain/approvalGate.ts:8-13`, `:51-58`).

**Enforced (structural, OS-level):**
1. **Execution posture** — default `contained` refuses `process.exec` and `filesystem.writeText` on any
   `local`-kind world; `uncontained-local` is an explicit operator opt-in
   (`src/executionPosture.ts:1-35`; refusal text `:27-34`). The guard sits at the registry's `get`, so it
   covers "the base world, cognitive directory-copy forks, and anything registered later… No caller can
   route around it" (`executionPosture.ts:11-13`).
2. **bubblewrap namespaces** for contained worlds (`@godspeedai/cognate-execution`'s
   `bubblewrapExecutionWorld`, `containedWorkspace.ts:8`). The settlement verifier launcher shows the
   shape (`settlement/launcher.ts:38-46`): `--unshare-all --unshare-user --disable-userns
   --assert-userns-disabled --new-session --die-with-parent --cap-drop ALL --clearenv`, ro-binds of only
   `/usr/bin /usr/lib /usr/lib64 /bin /lib /lib64`, tmpfs `/tmp`, and a **read-only** workspace bind.
   Binary provenance is checked against `provenance.binarySha256` (`:22-23`).
3. **Aggregate quotas** — one operator-configured ceiling across all contained executions in a store:
   `maxConcurrent` (leases + reaping), `cpuSecondsPerWindow`, `cpuSecondsPerExecution` (`RLIMIT_CPU` via
   `prlimit`), `maxAddressSpaceBytes` (`RLIMIT_AS`); "if the quota is configured but the tooling is
   unavailable inside the world, the execution is refused, never run uncapped"
   (`capabilities/aggregateContainment.ts:1-20`).
4. **Output cap** `MAX_OUTPUT = 20_000` chars, truncated visibly (`tools.ts:19-22`).
5. **Host containment config is operator-owned and structurally validated** — a caller may choose a
   workspace directory but never the sandbox implementation; `validateContainmentConfig` requires
   `operatorVerified:true`, `patchedCve202687766:true`, a 64-hex `binarySha256`, and runtime roots ⊆
   `{/usr/bin,/usr/lib,/usr/lib64,/bin,/lib,/lib64}` (`containedWorkspace.ts:17-55`).

**Prompt/app-only (NOT a sandbox):** the `authority.check` verdict and approval waterfall
(`agentCoding.ts:697-727`), the read-only-Expert approval insertion (`:718-720`), the RiskPreflight
mediation for destructive commands (`:728-741`), and the `read-only | workspace-write | full-access`
permission mode all decide *whether to dispatch*. Path checks are explicitly declared "not a sandbox"
(`src/executionPosture.ts:8-9`).

**Platform scope:** the only enforcement implementation in-tree is Linux bubblewrap. There is **no
seccomp/Landlock/seatbelt/sandbox-exec** code in `src/` (searching `landlock|seatbelt|sandbox-exec`
yields prose only). macOS/Windows containment is therefore not enforced by this repo; those hosts get
the posture refusal (`contained` refuses local effects) or the operator's explicit `uncontained-local`
opt-in. On the T3 branch this becomes an explicit per-kind matrix with `enforced|partial|not-enforced`
per restriction and "refusal is the default" (`_wt/cwb-t3:src/capabilities/execSpawn.ts:4-17`, `:50-70`).

---

## 10. Test inventory and the commands that run them

Runner: **`bun test`**. `package.json` scripts (`package.json:10-15`): `test` =
`bun test --timeout 15000 test`; `typecheck` = `tsc -p tsconfig.json`; `verify` = `typecheck && test`;
`test:smoke` = the contract/glob/skill-bundle trio. `justfile`: `just test` (`:11`),
`just test-shard shard total` (`:16-19`, sharded by `ci/test-timings.json`), `just verify` (starts pg0
first, `:21-23`), `just smoke` (`:25-26`), `just setup-realitytrace` (`:59-60`, needed by
`test/liveSettlement.ts`).

| Area | Test files | Asserts |
|---|---|---|
| Experts / Teams (UI) | `test/E2E-experts.test.ts:11,12,99` | default `cognate-team` renders; activating a custom Team with exactly one Orchestrator; create/edit/copy a routed Expert, reopen solo + private history |
| Team admission | `test/E2E-team-admission.test.ts:8`; `J-TEAM-admission`, `J-TEAM-bounds`, `J-TEAM-security` | effective admission policy + serialized refusal reasons; bounds; security invariants |
| Delegation | `test/J-F-delegation.test.ts:66-67,132` | child run starts, parent completes on the reflection text, result lands in the parent's next turn |
| Team execution | `test/J-TEAM-execution.test.ts:21,93,125,213` | delegate/handoff round-trip; durable retry after provider failure; restart resume without a browser turn; Builder edits via the durable effect boundary and Verifier reads the change |
| Team governance / settlement | `test/expert-governance.test.ts:5`; `J-TEAM-evaluation`; `expert-economics`; `expert-admission` | internal returns cannot be forged; Experts cannot approve protected effects |
| Governance (SEA-Forge) | `J-SEA-FORGE-AUTHORITY`; `E2E-onboarding`; `ui-governance-approval.test.tsx`; `approval-bar-render.test.tsx`; `approval-summary` | escalation parking, operator-only resolve, approval UI |
| Settlement | `J-SW01-SETTLEMENT.test.ts` (127 lines); `J-SW02-PROOF-CONTRACT`; `settlement-config`; `proof-current`; `test/liveSettlement.ts` | proof declaration/contract, settlement status, config gating |
| Capabilities (T1/T2) | `J-CAP-refusal.test.ts:33,60,152,161,178` + `J-CAP-{create,harness,prerequisite,reload,source,review-cr,review-r1..r4}` | enablement gate; CUJ-4 matrix (forged grants, beyond-grant, beyond-ceiling, system caller, alternate path, suspension, expiry, clamp revocation); principal mapping |
| Bundles / plugins | `skill-bundle`, `J-PLG-install`, `J-PLG-panels`, `J-PLG-toggle` | bundle materialization; plugin activation is verification-only |
| Cognitive | `E2E-cognitive-console.test.ts:39,154,201`; `J-COG-{attention,cleanup,liveverify,realitytrace,search}`; `cognitive-semantics/focus/bindings` | focus → "why did this fail?"; mutation needs the human; "settlement reachable != settled"; semantic preflight overrules a lower-trust waiver; a method recommendation stays a candidate until accepted |
| Containment | `containedWorkspace`, `J-RW-AGGREGATE-CONTAINMENT`, `J-RW-contained-workspace`, `J-EXEC-POSTURE`, `E2E-contained-workspace` | posture refusal, bwrap config validation, aggregate quota admission/reaping |
| Tool execution | `J-C-tool-execution`, `J-D-approval`, `J-TASK-*.red` | tool invocation through a world; approval waterfall |

---

## 11. RLM / recursive-runner code

**None exists.** `rg -n -i 'rlm|prime|recursi'` over `src`, `docs` and the root markdown returns no RLM,
no "prime", and no recursive-runner construct. The only `recursi*` hits are unrelated:
`mkdir/rm … {recursive:true}` call sites; recursive shell-wrapper unwrapping
(`src/workbench/shellSafety.ts:161-248`); recursive glob alternation (`src/workbench/glob.ts:132`); and
the one genuinely named recursion guard in the mediation layer (`src/cognitive/mediation.ts:64-65`,
`:120`):

```ts
export class TrustedSubchainRecursion extends Error {
  override readonly name = "TrustedSubchainRecursion";
...
if (stack.includes(m.id)) throw new TrustedSubchainRecursion(`trusted subchain re-entered by ${m.id}`);
```

The closest thing to a recursive *runner* is ordinary subagent/work delegation, not an RLM loop:
`subagentStart` (`src/capabilities/subagentStart.ts`) for parent/child coding runs (tested by
`test/J-F-delegation.test.ts`), and team delegation/handoff bounded by `maxDepth:2, maxRounds:8`
(`src/experts/state.ts:73`). No recursive context/token loop, no self-critique recursion, no RLM
executor.

---

## VERIFIED / UNVERIFIED

**VERIFIED** (read at the cited lines, or established by mechanical search):
- Grant contracts, shape, permission vocabulary, immutability, revocation/expiry batches (§1).
- The single authorization point `authorizeCapabilityOp`, its call sites, and the dispatcher /
  operator-guard gates; preset + harness narrowing (§1).
- `governanceActive` does not exist; the real toggle is `COGNATE_GOVERNANCE` → `ExternalAuthority`,
  with every producer call site listed (§2).
- `identity.connectHorizon` is the only capability id in `connectHorizon.ts`, and it invokes no other
  capability ids (§3).
- The 20 cognitive capability ids, what they fold/wrap, and that no external runner is invoked (§4).
- Both settlement machines, their event types and statuses, the absence of a local→external mislabel,
  and the SXR-stream conflation risk (§5).
- Bundle manifest schema, register/store/prune ceilings, and `no_executable_handler` on main (§6).
- Safe mode exists only on `feat/capability-runtime-t4a` and restores nothing; `seedExperts` has 3 call
  sites + boot and is one-shot via `receipts["seed:v1"]`, so no restore path exists (§7).
- `pluginRegistry` is live but self-declared legacy; `observations.ts` is an I/O/wiring module (§8).
- Tool execution path (`process.exec` per world), posture refusal, bwrap argv, aggregate quotas; no
  Landlock/seatbelt code in-tree (§9).
- Test file names, their first assertions, and the `package.json`/`justfile` commands (§10).
- No RLM/recursive-runner code (§11).

**UNVERIFIED** (asserted by comment/name, or out of reach for read-only recon):
- Whether `contained` is the effective posture in a given deployment (env-dependent; server not run).
- Completeness/consistency of T3's `execSpawn` containment matrix — I read its header, matrix and refusal
  lines only, not the full 10-round-hardened file.
- Whether any *external* consumer of the SXR ledger conflates `world.settled` with tool evidence — the
  §5c risk is inferred from event-set membership, not observed downstream.
- Whether `feat/capability-runtime-t4b`'s **uncommitted** `authorityFold.ts` changes the enforcement
  story (worktree is dirty; not read).
- Whether the §10 suites pass today: `bun test` was not run, and several need pg0 / RealityTrace /
  SEA-Forge fixtures.


