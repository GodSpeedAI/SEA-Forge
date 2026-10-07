# Independent source review: retained-version terminal fixture repair

Date: 2026-10-07  
Disposition: **APPROVE fixture source for a focused semantic RED attempt only**

## Scope and immutable identities

Reviewed the full original helper assignment, source receipt and hash correction; retained-publisher root decomposition; complete atomicity-repair assignment/result; prior rejection and both immutable review corrections; the new terminal-fixture assignment/result/hash receipt; the full helper and full test; and relevant trace-adapter vocabularies. No compiler, tests, scanner, formatter, typecheck, runtime, or Git command was run.

| Artifact | SHA-256 |
|---|---|
| Retained helper source | `d8df5498cc395e8ce45f52ea324ae1e31262ded874f44ca3dd5f9a56cfc1d331` |
| Current helper test | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| Terminal-fixture assignment | `3074a5a52e25dd5700d34bbcef191ae2995f38b0f6a5aed1459676145a939633` |
| Terminal-fixture result | `e8353326b2677766f4723bca80304bad0a7d7ed5dfd11e8cf72acc076045e131` |
| Assignment/result hash receipt | `4f5784a5e0917d41f542f24725f405e35a01a609ec173aa331e0b71a71921569` |

The exact preimage archive decodes to 26,472 bytes with SHA-256 `6ba0ab0ddbd80614d25c48d6485f1af70a75e8f670a72fac535c8ec128652d00`, matching the test immediately before this edit. A read-only diff from those exact bytes contains exactly three string replacements in the terminal refusal fixture: `candidateStatus` becomes `timed_out`, `over.Execution` becomes `failed`, and `over.Settlement` becomes `rejected`. The current file is 26,425 bytes with the hash above. The helper remains unchanged at its original stub hash. No assignment scope was expanded.

## Terminal fixture correction

The corrected values are valid according to the actual adapter contract: `failed` is in `AllRunExecutionStandings`, `rejected` is in `AllRunSettlementStandings`, and `timed_out` is in `AllRunTraceCommandExecutionStatuses` (`internal/contract/contract.go:426-436`). The adapter checks those run standings before constructing a snapshot (`internal/adapters/sfwp/run_trace.go:88-94`) and validates `command_finished` status through the status vocabulary (`:254-270`). `failed` is a terminal execution standing as authorized by the assignment; settlement does not determine terminality. Each candidate value differs from the prior safe state and its frame value (`active`, `unsettled`, `completed`). The candidate ID and timestamp are also unique.

The raw-string nonretention assertions at `run_observation_retained_version_test.go:350-353` now search only candidate strings that are absent from the prior image, including `rejected`, which does not occur in the canonical image field names or prior state. In particular, the corrected fixture avoids the `accepted`/`accepted_at` substring collision called out in the prior review correction. The complete prior-state equality plus marker-only image equality assertions remain intact (`:331-349`), as do candidate frame and ledger checks (`:355-363`).

## Prior matrix preservation

The complete current test still contains the repaired focused matrix: source/prior/result pointer nonaliasing with pre-dereference nil checks; exact first-ordinal preservation on reorder, shorter window, and reappearance; one-window duplicate rejection without winner selection; nonterminal refusal immutability and full-ledger recovery; terminal no-disclosure; measured exact 1 MiB and +1 byte images; 1,024/1,025 window boundary; and ordinal/generation overflow without wrap or partial mutation (`run_observation_retained_version_test.go:91-540`). These assertions match the original helper assignment. The only diff from the supplied immediate preimage is the three authorized replacements, so no previous assertion was dropped or weakened.

## Disposition and limits

Approve this frozen test source for a focused semantic RED attempt against the still-unavailable builder stub. This approval is source-only: it does not claim a test was run, that compilation succeeds, or that any RED was observed. It does not approve a helper algorithm, lifecycle integration, `Next`, production behavior, or a broader gate. Any future implementation still requires its own review and runtime evidence.

Graft saved ~29,080 tokens (~$0.02) this turn.
