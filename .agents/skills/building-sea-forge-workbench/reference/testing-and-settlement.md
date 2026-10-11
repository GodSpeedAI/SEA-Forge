# Testing and settlement evidence

A slice without its checks is unfinished. Tests live beside the layer they
prove and follow repository TDD conventions (root `AGENTS.md`): failing
focused test → implement → refactor; cover allow, deny, escalate, malformed,
timeout, nonzero exit, and false success; assert **absence of side effects on
denied paths**, not only the verdict.

## Test layers

### Rust bridge tests (Tauri host crate)
Integration tests for: every typed command (happy + denial + stale +
ambiguity), channel ordering and backpressure, reconnect with cursor resume,
`request.get_status` recovery after simulated socket drop mid-request,
desktop permission scopes (no FS/socket access beyond declared), packaging
smoke (bundle builds, launches, connects). Mirror the conformance-test style
of `crates/sea-forge-server/tests/`.

### Server conformance tests (when a slice adds verbs)
Per-verb tests in `crates/sea-forge-server/tests/` following the existing
`conformance_m*.rs` pattern: governed outcome variants, precondition/digest
rejection, no side effect on deny, unknown-verb rejection stays clean.
Spec-minimum proofs P1–P4b and the active full-spec milestone gate stay green.

### Contract tests (generated types)
Round-trip: Rust type → generated JSON Schema → generated TS → AJV validate
of a server-produced sample. A drift test fails when a Rust type changes
without regeneration (goldens under the frontend workspace, regenerated
deliberately in the same commit).

### State-machine tests (XState)
Model tests per machine: preflight→commit (including stale rejection path),
submission ambiguity (must pass through status-recovery state before any
retry state), event reconnect (gap → `events.get_range` → authoritative
refetch), agent permission (deny ≠ cancel), settlement declaration. Assert
illegal transitions are unreachable (e.g. no `committed` without
`preflight_ok`, no duplicate commit from `ambiguous`).

### Component tests (Vitest + RTL)
Semantic components: every state-domain variant renders its exact label and
non-color cue; disabled actions expose reason + repair path; evidence drawer
tabs (Why/Evidence/Provenance/Record) reachable by keyboard. axe-core has no
violations per story.

### Storybook fixtures
Every semantic-component state and organism reference state gets a story;
stories are the review artifact and the visual-regression basis. Fixture data
comes from generated types filled with realistic governed records — never
fabricated authority/evidence/score values presented as real.

### Visual-fidelity tests

When a checked-in mockup or story is the target, render the reference and the
implementation at the same viewport and capture both a reference screenshot
and an implementation screenshot. Assert stable computed geometry for shell
tracks and major regions, plus DOM/source-projection drift where practical.
Assert key layout rules through computed styles in normal and reduced-motion
modes so a selector trapped inside the wrong media query fails loudly.
Review semantic differences caused by live data separately from visual drift.
The browser pass also requires zero console errors and an axe result with no
violations; a source-only CSS or JSX review is not settlement evidence.

### Route integration tests
Route + guards + view model against a mocked bridge: guard denial renders the
governed denial surface (not a blank page), search-state round-trips, event
reduction updates the view, unknown enum variants render `unknown` and mark
the view stale.

### Desktop end-to-end tests (Playwright against the Tauri dev build)
Primary-path journeys, keyboard-only completion, and the proof scenarios
below against a real `sea-forge-server` on a temp root.

## Proof scenarios (the plan's required end-to-end evidence)

1. Happy command path (submit → authorized → executed → settled → visible).
2. Execution succeeds but settlement rejects — UI shows them separately.
3. Approval escalation path.
4. Agent permission denial **without** cancellation of the run.
5. Integrity halt surfaces as integrity, not generic error.
6. Ambiguous commit recovered via `request.get_status` without duplication.
7. Stale preflight → `rejected_as_stale` → repair path re-preflights.
8. Event gap → `events.get_range` + authoritative refetch (no silent loss).
9. Disclosure denial happens before retrieval (no content ever fetched).
10. Capability stays conservative after insufficient evidence.
11. CopilotKit adapter removal changes no canonical behavior (eval #4).
12. Shipped bundle contains no Bun runtime (packaging inspection).

## Settlement evidence required to call a slice complete

- Gate commands and their passing output (exact, copy-pasteable).
- The negative check: which test fails when the behavior is broken (show it
  failing once, or point to the assertion that has teeth).
- a11y evidence for new surfaces (axe pass + keyboard path note).
- Updated Storybook stories listed.
- Classifications (`EXISTS/PARTIAL/MISSING/...`) used, with `file:line`.
- Skipped platform tests reported as skipped with reasons — never as passed.
