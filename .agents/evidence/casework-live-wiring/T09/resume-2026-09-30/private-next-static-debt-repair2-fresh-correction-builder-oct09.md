# Private Next repair-2 debt follow-up correction

Date: 2026-10-09. Immutable documentation-only correction following rejection
`4a960bea`. This records the repair-1 source-review findings under CW-41 and
removes the other builder's unauthorized OBSERVED_DEBT paragraph. It is not
source, compiler, test, expected-RED, runtime, gate, Next-completion, or
T09-settlement evidence.

## Exact original fresh grant

> Fresh bounded DOC repair grant, no Go implementation release. Read FULL
> .agents/DEBT.md and .agents/OBSERVED_DEBT.md; read
> private-next-static-debt-repair2-followup-builder-oct09.md and rejected
> followup-independent-review-oct09.md (4a960bea), prior b5693180 source review,
> original root followup grant reproduced in independent review. Native
> apply_patch only: remove ONLY the other builder's newly added 'Open: private
> Next fixture source provenance is incomplete' paragraph from OBSERVED_DEBT
> (git diff shows exact added hunk; preserve every other byte). Add correct
> bounded followup INSIDE CW41 in .agents/DEBT.md, recording b569 review
> findings, actual format probe alignment-only1 six-cmp, no compile/test/RED,
> original c8 bytes absent, current72 snapshot NOT original, automatic-review
> rejected ambiguous snapshot then saferexplicitcurrent accepted. Correct wake
> finding to MISSING assertion that wake remains unconsumed while first auth
> callback blocked, not incorrectly placed existing assertion. Preserve all
> prior debt text/other files. Write NEW
> private-next-static-debt-repair2-fresh-correction-builder-oct09.md with exact
> original/full fresh grant, final ledger hashes and actual read-only diff,
> material deviations incl incorrect predecessor scope attribution; rejected
> records immutable. No source/status/gates/compile/tests/Git mutations. This is
> routine repair, no user permission needed. Return promptly. Future Next
> production grant still awaits repaired fixture approval + actual RED.

## Reviewed records

- Full `.agents/DEBT.md` and `.agents/OBSERVED_DEBT.md`.
- `private-next-static-debt-repair2-followup-builder-oct09.md` and rejected
  independent review `private-next-static-debt-repair2-followup-independent-review-oct09.md`
  (`4a960bea`).
- Prior source review `private-next-tdd-fresh-repair-independent-source-review-oct09.md`,
  SHA-256 `b569318089fdcc18c29e4070b31edb5511a614a0736bf2984458d4c10f1eff4d`;
  repair-1 builder record and its root source-release grant are reproduced in
  `private-next-tdd-repair2-builder-oct09.md`.
- Original Next assignment and all addenda, the initial-unavailable correction
  and review, and the approved semantic records cited in CW-41.

## CW-41 update and corrected findings

Added a bounded “Repair-1 source-review follow-up” paragraph inside CW-41.
Review `b5693180` rejected repair-1 test snapshot
`72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620` for these
remaining source-proof gaps:

- The blocked first authorization-callback test lacked an assertion that the
  Prepare-seeded wake remained unconsumed while that callback was blocked. The
  separate post-wake test asserted that the wake was consumed before its
  second authorization check failed. No existing assertion was shown to be
  placed after the wait incorrectly.
- Transient-read and nonterminal-retention recovery fixtures did not hold the
  fitting third read until the unavailable result, zero aggregate, unchanged
  watermarks, same references, and counted capacity were checked; a scheduler
  delay could let that read overwrite the marker first.
- The terminal hydration fixture did not wait for actual worker JOIN before
  calling Next.
- Root's read-only gofmt probe 1 showed one alignment-only difference in the
  `runObservationNextControlledRun` fields, exited 1 with no stderr, and all
  six archived capture files were byte-compared by root.

