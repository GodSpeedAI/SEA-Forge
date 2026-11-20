# Design critic (EGM Goldfish) — CognitiveOS Convergence package

Reviewer mode: REVIEW. Baselines checked: cognate `29dcd1e`, cognate-workbench `f60acb7` (plus
`feat/capability-runtime-t4a` `bbe6c73`, `feat/wp23-assignment-binding` `668f686`), Horizon
`0e34a34`, gauntlet `6a7fe6a`, sea-rs `2f35ad27`, cep `5111ec3`, sxr `42f38ca`.
No build, test, or live run was executed. Every finding below quotes the design text and the
`path:line` I read.

The package is far above average on citation discipline — `types.ts:58-68`, `service.ts:190`,
`:195-199`, `:400-423`, `:424-430`, `harness.ts:89`, `server.ts:785`, `observations.ts:28-35`,
`agent_runner.rs:46-48`, and `A5`'s version table all check out exactly. That is what makes the
failures below worth reporting: they are not sloppiness, they are places where the design's
central claims do not survive contact with the mechanisms it chose to reuse.

---

## Blockers

### B1 — `native` cannot be both "one child run" and "the existing in-process Expert loop", and Cognate owns no Expert agent

*Affected:* `design.md:80-85`, `D-minimal-contracts.md:108-112`, `F-implementation-sequence.md:31-43`,
`:121-123`, `J1`/`J4` (`E-acceptance-journeys.md:14,45-58`).

> "`rlm.invoke` starts exactly one child run of an existing Assignment … the `native` runner is the
> existing in-process Expert loop and reports `contextDecomposition:false`" — `design.md:81-84`

> "1. **One invocation, one run.** `rlm.invoke` creates exactly one Cognate run with
> `idempotencyKey = "rlm:" + invocationId` and `correlationId = "rlm:" + parentRunId + ":" + invocationId`."
> — `D:108-109`

> "I1 is deliberately Cognate-only: it proves the port is truthful before any consumer depends on it,
> and it needs no Workbench, no Prime and no Gauntlet." — `F:121-122`

Three facts make both halves impossible:

1. A Workbench Assignment is bound to exactly one run. `cognate-workbench/src/experts/types.ts:69-83`
   declares `execution: ExpertExecution; runId: string | null;` (single-valued), and
   `src/experts/service.ts:119-122` refuses a second binding:
   `if (current.runId && current.runId !== ctx.runId) throw new Error("Assignment already bound");`.
   The one-run-per-attempt dispatch is `src/experts/bridge.ts:47-48`
   (`idempotencyKey: expert-run:${a.id}:${a.attempt}`).
2. At the case level it is also one run per assignment: `cognate/packages/runtime-bun/src/service.ts:145`
   writes the `ASSIGNMENTS` collection entry with `expectedVersion: 0` for
   `${tenant}/${assignment.assignment_id}` (collection documented as "one entry per assignment id …
   the run it was opened as", `packages/runtime-bun/src/assignment.ts:15-16`). A second run opening
   the *same* CaseAssignment is a CAS conflict, so `rlm.invoke` cannot start a child run of an
   existing Assignment whose identity it claims to inherit (`design.md:28-30`).
3. The Expert loop is not in Cognate. `EXPERT_AGENT = "agent.expert"` is declared at
   `cognate-workbench/src/experts/types.ts:9` and implemented at `src/agents/agentCoding.ts:472`,
   registered at `src/server.ts:55` (`createAgentCodingAgent`). `grep -rn 'agent\.expert\|agent\.coding'
   packages` in cognate returns **nothing**; `grep -rn 'depth'` over
   `packages/runtime-api/src` + `packages/kernel-api/src` returns **nothing** either.

So: if `native` means "in-process", D4 rule 1 is false for the only provider that ships in I1, and
`rlm.invoke(native)` has no run to return. If `native` means "child run", it needs an Expert agent
Cognate does not have, and a second run of an Assignment the Workbench model forbids. I1's
"Cognate-only, needs no Workbench" is therefore not achievable as specified.
*Recommendation:* pick one. Either `native` is a registry *fact* only (no `rlm.invoke` for it, which
requires rewording D4.1 from a universal rule to a provider rule) or the child run is a distinct
Cognate-owned `agent.rlm-child` whose lineage lives in `caseContext`/`correlationId`, with the
Assignment binding explicitly declared *not* to be reused.

### B2 — The extracted spawn cannot carry the provider path in any governed or contained configuration

*Affected:* `B:30-48` (G2), `F:58-69` (I3), `F:95-106` (I6), `design.md:129-131`, `E:147-157` (J13),
`C:79-98` (C4).

> "**EXTRACT** into `cognate/packages/runtime-bun` as capability `process.spawn.durable`" — `B:39-40`
> "**Reusable**: everything in `execSpawn.ts` (containment matrix `worldMatrix`, `decideContainment`,
> posture record, intent/spawned/exited records, long-lease slot)" — `B:36-38`

