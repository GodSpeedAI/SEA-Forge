# C2 clean-publication candidate privacy scan — independent review

Date: 2026-10-09.

## Verdict

The exact 544-file manifest candidate snapshot passes the credential-argument
scan. Publication remains **on hold** because the temporary clean worktree does
not yet match all ten source/status/document copies, and ancestry plus required
gate review are separate outstanding checks.

## Scope and checks

Reviewed `c2-clean-publication-copy-manifest-oct09.json`, SHA-256
`24765dc564ba0449ea778d5f20ac4eb870b9db0de9ae2b3d4926ccff3d769553`. Its
544 listed files comprise 294 tracked evidence paths, 240 untracked task
evidence paths, and ten approved source/status paths. All 544 current source
files matched the manifest byte counts and SHA-256 values.

The detector `--csrf[-_]token(?:=|\s+)\S+` found zero occurrences in the
candidate files or their decoded payloads. The scan examined 447 nested
`data`/`raw_base64` fields, detected XZ by its magic bytes and decompressed all
447, with zero decode errors. The same detector found one occurrence in each
of the 18 inventory-matched original captures (18 positive controls). No
credential values or encoded payloads were printed.

This scans current source-worktree candidate bytes, including the corrected
preflight archives. It does not scan historical Git objects. A separate
read-only scan found the detector in eight pre-correction archives in local
`HEAD`; those are historical risk and are outside the current-file payload
scan. The intended clean branch must exclude that ancestry as planned; this
receipt does not prove its ancestry.

## Copy state and remaining review

Compared the ten manifest source/status paths with
`/tmp/sea-rs-casework-publication-oct09`: seven matched; three differ:
`.agents/CURRENT_STATUS.md`, `.agents/CURRENT_STATUS.yaml`, and
`.agents/DEBT.md`. The two status files are stale and the debt copy predates
the latest repair. Resynchronize and repeat the copy comparison against the
final manifest before treating the candidate scan as proof of the clean
worktree's bytes. The CW-42 status wording also remains pending a fresh repair
of its local-commit authorization statement. Clean ancestry and required
verification gates remain outside this privacy scan.

No source code, Git state, compiler, tests, or gates were changed or run by
this review. This receipt is the only new file from the reviewer.
