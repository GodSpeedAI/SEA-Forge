# Task-owned DEBT extraction revision 2

Date: 2026-10-08. This immutable evidence copy corrects one sentence in the
task-owned snapshot only. It does not change `.agents/DEBT.md`, the Git index,
or the operator's staged migration baseline.

## Exact transformation

Input: `/tmp/sea-casework-task-owned-DEBT-oct08.md`, 65,819 bytes, SHA-256
`148cd672887093fa37d95da9c0c18b70868e9e796a249a2efb8cb9848a188f58`.
The input contained the target sentence exactly once. Revision 2 is
`task-owned-DEBT-revision2-oct08.md`, 65,843 bytes, SHA-256
`46a26628a7216078bc15c5d2766e1882fe57d48a4d270457a2894660e248e3db`.

The sole change is this literal replacement, preserving the two-space
continuation indentation:

```text
The earlier migration-worktree failures below are kept
  as historical evidence.
```

becomes:

```text
The earlier migration-worktree failures remain in the operator's
  separately staged migration baseline.
```

The output has exactly one replacement occurrence. Replacing it back in the
output reproduces the supplied input byte-for-byte. Every other source byte,
including the M-18 task update, remains unchanged; no foreign staged base was
imported.

No source, worktree DEBT, Git index, or history was edited by this task. The
revision is an evidence-only extraction correction for independent review; no
compiler, tests, gates, or Git mutation were run.
