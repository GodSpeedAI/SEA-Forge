# Branch ledger: Cognate Workbench (fde/integration @ 8e59664, pushed to origin)

Base: origin/main d1a7885 (Cognate 0.9.1). Worktree: ~/projects/_wt/fde-workbench-integration. @godspeedai/cognate-* stay 0.9.1 (package.json untouched).

## feat/horizon-connect (c4096b9, local only, never pushed)
`git cherry` vs origin desktop: 7 unique commits.
| Commit | Verdict | Reason |
|---|---|---|
| fb7104f Connect Horizon + session deep link | PICKED (0003484) | one conflict: src/ui/App.tsx import block (branch assumes desktop-branch shell imports); kept only `resolveSessionLink`. typecheck + 3 journey tests green |
| a25f895 agents propose case steps (case_add_item) | PICKED then REVERTED (7d19939, 8e59664) | test/boundary-sfwp.test.ts forbids a raw SFWP socket client in src/; main replaced seaForgeCall with typed SeaForgeAuthority (no case_add_item verb). Port with a local socket helper broke that guard, so it is held out. Needs a typed verb in Cognate |
| a9bf1f7 observe model requests/user turns with sha256 refs | PICKED (c13edde) | clean |
| 9695cad experts review steps start without author memory | PICKED (5b83573) | clean |
| c4096b9 runner review step leaves out author memory | PICKED (20c772a) | clean |
| 35ee1cf docs(status) rev 40 | KEPT OUT | commits dist-package/ binaries (8 blobs >50 MB); rest is desktop-branch docs (src/terminal, browser, preview READMEs) |
| 78c4bb2 B2 packaged smoke harness | KEPT OUT | commits dist-package/ binaries (267 MB zip, 122 MB setup exe, ~100 MB exes/binaries); scripts/qualify-tauri.sh already on origin desktop branch (later commits e25e363, e6de37a) |

### Why the push of feat/horizon-connect was rejected
35ee1cf and 78c4bb2 add `dist-package/` build artifacts: cognate-workbench-0.1.0-windows-x64.zip (267,827,200 B), -setup.exe (122,013,098 B), windows .exe x2 (102,461,952 / 102,461,440 B), linux binary (97,862,856 B), a 72 MB tar.gz and .deb. Several exceed GitHub's 100 MB per-file limit; `git rev-list --objects origin/main..feat/horizon-connect` lists them. No secret-like content was looked for beyond this; the size cause is sufficient. origin/feat/desktop-terminal-browser and origin/main contain no dist-package/ files. None of this is in fde/integration.

## Other local branches not merged into origin/main
| Branch | Tip | Verdict | Reason |
|---|---|---|---|
| ci/install-pinned-domainforge | 1d4cc16 | LANDED | cherry 0 unique; main has it (#6) |
| fde/cognate-0.9.1 | bda1517 | LANDED | main d1a7885 (#32) |
| fix/live-test-domainforge-version | 3730604 | LANDED | main 74601f3 (#31) |
| feat/ui-production-readiness | 448a095 | KEPT OUT | open PR #27, 49 unique commits, 49 behind; contained in feat/desktop-terminal-browser |
| feat/desktop-terminal-browser | e45e97d | KEPT OUT | open PR #28, 140 unique, 140 behind main; pins Cognate 0.8.0; local tip is 19 ahead of origin (unpushed); needs its own merge decision. 37 dirty PNGs in its checkout untouched |
| feat/horizon-connect | c4096b9 | PARTIAL | see above |

## Numbers (bun run typecheck; bun run test, --timeout 15000, SXR_BIN 0.3.0 from main checkout, pg0 on 5432)
| | pass | skip | fail | tests |
|---|---|---|---|---|
| Baseline main | 1402 | 12 | 3 | 1417 / 160 files, 1031 s |
| Final run A (with case_add_item port) | n/a | | 8 | killed by 1500 s timeout; found boundary-sfwp failure |
| Final (8e59664) | 1405 | 12 | 10 | 1427 / 162 files, 1678 s (host under load) |
typecheck clean at baseline and final.
Baseline failures (carry through): E2E J-WS-directory-discovery (RED), RED journeys directory suggestions, E2E remote workspaces over SSH.
Extra 7 final failures: J-CFG-config-path x4, J-EXPERT-workspace-admission, E2E Experts and Teams x2 (timeouts at 15000 ms / 60 s+ during a 1678 s slow run). J-CFG-config-path, J-EXPERT-workspace-admission, E2E-experts and boundary-sfwp rerun together: 13 pass, 0 fail. Not attributable to merged code; E2E Experts and Teams "default Team" test not individually rerun beyond E2E-experts.test.ts (in the 13). 
New tests passing: J-AUTH-CONNECT-HORIZON, J-LINK-session-deeplink, J-RT-realitytrace additions, J-EXPERT-memory.

## Unfinished
- case.proposeStep (a25f895) needs a typed case_add_item verb in SeaForgeAuthority (Cognate) before it can land.
- Horizon deep link in src/ui/App.tsx uses `#sessions/<id>`; recon says Workbench uses `#session/<id>` on the desktop branch (src/ui/sessionLink.ts) - confirm which hash form Horizon emits.
- Workbench still pins SXR 0.3.0 (justfile setup-realitytrace); Cognate main is on 0.4.0.
- Commit 0003484 (cherry-pick of fb7104f) lacks the Co-Authored-By trailer.
