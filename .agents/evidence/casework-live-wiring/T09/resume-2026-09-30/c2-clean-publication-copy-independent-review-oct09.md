# C2 clean-publication evidence-copy review

Date: 2026-10-09.

## Verdict

**Approve the bounded 534-file evidence copy.** This verdict covers byte
equivalence of the listed evidence files and the clean worktree base. It does
not approve publication or the ten retained source/status paths.

## Scope and evidence

Reviewed the exact copy scope in
`c2-clean-publication-copy-manifest-oct09.json`, SHA-256
`24765dc564ba0449ea778d5f20ac4eb870b9db0de9ae2b3d4926ccff3d769553`. The
manifest has 544 paths: 534 evidence files and ten source/status files. The
builder's assignment was to copy only the 534 evidence paths to
`/tmp/sea-rs-casework-publication-oct09` using native patch batches; the ten
source/status paths, manifest itself, and new review receipts were excluded.
No Git mutation, compiler, or gate was authorized for this copy step.

Independent byte-level comparison checked each evidence source and destination
against the manifest length and SHA-256. All 534 source files and all 534
destination files matched; source, destination, and manifest byte totals are
1,231,421. There were zero missing files, hash/length mismatches, or
source-to-destination byte mismatches. The temporary worktree's read-only Git
status contains exactly the 544 manifest paths, with no unexpected or absent
paths. The manifest itself and all three reviewer receipts were absent from
the destination.

The ten excluded source/status paths remain separate. Seven still match their
manifest contents; `.agents/CURRENT_STATUS.md`,
`.agents/CURRENT_STATUS.yaml`, and `.agents/DEBT.md` are stale against the
current source state, as expected after source status advanced. This copy
review did not synchronize them.

## Privacy archive checks

All 18 corrected preflight archives in the evidence copy are byte-identical to
their source files. Each decoded XZ/Base64 payload matches its correction
manifest hash and length, carries the sanitized-projection provenance, and
has no match for the credential-argument detector. The original captures
remain independently matched to the inventory, providing 18 positive-control
matches. Across the 18 corrected projections, 2,427 PID/process-name rows
retain the original PID/name pairs; each output row has only those two fields.
Pre-list bytes and post-list facts through the standalone HEAD anchor and
through end remain byte-identical. No command arguments were printed.

## Clean base and process deviations

Read-only Git inspection confirmed the temporary worktree `HEAD` is exactly
`9efd039e47ee095c1e6ac4a7b9d0f99fcb89beaa`; that base is an ancestor of itself.
The unsafe commit `608309fab24973b9b231a666c1ce6bc9c7669d6b` exists in the local
object database but is not an ancestor of this worktree; the read-only
ancestry check returned 1. No Git state was changed.

The builder reported two transient progress issues: its parser initially
expected JSON but received verifier text and resumed after two successful
batches; an intermediate byte total was 95 bytes high because its subtotal
started incorrectly. The builder corrected the progress total with a final
whole-set comparison. My independent 534-file comparison confirmed no
remaining copy mismatch. No patch collision, rejection, or partial copy
remains in the inspected destination.

## Remaining limits

The 534 evidence-copy proof is complete. Final status/debt resynchronization,
new captures and receipts, final manifest refresh, and publication review are
separate pending work. This receipt does not claim clean publication or
completion of other required gates.

This review ran no compiler, tests, gates, or Git mutation. This receipt is
the only new reviewer file and was not copied to the temporary worktree.
