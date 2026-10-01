# Case-owned run children test-first assignment

Prepared by root for the next bounded T09 unit. HOLD all Go/test writes until Ask broader
Go gates complete; a new failing package must not contaminate that source graph.
Read approved proposal, runtime-integration-decisions.md, run-observation-source-preparation.md,
root/scoped instructions and source with Graft. No compiler token until explicit transfer.

First builder owns only a new internal/projection/run_children_test.go, using existing
CaseFacts, RunSummary and horizon shapes, without changing implementation or declaring APIs.
Use a nearby projection test pattern. Tests must compile against the pre-child implementation
and distinguish missing execution_trace children via actual assertions, not build failure.

Cover two or more real case-owned summaries attached to an actual captured horizon item:
one child per summary, exact run ID, execution_trace kind, exact actual item ParentID,
empty actions, no trace frames/current observations/poller state. More than eight valid runs
still produce all children; live hydration's limit is separate. Exercise all six execution
and four settlement spellings with badge/explanation exposing each actual standing separately;
completed/unsettled must not imply accepted settlement or invent progress percentages.
Use existing CognitiveObject fields only; no added schema or DTO fields.

Pure Build guards must never fabricate children for empty/duplicate run IDs, foreign case,
missing/empty/absent actual parent, object ID collisions or unknown standing values. Tests
must assert deterministic valid-child output and preserve parent consequential actions/focus.
Captured historical facts must remain unchanged if later run summaries are modified: use
the actual Store cloning pattern, without adding any observation buffers to CaseFacts.

Runtime follow-up after independent expected RED will add explicit NewRunListForCase and
CaseAuthorityPort.RunsListForCase(ctx, CaseRef) (RunListResult,error), preserving the legacy
unscoped methods. Scoped result contains semantic summaries and unreadable IDs. LiveSource
uses exactly one scoped call, no global fallback, retains factual unreadable IDs distinctly
and Store clones that slice. Source validates requested Record/Overview/Horizon refs, readable
case ownership and real parent identity; malformed/duplicate/overlapping IDs fail honestly.
Adapter/source tests and exact source ownership will be specified before implementation.

Do not require HorizonItem.RunIDs membership: sequential Horizon then RunList can observe
a new run after the earlier horizon capture. Actual scoped summary case/item references and
the captured parent item's existence are the approved relationship; no joint atomic read is
promised. Unit5 later validates actual hydrated run records separately.

No run.get hydration, polling, SSE, frame buffers, UI, kernel API, dependencies or golden edits
in this fixture unit. Scoped ownership does not bound upstream directory enumeration. Freeze
source/hash/deviations; independent critic gets these original instructions plus actual tests,
source-reviews then runs expected RED with host RAM/process preflight and the sole compiler token.
Fresh builder after rejection; no approval without direct evidence. Full T09 remains open.
