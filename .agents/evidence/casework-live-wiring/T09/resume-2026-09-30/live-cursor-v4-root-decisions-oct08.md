# C2 concrete root architecture choices for the next candidate

Date: 2026-10-08. PROPOSAL ONLY. No source, schema, dependency, persisted-state,
authority, or public implementation release. C2 remains held until independent
review and exact operator approval. Preserve prior candidates/rejections.

Read complete candidate/revision2/rejection, bootstrap revision3 and its review,
all-writer recon152870db and current source. The facts disprove global-event
coverage of CLI/CaseRunner mutations and best-effort server callbacks. An index
over existing global events cannot repair missing notification coverage.

## Chosen direction: migrate supported writers, fail closed on dirty state

Propose a shared case-projection mutation journal in sea-forge-case-runner,
already a dependency of CLI/server and already dependent on sea-forge-ledger.
No new dependency: pinned Rust1.92 and existing ledger File::lock suffice.
Migrate EVERY in-tree writer of case Facts/records/local case events, including
case_ops, direct CaseRunner writes, CLI creation/task/manager/project/run/resume/
migration and server case/approval/delegation paths where they affect Facts.
The candidate must supply an exact bounded source inventory; missing writers
remain an implementation-readiness HOLD, never a claim of complete coverage.

Use one cell-scoped interprocess case-projection writer lock, shared by every
supported mutation and coherent Facts capture. It is separate from ledger's
append lock; acquire writer lock before append lock, never reverse. Synchronous
kernel APIs remain synchronous; async edges execute blocking transactions on
their existing blocking boundary, not by adding async to kernel crates.
No auth callback/network I/O under the writer lock; authorization must precede
journal effects and existing authority/domain/settlement checks remain required.
No new credential, identity rule or bypass of D2.

Before any case-visible writes, durably record a pending operation in a proposed
versioned derived journal at .sea-forge/case-projection-journal/v1/. After those
writes, append their real safe global case-notification event, then durably mark
the journal operation complete. Publication errors are not consumed as success.
Readers cannot expose affected dirty cases as ready. A crash between file writes
and append leaves a pending marker; it does not create a historical success.
Recovery under the same writer lock reads actual current state and emits a real
new case.projection_reconciled ledger event before clearing the marker. That
event means present state reconciled, not that the interrupted operation settled
successfully. Retain original failures and operation provenance; no backdated
events or fabricated past Facts. Deduplicate recovery against the actual append
identity before retrying. The candidate must concretize durable ordering,
operation identity, fsync/rename, failure/recovery and journal count/byte limits.

This adds a persisted derived schema and a shared writer boundary, requiring
EXACT operator approval. It cannot be silently counted inside earlier V4
approval. Arbitrary out-of-tree/manual filesystem writes do not participate in
cooperative locks and are explicitly unsupported while the live cell is active;
do not claim these locks fence an uncooperative writer. Deployment must state
that limit. If the milestone requires those writes too, retain HOLD and escalate.

## Remaining concrete choices

- Adopt lossless canonical decimal-string u64 wire ordinals/checked BigInt from
  revision2. Actual opaque ledger IDs remain cursor/SSE IDs; no parsed-ID order.
- Keep proposed page caps:500 scanned rows,500 frames,1MiB,50ms lock wait;
  reconciliation scheduling turn at most2pages/500ms. Make rebuild ceiling
  cumulative per detected index lineage:1,000,000 rows OR10minutes active work.
  Hitting either retains resumable state and unavailable; no automatic reset of
  the ceiling. Increasing it requires an explicit operational decision.
- Derived case-head index4096cases/8MiB; strict inventory4096cases/4MiB.
  The index is never authority and cannot imply case-facts notification coverage.
  Preserve corrupt/dirty evidence and rebuild only from the actual ledger.
- Store and streams retain candidate caps: snapshot1MiB;64revisions/4MiB per
  case;64MiB Store bytes;128retained case heads;32streams;2live frames/2MiB;
  separate replay4MiB and aggregate subscriptions64MiB including in-flight.
  These serialized-byte availability limits are not RSS/heap guarantees.
- Write deadline5seconds for headers and each frame. One already permitted
  in-flight write may finish after invalidation; later permits are barred.
  No network I/O/cancel/join under Store mutex. Failure to bound a write is HOLD.
- Digest: sha256:v1 over Go-owned encoding/json serialization of the fixed
  public snapshot struct excluding capture_digest. Clients treat digest opaque;
  no JCS dependency or cross-language reserialization claim. Exact golden bytes
  and exclusions must be specified; no maps/extension fields with ambiguous
  canonical meaning. Same-cursor restart captures may differ and must resync.
- Bind client_capture_digest into every non-create intent's UNTRUSTED stale
  precondition, alongside exact client_cursor and selected case. Validate it
  against the current authorized retained capture, not as an auth credential.
  Missing/mismatch refuses before dispatch. Preserve PROPOSE_CASE's narrow
  empty-cell exception and all D2/kernel authority checks. This distinguishes
  old restart captures; it does not promise an atomic check-to-mutation interval.
- Inventory member names use exact generated case_YYYYMMDDTHHMMSSZ_<6hex> OR
  frozen legacy case-[a-z0-9_.-]+ grammar. Matching directories with missing or
  unreadable records make inventory unavailable, never empty. Unsupported child
  directories/non-UTF8 names make strict inventory unavailable, not silently
  omitted. Define legacy flat files explicitly from actual supported layout.
- No-case bootstrap remains capture-time complete-empty only, not a continuous
  empty interval or global revision. Recheck authoritative complete inventory
  for create preflight; do not alter its reviewed revision3 contract.
- events.get_range compatibility: optional range_version=2 selects the new
  bounded page contract; omitted version retains existing response shape but
  must use bounded readers too. No new kernel verb. Unknown version refuses.
- Keep explicit typed failure vocabulary: inventory_unavailable,
  frontier_rebuilding, projection_gap, case_not_ready, resync_required;
  candidate must map SFWP/HTTP status and retry behavior exactly. No successful
  hello/world headers precede readiness/subscription admission.

The next builder produces a complete self-contained candidate covering ALL
choices and exact approval set, with source-backed migration inventory and
unrun positive/negative verification matrix. Root and a fresh independent critic
review it before any exact approval request. This file alone approves nothing.
