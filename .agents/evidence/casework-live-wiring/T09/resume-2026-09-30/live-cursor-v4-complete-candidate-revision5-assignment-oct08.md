# Original revision 5 assignment — 2026-10-08

FRESH C2 DOC-repair builder; no source implementation is authorized. Read the full originals: `live-cursor-v4-complete-candidate-assignment-oct08.md`, `live-cursor-v4-complete-candidate-revision3-assignment-oct08.md`, root decisions and capture clarification, full root revision-4 review, independent rejection `c803ad18`, actual candidate v4 SHA `3cbf9e42...`, revision 2 historical/SSE digest requirements, and bootstrap revision 3 plus review. Preserve the full history and all existing files.

Produce one NEW immutable, self-contained `live-cursor-v4-complete-candidate-revision5-oct08.md`, proposal only, about 2,200 words maximum if practical. Address all review findings:

1. Exact historical GET requires cursor plus matching `capture_digest`; SSE reconnect requires `last` plus matching `last_capture_digest`, with exact pre-header mismatch behavior and client reset/resync flow.
2. State the 1 MiB maximum per snapshot/event and maximum 64 replay revisions, in addition to other caps.
3. Define exact Go-owned fixed-struct digest bytes, field order/exclusions, and golden profile. Clients treat the digest as opaque; do not require JCS or client reserialization.
4. Persist bounded operation-ID scan continuation. Absence is proven only after the scan completes through the pinned head; partial recovery stays pending.
5. Any pending journal operation blocks complete-empty/bootstrap success.
6. Enumerate delegated-agent permission-broker and `settlement.json` writers with full source paths.
7. Keep full migration coverage audit as a gate before implementation readiness.
8. Disclose that the earlier v4 preimage was lost. Do not reconstruct it.

Preserve the shared journal root architecture unless actual source proves a dependency cycle; if so, report the exact path. No new dependency, authority change, I/O under locks, or fake history. The candidate remains proposal only, not source/test/gate/Git/status work. Use native `apply_patch` as the sole persistent writer. Archive this complete assignment first. A fresh critic will receive the full original instructions and frozen revision 5 candidate. Never overwrite an old artifact.
