# UI (React panels and chrome)

## Overview
The React 19 presentation components of the cognitive environment: composer, chrome and status bar, artifact dock and excerpts, judgment panel, discretionary drawer, execution pill/panel, time strip, compare bar, session badge, login screen, outline, workbench chrome, template/case design panels, and grounded Thoth answers. Components are deliberately thin: they render store state and hand intent decisions to `src/app` — the UI never decides authority.

## Interface
- Chrome and navigation: `Brand`, `Breadcrumb`, `StatusBar`, `UserMark`, `CoreAnchorLabel` [Chrome.tsx:Brand, Chrome.tsx:StatusBar, Chrome.tsx:CoreAnchorLabel]; `WorkbenchChrome` (`RailItem`, `TabItem`) [WorkbenchChrome.tsx:WorkbenchChrome]; `OutlineView` / `OutlineItem` (non-spatial list of what is on screen) [OutlineView.tsx:OutlineView]
- Composition: `Composer` (composer input, home and focused variants) [Composer.tsx:Composer]
- Decisions: `JudgmentPanel` (`JudgmentOption`, `ReasonDraft`, `decisionKeyOf`, `reasonFor`) [JudgmentPanel.tsx:JudgmentPanel]; `DiscretionaryDrawer` (ADD_DISCRETIONARY_WORK typed payload surface) [DiscretionaryDrawer.tsx:DiscretionaryDrawer]
- Artifacts: `ArtifactDock` (tabs over an object's bound artifacts) [ArtifactDock.tsx:ArtifactDock]; `ArtifactExcerpt` (pinned card excerpt) [ArtifactExcerpt.tsx:ArtifactExcerpt]
- Execution and time: `ExecutionPill` (progress `null` when only snapshot standing is known — never guessed) [ExecutionPill.tsx:ExecutionPill]; `ExecutionPanel` [ExecutionPanel.tsx:ExecutionPanel]; `TimeStrip` [TimeStrip.tsx:TimeStrip]; `CompareBar` [CompareBar.tsx:CompareBar]
- Identity: `LoginScreen` (shown only while the session port reports no identity) [LoginScreen.tsx:LoginScreen]; `SessionBadge` (who the session resolved, role named so refusals read against it) [SessionBadge.tsx:SessionBadge]
- Case design: `TemplateDesignPanel` (template picker → parameters → preflight digest → commit gate) [TemplateDesignPanel.tsx:TemplateDesignPanel]; `CaseDesignPanel` (`DesignItem`) [CaseDesignPanel.tsx:CaseDesignPanel]
- Grounded disclosure: `GroundedAnswer` (renders a `ThothAnswerView` with active claim and pause state) [GroundedAnswer.tsx:GroundedAnswer]
- Honest availability: `UNAVAILABLE_ACTIONS` / `UnavailableActions` — CJ10–CJ12 render as static descriptors, never buttons [UnavailableActions.tsx:UNAVAILABLE_ACTIONS]
- Journey harness (component-level wiring tests over the real adapter/store, `react-dom/server`): `render`, `northstarHistory`, `makeJourneyAdapter`, `makeStore`, `ACTOR`, `DEV_ACTOR` [journeys.harness.tsx:render, journeys.harness.tsx:makeJourneyAdapter]

## Invariants
- The client never asserts identity: `LoginScreen` forwards credentials to the adapter, and `SessionBadge` shows only the verified session (or the fixture identity the local adapter self-reports) [LoginScreen.tsx:LoginScreen, SessionBadge.tsx:SessionBadge]
- `TemplateDesignPanel` keeps the submit button disabled until a preflight for the CURRENT parameters has passed — the digest binds the commit to exactly what was reviewed [TemplateDesignPanel.tsx:TemplateDesignPanel]
- `UnavailableActions` renders CJ10–CJ12 as plain list items so they cannot be dispatched [UnavailableActions.tsx:UnavailableActions]
- Execution progress is shown only when the stream reported it; a `null` progress never becomes a guessed number [ExecutionPill.tsx:ExecutionPillProps]
- What the discretionary drawer submits is a proposal, not a mutation — the backend decides [DiscretionaryDrawer.tsx:DiscretionaryDrawer]

## Side Effects
- ["None"] (components render state and invoke callbacks; all network/store effects live in `src/app` and the adapters; each component pairs with a co-located CSS file, e.g. `chrome.css`, `composer.css`)

## Verification
- Run: `bun test` (working directory: `apps/godspeed-cognitive-ui`; component journeys: `src/ui/journeys.test.tsx` via `journeys.harness.tsx`)
