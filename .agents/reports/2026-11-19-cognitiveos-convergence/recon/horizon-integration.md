# Horizon → Cognate Workbench / SEA-Forge integration recon

Repo: `/home/sprime01/projects/Horizon` — branch `main`, HEAD `0e34a34`
("build(deps): bump @mcp-b/webmcp-polyfill from 5.1.0 to 6.2.0 and use installWebMCP (#54)").
All paths are relative to that repo root unless absolute.

Horizon is a Bun/TypeScript "observer-relative navigation environment" (package.json:44,
`"description": "Observer-relative navigation environment walking skeleton"`). It runs no model of its
own: agent work is delegated to the Workbench over Connect RPC and authority to SEA-Forge over a Unix
socket.

---

## 1. `src/adapters/workshop.ts` — the Workbench seam

### Transport: Connect RPC over HTTP(S), unary, fetch-based. Not MCP, not plain REST.

```ts
// src/adapters/workshop.ts:10-12
import { createClient } from "@godspeedai/cognate-client";
import { createRemoteRuntimeService } from "@godspeedai/cognate-protocol-connect";
import { connectTransport } from "@godspeedai/cognate-protocol-connect-bun";
```

- `connectWorkshop` (src/adapters/workshop.ts:66-70) builds
  `createRemoteRuntimeService(connectTransport(config.url), () => readKey(config.key_path) ?? "")`
  then `createClient({ service, caller })`.
- `connectTransport` is "A fetch-based Connect transport usable from Bun and browsers" with
  `cache: "no-store"` (`node_modules/@godspeedai/cognate-protocol-connect-bun/src/index.ts:99-108`).
- The credential becomes an HTTP header in the remote service:
  ``const headersFor = (caller: Caller) => ({ authorization: `Bearer ${tokenFor(caller)}` })``
  (`node_modules/@godspeedai/cognate-protocol-connect/src/index.ts:557-559`), attached to every unary
  POST (e.g. `:572`, `:596`).
- The file header states the path: "`connectTransport` -> `createRemoteRuntimeService` -> `createClient`,
  the same path the workshop's own tests use" (src/adapters/workshop.ts:4).

### Capabilities/RPCs invoked

Production runtime code invokes exactly **one** Workbench capability:

```ts
// src/adapters/workshop.ts:174
const result = await guarded(access.client.invoke("expert.converse", { id, requestId: id, expertId: request.expertId, workspaceId: request.workspaceId, message: request.message, ...(request.reviews ? { reviews: request.reviews } : {}) }), request.timeoutMs ?? DEFAULT_WORKSHOP_TIMEOUT_MS);
```

`client.invoke(capability, input)` is a generic capability call, not a fixed RPC:
`async invoke(capability, input) { return service.invoke(caller, { capability, input, idempotencyKey: crypto.randomUUID() }); }`
(`node_modules/@godspeedai/cognate-client/src/index.ts:777-779`).

Setup-only calls live in the seed script, not the serving path:
- `"workspace.select"` with `{ path: folder }` — points the Workbench at a local folder (scripts/seed.ts:309)
- `"expert.save"` with `{ expertId, name, specialization, instructions }` (scripts/seed.ts:311)
- `"expert.archive"` with `{ expertId }` (scripts/seed.ts:330)

`expert_id` / `workspace_id` come from Horizon config (src/config/types.ts:62-69; used at
src/app.ts:3167).

### Credential provisioning and storage

- Config shape: `{ url, key_path, expert_id?, workspace_id? }` — "Where the workshop (Cognate Workbench)
  listens and where its Horizon key file lives" (src/config/types.ts:62-69); parsed at
  src/config/load.ts:202-210. The committed config has **no** `[workshop]` table
  (config/horizon.toml:1-22), so the whole seam is optional.
- The key is a **file**, read at call time, never persisted by Horizon: "The session key is read from
  `workshop.key_path` at call time and handed to the transport; it is never logged, stored, exported or
  put in an error message" (src/adapters/workshop.ts:5-7).
- Permission gate: "A key file that other people on this computer can read (any group or world
  permission bit) is refused: the key is a password, so the file must be mode 0600."
  (src/adapters/workshop.ts:24-25; `if ((statSync(keyPath).mode & 0o077) !== 0) return { state: "unsafe" }`
  at :29).
- Companion metadata `${key_path}.json` holds `{ sessionId, expiresAt }` and "no secret"
  (src/adapters/workshop.ts:6-7, read at :49-50).
- Provisioning happens in the Workbench UI, and Horizon's words say so: "Make one in the workshop
  settings." (src/adapters/workshop.ts:73-84); the seed adds "Connect Horizon in the workshop's
  Settings > Governance & evidence first" (scripts/seed.ts:220).
- The key never appears in state, logs, records or exports — encoded as a test
  (test/workshop-key-secrecy.test.ts:26-52).

### Identity Horizon presents to the Workbench

- The `Caller` handed to `createClient` is the single local operator:
  `const caller: Caller = { actor: loaded.operator.principal, tenant: loaded.operator.tenant };`
  (src/app.ts:553), principal `{ id: <identity.operator_ref>, kind: "user" }`
  (src/config/load.ts:345-352; config/horizon.toml:8-12 → `operator_ref = "actor:operator"`).
- **But the caller is not what the Workbench authenticates.** The token supplier ignores it —
  `() => readKey(config.key_path) ?? ""` (src/adapters/workshop.ts:68) — and only the bearer key is
  sent (cognate-protocol-connect/src/index.ts:559). The identity Workbench sees is whatever session the
  key belongs to; Horizon adds no `actor:`/tenant header and no operator→Workbench-actor mapping exists
  on this path (contrast SEA-Forge, §3).
