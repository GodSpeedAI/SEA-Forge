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

Admission-map clarification before fixture implementation: the per-run frame map contains
an entry for every locally admitted run, even when its new-frame array is empty. A run
rejected at the32key cap has no map entry. Thus the app can update admitted metadata-only
annotations without creating an unbounded sidecar key for a capacity-rejected run. Keep
the original envelope intact; do not infer local admission from server selected/count fields
or use an empty array to conflate capacity rejection with deduplicated metadata-only updates.
The app's sidecar must independently respect the32key/4096identity/1024per-run limits.
Root requires independent source critique of this clarification before UI fixture release.

Independent read-only critique by t09_run_children_fixture_builder found this clarification
coherent and source-compatible with the approved seam. Root accepts its fixture requirements:
map membership uses own-property checks, entries are only unique run IDs in the validated
envelope, and mapped frames belong to that run. An admitted unavailable row keeps its last
validated annotation and explicit unavailable indication; an empty frame array must not
erase it or imply current success. An admitted empty/deduplicated frame set still delivers
metadata. Pair identity is (run_id,event_id); dedupe is bounded to cached pairs. Local capacity
does not call onError or prevent ordinary revisions. Store cleanup and historical isolation
remain mandatory. No UI fixture or implementation write release is made by this record.
