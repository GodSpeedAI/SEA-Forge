# T09 proposal independent architecture review, round four

Root recorded the delivered independent review from Luna critic `t08_final_static`.
The critic did not author the proposal or its repairs. Latest fresh repair builder was
`sse_loopback_test_builder`; original instruction was to remove unsupported cumulative
256 MiB hydration arithmetic that omitted run.list and inspect retry, state only actual
per-line/logical-call/concurrency/projected-metadata limits, and record the operator approval.

**APPROVE: proposal architecture only.** The current proposal distinguishes one run.list
from at most eight logical run.get calls, retains inspect retry once, caps individual
response lines at 32 MiB, concurrent run.get at two, and projected hydration metadata at
1 MiB. It explicitly disclaims aggregate wire-byte/SSE-total and upstream CPU/filesystem
bounds. All limits remain proposed implementation work, not claims about current allocation.

Independent source inspection confirms preserved fixes: actual Ask wire tag `ask` versus
catalog name `thoth.ask`; real `(run_id,event_id)` deduplication with no SSE id or case cursor
mutation; frozen historical snapshots; finite actual question kinds and complete Thoth
disclosure DTO; protected Ask auth/CSRF/rate/body limits; bounded poller and UI caches;
canonical contract path and explicit schema-test inventory.

The relevant evidence is the proposal's Current implementation gaps, Proposed execution
observation, response-cap prerequisite and required-tests sections, with its cited source
anchors (`client.go` read/retry paths, server Ask enum/dispatch, canonical types, HTTP adapter
parser and contract-conformance test inventory). Material differences from current code
are explicitly future changes: client response cap, hydration/pollers, UI cache, /api/ask
and additive D-1/spec/golden amendments. No runtime/test gate ran for this source-only review.
This does not approve an implementation, settle T09, or supersede T07/T08 dependencies.
