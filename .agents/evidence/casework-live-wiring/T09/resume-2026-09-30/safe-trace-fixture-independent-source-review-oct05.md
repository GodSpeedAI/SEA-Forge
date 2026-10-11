# Safe-trace Phase1 fixture: independent source review

Date: 2026-10-05. Verdict: **REJECT for independent fixture approval**. This is a source-only review of the temporary Phase1 test fixture/declarations. No Go test, Go compiler, Cargo, Clippy, or Bun command was run; no runtime RED is claimed. The rejection is limited to fixture coverage/proof defects below and does not authorize production trace code.

## Inputs and exact scope

Reviewed the original safe-trace test-first assignment, original proposal and v1 rejection, anchor clarification, v2 proposal/rejection, repaired v3 proposal and v3 source-review acceptance, root Unit5A acceptance, builder record, the actual three Go files, and the separate delegated-identity Clippy failure record. V3 is the binding semantic proposal; its acceptance only accepts the proposal, not this fixture or implementation.

The actual hashes match the builder handoff:

| File | SHA-256 | Review |
| --- | --- | --- |
| `apps/godspeed-casework-go/internal/ports/run_trace.go` | `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5` | Separate semantic port and safe-value types only. |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51` | Compile-time port assertion and typed-unavailable test-first stub. |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `3ffab96a1bdf8235db876d4ada4cc6978279952b0db676ba15b8594e991bcf64` | Bounded Unix-socket fixture and v3 matrix. |

The semantic port contains only run/case/item IDs, execution/settlement strings, frames and returned-array count; frame fields are event ID, kind, original timestamp and optional status/exit values. It has no JSON tags and imports no adapter/raw DTO. The adapter file deliberately contains no `Client.Do`, decoder, trace projection or fallback. This matches the Phase1 test-first boundary. No `run_trace` Rust path was added or changed. The only modified Rust path observed is `crates/sea-forge-server/tests/sfwp_delegated_identity.rs`, belonging to the separately authorized Clippy `rfind` repair; this safe-trace task introduced no Rust edit.

## Matrix review

The fixture represents all ten allowlisted event kinds, all six execution standings crossed with all four settlement standings, unknown-kind omission including malformed ignored timestamp/payload and an unknown/selected ID collision, selected duplicate IDs including a duplicate beyond the newest-1024 retention boundary, 0/1024/1027 returned-frame counts and newest-order retention, source order with intentionally non-monotonic timestamps, returned-row-count wording, required trace-record presence/cardinality and non-atomic zero/positive byte combinations, exact `u64` maximum/overflow, the command-finished optional status and exit-code values/safe-number boundaries, ignored non-command payload metadata, semantic privacy-marker absence, blank expected identity before dialing, exact run_get request shape, and authority refusal-class preservation. Existing artifact code/tests are outside the three-file change.

The fixture’s exact wire-request checker requires one received request line, `verb == "run_get"`, the requested run ID, and exactly the two expected keys. Blank expected identities check for zero dial and request counters. For valid-return cases, `assertRunTraceCase` checks the result/error before calling the request checker; therefore the temporary unavailable stub reaches a behavior assertion without waiting for its nonexistent request. For error cases the checker is also nonblocking (`default`).

Cleanup owns one listener, accepted connection and worker. It closes the client, listener and accepted connection and joins the worker with a five-second bound; the peer read/write deadlines are four seconds. This is bounded owned cleanup by source inspection. Because no test command was run, no assertion RED, cleanup runtime, or fixture execution result is independently proven here.

## Blocking defects

1. **The missing response-identity cases do not construct valid responses.** In `run_trace_test.go:240-248`, the helper deletes one identity key from a map and joins all three map lookups with commas. For missing `run_id`, the joined string begins with a comma; for missing `case_id` it contains a doubled comma; for missing `plan_item_id` it ends with a comma. These become malformed JSON when inserted into the response object. The cases therefore prove only that malformed JSON is unavailable, not that a valid response with each identity field absent is rejected. Exact missing response ownership is a v3 requirement.

2. **Error cases do not assert an empty safe result.** `assertRunTraceCase` checks only the error kind in its error branch (`:193-197`) and ignores its `want` snapshot argument there. Thus the many cases passing `ports.RunTraceSnapshot{}` do not prove the v3 “unavailable with no safe result” rule for mismatched/missing ownership or malformed projection. They would still pass this helper if the implementation returned a populated snapshot together with the expected error.

These are approval blockers: the first leaves a required identity invariant untested; the second omits an explicit security/data-boundary assertion from the common negative-case helper.

## Additional v3 coverage gaps to close before approval

* Selected rows cover missing/blank event ID and blank/malformed timestamp strings (`:312-320`), but not null/wrong-typed event IDs, missing/null/wrong-typed timestamps, or a null/scalar array element. V3 requires selected-row identity/timestamp validation and malformed trace-shape rejection.
* Presence cases cover missing bytes, false `present`, wrong-typed `record`, and numeric/string/bool/fractional/negative bytes (`:371-388`), but do not cover missing/null/wrong-typed `present` or `bytes: null`. Those are distinct malformed presence-metadata shapes under the v3 required `present: true` and exact-integer rules.
* The fixture covers malformed response projections but no raw invalid-JSON response case. The proposal requires malformed responses to yield unavailable; this behavior is not directly represented in the matrix.

The core requested rules otherwise align with v3: unknown rows are removed before identity/duplicate checks; selected duplicates are checked through the full returned array before retention; counts are returned-row safe counts; bytes are not cross-checked against row count; source completeness is explicitly disclaimed; safe metadata bounds and privacy fields are represented. V3’s key material changes from original/v2 are followed: exact reported trace-presence metadata is required without claiming parse completeness, unknown kinds cannot poison selected rows, returned-prefix totals are not source totals, and missing/null/scalar whole payloads are treated as absent command metadata.

## Rejection evidence and next owner

The supplied prior Clippy failure is a source-review input only: `clippy::filter_next` correctly rejected the separate delegated-identity `.filter(...).next_back()` call and recommended `.rfind(...)`. The current Rust worktree path is that separate authorized test file; it does not change this Go review. The previous temporary Rust path conflict is not a safe-trace scope requirement and is not found among the three Go files.

No code, test, status, debt, or Git edit was made for this review; only this new review artifact was added. No compiler token was used. Graft retrieval was used before source inspection and reported approximately 48,204 tokens saved (~$0.04) this turn.
