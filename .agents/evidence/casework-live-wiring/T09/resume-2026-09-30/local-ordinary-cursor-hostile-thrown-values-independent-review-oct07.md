# Independent source review: hostile thrown-value repair

Date: 2026-10-07  
Disposition: **APPROVE source for the separate focused runtime review; no runtime or production approval**

## Scope, authority, and identities

Re-read the complete private ordinary-cursor implementation assignment and result record, accepted proposal revision 4, its independent review and citation erratum, original phase 1/revision 4 assignments, root bounds-RED01 acceptance, root atomic two-ordinal settlement decision, prior independent source rejection and citation correction, hostile-thrown-value repair assignment/result/hash correction, full current `localAdapter.ts`, the new hostile test, atomicity supplement, frozen order/bounds tests, existing local tests, and conformance fixture.

The previously rejected source is preserved as decoded UTF-8 JSON content. I decoded `local-adapter-828069-preimage-utf8-json-oct07.json` and verified 38,106 bytes with SHA-256 `828069b6c09d07c728778356c8921bfa553201f7882ffbbdfe83c86d96c21074`. The corresponding settlement test JSON decodes to 7,631 bytes with SHA-256 `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819`, and exactly matches the current settlement test. The package `.raw` backup files have an extra terminal LF and are correctly not claimed as byte-exact. A source-level unified diff against the decoded `828069` content shows only the new fallback constant and the guarded `asError` implementation changed in `localAdapter.ts`.

Frozen/current artifact hashes independently checked:

| Artifact | SHA-256 |
|---|---|
| `localAdapter.ts` | `19c767cee3da406f5729a29177f360638d2b6ac3830294126f564c5d857b4700` |
| `localAdapter.hostileThrownValues.test.ts` | `31f8a71c2fa550ef0ea6f22259e25c48742a836fea85e52616d6da3356913920` |
| `localAdapter.settlementAtomicity.test.ts` | `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` |
| frozen cursor-bounds test | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` |
| frozen cursor-order test | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| frozen conformance fixture | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |
| existing local adapter test | `07dbef23421395a251d6526febdab30b0ec51d0c87dc68f7a57d0de1c0fcc35b` |

## Prior finding and repair

The previous independent review found `asError` could itself throw while evaluating `value instanceof Error` (for a Proxy `getPrototypeOf` trap) or `String(value)` (for a hostile `Symbol.toPrimitive`/`toString`). In the `onEvent` catch, this skipped `onError`, allowing an uncaught timer callback. The flaw also affected other conversion sites.

Current `asError` (`localAdapter.ts:117-124`) puts both the `instanceof` check and string conversion inside one `try`; the catch returns an `Error` with a fixed generic message and does not inspect or coerce the thrown value again. A normal `Error` is returned unchanged; normal non-Error values retain their string message. The same helper call sites remain covered (`:474,686,752,775`). The fixed text is generic because the helper also handles operation and timer errors, as explained by the immutable hash correction.

The new test (`localAdapter.hostileThrownValues.test.ts:51-103`) exercises four actual thrown values: a null-prototype object, an object with throwing `Symbol.toPrimitive`, an object with throwing `toString`, and a Proxy with a throwing `getPrototypeOf`. It registers two real subscribers, runs the offender's controlled timer inside `expect(...).not.toThrow()`, drains the healthy subscriber, and asserts exactly one offending callback, exactly one fallback `Error`, no later callback/error to that disposed subscriber, and delivery of both the original and later event to the healthy subscriber. Manual timer globals are restored in `finally`, and subscription cleanup is also in that `finally` block.

The preserved bounds test already contains a normal `throw new Error('listener failed')` case and asserts that the original message reaches `onError` (`localAdapter.cursorBounds.test.ts:349-368`). The source returns ordinary Error instances unchanged, so the repair preserves that behavior. The test set is deterministic and synchronous: the offending subscriber is registered first, publication queues both subscribers, `runNext` fires the offender's timer, and `runAll` drains the healthy timer.

## Full candidate and residual matrix

The source diff is limited to `asError`; the previously reviewed ordinary-cursor algorithm remains in the candidate byte-for-byte relative to its rejected source except for this normalization change. I rechecked its previously reviewed matrix and atomicity integration: per-case ordinary frontier and exact seed match, opaque epoch spelling and safe decimal parsing, prepare-before-commit allocator, unique ordered progress/snapshot/settlement ordinals, two-ordinal atomic settlement with unchanged state on exhaustion, exclusive future subscription floors, FIFO-64 and disposal/error handling, and unchanged public contract/conformance. The new test adds the exact hostile coercion edge without cloning or otherwise changing shared event objects.

The existing shared mutable event-object risk remains explicitly outside this repair: `publishBatch` queues the same event object across subscribers. It remains a recorded scope/debt concern and is not promoted to a new gate or silently repaired here. No public interface, schema, dependency, config, HTTP behavior, conformance fixture, frozen test, or settlement source/test changed. The repair does not widen authority beyond private `LocalContractAdapter` ordinary cursor behavior.

## Evidence boundary

Approve this frozen source/test pair for root to authorize the separate focused hostile-thrown-value runtime review. The new test has not been run; this review does not claim GREEN behavior, compiler/typecheck success, the full UI matrix, canonical UI gates, production integration, or T09 settlement. No Bun, compiler, typecheck, test, scanner, Git, or runtime command was run for this review. The byte-exact source preimage evidence is the decoded JSON record, not the package `.raw` copies.

Graft first-pass retrieval saved approximately 32,329 tokens (~$0.03) this turn.
