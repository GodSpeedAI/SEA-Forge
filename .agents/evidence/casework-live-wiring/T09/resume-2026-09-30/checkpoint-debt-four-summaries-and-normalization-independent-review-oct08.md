# Checkpoint debt summaries and normalization record — independent review

Date: 2026-10-08. **APPROVE the bounded documentation corrections and
historical-byte preservation only.** No runtime/security gate or public C-2
approval is granted.

## Four DEBT summaries

The current `.agents/DEBT.md` worktree file is SHA-256
`1ffd9a86e01c2cb8a1aa22e07a6ee02b8b05e7411c485d62ff83ca0a34d81a39`.
Against the staged baseline, the diff updates only the targeted current
summaries and adds the CW-18 normalization recurrence; historical failures are
retained.

- **M-18:** accurately limits the result to this repository and records that
  push02 completed with normal hooks. The push receipt reports exit 0, the
  verified remote SHA, 1,110 passed/0 failed/4 ignored, and all CI gates green.
- **CW-02:** accurately records current scoped lint/security/push success while
  retaining earlier Clippy and scanner failures as history.
- **CW-28:** accurately says the private Prepare/shared-poller/Stop unit was
  approved and published, while Next/SSE and T09 settlement remain open.
- **CW-33:** the current status now matches the canonical Go, full-module race,
  and normal push02 outcomes; the earlier formatting failure remains explicitly
  historical.
- **CW-18:** the recurrence note reports one Git line-ending normalization and
  does not promote the preserved preflight into command-execution evidence.
  The referenced correction and wrapper support that claim.

`CURRENT_STATUS.md` and `current_status.yml` now state that revision 5 was
independently rejected for the global-versus-case-local ordinal-gap ambiguity,
that C-2 remains held, and that T09 remains partial. They preserve Next/SSE
integration as unfinished. The decision-log entry and Section 0.2 correction
remain proposal/history records, not operator approval.

## Lossless-wrapper verification

The JSON wrapper decodes to the current 857-byte raw file exactly (SHA-256
`824c110962c6b62f62e3f9f4088555c9249f3da8570c82a69be508398c9cf992`). It has
14 CRLF pairs. Replacing CRLF with LF compares byte-for-byte to the 843-byte
raw-file blob in commit `36bcd1b` (SHA-256
`7a00a48311963e23f6665d35b5a82e999d6a504e7a12ef6814594255a1e4c643`). The
wrapper labels the source as historical preflight only and explicitly denies
that it proves a scanner ran or passed. I independently verified these target
bytes; the reported 454/455 and 56-source-file comparisons remain root-attested
facts, not comparisons rerun here.

## Checkpoint boundary

The DEBT file remains `AM`: its staged blob `8eda0d3` is a foreign baseline,
and the current worktree includes task updates. Preserve that index blob and
the other foreign staged OIDs. The task-only alternate-index plan is required
to avoid committing foreign staged content. No source, tests, compilation,
scanner, or Git mutation was performed in this review.
