# Branch ledger: Cognate (fde/integration @ 86e3a6c, pushed to origin)

Base: origin/main 8c87a1c (0.9.1, DomainForge 0.21.2, RealityTrace CLI 0.4.0). Worktree: ~/projects/_wt/fde-cognate-integration.

| Branch | Tip | Verdict | Reason |
|---|---|---|---|
| feat/realitytrace-real-snapshot (PR #76) | ead573a | MERGED (merge commit) | clean; sxr-cli.ts +23, test +90 |
| horizon/shared-identity | f4d4db6 | MERGED (merge commit) | clean; adds packages/identity (15 files). bun.lock needed refresh (new workspace); also syncs stale 0.6.1 workspace versions to 0.9.1 |
| fde/domainforge-0.21.2 | c3aaf2f | LANDED | main has it as #77 (squash) |
| fde/domainforge-pin-exception | bd66f91 | LANDED | main has it as #78 |
| fde/realitytrace-0.4.0 | 902a022 | LANDED | main has it as #80 |
| pr-70 | 0c38b34 | SUPERSEDED | release-please 0.8.0 branch on pre-rewrite history (2137-file diff); main has later releases (#72, #79), now 0.9.1 |

DomainForge pin check: packages/domainforge depends on @godspeedai/domainforge 0.21.2; @godspeedai/cognate-* 0.9.1. Neither merged branch touches them.

## Numbers
| | pass | skip | fail | files |
|---|---|---|---|---|
| Baseline (bun run test) | 1053 | 32 | 0 | 134 (1085 tests) |
| Final run 1 | 1091 | 32 | 1 | 139 (1124) |
| Final run 2 | 1092 | 32 | 0 | 139 (1124) |
bun run check: substrate passed (81/81) both; tsc typecheck clean after merge.
Run-1 failure: execution-ssh "resolve canonicalizes inside the root" (conformance.ts:36); passes 3/3 in isolation and in full run 2 -> load flake, not caused by merges.
Env: SXR_BIN = fde-cognate-rt040 0.4.0 binary, pg0 copied to .tmp/pg0, pg0 instance already running on 127.0.0.1:5432, TMPDIR=~/projects/_wt/tmp.

## Conflicts
None.

## Unfinished / watch
- packages/identity has no consumer in the repo; scope is @godspeed/identity (not @godspeedai/cognate-*), see packages/identity/package.json name field.
- packages/identity tests (4 files) now run in the main suite and pass.
- PR #76 snapshot path has no Workbench-side test (Workbench pins SXR v0.3.0 in src/observations.ts).
