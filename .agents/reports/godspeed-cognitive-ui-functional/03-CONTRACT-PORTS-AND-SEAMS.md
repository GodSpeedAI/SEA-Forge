# Cognitive UI: contract ports, local adapter and seams (2026-09-23)

The UI crosses one boundary: `apps/godspeed-cognitive-ui/src/ports/contract.ts`. Everything above that line reads only the projection (`src/ports/project.ts`), never contract JSON or fixture structures. If replacing the local adapter with a Go adapter changes anything above the port, that is a defect.

## Ports

| Port | Shape | Source of truth | Implemented by (this phase) |
|---|---|---|---|
| `CaseworkPort` | the contract `CaseworkAdapter` (typescript/mock-adapter.ts): `getSnapshot`, `getSnapshotAt`, `dispatchIntent`, `resolveArtifact`, `queryTemporalTrajectory`, `subscribeEvents` | interface-contracts `typescript/types.ts` | `src/adapters/local/localAdapter.ts` (**local, contract-conformant; not Go**) |
| `NarrationPort` | `explain(question, ctx) → AgentNarrationStream \| null` (spec 04 §8: `NarrationBeat`, `NarrationDirective`, `interrupt()`) | spec 04 §8 | `src/narrative/localAgent.ts` (scripted; no LLM) |
| Intent path | `createIntentPath(store, port).submit(actor, object, action, {justification})` builds `InteractionIntent` (kind, action_name, client_cursor, actor) | types.ts `InteractionIntent`, `IntentResponse` | `src/app/intents.ts`. Human and agent use the same function (VAR-007) |
| Live events | `subscribeEvents` → `snapshot`, `execution_progress`, `settlement_recorded` | spec 04 §6 | `src/app/live.ts` |

## What the local adapter does (and why it is not integration)

- It serves the Northstar case as **contract data at rest**: `src/adapters/local/data/northstar.snapshots.json` (6 `CognitiveWorldSnapshot`s) plus `northstar.trajectory.json`, and the template world `templateData.ts`. Payloads (`payloads.ts`) are `ArtifactPayload`s with digest and provenance.
- **Authority**, in the order the contract describes:
  1. `DUPLICATE_IN_FLIGHT` (a repeated `intent_id`).
  2. `STALE_PROJECTION` (`client_cursor` ≠ head).
  3. `UNAVAILABLE` (the action is not offered).
  4. `AUTHORITY_DENIED` (actor role `agent_operator`).
  5. `UNAUTHORIZED_ROLE` (approve or reject without `case_architect` or `security_officer`).
  6. `JUSTIFICATION_REQUIRED`.
  A refusal appends nothing.
- **Execution**, as contract events:
  1. An accepted approval appends a snapshot (work in progress, run object).
  2. Then `execution_progress` events for orchestrator, builder, critic and verifier.
  3. Then an *executed* snapshot: the evidence object appears and there is **no settlement**.
  4. Then `execution_progress` for `settling`.
  5. Then `settlement_recorded` (`OperationalSettlement`) with a snapshot carrying `x.settlement`.
  History is append-only and frozen.
- The UI reads settlement **only** from the snapshot (`x.settlement`). Completion is shown as "Executed · awaiting settlement".
- Recovery switches: `?failArtifact=`, `?corruptArtifact=`, `?failRenderer=`, `?agent=off`, `?agentFailAfter=N`.
- Pacing switches: `?speed=N` (execution) and `?beatPace=N` (narration holds; the E2E ladder uses 4 because the headless browser renders at about 5 fps).

**Not proven here:**
- Go system front end behaviour.
- Real SEA Forge authority.
- Gauntlet execution.
- RealityTrace evidence.
- SSE reconnect and resync (RECOV-002).
- Go restart and lease reconciliation (RECOV-001).

## Proposed contract extensions used by the UI (seams)

All are optional fields under `x` or spec-04 shapes not yet in `types.ts`. A Go adapter must supply them, or the UI degrades to generic behaviour (ring layout, no causal reading, no settlement until the field exists).

| Seam | Field | Why the UI needs it | Degrades to |
|---|---|---|---|
| S1 | `CognitiveRelationship` (spec 04 §3) under `snapshot.x.relationships` | relationships and causal links | `depends_on` only |
| S1 | `CognitiveArtifact` (spec 04 §7) under `object.x.artifacts` | progressive disclosure: which artifacts an object has | no artifact pill |
| S1 | `x.presentation` (`region`, `case`, `facet`, `item`, `person`, `run`) | the system surface above cases (Home regions); types.ts object kinds are case-internal | derived from the contract `kind` |
| S1 | `x.representations.causal` {order, role, label, explanation, badge, tone} | the causal representation as a per-object representation (same object, different representation) | no causal view offered |
| S1 | `x.label_side`, `x.metric`, `x.tone`, `x.ghost`, `x.dormant`, `x.residue` | presentation hints | defaults |
| S2 | `x.settlement` (`OperationalSettlement`) on the settled object | settlement must be authoritative and in the projection, not inferred from events | shown as awaiting settlement |
| S2 | `x.template_case_id` | case → designable template (Case Design) | no "Design case" affordance |
| S3 | `NarrationDirective.x` {arrange, temporalTo, compare, reveal, dim} | the grammar's remaining operators (spec 04 only has focus, camera, highlight, annotate, openArtifact, temporalStep) | narration limited to those six |
| S4 | *(missing)* an operation to submit a case-template proposal | Case Design "Submit proposal" | the button is disabled with this reason; drafts stay local |
| S5 | *(missing)* persisted artifact pins | "Pin" | pins are local UI state, labelled "Pinned · local" |

Artifact content conventions (types.ts allows `text/markdown`, `application/json`, `text/x-diff`, `text/plain`):
- JSON carries a `schema` discriminator: `table`, `chart`, `graph`, `realitytrace` or `timeline`.
- Unknown JSON is shown as a tree.
- Anything unparseable is a bounded error that keeps the ref and digest.
