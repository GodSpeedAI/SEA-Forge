# App (shell and intent wiring)

## Overview
The application shell and the wiring that makes the world honest: `App.tsx` composes the scene, panels, composer and live connection; `intents.ts` is the one path consequential work takes out of the UI for humans and agents alike (VAR-007); `commands.ts` routes composer text onto the shared action vocabulary; `live.ts` keeps the store in step with the port's event stream; `proposals.ts` and `design.ts` implement the case-design flow.

## Interface
- `App(p: AppProps)` — the shell: boot, session, world scene, panels, composer, narrative player and live events [App.tsx:App, App.tsx:AppProps]
- `createIntentPath(store, port): IntentPath` — `submit(actor, object, action, opts?)` builds a contract `InteractionIntent` (stamps the actor and the cursor the actor was looking at), dispatches it, records the backend's answer, and on `STALE_PROJECTION` refreshes standing once without replaying the intent [intents.ts:createIntentPath, intents.ts:IntentPath]
- `judgmentChoices(actions)` — the decisions the judgment panel offers: consequential actions except discretionary work (a typed drawer surface, never a choice) [intents.ts:judgmentChoices]
- `runCommand(ctx, raw)` — composer routing: pause/resume, home/back, theme, timeline, compare, causal, design, execution inspect, agent-vs-human approval, focus by title, artifact open, or explain; targets are found by what the backend says about objects, never by hard-coded ids [commands.ts:runCommand, commands.ts:CommandContext]
- `executionStateOf(runStatus, settlement, provenance)` — where an execution stands; an authoritative settlement decides, and a live snapshot's typed run status decides otherwise (`WAITING_ON_OTHERS`/`ACTION_REQUIRED` read as executed, never settled) [live.ts:executionStateOf]
- `connectLive(port, store, caseId, raw, summaries, options?)` — subscribes to the port's event stream: snapshots append history (never rewrite it), execution progress is projected, connection state moves live → reconnecting → interrupted on a bounded grace period, and recovery is proven only by a source-backed state frame or an authoritative snapshot probe [live.ts:connectLive]
- `createProposalFlow(deps)` — the case-design proposal flow (T09, CJ04): template picker → parameters → preflight → commit; `PROPOSE_CASE` goes out only after a preflight for the CURRENT parameters passed and carries that preflight's digest [proposals.ts:createProposalFlow]
- `loadAndFocusCase(port, store, caseId, actor)` — loads a case's history, focuses its case object, and returns raw snapshots for event-stream re-attachment [proposals.ts:loadAndFocusCase]
- `designModel(snap, versionLabel)`, `moveItem(snap, itemId, dir)`, `toggleRequired(snap, itemId)`, `isDirty(draft, h)` — case design over the template world as local draft proposals; proposals never touch published versions (no contract operation to submit them yet, seam S4) [design.ts:designModel, design.ts:moveItem, design.ts:toggleRequired, design.ts:isDirty]
- `emptyWorldOffersDesign(objects)` — an empty home (only core/category objects) must still offer starting a case from a template [proposals.ts:emptyWorldOffersDesign]

## Invariants
- The UI never decides authority: a refusal is shown as the backend phrased it, and world changes arrive only as new snapshots [intents.ts:createIntentPath]
- One path for consequential work, human or agent (VAR-007): "let the agent approve" goes through the same intent path as a human decision [commands.ts:runCommand, intents.ts:createIntentPath]
- The intent itself is never replayed after a `STALE_PROJECTION` refusal; standing is refreshed once, and a snapshot for another case is never merged into the newly selected history [intents.ts:createIntentPath]
- Snapshots append history and never rewrite it; a `settlement_recorded` event is shown only once the snapshot carrying the settlement arrives [live.ts:connectLive]
- An honest execution pill shows the phase, not a guessed number: progress comes only from stream-reported `execution_progress` [live.ts:connectLive]
- Heartbeats and control frames cannot make an interrupted connection look live again; only source-backed state frames or an authoritative snapshot read prove recovery [live.ts:connectLive, live.ts:probeSource]
- The past is read-only: approval and agent commands refuse when `isPast(s)` [commands.ts:runCommand]

## Side Effects
- Store dispatches (`intentSent`, `intentSettled`, `loadHistory`, `execution`, `connectionState`, proposal/design actions) [intents.ts:createIntentPath, live.ts:connectLive]
- Port calls: `dispatchIntent`, `getSnapshot` (stale-projection refresh and recovery probe), `subscribeEvents`, template reads/preflight [intents.ts:createIntentPath, live.ts:connectLive, proposals.ts:createProposalFlow]
- Timers: outage grace timer and recovery probe scheduling in `connectLive` [live.ts:connectLive]
- Console warning on stream errors (`console.warn('[event stream]', err)`) [live.ts:connectLive]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; tests: `src/app/intents.test.ts`, `live.test.ts`, `design.test.ts`, `emptyHomeDesign.test.ts`)
