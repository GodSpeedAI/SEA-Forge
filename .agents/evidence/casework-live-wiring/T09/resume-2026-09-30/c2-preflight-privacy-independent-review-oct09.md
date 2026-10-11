# C2 preflight privacy correction — independent review

Date: 2026-10-09.

## Verdict

**Approve the bounded 18-archive privacy correction.** This verdict covers the
listed correction unit only. It does not approve publication or establish that
the broader clean-branch candidate set is clear; that audit remains pending.

## Evidence checked

- Immutable inventory: `c2-preflight-privacy-inventory-oct09.json`.
- Correction manifest: `c2-preflight-privacy-corrections-oct09.json`,
  SHA-256 `015b39b44768e615890afa36fc0698929a30ea7ab6382aaececa3246fe044a6c`.
- Correction note: `c2-preflight-privacy-correction-oct09.md`, SHA-256
  `39a5859f866272d89c92e246d1de4ee4d2cd2977703f1299cd08662ab2c12cba`.

Independent checks verified all 18 original temporary captures against the
inventory's byte counts and SHA-256 values. The credential-argument detector
`--csrf[-_]token(?:=|\s+)\S+` found one positive control in each original
capture (18 total) and zero occurrences in all 18 corrected decoded
projections. All corrected outer archive hashes, decoded lengths and hashes,
projection labels, and non-lossless provenance metadata matched the manifest.
The corrected payloads contain 2,427 strict PID plus process-name rows; their
PID/name pairs match the original rows and the manifest. No command-argument
tokens remain in those rows. For all 18 archives, bytes before the process-list
header, post-list facts through the standalone HEAD anchor, and bytes from that
anchor through end were byte-identical to the original capture.

## Publication scope and limitations

A separate scan of Git `HEAD` found the same credential-argument pattern in
eight historical tracked preflight archives. The inventory identifies those
eight as local-HEAD archives and none as present in the last published commit.
They are historical risk evidence; this finding is not a scan of the intended
clean-branch candidate bytes, which are based on current-worktree copies.

The clean-branch copy review is incomplete. The prepared worktree has ten
changed paths; seven of eight non-status paths currently match the source
byte-for-byte, while `.agents/DEBT.md` has since changed and both copied status
files are stale. Do not treat those comparisons as proof of the final copy
manifest. Publication remains on hold pending the full current-candidate
payload scan, final copy equivalence after status/debt resynchronization, and
the remaining branch/gate review.

The current CW-42 debt wording still makes an exact local-commit approval a
condition. Root has determined that the operator already authorized meaningful
local commits and clean-branch preparation; only publication needs exact
commit or push approval. Repair that wording before relying on the debt entry.

No source code, Git state, compiler, tests, gates, status, or debt were changed
by this review. This receipt is the only new file from the reviewer.