There is exactly one spawn path in that file, and it is closed in every configuration the design
needs. Citations are to `feat/capability-runtime-t4a` (`bbe6c73`), `src/capabilities/execSpawn.ts`
(the design's `_wt/cwb-t4a/...` path does not exist — see M6):

* `:195` — `if (world.kind !== "local") return refuse("stream-port-unavailable", ...)`. Only a
  `local` world can hold a live stdio process. Contained (bubblewrap) workspaces cannot.
* `:196` — `if (!deps.localHostSpawnAllowed()) return refuse("posture-contained", "the execution
  posture is contained: host-process spawns on the local world are refused")`.
* `cognate-workbench/src/server.ts:510-511` — `if (externalAuthority && executionPosture !==
  "contained") throw new Error("SEA-Forge governance requires COGNATE_EXECUTION_POSTURE=contained:
  under uncontained-local an agent could reach the SEA-Forge socket and approve its own escalation")`.
  `executionPosture` defaults to `"contained"` (`src/executionPosture.ts:24-27`).
* `execSpawn.ts:53,70,76-80` — the matrix says `local` has `network: "not-enforced"`, and
  `decideContainment` requires `["network","subprocess","termination"]`, refusing with
  `unenforceable:<restriction>` unless the operator's posture *record* authorizes the weaker level by
  name. `subprocess` is `"not-enforced"` for **all four** kinds (`:52-55`) — and a recursive provider
  is by definition a process that spawns processes.
* `execSpawn.ts:64-66` — even in a bubblewrap world, `--unshare-all` means "a declared outbound scope
  cannot be satisfied there". A Prime-style provider cannot reach a model API in the one world kind
  that is network-enforced.

Consequences the design does not state anywhere (this appears nowhere in `B`, `F`, `G`, or `E`):

* Under SEA-Forge governance the provider path and `case.escalate` are **refused at the spawn seam**
  (`:196`), because governance mandates the very posture that forbids host-process spawns. I4 and I6
  deliver nothing for the governed case — the case G7 exists to fix.
* Without governance you must run `uncontained-local` with a relaxed posture record: the exact
  "external CLI whose tool calls cannot be governed or settled" class `src/capabilities/harness.ts:89`
  refuses, and the exact self-approval hole `server.ts:508-511` closes by construction.
* J13 asserts the loss is macOS/Windows (`E:149-151`). It is not: the refusal is matrix- and
  posture-driven on **every** platform; macOS is just `kind: "local"`. The only platform-specific
  refusal in the wrapper is `win32` process groups (`:197`).

*Recommendation:* before I3 is scheduled, publish the admission table honestly — which
(world kind x posture x governance on/off) cell admits an external provider. If the answer is "none
under governance", say so in `design.md`, delete the G7 reframing, and let I4/I6 stand on local-only
use with the self-approval risk raised as an owner decision instead of sitting in `G3`.

### B3 — D4's normative rules 3-5 have no enforcement point in Cognate, and the contract cannot express its own inputs

*Affected:* `D:106-126` (rules 3, 4, 5), `D:34-53` (`rlm.invoke` input), `D:127-141` (codes),
`design.md:129-130`, `D:62` (`payment.basis`).

> "3. **Authority narrows.** `tools(child) = tools(parent) ∩ request.tools ∩ provider.toolPolicy` …
> 4. **Depth.** `depth(child) = depth(parent) + 1`, refused above the governing limit (Team
> `maxDepth` for Team work; 1 for solo Expert work).
> 5. **Budgets.** A numeric ceiling is checked against the durable reservation before dispatch." — `D:113-119`

* **Authority.** Cognate has no per-run tool grant to intersect with. `grep -rn
  'allowlist|toolGrants|authorityIndex' packages/kernel-api/src packages/runtime-api/src` returns
  nothing; Cognate's authority is `GovernedOperation`, which authorizes *capability* calls through
  `ActionPolicy`/SEA-Forge (`cognate/packages/runtime-bun/src/governed-operation.ts:1-25`), not a tool
  set. The only tool-grant set in the ecosystem is `Assignment.authorityTools: ExpertTool[]`
  (`cognate-workbench/src/experts/types.ts:71`, computed at `src/experts/service.ts:195-199`) — in the
  app. `rlm.invoke`'s input supplies `authority: {tools, mode}` (`D:45`) with no parent grant, so
  `rlm-authority-exceeds-parent` (`D:133`) is computable only if the caller is trusted to declare its
  own ceiling. A direct `RuntimeService.invoke` then bypasses the intersection entirely.
* **Depth.** `rlm.invoke`'s input has no `depth` and no limit (`D:36-48`); Cognate has no `depth`
  anywhere (`grep -rn depth packages/runtime-api/src packages/kernel-api/src` -> nothing). The
  governing limit is Workbench's Team field (`src/experts/types.ts:31-38`,
  `src/experts/service.ts:190`). `rlm-depth-exceeded` (`D:134`) cannot be evaluated from the declared
  input.
