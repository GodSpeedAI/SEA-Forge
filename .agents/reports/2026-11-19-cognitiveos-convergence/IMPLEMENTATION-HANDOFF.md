# CognitiveOS convergence — implementation handoff

Durable checkpoint for the `/implement-spec` work on parent issue
[#108](https://github.com/GodSpeedAI/SEA-Forge/issues/108) and tickets
[#109–#122](https://github.com/GodSpeedAI/SEA-Forge/issues/109). Git commits, the
issue tracker and executable verification override every claim in this file.

## Session log

| Recorded | Session | Last safe action | Next action |
|---|---|---|---|
| 2026-11-20 | first implementation session (DeepSeek V4.1 Flash / OpenCode) | Integration branches created; #109 version alignment and standalone evidence recorded; expert/team/review/plugin and coding suites pass on the combined Workbench head | Commit the #109 version gate, then implement #110, #111 and #117 in isolated worktrees |

## Parent and specification

- Parent: [#108 — CognitiveOS convergence specification](https://github.com/GodSpeedAI/SEA-Forge/issues/108)
- Published tickets: [#109](https://github.com/GodSpeedAI/SEA-Forge/issues/109) · [#110](https://github.com/GodSpeedAI/SEA-Forge/issues/110) · [#111](https://github.com/GodSpeedAI/SEA-Forge/issues/111) · [#112](https://github.com/GodSpeedAI/SEA-Forge/issues/112) · [#113](https://github.com/GodSpeedAI/SEA-Forge/issues/113) · [#114](https://github.com/GodSpeedAI/SEA-Forge/issues/114) · [#115](https://github.com/GodSpeedAI/SEA-Forge/issues/115) · [#116](https://github.com/GodSpeedAI/SEA-Forge/issues/116) · [#117](https://github.com/GodSpeedAI/SEA-Forge/issues/117) · [#118](https://github.com/GodSpeedAI/SEA-Forge/issues/118) · [#119](https://github.com/GodSpeedAI/SEA-Forge/issues/119) · [#120](https://github.com/GodSpeedAI/SEA-Forge/issues/120) · [#121](https://github.com/GodSpeedAI/SEA-Forge/issues/121) · [#122](https://github.com/GodSpeedAI/SEA-Forge/issues/122)
- Design package: [`A`](A-actual-state.md) · [`B`](B-gap-and-convergence-ledger.md) · [`C`](C-target-architecture.md) · [`D`](D-minimal-contracts.md) · [`E`](E-acceptance-journeys.md) · [`F`](F-implementation-sequence.md) · [`G`](G-residual-uncertainty.md) · [`H`](H-transformation-brief.md) · [`I`](I-closure-register.md) · [`DECISION-LOG`](DECISION-LOG.md)

## Repositories, bases and integration branches

One integration branch per repository. No branch may be merged into a protected
`main` without normal approval.

| Repository | Path | Base | Integration branch | Head |
|---|---|---|---|---|
| SEA-Forge | `~/projects/sea-rs` | `fde/integration` `a5060197` | `integration/cognitiveos-108` | see `git log -1` |
| Cognate | `~/projects/cognate` | `main` `6c42025` (0.10.0 line) | `integration/cognitiveos-108` | `6c42025` |
| Cognate Workbench | `~/projects/cognate-workbench` | `feat/capability-runtime-t4a` `9824735` + `feat/wp23-assignment-binding` `668f686` | `integration/cognitiveos-108` | `1e3bfeb` |
| Horizon | `~/projects/Horizon` | `origin/main` `0e34a34` (0.10.0 line) | `integration/cognitiveos-108` | `18b27d1` |

Worktrees live under `~/projects/_wt/cog-os/`: `sea-int`, `cognate-int`,
`cwb-int`, `horizon-int`. Ticket worktrees are created per implementer and
removed after integration.

### Workbench integration note

The integration head is `feat/capability-runtime-t4a` (which already contains
T3 `efdcc1c` and T4a/T4b) with `feat/wp23-assignment-binding` merged in and
the domain-vocabulary commit. The WP-23 branch predates T3, so the merge
initially dropped T4a-only dependencies; `@modelcontextprotocol/sdk` and
`fast-check` were restored and `bun install` regenerated the lockfile. Every
`@godspeedai/cognate-*` pin is now `0.10.0` and every installed copy resolves
to `0.10.0`.

## Ticket states

States: NOT_STARTED · IN_PROGRESS · IMPLEMENTED_UNVERIFIED · VERIFIED_UNMERGED ·
LANDED · BLOCKED · DEFERRED_BY_APPROVED_SCOPE.

| Ticket | Increment | State | Evidence / blocker |
|---|---|---|---|
| #109 | I0 version alignment + standalone | VERIFIED_UNMERGED | Workbench typecheck; 107 expert/team/review/plugin tests; 76 coding/contract tests; Horizon single-cognate (28 packages) and `check`; all pins and installed copies at 0.10.0; `CaseContext` + `caseContextIssues` exported by `cognate-runtime-api`. Pending: commit the version gate, push. |
| #110 | I5 `experts.restore` | NOT_STARTED | Frontier. Workbench only. |
| #111 | I5 `affordance.field` | NOT_STARTED | Frontier. Workbench only. |
| #112 | I1/I2 deterministic child run | BLOCKED (consumer half) | Ready to implement the Cognate `rlm` contract + test provider. Workbench consumption needs a published Cognate version carrying it; no npm credentials on this host. |
| #113 | recovery/reconciliation | NOT_STARTED | Depends on #112. |
| #114 | I3b contained streaming | NOT_STARTED | Depends on #112 contract; uses T4a spawn. |
| #115 | egress + cumulative payment | NOT_STARTED | Depends on #114. |
| #116 | committed allow ↔ enforced profile | NOT_STARTED | Depends on #114; SEA authority side. |
| #117 | provenance projection | NOT_STARTED | Frontier. Workbench only. |
| #118 | 7B joined proof | NOT_STARTED | Depends on #112–#117. |
| #119 | Horizon exploratory handoff | NOT_STARTED | Depends on #112 (caseContext freeze). |
| #120 | Horizon evaluated WP-23 binding | NOT_STARTED | Depends on #119. |
| #121 | optional Prime adapter | NOT_STARTED | Depends on #114. Optional. |
| #122 | optional Gauntlet escalation | NOT_STARTED | Frontier, optional, no core dependency. |

## Completed verification (exact commands)

On `cwb-int` at `1e3bfeb` with `0.10.0` installed:

- `bun run typecheck` — pass
- `bun test --timeout 15000` over the expert/team/review/plugin suites listed in
  the session log — 107 pass, 0 fail, 1053 assertions
- `bun test --timeout 15000 test/J-B-basic-session.test.ts test/J-C-tool-execution.test.ts test/workbench-contract.test.ts test/workbench-glob.test.ts test/skill-bundle.test.ts` — 76 pass, 0 fail
- SXR absent from `PATH`, so the standalone runs exercised the no-optional-product posture

On `horizon-int` at `18b27d1`:

- `bun run scripts/check-single-cognate.ts` — ok, 28 packages, one copy each
- `bun run check` — ok

Failed or not run:

- Full 246-file Workbench suite, Playwright/browser journeys and Postgres-backed suites were not run in this session.
- Horizon full `verify` (pytest, ledger validators) was not run.
- No cross-process proof, containment proof or settlement proof has been run yet.

## Known blockers and limitations

1. **Cognate publication.** Workbench consumes published `@godspeedai/cognate-*`
   packages and may not import monorepo source, vendored tarballs or workspace
   aliases. The `rlm` contract (#112) therefore needs a published Cognate
   version before the Workbench consumer half can be verified end to end. There
   is no npm credential on this host (`npm whoami` → 401); the Cognate release
   workflow owns publication. Do not substitute a path alias or tarball.
2. **Host budget.** 8 GB WSL host. Serialize Cargo; use at most two jobs.
3. **pg0.** `cwb-int/.tmp/pg0` is a symlink to the main checkout's pg0. Starts
   use the default instance name; avoid concurrent DB-heavy suites.
4. **No integration branch is pushed yet.** All commits are local-only.

## Next eligible work

1. Commit the #109 version gate and push the Workbench/Horizon integration
   branches (no protected-main merge).
2. Implement #110, #111 and #117 in isolated worktrees from
   `cwb-int`, merging the integration tip before reporting done.
3. Implement the Cognate half of #112 (`rlm` contract, test provider, refusal
   semantics), then resolve the publication gate with the owner before claiming
   the Workbench slice.
4. Keep #122 optional and out of the central path.
