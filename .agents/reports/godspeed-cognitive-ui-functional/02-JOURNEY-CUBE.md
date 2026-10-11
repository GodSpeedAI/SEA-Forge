# Cognitive UI: affordance-dependency ladder and Journey Cube (2026-09-23)

The ladder lives in `apps/godspeed-cognitive-ui/e2e/journeys/*.ts` and runs with `bun run e2e` (runner: `e2e/run.ts`, harness: `e2e/ladder.ts`).

- A journey runs only when every journey it `depends_on` passed in the same run. Otherwise it is **BLOCKED** and names the unsettled dependency. A higher journey cannot compensate for a broken lower one.
- Every interaction goes through the rendered UI: real pointer input at on-screen coordinates, key presses, and the composer.
- `window.__gs` is only read, to inspect state. No journey dispatches actions through it.
- Everything runs against the **local contract adapter**, not the Go system front end.

## Dependency graph

```
J0 orientation
 └─ J1 focus
     ├─ J2 artifact inspection ─────────────┐
     └─ J3 representation switching         │
         ├─ J4 time + comparison (J2, J3)   │
         │   └─ J5 narrated beats (J2, J4) ─┤
         └─ J6 causal reasoning (J2, J3)    │
             └─ J7 case design (J3, J6)     │
                 └─ J8 consequential action (J2, J7)
J9 integrated journey (J4, J5, J6, J7, J8)
RECOVERY (J2, J4, J5)
```

| Journey | depends_on | settles | unlocks |
|---|---|---|---|
| J0 Orientation | — | A stable Core-centred orientation surface from which selection is possible | J1 |
| J1 Focus | J0 | Focus is a reusable primitive that preserves Core as the orientation anchor and keeps object identity | J2, J3 |
| J2 Artifact inspection | J0, J1 | Artifacts disclose progressively (pill → excerpt → viewer) without destroying spatial context | J4, J5, J6, J7 |
| J3 Representation switching | J1 | Orbital, causal and comparison are projections of the same objects (identity, selection and state carry across) | J4, J6 |
| J4 Time + comparison | J2, J3 | Time/version and comparison are trustworthy operators that never mutate source history | J5, J9 |
| J5 Narrated beats | J2, J4 | Narration composes focus, arrangement, artifacts and time through the same grammar, and is interruptible | J9 |
| J6 Causal reasoning | J2, J3 | The causal projection is a working representation with inspectable evidence | J7, J9 |
| J7 Case Design | J3, J6 | Case Design is a bounded proposal environment that never mutates authoritative state | J8, J9 |
| J8 Consequential action | J2, J7 | Human and agent consequences share one intent path; authority, execution, evidence and settlement are distinct | J9 |
| J9 Integrated | J4–J8 | The full reference path works end to end on settled primitives, with no deep links | RECOVERY |
| RECOVERY | J2, J4, J5 | Failures stay bounded; authoritative frontend state and the persistent Core survive | — |

## Journey Cube

Each row gives: user intent → visible affordance → interaction → state transition → port/contract → adapter behaviour → artifact/source → evidence → recovery path.