* **Budget.** The "durable reservation" is Workbench's: `src/experts/billing.ts:1-9,14-17` (thread
  `experts:billing:v1`, kinds `expert-model`/`session-model`, USD price pairs). Cognate's own budget
  vocabulary is a third one: `Currency = "time" | "attention" | "money" | "risk" | "recovery"`
  with `limit: number | "unknown"` (`cognate/packages/runtime-api/src/assignment.ts:19-27`). Three
  vocabularies (`{totalTokens,maxCostUsd,currency}`, `Currency`, USD prices) with no mapping rule, and
  `rlm-budget-unverifiable` is decided by an app the kernel cannot see.

The consequence is not stylistic. `design.md:129` claims "no prompt-only enforcement; authority
intersection at the kernel", and the contract as written places all three intersections in the
callee's *declared inputs*. Either the enforcement is in Workbench (the NFR is false) or Cognate must
grow a per-run grant + depth + reservation record, which is the new execution state
`design.md:47` says is out of scope.
*Recommendation:* add `parentGrant`, `depth`, and `budgetReservationRef` to the `rlm.invoke` input and
state them as verified against the parent run record, or move the enforcement claim to Workbench and
delete the kernel sentence from the NFRs.

---

## Major

### M1 — G12's premise is wrong: governed settlement does not reach the observation stream at all

*Affected:* `B:217-233` (G12), `A:72`, `design.md:22-24`, `E:94-106` (J9).

> "But both are folded into the SXR observation stream via `OBSERVED_EVENT_TYPES`
> (`src/observations.ts:28-37`) with nothing marking scope." — `B:220-222`

The set is `cognate-workbench/src/observations.ts:28-35` and it contains `"world.settled"` (`:31`)
but **not** `"workbench.settlement.recorded"` (`SETTLEMENT_RECORDED`, defined
`src/settlement/orchestrator.ts:25`). `grep -n 'settlement' src/observations.ts` returns only the
comment on line 30.

