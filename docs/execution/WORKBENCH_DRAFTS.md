# Workbench local drafts

Status: retained local capability pending a scoped draft-authoring product decision. Last reviewed: 2026-08-31.

## Purpose and boundary

The desktop host exposes `draft_save`, `draft_load`, `draft_list`, and
`draft_delete` for case-authoring drafts. A draft is local, reversible,
non-authoritative renderer state. It is stored as one versioned JSON file under
the Tauri application-data directory. It is never written to `.sea-forge/`,
the server socket, a ledger, or a governed record.

A draft becomes canonical case state only through the existing governed
`case.preflight` then `case.commit` path. Saving, loading, listing, or deleting
a draft does not request authority and must not be represented as a case,
execution, approval, evidence record, or settlement.

## Current contract

| Command | Contract |
| --- | --- |
| `draft_save(draft_id, state)` | Creates one local JSON envelope and increments its per-draft version. On platforms where `rename` replaces an existing destination, it also replaces and atomically publishes an existing draft; replacement is unsupported by the current implementation where that rename behavior is unavailable. |
| `draft_load(draft_id)` | Returns the stored envelope or a bridge error string. |
| `draft_list()` | Returns well-formed drafts ordered newest first; malformed local files are skipped. |
| `draft_delete(draft_id)` | Removes the local file; a missing draft is a successful idempotent no-op. |

Draft IDs are a single `[A-Za-z0-9_-]{1,64}` path segment. Invalid IDs are
refused before filesystem access. The host fsyncs the temporary file before
rename. On platforms that support replacement by rename, this prevents a
partially written destination from being published.

## Retained capability proposal

The commands remain registered but intentionally have no renderer call site
until a scoped draft-authoring story exists. This is a retained-capability
proposal, not a settled product decision. It avoids removing a tested,
reversible local persistence boundary merely because its UI is deferred.
`sfwp_cell` is not part of this proposal: it already has renderer callers.

Any draft-authoring UI change MUST:

1. use only these commands for local draft persistence;
2. show that drafts are local and not submitted work;
3. require explicit preflight and commit to make a draft canonical; and
4. add renderer tests covering save, reload/list, discard, invalid-ID refusal,
   and the transition from draft to governed preflight/commit.

A future concurrent multi-window or attachment requirement is a trigger to
reconsider the JSON-file implementation. It does not authorize SQLite, a
Tauri store plugin, synchronization, or any new dependency by itself.
