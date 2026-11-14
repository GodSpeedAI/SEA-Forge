# Recon brief (shared by all recon agents)

Mission context: GodSpeed AI is converging its existing projects into one FDE (forward-deployed engineering) /
knowledge-work operating environment. Key concepts to look for in every repo: Field Case / case state, declared
semantic model (DomainForge `.sea`), operational evidence + provenance, observed-vs-declared comparison (divergence),
authority / identity / approval, governed execution, settlement, developmental revision (versioned model change),
agent sessions/delegation/interrupt/resume, Horizon inspectability, cross-repo contracts.

## Hard rules
- READ-ONLY on every repo. No builds, no test runs, no installs, no cargo/bun/npm/pytest (the host OOMs). No git
  commands that change state (no checkout, pull, reset, stash, commit, fetch). `git log/status/diff/branch/worktree list/show` are fine.
- Never open or print secrets (.env, keys, tokens, auth files).
- The ONLY file you may write is your assigned report file. Write it with the Write tool.
- Cite evidence as `path:line` (absolute or repo-relative with the repo named). Mark every claim OBSERVED (you saw
  the code/test/doc) or INFERRED. A doc saying something is done is weaker than code+test: say which you saw.
- Prefer progressive disclosure: READMEs, AGENTS.md, CURRENT_STATUS, specs, directory listings, `git log`, symbol
  greps, then targeted source ranges. Do not read whole large files.
- Report file max ~14 KB. Dense tables over prose. No filler.

## Report template (use these exact headings)
1. Identity — path(s), remote, current branch, HEAD sha, dirty file count + what is dirty; every branch/worktree
   with ahead/behind vs origin default branch and a one-line purpose; open-PR hints if visible locally.
2. Role and authoritative ownership — which consequential state and actions this system OWNS (and explicitly does not).
3. Implemented capabilities — table: capability | evidence path:line | tests that exist (path) | status
   (IMPLEMENTED / PARTIAL / SPEC-ONLY / UNVERIFIED).
4. Interfaces and contracts — APIs/routes, CLIs, schemas, event types, identifiers/ID formats, config, with paths.
5. Cross-system integrations — which other GodSpeed systems, mechanism, direction, and any end-to-end evidence
   (e2e scripts, recorded runs, reports).
6. UX surfaces — routes/screens/components, design system/tokens, interaction grammar (if any UI).
7. Unfinished work — per branch/worktree: what is incomplete, TODO/FIXME clusters, residual-gap docs, failing or
   missing tests noted in reports, dirty working-tree changes (summarize the diff intent).
8. Contradictions, duplicated responsibilities, incompatible contracts, risks.
9. Unknowns — what you could not determine and where to look.

Final chat reply: <= 250 words: the 8 most consequential findings + the report path. Nothing else.
