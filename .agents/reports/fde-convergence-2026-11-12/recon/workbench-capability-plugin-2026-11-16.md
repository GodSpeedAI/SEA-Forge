# Cognate Workbench capability / plugin / skill / MCP integration recon (2026-11-16)

Read-only recon. Fetched origin for cognate-workbench and Cognate (no checkout, no branch or worktree changes). Local `main` in cognate-workbench is stale (69 behind origin/main); comparisons below use `origin/main`.

## (a) Intended design

- `cognate-workbench:.agents/plans/capability-bundles/design.md` (rev 5, "Proposed — review-ready", 2026-11-08; lives only on local `feat/desktop-terminal-browser`, not main). Makes capabilities hot-loadable in-session: bundles (payload, store, manifest), skill loader (§5), untrusted executables and MCP lifecycle under authority grants (§6), presentation slots and safe mode (§7), 13 CUJs. Overview states the current extension surface is a static code-defined plugin catalog and skill OS registers capability definitions that "nothing dispatches".
- `.../capability-bundles/HANDOFF.md` (same branch): tranche status and remaining work (listed in (d)).
- Also on that branch: `concepts_brief.md`, `reconciliation.md`, `verification.md`, `critic_report.md`.
- `cognate-workbench:docs/subsystems/capabilities.md` and `docs/known-gaps.md` (main): current model, `skill.*` families in `src/capabilities/workbench.ts`, known gaps such as `routeExecutor` (nothing calls it) and capability substitution.
- `sea-rs:.agents/reports/fde-convergence-2026-11-12/recon/03-cognate-workbench.md` line 97: capability runtime marked PARTIAL (tranches P, M, T1 committed). Line 158 notes no Workbench/SEA-Forge naming reference.
- FDE `F-implementation-plan.md` WP-23 (assignment binding), WP-42 (Workbench seam), WP-44 (context bundle): none targets plugins, skills or MCP. "Capability" in the FDE docs means the operator capability ladder and reach, not plugins.

## (b) What is on main (origin/main 1076847)

- Static plugin catalog: `src/components/pluginRegistry.ts`, `src/plugins.ts`, `src/projections/plugins.ts`, `src/capabilities/pluginInstall.ts`; tests `test/workbench-plugins.test.ts`, `J-PLG-install`, `J-PLG-toggle`.
- Skill OS: `skill.*` capabilities in `src/capabilities/workbench.ts`; `test/workbench-skillos.test.ts`, `test/skill-bundle.test.ts`.
- Authority/contract: `src/workbench/authority.ts`, `src/workbench/contract.ts`, `src/ui/webmcp.ts` (browser WebMCP, not a server).
- Dev skills only under `.agents/skills/` (`building-with-cognate`, `domainforge-sea`, `deriving-interaction-domain-models`). These are not runtime skills.
- Not found on main: `src/mcp/`, `src/capabilities/capabilityRuntime.ts`, `bundleStore.ts`, `authorityIndex.ts`. Inferred from a path grep, not an exhaustive tree check.
- `git grep` footprint on main for mcp|skill|plugin|capabilit under src/docs/test: about 90 files, mostly docs and UI.

## (c) Stranded work

Workbench (origin/main..ref):
1. **feat/desktop-terminal-browser (local only)**: 140 ahead, 20 behind main. Origin copy (32582af, 2026-11-08) is 121 ahead. **19 commits are local-only**, 2026-11-08 to 2026-11-11, never pushed: `c56ab9a`, `814c63b` (tranche P, §1.6 operator-guard remediation), `15d8646` (tranche M, SEA model change: Plugin re-scoped, CapabilityBundle/AuthorityGrant/CapabilityActivation), `3912074`, `a5ca1ef`, `3e27bf9`, `7462394` (tranche T1: authority index, lifecycle recorder, bundle store, ajv validation), `34f67e9`, `b3ef296`, `77e9774` (T2 dispatcher, `src/capabilities/capabilityRuntime.ts`), `45c50d2`, `33ba7ab`, `e45e97d` (verification and HANDOFF docs), `93d7836`, `465ec1d` (pg0 test-harness fixes), `bf4c09f` (ajv dep), plus `328e851`, `0d34ae8`, `65510bb` (design docs). Only ref containing them is this local branch. The primary checkout `~/projects/cognate-workbench` (HEAD e45e97d) has 37 uncommitted changes, all PNG/report files, none capability code. Stash list is empty. Origin-only commits from that branch: 0. The branch also carries the open PR #28 head (see e).
2. **feat/ui-production-readiness**: 49 ahead, last 2026-11-01. Contains D-23 `session.rename` capability (`6bf71d6`). PR #27 open.
3. **feat/horizon-connect** (worktree `cognate-workbench-horizon`): 75 ahead, last 2026-11-07. Includes `6bf71d6` (session.rename) and B4 desktop contract. Not plugin work.
4. Dependency pins, 1 commit each, no capability content: `chore/domainforge-0.22.0` (2026-11-14, 0.9.2 lockstep), `fde/cognate-0.9.1` (11-13), `fde/realitytrace-0.4.0` (11-13), `fix/live-test-domainforge-version` (11-05), `ci/install-pinned-domainforge` (10-22).
5. No unique work: `origin/feature/expert-teams` (0 ahead, 118 behind), `origin/test/red-session-workspace-task-contracts-20261020` (0 ahead, 80 behind). Prunable worktree `/tmp/wb-baseline` (detached 448a095). `gsr-heads/cognate-workbench` is detached at bd4c0ea.

