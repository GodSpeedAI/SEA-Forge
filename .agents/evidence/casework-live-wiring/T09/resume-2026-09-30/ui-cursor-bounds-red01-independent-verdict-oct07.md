# UI cursor bounds RED01 — independent actual-run verdict

Date: 2026-10-07. Independent review of the archived focused Bun attempt only. This is not a production approval or a claim that the full revision 4 obligation matrix passed.

## Evidence identity

I compared each `/tmp/sea-ui-cursor-bounds-red01-resume-oct06.*.raw` original with its archived `ui-cursor-bounds-red01-*.raw` counterpart. All three `cmp` calls returned 0. SHA-256 values match on both copies:

| Capture | SHA-256 |
| --- | --- |
| preflight | `508d5773da6e786adfa46927b19ce059b688184fbc2698806739386100603ec8` |
| Bun output | `83bdca48676225d5f09881789ddf669aa8b152be675af4a7e21be3f6bc327388` |
| exit | `cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22` |

The preflight records the assigned fixture SHA `134ba6162b60eddcf46b9b92347a8da4feda8cca1a26b68ae31edb1922285e8a`, frozen cursor-order test SHA `373a2b5db74fda2210f84ea2962783d115d5b8ef413f118c9b52977b7d6fdbd1`, production adapter SHA `d8b0feb02e489587672968c7450373f000ba02e3fd3d0d509888ca1e0f595f34`, and conformance SHA `edf8ed69fc9d4cf3c1ed2a136cc5f076ada72117497a92865cfdb19f3ef31c1c`. Its fresh resource sample records 4,051 MiB available memory and 9,203 MiB free swap. The archived output records Bun `1.4.0`, 14 tests, 2 pass, 12 fail, 38 assertions, 204 ms; exit capture records exit 1. This review did not rerun Bun or other gates.

## Failures that are meaningful negative evidence

All failures below reach assertions in the intended cases; the output does not show a compile, fixture initialization, or timeout failure.

1. **Constructor seed validation:** changing the trajectory's final cursor to disagree with the snapshot seed did not make construction throw (test line 135).
2. **Leading-zero exclusive floor:** the event exactly at the numeric floor was delivered (line 181), violating the expected equality suppression.
3. **Invalid-floor rejection:** the first table entry, the empty string, produced zero errors instead of one (line 214). This establishes a failure for that input only; the loop aborts there, so the other seven malformed/unsafe/wrong-epoch/over-ceiling examples were not exercised to their assertions.
4. **Ordinary future floor:** the exact boundary event was delivered (line 238), instead of being suppressed.
5. **64/65 overflow:** after the 65th event, no overflow error was reported (line 274). Assertions after that point—including overflow disposal and later healthy-subscriber continuity—were not reached in this test.
6. **Self-unsubscribe:** the self-disposing subscriber received the second queued event as well as the first (line 342). Earlier reentrant FIFO and healthy-subscriber assertions in this same test did pass before this failure.
7. **Throwing `onEvent`:** the listener's `listener failed` exception escaped the controlled timer drain at line 355. That is the intended callback-isolation failure, not a setup exception. Subsequent assertions for one error notification and continued healthy delivery were not reached.
8. **Throwing `onError`:** the 65th event did not trigger the expected error callback (`onErrorCalls` was zero at line 389). This is evidence that the overflow precondition failed; it does not establish how a throwing `onError` affects another subscriber once invoked.
9. **Sequence ceiling:** allocation beyond `1.9999999999` succeeded as `1.10000000000` (line 409); history/frontier immutability assertions following the throw expectation were not reached.
10. **Progress exhaustion:** the attempted progress update reported zero errors (line 437). No later event/history/frontier assertions in that test were reached.
11. **Settlement exhaustion:** dispatch response, start delivery, and targeted timer invocation reached their preceding assertions; the settlement path then produced zero errors (line 483). The later no-event/history/frontier assertions were not reached.
12. **Unsubscribe/cancellation:** after unsubscribing with a queued delivery, the manual timer still had pending work (line 503). The subsequent no-callback assertion and the separate post-commit timer-scheduling-failure scenario were not reached.

## Coverage limits and disposition

The two passing cases establish template-committed seed agreement and draining exactly 64 queued events in FIFO order. The run establishes actual negative behavior for the individual failures above, with the stated assertion-level limits. It does **not** establish the remaining invalid-floor inputs, post-overflow behavior, callback recovery after a thrown listener, progress/settlement no-partial-commit invariants, schedule-failure handling, or full revision 4 coverage. The 64-item drain pass does not imply that the 65th-item cap works.

I approve this capture set as a genuine focused **RED** record for the reached semantic defects above. I do not approve production behavior or claim the full feature matrix is covered. The failure at the throwing `onEvent` callback is a substantive missing-isolation result, not a fixture/setup failure. The test's early assertions materially mask the later obligations listed above; those obligations remain unproved by this run.
