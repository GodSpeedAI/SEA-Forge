# Unit5 safe trace-port review: source-anchor clarification

This is an append-only clarification to
`observation-safe-trace-port-proposal-independent-review.md`. The original review remains
unchanged and retains its original SHA-256. This clarification corrects the `read_jsonl`
line citation and narrows the completeness finding to what the existing source proves.

## Correct source locations

- `crates/sea-forge-server/src/sfwp/run_views.rs:425-436`: `read_jsonl`. It returns an empty
  vector if the journal is over cap (`:426-428`) or `read_to_string` fails (`:429-431`). It
  filters blank lines and uses `map_while` (`:432-435`), so a malformed line stops reading at
  the valid prefix.
- `crates/sea-forge-server/src/sfwp/run_views.rs:240-250`: `RecordPresence` exposes only the
  record filename, `present`, and optional byte size. The fixed record inventory contains
  `trace.jsonl` at `:85-95`; the `run.get` population is at `:870-880`.
- `crates/sea-forge-server/src/sfwp/run_views.rs:842-855`: `get` reads the trace through
  `read_jsonl`; it rejects only when events, settlement, and plan are all absent/unreadable.
  Thus an existing settlement or plan allows a run response even if the trace read produced
  no rows.
- `crates/sea-forge-server/src/sfwp/run_views.rs:283-326`: `RunRecord.trace` is a defaulted
  vector serialized as the returned trace array.

## Semantic distinction

The port can exactly count and retain **safe allowlisted rows in the `run.get` trace array it
received**. The proposal's `total safe-frame count` is defensible if explicitly scoped to that
returned array and is computed before the port's 1024-row retention. This count is not a
source-journal row count or a proof that all source rows were returned. No existing `run.get`
field reports JSONL parse completeness; `RecordPresence` reports only filesystem presence and
size. A present nonempty journal can therefore still yield an empty array on read failure or a
valid prefix after a malformed row. Do not silently imply full-journal completeness with
`complete`, `total`, or truncation language unless the field is defined against the returned
array or a source completeness signal is established.

Record presence can distinguish a reported absent `trace.jsonl` from a present file, but it
cannot distinguish successful parsing from read failure, over-cap fallback, or malformed
content. Even a zero-byte present file is a positive filesystem-presence fact; it does not
provide a general parse-status field. The port may use presence to reject absent trace records,
but it must not claim that this solves all source-read ambiguity.

Accordingly, the first review's core evidence is upheld with a narrower formulation: the
proposal must define its counts/state as a projection of the returned `run.get` rows and must
not claim complete source-journal observation. The available metadata cannot make a true source
empty journal distinguishable from every unreadable/malformed case. This is a limitation of the
existing kernel view, not authorization to alter its semantics in this unit. A proposal repair
may transparently record the returned-row scope and this limitation; any new source-completeness
signal or Rust behavior change requires separate scope/authorization.

No tests, compiler, or source edits were made.
