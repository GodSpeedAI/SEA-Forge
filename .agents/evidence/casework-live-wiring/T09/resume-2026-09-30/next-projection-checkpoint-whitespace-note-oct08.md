# Checkpoint pre-commit whitespace diagnostic

The first checkpoint command exited2 at the optional `git diff --cached
--check` diagnostic, before invoking `git commit` or its hooks. Its exact six
original captures are archived as `next-projection-checkpoint01-*-oct08.raw.json`
and independently decoded/compared by root.

The findings are two-space Markdown line endings in immutable evidence
headers. They are valid Markdown hard breaks; changing them would alter the
reviewed evidence identities. Preserve those originals. A subsequent check
covers the staged Go, YAML, and scanner configuration paths, followed by a
normal commit with all existing hooks enabled. No whitespace configuration,
hook, required repository gate or evidence byte is changed.
