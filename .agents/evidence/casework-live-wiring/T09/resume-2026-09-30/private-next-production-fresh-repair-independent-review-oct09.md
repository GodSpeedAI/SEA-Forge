# Private Next production fresh-repair independent review

Date: 2026-10-09  
Verdict: **REJECT source readiness pending one whitespace correction and required independent runtime gates**

## Scope and identities

I reviewed the full original private Next assignment and addenda, approved
initial-unavailable correction, root integration decisions, the preceding
source rejection, the frozen fresh-repair builder packet, and the complete
changed `manager.go` and `next.go` source. The worker is unchanged; the test
file retains the approved frozen identity.

| File | SHA-256 |
|---|---|
| `private-next-production-fresh-repair-builder-oct09.md` | `52b1b4dba391a28dd10a7aacac438a960f2167c9112d1ef492c7488ca006c28c` |
| `run_observation_manager.go` | `941851e03da13a8781eb3dd3324ba331be67254c939d0c8fd274531459ff60ee` |
| `run_observation_next.go` | `6ed10dec3ea33e660dad5ea59f92764210592f56c7a9cb9dc7034a2238a9b3f5` |
| `run_observation_poller_worker.go` | `fdd84ca4d5ef891baee20a88992f34b95bee1e0d28abe2242c309a4e399bb8f7` |
| `run_observation_next_test.go` | `36dab5455a24521a6715f5ca8f9597a2cad28a070f7c5f09df56598c28386780` |

## Review of prior findings

Direct source inspection confirms the three lifecycle findings from review
`c9bf1599` are corrected:

* Both Prepare paths store list state under `m.mu`, unlock, then seed the
  wake while the creator operation is still owned.
* Final context/token/cohort/entry/reference failures unlock and enter
  `terminalFailure`, preserving no-commit and deferred operation release.
* The captured read/retention-unavailable marker path checks `ctx.Err()` under
  the capture lock; visible cancellation unlocks and schedules terminal drain.

The implementation otherwise retains the reviewed transaction and notifier
ownership structure. The frozen source diffs are confined to the granted
manager and Next files; the worker and test identities above are unchanged.

## Remaining source blocker: exact gofmt alignment

Root’s read-only format probe `next-private-format01` at
`/tmp/next-private-format01-edjesvwe` exited 1. Its stdout SHA-256 is
`1ea80008bb775710a68688e4eac064bfc58f3048c60bf27ff1eb986b6743c99f`; stderr
was empty, and all six captures were compared. The only reported difference
is `run_observation_manager.go` line 134: `wake:` has seven spaces before
`make`, while gofmt requires eight. This is a one-space source correction;
it does not indicate a parse failure. The builder should apply the exact
alignment with a native patch and freeze a new source identity.

## Coupled C2 lifecycle metadata

I verified the accepted traceability review and root acceptance record
`c2-reader-normative-lifecycle-root-acceptance-oct09.md` (SHA-256
`cc40c0034eacd0afad82e386fb3f243a86f2a0663cf843cadd201f0300ba58c5`). The C2
supplement metadata and its entry in the parent now both say `approved`; the
supplement’s `implementation_status` remains `held`. This is the accepted
normative lifecycle metadata transition only. It grants no Next source
approval, runtime release, public readiness, or T09 settlement.

## Runtime evidence and limits

Root reports focused gate `next-private-focused02` passed all 50 checks with
exit 0 and all six actual captures compared. The canonical
`just casework-go-check` attempt `next-private-go01` exited 1 because gofmt
would rewrite `manager.go`; no vet or test ran in that attempt. These root-run
results do not establish source acceptance while the format discrepancy
remains. This critic ran no compiler, tests, formatter, or gate. After the
bounded whitespace repair, independent review and all three separately
granted runtime gates remain required before source/runtime approval.
