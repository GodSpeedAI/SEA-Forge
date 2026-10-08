# Independent review: task-owned DEBT extraction revision 2

## Scope

Reviewed the revision 2 extraction and its receipt against the revision 1 input,
the current worktree DEBT file, and the stated alternate-index preservation
requirements. This review covers the text extraction and proposed workflow;
it does not approve source behavior or perform Git operations.

## Result: approve the corrected task-only extraction

The v1 input is 65,819 bytes with SHA-256
`148cd672887093fa37d95da9c0c18b70868e9e796a249a2efb8cb9848a188f58`. The v2
artifact at `task-owned-DEBT-revision2-oct08.md` is 65,843 bytes with SHA-256
`46a26628a7216078bc15c5d2766e1882fe57d48a4d270457a2894660e248e3db`. The
receipt's stated substitution occurs once; reversing it reproduces v1
byte-for-byte. It changes the misleading “failures below” reference to say the
earlier failures remain in the operator's separately staged migration
baseline.

The Casework section remains byte-identical to the current `.agents/DEBT.md`
section: 64,076 bytes, SHA-256
`a76ecb0a58e9e25d56f0ac3af5bd28c9a3459b4571796586581dc3e73792a9ed`. The
task-owned M-18 status, Oct 05 follow-up, Oct 08 outcome, and close are retained.
The foreign migration M-18 body and M-01–M-53 are absent; the header explicitly
states that the migration baseline remains separately staged. This is a
standalone task ledger, so its presentation differs from the full worktree
DEBT file by design.

The builder's requested `/tmp` destination was not used: v2 is in the T09 BASE
evidence directory as an immutable checkpoint artifact. The root explicitly
accepted that additive documentation path. The correction itself changes only
the temporary extraction text and does not alter `.agents/DEBT.md`, the index,
or the foreign staged blob.

The alternate-index workflow remains safe if every Git command and hook uses
the temporary index initialized from the intended HEAD, only explicit task
paths are staged there, and the real index is never replaced or reset. After
commit, only explicit committed non-DEBT paths should be refreshed in the real
index, followed by verification that all eleven foreign stage-0 OIDs are
unchanged. This review did not execute that workflow.
