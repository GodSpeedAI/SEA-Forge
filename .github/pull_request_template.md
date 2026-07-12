<!--
Pull request template. Keep it short. The PR title is the canonical
squash-commit title and MUST follow Conventional Commits so Release Please can
derive the next release:

  feat(scope): ...    fix(scope): ...    perf(scope): ...
  refactor(scope): ...   docs(scope): ...   test(scope): ...
  build(scope): ...   ci(scope): ...   chore(scope): ...

Use `!` for a breaking change: `feat(cli)!: drop legacy flag`.
Add the `!` either after the type or after the scope.
-->

## Why

<!-- What problem exists today and why it matters. One short paragraph. -->

## What changed

<!-- Bullet list. Call out anything risky. -->

## Validation

<!-- Exactly what you ran. Prefer copy-paste of commands + pass counts. -->

- [ ] `just check-fast` passed locally
- [ ] `just ci` passed locally (or CI is green on this PR)
- [ ] `just context-check` passed (CURRENT_STATUS.md updated if project files changed)
- [ ] Tests cover the new behavior, or N/A explained

## Risk and rollback

<!-- Worst case if this is wrong. The one-line revert command. -->

## Release note

<!-- One user-visible line for the changelog, or "internal: no user-visible change". -->

## Breaking change

- [ ] No
- [ ] Yes — PR title uses `!` and this section explains the migration.