SEA-Forge (not Workbench plugin work; "capability" here means sandbox/authority capability):
6. **origin/feat/hardening-capability-harvest**: 49 unique, last 2026-11-14, first 2026-10-26. Sandbox fixes (`crates/sea-forge-sandbox` 21 files, server 8, CI 4). Adds `.agents/plans/2026-10-26-capability-harvest.md` and skills `governed-runtime-probing` and `sea-forge-hardening-review`. Local copy in `_wt/arm64`, 0 unpushed. Already listed in `recon/branch-inventory-2026-11-16.md` line 1111.
7. **origin/feat/openshell-capability**: 16 unique, last 2026-10-25. OpenShell enforcement capability, workload identity, typed WorkloadImage, Windows lane. Touches `crates/sea-forge-server/src/mcp.rs` and `tests/mcp_workload_broker.rs`; `mcp.rs` already exists on main, so this is an MCP workload broker change, small (5/1 lines). Local copy in `_wt/arm64`, 0 unpushed. Listed at inventory line 1133.

## (d) Gaps between design and main (design rev 5 vs HANDOFF)

Done on the stranded branch (tranches P, M, T1, T2 dispatcher): committed but unmerged and partly unpushed. Gates recorded: J-CAP-prerequisite 6 tests, J-CAP-create 3 tests, authority index 5 tests. T1 "closed" via chunked verify; a 322-failure single-process run is classed as a host-concurrency flake.

Remaining per HANDOFF (none on any ref):
- T2: run-start resolver (`src/agents/runStartResolver.ts`), `src/workbench/harnessPatches.ts`, `src/capabilities/skillLoader.ts`, `src/workbench/bundleSources.ts`, J-CAP reload/source/refusal/harness suites, chunked verify.
- T3 executables: `src/mcp/client.ts` (official MCP SDK over provider streams), `src/capabilities/execSpawn.ts` (containment, record-before-spawn), `src/capabilities/mcpManage.ts`, `test/fixtures/mcp-stub/`, J-CAP-mcp and J-CAP-suspend.
- T4: `src/ui/FragmentSlot.tsx`, `CapabilityDetail.tsx`, `src/workbench/safeMode.ts`, J-CAP-safemode, restart and persist suites.
- Docs and enablement: `workbench.capability_runtime.enabled` not written; `docs/known-gaps.md` entries; CURRENT_STATUS revisions.
- Design open items unknown (not checked): owner sign-off on design rev 5; whether Hallmark and Impeccable example skills exist.
- Guess: the main-branch plugin catalog is what the design replaces; migration (design §9) is only in the stranded M tranche.

## (e) Open PRs

- cognate-workbench #28 `feat/desktop-terminal-browser` (head 32582af, origin copy, 2026-11-08; base `feat/ui-production-readiness`, not main; 381 changed files). Does NOT include the 19 local-only capability-runtime commits.
- cognate-workbench #27 `feat/ui-production-readiness` (rev 37 readiness proof).
- cognate-workbench #2 `test/red-session-workspace-task-contracts-20261020` (red journeys).
- Cognate: only release-please #86 (`chore(main): release main`). No open PRs on Cognate touch MCP, capabilities or skills.

## Cognate repo on main (a08d244)

- `packages/mcp` (`@godspeedai/cognate-mcp`): exposes capabilities as MCP tools, calls go through the runtime authorization path. Last changed 2026-11-16 (release).
- `packages/capabilities`: deterministic capability selection and `ModelProvider` contract.
- `packages/agent-discovery`: passive discovery of local agent skills, extensions, hooks and MCP config.
- `packages/hook-interoperability`.
