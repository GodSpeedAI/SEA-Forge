# Narrative

## Overview
The agent side of the environment (spec 04 §8): the narration port conductor that turns an agent's beats into UI-grammar directives, the beat player (pause/checkpoint/resume), and the two narration sources — a scripted local agent (dev/test) and governed Thoth Ask (live). The UI never depends on a narration port being present.

## Interface
- `explain(store, agent, question): boolean` — runs a question through the narration port and streams its beats into the store; returns false when there is nothing to say [conduct.ts:explain]
- `beatFromNarration(nb, s, fromCursor)` — translates one narration beat; a directive can only select, focus, arrange, reveal, highlight, annotate, open an artifact, move through time or open a comparison — it cannot create UI or change case data [conduct.ts:beatFromNarration]
- `startNarrativePlayer(store, clock?, pace?)` — plays beats in order as they stream in; each beat holds for its `holdMs`; pausing keeps a checkpoint with time left; resuming continues from that beat (returns a stop function) [player.ts:startNarrativePlayer]
- `DEFAULT_HOLD_MS = 3200`; injectable `Clock` for deterministic tests [player.ts:DEFAULT_HOLD_MS, player.ts:Clock]
- `createLocalAgent(opts): NarrationPort` — scripted local stand-in (no LLM) over the Northstar fixtures; `normalize(text)` is its question normalizer [localAgent.ts:createLocalAgent, localAgent.ts:normalize]
- `createThothNarration(source): NarrationPort` — live narration over governed Thoth Ask (POST /api/ask via the HTTP adapter's `ask`) [thothNarration.ts:createThothNarration]
- `parseThothQuestion(text, caseId)` — maps a question onto the typed Thoth question surface (`THOTH_QUESTION_KINDS`) or answers honestly as unsupported [thothNarration.ts:parseThothQuestion, thothNarration.ts:THOTH_QUESTION_KINDS]
- `beatsOfAnswer(answer)` — grounded beats built only from a returned answer's claims, fixed disposition copy and references [thothNarration.ts:beatsOfAnswer]

## Invariants
- Narration directives use only the UI's declared vocabulary; losing the agent adapter ends the explanation where it is and leaves direct interaction untouched [conduct.ts:beatFromNarration]
- Every word of a grounded beat comes from the returned Thoth answer and every citation from a returned reference; the client never invents claims, citations or directives, and an answer confers no authority [thothNarration.ts:beatsOfAnswer]
- Thoth answers a finite set of typed questions (kind + subject), not free text; unsupported questions get `UNSUPPORTED_COPY` rather than a fabricated answer [thothNarration.ts:parseThothQuestion, thothNarration.ts:UNSUPPORTED_COPY]
- Progress percentage is never guessed: the player shows beats only, and execution progress honesty is enforced in `src/app/live.ts` [player.ts:startNarrativePlayer]

## Side Effects
- Timers: the player schedules beat advancement and pause checkpoints via `Clock` (real `setTimeout` by default) [player.ts:startNarrativePlayer]
- Store dispatches: beats, focus, highlights, annotations, artifact opens and temporal steps are applied as store actions [conduct.ts:explain]
- Live path performs a network call per question (Thoth Ask) through `AskSource` [thothNarration.ts:AskSource]

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; tests: `src/narrative/conduct.test.ts`, `player.test.ts`, `thothNarration.test.ts`)
