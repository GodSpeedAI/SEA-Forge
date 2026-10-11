# Workbench slice prompt

Paste the block below verbatim to advance the SEA Forge Governed Workbench by
one vertical slice. It is idempotent: it derives current state from the
repository every time rather than from a remembered plan, so running it against
an unchanged tree picks the same next slice, and running it after a completed
slice picks the following one. It never re-does finished work and never needs
editing between runs.

---

```text
/building-sea-forge-workbench

Advance the workbench by exactly ONE vertical slice, then stop and report.

FIRST, re-derive current state from the repository — do not trust any plan,
summary, or memory of what was done:
  - `.agents/CURRENT_STATUS.md` and `.agents/OBSERVED_DEBT.md` (what shipped, what's open)
  - `sfwp::IMPLEMENTED_METHODS` in crates/sea-forge-server/src/sfwp/mod.rs (what the kernel actually speaks)
  - workbench/apps/desktop/src/router.tsx (what surfaces exist, which are still UnbackedSurface)
  - .agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md (the story map)

THEN pick the next slice by blast radius, not epic order: the story whose
implementation unblocks the most downstream stories, resolves the most
currently-dead references, or removes a structural blocker that no amount of
work elsewhere could close. Say which story IDs it settles and which it
unblocks, and why you picked it over the runner-up. If the highest-leverage
slice is already implemented, move to the next one — never redo shipped work.

CONSTRAINTS:
  - Leverage what exists. Prefer a tweak or small refactor over a rewrite when
    it achieves the outcome sufficiently.
  - Restructure only where it is load-bearing: real duplication you are about
    to add a third copy of, or a rule that would drift if left copied. Use the
    neatcode skill's judgment there. No speculative abstractions.
  - Follow the repo's existing seams: additive SFWP verbs per ADR-003, Rust
    types -> schemars -> generated TS + AJV, closed Tauri bridge enums, hooks
    over `queryGoverned`, standing maps from `pages/standing.ts`.
  - Never weaken an epic invariant to simplify the UI. In particular: authority
    before side effects, execution never equal to settlement, denial as a
    governed outcome, absence reported rather than inferred, and projections
    that never become truth.
  - Every non-trivial behavior leaves a test that FAILS if the behavior breaks.

FINISH by running and reporting honestly:
  cargo fmt --all -- --check
  devbox run -- just check
  cargo test --workspace --all-features --locked
  cd workbench && bun run check && bun run --cwd apps/desktop test
Report skipped or flaky tests as skipped or flaky — never as passed. Then append
the slice to `.agents/CURRENT_STATUS.md` and log out-of-scope findings in
`.agents/OBSERVED_DEBT.md`, and name the next spendable slice.

- **Use the neatcode skill**: append `Use the neatcode skill to refactor and restructure where it is load-bearing, but not for speculative abstractions.`

Do not commit unless I ask.
```

---

## Tuning it

- **Aim it at a specific journey**: append `Constrain the slice to journey N.`
- **Ask for more per run**: replace "exactly ONE vertical slice" with
  `two vertical slices, gating after each`. Keep the gate between them — a
  second slice built on an ungated first is where drift enters.
- **Get a decision before code**: append `Report the chosen slice and its
  classification table first, and wait for my go-ahead before writing code.`