| # | Intent | Affordance | Interaction | Transition | Port / contract | Local adapter | Artifact / source | Expected evidence | Recovery | Test |
|---|---|---|---|---|---|---|---|---|---|---|
| J0 | Get oriented | Core, region spheres, composer | Open app; move pointer | idle → awake (chrome, attention) | `queryTemporalTrajectory` + `getSnapshotAt` ×6 | serves 6 snapshots | none (no renderer loaded) | 1 canvas and 1 Core node; ≤8 shown of ≥25 objects; attention surfaced | reload | J0 |
| J1 | Look at a case | object spheres; Core anchor | click region, then case; Esc; Core anchor | focus stack; camera flight | projection only | — | — | focus path; Core instance unchanged; same ids on refocus | Esc / `h` | J1 |
| J2 | See the evidence | "▤ N artifacts" pill → excerpt card → right-hand viewer | click pill; click card; search; select; tab; pin; close | `materializeArtifact` → `expandArtifact` (ViewSnapshot) → `collapseArtifact` | `resolveArtifact` → `ArtifactPayload` | clone of payload plus provenance | diff (PR #491), trace | right dock; only `diff` then `trace` chunks load; provenance; exact camera restore; pin kept | close / Esc | J2 |
| J3 | Read it differently | center chips: Causal view, Compare with earlier | click chips; select in causal | `arrange` / `openCompare` / `closeCompare`; selection persists | projection (`x.representations.causal`, relationships) | — | — | same DOM element per id; role pills; compare notes; no duplicates | Exit compare / Esc | J3 |
| J4 | What changed over time | History chip; time strip (markers, Mark, Compare A↔B, Return to live) | seek 3 positions; mark 2; compare; open artifact; exit; live | `setTime` / `markTime` / `openCompare` / `closeCompare` / `returnToNow` | `getSnapshotAt` (historical, immutable) | frozen history | diff during comparison | position labels; A/B cursors; restored comparison after the viewer; history hash unchanged | Return to live / `h` | J4 |
| J5 | Explain why | composer question; caption; excerpts; "continue" | ask; click excerpt mid-beat; Esc; continue | `narrativeStarted` / `narrativeBeat` / `applyBeat` / pause / resume / end | `NarrationPort` (`NarrationBeat` + directives) | scripted agent streams 4 beats | markdown record + JSON trace | paused index held; resumed ≥ checkpoint; causal end; citations on every beat | agent lost → partial kept, UI usable, honest hint | J5 |
| J6 | Reason about cause | Causal view; nodes; artifact pill | click chip; select node; open trace | causal layout from representations; selection | projection | — | trace (expected vs observed) | 5 roles; ≥4 labelled links; trace rows; round trip keeps selection | Esc | J6 |
| J7 | Improve the case | "Design case" chip; design panel (versions, stage ▲▼, required evidence, Compare, Save draft, Submit disabled) | select node; switch version; reorder; toggle; save; compare; close | `enterDesign` / `designShow` / `designDraft` / `openCompare` (design) / `setMode` | `queryTemporalTrajectory` + `getSnapshotAt` (template case) | 2 published versions | — | 5 → 4 stages; draft labelled local; Submit disabled with reason; published unchanged; live history hash unchanged | Discard / close | J7 |
| J8 | Decide | action chip "Approve release →"; judgment panel from `ActionDescriptor`s | agent attempt via composer; Escalate without a reason; human approve; pill; panel | `invoke` → `intentSent` → `intentSettled`; `execution`; `loadHistory` | `dispatchIntent` (`InteractionIntent` → `IntentResponse`); `subscribeEvents` | AUTHORITY_DENIED for the agent; accepted for the human; progress → executed → settlement | rollout checks (table) | same action name for both actors; no effect on denial; executed-without-settlement observed; settled only from the snapshot | a fresh attempt after refusal | J8 |
| J9 | Do the whole thing | all of the above | full path, no deep links | all | all | all | diff, record, trace, checks | settled world; Core intact; history unchanged by inspection | — | J9 |
| R | Things break | error cards, honest hints | failing renderer, corrupt/unavailable payload, leave mid-state, interrupt flight, resize, 5× open/close | bounded errors; state preserved | same ports | `?failRenderer`, `?corruptArtifact`, `?failArtifact` | graph (fails), checks, chart, timeline | error names the ref; other renderers work; state coherent; no leaked viewers or excerpts; Core intact | built in | RECOVERY |

## Variations (spec `required_variations`)

| ID | Where proven | UI-side proof | Deferred (needs Go / real backend) |
|---|---|---|---|
| VAR-001 | J0 | ≤8 objects shown of ≥25 at idle Home; detail reachable by focus and zoom (J1, J9) | — |
| VAR-002 | J0 | on wake, only attention-toned members surface; the Core says what needs you | — |
| VAR-003 | J3, J4 | same DOM element per id across orbital, causal, compare and historical positions | — |
| VAR-004 | J4 | 3 positions traversed, 2 compared, history hash unchanged | real history from Go |
| VAR-005 | J5 | 4-beat explanation, markdown and trace artifacts, interrupted and resumed from the checkpoint | a real agent/LLM |
| VAR-006 | J0, J2 | zero renderer chunks at boot; only the opened kinds load | — |
| VAR-007 | J8, J9 | agent and human use `createIntentPath().submit` with the same `InteractionIntent`; the adapter denies the agent | real SEA Forge authority |

## Recovery (spec `required_recovery` and UI cases)

| Case | Proven | Deferred |
|---|---|---|
| RECOV-003 lose the agent adapter | J5 (`agentFailAfter`): partial explanation kept, direct UI usable, composer hint honest | — |
| RECOV-004 optional renderer fails | RECOVERY (`failRenderer=graph`): bounded error shows the ref and digest; other tab renders | — |
| Invalid or missing payload | RECOVERY (`corruptArtifact`, `failArtifact`) | — |
| Leave temporal/compare mid-state; interrupt a transition; resize; repeated open/close; Core never recreated | RECOVERY | — |
| RECOV-001 Go restart with a lease | — | **deferred**: needs the Go service |
| RECOV-002 missed event → resync | — | **deferred**: the local adapter has no gaps to resync |
