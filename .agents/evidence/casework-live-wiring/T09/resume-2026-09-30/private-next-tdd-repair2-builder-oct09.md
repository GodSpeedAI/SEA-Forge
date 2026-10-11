# Private Next TDD fresh repair 2 builder record

Date: 2026-10-09. Immutable source-preparation record for the narrow repair
following the accepted static source rejection. No compile, test, RED, runtime,
gate, Next completion, or T09 settlement is claimed.

## Full source-release grant

> SOURCE RELEASE for fresh narrow fixture repair2. Root FULL-read accepted
> REJECT b569318089fdcc18c29e4070b31edb5511a614a0736bf2984458d4c10f1eff4d in
> private-next-tdd-fresh-repair-independent-source-review-oct09.md; read FULL
> plus original assignment/all addenda/approved correction/prior builder
> packet. Only next_test.go, current72c37999. Exact current repair1 snapshot
> saved+root decoded/cmp at
> private-next-current-repair1-72c37999-source-snapshot-oct09.json (NOT original
> c8e825; that preimage unavailable, no old exact preservation claim). Five
> narrowly required changes: (1) blocked first Next auth callback asserts
> seeded wake remains during blocked pre-wait auth callback; (2) transient read
> recovery third read gate, held until failure/no-WM/same-ref/capacity
> assertions, then release actual fitting read; (3) analogous NONTERMINAL
> retention recovery gate +ownership checks, preserve separate terminal drain
> case; (4) eight terminal hydration workers all actual workerDone JOIN before
> Next, retained current still exact refs; (5) apply exact alignment-only gofmt
> diff captured /tmp/next-fixture-format-probe01-5qi6demv/stdout.raw or archived
> json. Once-backed cleanup for all gates even failures, worker cancellation
> safe. Preserve all other current72 assertions and accepted preNext tests. No
> production/scaffold edits, no compile/tests/formatter writes/gates/Git
> mutations/status/debt. Native apply_patch only. Freeze, then NEW
> private-next-tdd-repair2-builder-oct09.md with full grant refs, all4 finalSHA,
> exact diff vs decoded ACTUAL72 snapshot, finding matrix+materialdifferences+
> provenance limit. No claim compile. Report promptly, root settles blockers.

## Full original assignment and approved addenda

The operative assignment packet remains the full
`run-observation-private-next-assignment-oct09.md`, plus
`run-observation-private-next-assignment-addendum-oct09.md`,
`run-observation-private-next-notifier-proof-addendum-oct09.md`, and
`run-observation-private-next-projection-failure-addendum-oct09.md`. The
approved initial failure clarification is
`private-next-initial-unavailable-fixture-correction-oct09.md`, independently
approved in
`private-next-initial-unavailable-correction-independent-review-oct09.md`
(review SHA-256
`060a5349b1e3538b98ba9c9581f19ca5a6cfdb931edf5c2c68d1898d94146842`). The
requirements remain fixture-only scope, preserved current assertions, no fake
watermarks, real recovery only after accepted current attachment, terminal
retained-current delivery after worker JOIN, and no compiler/gate claim from
source preparation.

## References and provenance

I read the full accepted rejection
`private-next-tdd-fresh-repair-independent-source-review-oct09.md` (SHA-256
`b569318089fdcc18c29e4070b31edb5511a614a0736bf2984458d4c10f1eff4d`), the
preceding packet `private-next-tdd-fresh-repair-builder-oct09.md` (SHA-256
`a3e5d1fb8c82cf80d4bd9b602242f92d8858299e2dafb1b87ed55004dcc051f6`), the
original TDD builder result and static rejection, every assignment/addendum
listed above, the initial-unavailable correction and approved review, and the
complete test file before editing.

