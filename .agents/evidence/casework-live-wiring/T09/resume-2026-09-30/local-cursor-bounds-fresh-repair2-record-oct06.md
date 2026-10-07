# Local cursor bounds fixture — fresh repair 2 record

Date: 2026-10-06. TESTONLY repair; no runtime or compiler claim.

## Assignment and review inputs

The original repair instructions were:

> Freshdifferent TESTONLY repair builder vs3a author. Read fullcursorBounds3a+
> fresh-repair-record+new independent reviewc07a and originalproposal4/
> testassignments. Preserveactual3a originalbytesNEWfile BEFOREedit.
> ONLY editboundsfixture (no373a/prodd8/conformanceedf/config). Fixall5review
> defects minimal: subscribehelper caseIDparam withNorthstardefault, templatecase
> usesactualreturnedcaseId; deriveH numerically lastsnapshot, leading-zero
> boundaryH+1 suppress equality and thenH+2onlydelivery, preserveopaqueemitted
> strings; slowwatcherzeroerrors BEFORE65 inBOTHoverflow/onErrorthrow tests
> exact64FIFO; frontierBefore deepcopy scalarobject notalias; settlementtest
> actualnormalexecutionstart first viaexistingpublicdispatch+controlled timers
> precedentcursorOrder, identifygenuineofferedEXECUTE_ITEM target dynamically,
> thenseedbothhistory+optionalordinaryFrontiers toceil AFTERstart+initialsnapshot
> delivery. Captureexecutiontimer handle range/newregistrations and chooseactual
> settlementcallback deterministic schedule (maxdelay/lasttimerofexecution), no
> globalhardcoded7. Rootdirectsourceexecute348–388 confirmsprivateexecute itself
> schedules7timers withoutinitialappend; avoidincorrectclaim itinitialappendsthrows.
> Futureprivatefrontierfixtureseam rootallowedconditionalmap {epochText,epoch,
> sequence}; no baseline nonexistent-map setup error. DoNOT weakenassertions/tests.
> Nativepatchpersistwrites; NEW boundedrepairrecordoriginalinstructions+all
> changes/limitations/hashes. NO tests/typecheck/compiler/git. Differentcritic+
> rootbefore ONEactualRED.

This is a fresh fixture repair, different from the author of cursor bounds
fixture 3a. The reviewed test was
`apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts`
SHA-256 `3a801b86b5e39457f6b057777bbd29ea535634005eb8bb0211d2d888aaf14e11`.
Before editing, its complete original bytes were copied to
`apps/godspeed-cognitive-ui/src/adapters/local/localAdapter.cursorBounds.test.ts.3a801b86.original.raw`;
that preserved copy has the same SHA-256.

Inputs read: original cursor revision 4 assignment
`local-subscription-ordered-cursor-revision4-original-assignment-oct06.md`
(SHA-256 `0af4db3bfee9d17621eb8f6a1e1f8ebb18c11b456b2f6140accb2a290b31d7b8`),
the complete revision 4 proposal
`local-subscription-ordered-cursor-concrete-proposal-revision4-oct06.md`
(SHA-256 `6d7baafd41a098cc24556b32ea222c999ffe4a116998740339acd2086131873`),
the test-only original assignment
`local-cursor-order-fixture-repair-original-assignment-oct06.md` (SHA-256
`f01bfe20e2ebc48fe94b5c4b4107d833fe6aa352ea06cd1c875f3272459c1cca`),
prior 3a repair record (SHA-256
`2e344112bc186b23e46ee7a6e0561a6c306abe27e6817e9299e43cab4afc252f`), and
the complete independent review of 3a
`local-cursor-bounds-fresh-repair-independent-source-review-oct06.md` (SHA-256
`c07a952fb912b8f48370b41edacde8e00638d14b8cadc0ce284a7ba21981bb39`).
The original assignment and proposal remain unchanged.

## Repairs made

Only the cursor-bounds test fixture was edited. The five source-review defects
were repaired as follows:

1. `subscribe` accepts an optional case ID defaulting to Northstar. The
   template-commit test passes the newly returned committed case ID, so its
   listener and appended event belong to the same case.
2. The leading-zero future-floor test reads the final snapshot cursor, derives
   numeric `H`, subscribes at a zero-padded numeric-equivalent `H+1`, verifies
   that exact boundary allocation is suppressed, and verifies only the genuine
   `H+2` event arrives. Assertions compare the exact stored/emitted cursor
   strings; they do not normalize emitted values.
3. Both slow-watcher capacity tests assert zero overflow/error callbacks after
   exactly 64 queued events, before attempting event 65. Existing 64-event
   FIFO/healthy-watcher assertions and the 65th-event disposal assertions stay
   intact.
4. The optional ordinary-frontier state is copied by value before the failed
   allocation; the post-failure equality assertion no longer aliases the
   object under test.
5. Settlement exhaustion now obtains the actual offered `EXECUTE_ITEM` target
   from the initial snapshot and starts it through public `dispatchIntent`.
   Controlled timers deliver and verify the start snapshot before the fixture
   seeds the retained snapshot, trajectory point, and optional existing
   `ordinaryFrontiers` entry to the ceiling. Timer registration handles after
   dispatch are captured; the settlement callback is selected as the final
   greatest-delay timer in that execution's captured registrations, not by a
   global hardcoded handle. The starting snapshot is cleared from the
   assertion's event collection before checking that the exhausted settlement
   emits no event.

The timer fixture now records each scheduled delay and exposes registrations
after a captured handle to support that deterministic selection. It does not
modify production scheduling or add a source seam. Root directly inspected
`localAdapter.ts:348-388`: the private execution path schedules its sequence
of seven callbacks and does not itself append the initial `EXECUTE_ITEM`
snapshot. This repair does not claim that it does; the test exercises the
public dispatch path that produces the start snapshot before the timer path.

The optional private `ordinaryFrontiers` fixture seam remains conditional. If
the map is absent on the current baseline, setup does not fail; when present,
the existing helper writes `{ epochText, epoch, sequence }`. No nonexistent-map
setup-error assertion or public hook was added.

## Identities and scope

Repaired fixture SHA-256:
`3d3e1da51057fafde43508b76ef4e8a33d2f4f91324924ec2abddd53db62f576`.

Preserved/current identities:

| Artifact | SHA-256 |
|---|---|
| Frozen original 3a fixture bytes (`.3a801b86.original.raw`) | `3a801b86b5e39457f6b057777bbd29ea535634005eb8bb0211d2d888aaf14e11` |
| Production `localAdapter.ts` | `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34` |
| Frozen cursor-order fixture `localAdapter.cursorOrder.test.ts` | `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1` |
| Shared conformance fixture `caseworkPortConformance.ts` | `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c` |

No production, cursor-order, conformance, configuration, schema, public
interface, or runtime file was edited. Prior rejected fixture bytes and
reviews remain preserved. No assertions were weakened or removed; the direct
private settlement invocation was replaced by public dispatch and controlled
timer handling to exercise the intended start lifecycle.

## Verification boundary and next step

No Bun tests, typecheck, compiler, build, Git, or network command was run. The
fixture still has no independent approval or runtime RED result. A different
critic must inspect the original assignment, full cursor revision 4 proposal,
review c07a, preserved original 3a bytes, this repaired fixture, and this
record. Root must review that verdict before authorizing one actual RED run.
No production source, conformance, or configuration edit is authorized by
this repair.
