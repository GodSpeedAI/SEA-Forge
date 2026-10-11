# T07 recovery-round3 verification handoff

## Static review status

The provisional static review is in `static-review.md`. It found the session-bound retained-read implementation aligned with the bounded repair spec on source inspection. The original stale-comment observations were subsequently corrected by the artifact-provenance builder and recorded in `session-read-fix/material-diff.md`; the static review remains an immutable snapshot from before that correction. This is not approval.

## Required independent verification still outstanding

- Fresh Go vet, focused T07 race gate, and global Go race gate after the final Go source freeze.
- Build a fresh gateway binary from the reviewed source.
- Against a fresh temporary kernel cell and that binary, authenticate separate operator and R-SO HTTP sessions; preflight and commit a real sentry/signoff fixture; capture an old cursor; execute a governed mutation; compare current versus historical state; compare historical, replayed-SSE, and live-SSE actor identity and role-filtered action offers; reject actor/role query overrides and cross-case cursors; and demonstrate fail-closed behavior for an unallowlisted mapping and a kernel delegation revoked after session login.
- Run the recorded four-crate Rust gate after the Go gates, per the coordinator's instruction.
- UI gates and the local ladder remain blocked on separate UI source stabilization and coordinator allocation; this reviewer has not run them.

No round-three Go/Rust test, vet, build, compile, or runtime probe was started before the coordinator's wrap instruction. There is no running compiler or child process owned by this reviewer. The round-two rejection and round-three static note remain the available evidence; no approval is issued.
