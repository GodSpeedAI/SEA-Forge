# Shared replay assertion: semantic review

**Verdict: approve the bounded test-only semantic correction.** This review covers the
proposed common conformance expectation only. No source edit, compile, test, or runtime
verification was performed.

## Basis

- `src/adapters/conformance/caseworkPortConformance.ts:93-95` requires an accepted
  mutation's `new_cursor` to advance from the pre-dispatch snapshot.
- Lines 109-126 resume from that earlier snapshot but currently take the first event
  after it as the replay result, then incorrectly require that first event to be at or
  beyond `accepted.new_cursor`. A retained, chronological intermediate frame can validly
  precede the receipt point.
- Lines 136-139 subsequently require the exact accepted cursor to be present in retained
  trajectory history.
- `internal/projection/store.go:196-219` defines replay as every retained revision whose
  cursor is strictly after the requested cursor, in store order. The store requires
  strictly advancing cursors on append. It does not promise that the first replay frame is
  the latest accepted mutation.
- `internal/intents/intents.go:172,423-428` returns the per-case post-mutation kernel cursor
  as `new_cursor`. The live fixture selects `replay-retained`, uses the default retention
  budget, and executes one case mutation (`e2e/live-conformance.ts:311-325`), so this
  accepted point should still be retained when the replay is requested.

## Required assertions in the correction

Wait until the resumed stream yields the **exact** `accepted.new_cursor`. Validate that
every earlier replay frame is strictly after the requested snapshot cursor and strictly
after the preceding frame. Fail if the stream advances beyond the accepted cursor without
emitting that exact cursor, or if it times out. Keep the existing trajectory membership
and old-snapshot immutability checks, plus the local future-only branch and all other
conformance assertions. Do not filter production events or use a synthetic stream as live
evidence.

Focused test coverage should include an ordered prefix `D` followed by the accepted
receipt frame `R` (must wait for and pass on `R`), a prefix-only stream (must fail by
timeout), and an unordered prefix (must fail). These helper tests are not substitutes for
the subsequent local and real shared-live runs.

## Source hashes reviewed

```text
85e118edc991d175e422069c93a5f1baecdd3b2cb5427bf8695fdb2155abf295  apps/godspeed-cognitive-ui/src/adapters/conformance/caseworkPortConformance.ts
044a44b4b42efb6e79f7bf0a61d003d174d17500f147d2eb80e1a61779e43e64  apps/godspeed-casework-go/internal/projection/store.go
9ce79c87c64d9a38c7586c033739ff1de45c7374d516a9e0f56fa482cdcf86fa  apps/godspeed-casework-go/internal/intents/intents.go
52d881816346d3a71ba1e457efd72f65c46c021513837ea42fd1a0c93f4db6bf  apps/godspeed-cognitive-ui/e2e/live-conformance.ts
```
