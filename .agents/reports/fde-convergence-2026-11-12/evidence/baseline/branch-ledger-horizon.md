# Horizon branch ledger (WP-00c)

Integration branch: `fde/integration` (worktree `~/projects/_wt/fde-horizon-integration`), base origin/main `d9b5869`
(Cognate 0.9.1 / DomainForge 0.21.2, includes #29 single-flight init). Merges, no rebases. No branch deleted or rewritten.

## Gate numbers
| Gate | Baseline (origin/main) | Final |
|---|---|---|
| bun install --frozen-lockfile | ok | ok |
| just check-toolchain / check / boundaries / idm-integrity | ok / ok / ok / ok | ok / ok / ok / ok |
| bun test | 979 pass, 16 skip, 1 fail (1 test file ran 996 tests/95 files) | 1639 pass, 16 skip, 0 fail (1655 tests / 183 files) |

Flakes (load-sensitive, pass in isolation): baseline `setup-scripts > setup-check-empty` failed once in the full run (passes alone, 18/18);
after the gaps merge `health-open > the diagnostics read ...` timed out at 5 s once in a full run (passes alone, 5/5). The final full run was 0 fail.
Run with TMPDIR=$HOME/projects/_wt/tmp, DOMAINFORGE_BIN=mise shim.

## Per-branch verdicts
| Branch | Tip | Verdict |
|---|---|---|
| horizon-desk-gaps | e5b98b8 | MERGED (1531c18). Conflicts: package.json, bun.lock. |
| chore/drop-unused-identity-tarball | 302c09f | MERGED (merge commit; deletes unreferenced godspeed-identity-0.1.0-ed6c52a.tgz). |
| chore/gateway-unit-killmode | f57675a | landed (#26; reverse-apply check clean) |
| fix/gateway-socket-dir-at-boot | 374b58e | landed (reverse-apply clean) |
| fix/j02r2-rig-operator-ref | 2e2c290 | landed (#17) |
| fix/settlement-pending-error-class | 3f62b04 | landed (content identical in HEAD) |
| settlement-approval-gate | 8894d6d | landed (content identical in HEAD) |
| settlement-external-approval-gate | a2d98ff | landed (reverse-apply clean) |
| t1/reconcile-main | 781231e | landed (its only unique commit equals settlement-external-approval-gate content; reverse-apply clean) |
| fix/followups-19-21-24 | 40fc832 | landed as #25 da683e0 (installer PLAN_TABLE row reverse-applies clean); the approval-inbox part is superseded because the desk retired src/ui/workspace/approval-inbox.tsx on main and gaps |
| feat/ignition-onboarding | 6f57bfe | landed as #10 28ac589 (squash) |
| feat/governed-operate | 32a5344 | superseded by main #22 ed69e8c (Cognate 0.9 SeaForgeAuthority adoption; governance option already in src/runtime/create.ts and app.ts). Its 0.6.2 vendored tarballs are obsolete. |
| fde/cognate-0.9.1, fde/fix-obligation-repo-init-race | 7f19cd1, c1495df | landed (#30, #29); #29 verified present (`repoInit ??=`, `snapshotInit` in src/adapters/realitytrace-obligation.ts) |
| stage0b-containment-pr1-narrow | 7115b20 | PARTIAL port: `test/command-route-authority.test.ts` ported (452f6b3, 4 pass). `test/remote-containment.test.ts` kept out: NE-74 expectation (pair/await 503) contradicts gaps' A3-UV enrolment design. Source/catalog work superseded by gaps' rewritten containment. |
| stage0b-containment-pr1, stage0b-pr1-broad-snapshot | 43e8327 | kept out (same tip; broad 148k-line snapshot; superseded by gaps containment; remote-containment test as above) |
| stage0b-pr1-narrowing-wip | 3249d2b | kept out (unreviewed wip checkpoint, not for PR) |
| stage0b-containment-core | 23db068 | kept out (3 wip commits, "not validated"; superseded by gaps). Its 8 dirty tracked files in ~/projects/Horizon-core were not ported. |
| horizon-desk-v2, horizon-desk-v2-merge, feat/github-connector, feat/seed | 73678b2, b4e1843, 5369b49, f7b411c | already ancestors of the integration branch (landed) |
| Horizon-core untracked `test/enforcement-probe.test.ts` | n/a | COPIED (fdb9b24), passes (1/1). `test/support/settle-waiting.ts` (untracked, unused here) not copied. |

## Conflicts resolved
- package.json: main's 0.9.1 pins for all cognate packages, plus gaps' added `@godspeedai/cognate-store-sqlite` pinned at 0.9.1 (gaps had 0.9.0). Also NOTICE pin text 0.9.0 -> 0.9.1.
- bun.lock: took main's, `bun install` regenerated the store-sqlite entry (0.9.1); `--frozen-lockfile` passes.
- src/adapters/realitytrace-obligation.ts merged cleanly with main's #29.

## Unfinished work remaining (not completed here, per brief)
Source: `.agents/reports/2026-11-08-gaps-status-and-decisions.md` (stale), `2026-11-12-remediation-4-stage-0b-a3uv-enrolment.md`, `2026-11-13-stage-0b-integration-dependencies.md`.
- Production device enrolment refused: `src/remote/enrollment.ts:97,144` (`authority_unavailable`) until D1-D6 close (SEA-Forge P3 decision API `sea_forge.human_entitlement`, identity service accepting authorization evidence, controller scope enforcement, bootstrap/recovery authority, saga with reconciliation, first controller). NE-75 passes only against `test/support/p3-authority-double.ts`.
- No bootstrap of first controller: `pair/authorize` returns 409 `no_controller` (src/remote/server.ts:534,548).
- Browser-agent-as-person (critic item 4) open: a browser agent that omits the page-agent header can use a paired session's human authority (gaps-status:49-57).
- Gap A: no request ids for route/reassign/withdraw/remove/put; Gap B: no Reassign control for conflicted answers; BACKLOG/BLOCKED states have no UI setter.
- Agent containment (grant.revoke, gauntlet.run.cancel) has no runtime path (SEA-Forge unpublished); critic F5/F6/F8 items, 14 P2 rows `none_required`.
- Open Low/Medium: `/m/api/pair/await` unauthenticated vs catalog `caller_own_session`; approver sees only UA label; `pending-confirmations.ts` auto-confirm test seam.
- Unverified: power-loss durability, desk browser UI end to end, migrations 7-8 fixture-only.
- Housekeeping: `.cognate/vendor` still tracks 0.8.0 cognate tarballs (unreferenced after de-vendoring); not removed here.
