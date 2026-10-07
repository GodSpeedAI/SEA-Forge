# Independent source review: focused cursor fixture repair

Date: 2026-10-07  
Disposition: **SOURCE-APPROVE the three narrowly authorized test-only edits for a fresh root-owned verification attempt. This is not runtime, typecheck, or implementation approval.**

## Scope and authority

Reviewed the original implementation record, accepted revision 4 cursor proposal, root settlement atomicity decision, attempt 01 independent review, fixture repair result and its citation correction. Root explicitly released exactly three test-only edits: replace the post-unsubscribe `runNext()` with a no-pending-timer assertion, and add a `new_cursor` presence assertion before each of the two cursor equality checks. The release required preserving every other assertion and logic, preserving exact preimages, and prohibited runtime/compile/typecheck/scanner/Git work.

The repair result records an important process deviation: its author did not create a separate task-specific full-instructions record before editing. The original full implementation assignment existed already and was read, and the repair result records that the exact pre-edit JSON source wrappers were created and decoded-hash verified before edits. I verified those wrappers against their current fixtures and the recorded original hashes. This is a real documentation-sequencing miss; it is not represented here as compliance. It does not change the root's explicit bounded authorization or the observable source diff. The missing contemporaneous assignment artifact cannot be reconstructed as if it were written before editing.

## Preimage and candidate identity verification

I decoded the three package-local JSON wrappers and compared each decoded UTF-8 source directly to the corresponding current fixture. The decoded bytes and hashes match the frozen original assignments, and each candidate diff contains only the released hunk:

| Fixture | Preserved preimage hash | Current hash | Source diff |
|---|---|---|---|
| `localAdapter.cursorOrder.test.ts` | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` | `b2a8a22b123382fe6b0fd788c4f385de77ee58bccfee58cdf32af272af0ec552` | one `runNext()` → `expect(timers.hasPending()).toBe(false)` replacement |
| `localAdapter.cursorBounds.test.ts` | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` | `a5ca9298467460a32350e85dee1c412a0b4a636efbe8ca3786ce4ab0802390f5` | presence assertion inserted; cursor equality retained with non-null narrowing |
| `localAdapter.settlementAtomicity.test.ts` | `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` | `ef938f96e04a04198793fa65d1a1331b874873314a3cd07fddd7db30babe674c` | presence assertion inserted; cursor equality retained with non-null narrowing |

The wrappers themselves hash to `a2201cf8249f73faf28f0c8c9f478eb8e654929ff833ca3a68f77e368de5b995`, `457ec83b96fd38b1452b1cdea4d0b89823841bb33c30041ca6ee6e904b664ac9`, and `6b9943bb9a8667452f1e19d592ab6551649a2dcb5014d9f9eb90b51e5ff44f30`, respectively. They are package-local review artifacts, not production inputs.

The production adapter remains at SHA-256 `19c767cee3da406f5729a29177f360638d2b6ac3830294126f564c5d857b4700`. No other production algorithm or test change is in this repair's reported scope. The recorded hostile-value fixture hash remains `31f8a71c2fa550ef0ea6f22259e25c48742a836fea85e52616d6da3356913920`; the prior review's frozen conformance hash is `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.

## Semantic review of the three hunks

1. In `localAdapter.cursorOrder.test.ts`, the Northstar event is queued, then `stopFirst()` unsubscribes the only subscriber with queued work. The Template subscriber has no work yet. The accepted contract and existing bounds test both require unsubscribe to cancel the timer and clear queued delivery. The new `hasPending() === false` assertion checks that exact behavior and lets the later independent Template-case cursor allocation and delivery assertions run. Those later assertions remain byte-for-byte unchanged. This corrects the harness contradiction identified at line 279; it does not claim their runtime success.
2. In `localAdapter.cursorBounds.test.ts`, the new `toBeDefined()` assertion precedes the equality check against the snapshot cursor. The non-null assertion only narrows the same value for TypeScript; it does not make a missing cursor pass because the presence matcher remains a runtime assertion.
3. In `localAdapter.settlementAtomicity.test.ts`, the same narrowing pattern preserves the successful-dispatch cursor requirement and equality assertion. The root's two-ordinal settlement rule and atomicity assertions are unchanged.

No assertion was removed or weakened; no ordering, setup, timer policy, event expectation, state mutation, or production behavior was changed. The captured attempt 01 runtime/typecheck observations remain immutable. This fixture repair does not convert those captures into GREEN evidence.

## Material differences and limits

The source repair matches all three released hunks and preserves the frozen fixture contents as recoverable exact preimages. The only material process difference is the missing new task-specific instructions artifact created before edits, acknowledged above. Because the authorization was explicit and the source wrappers establish the precise preimages, this source review finds no scope expansion or concealed change. The record should remain a transparent after-the-fact account, not be backdated or edited to claim otherwise.

No compiler or runtime verification was performed. This review approves only that these source edits are suitable for the separately authorized, root-owned verification attempt. It says nothing about whether the implementation passes, whether every test assertion is reached, or whether the broader cursor contract is satisfied.

## Evidence read

- `local-ordinary-cursor-implementation-original-record-oct07.md` (corrected SHA-256 `21f835418a4d3380469309abc0f6d4bee421c6d9723e5870bed8f18364d9b019`).
- `local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md` (SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4ea116998740339acd2086131873`).
- `local-settlement-two-ordinal-atomicity-root-decision-oct07.md`.
- `ui-private-focused-attempt01-independent-review-oct07.md` (SHA-256 `6a24f619f99e37767bd53cf3b79dab5b400535481cbaf292f25afc588087530a`).
- `local-cursor-focused-attempt01-test-fixture-repair-result-oct07.md` and its separate citation correction.

Review was source-only. No files were changed except this new review record; no tests, compiler, typecheck, scanner, formatter, or Git command was run.