So the downstream defect is the opposite of the one stated: a governed settlement is *absent* from the
observation stream, not "indistinguishable" in it. Adding `settlementScope` to the payload
(`B:226-227`) changes nothing downstream for the governed side, and J9's pass criterion — "the SXR
observation stream shows the scope" (`E:101`) — fails for exactly the case the field exists to label.
Secondary cost fact the design also misses: `observations.ts:26-27` says "every observation is an SXR
ingest subprocess", so the correct fix (observe the settlement family) buys a new subprocess per
settled operation.
*Recommendation:* restate G12, and split I5 into (a) label both payloads, (b) add the settlement
family to the observation fold with an explicit budget for the extra SXR ingests.

### M2 — `G1` is labelled `NEW` although the port already exists in Gauntlet, and the one dimension dropped from the copy is the one whose enforcement is admitted to be unproven

*Affected:* `B:8-28` (G1), `B:352-359`, `D:8-32`, `D:55-66`, `D:42-44`, `E:57-58`, `G:32-34`.

> "**Disposition**: **NEW** … Justification: no existing port describes provider selection, invocation
> correlation, terminal status or result provenance. REUSE was tried first" — `B:15-19`

Gauntlet's `AgentRunner` port describes exactly that: `invoke` / `poll` / `collect` / `cancel` /
`capabilities()` (`gauntlet/crates/gauntlet-ports/src/agent_runner.rs:442-484`), a status enum
`Admitted | Running{last_event_at, payment_so_far} | Completed{result_ref} | Failed{class} | TimedOut |
Cancelled` with pinned post-terminal semantics (`:150-182`), and `RunnerCapabilities
{supports_native_subagents, supports_streaming_events, supports_model_selection, isolation,
result_delivery, max_depth}` (`:199-216`). `D:12-27` is that list plus `unknown`, minus `isolation`,
with `maxDepth` renamed. `E:57-58` says so outright: "`agent_runner.rs:162-211` for the
status/capability vocabulary adopted".

Two consequences. (a) `B:352-359`'s "no existing port expresses …" sentence is what the
implementation brief will lean on to justify a new contract module; it should read EXTRACT/CONSOLIDATE
a Rust port into TypeScript, which also weakens Alternative B's premise — the design refuses the
*implementation* while adopting the *contract* without saying so. (b) The dropped field is the one
that matters: `isolation` is replaced by `toolPolicy: "bridge-only" | "contained" | "unattested"`
(`D:23`), and `G:32-34` concedes "`toolPolicy` is a declaration … it is attribution, not proof".
Gauntlet's own `capabilities()` is honest-by-construction for this axis
(`gauntlet/crates/adapters/gauntlet-adapter-agent-prime/src/runner.rs:1001-1014`:
`isolation: Isolation::None`, commented "Honest by construction (Annex C.2): no containment").
*Recommendation:* relabel G1, and record why the truthful `isolation` axis was replaced by a
self-declared one.

### M3 — `case.escalate` invents Gauntlet's CLI, its status vocabulary, and most of its project

*Affected:* `C:79-98` (C4), `D:98`, `D:101-104`, `E:177-188` (J7), `DECISION-LOG.md:105-129` (§8, §9).

> "P->>G: gauntlet run --project <ref dir>" — `C:92`
> "`status` values are Gauntlet's report words (`settled | unfinished | refused | failed`) prefixed
> with `gauntlet:` to prevent confusion with Workbench settlement." — `D:102-103`

Checked at `gauntlet` `6a7fe6a`:

* `gauntlet run` accepts exactly two flags: `--dry-run` (`crates/gauntlet-cli/src/run_cmd.rs:35`) and
  `--config` (`:43`, `:95`). There is **no** `--project`.