The root-saved repair1 snapshot is
`private-next-current-repair1-72c37999-source-snapshot-oct09.json`, SHA-256
`c20c046de680438d732c2f69803560c9b28170bea9796092f0aa9818e13ecb54`; metadata
declares embedded source SHA-256
`72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620`. I decoded
its xz/base64 payload to `/tmp/next-fixture-repair2-base72.go`; `cmp` confirmed
it byte-matched the source before editing. This actual72 snapshot is the
comparison base. The original untracked c8e825 test preimage is unavailable;
no assertion-by-assertion preservation claim against it is made.

## Final source identities

| File | SHA-256 after final edit |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `b45dc39bc7466160cec9bc441d2635d577ceae69f64f165f19dbca7f7605737d` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `2e5ef4ee58b1c821b6b5a9fdee83a2bce38f504dae3001f08e2b73306f350766` |
| `apps/godspeed-casework-go/internal/server/run_observation_next.go` | `f362b461744b7b474f71a6d91c01f01eba76752c4c82761af5fa06ddb3db4015` |
| `apps/godspeed-casework-go/internal/server/run_observation_next_test.go` | `1f23ab0e55e30fe95a38fc67733a5d961209dd3874f82ce4966e444ccc8c7112` |

Only the test file changed; the three scaffold source identities are unchanged.

## Finding matrix and material changes

| Rejection finding | Repair |
|---|---|
| Seeded wake not shown unconsumed while first Next authorization was blocked. | `TestRunObservationNextBlockedAuthCallbackIsOwnedByDetach` now asserts `len(lease.wake)==1` and `cap(lease.wake)==1` at that callback. |
| Transient read marker could be overwritten by an ungated fitting read. | The third read waits on a controlled gate. Before release, the test checks typed unavailable/zero aggregate, unchanged full watermarks, the same poller pointer in both directions, `retainedReadUnavailable`, and exact one-entry/one-cohort capacity. The fitting snapshot is returned only after the gate opens. |
| Nonterminal retention marker had the same overwrite race. | The nonterminal third read is gated and released only after the same zero-result, unchanged-watermark, `retainedRetentionUnavailable`, exact-reference, and counted-capacity checks. Terminal retention remains its distinct drain branch. |
| Terminal Next was not sequenced after all worker JOINs. | The hydration fixture snapshots all eight poller and retained-current pointers, waits for every actual `workerDone`, verifies the same current pointers and both attachment directions remain counted, then calls Next. |
| Formatter probe found one alignment-only difference. | Applied only the recorded `runObservationNextControlledRun` field alignment; no formatter was run. |

The new read gates use once-backed `releaseRunObservationNextBarrier`; manager
cleanup receives the same release callback so a held worker can be unblocked
before shutdown waits. Each callback also selects on worker context
cancellation. Existing assertions outside the five authorized repairs are
preserved against decoded current72. No H3 atomicity requirement was added;
root waived it because the separate blocked-worker JOIN test proves the
relevant ownership boundary.

## Diff against decoded actual72 snapshot

I generated and inspected the complete unified diff using the decoded
root-saved actual72 snapshot as the left side and the final test file as the
right side. The exact byte identities are recorded above: base source
`72c379991b6256ac38c77602e33127c9ac7a8671657a89d97406eee7e14b9620`; final
source `1f23ab0e55e30fe95a38fc67733a5d961209dd3874f82ce4966e444ccc8c7112`.
The diff contains only the alignment hunk, terminal hydration JOIN/current
identity hunk, transient recovery third-read gate and marker/ownership/capacity
checks, nonterminal retention equivalent gate and checks (including the
retained marker while gated), and blocked-auth seeded-wake assertion described
above. No other hunks were present. A reviewer can reconstruct the exact diff
byte-for-byte from the saved snapshot payload and final source identity.

## Limitations

No production/scaffold, status, debt, or other existing file was edited. No
compiler, test, formatter write, gate, or Git mutation was run. This packet is
not independent approval; the fresh critic must inspect the frozen diff and
hashes. The original c8e825 preimage is unavailable, so comparison is only
against the root-saved actual72 snapshot. No compile or behavioral RED is
claimed.
