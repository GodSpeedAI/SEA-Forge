# UI observation boundary decision

Root accepts the optional fifth typed callback seam in ui-observation-boundary-proposal.md
after independent read-only source review by t09_run_children_fixture_builder. No UI
implementation is approved by this record; test-first assignment follows separately.

Direct review anchors: ports/contract.ts204–216, httpCaseworkAdapter.ts297–447 and530–549,
localAdapter.ts168–173, caseworkPortConformance.ts89 onward, app/live.ts44–129,
model/types.ts399–407/467, store.ts288–289/384–389 and App.tsx89–92/354–362/531–543,
under apps/godspeed-cognitive-ui/src. The critic made no edits and ran no tests.

Keep the original validated canonical envelope and server counts. Typed onUpdate delivers
that envelope plus locally admitted frames; typed capacity notices remain UI-local and never
use transport onError. Legacy onEvent receives the original envelope. The app must ignore
observations in its ordinary event handler and consume the typed path once. Metadata-only
updates are required. Deduplication is limited to identities still cached.

Two required implementation details from review: keep the subscription cache across internal
reconnects but clear it on unsubscribe; separately clear the Store observation sidecar in
connectLive cleanup and on case changes. Hide all current observation annotations while
viewing a historical revision; existing global execution rendering needs an explicit guard.
Branch on strictly validated observations before cursor comparisons in both transport paths,
without updating lastCursor. Native event allowlist must include execution_observation.

Both ingestion/cache and app/model/view integration remain required for full T09 approval.
