# Focused local cursor attempt 01 — bounded fixture repair result

Date: 2026-10-07. Three authorized test-only expectation corrections. No tests, Bun, compiler, typecheck, scanner, formatter, Git, or runtime command was run.

## Authority and assignment

The full original implementation instructions remain in
`local-ordinary-cursor-implementation-original-record-oct07.md` (SHA-256
`21f835418a4d3380469309abc3d8f6d4bee421c6d9723e5870bed8f18364d9b019`),
with accepted revision 4, root settlement decision, and the focused attempt 01
independent review. The exact bounded repair authorization from root was:

> ROOT EXPLICITLY RELEASES only: cursorOrder373a line279 timers.runNext after
> stopFirst => expect(timers.hasPending()).toBe(false), preserving ALL
> subsequent delivery/case-local assertions (can await bounded drain
> additionally if useful); cursorBounds134ba line475 and settlement35f2 line99
> add explicit expect(response.new_cursor).toBeDefined() immediately before
> equality then response.new_cursor! comparison. Preserve every other
> assertion/logic; no weakened tests. No edits production source19c/hostile31f/
> conformanceedf, no Next/manager changes. Preserve all3 EXACT immediate
> preimages before edit via nativepatch UTF8JSON wrapper+decodedhash verification
> (NOT extraLFrawcp). New immutable full original instructions record BEFOREedit
> and result hashes/exact bounded diff. SOURCEONLY no runtime/compile/typecheck/
> scanner/Git. Root will differentmanager_revision sourcecritic and sole-heavy
> rerun. If lackspermissionsread-onlyJSON root can help. Keep evidence immutable
> including6a24; its phrase rejectasverification means no GREEN, actualfailure-
> captures remainvalidobservations.

**Chronology deviation:** I did not create a separate task-specific original-
instructions artifact before editing. I read the pre-existing full original
implementation record and the root authorization above before making changes;
the required exact source preimages were created and decoded-hash verified
before edits. This result records the authorization after the edits and does
not claim it preceded them. No earlier record was overwritten.

## Exact preimages preserved before editing

Each package-local JSON wrapper was added with native `apply_patch` before any
test file edit. Decoded UTF-8 content hashes and byte counts were verified
against the frozen assignments:

| Original fixture | Pre-edit bytes | SHA-256 | Exact wrapper |
|---|---:|---|---|
| `localAdapter.cursorOrder.test.ts` | 13,835 | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` | `localAdapter.cursorOrder.before-focused-attempt01-repair-oct07.json` |
| `localAdapter.cursorBounds.test.ts` | 23,769 | `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a` | `localAdapter.cursorBounds.before-focused-attempt01-repair-oct07.json` |
| `localAdapter.settlementAtomicity.test.ts` | 7,631 | `35f2a8c93a5213fc22ac0a6fc4ee2963e330eebc3c951125e6d242c448775819` | `localAdapter.settlementAtomicity.before-focused-attempt01-repair-oct07.json` |

All wrappers include the source path, expected source hash, and exact UTF-8
content. Decoding each wrapper reproduces the recorded source hash.

## Exact changes and resulting identities

The unified diff against each exact decoded preimage contains only the
following authorized edits:

1. `localAdapter.cursorOrder.test.ts`: after `stopFirst()`, replace
   `timers.runNext()` with `expect(timers.hasPending()).toBe(false)`. The
   unsubscribe has canceled the only queued Northstar timer and the second
   case has no event queued. All subsequent no-delivery and independent
   case-local cursor/delivery assertions remain unchanged.
2. `localAdapter.cursorBounds.test.ts`: immediately before comparing
   `startSnapshot!.cursor` to the dispatch response, assert
   `expect(response.new_cursor).toBeDefined()`, then compare against
   `response.new_cursor!`.
3. `localAdapter.settlementAtomicity.test.ts`: add the same explicit
   `new_cursor` presence assertion immediately before its equality, followed
   by the non-null comparison. No settlement or cursor assertion was removed.

| Artifact | SHA-256 after repair |
|---|---|
| `localAdapter.cursorOrder.test.ts` | `b2a8a22b123382fe6b0fd788c4f385de77ee58bccfee58cdf32af272af0ec552` |
| `localAdapter.cursorBounds.test.ts` | `a5ca9298467460a32350e85dee1c412a0b4a636efbe8ca3786ce4ab0802390f5` |
| `localAdapter.settlementAtomicity.test.ts` | `ef938f96e04a04198793fa65d1a1331b874873314a3cd07fddd7db30babe674c` |

The production `localAdapter.ts` remains SHA-256
`19c767cee3da406f5729a29177f360638d2b6ac3830294126f564c5d857b4700`; the
hostile-value test remains `31f8a71c2fa550ef0ea6f22259e25c48742a836fea85e52616d6da3356913920`;
conformance remains `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`.
No production code or other file was changed.

## Evidence limits and next review

The attempt 01 independent review rejected the captured test result as a
verification attempt because the old cursor-order fixture called `runNext`
after unsubscribe had canceled its only timer. It also identified the two
optional-cursor type assertions fixed above. This repair changes test
expectations only; it does not convert the rejected attempt into GREEN, does
not erase its actual failure captures, and does not make a new runtime claim.
All test and production behavior remains unverified here. A different source
critic and root-owned focused rerun/typecheck remain required. No extra
assertion, fixture change, production behavior, public contract, dependency,
configuration, or manager behavior was changed.
