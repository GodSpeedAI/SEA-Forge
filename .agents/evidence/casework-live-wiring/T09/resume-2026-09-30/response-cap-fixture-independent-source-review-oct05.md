# Response-cap retry fixture repair: independent source review

Date: 2026-10-05. **Verdict: READY** for root's independent gates. This is source-only review; no compiler or tests were run and no Go/source/status/debt/Git files were changed.

## Inputs and identity

Reviewed the original instructions in `response-cap-fixture-repair-root-assignment-oct05.md`, the builder record in `response-cap-fixture-builder-oct05.md`, and the actual worktree file `apps/godspeed-casework-go/internal/adapters/sfwp/response_limit_test.go`. The builder's reported before SHA-256 is `7ec880bc2a7cdf15eb8c4dfa26daca7d4ea021b7c50940a20e16839cec7b6299`; the actual current file hash was independently recomputed as `dfb98f494880faf946ceaf3d0d10a139a20ed3260647112dd08dfc6372ccaa88`, matching the builder's resulting-file identity exactly. `dfb98f494...` is a content hash, not a Git object: `git cat-file -t dfb98f494` reports “Not a valid object name.” The source currently appears as a worktree diff against `HEAD`; I reviewed that exact diff rather than claiming a commit.

## Scope and behavior review

`git diff --numstat HEAD` reports exactly 10 insertions and 4 deletions in the single target file. The diff is confined to `TestInspectOverLimitResponseRetriesOnceOnFreshConnection` (`response_limit_test.go:249-285`) and `TestOverLimitMutationAndRecoveryResponsesNeverResendMutation` (`:287-338`). Each adds `const responseLineLimit = 1024`, derives its malformed oversized body as 1,025 `x` bytes, sets `config.MaxResponseLineBytes` before calling `New`, and uses the existing fake server that appends LF. The actual invalid-JSON line is therefore 1,026 bytes against the 1,024-byte line cap; overflow is encountered before `DecodeResponse` can classify it as malformed JSON.

The inspect test preserves the successful safe retry and exact request/connection assertions: two `case_list` requests and two connections (`:276-284`). The mutation recovery test preserves the actual successful `case.commit` outcome, the expected recovered case ID, and exactly one commit request with two status reads (`:319-337`). The recovery JSON response is 153 bytes plus LF (154 total), fitting the 1,024-byte cap. No timeout, recovery budget, request count, connection count, mutation resend, or recovered-outcome assertion was removed or weakened.

The complete default-boundary fixture remains unchanged at `:199-227`: it still accepts exactly `approvedResponseLineLimit` (32 MiB including LF), rejects one byte over as typed unavailable, and verifies the connection is poisoned and cannot be reused. The approved constant remains `32 << 20` at `:20-21`. The shared `testConfig` is unchanged (`client_test.go:113-123`), including its existing request/connect/recovery/backoff/subscription timeouts. `git diff --check` produced no output. No formatting or runtime/test behavior beyond the two requested fixture caps differs from the assignment.

## Source anchors and deviation record

Assignment requires only those two fixture-local caps, cap-relative invalid responses, pre-`New` config assignment, retained assertions, and valid recovery under cap. The actual diff satisfies each. The builder record says no material deviations; independent review confirms that. The only identity clarification is that the supplied `dfb98f494...` is the SHA-256 of the resulting file, not a commit hash/object.

Graft was used to orient before exact source/diff reads (`graft ask` on the fixture repair and constraints). Graft reported approximately 66,756 tokens saved this turn, worth about $0.05 at its displayed rates. No gates are certified here; they remain for the root's sole compiler-token owner.
