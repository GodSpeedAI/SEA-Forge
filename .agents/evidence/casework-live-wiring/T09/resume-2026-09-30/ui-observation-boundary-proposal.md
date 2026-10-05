# UI observation boundary proposal

Root architecture preparation, 2026-10-01; no implementation approval or source changes.
Read approved contract extension, prepared units and UI source preparation before implementation.

Split unit6 into ingestion/cache and required app/model/view integration. Both must finish before
T09 completion. Captured snapshots, world objects and WorldHistory never retain current traces.
The app integration uses a bounded ephemeral run-ID sidecar and displays it only at now; case
switch and disposal clear it. Historical inspection displays captured standing only.

Preserve the canonical observation envelope unchanged after strict validation. Removing cached
duplicate frames from its payload while preserving server retained/omitted counts would create
an internally inconsistent source record. Local cache delivery decisions need a separate typed
boundary, with no new wire StreamEvent or invented server capacity/count claims.

Proposed smallest compatible seam: an optional fifth RunObservationCallbacks argument to the
internal CaseworkPort.subscribeEvents. It exposes onUpdate(original validated event, per-run
newly admitted frames) and onCapacityNotice(typed UI-local notice). Legacy onEvent continues to
receive the original valid envelope; app integration consumes the typed update path to replace
annotations by real run/event IDs, rather than append duplicate logs. onError remains transport
failure only. UI-local types are explicitly separate from canonical mirrored DTOs.

The identity cache belongs to each subscription, survives internal native/fetch reconnects,
and clears at disposal/case switch. At most32 run keys,4096 identities globally,1024 per run;
evict oldest identities at either ID bound, terminal-key LRU at key capacity. Never evict a
nonterminal key solely to admit another nonterminal key. If all keys are nonterminal, drop
the unseen run's local frame admission and emit typed local capacity notice while case revisions
continue. Dedupe holds only while identity is cached; later replay after eviction is allowed.
The source envelope and its counts are never rewritten to describe these local decisions.

Both native and fetch ingestion validate/narrow the named event before ordinary case-cursor
logic, never compare/update lastCursor for observations and keep actual source timestamps/cursor.
Native allowlist adds execution_observation. Validate requested case, bounded cohort/read/frame
counts, arithmetic invariants, exact vocabulary and safe optional numeric fields; reject unknown
trace payload fields. Required metadata-only updates remain deliverable even with no fresh frames.

An independent source-grounded critic must assess this seam against actual port implementations,
subscribers and native/fetch tests before root finalizes the bounded fixture assignment. In
particular, review compatibility, canonical-count preservation and scope of bounded dedupe.
No source/test writes or compilation are authorized by this preparation document.