* No terminal word `unfinished`, `refused`, or `failed` exists in a report. Terminal states are
  `Settled | Blocked | BudgetExhausted | Escalated | Aborted | Inconclusive`
  (`crates/gauntlet-domain/src/values.rs:266-268`); the report body is six `StatusCategory` buckets
  (`implemented_and_proven`, `provisional`, `configured_but_untested`, `proposed`, `blocked`,
  `unknown` — `crates/gauntlet-app/src/report/mod.rs:90-120`) plus a `terminal_state` field
  (`:258-320`). D3's four words are invented, so the `gauntlet:` prefix it declares normative maps
  nothing.
* The report goes to **stdout**, interleaved with `human_summary()` and the earlier `run: terminal=…`
  lines (`run_cmd.rs:464-476`, `:430-442`). A Workbench parser cannot treat stdout as JSON without a
  split rule; `D:101` says only "parse report".
* A runnable project is not two files. `gauntlet init`'s template requires `runners.default =
  mock-agent | prime-agent | cli-agent` — "unset refuses dispatch" — plus `[reference]`,
  `[observation] facets`, and `[budget]` ceilings (`crates/gauntlet-cli/src/init_cmd.rs:77-95`). The
  projector must therefore author Gauntlet's *config layer*, including choosing a runner. Choosing
  `prime-agent` from inside Workbench would smuggle a Prime dependency into `case.escalate` — the
  inversion `C:47-49` forbids.

*Recommendation:* re-derive C4 and D3 from `run_cmd.rs` + `report/mod.rs` + `init_cmd.rs` before I6 is
scoped, and decide explicitly who owns `runners.default` in a projected project.

### M4 — `requestedBy` can never match, because Horizon holds the operator's own session

*Affected:* `D:95`, `B:198-212` (G11), `E:79-92` (J8), `G:35-37`.

> "optional `requestedBy?: string` | `requestedBy` not matching the session actor" is refused — `D:95`
> "No operator->Workbench-actor map exists." — `B:201`

Both are true, and together they make the field inert. `connectHorizonComponent` mints **one** session
for the single operator identity (`cognate-workbench/src/capabilities/connectHorizon.ts:41-49`; it
refuses unless `ctx.actor.id === options.operatorActorId`), and `authenticateSessionToken` returns
`{ tenant: identity.tenant, actor: { id: identity.actorId, kind: "user" } }`
(`src/auth/identity.ts:188-196`). Horizon therefore always presents the Workbench operator's actor id.
A Horizon user's `requestedBy` matches only if Horizon hard-codes the Workbench operator's actor id —
the map G11 says does not exist and the disposition refuses to build. J8's positive branch ("Then the
resulting Workbench run carries … and `requestedBy`", `E:84-85`) has no stated way to be reached; only
its negative test is reachable.
*Recommendation:* either build the map (declare the scope increase) or demote the field to
`assertedBy`, record `onBehalfOf` as unverified, and delete the mismatch refusal.

### M5 — The CEP identity I7 carries is caller-asserted, not corroborated

*Affected:* `B:162-178` (G9), `D:95`, `E:79-92` (J8), `A:100-116`.

> "a run that addresses a semantic world must carry `world_ref` + semantic revision, and must refuse
> when it claims to be evaluated without one." — `B:166-167`

The only refusal condition is presence: "`caseContext.kind === "evaluated"` without `worldRef`"
(`D:95`). Cognate's `CaseContext` records `worldRef`, `semanticRef`, `caseId`, `planItemId`, `role`,
`kind` as caller-supplied strings "recorded once, on `run.started`, so it is frozen for the run's life"
(`cognate/packages/runtime-api/src/types.ts:10-38`); nothing validates that the world exists, that
`semanticRef` is in it, or that `role` was ever the run's. `role` is defined as the "SOP role of the
run's principal" (`types.ts:23`) — Workbench has no SOP roles (its Expert carries `specialization`,
`src/experts/types.ts:13-24`), so Horizon must invent a value Workbench then freezes as fact.

Contrast the same package's own standard: `A:108` and `sea-rs`
`crates/sea-forge-authority/src/cep.rs:1356-1364` corroborate 7 fields against RealityTrace and refuse
a mismatch (`:1405-1412`). `A:114-116` states that no verified producer/consumer pair exists for a CEP
envelope entering Workbench — I7 moves the fields across that boundary while adding no corroboration,
and `G:32-34` already has the vocabulary for this ("attribution, not proof") but does not apply it.
*Recommendation:* add a server-side world-registry check in Workbench, or state in J8 that a caller may
assert any world and the evidence will say "asserted".

### M6 — Load-bearing paths that do not exist: every `_wt/…` citation

*Affected:* `A:68,71,143`, `B:33,42-44,309`, `E:92,145,157`, `F:61`, `README.md:50`.

> "Every architectural claim cites a path and line, a commit, or a named test." — `README.md:50`

`ls _wt` in `cognate-workbench` at `f60acb7` answers `No such file or directory`. The content exists on
branches: `src/capabilities/execSpawn.ts` and `src/workbench/safeMode.ts` at
`feat/capability-runtime-t4a` (`bbe6c73`); `src/assignments/assignmentBinding.ts` at
`feat/wp23-assignment-binding` (`668f686`). Every `_wt/cwb-t4a/…`, `_wt/wp23-cwb/…` citation is
therefore unverifiable as written — and these are the extraction *sources* for I3 and I4, the two
increments whose whole claim is "we remove code". `F:60`'s "T3 merged or its diff carried" compounds
it by leaving two candidate baselines.
*Recommendation:* re-cite as `<branch>:<path>:<line>` and name one baseline per increment.

### M7 — `G2`'s "EXTRACT … everything in `execSpawn.ts`" hides four app-owned dependency seams

*Affected:* `B:36-48`, `F:58-69`.

> "**Reusable**: everything in `execSpawn.ts` (containment matrix `worldMatrix`, `decideContainment`,
> posture record, intent/spawned/exited records, long-lease slot), plus `aggregateContainment.ts` and
> `executionPosture.ts`." — `B:36-38`

`SpawnDeps` (`feat/capability-runtime-t4a:src/capabilities/execSpawn.ts:109-127`) requires
`getPosture`, `ledger`, `recordRefusal`, `slots` (with `releaseAll`/`keep` for the boot sweep),
`localHostSpawnAllowed`, and `beforeIntent` (documented as "the LAST step before the intent commit: the
authorization (authorizeCapabilityOp)"). Cognate has no posture-record store, no long-lease slot table,
and no `authorizeCapabilityOp` for spawns (it has `GovernedOperation`, a different shape —
`cognate/packages/runtime-bun/src/governed-operation.ts:16-25`). So I3 is "move one function, then
invent four stores and an authorizer in Cognate", and it separates a spawn-intent ledger from the app
that knows which workspace it belongs to. `F:63-65` names two files.
*Recommendation:* enumerate the five injected dependencies and where each lands in Cognate, or scope I3
as a new component with an explicit store schema.

### M8 — `experts.restore` is specified against a seed function that already behaves differently, and its two clauses contradict each other

*Affected:* `B:235-247` (G13), `DECISION-LOG.md:92-103` (§7), `E:134-145` (J12), `G:25` (O5).

> "**EXTEND** with an operator-only `experts.restore` that re-creates only missing/archived baseline
> ids and never overwrites an existing id. **DELETE** the permanent lock: the boot seed becomes
> idempotent per id rather than per receipt." — `B:241-243`

`cognate-workbench/src/experts/state.ts:57-58` reads `// Seed once, including deliberate user
removals/archives. No restart overwrites customization.` then `if (state.receipts["seed:v1"]) return;`,
and the body already uses `state.experts[id] ??=` and `state.teams["cognate-team"] ??=` (`:59-73`). The
body is *already* per-id idempotent; the receipt is what stops a **deliberately deleted** Expert from
coming back. Deleting the receipt therefore (a) restores nothing archived — an archived Expert is
still a present key (`lifecycle: "active" | "archived"`, `src/experts/types.ts:24`), so `??=` skips it —
and (b) resurrects deleted baseline Experts on every boot, a behaviour change the design lists nowhere
and J12 does not test (it checks only "a renamed custom Expert is untouched", `E:142`). Meanwhile
"re-creates only missing/archived … and never overwrites an existing id" is self-contradictory:
un-archiving *is* writing an existing id. O5 ("verbatim baseline" vs "latest seed", `G:25`) is
unanswered and "verbatim" is impossible for a deleted id whose customized text is gone.
*Recommendation:* keep the receipt, implement `experts.restore` as an explicit operator command over a
frozen baseline snapshot, and record the reversal of "no restart overwrites customization" as a
decision with its own test.

---

## Minor

1. **`design.md:80` counts wrong.** "four capabilities (`rlm.providers`, `rlm.invoke`, `rlm.inspect`,
   `rlm.cancel`, plus operator-only `rlm.reconcile`)" — five are named; `D:8-81` defines five.
2. **`D:30-32` native entry cannot be static.** "The built-in entry is always present: `id:"native"`,
   `maxDepth` = the Team limit … `toolPolicy:"bridge-only"`". A registry listing cannot know a
   per-conversation Team limit (Team limits live in `cognate-workbench/src/experts/types.ts:31-38`),
   and an in-process loop has no bridge for `bridge-only` to describe.
3. **`D2`/`D7` are spliced.** `D:86-90` ends mid-interface (`invoke(req, ctx)` then a bare fence at
   `:90`), and an orphan code block sits at `D:162-169` (`inspect(runId)`/`cancel`/`reconcile?`) with no
   heading. In a document that declares "TypeScript signatures are normative shapes" (`D:3`) this is a
   defect, not formatting.
4. **`E` has two truncated journeys.** `J6` stops mid-sentence at `E:77` ("…assert via the spawn" then
   `J8` begins at `:79`), and `J7`'s negative-test paragraph is orphaned after `J14` at `E:172-175`.
   `design.md:67-68` calls these the journeys that "decide whether this design is real".
5. **`rlm.providers` has an unbounded read cost.** `D:24` requires `executable: {version, sha256}`, and
   `D4.6` (`D:120-121`) records "the executable digest" per terminal outcome. Hashing provider binaries
   per listing call, on every UI render, with no cache or bound rule, contradicts `design.md:133`
   ("provider probing is lazy and bounded").
6. **Three budget vocabularies, no mapping.** `D:46` `{totalTokens?, maxCostUsd?, currency?}`;
   `cognate/packages/runtime-api/src/assignment.ts:19-20` `Currency =
   time|attention|money|risk|recovery` with `limit: number | "unknown"`;
   `cognate-workbench/src/experts/billing.ts:14` `inputUsdPerMillion`/`outputUsdPerMillion`. `D:119`
   states Workbench-only semantics at Cognate level.
7. **Contradictory citations for one claim.** `A:158` cites SEA-Forge evidence corroboration as
   `cep_settlement.rs:716-814`; that path is a *test*
   (`crates/sea-forge-authority/tests/cep_settlement.rs`, 3290 lines) and `:714-720` is a `settle_with`
   helper. The real logic is `crates/sea-forge-authority/src/cep.rs:1356-1412`, which `E:170` cites
   correctly. One of the two must go.
8. **`I4`'s file list omits a dependency.** `B:61`/`F:75` name `real-prime-host`, `prime-host`,
   `prime-recursion`, `prime-context-envelope`;
   `gauntlet/workbench/src/real-prime-host.ts:57` also imports `{ type SpawnPathPolicy,
   validateSpawnRequest } from "./terminal-session"`, described in the header (`:26-28`) as the
   privileged spawn boundary. Extraction scope is five files, and `G:14`'s "imports only fs/path +
   local modules" is understated.

---

## Claims I could not verify

1. **`G:7` #1** — "Prime can be launched so that its only tool path is the MCP bridge". I also found no
   allowlist in the adapter, so absence of evidence stands on both sides. This remains the largest
   unproven dependency of I4, and B2 shows the transport is refused before the question is even reached.
2. **`G:9` #3** — "Cognate 0.9.2 contains `CaseContext`". I confirmed the negative half (installed
   0.8.0 has no `CaseContext` under `node_modules/@godspeedai/cognate-runtime-api/src`) and the pins
   (`cognate-workbench/package.json:19-29` = 0.9.2; `Horizon/package.json:20-27` = 0.10.0); the 0.9.2
   tarball was not fetched, so I1 inherits `A5`'s risk in full exactly as `G:46-48` says.
3. **`G:14` #8** — "Gauntlet's TS host can be extracted without dragging OpenTUI into the package".
   `gauntlet/workbench` has no `package.json` boundary in the tree I read, and the host imports
   `./terminal-session` (minor 8), so the import closure is not four files as stated. Not settled.
4. **`E:170`** — `sea-rs` `cep.rs:1582-1688` as the acceptance path. I verified `CORROBORATED_FIELDS`
   and `corroborate` (`cep.rs:1356-1412`), not `1582-1688`.
5. **`A:106-109`** — the authority/settlement NDJSON assertions are marked **‡** and I did not
   re-derive them. They are load-bearing for `C6`, not for any increment I could block.
6. **Whether anything still imports `pluginRegistry`** (`B:280-295`, `G:44-45`). The design gates the
   deletion on an import inventory that has not been run; I did not run it either, so `DELETE` in
   `B4:342` remains a hypothesis.
7. **`README.md:44-45`** — "`sea-rs` `fde/integration` is one day newer than `main`". The pinned
   `537a50b7` exists but the branch tip is now `6187558a`; the pin is accurate and already stale, and
   `G:10` #4 ("no diff review was performed") still stands. Not resolved here.

---

## What the design gets right (briefly)

* **`A5` and the I0 prerequisite.** Exactly right, and the highest-value observation in the package:
  Workbench pins `0.9.2` and Horizon `0.10.0` (`package.json:19-29`, `:20-27`) while six installed
  packages resolve `0.8.0`, and `CaseContext` is genuinely absent from the installed tree. Sequencing
  I0 first and separately (`F:19-29`) is correct, including "do not paper over it with path aliases".
* **The refusal to add a fourth child-execution mechanism** (`design.md:26-30`) and the resulting KEEP
  list. Every item I checked is real: one-Orchestrator in the mutation path (`experts/state.ts:39-47`),
  the authority intersection (`service.ts:195-199`), the continuation wait and boot reconcile
  (`agentCoding.ts:805-810`, `bridge.ts:96-104`), review independence (`service.ts:316-325`), durable
  pre-dispatch billing with unknown != zero (`billing.ts:1-9`).
* **D4.1/4.6/4.7/4.8** — correlation-key discipline, terminal stability, `unknown` != `failed`, and "no
  silent fallback" are lifted faithfully from Gauntlet's port contract (`agent_runner.rs:428-448`) and
  are the right invariants even though the disposition label is wrong (M2).
* **The `prime`-token absence rule** (`D:167-169`) is a cheap, checkable invariant that makes
  "provider-independent" falsifiable. It should be extended to `agent_runner`/`InvocationStatus`
  vocabulary so M2's unacknowledged adoption is caught mechanically.
* **`G`'s discipline.** Listing unexecuted claims with a settling action and a blocking increment
  (`G:5-15`), and pre-committing a falsifier per decision (`DECISION-LOG.md:3`), is better than most
  design packages carry. The failures above are exactly the ones that fell between `G`'s own rows.