- Timeout: `DEFAULT_WORKSHOP_TIMEOUT_MS = 15_000` (src/adapters/workshop.ts:112), enforced by `guarded`
  (:138-149).
- Idempotency: assignment id and request id are both `assign:<caseId>:<stepId>`
  (``export const assignmentId = (caseId, stepId) => `assign:${caseId}:${stepId}``, src/adapters/workshop.ts:151),
  so a retry replays and a different request for the same step is refused "Assignment identity already
  exists" (:165-176, :118).

---

## 2. `src/capabilities/handoff.ts` — the case-handoff payload

The payload handed to the case keeper is `handoffItem` (src/capabilities/handoff.ts:77-86):

```ts
export function handoffItem(draftId: string, text: string, review = false): Record<string, unknown> {
  return {
    plan_item_id: handoffStepId(draftId),
    name: stepNameOf(text),
    item_kind: "human_task",
    settlement_criteria: { required_artifacts: [], required_declarations: [] },
    markers: { required: false, review },
  };
}
```

| Field | Value | Citation |
|---|---|---|
| `plan_item_id` | `handoff_<uuid with non-alphanumerics stripped>` | src/capabilities/handoff.ts:67, 80 |
| `name` | first 200 chars of the draft, whitespace-collapsed, `…`-truncated | :72-75 |
| `item_kind` | literal `"human_task"` | :82 |
| `settlement_criteria` | `{ required_artifacts: [], required_declarations: [] }` — **always empty** | :83 |
| `markers` | `{ required: false, review }` | :84 |

Envelope around it (src/app.ts:3151-3157): `{ case_id, item, request_id, dry_run: false,
preconditions: { records: [{ ref, expected_digest }] } }`, the digest being the dry run's
`plan_precondition` (src/app.ts:3136-3145). `request_id` = `handoff:<draftId>`
(src/capabilities/handoff.ts:66), keyed to a durable draft so a retry re-uses the number.

Draft state (Horizon's own record, collection `horizon-ui-state`, id `draft:<tenant>:<operator>`):
`{ draftId (UUID), text, mode: "write"|"find"|"ask"|"handoff" }` (src/capabilities/handoff.ts:11-19,
27-38; `UI_STATE_COLLECTION` at src/capabilities/workspace-records.ts:97). Text cap
`DRAFT_TEXT_MAX = 2_000` (:10).

### Is a CEP profile or `world_ref` carried?

**No.** The payload is those five keys: no profile, no `world_ref`, no case template ref, no
`model_route`, no authority envelope. Search:

```
$ rg -n 'world_ref|profile|cep' src/capabilities/handoff.ts src/adapters/sea-forge-case.ts
src/adapters/sea-forge-case.ts:76:  return `The case keeper is not accepting connections.${tail}`;   # substring match only
```

`world_ref` exists elsewhere — the semantic adapter and the read-only FDE adapters
(src/adapters/fde-read.ts:110-121, 198) — but never in a handoff item.

### Authority / provenance / uncertainty

- **Authority**: not in the item. The case keeper records the *proposer* from the wire envelope
  (`on_behalf_of` / `actor`): "The case keeper records the proposer inside the step, so the preview is
  made for the same operator as the real add" (src/adapters/sea-forge-case.ts:230-232).
  `policy: "policy.yaml"` is added by the adapter, not the payload (:21, :235).
- **Provenance**: partial, only via the request id (`handoff:<draftId>`) and the pinned plan digest
  precondition. No citation, episode ref or full text: "The whole text is not kept in the plan: the plan
  holds names, not paragraphs" (src/capabilities/handoff.ts:71).
- **Uncertainty**: dropped. `settlement_criteria` is deliberately empty and nothing like confidence,
  assumption or "known afterwards" travels with the step. (Contrast the desk's certainty vocabulary
  `CERTAINTIES = ["certain", "follows from the rules", "expected", "unknown", "only known afterwards"]`
  at src/ui/desk/model.ts:73 — it never reaches the case keeper.)
- The `review` flag changes semantics: a review step points the Workbench at another assignment so
  "the workshop then starts the reviewer without the author's memory" (src/adapters/workshop.ts:160-161;
  `reviews: assignmentId(caseId, reviewStepId)` at src/app.ts:3167).

---

## 3. `src/adapters/sea-forge-case.ts` — the SEA-Forge seam

### Transport: raw NDJSON over a Unix domain socket (not HTTP, not the SDK)

```ts
// src/adapters/sea-forge-wire.ts:9
export async function seaForgeCall(socketPath: string, request: Record<string, unknown>, timeoutMs = 5_000)
// :25-31
Bun.connect({ unix: socketPath, socket: { open(socket) { socket.write(`${JSON.stringify(request)}\n`); } ...
```

One JSON line out, one back, socket closed on every outcome; default 5 s timeout; typed errors
(`SEA-Forge did not answer in time`, `closed the connection without an answer`, `answered with invalid
JSON`) — src/adapters/sea-forge-wire.ts:9-56.

Scope, per the header: "The raw SEA-Forge NDJSON call, for the verbs the Cognate SDK client
(`SeaForgeAuthority`) does not carry: the case keeper (`case_*`, `human_task_*`), the ledger reads and the
 event follower. The SDK client owns every governance verb (decide, approvals, closure, settle)"
(src/adapters/sea-forge-wire.ts:1-6).

Envelope per call (src/adapters/sea-forge-case.ts:137-140):

```ts
reply = await seaForgeCall(keeper.socketPath, { ...request, actor: keeper.service, ...(write && keeper.operator ? { on_behalf_of: keeper.operator } : {}) }, keeper.timeoutMs ?? 5_000);
```

Reads go as the service actor; writes additionally name the operator (`{ actor_id, role: "operator" }` —
src/adapters/sea-forge-config.ts:60-64), because "a requester cannot approve its own escalation" (:59).

### Verbs exposed by this adapter

| Function | `verb` | Payload | Citation |
|---|---|---|---|
| `preflight` | `case_preflight` | `{ template_ref, params }` | src/adapters/sea-forge-case.ts:180-192 |
| `commit` | `case_commit` | `{ template_ref, params, request_id, preconditions }` | :194-199 |
| `decide` | `approval_decide` | `{ case_id, approval_id, decision: "approve"\|"reject", policy, note?, request_id, preconditions }` | :201-213 |
| `addItem` | `case_add_item` | `{ case_id, item, request_id?, dry_run, policy, preconditions?, on_behalf_of?, external_ref? }` | :215-242 |
| `listCases` | `case_list` | `{}` → `{ cases[], unreadable[] }` | :257-270 |
| `caseSteps` | `case_get_horizon` | `{ case_id }` → `{ case_state, items[] }` | :282-290 |
| `humanTaskComplete` | `human_task_complete` | `{ case_id, item_id, justification, policy, request_id }` | :296-298 |
| `caseDigest` | `case_get_overview` | `{ case_id }` → `{ digest, case_state, template_ref }` | :301-305 |
| `entryOptions` | `case_entry_options` | `{}` → `{ templates[] }` | :313-318 |
| `advance` | `case_advance` | `{ case_id, policy, request_id }` | :324-326 |
| `approvalList` | `approval_list` | `{ case_id? }` → `{ approvals[], unreadable }` | :339-349 |
| `followEvents` | `events_get_range`, `events_subscribe` | `{ from_cursor?, limit }` | src/adapters/sea-forge-events.ts:80, 117 |

Policy is pinned in code: `export const CELL_POLICY = "policy.yaml"` — "The policy file the cell writes
(ignite) and the case keeper must be told to read; its own default name is different"
(src/adapters/sea-forge-case.ts:20-21).

Preconditions are digests read from the keeper, never computed locally:
`Preconditions = { policy_bundle_digest?, records: RecordDigest[] }`,
`RecordDigest = { ref, expected_digest }` (:23-30); "Digests are read from the case keeper, never
computed here" (:13-14).

### World operations

Not in `sea-forge-case.ts`. The **governance/world** verbs belong to the Cognate SDK's
`SeaForgeAuthority`, which Horizon instantiates rather than re-implements
(`new SeaForgeAuthority({ socketPath: delegationSocket, actor: serviceActor, requestIdPrefix: "horizon", … })`
at src/app.ts:568-572). That client sends:

- `authority_request` with `{ envelope, request_id, actor }`, decided per operation *and per `worldRef`*
  (`node_modules/@godspeedai/cognate-runtime-bun/src/sea-forge-authority.ts:324-333`)
- `authority_transitions` (:355), `authority_evidence` (:388), `authority_settle` (:406),
  `authority_approvals` (:416)

World identity on Horizon's side: `world_ref = "world:horizon@sha256:0a10c20d…"` (config/horizon.toml:5),
loaded by `loadWorld` (src/app.ts:551); the live test records that both sides derive the same world —
"The world SEA-Forge registers is Horizon's own `domain/` source set" (test/J02-live.test.ts:8-9;
`worlds: [{ name: "horizon", base: "worlds", entry: "interaction-model.sea", files }]` at :94).

Outcome model (never throws for an expected state): `ok | stale | refused | unreachable`
(src/adapters/sea-forge-case.ts:48-52), `unreachable.why ∈ socket_missing | connection_refused |
timeout | garbled | closed | other` (:40, classified :56-64), `maybe_written` only for timeout/closed/other
on writes (:143). Error classes get their own words — `authority_denied`, `missing_config_error`,
`request_id_reused`, `invalid_decision`, `durable_locator_required` (:113-129).

### Authority/containment design around this seam

- Governed mode consumes SEA-Forge's authority decision and Horizon **fails closed at startup** while
the owner has not published it:

```ts
// src/app.ts:556-559
if (configured.mode === "sea-forge" && !options.unpublishedAuthorityForTests && interfacePublicationStatus(AUTHORITY_DECISION_INTERFACE) !== "published") {
  throw new Error("SEA-Forge governance is refused: sea_forge.authority_decision is specified_not_published, so Horizon cannot read a committed decision. ...");
}
```

- That interface and two siblings are recorded in `contracts/identity/v1/external-interfaces.json`
  (`sea_forge.authority_decision` :27-39, `sea_forge.action_grant` :319, `sea_forge.human_entitlement`
  :411), owner `GodSpeedAI/SEA-Forge`, `repository: https://github.com/GodSpeedAI/SEA-Forge.git` (:29-30).
- Delegation is identity-checked: a session operator's commit goes to SEA-Forge as the gateway acting
  `on_behalf_of` the operator's mapped actor; an unmapped operator is refused "rather than falling back to
  another identity" (src/adapters/delegating-authority.ts:1-8, 36-39), and the gateway identity may never
  act as the human (:40).
- Operator ref → SEA-Forge actor is one validated mapping (`actor:<id>` → `<id>`, `/^[A-Za-z0-9_-]{1,64}$/`)
  — src/adapters/sea-forge-config.ts:19-29.
- Environment contract: `COGNATE_GOVERNANCE=sea-forge`, `COGNATE_SEA_FORGE_SOCKET`,
  `COGNATE_SEA_FORGE_ACTOR`, `COGNATE_SEA_FORGE_OPERATOR`; a partial/inconsistent configuration is a
  startup error, "never a guess" (src/adapters/sea-forge-config.ts:45-52).

---

## 4. `src/runtime/create.ts` — runtime composition

`createHorizonRuntime(config, world, options)` (src/runtime/create.ts:337-386) makes one `createRuntime`
call:

```ts
// src/runtime/create.ts:369-384
const runtime = await createRuntime({
  store: options.store === undefined ? storePath : { store: options.store },
  policy, actions,
  agents: options.agents ?? [healthAgent],
  components: [semanticWorldComponent(world), contextReferenceComponent(config.context_kernel, …), ...(options.components ?? [])],
  projections: [ { projection: navigationContextProjection, public: true }, { projection: operationalObjectsProjection, public: true } ],
  tickMs: 5,
  ...(options.governance ? { governance: options.governance } : {}),
  ...(options.outboxFor ? { outboxFor: options.outboxFor } : {}),
});
```

**Mandatory** (no branch removes them):
- a durable SQLite store path — `assertStoreBoundary` refuses `:memory:` and `postgres` under
  `local_operator` (:323-335)
- the authorization `Policy`: an allow-list of exactly three capability ids — `horizon.semantics.world`,
  `horizon.navigation.intent.commit`, `CONTEXT_REFERENCE.id`; everything else is
  `horizon.local.default-deny` (:28-42)
- the `ActionPolicy`: only `run.start` / `run.cancel`, and only for `user` or `agent` actors (:44-53)
- `semanticWorldComponent(world)` and `contextReferenceComponent(...)`: "Retrieval is registered at boot
  whether or not a Context Kernel is configured: the capability answers `absent` itself, so a route has
  one path and no configuration branch (REQ-STACK-010)" (:374-376)
- both projections, published (`public: true`)
- `healthAgent` as the fallback agent list (:373)

**Optional / injected by the application** (src/runtime/create.ts:340-366): `storePath`, `components`,
`agents`, `contextAdapter`, `governance`, `outboxFor`, `store`. src/app.ts:646-661 injects
`components: [navigationIntentComponent(frames)]`,
`agents: [structuralObservationAgent(frames), navigationCommitAgent(...)]`, `governance`, and
`outboxFor: observationOutbox(...)` when RealityTrace is configured.

Read-only composition is a separate guarded path: `openReadOnlyStore` (:306-321) refuses a missing store,
refuses a store older than the build's schema version, and proxies every write method to a rejection
(`ReadOnlyStoreWrite` :81-87, `WRITE_METHODS` :62-79).

### Is there expert/RLM/agent *execution* in Horizon?

**No.** Horizon contains observation/commit agents, not model runners:

- `src/agents/health.ts` — `healthAgent` / `structuralObservationAgent`, which call
  `context.invoke("horizon.semantics.world", {})` (src/agents/health.ts:30)
- `src/agents/navigation-commit.ts` — `navigationCommitAgent`, which invokes
  `NAVIGATION_INTENT_COMMIT.id` (src/agents/navigation-commit.ts:70)
- `src/verifiers/intent-recorded.ts` — an intent-recorded verifier

All model work is delegated:
- step execution → Workbench `expert.converse` (§1); the desk says "An agent runs only once the workshop
  has a model chosen (workshop Settings > Models)" (scripts/seed.ts:317)
- authority → SEA-Forge (§3)
- retrieval → Context Kernel (`query_reference`, `get_status` over JSON-RPC lines;
  src/adapters/context-kernel.ts:1-28), behind the `horizon.context.query_reference` capability
- research → an optional Python contract (`observeResearch`, src/adapters/research.ts:80-120) whose
  absence is "a *degraded typed observation*, never a startup-wide prerequisite" (:6-12)

---

## 5. `src/ui/desk/` — operator-facing concepts

Five views, each with a plain-language question (src/ui/desk/model.ts:12-18):

| id | label | asks | key |
|---|---|---|---|
| `today` | Today | "What am I doing?" | 1 |
| `conversation` | Conversation | "What has been done, and what is waiting for me?" | 5 |
| `explore` | Explore | "What is here?" | 2 |
| `decide` | Decide | "What needs me?" | 3 |
| `work` | Work | "What is running and how did it go?" | 4 |

View components: `src/ui/desk/views/{Today,Work,Decide,Explore,Home,Cost}.tsx`. Shell and parts:
`Desk.tsx`, `Nav.tsx`, `StatusBar.tsx`, `Conversation.tsx`, `Composer.tsx`, `Preview.tsx`, `Inspector.tsx`,
`DecisionCard.tsx`, `NextDecision.tsx`, `Panels.tsx`, `Answers.tsx`, `NoteEditor.tsx`,
`ComparisonView.tsx`, `LabelControl.tsx`, `EpisodeEvidence.tsx`, `RemotePanel.tsx`, `Commitment.tsx`,
`StateWord.tsx`, `Words.tsx`, plus a block grammar under
`src/ui/desk/blocks/{index,ask,data,read,viz,guard,ctx,shared}.tsx`.

### Concepts and the data behind them

- **Cases** — `DeskCase = { case_ref, purpose, opened_at, source?: "horizon" | "case keeper", state? }`
  (src/ui/desk/model.ts:49-58). Sources: Horizon's own case notebook (`horizon-cases`,
  src/capabilities/workspace-records.ts:10) or the SEA-Forge case keeper via `listCases` / `caseSteps`
  (src/app.ts:1743-1770, 1895-1910).
- **Questions** — agent-authored panel cards the operator answers. `Panels.tsx` renders "what an agent put
  in front of you … You can answer its questions, pin it so it stays open, and close it"
  (src/ui/desk/Panels.tsx:1-9); its IO is
  `answer({panel, ask, outcome: "answered" | "deferred" | "cancelled", value?, note?, version, owner_rev})`
  (:14-19). Storage: `horizon-panels`, `horizon-panel-answers`, `horizon-agreed-answers`
  (src/capabilities/panel-records.ts:14-15; src/capabilities/panel-answers.ts:21, 354). A question's owner
  is the agent key id; "The display name is a label the agent sends; it never decides who owns a question"
  (src/desk/agent-keys.ts:1-8).
- **Moves** — a `WorkspaceMove` carrying three separately-stated claims `decided` / `recorded` /
  `observed` (`ClaimRows`, src/ui/desk/views/Home.tsx:18-22; receipt :120-170; "A move in the
  conversation: what decided, what recorded and what observed it. Anything missing says 'Not seen yet',
  never 'no'" — src/ui/desk/Inspector.tsx:192-210). Type at src/ui/workspace/model.ts:74.
- **Decisions** — Decide is "What needs you, oldest first. Nothing is decided until you confirm a
  preview" (src/ui/desk/views/Decide.tsx:27-33). Cards pin digests as `expected_digest` preconditions
  (src/ui/desk/decide-build.ts:23, 63, 113, 186) and post to `POST /desk/decide`. "Next decision" choices
  only *record an intent*: "Choosing records a request. It does not approve, run or sign off anything."
  (src/ui/desk/NextDecision.tsx:4-24). Horizon's own holds/decisions are `horizon-holds` /
  `horizon-decisions` (src/capabilities/decisions.ts:8-9).
- **Expected vs observed** — comparison rows are
  `["expected", "observed", "difference", "rules in force", "allowed by", "record intact", "accepted", "journey score"]`
  (src/ui/workspace/compare-model.ts:14). The `expected` row is honest about the gap: `NOT_RECORDED` with
  reason "the expectation is not written down in the record before the run yet (case steps arrive in
  stage 4)" (:204); `observed` derives from declared checks (:208-221).
- **Status bar / parts** — `partsOf` reports `Rules`, `Cases`, `Records`, `Agent work`, `Reference library`
  as "connected" / "Not connected" only, with key-expiry warnings (src/ui/desk/model.ts:218-235).

Desk HTTP surface (src/app.ts:3437-3536, `workspace*` at :3499-3780): `/desk`, `/desk/state.json`,
`/desk/search` (GET+POST), `/desk/conversation`, `/desk/conversation/discard`, `/desk/note`, `/desk/draft`
(GET+POST), `/desk/start`, `/desk/panels/op`, `/desk/panels/catalogue`, `/desk/handoff/preview`,
`/desk/handoff`, `/desk/decide.json`, `/desk/decide`, `/desk/lesson`, `/desk/lessons.json`,
`/desk/work.json`, `/desk/next.json`, `/desk/first-move.json`, `/desk/setup-guide`, `/desk/remote.json`,
`/workspace`, `/workspace.json`, `/workspace/approvals`, `/workspace/economics`, `/workspace/compare`,
`/workspace/inspect`, `/workspace/intent`, `/workspace/case`, `/workspace/export`, `/workspace/label`,
`/workspace/presenting`, `/workspace/remote*`. Agent-facing door: `/agent/panels`,
`/agent/panels/catalogue`, `/agent/panels/answers` (src/app.ts:2576-2593, 3307).

---

## 6. External agent CLI / recursive model runner — absent

Search 1 (subprocess primitives, `src/` + `scripts/`):

```
$ rg -n 'Bun\.spawn|child_process|spawnSync|execSync|"claude|codex|opencode|gemini-cli|subprocess' --include='*.ts' --include='*.tsx' src/ scripts/
src/adapters/realitytrace-economics.ts:12,40   spawnSync(<realitytrace bin>, …)
src/adapters/context-kernel.ts:21             spawn  (Context Kernel JSON-RPC server)
src/adapters/research.ts:92,94                Bun.spawn([...research_command, "--health"])
src/adapters/run-child.ts:43                  Bun.spawn   (generic bounded child runner)
src/adapters/settlement.ts:152                Bun.spawn([bin, "ledger", "export", …])
src/capabilities/desk-lock.ts:13              spawn       (lock holder)
scripts/*                                     bun test / gh / id / getent / ss / bun build only
```

Search 2 (capability invocations that are not Horizon's own):

```
$ rg -n 'invoke\("' scripts/*.ts src/ | grep -v 'horizon\.'
scripts/seed.ts:309  "workspace.select"
scripts/seed.ts:311  "expert.save"
scripts/seed.ts:330  "expert.archive"
src/adapters/workshop.ts:174  "expert.converse"
```

Search 3 (recursion / sub-agent vocabulary):

```
$ rg -ni 'recursive|rlm|deep research|sub-agent|subagent|expert\.' --include='*.ts' --include='*.tsx' src/
… only mkdirSync({recursive}), schema walkers, and the expert.converse call above …
```

Every spawned process is RealityTrace, `git`, the Python research contract, the Context Kernel server, or
a build/test tool. `runChild` is a containment utility for those (own process group, byte cap,
kill-on-return — src/adapters/run-child.ts:1-9, 43-70). **No agent CLI is launched and no recursive
model runner exists.** Horizon's only model-adjacent lever is the Workbench capability call.

---

## 7. Persistence: what and where

### Primary store — one SQLite file

- Path: `runtime_store = ".cognate/app.sqlite"` (config/horizon.toml:6); `HORIZON_STORE` overrides it
  (src/config/load.ts:382-387); desktop mode defaults to `$XDG_DATA_HOME/horizon` + `app.sqlite`
  (src/desktop.ts:34, 63).
- Opened by `openSqliteStore({ path })` from `@godspeedai/cognate-store-sqlite`
  (src/runtime/create.ts:8, 121, 609).
- Two independent schema/version layers:
  1. **SQLite store schema**: a `schema_versions` table, read to compare the store's version against the
     build's (src/runtime/create.ts:98-128, 306-311); a store older than the build is refused for
     read-only surfacing — "the desk's database is at schema version N, older than this build" (:310).
     The opener's migrations are not exported, so the build version is probed by creating a scratch store
     (`void openSqliteStore({ path: probe }).close()`, :116-128).
  2. **Desk record migrations**: versions 1..8 with a `marker:<scope>` record in
     `horizon-desk-migrations` (src/capabilities/desk-migrations.ts:2-13; list at :29-41 — answer states,
     answers from before the upgrade, question states, sealed questions, reconciled identities, deferred
     answers, operator scope (layout), conflicts from deferred answers). Until the run finishes "the desk
     refuses every change" (:5-6; src/app.ts:3306).
- Record collections (each a literal at its declaration site): `horizon-cases`, `horizon-onboarding`,
  `horizon-ui-state` (src/capabilities/workspace-records.ts:10-11, 97); `horizon-notes`,
  `horizon-note-revisions`, `horizon-note-placements`, `horizon-tags` (src/capabilities/notes.ts:11-14);
  `horizon-panels`, `horizon-desk-board` (src/capabilities/panel-records.ts:14-15); `horizon-panel-answers`,
  `horizon-agreed-answers` (src/capabilities/panel-answers.ts:21, 354); `horizon-desk-requests`
  (src/capabilities/desk-requests.ts:15); `horizon-conversation` (src/capabilities/conversation.ts:11);
  `horizon-holds`, `horizon-decisions` (src/capabilities/decisions.ts:8-9); `horizon-refusals`
  (src/capabilities/refusals.ts:14); `horizon-lessons` (src/capabilities/lessons.ts:9); `horizon-labels`
  (src/capabilities/labels.ts:9); `horizon-pending`, `horizon-pending-heads`, `horizon-pending-transitions`
  (src/capabilities/pending-confirmations.ts:21-23); `horizon.navigation-intent-index`,
  `horizon.navigation-intent-audit` (src/capabilities/navigation-intent.ts:102-104);
  `horizon-navigation-intent-suspension` (src/capabilities/contract-upgrade.ts:21);
  `horizon.navigation-intent-settlement` (src/capabilities/navigation-settlement.ts:12);
  `horizon.capacity-ledger`, `horizon.store-meta` (src/capabilities/capacity-ledger.ts:14-15);
  `horizon-reconciliation` (src/capabilities/reconciliation.ts:7); `horizon-desk-lock`
  (src/capabilities/desk-lock.ts:17); `horizon-remote-devices`, `horizon-remote-audit`
  (src/capabilities/remote-records.ts:9-10); `horizon-remote-enrollments`,
  `horizon-remote-enrollment-approvals`, `horizon-remote-p3-decisions` (src/remote/enrollment.ts:34-38);
  `horizon-desk-migrations` (src/capabilities/desk-migrations.ts:13). Identity collections are owned by
  `@godspeed/identity` in the *same* store — "same store, same transactions, no second database"
  (src/remote/identity-store.ts:1-7).
- SQLite tables read directly for surfacing: `events` + `payloads`, `event_streams`, `outbox`, `blobs`,
  `projection_checkpoints`, `projection_state` (src/runtime/create.ts:133, 199-256, 288).

### JSON / file stores (mode 0600 unless noted)

- Launch secret: config/horizon.toml:11-13 → `.cognate/local-operator.secret`; validated by
  `/^[0-9a-f]{64}$/` and written mode 0600 (src/config/load.ts:301-323). A read-only open takes it as-is
  and never rotates it (:303-307).
- Agent keys: `agent.key` (0600, written at src/config/load.ts:443-445) plus a **registry** `agents.json`
  beside it holding SHA-256 hashes, never the keys (`AGENTS_REGISTRY_SCHEMA = "horizon.agents/1"`,
  src/desk/agent-keys.ts:9-24, 89; path at src/config/load.ts:403, 411-413).
- Presenting key: `<deskRoot>/secrets/presenting.key`, 32 random bytes, mode 0600, `flag: "wx"`
  (src/app.ts:1625-1648).
- Desk working dirs: `cache/`, `exports/` under `deskRoot` (default `dirname(runtime_store)/desk`) —
  src/app.ts:1625-1628.
- No `.sql` migration files and no ORM schema files exist. The "schemas" are the JSON/YAML contracts under
  `contracts/identity/v1/` (`records.json`, `params.yaml`, `policy.yaml`, `operations.yaml`,
  `external-interfaces.json`, `platform-claims.yaml`, `issuer-profile.yaml`, `a3-mechanism.yaml`) and the
  DomainForge `.sea` semantic source set (`domain/interaction-model.sea`, cf. cognate.config.ts:1-11).

---

## 8. Tests for the Workbench connection and the SEA-Forge case adapter

### Workbench

| File | What it pins |
|---|---|
| `test/workshop-assign.test.ts` | capability string is `"expert.converse"`; `id` and `requestId` are both `assign:case_1:step_1`; the same request replays (one assignment); a different request for the same step → `already_assigned` ("already has an agent assigned"); `not_connected` sends nothing and returns `refused` (assertions :26-56) |
| `test/workshop-key-secrecy.test.ts` | a real `Bun.serve` stub records the `authorization` header and must see exactly `` `Bearer ${TOKEN}` `` (:44); the token must not appear in `keyState()` JSON, the connection object, the error from a 401, label records, or an export (:34-52) |
| `test/adversarial-workshop.test.ts` | Annex J.1: "Each case drives a stand-in workshop (or a bad key file) and checks the rule: Horizon says the honest state in words, never shows stale data as current, never loses or doubles a write, never leaks the key" (:1-5); real HTTP stubs plus `chmodSync` for the unsafe-key case (:6-28) |
| `test/seed-workshop.test.ts` | the seed's `workspace.select` / `expert.save` / `expert.archive` path against a stand-in workshop, and `[workshop]` table rewriting in `horizon.toml` (:1-30) |

### SEA-Forge case adapter

| File | What it pins |
|---|---|
| `test/adversarial-case-keeper.test.ts` | "every way the case keeper can fail, against a fake Unix socket server. The rule each test proves: Horizon says the honest state in words, never shows stale data as current, and never loses or doubles a write" (:1-6); imports every adapter verb (`addItem, advance, approvalList, commit, decide, humanTaskComplete, listCases, preflight, reasonOf, STALE_WORDS`) plus `followEvents`, driving `fakeCaseKeeper(script)` with a scripted reply per request (:8-30) |
| `test/sea-forge-wire.test.ts` | "the bytes Horizon sends through the SDK equal the pre-migration format … Captured as the raw line on a scripted Unix-socket server" (:1-5); `buildAuthorityRequest` + `delegatingAuthority` envelope equality (:6-30) |
| `test/handoff.test.ts` | stage-4 handoff against a stand-in case keeper plus the real desk app: dry-run preview, plan digest as precondition, `on_behalf_of` proposer participating in the digest, request-number replay, and Workbench assignment via an injected `Access` (:1-45) |
| `test/J02-live.test.ts` | opt-in live proof against a **real** `sea-forge-server` (`SEA_FORGE_SERVER_BIN`): allow / deny / offline / escalate / settle — "Horizon never grants authority itself … a mocked SEA-Forge would prove nothing", same `world_ref` on both sides (:1-20, 31) |
| `test/J02r2-live.test.ts` | the second live governance round |

Fixtures: `test/support/fake-case-keeper.ts`, `fake-case-keeper-stateful.ts`, `sea-forge-raw.ts`,
`desk-app.ts`, `p3-authority-double.ts`.

---

## 9. Dependencies on `@godspeedai/cognate-*` and on `sea-rs` / SEA-Forge crates

`package.json:18-41` (all `0.10.0` unless noted):

```
@godspeed/identity                       file:./.cognate/vendor/godspeed-identity-0.1.0.tgz
@godspeedai/cognate-cli                  0.10.0
@godspeedai/cognate-client               0.10.0
@godspeedai/cognate-domainforge          0.10.0
@godspeedai/cognate-events               0.10.0
@godspeedai/cognate-kernel-api           0.10.0
@godspeedai/cognate-profile-harness      0.10.0
@godspeedai/cognate-projections          0.10.0
@godspeedai/cognate-protocol-connect     0.10.0
@godspeedai/cognate-protocol-connect-bun 0.10.0
@godspeedai/cognate-provider-local       0.10.0
@godspeedai/cognate-react                0.10.0
@godspeedai/cognate-realitytrace         0.10.0
@godspeedai/cognate-runtime-api          0.10.0
@godspeedai/cognate-runtime-bun          0.10.0
@godspeedai/cognate-store-sqlite         0.10.0
```

The composition-critical subset is asserted by `REQUIRED_COGNATE` (src/runtime/required-cognate.ts:8-14:
`cognate-cli`, `cognate-domainforge`, `cognate-runtime-bun`, `cognate-projections`,
`cognate-kernel-api`), checked by `scripts/check-boundaries.ts` and `test/app.test.ts` per that file's
comment (:1-7).

**No SEA-Forge Rust crates.** The only Rust manifests are `src-tauri/Cargo.toml` and
`src-tauri/supervisor/Cargo.toml`; dependencies are `tauri = 2.12.5`, `tauri-build = 2.7.4`,
`serde_json = 1.0.154`, `horizon-supervisor` (path), `flate2` on Linux — nothing from `sea-rs` or
`SEA-Forge` (src-tauri/Cargo.toml:14-22). SEA-Forge is a *runtime* dependency reached over a socket,
named only by env `COGNATE_SEA_FORGE_SOCKET` / `_ACTOR` / `_OPERATOR` and `COGNATE_GOVERNANCE`
(src/adapters/sea-forge-config.ts:13, 45-51), and declared as an external interface in
`contracts/identity/v1/external-interfaces.json:27-39`.

Observation (not investigated further): installed `node_modules` copies of several Cognate packages are at
`0.8.0` (`node_modules/@godspeedai/cognate-client/package.json` `"version": "0.8.0"`;
`cognate-protocol-connect` likewise) while `package.json` declares `0.10.0`. The vendored tree appears
stale relative to the manifest, so vendored line numbers are a weaker citation than Horizon's own sources.

---

## 10. Workbench absent/unreachable: refusal or degradation?

**Degradation, made explicit — with one refusal at the exact point where the operator asked for an agent.**

1. **Not configured at all** → `workshopAccess` returns a typed "not connected" with words:
   `if (config === undefined) return { kind: "not_connected", why: "not_set_up", words: "Agent work is not connected: the workshop is not set up." }`
   (src/adapters/workshop.ts:96-103). Nothing is sent.
2. **Key missing / empty / unsafe / expired** → `keyState` classifies it (src/adapters/workshop.ts:43-58)
   and `keyWords` gives the remedy — "there is no access key yet. Make one in the workshop settings." /
   "…can be read by other people on this computer, so Horizon will not use it…" (:73-84).
   `connectWorkshop` returns `null` rather than a half-configured client (:66-70).
3. **The desk keeps working.** The status bar reports `Agent work: Not connected` plus any key warning
   (src/ui/desk/model.ts:224-234); health parts are reported the same way (src/app.ts:1795-1797).
4. **Handoff without an agent still completes.** The step is added to the case keeper and started; the
   agent result is simply not attached and the reply says so:
   `assigned = result.kind === "ok" ? { done: true, words: "Given to an agent." } : { done: false, words: result.words }`
   (src/app.ts:3166-3168), ending "It was not given to an agent: …" (:3171).
5. **Handoff *asking* for an agent refuses before anything is sent, 409, no side effects:**
   ``if (wantsAgent && !agent.offered) return fail(409, "not_connected", `${agent.words} Nothing was sent.`)``
   (src/app.ts:3140); `deskAgent()` phrases it "Agent work: Not connected. …" (:3176-3179). `assign` also
   refuses defensively when access is not ready (src/adapters/workshop.ts:171), and the test proves nothing
   is sent (test/workshop-assign.test.ts:52-56).
6. **Reachable but failing** → classified, never guessed (src/adapters/workshop.ts:115-136): 401/expired →
   `unreachable`/`key_rejected`, `maybe_done: false`; timeout → `unreachable`/`slow` with, for a write,
   `maybe_done: true` and "The work may or may not have been handed over. Try again: the same step is never
   given out twice." (:127-128); 403 → `refused`; "configure a model" / "active expert not found" /
   `failed_precondition` → `refused` with a remedy (:130-134); anything else → `unreachable`/`down` (:135).
   Error text is read for its *kind* only and never copied into the words, since it "could echo a key" (:114).
7. **Comparison links** degrade to words: `workshopLink` returns `null` without a URL
   (src/ui/workspace/compare-model.ts:179-186) and the view renders
   `<span data-workshop-link-absent>No workshop link: {reason}.</span>` (src/ui/desk/ComparisonView.tsx:30-38;
   model at compare-model.ts:245).

Contrast SEA-Forge, stricter about *authority* but equally honest about *absence*: no socket →
`caseKeeper === null` (src/app.ts:1631-1634) → writes fail 503 "The case keeper is not set up, so this
cannot be decided." (src/app.ts:2110, 2138) while reads return empty-with-words (src/app.ts:1743-1746); a
live-but-broken keeper yields 502 plus its own words and a Details disclosure (src/app.ts:3101-3102;
src/adapters/sea-forge-case.ts:101-112). And governed authority cannot degrade at all — it refuses startup
while `sea_forge.authority_decision` is unpublished (src/app.ts:556-559).

---

## Cross-cutting notes for the convergence design

1. **Two different trust models on the two seams.** SEA-Forge: Horizon carries a mapped actor (`actor:<id>`
   → gateway actor), separates service from operator (`on_behalf_of`), and forbids a requester approving its
   own escalation. Workbench: identity is a single bearer session key; Horizon's `Caller` is not presented
   at all, and no operator→Workbench-actor map exists.
2. **Horizon owns no execution.** Its only "run" primitives are `run.start` / `run.cancel` authorization
   (src/runtime/create.ts:44-53) and in-process observation agents. Any expert/RLM capability added in the
   convergence must either live behind `expert.converse`-style Workbench capabilities or arrive as a new seam.
3. **The handoff item is a thin, authority-free envelope.** No profile, no `world_ref`, no expectation, no
   uncertainty; provenance rides only on `request_id` plus a pinned plan digest.
4. **Idempotency is uniform and deliberate** (`handoff:<draftId>`, `assign:<case>:<step>`, `advance:<sentId>`,
   keeper `request_id` replay, `maybe_written` honesty) — an invariant worth preserving if the seams are
   re-cut.

---

VERIFIED

Verified by reading the cited Horizon sources at HEAD `0e34a34`, plus the vendored
`@godspeedai/cognate-client` / `cognate-protocol-connect` / `cognate-protocol-connect-bun` /
`cognate-runtime-bun` sources under `/home/sprime01/projects/Horizon/node_modules/` for wire-level claims
(Connect transport, `authorization: Bearer` header, `RuntimeService.invoke`, `SeaForgeAuthority` verbs).
Not executed: no test suite, build, or live SEA-Forge/Workbench run was performed; runtime behaviour
claims rest on source reading and on the named tests' stated assertions.
UNVERIFIED: (a) that a live Workbench accepts `expert.converse` with exactly this input shape — the string
is not defined in any vendored package, so its contract is known only from Horizon's call site and
stand-in tests; (b) the installed Cognate packages are `0.8.0` while `package.json` pins `0.10.0`, so the
vendored wire code may not be what production resolves; (c) `[workshop]` is absent from the committed
`config/horizon.toml`, so no configured `expert_id` / `workspace_id` / `key_path` could be observed.
