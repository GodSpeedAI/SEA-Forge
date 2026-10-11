# Private Next production final independent review

Date: 2026-10-09  
Verdict: **APPROVE the bounded private Next source and independent runtime evidence**

## Scope and identities

I reviewed the full original private Next assignment and all binding addenda,
the approved initial-unavailable correction, integration root decisions, the
original production review, the fresh lifecycle repair and the exact
one-space format patch. The final source identities are:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `6a9fa1faede158fab40102089ca7a1997d87009f4c3083e77c39894dec30cf46` |
| `run_observation_next.go` | `6ed10dec3ea33e660dad5ea59f92764210592f56c7a9cb9dc7034a2238a9b3f5` |
| `run_observation_poller_worker.go` | `fdd84ca4d5ef891baee20a88992f34b95bee1e0d28abe2242c309a4e399bb8f7` |
| `run_observation_next_test.go` | `36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780` |

The source repair is recorded in
`private-next-production-fresh-repair-builder-oct09.md` (SHA-256
`52b1b4dba391a28dd10a7aacac438a960f2167c9112d1ef492c7488ca006c28c`); the
exact alignment-only repair is recorded in
`private-next-production-format-fresh-builder-oct09.md` (SHA-256
`08da66f3dff51b7cac479935f061c170eb90343ba79e83745d7077b9898aad31`). The
worker and approved test fixture remain byte-identical to their reviewed
identities. The original untracked `c8e825...` test preimage was not retained;
this review does not claim byte-exact test preservation against that missing
preimage. The accepted fixture review establishes its bounded diff against the
verified repair-2 snapshot.

## Source review

The three findings in my preceding production rejection are resolved:

* Both successful and unavailable-list Prepare paths update `listState` under
  `m.mu`, unlock, then seed `lease.wake` while the registered creator still
  owns Prepare. This keeps channel sends outside the mutex without adding
  fake notifier ownership.
* Final commit context, token, cohort, manager-entry, and reverse-reference
  failures unlock and enter the idempotent terminal drain path. No candidate
  watermark or aggregate is committed; deferred operation release remains
  ahead of drain completion.
* Before returning a recoverable read/retention marker, capture checks caller
  cancellation under the lock. Visible cancellation unlocks and schedules
  terminal drain; marker recovery remains available only when the context is
  still live.

The exact one-space alignment correction matches the read-only format probe's
sole difference. No other production source changed. On the full assignment
check, the transaction admits one exact lease token and operation before
callbacks; validates authorization/context before wait, after wake, and before
disclosure; captures exact attached entries, current pointers, copied ledgers,
and watermarks; commits all candidates only after final exact-ownership checks
while permitting current-pointer advancement; and returns zero/no commit on
any run error. Recoverable nonterminal markers remain distinct from projector
and terminal failures. The operation releases before drain waits. The worker
uses the same production notifier registration and nonblocking send/release
helpers, with target references acquired under the lock and sends/Done outside
it. Successful terminal current remains available after worker completion;
initial nil-current read failure retains its existing detach/JOIN contract.
Run/gap ordering and source-frame order follow the accepted fixtures. I found
no further material deviation in the frozen source.

## Independent gates

I independently ran each separately granted final-source gate with a
host-visible preflight. Every preflight showed `MemAvailable` above 1200 MiB,
`SwapFree` above 512 MiB, the host process list, no blocked compiler process,
HEAD `9efd039e47ee095c1e6ac4a7b9d0f99fcb89beaa`, and the four frozen source
hashes above. Root archived and losslessly compared all six captures for each
gate.

| Gate | Result | Captures |
|---|---|---|
| Focused manager/Next race, JSON output | Exit 0; 62 test-pass records, 0 failures, 1 package. The 62 includes nested test records; root's verbose run reported 50 top-level pass lines. | `/tmp/next-private-critic-focused02/`; root archive `next-private-focused02` |
| `just casework-go-check` | Exit 0; format, vet, and tests green; stdout included all packages and the recipe summary; stderr empty. | `/tmp/next-private-critic-go02/`; root archive `next-private-go02` |
| Full-module Go race | Exit 0; 752 test-pass records, 0 failures, 10 tested packages and 5 packages with no tests; stderr empty. | `/tmp/next-private-critic-race01/`; root archive `next-private-race01` |

The independent focused gate output was 75,032 bytes; the canonical check
output was 1,445 bytes; the full race output was 866,580 bytes. The six raw
files per attempt are command, preflight, preflight exit, stdout, stderr, and
exit. Root confirmed each matching archive against the actual captures.

Two earlier attempts are explicitly excluded as adequate independent gate
evidence. `next-private-critic-focused01` exited 0, but its preflight saw only
sandbox PIDs 1, 2, and 11; it lacked host process visibility. The first
`just casework-go-check`, `next-private-critic-go01`, did not execute its
recipe: Just could not create `/run/user/1000/just/just-WOX896` on a read-only
filesystem, and exited 1 with no stdout. Root compared all six captures for
both attempts. The later host-escalated retries above used normal cache access
and passed the required resource/process guard. No foreign process was killed.

Root's accepted expected-RED record `next-private-red01` preceded production
implementation: it contained the two explicit intended stub failures and a
passing initial-unavailable cleanup fixture, with no compiler failure. It is
not production-green evidence and is not counted as one of the independent
gates.

## Debt and normative lifecycle records

I reviewed `private-next-debt-progress-builder-oct09.md` (SHA-256
`21f23d2f0a40ea78da82bcdeb0f3596319a6a11c5c8d10d4658b4d82c423710b`) against
its CW-41/CW-39-only grant and exact `.agents/DEBT.md` diff (SHA-256
`503a1b625bb0fd629c9b3453e808864fc02834a42ed37b432ec9848f884de552`). It
correctly distinguishes the historically invalid original fixture from the
accepted repair-3 fixture, records the production rejection and repair,
expected RED, root-gate outcomes, and the initial deficient critic attempts.
The “critic pending” wording is a dated progress snapshot from before the
host-guarded retries and this final verdict; the root is to append the final
closure state without rewriting that immutable history. CW-39 records the
sandbox-process-visibility and read-only-Just-directory limitations and the
required escalated retry conditions. `.agents/OBSERVED_DEBT.md` is unchanged
(SHA-256 `be5c4554827d41bc5449f9c37d28434640aac5d459ec88dcc78bec4960f6c12a`).
No false claim that the current repair-3 fixture remains invalid or that the
expected RED is production success appears in the update.

The accepted C2 normative traceability review remains separate. The
supplement and parent status are both `approved`, while the supplement's
`implementation_status` remains `held`; root's coupled acceptance record
`c2-reader-normative-lifecycle-root-acceptance-oct09.md` (SHA-256
`cc40c0034eacd0afad82e386fb3f243a86f2a0663cf843cadd201f0300ba58c5`) grants
no Next source approval, runtime release, public readiness, or T09 settlement.

## Decision boundary

This approval covers the private Next source and its assigned focused,
canonical Go, and full-module race evidence. It does not establish T09
settlement, a public interface, public readiness, or any broader lifecycle
approval. Root's final unit acceptance and subsequent workbench/status
handoff remain outside this critic's approval. No source, test, debt,
specification, or Git file was changed during this final review.
