# Live cursor contract correction: review proposal

**HOLD: proposal only.** No canonical contract, public schema, ID generator or
runtime code change is authorized by this document. Independent critique and
operator approval under root AGENTS section4/spec change_rule precede changes.

## Evidence and reason

Independent source trace `live-cursor-contract-independent-recon-oct05.md`
(`c7fabaaa`) establishes that actual ledger/event/world/SSE/intent cursors are
26-character uppercase Crockford ULIDs. Canonical event/world/intent schemas
permit only decimal epoch plus dot plus ten-digit sequence. T09 requires the
latest real case cursor for observation events. A strict validator following
the existing pattern therefore rejects valid production observations.

The same mismatch affects ordinary live snapshots and client freshness cursors;
an observation-only regex replacement cannot correct the shared envelope.
T09's redesign trigger requires returning to the T01 contract boundary when
journey data is not represented. Existing T01/T09 confirmations remain immutable;
this supplemental correction must not silently replace their claims.

## Proposed bounded correction

Accept both the existing logical cursor form and the actual ledger ULID form
in authored canonical cursor validators:

`^(?:[0-9]+\.[0-9]{10}|[0-7][0-9A-HJKMNP-TV-Z]{25})$`

Apply the same rule to the existing world cursor, intent client_cursor, shared
event cursor and named execution_observation cursor definitions. Update directly
related canonical comments/docs and the ADR/spec before implementation, then
refresh generated mirrors through their established source/generator workflow.
Keep cursor strings and existing schema identifiers; this is an additive
validation correction for the already emitted format. The critic must assess
whether the existing schema/version conventions require an additional version
change before this choice is approved.

Preserve real cursor bytes in relay, Store, snapshot, intent freshness, history
lookup and SSE. Observation JSON uses the latest real CASE cursor, carries no
SSE id and never advances replay/history. Defer observation emission while that
case cursor is unknown; neither global head nor a synthetic substitute is valid.
The two accepted formats do not establish an ordering relation between mixed
producer histories; no such guarantee is added. Keep existing runtime source
ordering and identity generation within their current boundaries.

## Required proof and review

Identify every authored/generated cursor constraint and affected fixture before
editing; do not broaden arbitrary strings or change unrelated interface fields.
Positive schema/runtime-validator fixtures cover actual emitter-format ULIDs and
existing logical cursors. Reject wrong length, lowercase/non-Crockford alphabet,
overflow-leading ULID character, malformed logical width and arbitrary strings.
Copy a real temporary-cell event cursor verbatim into world/event/intent schema
proofs. Verify unchanged ordinary SSE resume, same-cursor no-ID observation
delivery, intent stale comparisons and frozen history. Run canonical parity,
mirror-generation and applicable Go/UI gates without weakening existing tests.

An independent critic must review the original instructions, source recon,
actual proposed constraints, compatibility/version scope, source truth and all
material deviations. Proposal approval is not implementation approval. Record
the resulting operator decision durably before builder release. Other approved
trace-port and push-prerequisite work may continue while this boundary is held.
