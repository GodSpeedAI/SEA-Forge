# Evidence-only legacy-lineage checkpoint inventory review

Date: 2026-10-08. This is a bounded read-only inventory review for a separate
evidence-only lineage checkpoint. It grants no runtime, implementation, or C-2
approval.

## Exact proposed untracked scope

Include all untracked regular files present at review time under these three
roots, preserving every path and byte:

* `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/` — 470 files,
  4,242,182 bytes.
* `apps/godspeed-casework-go/internal/server/` — 40 files, 361,498 bytes.
* `apps/godspeed-cognitive-ui/src/adapters/local/` — 16 files, 180,427 bytes.

Mechanical inventory total: 526 files and 4,784,107 bytes. File extensions are
328 Markdown, 130 `.raw`, 52 JSON, 14 `.exit`, and 2 `.txt`. No Go, Rust,
TypeScript, generated source, or build output appears in this untracked scope.
The source-directory records and their byte-identical historical mirrors are
retained in place and belong in the separate evidence-only lineage checkpoint;
do not deduplicate, move, or delete them. The exact manifest rule is the three
roots above intersected with the untracked regular files at checkpoint time.

Content hashing found 39 duplicate-content groups containing 122 files; 34
groups include at least one source-directory artifact. Equality establishes
byte identity only. It does not prove that duplicated captures came from
distinct executions or that every historical claim is valid. Each evidence
receipt remains responsible for its own provenance.

## Exclusions and status

The 11 already staged paths are foreign operator work and stay untouched and
outside this checkpoint. In particular, `.agents/DEBT.md` has staged and
unstaged content; preserve both states. Do not include tracked source, other
untracked paths, or modify the index as part of this inventory task.

The C-2 candidate is documentation-only and remains held. The current revision
5 independent review rejects it for exact operator approval because it leaves
global-versus-case-local ordinal-gap attribution ambiguous:
`live-cursor-v4-complete-candidate-revision5-independent-review-oct08.md`.
Including candidate/review history records the decision trail; it does not
approve the candidate, public contract, or implementation.

## Safety and verification limits

A read-only five-pattern heuristic over the 526 current files found zero
matches for selected private-key, cloud-key, provider-token, and bearer-token
shapes. This is not a complete secret audit and is not a substitute for the
repository's required security gate. No scanner or build was run for this
review. The canonical recipe is `just security` (`justfile`, `security`): it
runs `just deny` and `gitleaks detect --no-banner --redact`. The pre-commit
hook delegates to `devbox run -- just pre-commit` when available, otherwise
`just pre-commit`; leave that hook enabled. No repository recipe was found for
decoding JSON-escaped evidence archives before scanning; do not claim decoded
archive coverage from this inventory.

No pre-existing files or Git state were changed; this review adds only this
report. The proposed checkpoint is evidence-only; ordinary security hooks and all later C-2 review,
operator approval, and runtime verification remain separate requirements.