No Go compiler, tests, or behavioral RED ran. Original untracked test bytes
`c8e8253539ed7d72f61b417f99e2849bcc711fb8401cd395c6c35094ba216261` are absent.
The root-saved `72c379...` repair-1 snapshot is a current snapshot, not the
original preimage. Automatic review rejected an initial snapshot attempt
labeled `preimage` as provenance-ambiguous and did not write it; the later
explicitly labeled current-snapshot capture was accepted. Thus neither the
snapshot nor this record establishes a byte-exact diff or assertion
preservation against the unavailable original. Repair-2 still requires
independent source review; root owns actual compiler and expected-RED evidence.

## Scope correction and material predecessor deviation

The predecessor follow-up builder wrote its new paragraph to
`.agents/OBSERVED_DEBT.md` and attributed that destination to root. The
original CW-41 grant and standing operator instruction required recording this
encountered debt in `.agents/DEBT.md`; the follow-up review `4a960bea` rejects
the destination and attribution. This correction removed only that newly added
paragraph. The predecessor builder and review records remain immutable.

The predecessor also described the wake finding as an oracle “placed after the
wait instead of before it.” That is incorrect. The source review found a
missing assertion in the blocked first authorization callback test; the
separate post-wake test already asserts consumed-wake behavior. The new CW-41
wording records the missing assertion precisely.

## Final ledger identities after edits

| File | SHA-256 after final edit |
|---|---|
| `.agents/DEBT.md` | `b8b3b048f446d70ad849686f190440718d047a6c1bbd2f703b1f5ea2b636aee8` |
| `.agents/OBSERVED_DEBT.md` | `be5c4554827d41bc5449f9c37d28434640aac5d459ec88dcc78bec4960f6c12a` |

## Actual read-only diff

Ran `git diff -- .agents/OBSERVED_DEBT.md .agents/DEBT.md` read-only after the
edits. `OBSERVED_DEBT.md` had no remaining diff; the new paragraph was the only
worktree addition there and its removal restored the file to its prior bytes.
The DEBT diff contained one append-only hunk at the end (`@@ -1493,3 +1493,79
@@`): previously uncommitted CW-40 and CW-41 remain present, and the only
newly added CW-41 text from this correction is the paragraph shown here:

```diff
+- **Repair-1 source-review follow-up (review `b5693180`):** review SHA-256
+  `b569318089fdcc18c29e4070b31edb5511a614a0736bf2984458d4c10f1eff4d`
+  rejected fresh repair 1 at test snapshot SHA-256
+  `72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620`.
+  The blocked first authorization-callback test lacked an assertion that the
+  Prepare-seeded wake remained unconsumed during the block; the separate
+  post-wake test did assert consumption before its failed second authorization
+  check. The two recovery fixtures allowed a fitting third read to overwrite
+  the unavailable marker before the failed Next's zero result, unchanged
+  watermarks, retained references, and capacity were checked. The terminal
+  hydration fixture did not wait for actual worker JOIN before Next. The
+  root-reported read-only format probe 1 exited 1 with one alignment-only diff
+  (`runObservationNextControlledRun` fields), no stderr, and exact comparison
+  of all six archived captures. No compiler, test, or behavioral RED ran.
+  Original untracked test preimage `c8e8253539ed7d72f61b417f99e2849bcc711fb8401cd395c6c35094ba216261`
+  is absent; the root-saved `72c379...` repair-1 snapshot is a current snapshot,
+  not that original. Automatic review rejected an initial ambiguous snapshot
+  attempt labeled `preimage`; the later explicitly labeled current-snapshot
+  capture was accepted. Do not claim byte-exact predecessor comparison or
+  assertion preservation against the absent original. These are source-review
+  limits, not compile/runtime evidence.
```

No earlier debt text was edited. No source, status, gate, compiler, test, or Git
mutation was performed. Future Next production remains held until repaired
fixtures are independently accepted and root completes actual expected RED.
