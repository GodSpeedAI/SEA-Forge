# Local ordinary cursor implementation result

Date: 2026-10-07. This is a new completion record for the private local cursor
implementation. Earlier assignment and root-decision records remain immutable.

## Authority and scope

The work follows the accepted `local-subscription-ordered-cursor-concrete-
proposal-revision4-oct06.md` (SHA-256
`6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`), its
independent review and citation erratum, the root acceptance record
`ui-cursor-bounds-red01-root-acceptance-oct07.md`, and the implementation
assignment in `local-ordinary-cursor-implementation-record-oct07.md`.
The later root settlement decision is preserved verbatim in
`local-ordinary-settlement-atomicity-decision-oct07.md` (SHA-256
`af9732f2061ee94141e069fc62b9a1f2ba89b229ee35edeb75b80d9f5cd32077`).

The only code paths changed are `localAdapter.ts` and the newly scoped
`localAdapter.settlementAtomicity.test.ts`. No public contract, schema,
dependency, configuration, conformance fixture, frozen cursor-order test, or
frozen cursor-bounds test was changed. The original full assignment and
pre-edit source identity remain in the immutable implementation record.

## Implementation summary

`LocalContractAdapter` now owns a private ordinary cursor frontier per case.
Seed snapshot and final trajectory cursors must match exactly; their original
epoch text is retained. Snapshot appends, progress events, and settlement
side-events allocate from this same frontier. Candidate data, clones, history
arrays, events, and publication records are prepared before the synchronous
commit. Subscriber notifications are queued after commit and callbacks are
never invoked inside it.

Settlement is one atomic logical operation. It reserves two adjacent
ordinals, prepares the settlement snapshot/history and the `settlement_recorded`
side-event, and commits the final frontier only after both are ready. If fewer
than two ordinals remain, preparation fails before the snapshot, trajectory,
side-event, or frontier changes; active subscribers receive one safe error.
For success, subscribers receive the snapshot cursor first and the distinct
side-event cursor second. Only the snapshot is retained for exact historical
lookup.

Subscriptions capture the then-current frontier, combine it with a supplied
valid floor, and receive only later events. Invalid subscriptions report one
error and do not become active. Each active watcher has a FIFO limit of 64,
isolated callback/error handling, one drain timer, and per-watcher disposal on
overflow or callback/timer failure.

## Supplemental settlement cases

The new test file starts execution through public `dispatchIntent`, waits for
the actual start snapshot, and selects the actual settlement timer from the
execution timers by its greatest delay. It then exercises:

1. Frontier `1.9999999998`: only one ordinal remains; expects one error, no
   settlement events, and unchanged snapshot history, trajectory, and
   allocator.
2. Frontier `1.9999999997`: two ordinals remain; expects snapshot
   `1.9999999998`, then side-event `1.9999999999`, and exact historical lookup
   only for the snapshot.

These tests were written before the atomic settlement source change. They are
new coverage requirements, not baseline-red evidence. They have not been run.

## Artifact identities

| Artifact | SHA-256 |
| --- | --- |
| `localAdapter.ts` after source changes | `828069b6c09d07c728778356c8921bfa553201f7882ffbbdfe83c86d96c21074` |
| `localAdapter.settlementAtomicity.test.ts` | `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` |
| Initial assignment record (immutable) | `21f835418a4d3380469309abc0f6d4bee421c6d9723e5870bed8f18364d9b019` |
| Root settlement decision record (immutable) | `af9732f2061ee94141e069fc62b9a1f2ba89b229ee35edeb75b80d9f5cd32077` |
| Frozen cursor-order test (unchanged) | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| Frozen cursor-bounds test (unchanged) | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` |
| Existing local adapter test (unchanged) | `07dbef23421395a251d6526febdab30b0ec51d0c87dc68f7a57d0de1c0fcc35b` |
| Casework conformance fixture (unchanged) | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |

The source pre-edit SHA-256 is
`d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`.
When the atomicity clarification arrived, an intermediate sequential-settlement
draft had SHA-256
`989e11a831e482087d28f52bae7560a9608e61b44cd6d54e87e6d3dbbcdf603f`; it was
not treated as complete and was replaced by the atomic operation described
above. No test, compiler, Bun, typecheck, Git, runtime, or gate command has been
run in this task. Source has not received independent review or root approval
yet; do not treat this record as a test or merge authorization.

## Review matrix for the next reviewer

| Obligation | Implementation/test location | Evidence status |
| --- | --- | --- |
| Shared per-case allocator for snapshot and progress events | `allocateOrdinaryEvents`, `append`, `progress` | Source review pending |
| Exact seed cursor agreement and preserved epoch text | `localCursorSeed`, `installCase` | Source review pending |
| Exclusive future subscription floor and invalid-subscription error | `subscribeEvents` | Source review pending |
| FIFO64 and isolated watcher failures | `enqueue`, `scheduleDrain`, `drainSubscriber`, `disposeSubscriber` | Source review pending |
| Settlement two-ordinal reservation and no partial data commit | `recordSettlement`, `allocateOrdinaryEvents` | Source review pending; supplemental tests not run |
| One ordinal remaining rejects without mutation | supplemental atomicity case 1 | Not executed |
| Two final ordinals commit in order and remain distinct | supplemental atomicity case 2 | Not executed |
| Existing frozen cursor-order and cursor-bounds assertions preserved | their original files and hashes above | Hash-checked only; not executed here |
| Public contract and conformance unchanged | contract/conformance paths untouched | Hash-checked conformance only |

Graft first-pass context retrieval saved approximately 40,611 tokens
(approximately $0.03) for this task. This is context retrieval accounting, not
verification evidence.
