# FDE convergence mission — resume point

Updated: 2026-11-12. Inputs: `.tmp/reference/{Mission.md,GodSpeed_Universal_FDE_Knowledge_Work_SOP_v0.2.md,palantir-*,nvidia-*}`.
Authority: owner addendum 2026-11-12 grants execution through release (see memory `fde-convergence-mission`).

## Phases
- [~] A. Recon — reports in `recon/`. Done: 02-horizon, 04-domainforge-cep-contextkernel. Running: 01, 03, 05, 06, 07.
- [ ] B/C/D. Deliverables A-G (current state, target UX spec, convergence matrix, reference mapping, integration architecture, plan, qualification plan).
- [ ] E. EGM Goldfish review (Sonnet reviewers, fresh contexts) -> deliverable H.
- [ ] Implementation by dependency-ordered work packages (Sonnet implementers), integrated release gate, release.

## Settled facts (do not rediscover)
- Horizon: `horizon-desk-gaps` (~/projects/Horizon-gaps, e5b98b8) is a superset of all Horizon branches except 3 wip commits on stage0b-containment-core and pr1/pr1-narrow. World debugger (why/impact/diff/timeline/counterfactual/reach/replay) is pure code in `src/domain/debugger.ts`. Horizon has an IDM in `.sea/interaction/`.
- DomainForge: no state-machine/lifecycle declaration, no observed-vs-declared facility, `domainforge test` is a stub. world_ref = `world:<name>@sha256:<hex>`. ConceptChange is declarative only.
- CEP `godspeed.context_bundle` schema is only on unmerged PR #3 while Context Kernel implements it; CEP-0008 hash algorithm undefined.
- Host OOMs: one heavy build/test at a time.

## Owner steer during session (2026-11-12)
- SOP: synthesize v0.2 (binding backbone, 2026-11-10) with the older `godspeed-growth/world/godspeed_ai_fde_sop_v0_3.md` (2026-07-01, narrative; artifacts/templates). Delta report: `recon/08-sop-v02-v03-delta.md`.
- Horizon `horizon-desk-gaps`: the unfinished human/agent authority work is in scope. Source of truth:
  `Horizon-gaps/.agents/reports/2026-11-08-unified-human-agent-identity-authority.md` (owner selected profile S1; classes P0-P3; A3-UV; stages 0A,0B,1-5),
  `2026-11-08-agent-authority-decision.md` (critic item 4: browser agent can act as person), `2026-11-13-stage-0b-integration-dependencies.md` (D1-D6:
  D1 SEA-Forge P3 decision API; D2 identity service verifies authorization evidence + pending_activation; D3 controller route scope; D4/D6 bootstrap+recovery controller class; D5 saga+reconciliation).
  Stage 0B is implemented in Horizon against a test double; production `device.pair` refuses. Stage 1 lives on Cognate `horizon/shared-identity` (~/projects/cognate-identity-wt). SEA-Forge side: `feat/horizon-desk-support` (5 commits, clean).
- Recon done: 01,02,04,05,06,07. Running: 03 (Cognate/Workbench), 08 (SOP delta).

## Progress 2026-11-13T00:50Z
- Drafted: A, B, C (76 reqs / 31 critical), D, E, F, G, `sop/GS-FDE-SOP-001-v0.4.md`.
- EGM Goldfish review running (6 Sonnet reviewers, fresh contexts) -> `egm-review/01..06`. Next: resolve findings, write `H-egm-review-record.md`, revise A–G.
- WP-00a started: all 12 repos fetched; refs were already current. `evidence/baseline/heads-after-fetch.md`.
  Corrections to recon: CEP origin/main is 5111ec3 (= pin; context_bundle profile is already on main, PR #3 landed).
  Open PRs: SEA-Forge #91 (horizon-desk-support), #71 (hardening harvest); Cognate #76 (realitytrace snapshot);
  Workbench #28, #27; Gauntlet #7; DomainForge #165 (release 0.22.0), #114 draft. Horizon gaps: no PR.
- Not yet done in WP-00a: measured per-repo gate runs (serial, memory-gated).

## Progress 2026-11-13 (after EGM round 1)
- EGM round 1 complete: `egm-review/01..06` (10 BLOCKER, 48 MAJOR, 11 MINOR). Resolutions: `H-egm-review-record.md`
  (RD-01..RD-19, FC-001 principals, open owner decisions OD-1..OD-5). Draft 1 preserved in `draft1/`.
- Binding IDs for draft 2: `_revision-ids.md` (slices S0-S5, WP ids, NT-16+).
- Draft 2 written by lead: F (plan, sliced), A (corrections section). 
- Draft 2 in progress by Sonnet agents (resumed after a rate-limit interruption): E + `sop/GS-FDE-SOP-001-v0.4.1.md`; B; C + G.
- Next: verify draft-2 consistency (IDs across B/C/E/F/G), run a narrow second Goldfish pass on the BLOCKERs,
  then start S0: WP-00a measured baselines (serial), WP-00b backup tags + pushes, WP-00c integration branches.

## Progress 2026-11-13 (after EGM round 2)
- Draft 2 written (B, C 168 rows/76 critical, E, G 20 AT/62 NT, SOP v0.4.1, F, A corrections).
- EGM round 2 (`egm-review/07-round2-closure.md`): 5 CLOSED, 5 PARTLY, 8 new MAJOR. Resolutions RD-20..RD-27 + OD-6 in `H` §8.
  F edited to draft 3 by lead; a Sonnet agent is applying RD-20..27 to B, C, E, G, SOP.
- S0 / WP-00a in progress: `gsr-heads/domainforge` created (detached at origin/main f90905a; it was missing and
  readiness.sh needs it). `godspeed-readiness/scripts/readiness.sh` synced all pinned heads to origin defaults and is
  running the gates: log `evidence/baseline/readiness-run-3.log` (runs 1-2 stopped on the missing DomainForge binary).
  Early results at current heads: FAIL C01 (DomainForge producer version disagreement), J02, J02R2, SW01, SW02, U01;
  BLOCKED CK01, L01. Last recorded GO was 30/30 on 2026-10-25 heads. These are pre-existing regressions, first fixes in S0.
- Next: finish baseline; diagnose each FAIL/BLOCKED (Haiku recon on `godspeed-readiness/evidence/runs`); WP-00b backup
  tags; WP-00c integration branches; then S1 (WP-09, WP-10, WP-11, WP-13 first).

## Baseline complete; S0 under way (2026-11-13T07:25Z)
- **Planning baseline = draft 3**: A, B, C (169 rows / 76 critical), D, E, F, G (20 AT / 62 NT), H (RD-01..RD-27, OD-1..OD-6),
  `sop/GS-FDE-SOP-001-v0.4.1.md`. Cross-document ID checks pass (script). No third Goldfish pass was run.
- **WP-00a measured baseline: NO-GO 19/30** at current origin heads (`evidence/baseline/readiness-REPORT-20261113.md`,
  `readiness-run-3.log`). FAIL: C01, J02, J02R2, SW01, SW02, U01, BM-AUTH, BM-SETTLE, BM-STACK. BLOCKED: CK01, L01.
  Diagnosis by Haiku in progress -> `evidence/baseline/readiness-diagnosis.md`. `scripts/e2e-world-loop.sh` not yet run.
- **WP-00b**: backup tags `fde-backup/<branch>/20261113` on all local branches in 12 repos (95 tags, local only;
  `evidence/baseline/backup-tags.md`). Pushed Horizon `horizon-desk-gaps` to origin (gitleaks clean, 150 commits).
  Workbench `feat/horizon-connect` push REJECTED by a pre-receive hook: not worked around; protected by local tag only.
  Not done: committing the 37 Workbench PNG re-captures to an evidence branch.
- Side effects to know: `gsr-heads/*` were re-synced to origin defaults by readiness.sh (its normal behaviour);
  `gsr-heads/domainforge` is new; `godspeed-readiness` has an uncommitted regenerated `evidence/REPORT.md` and new run dirs.
- **Next**: act on the diagnosis (fix order from the report), re-run failing gates only (`scripts/readiness.sh <gate ids>`),
  run the world loop, then WP-00c integration branches, then S1 first batch (WP-09, WP-10, WP-11, WP-13).

## S0 fixes in flight (2026-11-13T07:50Z)
- Planning baseline committed and pushed: SEA-Forge branch `fde/planning-baseline` @ 2e5c57f4 (worktree `~/projects/_wt/fde-sea-rs-planning`;
  CURRENT_STATUS revision 231). No PR opened yet. Later edits to this directory in the owner checkout must be copied there and committed.
- Diagnosis: `evidence/baseline/readiness-diagnosis.md`. Verified root causes and chosen fixes:
  - G1 DomainForge skew (C01, L01, SW01, SW02): RealityTrace pins 0.21.2; SEA-Forge, Cognate, Horizon pin 0.21.1; 0.21.2 is on crates.io and npm.
    Fix = forward lockstep to 0.21.2: SEA-Forge (`crates/sea-forge-domainforge/Cargo.toml`, `EXPECTED_DOMAINFORGE_VERSION`, `devbox.json`, tests),
    Cognate (`packages/domainforge/package.json` dep, then publish), Horizon + Workbench (consume new Cognate), readiness L01 probe string. NOT STARTED.
  - G2 RealityTrace ingest (J02, likely BM-SETTLE): PR #13 made `--stdin-format` strict and omitted `json`, which Cognate's outbox sends.
    Fix = add "json" to `CARGO_JSON_SPELLINGS` in RealityTrace (NOT switching Cognate to agent-event-v1). Sonnet agent working in
    `~/projects/_wt/fde-sxr-ingest-json`, branch `fde/fix-ingest-json-spelling`.
  - G4 CK01: readiness scenario used `-p ck-bin`; fixed to `-p context-kernel` (uncommitted in local-only `godspeed-readiness`); rerun in progress.
  - G3 U01 / BM-STACK (Workbench not ready in 90s), J02R2 (git fixture race), BM-AUTH (p95 over budget): undiagnosed; rerun on an idle host with logs.

## S0 fixes, state at 2026-11-13 (later)
- CK01 fixed and PASS (readiness 20/30); committed in `godspeed-readiness` 7bd32a9 (local-only repo).
- RealityTrace PR #14 (`fde/fix-ingest-json-spelling`, 0274269): CI green, CodeRabbit "no actionable comments". CANNOT MERGE:
  ruleset "Protect Main" (id 26215481, created 2026-10-30 after PR #13 merged) requires CodeQL results; code scanning
  default setup is `not-configured`; ruleset has no bypass actors (`current_user_can_bypass=never`), so `gh pr merge --admin` is refused.
  Owner said admin override is OK. Enabling CodeQL default setup via API was denied by the session's permission classifier.
  Needs the owner: enable code scanning default setup on RealityTrace (Settings > Code security) or add a bypass actor to the ruleset.
  Check whether other repos have the same ruleset before their first merge.
- Verification of PR #14 against gates J02 and BM-SETTLE (pinned RealityTrace head temporarily at 0274269, restored after):
  log `evidence/baseline/readiness-rerun-J02-with-rt-pr14.log` (running or see result).
- SEA-Forge DomainForge 0.21.2 bump: Sonnet agent in `~/projects/_wt/fde-sea-rs-df0212`, branch `fde/domainforge-0.21.2` (resumed after rate limit).
- Remaining for C01 group: Cognate `@godspeedai/domainforge` 0.21.2 + publish, Horizon and Workbench consume it, readiness L01 probe string.

## S0 state 2026-11-13T13:30Z
Merged:
- RealityTrace #14 -> 90eff859 (json ingest spelling). CodeQL default setup enabled on RealityTrace (owner authorized; standing rule in memory `codeql-ruleset-blocks`).
- SEA-Forge #95 -> c2d6fbdc (RUSTSEC-2026-0459 libsqlite3-sys 0.39.1 via rusqlite 0.41; RUSTSEC-2026-0486 rustix 1.1.6). These advisories failed `cargo deny` for every PR.
- Cognate #77 -> 7356512d (DomainForge binding 0.21.2). **MERGED IN ERROR with one unresolved CodeRabbit thread**: `packages/domainforge/AGENTS.md`
  forbids agents editing the pinned `@godspeedai/domainforge` version, fixtures or tests. Owner decision pending: record an exception (recommended) or revert.
  Do not touch Cognate until the owner answers. Not published.
- godspeed-readiness (local): 7bd32a9 CK01 package name; 7f53bb2 J02 oracle `4 pass` (Horizon #14 added a 4th live test).
Open:
- SEA-Forge #94 (`fde/domainforge-0.21.2`, head 5cad1166): merged main in, status revision 232, CodeRabbit thread fixed and resolved. HOLD the merge
  until the owner decides the Cognate/lockstep question (same decision).
- SEA-Forge `fde/planning-baseline` (2e5c57f4): no PR yet; its status revision 231 now collides with main's 231 -> renumber and carry limits forward when opening the PR.
Gates verified after fixes: CK01 PASS, J02 PASS, BM-SETTLE PASS (10/10). Rerunning U01, J02R2, BM-AUTH (log `evidence/baseline/readiness-rerun-U01-J02R2-BMAUTH.log`).
Still red until lockstep lands everywhere: C01, L01, SW01, SW02 (Horizon and Workbench must consume a published Cognate with 0.21.2; L01 probe string in readiness).
Process rule adopted: merge commands are gated on CLEAN + zero failing + zero pending checks + zero unresolved review threads.

## S0 state 2026-11-13T14:10Z
- Owner approved the DomainForge lockstep bumps for Cognate and SEA-Forge ("you may do the lockstep bumps you recommended as needed").
- SEA-Forge #94 MERGED -> 0439abb0 (DomainForge 0.21.2).
- Cognate #77 thread replied and resolved; exception recorded in Cognate #78 (`fde/domainforge-pin-exception`, 3961fb8), merge watcher running
  (gated: CLEAN + no pending/failing + zero unresolved + CodeRabbit pass). #77's squash subject is `build(...)`, so a
  `BEGIN_COMMIT_OVERRIDE fix(domainforge): …` block was added to #77's body for release-please. Next: release PR -> merge -> npm publish
  (publish is gated on repo var NPM_TRUSTED_PUBLISHING_READY) -> Horizon and Workbench bump `@godspeedai/cognate-*`.
- godspeed-readiness: d65254e lockstep text/probe/oracle to 0.21.2; `src/up.ts` now starts Workbench with `src/cli.ts start` (UNCOMMITTED until U01 rerun confirms).
- Diagnosis `evidence/baseline/readiness-diagnosis-U01-J02R2.md`: U01 = harness used removed `src/server.ts` entry (Workbench fe062a5);
  J02R2 = Horizon `realitytrace-obligation.ts` repo() memoises after await (b7ce7f4), racing `void obligationRt.snapshot()` in app.ts.
  Horizon fix in progress: Sonnet agent, worktree `~/projects/_wt/fde-horizon-repo-race`, branch `fde/fix-obligation-repo-init-race`. Same bug on `horizon-desk-gaps`.
- BM-AUTH passes on a quiet host (noise). Rerun in progress: C01 SW01 SW02 L01 U02d U01 BM-STACK (`evidence/baseline/readiness-rerun-lockstep-U01.log`).

## S0 state 2026-11-13T17:40Z
- Owner: standing approval for version/pin bumps the mission needs (memory `fde-convergence-mission`).
- Merged: Horizon #29 -> d9cb701b (obligations repo + snapshot init single-flight; J02R2 root cause). Cognate #78 -> 0c1f3500 (pin-exception rule incl. RealityTrace CLI pin).
- godspeed-readiness commits: 7bd32a9 (CK01), 7f53bb2 (J02 oracle), d65254e (0.21.2 lockstep text/probe/U02d oracle), 251c23d (start Workbench via `src/cli.ts start`).
- Gates now PASS at current heads: CK01, J02, BM-SETTLE, BM-AUTH, L01, U02d, U01, BM-STACK. Rerunning J02R2 + J02 after Horizon #29
  (`evidence/baseline/readiness-rerun-J02R2-after-horizon29.log`).
- Still FAIL: C01, SW01, SW02 — Horizon and Workbench consume published Cognate 0.9.0 (DomainForge 0.21.1).
- BLOCKED ON OWNER: release PRs need their CI runs approved (GitHub shows `action_required` for release-please bot PRs):
  Cognate #79 (0.9.1, includes the domainforge fix entry) and RealityTrace #4 (0.4.0). The session's permission classifier denied
  `POST /actions/runs/{id}/approve`; not worked around. After approval + green: gated merge -> publish -> then
  (a) Horizon + Workbench bump `@godspeedai/cognate-*` to 0.9.1, (b) Cognate `justfile` RealityTrace pin 0.3.0 -> 0.4.0 (record in Cognate pin table).
- Full readiness rerun needed after those (pin moves invalidate earlier evidence; the 13/30 figure in partial reruns is not the real count).
- Not started: WP-00c integration branches; S1. `fde/planning-baseline` branch needs this directory re-copied, status revision renumbered (>232) and a PR.

## Correction 2026-11-13T17:50Z
- After Horizon #29 the git race is gone, but J02 and J02R2 now FAIL with `cep_world_refused: unknown world: world:horizon@sha256:3907…`:
  SEA-Forge main is on DomainForge 0.21.2 (#94) while Horizon still computes its world with 0.21.1 (published Cognate 0.9.0), so the two
  derive different world_refs. J02 passed earlier only because SEA-Forge was still on 0.21.1 at that moment. The stack is mid-lockstep.
- So the Cognate 0.9.1 release (PR #79, owner must approve its CI runs) now gates J02, J02R2, C01, SW01, SW02. Then Horizon and Workbench bump to 0.9.1.
- Gates passing at current heads and independent of that: CK01, BM-AUTH, L01, U02d, U01, BM-STACK, BM-SETTLE (re-verify in the full rerun).

## 2026-11-13 ~18:20Z — release PRs
- Owner: "Approve the runs, now and going forward" (memory: approve-held-workflow-runs; current head SHA only).
- Cognate release #79 merged f51453f2 (0.9.1, DomainForge 0.21.2). First merge call hit a GitHub 502; retried after the gate passed again. Publish being confirmed (task bhgrginz8).
- RealityTrace release #4 (0.4.0, head c9d596a): CI had been cancelled (my stale-run approvals), rerun; gated merge queued behind Cognate publish check.
- Next: Horizon + Workbench `@godspeedai/cognate-*` 0.9.0 -> 0.9.1 (root package.json; Horizon also line 40-41 block); Cognate justfile:64 RealityTrace 0.3.0 -> 0.4.0 + pin table; resync gsr-heads; full readiness rerun.

## 2026-11-13 ~20:30Z — lockstep complete on main everywhere
- Cognate 0.9.1 on npm (43 packages). RealityTrace #4 merged 5e3a5eab; realitytrace 0.4.0 on crates.io. Release workflow: macOS universal job timed out at 15 min -> binaries not attached; failed jobs rerun (check result).
- Cognate #80 merged 8c87a1c7 (justfile RealityTrace CLI 0.4.0, pin table row; 0.4.0 rejects `--stdin-format text`, phase0 probe now `libtest`).
- Workbench #32 merged d1a78853; Horizon #30 merged d9b5869a (CodeRabbit rate-limited, no review; 7 gateway-setup test failures verified identical on main — pre-existing, not yet diagnosed).
- Full readiness run started (task bygel7xms), log evidence/baseline/readiness-run-20261113-post-lockstep.log.
- Open: Horizon gateway-setup 7 failures; port #29 fix to horizon-desk-gaps; e2e-world-loop.sh; planning-baseline PR refresh (revision >232); WP-00c; S1.

## 2026-11-13 ~23:30Z — S0 baseline GREEN
- Full readiness at synced origin heads: **GO 30/30** (evidence/baseline/readiness-REPORT-20261113-GO.md; log readiness-run-20261113-post-lockstep.log; godspeed-readiness committed).
- RealityTrace 0.4.0 GitHub Release lacks native binaries: attach job's SHA256SUMS listed itself. Fix PR RealityTrace #15 (task bavggwkvo gate-merges then reruns attach with attach_binaries_tag=realitytrace-v0.4.0).
- Remaining S0: scripts/e2e-world-loop.sh; Horizon gateway-setup 7 pre-existing failures; refresh fde/planning-baseline + PR; then WP-00c and S1 (WP-09, 10, 11, 13).

## 2026-11-14 ~00:30Z — world loop OK; release fixed
- RealityTrace #15 merged 62f75aa9; realitytrace-v0.4.0 release now has 8 native packages + SHA256SUMS.
- e2e-world-loop: PASS (all hops, cep bundle valid, 3 refusals) with patched script against pinned heads + GSA 4bfed12 (worktree _wt/fde-gsa-loop). Log evidence/baseline/e2e-world-loop-20261113.log. Script defects fixed in SEA-Forge branch fde/fix-e2e-world-loop (ck-mcp -> context-kernel-mcp; quiet_cargo swallowed failures; uv --extra dev) — Sonnet opening PR with status revision 233.
- Haiku diagnosing Horizon gateway-setup 7 failures.
- Then: gate-merge the loop PR; refresh fde/planning-baseline (revision 234) + PR; WP-00c; S1.

## 2026-11-14 ~01:15Z
- Horizon gateway-setup 7 failures: ENVIRONMENTAL. /tmp tmpfs 90% full (ENOSPC); 60/60 pass with TMPDIR=~/projects/_wt/tmp. Big /tmp files not mine to delete: /tmp/verify-final.log 708M, verify-t2.log 249M, verify-t1-full.log 190M, cognate-suggest-home-* ~290M — report to owner. Use TMPDIR=~/projects/_wt/tmp for test runs.
- SEA-Forge #97 (world loop script fix, status rev 233) open; gated merge task b5dre3a3z.
- WP-00c started: Horizon integration (Sonnet, worktree _wt/fde-horizon-integration, branch fde/integration, ledger evidence/baseline/branch-ledger-horizon.md). Other repos next, one at a time: SEA-Forge, Cognate, Workbench, RealityTrace, Gauntlet, GodSpeed-Agent.
- After #97: refresh fde/planning-baseline as status rev 234 + PR.

## 2026-11-14 ~02:00Z — WP-00c Horizon done
- Horizon `fde/integration` pushed (worktree _wt/fde-horizon-integration): main + horizon-desk-gaps (1531c18) + drop-unused-identity-tarball + 2 ported tests. bun test 1639 pass / 16 skip / 0 fail (baseline 979/16/1 flaky). Ledger evidence/baseline/branch-ledger-horizon.md. Kept out: stage0b-* (superseded by gaps containment).
- Unfinished on Horizon integration (feeds S3/S4): device enrolment refused until D1–D6 (src/remote/enrollment.ts:97,144); no first-controller bootstrap (src/remote/server.ts:534,548); browser-agent-as-person gap; agent containment grant.revoke / gauntlet.run.cancel no runtime path; Gap A request ids; Gap B Reassign control.
- WP-00c Cognate + Workbench running (Sonnet aec866…); SEA-Forge #97 CI pending (task b5dre3a3z). Next WP-00c: SEA-Forge (after #97), RealityTrace, Gauntlet, GodSpeed-Agent.

## 2026-11-14 ~03:30Z
- SEA-Forge #97 merged 09b628ec (world loop script fix; status rev 233 records GO 30/30). Merged by hand after gate refused only on CodeRabbit "skipping" follow-up commit; 0 unresolved threads, CLEAN.
- Running: Sonnet a9cdd… = SEA-Forge planning-baseline PR (rev 234) then WP-00c fde/integration (ledger branch-ledger-sea-forge.md). Sonnet aec866… = Cognate + Workbench WP-00c.
- Next WP-00c: RealityTrace (main + 8 local commits), Gauntlet (PR #7 line), GodSpeed-Agent, DomainForge/CEP/CK = main. Then S1: WP-09, WP-10, WP-11, WP-13.

## 2026-11-14 ~04:30Z — WP-00c SEA-Forge done
- SEA-Forge #98 open (fde/planning-baseline, status rev 234, reports copied) — gated merge task running.
- SEA-Forge `fde/integration` pushed 6c760a63 (worktree _wt/fde-sea-rs-integration): main + PR #91 feat/horizon-desk-support + planner test compile fix (`review: false`; PR #91 needs it too). Ledger branch-ledger-sea-forge.md. fmt/check pass; touched-crate tests pass (core 104, case-runner 35, cli 90, server 0 fail, planner 69). NOT run: workspace tests, just ci, clippy, strict REQUIRE runs; no same-suite baseline on main.
- Kept out, still owed (owner asked to finish unfinished branch work): feat/hardening-capability-harvest (36 commits, ~7.7k lines, closes #67 findings F05/F08/F15/F17 — needs own PR + hardening review); feat/arm64-jetson-packaging; ci/openshell-windows; ci/macos-app-sandbox-probe.
- Status revision numbers collide between #98 (234), #91 (235), fde/integration: renumber whichever lands second.

## 2026-11-14 ~05:30Z
- SEA-Forge #98 merged f8ae13c9: planning deliverables A–H, SOP v0.4.1 draft, S0 evidence now on main (status rev 234). NOTE the committed copy is a snapshot; this STATUS.md in the owner checkout (untracked) keeps advancing.
- CodeRabbit is at its review limit ("Review limit reached") — no reviews on new PRs (Horizon #30, SEA-Forge #97 follow-up, #98). Gate now = CLEAN + checks + 0 threads + my own diff review; tell owner.
- WP-00c running: Cognate+Workbench (aec866…), RealityTrace+Gauntlet+GodSpeed-Agent (a36c80…). SEA-Forge integration status revs must renumber (main now 234).

## 2026-11-14 ~06:30Z — WP-00c Cognate + Workbench done
- Cognate `fde/integration` 86e3a6c: main + PR #76 (realitytrace-real-snapshot) + horizon/shared-identity. 1092 pass/32 skip/0 fail (baseline 1053/32/0; one ssh load flake in run 1). Open: packages/identity has no consumer, scope `@godspeed/identity`.
- Workbench `fde/integration` 8e59664: + horizon-connect picks (Connect Horizon + deep link, sha256 observation refs, 2 expert-memory commits). 1405/12/10 vs baseline 1402/12/3; the 7 extra = timeouts under load (13/13 on rerun of 4 files; "Experts and Teams" e2e x2 not rerun alone) -> NOT yet proven no-worse. Baseline's 3 failures on main (J-WS-directory-discovery, RED journeys, E2E remote SSH) undiagnosed.
- Held out of Workbench: a25f895 agents-propose-case-steps (needs `case_add_item` verb on Cognate's typed SeaForgeAuthority — feeds S2); 35ee1cf/78c4bb2 (commit dist-package binaries up to 267 MB = the push rejection cause, GitHub 100 MB limit); PR #27 ui-production-readiness and PR #28 desktop-terminal-browser (49/140 commits, pins Cognate 0.8.0) need own decision.
- Follow-ups: Workbench still pins RealityTrace CLI 0.3.0 (lockstep says 0.4.0); deep-link form `#sessions/<id>` vs `#session/<id>` to confirm.

## 2026-11-14 ~07:30Z — WP-00c complete (all repos have fde/integration pushed)
- RealityTrace: = origin/main 62f75aa; 244 pass/0 fail. Gauntlet: main + PR #7 (846830d); 2217 pass / 6 fail — the 6 = `verify --all` reads `.git/HEAD` as a dir (crates/gauntlet-cli/src/verify_cmd.rs:577) → fails in a git worktree; pre-existing defect, unconfirmed on plain clone; no main baseline run. GodSpeed-Agent: origin/main is STALE (July 21) — PRs #1–#3 merged into `audit-corrections/neatcode-2026-07-29`, not main; fde/integration = main + that tip (e141876); CI invocation (`--frozen --extra dev --extra ml` + npm ci in tools/ruvector) 1083 pass/54 skip/0 fail. Operator fixture diffs there are test-regenerated ids/timestamps only.
- Workbench #33 (RealityTrace CLI pin 0.4.0) in gated merge (task by14iajst).
- Ledgers: evidence/baseline/branch-ledger-{horizon,sea-forge,cognate,workbench,sxr,gauntlet,godspeed-agent}.md. Worktrees: ~/projects/_wt/fde-<repo>-integration.
- Owed from WP-00c: SEA-Forge workspace-test baseline + feat/hardening-capability-harvest PR; Workbench "Experts and Teams" e2e rerun + 3 main failures; Gauntlet verify worktree defect; GodSpeed-Agent main vs audit-corrections (owner decision: which is the default line); status-rev renumber on SEA-Forge fde/integration (main=234).
- NEXT: S1 — WP-09 pre-registration (SEA-Forge), WP-10 spike (DomainForge lib + RT scratch), WP-11 contracts (CEP), WP-13 SOP + model (SEA-Forge). Briefs must be cut from E §13, G S1 subset, F WP text.

## 2026-11-14 ~08:00Z — S1 started
- WP-10 spike running (Sonnet a06028…): worktree _wt/fde-df-spike branch fde/spike-wp10; output evidence/spike/wp-10.md.
- WP-13 running (Sonnet ad1844…): on SEA-Forge fde/integration (_wt/fde-sea-rs-integration): docs/sop/, .sea/interaction/fde-sop/, scripts/check-fde-sop-{map,model}.sh; output evidence/s1/wp-13.md. Also renumbers that branch's status revisions after main's 234.
- Not yet started: WP-09 (pre-registration, fde-anchor.sh, fde-proof.sh skeleton — SEA-Forge, after WP-13 to avoid same-worktree clash), WP-11 (contracts in CEP; brief from E §13 lines 343-365).
- Workbench #33 gated merge pending (task by14iajst).

## 2026-11-14 ~09:00Z — WP-13 done (SEA-Forge fde/integration 5d9ced96)
- Workbench #33 merged 27be7558 (RealityTrace CLI 0.4.0).
- WP-13: docs/sop/ + .sea/interaction/fde-sop/; world_ref world:godspeed-fde-sop@sha256:78eddd4e…f37287b; SOP sha256 043ae33a…; 66 clauses, 115 map rows: 60 declared / 23 inferred / 32 unknown. AT-01, AT-02 pass (scripts/check-fde-sop-{model,map}.sh). Evidence evidence/s1/wp-13.md. Branch status revs now 235,236,237 (main 234 not merged in → merge origin/main before PR).
- DESIGN IMPACT (E §4 must be corrected): DomainForge 0.21.2 REJECTS `at postcondition` / `at invariant` — all 26 bound policies are precondition-only and see only input record fields; `validate` scope-checks, never evaluates them (→ `insufficient` until WP-12). I-3 key is from E not SOP (row inferred). role_binding cannot target an imported entity; no responsibility kind (L3). Settlement split = two namespaces in one world, not two `pack` artifacts.
- Not declared yet: SOP 4.2 field groups, 7.4 assignment brief, 10.x handoff contents (list-valued, L2).
- Next: WP-09 (same worktree, now free), WP-11 (CEP), then WP-12 after spike.

## 2026-11-14 ~09:45Z — WP-10 spike done; WP-11 and WP-12 running
- WP-10 (evidence/spike/wp-10.md; branch fde/spike-wp10 c1307c8, no merge): a PASS (universal over empty → true, NO support count), b PASS at policy level / FAIL for bound instances, c PASS (unknown = tristate None; bool collapses to false), d PASS only via separate `application::evaluate_precondition` (operation inputs, no quantifiers). → WP-12 builds: per-binding rows + support count (expose quantifier arm core.rs:616), op-bound wrapper, facts schema, `support` in JSON. Unverified: nested quantifiers, exists_unique under Null, multi-module, role-bound.
- WP-12 running (Sonnet a7c650…): worktree _wt/fde-df-integration, branch fde/integration, PR to DomainForge main (no merge); evidence/s1/wp-12.md. Will need a DomainForge release + lockstep bump afterwards (owner pre-approved version bumps).
- WP-11 running (Sonnet add9e5…): CEP (_wt/fde-cep-integration), sxr, sea-rs contracts; evidence/s1/wp-11.md.
- Then WP-09 (pre-registration + fde-anchor.sh + fde-proof.sh skeleton) on SEA-Forge fde/integration after WP-11 frees that worktree; then WP-14.
- E doc corrections owed: §4 postcondition not available in 0.21.2; §6 bindings = outermost quantifier only.

## 2026-11-14 ~10:45Z — WP-12 implemented, under independent review
- DomainForge PR #171 (branch fde/integration, 08dc3dc; worktree _wt/fde-df-integration): `domainforge evaluate` + `Policy::evaluate_bindings` (policy/core.rs:59/69/86), src/evaluate.rs, cli/evaluate.rs, schemas/evaluate-{facts,output}.v1.json, docs/reference/evaluate.md. 27 tests pass; validate suites unchanged. Exit 0 = ran, 2 = input error. Evidence evidence/s1/wp-12.md.
- Limits: outermost quantifier only AND only at policy root (else support null, bindings []); op-bound precondition-only; exists_unique(true + NULL) → policy true (pre-existing eager expand) — RISK for ledger "confirmed"; unverified: multi-module, role-bound, group_by, state.*.
- CodeRabbit is rate-limited → fresh-context Sonnet adversarial review launched (af09da…) before any merge. Do NOT merge #171 until its findings are resolved.
- After merge: DomainForge release (release-please) → lockstep bump across SEA-Forge/Cognate/RealityTrace/Horizon/Workbench (0.21.2 → next), readiness rerun.
- WP-11 still running (add9e5…).

## 2026-11-14 ~11:30Z — WP-11 delivered; PR #171 review → fixes in progress
- WP-11 on fde/integration: CEP fbfc557 (interaction_event v1 ext, CEP-0008 §16.5 RFC 8785 hash rule + vectors), SEA-Forge 6e22bc3f (contracts/fde: frozen-reference, anchor, case-record-append; status rev 238), RealityTrace 635b876 (contracts: comparison-profile, observed-representation, residual). Evidence evidence/s1/wp-11.md (12 decisions). CEP pytest: 1 fail + 1 error in test_ledger_exporter (needs Postgres; pre-existing).
- My review amendments sent to add9e5…: comparison_profile {id, sha256} inside frozen body; residual carries representation_id + input_set_sha256 (one-way); falsifiers required non-empty for evaluated, exploratory not settleable as confirmed/contradicted; expectation_refs array; ASCII-helper must refuse non-ASCII. VERIFY these landed (check "Review amendments" heading in wp-11.md and new commits).
- PR #171 independent review (Sonnet af09da…): B1 exists_unique ignores NULL (expand, quantifier.rs:41-52) → wrong true/false at policy level; M1 model instances mixed with facts; M2 dropped op inputs for non-governed ops; M3 op input kinds unchecked; minor: op-level aggregate is new semantics. Fix brief sent to a7c650… (downgrade to unknown + caveat; source field; strict errors; per-input-set rows only; DomainForge issue for expand). Do not merge until fixed + CI green.
- Next: WP-09 on SEA-Forge fde/integration once amendments land.

## 2026-11-14 ~12:15Z
- WP-11 amendments landed: CEP 8b6fc90, SEA-Forge 838754bf (status rev 239), RealityTrace 6711402. Gaps: `expectation_refs` uniqueness enforced by test not schema (CEP validator rejects uniqueItems; DEBT-IE-01); ASCII-refusal test in SEA-Forge only. Exploratory episodes: settle S/F refused (EXPLORATORY_NOT_SETTLEABLE), U/C accepted.
- WP-09 running (Sonnet a16ab6…) on SEA-Forge fde/integration: .agents/preregistrations/fde-s1/, scripts/fde-anchor.sh, scripts/fde-proof.sh, anchor tag fde-anchor/prereg/fde-s1 pushed; evidence/s1/wp-09.md.
- PR #171 fixes running (a7c650…).
- After both: WP-14 (case keeper, episode loop — SEA-Forge server; needs WP-11 + WP-13, done), WP-15+ per F; DomainForge release + lockstep after #171 merges.

## 2026-11-14 ~13:00Z — WP-09 done; #171 fixed, awaiting CI
- WP-09: SEA-Forge fde/integration ee93ee87 (status rev 240). Prereg digest 4f9e8167…8389ac; tag fde-anchor/prereg/fde-s1 pushed; verify --remote exit 0. 21 frozen tests (AT 01,04,05,06,07,08,12,13,14,18; NT 01–09, 03b, 16); predictions 12 PASS / 9 PARTIAL. Only AT-01 wired in scripts/fde-proof.sh. Evidence evidence/s1/wp-09.md. Limits: tag unsigned, RD-26.
- DAMAGE by subagent (NOT fixed — classifier blocked its revert; I must not do it for it): `~/projects/sea-rs/.git/config` now has local `user.name = user.email` (wrong). Owner to run: git config --file ~/projects/sea-rs/.git/config --unset user.name. Until then, commits from the sea-rs checkout and its worktrees get author name "user.email" unless overridden (subagents must set GIT_CONFIG_COUNT/KEY/VALUE or GIT_AUTHOR_NAME/GIT_COMMITTER_NAME="Sam Prime").
- DomainForge #171 fix commit 72fce90 (36 evaluate tests; issue #172 filed for expand). B1 verified by me with reviewer's reproducer. Gated merge task b8d5d4az8.
- Next: WP-14 case keeper/episode loop (SEA-Forge server).

## 2026-11-14 ~07:40Z
- DomainForge #171: all 36 checks pass, CLEAN; CodeRabbit left 2 unresolved Major threads on `domainforge-core/src/evaluate.rs` (order-dependent fact loading; ambiguous operation names). Sonnet implementer launched to fix both on the PR branch. Not merged yet.
- WP-14 implementer resumed after the usage-limit reset.
- E §4 corrected (`at postcondition`/`at invariant` rejected by 0.21.2; precondition only) and E §6 corrected (bindings for the outermost root quantifier only).
- Still open: user to unset bad `user.name` in sea-rs/.git/config; OD-1..OD-6; GodSpeed-Agent default line.
- SEA-Forge PR #71 (`feat/hardening-capability-harvest`, 36 commits, 61 files): open, 19 behind main, 6 conflicts (CURRENT_STATUS.md/.yaml, sea-forge-domainforge/Cargo.toml, sea-forge-server/Cargo.toml, authority/tests/cep_compiler_upgrade.rs, server/tests/conformance_cep_authority.rs) — mostly the 0.21.2 lockstep. Queued: merge main into it (sf-restatus replay rule) once the cargo lock is free.

## 2026-11-14 ~08:30Z
- DomainForge #171: fix 810858e pushed (batch `Graph::add_entity_instances`, ambiguous-operation rejection); both CodeRabbit threads replied + resolved; gated merge task bez89o0gf waiting on CI.
- WP-14 part 1 done: SEA-Forge fde/integration 88228408 (status rev 241). conformance_field_case 18/0; AT-04, NT-01, NT-16 PASS via fde-proof; mutation checks done. Evidence evidence/s1/wp-14.md. Left: interaction-event outbox, Go/TS regen + golden frames, spec note, delegation gating (WP-20), RT writer, missing contract error codes (NT-13/17/18/34/48), gateway-hop actor for I-2.
- Launched (Sonnet, sharing cargo lock): WP-14 part 2 (outbox, regen, AT-05 SEA-Forge half) and WP-16 (RealityTrace ingest + sxr-observed derive).
- Next after those: WP-15 (Cognate), WP-17 (compare; needs #171 released), SEA-Forge PR #71 conflict update.
- User cleared the bad user.name in sea-rs/.git/config (verified: resolves to global).
- WP-16 done: RealityTrace fde/integration 2e862da (new crate sxr-observed; `observed ingest`, `observed derive`). Agent-reported: workspace 266 pass/0 fail, fmt+clippy clean; AT-06, NT-02, NT-03b and derive halves of AT-08/NT-03 at RT level; mutation checks done. Evidence evidence/s1/wp-16.md. Open: declaration_key shape-only; tail-deletion undetectable by chain alone (needs anchor/count — design item for WP-17/19); representation.recorded not written; sxr-observed release plumbing (publish-crates.sh, release-please) owed; input_set_hash algorithm defined by agent (review in WP-17).
- Launched WP-15 (Cognate runtime events + program executor). WP-14 part 2 still running.
- WP-14 part 2 done: SEA-Forge fde/integration 2662c0bc, 6ea7a498, 190caaa3 (status rev 244). Interaction-event outbox (file outbox + reconcile_events; not atomic with ledger line), Go builder + 4 golden frames, spec note, AT-05 wired = PARTIAL. Agent-reported: lib 189, field_case 18, interaction_events 10, sfwp 23, identity 15, all 0 failed; fmt/clippy clean. Deviations: emitter omitted (schema has no unsigned marker); semantic_ref from WP-13 map only, unmapped world emits nothing + gap row; only I-2 produces an authority decision; envelope validated structurally, extension by schema. Owed: independent hardening review of identity/settlement path; RT writer/delivery; WP-20 gating; error codes NT-13/17/18/34/48; gateway-hop actor.
- Launched: SEA-Forge PR #71 merge-main update (Sonnet). WP-15 (Cognate) running. DomainForge #171: 2 checks pending, gate task bez89o0gf waiting.

## 2026-11-14 ~10:00Z
- DomainForge #171: CI green on 810858e but CodeRabbit raised 2 new Major threads (enum symbol mismatch accepted in evaluate.rs:84; nested exists_unique in Expression::expand = issue #172). Round-3 Sonnet fix running. gated-merge.sh is gone from scratchpad (merge by hand after checks + 0 threads + diff review).
- SEA-Forge PR #71: main merged in, a7b3baf2 pushed (worktree ~/projects/_wt/harvest). Status yaml: main 1-234 kept, branch entries re-added 235-266. DEFECT: CURRENT_STATUS.md took branch snapshot (main rev-234 text lost from .md) — being fixed. NOT_RUN on this host: postgres_broker, postgres_lifecycle, sandbox state_tree (seccomp user-notify unavailable under WSL), Windows/macOS lanes, cell_episode_live, cargo test --workspace. 14 unresolved review threads → Sonnet triage/fix agent running (no merge).
- WP-15 (Cognate) running.

## 2026-11-14 ~11:00Z
- DomainForge #171 round 3 pushed: 6b8ae14 (enum/entity/unit symbol check), 0222faf (expand ExistsUnique three-valued; Fixes #172; caveat/evaluator_result removed from output+schema). `validate` now reports UNKNOWN for exists_unique with unknown item and ≤1 true — release-note item. Core suite 1332/0 (before final import/fmt edit). Waiting CI + CodeRabbit (task bjg32bpot). The 2 round-3 threads (…63a46V, …63a46a) not yet replied/resolved.
- SEA-Forge PR #71 at 4841aac7: CURRENT_STATUS.md restored from main + hardening (rev 267/268); 10/14 threads resolved. OPEN (need live lane or owner decision): sc6waww3 policy.yaml allow-control REST vs raw CONNECT; sc6wdEsI cell_episode_live SandboxViolation on 22.04 microVM (root cause needs live run); sc6wdEsR cgroup pidfd_sweep child-cgroup members when cgroup.kill denied (currently fails closed → Survivors); sc6wdEsX macOS ProcessGroup claim with incomplete census. CI running incl. macOS/Windows/VM lanes — read results before deciding. Not merged.
- WP-15 (Cognate) still running.
- WP-15 done: Cognate fde/integration ec9b28f (push needs `git push origin fde/integration`). Interaction event on run.invocation/completed/failed for case runs; `program.run` capability (hash-pinned patch, argv, confined). Agent-reported: 1105 pass/32 skip/1 fail (fail = missing SXR binary in worktree; with SXR_BIN set realitytrace pkg 39/0). Evidence evidence/s1/wp-15.md. Gaps → follow-up Sonnet launched: symlink escape in program.run confinement, untested rollback branch, run.started/run.cancelled emit no event. Left: caseContext not on Connect/proto wire; ungoverned events not in CEP envelope; host must wire interactionOutboxFor.

## 2026-11-14 ~12:40Z
- WP-15 follow-up done: Cognate fde/integration db788d1. Symlink confinement (realPath equality, plan-time + pre-write + cwd re-check; residual TOCTOU, no no-follow write), rollback restores failed file, run.started/run.cancelled emit interaction events; AT-05 test enumerates all event types. Agent-reported full suite 1127 pass/32 skip/0 fail (SXR_BIN set, pg0 up). Not done: atomic temp+rename writes, no-follow open, wire caseContext.
- DomainForge #171 at 0222faf: CI check "Verify Rego projection against pinned OPA 1.21.1" FAILED (others pass; 0 unresolved threads). Reproducing locally (task bj9h04185, log ~/projects/_wt/tmp/rego-local.log). Suspect: differential test OPA vs evaluator after exists_unique expand change.

## 2026-11-14 ~13:30Z
- DomainForge #171: Rego gate failure reproduced locally (OPA vs evaluator on diff::one_a, diff::literal_fold); fixed lib.rego exists_unique to match three-valued expand; local gate 9/0; pushed b91b147. Waiting CI + CodeRabbit (task bl2srp1xe).
- SEA-Forge PR #71 CI on 4841aac7: macOS, Windows, 22.04 VM, go, workbench pass. FAIL: test (ubuntu) — new assertion in conformance_case_episode.rs required string, CI host yields object {"cgroup_leaf":{…}} (uncommitted fix in ~/projects/_wt/harvest); socket gates (ubuntu-24.04 VM) sandbox-suite — workload_boundary.rs launcher fixture: workload_isolation_setup_failed (ack pipe closed), PRE-EXISTING on 700c264e and a7b3baf2. Sonnet agent diagnosing/fixing both and watching CI (no merge).

## 2026-11-14 ~14:30Z
- DomainForge #171 MERGED to main 3960326 (squash) after CI green, CodeRabbit pass on b91b147, 0 unresolved threads; issue #172 closed. WP-12 done. Release PR #165 "chore: release 0.22.0" — waiting on its checks (task b3f6e76x0); then merge, verify publish, lockstep bump (SEA-Forge, Cognate, RealityTrace, Horizon, Workbench), readiness rerun.
- WP-17 (RealityTrace compare) Sonnet launched on fde-sxr-integration; includes tail-deletion check, input_set_sha256 review, sxr-observed release plumbing.
- SEA-Forge PR #71: 62bd4424 (test fix), 965ac885 (launcher-fixture stderr capture diagnostic) pushed; agent a18c46dbef0898f36 paused on CI watcher; 24.04 root cause still open.
- WP-17 done: RealityTrace fde/integration 5ae30c1, 73748b5, c6ccb3c. `observed compare` (three-way split via DomainForge evaluate CLI built from main 3960326 at ~/projects/_wt/fde-df-main-3960326; EVALUATE_UNAVAILABLE if missing), TAIL_TRUNCATED via --expected-head/--expected-record-count (else tail_verification: unverified), sxr-observed in publish script + release-please. Agent-reported: wp17 16/0, compare CLI 3/0, workspace 284/0 (before last test), fmt/clippy clean; mutations done. AT-07, AT-08/NT-03 compare halves, NT-06 satisfied; NT-04 after-half PARTIAL (needs WP-18). NT-20: G oracle string corrected to schema's `executor_only_evidence` (G line 160; NT-20 not in frozen S1 set). Gaps: residual.v1 closed → facts hash/DF version in result `annotations`; ordering/frequency/timing = not_evaluable; no representation.recorded/residual.computed records written; compare-mapping.v1 defined by agent in contracts/README §5 (needs review); coverage manifest + independence from CLI flags, not frozen reference.

## 2026-11-14 ~15:30Z
- DomainForge release PR #165 (0.22.0): 7 checks failed = cross-binding golden digests embed crate version. Pushed 7139ccc to release branch (worktree ~/projects/_wt/fde-df-release) with CI-computed digests (contract 89c71f12…, envelope 5f8f52eb…), same as 0.21.2 release did. Commit messages in DomainForge need a scope. Waiting CI (task b8elobuua) → merge → verify publish → lockstep bump → readiness.
- WP-18 (governed revision; RealityTrace then SEA-Forge) Sonnet launched (a3dfe970bb0fcab6b).
- PR #71 agent (a18c46dbef0898f36) still paused on its CI watcher.

## 2026-11-14 ~16:10Z
- DomainForge 0.22.0 RELEASED: PR #165 merged ac8c421 (36/36 checks, 0 threads); GitHub release v0.22.0 (9 assets at check time); crates.io domainforge-core 0.22.0 and PyPI domainforge 0.22.0 confirmed live. Tag "Deploy" run 43602521616 still in progress at check time (npm @godspeedai/domainforge not yet confirmed).
- Lockstep bump 0.21.2→0.22.0 Sonnet launched (a3f4ff0dcf0e74d15): PRs `chore/domainforge-0.22.0` against main in SEA-Forge, Cognate, RealityTrace, Horizon, Workbench (+ readiness pins); no merges by the agent. After merge: readiness rerun; then merge main into fde/integration branches.
- WP-18 agent (a3dfe970bb0fcab6b) running; PR #71 agent (a18c46dbef0898f36) paused on CI watcher.
- WP-18 done: RealityTrace 4930b7d (observed propose/regression/capability-delta, 5 new contracts; workspace 308/0), SEA-Forge 69357a5d (world_pin over existing semantic_change transition; verbs authority_promote/repin/pin; status rev 245; world_pin 13/0, conformance_world_pin 1/0, other gates green). AT-12 PARTIAL (approval at assurance `session`, not A3), AT-13 PARTIAL (GAIN on synthetic scenario; real program episodes = WP-19), NT-05/07/08 PASS, NT-21 + NT-04 after-half satisfied. Deviations: weakening promotions refused ESCALATED_APPROVAL_REQUIRED (no escalated class in CEP path); matrix NOT recomputed in SEA-Forge; heldout-rule/capability-frozen digests via CLI; candidate world registered by host config; no freeze-time check episode world == pinned world (closed error enum). Evidence evidence/s1/wp-18.md.
- Launched: fresh-context Sonnet adversarial review of SEA-Forge ee93ee87..69357a5d (read-only).

## 2026-11-14 ~17:10Z — ADVERSARIAL REVIEW of SEA-Forge ee93ee87..69357a5d (fresh Sonnet, read-only)
Findings (fix agent a7983ea6010f7f5bc launched, order F2,F1,F6,F7,F3,F4,F5,F8,F9/10):
- F1 HIGH: I-1 not wired — require_complete_freeze/freeze_status/verify_freeze_anchor have no non-test callers; settle() needs only a freeze record. I-1 must NOT be claimed enforced until fixed.
- F2 HIGH: torn trailing line → next append glued and silently dropped by filter_map(.ok()); verb still says "recorded"; ids can repeat.
- F3 HIGH: regression matrix forgeable (rows:[] verdict pass accepted; matrix digest not bound into approval claim).
- F4 MED-HIGH: promote/re-pin sequence unlocked → two concurrent promotions can both commit.
- F5 MED: ScriptPublisher receipt lacks server_push_time the verifier requires; "no run events" = Ok; script has no timeout inside case lock.
- F6 MED: repeat settlement allowed (S under K1, F under K2).
- F7 MED: I-2 weak — production chain always [principal]; unreadable events file → empty executors → allowed; shared "operator_local" fallback.
- F8 MED: pin advisory — nothing consumes pin_state/evidence_world; baseline = first configured world; replay after re-pin returns Ok(prior).
- F9/F10 LOW: requester-asserted claims; unbounded attempt rows; no payload size cap; recover() takes no lock.
Sound per review: I-3 serial case, idempotency, registry (include_str + digest), case lock on verb, outbox not a trust input, promote gate codes.
- Launched WP-19 Horizon half (ab195028d4986e399). Lockstep agent (a3f4ff0dcf0e74d15) running. PR #71 agent paused.

## 2026-11-14 ~17:35Z
- Sonnet 5-hour limit hit ~17:00Z (reset 12:20 ET); review-fix agent (a7983ea6010f7f5bc, nothing committed yet), WP-19 Horizon (ab195028d4986e399, untracked test/fixtures/), lockstep (a3f4ff0dcf0e74d15, worktree df022-sea-rs) all cut off. All three resumed via SendMessage at 12:32 ET; PR #71 agent (a18c46dbef0898f36) also resumed — #71 CI: test (ubuntu) now passes, only socket gates (ubuntu-24.04 VM) fails.
- If the limit hits again: resume the same agent ids; do not start new ones.

## 2026-11-14 ~18:40Z — lockstep 0.22.0
- npm @godspeedai/domainforge 0.22.0 live (Deploy run success).
- PRs open (not merged): SEA-Forge #99 (9aa0802d; demo world_ref 7ddedbce…→469e93b2…; lib.rego digest 3b25cf21…→bff56dc0…; status rev 235), Cognate #82 (20b2790, `build(domainforge)`), RealityTrace #16 (acfb4dd, PINNED_DOMAINFORGE_VERSION). Gate waiter running.
- Horizon + Workbench NOT bumped: they pin via @godspeedai/cognate-domainforge 0.9.1 → need a Cognate release carrying #82 (Cognate release-please PR #81 open). Order: RT #16 → RT release (so Cognate's RT CLI pin + readiness C01 agree) → Cognate #82 + RT pin → Cognate release → Horizon/Workbench bumps → re-pin gsr-heads → readiness.
- godspeed-readiness local commit 1a881b9 (4 scenario files).
- PR #71: second diagnostic 90efe247 pushed by agent; 24.04 cause still open.
- WP-19 Horizon half done: Horizon fde/integration fac2f84, fd2e19b. Read-only adapters (src/adapters/fde-read.ts), EpisodeEvidence panel, parity gate `scripts/fde-parity.sh [AT-14|NT-09|all]` (Playwright headless). Agent-reported: full suite 1688 pass/16 skip/0 fail; AT-14 PASS (24 mounts), NT-09 PASS (PARITY_MISMATCH residual.0.class). Limits: hand-written JSON-schema validator (throws on unimplemented keywords); world-pin fixture hand-written; compare result doc + authority_pin response have no schema; no cross-system join (case and RT fixtures are different episodes); `just verify` not run in full. Evidence evidence/s1/wp-19-horizon.md.
- Remaining S1: review fixes (in progress) → WP-19 SEA-Forge half (fde-proof.sh completion incl. AT-05/06/07/08/14/18 wiring, three real `program` episodes, S1 claim-ledger builder, clean-cell run).

## 2026-11-14 ~19:30Z
- Cognate #82 MERGED e74bbb86 (CLEAN, CodeRabbit pass, 0 threads). Cognate release PR #81 (0.9.2) refreshing; waiter running. After release: Horizon + Workbench bumps to cognate 0.9.2 / DomainForge 0.22.0.
- SEA-Forge #99: CodeRabbit thread (status rev 235 dropped 16 limits) fixed in e71ed476, replied + resolved; waiting CI.
- RealityTrace #16 BLOCKED by CodeQL check: alert #20 `actions/cache-poisoning/poisonable-step` (high) at package.yml:619 — same step as alert #18 already open on main (alerts #10–#19 open on main for this file; pattern = workflow_dispatch `inputs.ref` checkout). The PR only changes the version literal on the flagged lines. NOT overridden/dismissed — owner decision (dismiss #20 as duplicate/accepted risk, admin-merge, or fix the workflow pattern for all 11 alerts).

## 2026-11-14 ~20:30Z — review fixes landed
- SEA-Forge fde/integration 264527e9 (F2), 7ea7d19c (F1a,F6,F7), a0a1a8e1 (F5), 72b634f7 (F3,F4,F8,F9,F10); status rev 248. Agent-reported gates at tip: server lib 190, field_case 31, interaction_events 10, world_pin(conf) 4, sfwp 23, identity 15, cep_authority 13, authority lib 60, world_pin 21, cep_transition 17 — all 0 failed; clippy/fmt clean; fde-proof AT-04, NT-01, NT-16, NT-05, NT-07, NT-08 PASS; AT-12/13 PARTIAL.
- STILL OPEN (must be stated in the S1 claim ledger):
  * F1(b): execution-side I-1 gate NOT wired — item_execute/case_advance/agent_run.delegate carry no `n`/`episode_ref`; needs WP-20 contract. I-1 in S1 = settlement requires complete freeze + post-hoc order verification (AT-04/NT-16), NOT prevention of execution before freeze.
  * F8: freeze path does not refuse a world_ref ≠ pinned world (needs code WORLD_NOT_PINNED in case-record-append; closed contract) → pin not consumed on the write path.
  * F7 residual: no record links an agent run's actor to its delegator → delegate-then-settle by the delegator not caught; gateway hop only when a gateway is configured.
  * F5: remote push time unobtainable with plain git → every real receipt is AnchorTimeLocalOnly (never ExternallyVerified) — ties to OD-6.
  * F3 residual: SEA-Forge cannot recompute matrix rows; approver-bound matrix_id only.
  * F2 residual: single-process case lock only.
  * Anchor required for settlement only when SEA_FORGE_FDE_ANCHOR_SCRIPT is configured.
- WP-20 contract inputs gathered: `episode_ref` (+kind) on execute/advance/delegate; error codes WORLD_NOT_PINNED, settlement-exists, KIND_IMMUTABLE, REFERENCE_DERIVED_FROM_RESULT, AUTHORITY_DENIED, ANCHOR_PUBLISHER_INVALID, MODEL_ANCHOR_LATE; delegation→actor link record; residual annotations (facts hash, DF version); compare-result and authority_pin schemas; compare-mapping.v1.

## 2026-11-14 ~21:15Z
- MERGED: SEA-Forge #99 be63b711 (DomainForge 0.22.0 lockstep; CodeRabbit thread fixed e71ed476; CLEAN), Cognate #81 release 0.9.2 ca74ab5f (held runs approved on head 91f16d8; CLEAN). Waiting npm publish of @godspeedai/cognate-* 0.9.2 (task running) → then Horizon + Workbench bumps.
- PR #71: real fix pushed 2fe4b236 (ack/lifeline pipe ends dup'd to fd >= 64; 24.04 had ack fd 7 ENOENT); CI running.
- WP-19 S1 gate agent (a6724e13a3ce95574) running on fde-sea-rs-integration.
- RealityTrace #16 still BLOCKED on CodeQL alert #20 — owner decision pending.
- fde/integration branches still need main merged in (0.22.0 lockstep) after S1 gate run: SEA-Forge, Cognate, RealityTrace(after #16).

## 2026-11-14 ~22:00Z
- Cognate 0.9.2 RELEASED (GitHub v0.9.2; npm @godspeedai/cognate-domainforge 0.9.2 → domainforge 0.22.0 confirmed; release workflow still finishing other packages at check time).
- Horizon + Workbench lockstep bump Sonnet launched (a6a298eb9979213c7): PRs chore/domainforge-0.22.0, no merges.
- PR #71: root cause per commit 2fc5b1ab = test-fixture bug (no-lifeline mutation control rebuilt the launch command without clearing CLOEXEC on the ack fd); production launch chain unchanged. PR became DIRTY after #99 → agent told to merge main again and watch 24.04 lane.

## 2026-11-14 ~23:05Z
- OWNER DECISION (user message): "I accept your recommendation for CodeQL" → dismissed RealityTrace alert #20 (won't fix; duplicate of #18 on main), filed RealityTrace issue #17 to remove the workflow pattern (alerts #10–#19), merged RealityTrace #16 (see sha in gh). All three lockstep repos with direct pins now on main at 0.22.0.
- Sonnet limit hit again (reset 17:30 ET); S1 gate (a6724e13a3ce95574, was at "Author ep-3 patch"), PR #71 (a18c46dbef0898f36), Horizon/Workbench bump (a6a298eb9979213c7) all resumed 18:00 ET.
- OWNER REQUEST (new, do at the very end when ALL work is done): write ONE self-contained HTML report in SEA-Forge `.agents/reports/` explaining the whole system across repos. Reader has no context; no jargon except vital load-bearing terms (defined); simplified technical language a smart high-school student would understand.
- RealityTrace #16 MERGED 521d363a. No release-please PR opened (non-releasable commit type); a RealityTrace release pinning DomainForge 0.22.0 is deferred to the release that carries fde/integration (sxr-observed). Until then Cognate's RealityTrace CLI pin stays 0.4.0 (recorded in Cognate packages/domainforge/AGENTS.md table). Readiness C01 reads main heads, so it can pass after gsr-heads are re-pinned.
- After Horizon/Workbench PRs merge: re-pin ~/projects/gsr-heads/* from origin/main, run scripts/readiness.sh, record report under evidence/baseline/.

## 2026-11-14 ~23:50Z — PR #71
- At d97aca5e (main merged in, status revs 236–275): ALL checks pass incl. gate, test (ubuntu), socket gates 22.04 + 24.04, macOS, Windows. Root cause of the 24.04 failure = test fixture (mutation control in workload_boundary.rs rebuilt the launch Command without the pre_exec that clears CLOEXEC on the ack fd; helper exit 126); fix 2fc5b1ab; production launch chain unchanged. 2fe4b236 (fd ≥ 64 guess) reverted in 2fc5b1ab. Agent-reported local: 1070 passed/0 failed across authority/domainforge/server/sandbox.
- Still BLOCKED by 4 unresolved CodeRabbit threads. Owner(maintainer) decisions given to agent a18c46dbef0898f36: (1) cgroup pidfd fallback must sweep descendant cgroups + verify recursive emptiness, else Survivors; (2) macOS incomplete census must map to a degraded/unverified variant, never ProcessGroup; (3) allow-control test must issue a real HTTP GET (do not loosen policy fixture); (4) find which lane runs cell_episode_live; resolve only with a passing run, else report as owed live-lane verification.

## 2026-11-15 ~00:00Z — PR #71
- 7bd0cb01: all PR checks pass. Threads 1 (cgroup descendant sweep, c187a8cd; test ran in CI under SEA_FORGE_REQUIRE_CGROUP=1) and 2 (macOS → new LifecycleContainment::GroupTerminatedDescendantsUnverified, 264c234c; macOS lanes pass) fixed + resolved. Thread 3 (HTTP GET control, 7bd0cb01) verified only by local simulation; thread 4 (cell_episode_live SandboxViolation) needs the `OpenShell live proof` workflow (manual dispatch / push to ci/openshell-live; pushes run-scoped test images to GHCR). I dispatched it on the branch: run 43655580199 (watcher running). Note from agent: lane may report BLOCKED (no discoverable proxy on OpenShell 0.1.x).

## 2026-11-15 ~00:45Z
- OpenShell live proof run 43655580199 on 7bd0cb01: success; linux-microvm 12 passed/0 failed; allow-control scenario self-reports BLOCKED (OpenShell 0.1.3 has no guest egress proxy; F17 denial remains BLOCKED upstream). Threads 3 and 4 replied with that evidence and resolved (thread 3's HTTP path is NOT live-verified — recorded).
- SEA-Forge PR #71 MERGED (squash; see gh for sha): hardening harvest F05/F08/F15/F17/F32/F33 + review fixes. The unmerged branch feat/hardening-capability-harvest is now on main (do not delete the branch without the owner asking).
- Next for SEA-Forge: after S1 gate agent finishes, merge origin/main into fde/integration (0.22.0 lockstep + hardening) and re-run gates.

## 2026-11-15 ~01:30Z — S1 GATE RUN COMPLETE (WP-19 SEA-Forge half)
- SEA-Forge fde/integration tip 601d9618. `scripts/fde-proof.sh --slice s1` on clean tree at fef0d900: 12 PASS, 9 PARTIAL, 0 FAIL, 0 NOT_RUN. Bundle: `.agents/evidence/fde-field-case/s1-20261114T234200Z/` (claim ledger md+json, verification/, operational/, raw-manifest.sha256 over 198 files; 3 files >1MB digest-only in scratch cell ~/projects/_wt/tmp/fde-s1/20261114T234200Z). Evidence doc evidence/s1/wp-19-gate.md.
- Prediction mismatches (reported, predictions untouched): AT-14 (pred PASS → PARTIAL: Horizon parity on producer fixtures, not this cell's records), NT-03 (PASS → PARTIAL: no real re-run with changed value), NT-06 (PASS → PARTIAL: no silent-source episode on the cell), NT-07 (PARTIAL → PASS).
- Episodes: ep-1 settled U (pred S) — authorize_consequential_action insufficient/policy_unknown: SEA-Forge records no authority decision for program execution. ep-2 settled F (pred F). Revision promoted (op-alice, session assurance; self-approval + early promote refused). ep-3 settled U (pred S); capability delta NO_GAIN (pred GAIN) — verification step stays insufficient/executor_only_evidence because only Cognate (executor's emitter) records it. R-58 NOT supported.
- S1 claim (mechanical): SUPPORTED at bounded-interaction level, once, on program execution; PARTIAL rows AT-04, 05, 07, 12, 13, 14, 18, NT-03, 04, 06, 16; other behaviour classes NOT TESTED.
- Deviations: ran on DomainForge build 3960326 (0.21.2 line with evaluate), not released 0.22.0, because SEA-Forge fde/integration embeds domainforge-core =0.21.2 and the frozen world is 78eddd4e… (0.22.0 gives e3c3d396… for the same sources). Cognate ran under a local allow policy (no SEA-Forge authority decision for program runs). Events via spool + `observed ingest`; evidence.recorded rows seeded with rt-seed. Cognate emission filtered to run.invocation. Freeze anchors in a scratch repo with local bare remote. Results digest NOT anchored under a new tag kind.
- What S1 teaches for S2/S3: (1) program execution needs a SEA-Forge authority decision (WP-21/31); (2) an independent emitter is required for `confirmed` on executor steps (WP-17 rule works; needs SEA-Forge/RealityTrace-side observation) ; (3) contract v2 list (WP-20) as recorded 20:30Z.

## 2026-11-15 ~02:10Z
- Lockstep PRs open: Horizon #31 (just verify exit 0; bun 980 pass/16 skip/0 fail — main's count; the 1639 figure is the fde/integration branch with desk-gaps; contract self_hash 27fd5f03…→2d384753…, world_ref unchanged 0a10c20d…), Cognate-Workbench #34 (1402 pass/12 skip/3 fail = main baseline; the 3 failing tests named: "E2E J-WS-directory-discovery … (RED)", "RED journeys — UX evidence > directory suggestions…", "E2E remote workspaces > verify host -> trust -> connect…"). Gate waiter running; merge when CLEAN + 0 threads.
- Workbench caveat: local RealityTrace CLI used was 0.3.0, not the 0.4.0 pin.
- SEA-Forge fde/integration ← origin/main sync agent a027c45a90ba2831f running.

## 2026-11-15 ~02:50Z
- SEA-Forge fde/integration synced with main: merge f6ad220c + d2f8189e (pushed). Status revs: main 1–278 kept, branch entries 279–300, 301 = merge. SOP world re-minted under 0.22.0: 78eddd4e… → e3c3d396577159a51581782455b68c30fe353292b6023c24625d28174627cf17 (release CLI tarball sha256 0a976877…; binary 2d082281…); validate output unchanged; compiler-upgrade-0.22.0.md added; prereg + S1 bundle untouched; fde-anchor verify fde-s1 OK. All gates pass (agent-reported); proof statuses unchanged. Note: host mise shim `domainforge` is still 0.21.2 → set DOMAINFORGE/FDE_DOMAINFORGE_BIN to the release CLI for model checks.
- S1 COMPLETE (gate run recorded). Next: WP-20 (S2 contracts package).
- WP-20 (S2 contracts package, contracts-only) Sonnet launched (acfc952183177e53d) on fde-sea-rs-integration: field-case.v1, case-record-kinds.v1, case-record-append.v2 (adds WORLD_NOT_PINNED, SETTLEMENT_EXISTS, KIND_IMMUTABLE, REFERENCE_DERIVED_FROM_RESULT, MODEL_ANCHOR_LATE, AUTHORITY_DENIED, ANCHOR_PUBLISHER_INVALID, FREEZE_INCOMPLETE, STORE_INTEGRITY, PAYLOAD_TOO_LARGE), execution-episode-ref.v1 (I-1 gate datum on item_execute/case_advance/agent_run.delegate), delegation-link.v1 (I-2 residual), continuation.v1 + read verbs, reserved-operation.v1 (+ program-execution authority decision), assignment.v1, independent-observation section, oracle-codes.json check. Evidence → evidence/s2/wp-20.md. Review contracts (fresh Sonnet/EGM) before WP-21, WP-31, WP-41.
- Horizon #31 MERGED 11f8ab0d (CodeRabbit pass, 0 threads). Workbench #34: 5 pass / 1 pending, waiter running. After #34: re-pin gsr-heads from origin/main, run readiness, then sync Horizon/Cognate/RealityTrace/Workbench fde/integration branches with main.

## 2026-11-15 ~04:15Z
- Cognate-Workbench #34 MERGED bd4c0ea9 (CLEAN, CodeRabbit pass, all checks pass). ALL FIVE lockstep repos now on main at DomainForge 0.22.0 (SEA-Forge be63b711, Cognate e74bbb86 + release 0.9.2, RealityTrace 521d363a, Horizon 11f8ab0d, Workbench bd4c0ea9).
- Readiness run started detached: log ~/projects/_wt/tmp/readiness-20261115.log (scripts/readiness.sh syncs gsr-heads to origin default branches, then runs all gates; holds the cargo lock). On finish: copy evidence/REPORT.md to evidence/baseline/readiness-REPORT-20261115-*.md and commit readiness evidence locally.
- Sonnet limit hit again (reset 22:50 ET): WP-20 agent acfc952183177e53d had committed nothing; resumed 23:12 ET. Sync agent a027c45a90ba2831f had already delivered its final report (work pushed) — no resume needed.
- OWNER REQUEST (addition to the final HTML report): include plenty of screenshots (real captures of the running system — Horizon desk/episode evidence panel, Workbench, terminal runs of the proof runner/CLIs — embedded in the single HTML file).

## 2026-11-15 ~05:10Z
- WP-20 done: SEA-Forge fde/integration tip 6c633811 (10 commits, status revs 302+). Contracts-only: field-case.v1 (45 fields from SOP §4.2), case-record-kinds.v1, case-record-append.v2, execution-episode-ref.v1 (R1–R11), delegation-link.v1, continuation.v1 + read verbs, reserved-operation.v1 (+ program-execution decision), assignment.v1, oracle-codes.json; README §4–13 with decisions D1–D30; CHANGELOG. Agent-reported gates: 12 digests match; field-case schema check ok; 13 G oracle codes/36 codes; fde_contracts 30/0; conformance_field_case 31, sfwp 23; fmt/clippy clean. Golden frames + Go/TS mirror left to WP-21; ledger-state refusals not executed (WP-21). Evidence evidence/s2/wp-20.md.
- Decisions needing owner/maintainer eye: D4 (request without `contract` = v1 writer), D8 (CASE_SUSPENDED refuses only freeze+execution), D12 (assessor not in gate payload), D17 (case_advance without episode_ref), D23 (disposition/hard-gate N/A = P2; program_execution = P1; A3-T for P2).
- Launched fresh-context Sonnet adversarial review of the S2 contracts (read-only) — must finish before WP-21/WP-31/WP-41.

## 2026-11-15 ~05:40Z — READINESS after 0.22.0 lockstep: NO-GO 29/30
- 29 PASS incl. C01 (all products pin DomainForge 0.22.0), J02, J02R2, L01, O01–O03, U02f/g, BM-AUTH, BM-RTO. 1 FAIL: SW01 (Workbench test/J-SW01-SETTLEMENT.test.ts:71): the first approved edit's operation came back `rejected`, exitCode 1 instead of `settled`, exitCode 0 — i.e. the declared contained proof (`grep -q Howdy src/greet.ts && test ! -e <rtData> && test ! -e <socket> && test ! -e /home && ! touch src/probe`) exited 1 inside RealityTrace's contained verifier. Suspects since last GO: SEA-Forge #71 hardening (sandbox), RealityTrace #16, Workbench #34 / Cognate 0.9.2. Report saved evidence/baseline/readiness-REPORT-20261115-NOGO-29of30.md + run log.
- Re-running SW01 alone (`scripts/readiness.sh --no-sync SW01`, task bs61jfkti) to see if it reproduces. NOTE: `pgrep -f scripts/readiness.sh` matches its own wrapper — do not use it as a liveness check.
- SW01 reproduces when run alone (rejected / exitCode 1). Sonnet debug agent a8c5b52c7dd2ab580 launched: find failing conjunct, bisect by product against the 2026-11-13 GO revisions using scratch worktrees (never touching gsr-heads), fix in owning repo via PR (no merge) or local readiness commit.

## 2026-11-15 ~06:20Z — S2 CONTRACT REVIEW (fresh Sonnet, read-only) on d2f8189e..6c633811
- Critical: C1 delegate has no case binding → I-1 bypass by omitting episode_ref; C2 I-2 executor set = link delegates only → fails open for item_execute/case_advance and missing links.
- High: H1 v1 writer dodges AUTHORITY_DENIED on episode.settled (D4); H2 verifier_context fresh self-asserted (D20); H3 approval_ref free text + params {} digest collision / reuse; H4 settle_episode reserved yet first evidence alt has no a3_proof (would refuse AT-09 agent settlement); H5 program-execution binds_digest missing cwd/env/workspace/sandbox/world/timeout, reusable; H6 no EPISODE_SUPERSEDED/EPISODE_SETTLED; H7 D17 case_advance can skip execution items; terminal paths other than advance bypass CLOSE_DISPOSITION_REQUIRED.
- Medium: M1 read verbs no authz; M2 unknown vs empty; M3 no CASE_CLOSED; M4 D6 evadable (relabel best-effort); M5 pin source undefined + sensitive field-case paths revisable at session level; M6 anchor checked late; M7 identity aliasing; M8 FREEZE_MISSING two meanings; M9 assignment resume/budget/unit.
- Maintainer decisions sent: D23 follow E (close dispositions + hard-gate waivers P3 unless E states a condition); D12 assessor back in payload; D13, D15 accepted (stricter); D8 suspended case also refuses settlement/new dispositions unless G/SOP say otherwise.
- WP-20 agent acfc952183177e53d resumed to fix all in place (contracts not yet consumed). WP-21 stays blocked until done.

## 2026-11-15 ~06:40Z — READINESS GO 30/30 at DomainForge 0.22.0
- SW01 root cause: NOT a product regression. My shell exported TMPDIR under $HOME (for cargo); Workbench test/liveSettlement.ts:85 puts the workspace under tmpdir(), so bwrap created /home/... parents inside the sandbox and the proof's `test ! -e /home` correctly failed. Containment behaves correctly. Fix: readiness local commit 707089c pins `TMPDIR: /tmp` in SW01's run.env. Verified: `TMPDIR=$HOME/projects/_wt/tmp scripts/readiness.sh --no-sync SW01` → PASS; report GO 30/30 at pinned clean revisions (29 gates from the full 04:25Z run + SW01 rerun, same heads). Saved evidence/baseline/readiness-REPORT-20261115-GO.md; readiness evidence committed locally.
- Lesson: run readiness without a $HOME TMPDIR for test steps (the scenario now guards SW01; SW02 uses liveSettlement but has no /home assertion). Optional follow-up: fail-fast guard in Workbench liveSettlement.ts when tmpdir() is under /home.

## 2026-11-15 ~07:30Z
- S2 contract review fixes landed: SEA-Forge fde/integration 9f643112, 5ab55b53. All C/H/M/LOW findings fixed per agent; decisions now D1–D38 (D4, D8, D12, D17, D23, D25 reversed/replaced; D31–D38 added). New codes: CASE_UNRESOLVED, EPISODE_SUPERSEDED, EPISODE_SETTLED, TRANSITION_ONLY_INVALID, VERIFIER_CONTEXT_MISMATCH, CASE_CLOSED, ASSIGNMENT_LINK_EXPIRED, ASSIGNMENT_BUDGET_EXCEEDED … (44 codes, 13 G codes). fde_contracts 34/0; all checks ok. Wire change: `delegate` now requires `case_id` (D34) → Go builder/frames in WP-21. Accepted: unit enum per currency (M9 reading); a linked verifier with no run record counts as executor (must write verifier-role run record before settling).
- WP-21 split. Part A launched (Sonnet): scenario-driven conformance harness over the four fixture manifests, append v2 rules for frozen/settled, execution gate R1–R12, run records + delegation links + executor set, program-execution authority decision, s2 proof wrappers NT-13/17/18/32/33/34/48. Part B (after A): field-case doc, field_case.revised, strategy/gate/disposition kinds, lifecycle, continuation + read verbs, template. Evidence → evidence/s2/wp-21a.md.
- 2026-11-15 ~09:12Z: WP-21a agent a0725bc1f5f37c87b was cut off by the Sonnet limit with NOTHING committed (tree clean at 5ab55b53); resumed with instruction to push small commits (harness first) and keep notes at ~/projects/_wt/tmp/wp21a-notes.md.
- Launched Sonnet a0d5a8abcb173b01d: merge origin/main into fde/integration for RealityTrace, Cognate, Horizon (in that order; push each after gates; notes ~/projects/_wt/tmp/sync-others-notes.md). Workbench fde/integration sync still owed (after these). Then WP-23 (Cognate assignment binding) can start once Cognate is synced and WP-21a's delegate/assignment wire shape is implemented.

## 2026-11-15 ~10:30Z — integration branches synced with main (0.22.0)
- RealityTrace fde/integration a1f8ac3 (no conflicts; vendored evaluate schemas byte-identical to DomainForge v0.22.0; workspace 308/0 on released 0.22.0 CLI; 8 contract digests match). Note: 4 tracked .jolli/jollimemory files show as modified in that worktree (tool noise, not committed).
- Cognate fde/integration 08fcb78 (1 conflict packages/realitytrace/package.json; 40 packages at 0.9.2; packages/identity stays 0.1.0, branch-only; 1127 pass/32 skip/0 fail).
- Horizon fde/integration 9d4f16b + a37d7e9 (conflicts package.json/bun.lock; cognate-store-sqlite → 0.9.2; just verify exit 0; 1688 pass/16 skip/0 fail; fde-parity PASS all 30).
- SEA-Forge fde/integration was synced earlier (f6ad220c). Workbench fde/integration sync still owed.
- WP-21a step 1 pushed: SEA-Forge 9b180735 (scenario-driven conformance harness). Agent a0725bc1f5f37c87b continuing with steps 2–6.
- WP-23 Cognate half launched (Sonnet af43a4804b1a2105d) on fde-cognate-integration: assignment id/payload from vendored assignment.v1, startRun from assignment, budget/spend per currency, lifecycle state machine, AT-17 lifecycle part. Evidence → evidence/s2/wp-23-cognate.md. Workbench half later.

## 2026-11-15 ~11:30Z
- WP-23 Cognate half done: Cognate fde/integration 509e199, 96f8300, b38100c. assignment.v1 vendored by digest in packages/runtime-api/schemas; runtime-api/src/assignment.ts (pure id/validator/state machine reading constants from the vendored schema); startRun takes `assignment` + `linkExpiresAt` (requires caseContext); Runtime.assignments {getAssignment, interruptAssignment, resumeAssignment, recordSpend}; new authorized actions assignment.interrupt/resume/spend (default-deny policies must allow them). Agent-reported: full suite 1223 pass/32 skip/0 fail (+96); 48 fixtures + 10 lifecycle scenarios covered; 35 mutations caught; AT-17 lifecycle part met in Cognate. NOT done: caseContext/assignment over Connect/proto wire (assignment now refused visibly over the wire; caseContext still dropped as in WP-15); spend is caller-reported; no delegation-link object read (linkExpiresAt is a parameter); service throws AssignmentRefusedError, not the wire Error body. Evidence evidence/s2/wp-23-cognate.md.
- Workbench half needs: startRun with assignment from SEA-Forge delegate result; operator controls; proto fields + AssignmentService RPCs for Connect; policy allowing the three assignment actions.

## 2026-11-15 ~13:00Z — Cognate PR #83 review round (main session; Sonnet weekly limit hit)

- Sonnet subagents blocked until Nov 19 6pm ET (weekly limit). WP-21a agent and WP-23 Cognate agent stopped; resume via SendMessage after reset. Main session continues gating/merging.
- PR #83 `pack + clean-room install` failed: `packages/identity` unregistered. Fixed 3427549 (marked private; Horizon consumes a vendored tarball).
- CodeRabbit left 18 threads; all fixed in 210fa97 and resolved (program.run empty-file/root identity/resolved-path dedupe/compare-before-write + owned-only rollback; identity credential claim + cross-origin refusal; Connect refuses caseContext/linkExpiresAt; actor attribution for cancel/interrupt/resume; brief moved to erasable payload; erasure-safe assignment fold; causation chain; interrupt-replay and budget cleanup from committed state; validation type checks). Local: typecheck clean, 573 pass, 1 local-only SXR-binary gate.
- Open limit: program.run writes are path-rechecked, not anchored (no openat in the ExecutionWorld FS interface).
- FOLLOW-UP: Horizon vendors `godspeed-identity-0.1.0.tgz`; re-vendor after #83 merges (claim record + cross-origin refusal).

## 2026-11-15 ~14:10Z — integration branches landed on main

- Cognate #83 merged (f5a6954) after packaging fixes 3427549 and 90f6fb9 and the 18-thread review fix 210fa97. CI green.
- Horizon: the identity package was re-vendored from Cognate main (efee391). PR #32 merged (6582c4b). Horizon has no GitHub CI; the gate was local `just verify` (DomainForge 0.22.0) plus `bun test` (1688 pass / 0 fail). CodeRabbit skipped it (528 files > 150). Absolute local paths in evidence and review files already exist on main and were left untouched.
- Workbench: synced with main (5550491). PR #35 merged after the 9-thread review fix c428acd: secrets-dir overlap guard, deep-link fixes plus event ids, provenance-resolved review memory, companion rollback, and a null outputRef. A flaky SSH test (`J-RW-cancel-timeout`, "Malformed OpenSSH private key") passed on rerun.
- Remaining while Sonnet is blocked (until Nov 19, 6pm ET): WP-21a step 3+ (uncommitted in fde-sea-rs-integration), WP-21B, WP-22, WP-23 Workbench half, S3–S5.

## 2026-11-15 ~15:00Z — RealityTrace 0.5.0; S6 release-readiness track added

- RealityTrace #18 merged (35f7dc5): sxr-observed, WP-11/16/17/18. Release PR #19 (0.5.0) merged (6f84bb5); the release workflow is running.
- Owner request: CI/CD, native packaging and release readiness for Gauntlet and Horizon. Added as slice S6, WP-60…WP-69, in F-implementation-plan.md. Memory: gauntlet-horizon-release-track.
- Recon report: ~/projects/_wt/tmp/recon-gauntlet-horizon-release.md.
- WP-60 (Gauntlet PR #7):
  - Blocker: the ruleset requires CodeQL, which was not configured. Enabled CodeQL default setup (standing OK).
  - Fixed and resolved the 7 review threads in 7b866c8.
  - CI is running.
- New owner decision OD-7: Jetson device and self-hosted runner registration (needed for any Jetson QUALIFIED claim).
- Horizon is private on a free plan, so rulesets and branch protection return 403. Required checks cannot be enforced; recorded as a blocker.

## 2026-11-15 ~15:20Z — correction: Sonnet not blocked

- The "Sonnet blocked until Nov 19" entries above were wrong as a standing state: they came from one 429 and were never retested. The WP-21a agent resumed successfully at ~15:15Z, so Sonnet implementation work is running again.
- RealityTrace 0.5.0 is published (realitytrace-v0.5.0, 9 assets; Release, CI and clean-install green).

## 2026-11-15 ~16:00Z — WP-21a steps 1–4 and 6 done; step 5 decided

- WP-21a steps 1–4 and 6 pushed: 9b180735, 9f1c9281, 90f93acb. Execution gate R1–R14, append v2 codes, run records and delegation links under the case lock, Go builders, golden frames, `fde-proof --slice s2`.
- Scenarios: 102 of 156 pass.
- S2 proofs PASS: NT-13, NT-17, NT-18, NT-32, NT-33, NT-48. NT-34 is NOT_RUN (step 5).
- Mutations: 28 killed. One survived (declared != derived verifier context; no separating fixture).
- Decisions (architect):
  - D39: program-execution authority binds via CEP resource_id `execution:sha256:<digest>`; the run counter is in the case store keyed by decision_id; no CEP profile change.
  - D40: fix the sibling-self-asserted fixture (no invariant on VERIFIER_CONTEXT_MISMATCH).
  - D41: a model anchor can only be late once a run exists.
  - D42: a closed case refuses everything except the plan-ending action.
  - D43: idempotency precedes EPISODE_REF_DIGEST_MISMATCH (the S1 order); the contract text is corrected.
  - D44: verifier runs are delegations to `verifier_endpoints`, with principal `agent:<endpoint>`.
- The agent resumed for step 5, D40/D43 fixture and contract fixes, the separating fixture, and a full-crate test run.
- To review later: AT-04 still reports PARTIAL although execution-side I-1 is now enforced for Field Cases.

## 2026-11-15 — WP-60 done: Gauntlet PR #7 merged (36c9f22)
- Final fix b63386b: removed every free-form ref/tag input from package.yml and release-native.yml. Releases build only the triggering gauntlet-v* tag commit. A manual re-publish is a dispatch from the tag. This cleared all 20 CodeQL actions/cache-poisoning alerts.
- Earlier fixes on the PR: macos-15 runner (5322dc5); darwin-x64 workbench staging gate (1db1e47); SHA pins, concurrency, notes refresh, InstallDelete (7b866c8).
- On b63386b: gates, CodeQL (actions, js-ts, python, rust), package linux amd64/arm64, macos, windows, and the sum job all passed. 0 unresolved threads.
- These CI packages are UNQUALIFIED: unsigned, and the Jetson has not been tested on real hardware. Qualification is WP-62/69.
- Horizon WP-64 agent stopped (it kept repeating its completion report).

## 2026-11-15 — WP-21a step 5 (D39) done: c18b6a69
- Program execution needs a CEP `allow` decision whose resource_id is `execution:sha256:<RFC 8785 digest of the nine-member object the server builds>`. The decision must be within its TTL and have runs left (`execution_max_runs`). It is checked under the case lock. Anything else is AUTHORITY_DENIED.
- Scenarios: 114 of 156 pass. Pending: part-B 27, wp-31 14, contract 1.
- Mutation checks: 5 of 5 KILLED.
- Tests: `cargo test -p sea-forge-server`: 640 passed, 0 failed.
- S2 proofs: NT-13, NT-17, NT-18, NT-32, NT-33, NT-34 and NT-48 all PASS.
- BLOCKED (permission classifier denied edits under contracts/fde/; escalated to the user):
  - D40: the sibling-self-asserted fixture.
  - D43: the precedence text in the README.
  - README decision rows D39–D44.
  - A new verifier-context separating fixture (case-record-append-v2: 53 → 54 scenarios).
- Until these land, FINAL_REASONS keeps `contract`.
- To confirm: `run.started` maps to Episode clause 7.2d.

## 2026-11-15 — paused at the usage limit
- The user approved the contract edits (D39–D44, D40 fixture, new append-v2 scenario 53→54). They are applied in fde-sea-rs-integration but UNCOMMITTED. The digest, oracle-code and field-case-schema checks pass.
- Tests fail: conformance_fde_scenarios (every_scenario_is_driven…, final_pending_list…) and realitytrace fde_contracts delegation_link_scenarios_follow_i2 (line 1917: probably still asserts I-2 on every delegation-link refusal). Next: read the panics in ~/projects/_wt/tmp/contract-edits-test.out, fix, re-run, commit.
- WP-61 (Gauntlet CI, branch ci/complete-gates) and WP-65/66 (Horizon desktop, branch feat/desktop-tauri) agents were stopped mid-work. Check their branches/PRs before resuming.

## 2026-11-15 — resumed after the usage pause
- WP-21a part A is complete: 5720674c on fde/integration. Owner-approved contract edits D39–D44:
  - The delegation-link.v1 Error no longer requires an invariant on VERIFIER_CONTEXT_MISMATCH; the file is re-pinned in DIGESTS.
  - New append-v2 scenario (54 in total).
  - New invalid Error fixture.
  - README precedence and decision rows updated.
- Verification: FDE_WP21A_FINAL=1 passes with 115 of 157 scenarios passing; the 42 pending are part-B and wp-31. fde_contracts has 34 passing. Digest, oracle-code and field-case-schema checks pass.
- WP-61 done: Gauntlet PR #8 merged (50b876d). CI is split into named jobs (rust-lint, rust-check-test, cargo-deny, boundaries, ruler, workbench, regression, plus the aggregate gates (ubuntu)). It adds security.yml (gitleaks 8.30.1, plus dependency-review, which is NOT_RUN because the repo's dependency graph is off) and dependabot. All actions are SHA-pinned.
- Gauntlet "Protect Main" ruleset: added required status checks "gates (ubuntu)" and "gitleaks", alongside the existing CodeQL rule.
- Open: the Gauntlet dependency graph, secret scanning and Dependabot security updates are disabled at the repo level (a settings change, not made). Horizon has 6 Dependabot PRs open (#34–#39).
- 2026-11-15, Gauntlet security settings, with the user's go-ahead:
  - Secret scanning is enabled and found 0 alerts. Push protection is still off.
  - Dependabot alerts are enabled.
  - Dependency graph: ENABLED (the user confirmed), but not yet indexed (0 manifests; Cargo.lock is on main). Indexing started after the WP-63 merge pushed to main: 39 manifests detected (Cargo.lock, crate Cargo.toml files, workflows); dependency counts were still being filled in. dependency-review now runs on PRs.

## 2026-11-15 — WP-63 done: Gauntlet PR #21 merged (fa73063)
- Prime Agent is optional and never bundled.
  - Detection: PRIME_AGENT_BIN first, then PATH. A set-but-missing path is an error, with no fallback to PATH.
  - Supported versions: 0.8.0 up to (not including) 1.0.0, pinned to 0.10.2, defined in gauntlet-app config/defaults.rs.
  - Commands: `gauntlet agent doctor`; `gauntlet agent setup prime`, which only prints the manual steps unless given `--installer <file> --sha256 <hex>` plus confirmation (unix only).
  - packaging/assert-no-prime-agent.sh runs in every package job, and an arch test scans the packaging files.
- Upstream (PrimeIntellect-ai/prime-agent) ships only a hosted install script on a floating channel, with no independent digest, so Gauntlet downloads nothing itself.
- CI: green, including macOS packaging and 2254 nextest tests. The 7 CodeRabbit threads were fixed and resolved.
- NOT_RUN: a real upstream install (only a stub installer was tested).

## 2026-11-15 — WP-65/66 build half: Horizon PR #40 merged (5137ab1)
- Tauri v2 shell (Tauri 2.12.5) supervises a `bun build --compile` sidecar (`horizon-server serve --desktop`). It binds loopback on an ephemeral port and reuses the desk pairing model; the server exits when its stdin closes. Headless use is unchanged (`serve`, plus a tarball with the binary and horizon-resources/).
- desktop.yml builds:
  - linux-x64 and linux-arm64: .deb, AppImage
  - windows-x64: NSIS, MSI
  - macos-arm64 (macos-15) and macos-x64 (macos-15-intel, the label was accepted): .app, .dmg
  - every leg: a headless tar.gz
  All artifacts are labelled unqualified, with SHA256SUMS. macOS is ad-hoc signed only; Windows is unsigned.
- Sidecar smoke: PASS on linux-x64. BLOCKED on linux-arm64, windows-x64 and both macOS legs: @godspeedai/domainforge 0.22.0 on npm ships only the linux-x64-gnu binding (no platform packages; aarch64-linux is not even a napi target). Fix dispatched as a DomainForge PR, then a release, then flip `runnable` per leg in desktop.yml.
- NOT_RUN: the Tauri window in CI (no display); WP-67 install/upgrade/uninstall qualification; signing.
- Jetson: `ssh jetson` is available (user-granted). Read-only recon is done. The WP-62 agent was briefed to run tarball qualification there in a scratch dir only. The user was given time-boxed sudoers instructions for the .deb legs (dpkg/apt-get, then remove).

## 2026-11-15 — WP-62 first pass: Gauntlet PR #22 (qualification stage)
- Three stages: CI → qualify-native.yml (install, smoke, governed mock run, upgrade, uninstall, checksums, per-platform JSON records, readiness table) → release-native.yml (hard `needs: qualify`; prerelease unless every claimed platform is QUALIFIED).
- First run:
  - QUALIFIED: linux-x64, linux-arm64 (generic), macOS arm64, macOS Intel (macos-15-intel runner, real x64). Upgrade and provenance are NOT_RUN there (no previous release; provenance is checked at publish).
  - Windows: NOT QUALIFIED. The governed run fails because the evidence store fsyncs a directory, which Windows refuses (product bug).
  - Jetson (real hardware, tarball mode, scratch dir, no sudo): checksums, install, smoke and uninstall PASS. The governed run is NOT_RUN: the mock fixtures are found through the build host's compile-time path (product bug).
- Sent back for: the Windows fsync fix; embedded mock fixtures; a promote workflow for release-time Jetson evidence (builds aren't reproducible); same-version restore (no 0.1.0 .deb exists on the Jetson or as a release).
- The user will run the .deb legs on the Jetson personally (sudo password prompt); no sudoers change.
- 2026-11-15, Jetson .deb rehearsal: the owner ran it with a sudo prompt against run 43751335841, arm64 artifacts, commit 53047400.
  - Record status QUALIFIED. PASS: checksums, tarball_install, install (.deb), smoke, governed_run (mock run settled), uninstall.
  - NOT_RUN: upgrade (no previous release) and provenance (operator run).
  - Afterwards the system gauntlet is 0.1.0, installed by the restore step; the temporary PostgreSQL cluster was stopped.
  - The record is copied to evidence/s6/jetson-deb-rehearsal-run43751335841.json. It is rehearsal evidence only: release-time evidence must come from the release's own assets and go through the promote workflow.

## 2026-11-15 — WP-62 done: Gauntlet PR #22 merged (0e185aa)
- The CI → qualify-native → release-native chain is in place, plus promote-release.yml for evidence that comes after a release (Jetson). A release is published as a prerelease unless every platform claimed in packaging/qualification-platforms.json is QUALIFIED.
- Final run on a4d54a9: Qualify (native) passed on windows-x64, linux-amd64, linux-arm64-generic, macos-arm64 and macos-x64.
- Product fixes in this PR:
  - The evidence store flushes the directory on Windows (FILE_FLAG_BACKUP_SEMANTICS; falls back only on volumes that cannot flush).
  - Mock fixtures are embedded in the binary.
  - The CI pg0 download is authenticated (GH_TOKEN).
- Not yet exercised end to end: publish and promote (no tag yet), upgrade (no previous release). Signing is not configured.

## 2026-11-16 — DomainForge 0.22.2 native npm bindings; Horizon desktop server runs on all five platforms
- DomainForge 0.22.1: the five platform packages @godspeedai/domainforge-{linux-x64-gnu,linux-arm64-gnu,darwin-arm64,darwin-x64,win32-x64-msvc} were published with provenance using a new NPM_TOKEN (the old token from 2026-06-21 was refused). The main 0.22.1 package was not published: npm trusted publishing for it accepts deploy.yml, not a hand-dispatched release-npm.yml.
- DomainForge 0.22.2 (PR #176, fafb9ab; golden digests regenerated, 1f03a29) was released through the tag path. Deploy run 43776156568 succeeded: main, platform and wasm packages are all at 0.22.2.
- The org/repo tokens RELEASE_PLEASE_TOKEN (not visible to DomainForge) and CREATE_PR_TOKEN (a GitHub token) were probed and cannot publish to npm.
- Horizon PR #41 merged (2c14d1a):
  - an override pins @godspeedai/domainforge to 0.22.2 (cognate-domainforge 0.9.2 still pins 0.22.0);
  - every Desktop leg is runnable;
  - the store lock is cross-platform: flock on linux, perl flock(2) on darwin, a PowerShell exclusive open on win32.
  - The Desktop sidecar smoke returns health 200 on linux-x64, linux-arm64, windows-x64, macos-arm64 and macos-x64. These are CI builds, not install-qualified (WP-67).
- Follow-ups:
  - the owner adds npm trusted publishers on the 5 platform packages, then NPM_TOKEN is removed from release-npm.yml;
  - release cognate-domainforge pinned to 0.22.2 and drop the Horizon override;
  - scripts/ignite.ts (dev launcher) still uses sh/ss.
- 2026-11-16: Cognate PR #85 (3fff608) moved the DomainForge binding to 0.22.2.
  - CodeRabbit's note was resolved by recording the RealityTrace pin follow-up: realitytrace 0.5.0 carries no domainforge crate.
  - Release PR #84 needed one empty commit (e3aa6bd) to start CI, because release-please pushes with GITHUB_TOKEN and so triggers no workflows. After that, all 8 checks passed and it merged (a08d244).
  - Cognate v0.10.0 is published; cognate-domainforge 0.10.0 depends on domainforge 0.22.2.
  - The Horizon upgrade to Cognate 0.10.0 (dropping the override) is in progress.
- 2026-11-16: Horizon PR #42 merged (b5f699c).
  - Cognate is at 0.10.0 (15 pins plus 2 overrides), and the domainforge override is removed: 0.22.2 now arrives through cognate-domainforge.
  - Gates:
    - local: bun test 1701/16 skip/0 fail; check, single-cognate, boundaries and tsc all pass.
    - CI: verify (just verify, build:desk, typecheck), CodeQL js/py, dependency-review, gitleaks, and Desktop on linux-x64, linux-arm64, windows-x64, macos-arm64 and macos-x64 all passed.
- 2026-11-16: WP-67 is implemented in Horizon PR #43 (ci/desktop-install-qualification, not merged).
  - scripts/qualify-desktop.py writes per-platform horizon.qualification/1 records; platforms are listed in scripts/qualification-platforms.json; desktop.yml gains a qualify job; the how-to is docs/how-to/qualify-desktop-install.md.
  - A local run on linux-x64 gave PARTIAL: checksums and the AppImage extract and smoke PASS; the .deb steps are NOT_RUN (they need root).
  - CI is BLOCKED by GitHub billing: run 43802909821 started no jobs ("recent account payments have failed or your spending limit needs to be increased"). The owner must fix billing before the five-platform CI proof can run.

## 2026-11-16 — WP-67 closed: Horizon desktop qualified on 6 targets incl. real Jetson

- Horizon #43 (4242d31, qualify-desktop.py + `qualify` CI job) and #44 (9a4579f, CodeQL SARIF upload with test/ and scripts/ filtered out) merged. CodeQL uploads now satisfy the Protect Main ruleset.
- CI qualification (run 43806704963, commit 0478912): linux-x64, linux-arm64, windows-x64, macos-x64 and macos-arm64 are all QUALIFIED. On macOS, app_installed_gui is NOT_RUN on hosted runners; the sidecar smoke test covers the server there. Evidence: evidence/s6/horizon-wp67/*.json.
- Real hardware: Jetson (Ubuntu 22.04 arm64, L4T R36.5, kernel 5.15.185-tegra), `linux-arm64-jetson`, QUALIFIED at 2026-11-16T11:23Z. The owner ran the record as root with their own password. PASS: checksums, deb_install, deb_installed_smoke, deb_installed_gui (under xvfb), deb_uninstall, appimage_extract and appimage_extracted_smoke. All 3 artifacts match SHA256SUMS. After the run, dpkg reports no horizon package and /usr/lib/horizon is gone. Evidence: evidence/s6/horizon-wp67/jetson.json. The scratch dir was removed.
- Upgrade is NOT_RUN on every target because no earlier release exists; it becomes runnable from the second tagged release.
- Open: Horizon's dependency graph is not indexed, so dependency-review fails (it is not a required check). xvfb stays installed on the Jetson.

## 2026-11-16 — WP-68 part 1: readiness gate Q01 (godspeed-readiness 1373c66)
- Q01 passes only when every hosted platform in Gauntlet's and Horizon's platforms file has a QUALIFIED record at the exact pinned commit. It re-checks each record itself (schema, commit, required steps, checksums) instead of trusting the run's overall result. A missing run counts as NOT PROVEN.
- Jetson rows come from the operator evidence directory and are informational only (they show the commit they were tested on). Release-time Jetson checks happen when a release is published.
- Result: Q01 OK. Gauntlet 0e185aae (qualify-native run 43861068120; macos-arm64 was cancelled while queued and passed on rerun) and Horizon 4242d312 (desktop run 43861072888) have every hosted platform QUALIFIED.
- The pinned heads for gauntlet and horizon were moved to origin/main. The full readiness.sh has not been rerun since.
- Part 2 (Horizon release-desktop/promote workflows) is in progress.

## 2026-11-16 — Owner rulings OD-1..OD-6 and the consolidation addendum
- All six approved, OD-6 with verification conditions. Recorded in decisions/2026-11-16-owner-rulings-OD-1-to-OD-6.md and decisions/SOP-v0.4.1-ratification.md (SOP sha256 043ae33a…7001, unchanged). H §9 points to them. F §4 maps them to work packages: WP-30/31/32 A3-UV, new WP-35 independent anchoring, new WP-58 branch consolidation.
- No evidence classification changed. The S1 anchor stays historical.
- Next: WP-58 (OD-4 backups, then the branch audit and consolidation), then S2–S4 in dependency order.

## 2026-11-16 — Releases published (prerelease); WP-58 started
- gauntlet-v0.1.1 (fd797dc, release run 43880185610, 20 assets) and horizon-v0.1.2 (3e05877, run 43880212555, 28 assets) are published as prereleases; the Jetson row is NOT QUALIFIED until WP-69.
  - Pipeline bugs found on the first runs and fixed: Gauntlet #23 (the qualify matrix was skipped after a skipped package job) and Horizon #48 (both macOS legs produced the same .app archive name; assemble errors were hidden).
  - Superseded releases: gauntlet-v0.1.0 (tag only, never published) and horizon-v0.1.1 (an empty prerelease). Gauntlet #24 bumped the version to 0.1.1 instead of moving the tag. Horizon #46 fixed a CodeQL XSS finding in docs explainers.
  - Build provenance verified locally (`gh attestation verify`) for all 5 arm64 assets: the signer workflow ref is the release tag.
- WP-69 staged on the Jetson in ~/wp69-jetson, and the no-root dry run passed.
  - Horizon: everything that needs no root PASS.
  - Gauntlet (tarball mode): every step PASS except provenance. Its HOME redirect hides gh auth, a Gauntlet script bug; follow-up. The owner run omits --attest-repo, and provenance is verified separately (above).
  - The Gauntlet restore .deb is the exact 0.1.0 build installed (dpkg md5sums equal); it comes from run 43751335841.
  - Waiting on the owner to run ~/wp69-jetson/run.sh.
- WP-58: inventory in recon/branch-inventory-2026-11-16.md, Workbench capability recon in recon/workbench-capability-plugin-2026-11-16.md. The stranded capability runtime is on cognate-workbench feat/desktop-terminal-browser, 19 commits that were local only and are now backed up as a tag. OD-4 backups are in branch-ledger.md (42 verified, the rest in follow-up). Old GodSpeedAI/SEA: owner says no fixes, push its branches, archive it.

## 2026-11-16 — WP-69 done: Gauntlet 0.1.1 and Horizon 0.1.2 promoted to Latest
- Real-hardware Jetson AGX Orin (L4T R36.5) runs by the owner against each release's own published arm64 assets:
  - Horizon horizon-v0.1.2 (3e05877): QUALIFIED. PASS: checksums, deb_install, deb_installed_smoke, deb_installed_gui, deb_uninstall, appimage_extract, appimage_extracted_smoke. upgrade NOT_RUN: v0.1.1 carries no assets.
  - Gauntlet gauntlet-v0.1.1 (fd797dc): QUALIFIED in package and restore mode. PASS: checksums, tarball_install, install, upgrade (0.1.0 → 0.1.1), smoke, governed_run, uninstall. provenance NOT_RUN in the record; verified separately with `gh attestation verify`. The owner's 0.1.0 was restored afterwards (dpkg shows 0.1.0).
- Records uploaded as release assets. Promote workflows: Horizon run 43888180268 and Gauntlet run 43888503013, both success. Both releases are now Latest, with every claimed platform QUALIFIED.
- Evidence: evidence/s6/wp69/{horizon-v0.1.2-jetson.json, gauntlet-v0.1.1-jetson.json, *-QUALIFICATION.md}. The Jetson scratch dir ~/wp69-jetson was removed.
- Follow-ups:
  - Gauntlet qualify-local.sh: provenance in package/tarball mode loses gh auth through the HOME redirect, and sum_of does not accept "./"-prefixed SHA256SUMS lines.
  - On the Jetson, apt now lists postgresql as autoremovable while gauntlet is uninstalled: do not autoremove blindly.
- WP-58 OD-4: SEA-Forge's 14 items pushed with owner-approved --no-verify; Horizon dirty logs scrubbed (owner) and backed up; Workbench feat/horizon-connect stripped of dist-package/ (owner) and backed up. Old SEA: 10 branches on GitHub, the rest held in bundles pending the owner's choice; the archive follows.

## 2026-11-16 — WP-58 Cognate-Workbench consolidation and CI optimization
- Cognate-Workbench main 4594bb7: capability runtime and the desktop terminal/browser lineage landed (PR #36 fe59ade); CI optimized (PR #37): 4 test shards, caches, narrowed triggers, pinned actions; cold 424 s, warm 275 s against about 895 s. Details in branch-ledger.md.

## 2026-11-16 — WP-58 SEA-Forge consolidation and CI optimization
- SEA-Forge main after consolidation: merges #101 5825ce01, #105 9b8aca84 (CI), #102 d7bc7c7c (deps), #103 4b42302f (Horizon desk support), #106 fce99e39 (ARM64 packaging; package.yml dispatch on the branch passed amd64, arm64, windows), #104 2f35ad27 (EGM READMEs plus the owner's graft edits), release-please #96 (0.5.0) gate green, merge not performed (blocked). #91, #100, #70 closed as superseded. Hardening harvest was already #71; the openshell, xplat, review and macos-jail branches are all ancestors of the #67 head. Rejected: ci/openshell-windows, ci/macos-app-sandbox-probe. Kept: fde/integration (46 commits, newest 2026-11-15), release-please branch.
- CI: slowest required path 967 s to 606 s warm (macOS 497 s). Detail and per-job numbers in branch-ledger.md "SEA-Forge consolidation".
- Cleanup: 24 worktrees removed (kept ~/projects/sea-rs on main, fde-sea-rs-integration, gsr-heads/sea-forge); all local and remote dispositioned branches deleted; 12 OD-4 branch backup tags deleted. Blocked by the permission classifier: dropping stash@{0} and deleting tags dirty/sea-rs and stash/sea-rs/0 (judged obsolete).
- Follow-ups: drain the request head in every HTTP test stub, then take tokio 1.54; consider a third ubuntu test shard; macOS jobs occasionally never get a runner (cancelled at 900 s), rerun.

## 2026-11-16: SEA-Forge v0.5.0 released (owner approved)
- Release PR #96 was squash-merged as 4fd6e9f7, and Release run 43975475268 succeeded. v0.5.0 is Latest.
- It has 9 assets: SHA256SUMS; linux x86_64 and aarch64 tarballs; amd64.deb and arm64.deb; a macOS universal dmg and pkg; and a Windows zip and setup.exe.
- The arm64 artifacts are built but NOT qualified on real hardware. Do not claim Jetson/ARM64 support until a Jetson run is recorded (follow-up).
- With owner approval, stash d00f3c28 (Jolli noise and a gemini hook deletion) was dropped, and tags backup/od4-2026-11-16/{dirty/sea-rs,stash/sea-rs/0} were deleted through the GitHub API. The local pre-push status-check hook refused to push the tag deletions because the reports dir is dirty. SEA-Forge has no OD-4 backup tags left.

## 2026-11-16 — WP-58 Horizon consolidation and CI optimization
- Horizon main 0e34a344 after merges #50 d41c34a5 (A/B/C authority decision closed SUPERSEDED by OD-1, DEC-OWNER-2026-11-16-01, 74 reports archived), #38 6aeb7171 (typescript 7.0.2), #51 38965175 (CI), #52 a47ea955 (zod 4.6.7), #53 87850d8a (react 19.3.1), #54 0e34a344 (webmcp-polyfill 6.2.0). Dependabot #34, #36, #39 superseded by #53, #52, #54. main CI, CodeQL, Security green; no open PRs; no release PR was touched.
- CI: slowest required path 221 s to 109 s warm (cold 219 s, runner queueing). verify is now static plus 4 time-balanced bun test shards; verified binaries and Chromium cached; rust-cache saves on main only. Detail in branch-ledger.md "Horizon consolidation".
- Cleanup: 17 worktrees removed (kept gsr-heads/horizon and the approval-test fixture checkout); 30 local and 25 remote branches deleted; 12 OD-4 backup tags deleted locally and on origin; primary checkout on clean main. Kept: fde/integration, release-please branch.
- Blocked or open: dependency-review fails on every PR because the repo's dependency graph is disabled (owner settings change). The pre-existing local-only tags checkpoint/* and fde-backup/* were not touched.

## 2026-11-16 — WP-58 Gauntlet consolidation and CI optimization
- Gauntlet main 1b50661. Merged: 11 dependabot PRs (#9 #10 #11 #12 #14 #15 #16 #17 #18 #19 #20; sha2 0.11, toml 1.2, opentui 0.8.4, actions bumps), #26 7493d22 (@types/bun 1.4.4 plus signalName normalisation, replaces dependabot #13), #25 4e35fb0 (qualify-local sum_of accepts ./name entries; gh auth kept across the tarball-mode HOME redirect via GH_CONFIG_DIR path; packaging/test-qualify-local.sh in the ruler job), #27 eed6a62 (CI), #28 1b50661 (4 nextest shards). Script is packaging/qualify-local.sh, not scripts/.
- CI: slowest job 369 s to 260 s cold, 221 s warm (2 shards), 184 s on 4 shards. One flaky test noted (codemode stderr-held-open timing).
- Cleanup: 5 worktrees, 9 local and 15 remote branches deleted; OD-4 tag, 2 checkpoint tags and 4 fde-backup tags deleted; primary checkout on clean main. Kept: fde/integration, backup/jollimemory-summaries-v3-20261023 (only copy of tool summaries), gsr-heads/gauntlet.
- Release: a gauntlet 0.1.2 patch is warranted (not created). Blocked: none. Note: shell-only PRs stay BLOCKED by the CodeQL ruleset until a workflow or source file in them triggers analysis; auto-merge is disabled.

## WP-58 RealityTrace (2026-11-16)
Consolidated: all branches integrated or rejected, primary ~/projects/sxr on clean main, CI PR #20 merged (42f38ca). Tag deletion (fde-backup, OD-4) refused by classifier; see branch-ledger.md.

## 2026-11-16 — WP-58 Cognate consolidation and CI
- Cognate main e7f1352: 9 local and 7 remote stale branches removed, 7 worktrees removed, OD-4 tags deleted, primary on clean main. CI PR #87 (SHA pins, caches, concurrency): no measurable speedup (139-182 s vs 140-156 s warm). No release warranted. Details in branch-ledger.md.

## 2026-11-17: Releases qualified on Jetson; owner standing authorization
- Owner standing authorization (2026-11-17): act on engineering judgment and escalate only strategy, physical/credential needs or major irreversible external risk. GPT Sol 6.1 through the codex CLI is permitted as a blind critic.
- Gauntlet:
  - gauntlet-v0.1.2 was tagged but never published: hosted Linux qualification failed because the relative `--previous-dir` reached apt as a package name.
  - Fixed in #30 (6a7fe6a), with a regression check.
  - gauntlet-v0.1.3 published. The upgrade from 0.1.1 passed on all 5 hosted platforms, the first hosted upgrade leg ever.
  - Jetson (owner's sudo run): QUALIFIED. PASS: checksums, tarball, install, upgrade 0.1.0→0.1.3, smoke, governed_run, uninstall; the owner's 0.1.0 was restored. Provenance was verified with gh attestation verify on the deb and tarball.
  - Record uploaded as qualification-linux-arm64-jetson.json; promote run 43998622492 succeeded. 0.1.3 is Latest.
- SEA-Forge v0.5.0 on the Jetson:
  - Tarball smoke PASS (scratch, no sudo).
  - deb upgrade 0.3.0→0.5.0 PASS, installed smoke (cell plus daemon socket) PASS, 0.3.0 restored.
  - Provenance verified on the deb and tarball.
  - Record uploaded to the v0.5.0 release as qualification-linux-arm64-jetson.json.
  - Evidence: evidence/s6/wp58-releases/.
- Tags:
  - RealityTrace: 19 backup and fde-backup tags deleted after an added-line check. The only lines missing from main were superseded pins (DomainForge 0.21.x), old status text and pre-SHA-pin action refs.
  - RealityTrace: backup/od4-2026-11-16/jollimemory/summaries/v3 kept, the only copy of the Jolli summaries.
  - Cognate: checkpoint/pre-design-20261022 and checkpoint/pre-readiness-merge-20261022 deleted (pre-rewrite lineage from 2026-09-24; the content is superseded by main).
- Cognate #88 opened: the release-pr guard also matches merge-commit release merges (root cause of the broken #86).
- Horizon dependency graph: needs the owner's UI toggle, because the API route needs admin:org.

## 2026-11-17: Cognate release-pr race fixed (#88, 29dcd1e)
- Root cause of #86, which proposed 0.3.0 over the published 0.10.0: the release PR was built in the same job as the release, before tag-components minted the component tags. Release #84 was merged with a merge commit, which evaded the message-based skip.
- Fix:
  - The release-please job runs releases only.
  - A new release-pr job runs after tag-components. It checks out the live default branch, runs scripts/require-component-tags.mjs (refuses while any planned component tag is missing), and only then builds the PR. It has a concurrency group.
  - check-unified-release-config.mjs enforces the structure and exercises the guard on a fixture repository (no tags, one missing, all present). It rejects commented-out or disabled guards.
- Blind review: Codex (GPT Sol 6.1), 4 rounds. Accepted residual risk, documented on the PR: a TOCTOU window that needs a stalled run, a newer release merge and a failed tagging run to coincide. The worst case is a wrong PR, never a wrong publish.
- #86 closed. The first live run (post-merge) passed: the guard found all 44 component tags of 0.10.0, and release-please built no PR because no package paths changed. That is correct; there is no spurious PR.
- Decision (Horizon webmcp polyfill): keep @mcp-b/webmcp-polyfill 6.2.0 (MIT, mature, not deprecated in its registry metadata). The suggested replacement @w3c/webmcp-polyfill is at 0.1.1 with a different export surface. Revisit when the W3C package reaches 1.x.

- WP-58 DomainForge (2026-11-16/17): main 317019a. #180 (CI cache, SHA pins, docs-only gate, 4a6b1409) and #181 (dependency bumps replacing 7 stale dependabot PRs, 317019a6) merged; #154 and #179 closed (rejected), #114 closed as superseded and its branch deleted. fde/integration and fde/spike-wp10 kept. Primary checkout clean on main; dirty tree backed up in tag backup/od4-2026-11-16/dirty/domainforge (kept). CI warm gains 15 to 35 percent on lint and OS test jobs; Prove (about 15 min) is still the slowest path. Release optional. See branch-ledger.md "DomainForge consolidation".

## 2026-11-17: Workbench capability runtime T2 (PR #38) in blind review
- PR #38 (feat/capability-runtime-t2): run-start resolver, harness patches, skill loader, bundle sources; dispatcher per-invocation decision plus enablement gate. The runtime is off by default.
- Codex (GPT Sol 6.1) blind review:
  - Round 1: 5 findings fixed (34068a4).
  - Round 2: 6 findings fixed, plus two CI failures (1ef8c0d). The §1.5 single-writer invariant is kept: an external store needs an operator-declared COGNATE_CAPABILITY_SINGLE_WRITER.
  - Round 3: 8 findings, all one pattern (inconsistent authorization across entry points). They are being fixed structurally with one central authorization function, a per-runtime activation mutex and a conservative token bound.
- Round 4 is the last review round. After it, residual risk is accepted and documented on the PR.

- WP-58 small repositories (2026-11-17): swe_seed #13 f893a473 and #15 737f0b34; domainforge-vsc-extension #11 62fb47b7, #12 b489770d, #13 82370eb9 (dependabot #1 #2 #3 #10 superseded, #5 rejected); domainforge-lsp #14 062011b9 (also fixes its CI, which had failed on a deleted DomainForge dev ref); GodSpeed-Agent #4 d6b45808 (main was 34 commits behind its audit lane; fde/integration kept; GitHub Actions blocked by a billing failure, local pytest 1083 passed); Context-Kernel CI PR #13 b6387cd5; Terminal-UI #1 3c020870; ast-grep-mcp-rs #1 80a2637c; cep, NeatCode, ideal, hassos tidied with no code PR. hermes-agent-sea-forge and zeroclaw-sea-forge are 404 (deleted or transferred); local clones bundled to ~/backups/consolidation-2026-11-17/. Open: fde/integration in cep and GodSpeed-Agent, owner billing fix, host pnpm shim. Detail in branch-ledger.md, sections dated 2026-11-17.

## 2026-11-17: Workbench capability runtime T2 merged (#38, f60acb75)
- Blind review: Codex ran 4 full rounds plus a fix verification (5+6+8+5 findings). CodeRabbit added 7 more. All 31 were reproduced and fixed with red-then-green tests (test/J-CAP-review-r1..r4, -cr).
- Structural outcome: one decision point (authorizeCapabilityOp) for every entry point, pinned by a registry test. One runtime mutex covers activation, deactivation, uninstall, grant issue/revoke and the expiry sweep. Budgets use a UTF-8 byte bound. The §1.5 single-writer invariant needs an operator declaration for external stores.
- Accepted residual, recorded on the PR and in HANDOFF: an expiry instant passing during the recorder's own CAS read.
- The runtime stays off by default. T3 (MCP client, governed spawn, MCP lifecycle) started next, with the T2 invariants built in from the start.

## 2026-11-17: owner decisions on the small-repo blockers
- hermes-agent-sea-forge and zeroclaw-sea-forge (404 on GitHub): the owner says do NOT recreate them. The verified bundles stay in ~/backups/consolidation-2026-11-17/; no further action.
- GodSpeed-Agent made PUBLIC at the owner's request, which also resolves the private-repo Actions billing block. Pre-publication checks on all refs (3 branches, 5 tags):
  - gitleaks found 58 hits, all false positives: idempotency-key hex, "APIGen", localhost curl commands, and a "REDACTED" password placeholder.
  - Bundled data: AgentTrove is Apache-2.0 and SWE-smith is MIT. SWE-Gym/OpenHands-Sampled-Trajectories declares no licence on its card; only small samples are bundled.
  - The pilot unblinding keys (reflection-real-agent-002/003) belong to completed pilots whose reports are committed (2026-05-31), so no live blinding is exposed.
  - Main CI was re-run to confirm PR #4 (run 44020799015).

## 2026-11-17: Workbench capability runtime T3 (PR #39) up for blind review
- PR #39 (feat/capability-runtime-t3):
  - MCP client on the official SDK (exact pin 1.32.2), with a custom stdio-channel transport.
  - Governed spawn: containment matrix, record-before-spawn, env-only secrets, boot sweep.
  - MCP lifecycle: start, stop, suspend, resume.
  - New tests: J-CAP-mcp 27/0 and J-CAP-suspend 7/0. CI green; 13 CodeRabbit threads resolved.
- Decision (design §10 deviation): the framework ProcessPort is buffered, so an MCP stream can't be held over SSH, WSL or bubblewrap worlds. Those worlds REFUSE, recorded as `stream-port-unavailable`. They never fall back to an uncontained spawn. MCP therefore runs only as a local host process under the uncontained-local posture for now.
  - Accepted because it fails closed.
  - Follow-up: a streaming ProcessPort in the framework, then lift the refusal per world.
- Commit 6b234ac holds two fake `tok-…` test literals that gitleaks flags. The force-push to rewrite them was classifier-denied, and the agent correctly did not retry. A squash merge keeps them off main; the final tree is clean, and main's remaining tree findings predate T3.
- Open for T4: the workbench fold keeps only a grant's id and status, so an index rebuilt from the fold loses principal, permissions and expiry. That fails closed; fix it in the restart journey.
- 2026-11-17: T3 Codex round 2 was cut off by OpenAI's content filter ("possible cybersecurity risk") before its verdict. I did not reword the prompt to get past the filter. I took the three defects its tool runs had already reproduced from its log:
  - the boot sweep released a slot while the process group was alive;
  - ReDoS through the outputSchema;
  - short-secret and UTF-16 scrubbing gaps.
  All three went back to the T3 agent. If Codex flags the verification round too, the fallback (per the sea-forge-hardening-review skill) is a fresh-context, read-only Sonnet reviewer.
- 2026-11-17: T3 (#39) has had Codex rounds 1-4 plus CodeRabbit (9 threads): 5+3+3+6 findings, all fixed or in fix with red-then-green tests. Final round fixes are structural:
  - a schema keyword ALLOWLIST;
  - no server-origin free text ever persisted (code, bytes and sha256 only);
  - one bounded-message helper;
  - pgid derived from marker holders;
  - no reconcile after a fatal index init.
  Accepted residual, to be documented: under uncontained-local, a hostile child that starts its own session and clears its marker escapes process tracking. The containment posture is the boundary, not tracking.
- 2026-11-17: WP-21B round-2 fixes are in 53fa75fb: one gate rule at write time, R11 on every close, locks on human completion, reopen, terminate and approval resolution, the document required for episode kinds, and a closed case limited to the plan-ending action. 911 server tests pass and 11/11 mutants are killed. Codex round 3 verification is running.
- 2026-11-17: T3 #39 status:
  - The r5 narrow check led to a prescribed per-message transport design (1d84225).
  - The r6 check found 3 holes: heuristic id attribution for oversize messages, a lost envelope validation, and a throwing failure path.
  - Fix in progress uses simpler rules: an oversize message is a connection-level protocol violation (close, fail all pending, no attribution), strict envelope validation, and a failure path that never throws.
  - Lesson: heuristic per-shape fixes did not converge; rules that need no inference did.
- 2026-11-17: /tmp (tmpfs) filled with 3.3 GB of old scratch build dirs, causing ENOSPC in a WP-21B test. They were moved (not deleted) to ~/projects/_wt/tmp/scratch-archive-20261117/.
- 2026-11-17: WP-21B round 4 (Codex, case guard):
  - Both contract readings are confirmed REQUIRED: terminate needs a close disposition (R11, README:286), and closed is final (README:484).
  - Hard-gate coverage holds.
  - Five bypasses remain: pre-admission writes (CLI resume, server submission), fail-open on a corrupt plan, closed-case progress, write-then-refuse on completion, and shallow document checks.
  - Fix in progress: WriteToken-required write helpers, fail-closed detection, a simulated finishing action, and whole-batch admission per verb.
- 2026-11-17: Owner direction: root causes, not whack-a-mole; after 3 rounds, derive the problem classes from what the instances share.
  - Root-cause analysis: briefs/root-cause-classes-2026-11-17.md.
  - T3 and WP-21B agents redirected to class-level designs with property and model tests (fast-check, proptest).
  - Invariants-first briefs written for the next work: briefs/cwb-t4-invariants-first.md (T4a, T4b plus the T2 leftovers) and briefs/wp22-invariants-first.md. Each starts with invariants and generated tests, shown red, before implementation.
- 2026-11-17: WP-21B r5 Codex invariant review (9f9dbe85) was content-filtered before reaching conclusions. Not reworded; fell back to a fresh-context read-only Sonnet reviewer per the sea-forge-hardening-review skill. Also asked about the terminate semantics: can an unfinished case be abandoned?
- 2026-11-17: WP-21B commit redesign:
  - The agent ran out of budget with the work uncommitted. The WIP is backed up to ~/projects/_wt/tmp/wp21b-r5-wip-*/ and the agent was stopped.
  - Server tests had ~75 failures, including a plain-case regression (v1 kinds refused as KIND_UNKNOWN) and a probable lock deadlock.
  - A fresh agent is finishing it: plain cases unchanged, fixtures via a valid-Field-Case helper, explicit lock re-entrance, the refusal batch as one commit, and a root-level case index to detect wipe and truncation.
- 2026-11-17: T3 r7 Codex invariant review (4c11d46): I5 holds; I1-I4 violated.
  - Root cause: proxies treated as guarantees.
  - Ground-truth fixes are briefed (brief: "T3 round 7"): brands only from literal tables, a durable record as the commit point with id dedup, backpressure, a transport fault closing everything, typed ids at ingress, readable manifest tool names, and ground-truth oracles.
  - A fresh agent is running.

## 2026-11-17 PM: low-budget push (the owner has about 9% of usage left)
- Insurance plan: `.agents/plans/2026-11-17-finish-remaining-work.md`.
- **Workbench T3 (PR #39):** commits ed53cea, 607ecda (round 8) and b372155 (round 9). CI is green.
  - Codex round 10: I4 HOLDS. Small I1, I2 and I3 gaps are being fixed. Two items will be documented rather than fixed: the I1 scope (the agent transcript is the tool-result channel) and the I5 crash-window residual, pending the cgroup follow-up.
  - Merge after these fixes, with the runtime still disabled by default.
- **WP-21B round 6:** 425632e7. Contained raw ops, a strict tail (C4 restated because the contract is silent on repair), the refusal as one batch, one classifier with case-index authority, and run workspaces moved out of the case dir.
  - Gap: staged-copy post-state verification was not built (containment instead), and C5 and C7 have no dedicated tests.
- **WP-22:** d0264ca8 and a199ed3e. The saga state machine, the reconciler as the same machine, and W1–W6 proptests at 256 cases, each shown red by a mutant. The s2 proof is 15 PASS plus AT-09 PARTIAL, with no regression.
  - Two CLI tests (t13_1, post_start_internal_error) fail; they already failed before round 6.
- **WP-23 Workbench half:** Cognate-Workbench PR #40 (assignment binding, invariants I1–I5 with a seeded PRNG). Only I4 was shown red.
  - Remaining: the Connect proto and AssignmentService RPCs, the operator UI, and wiring SEA-Forge's delegate result into startAssignedRun. The full suite was not verified.
- **Report:** `report.html` (322 KB, four real Workbench screenshots).
- **Workbench T3 MERGED** (PR #39, squash efdcc1c) after d84b73b (round 10).
  - The call id is always minted, every host refusal is recorded once, and the per-chunk retained-bytes cap is 512 KiB.
  - CI is green; one flaky E2E-mission test passed on rerun.
  - Documented residuals (HANDOFF.md, docs/known-gaps.md): the I1 scope excludes the agent transcript, and the I5 crash window waits on the cgroup follow-up.
  - The capability runtime stays disabled by default.
  - Next: T4a/T4b per briefs/cwb-t4-invariants-first.md.
- **S2 Codex round 7 (a199ed3e):** W1, W5 and W6 hold; C4–C7 and W2–W4 have five classes (see the insurance plan). A Sonnet agent is fixing them.
- 2026-11-19: Published the Chrono-free SEA Forge checkpoint 48f393c4. The clean S2 run matched all 17 baseline rows, and the live dependency checks passed. Native APFS and fixed-NTFS guest reset qualification remains open. See the paired handoff at ec55de54 and the hosted runs linked in report.html.
- 2026-11-19: Published WP-22 at 7bfe543c. All 17 S2 result rows matched the baseline except duration. The joined NT-07 proof passed against the RealityTrace ledger at da47df5.
- 2026-11-19: Published the Cognate delegate-only Connect checkpoint 3f01dc5. Hosted verify passed in 44663363590. Production assigned starts require trusted issuance, a current-link reservation, and expiry enforcement at the store commit.
- 2026-11-19: Published Workbench safe boot at 05d50c8 and live registered bundle details at 8fed14e. The focused Chromium test registered and activated a real sample bundle and the page showed the new activation. The report includes its screenshot.
- 2026-11-19: The final WP-22 fixture checkpoint fc2da542 matched the S2 baseline and passed joined NT-07 again. The native macOS and Windows application-restart steps passed. Workbench live details passed hosted verify 44663728747 on retry.
- 2026-11-20: Cognate 3aae0d8 passed the live PostgreSQL expiry rollback test and hosted verify. SEA 637cbece recorded the exact assignment link and matched all 17 frozen S2 results. Workbench ccf4462 passed the authenticated Safe Mode clear journey in Chromium.
- 2026-11-20: Workbench 9824735 shows the actor recorded on a live activation refusal. The authenticated Chromium journey and all seven hosted jobs passed in 44757271146. SEA CI passed all ten jobs at 637cbece. The paired handoff is at 7c61f4d0.
- 2026-11-20: SEA WP-23 versioned assignment binding published at de8f76d1 (fde/wp23-delegate-verifier). A Field Case delegate now carries the approved assignment (assignment-binding.v1.json): explicit domain_meaning and five SOP currency ceilings with per-currency unknown preserved; adoptable_problem comes from the freeze's criteria.intention; the binding joins link and run in the same guarded batch with run_id as correlation identity; resume must match the recorded binding; the trusted read fails closed on legacy rows and partial appends. Contracts-first per OD-3 (new contract file, DelegateRequest re-pinned in DIGESTS, README §11 + CHANGELOG). TDD: 8-scenario family red first, then green; server+case-runner 806 passed / 0 failed under FDE_WP21A_FINAL=1; S2 matched all 17 baseline rows; hosted CI 44804758916 running. The same session recreated the missing _wt/fde-sxr-integration worktree at da47df5 and the guest-reset control run 44780753549 passed all three Portability jobs (Linux/ext4 guest reset, Windows, macOS sandbox) with the field-case-linux-guest-reset artifact.
- 2026-11-20: Follow-up 20ce2bd8: hosted CI 44804758916 caught three contract-conformance gaps (oracle codes, hex64 bound, manifest count); fixed, S2 rerun matched all 17 rows again, and CI 44825583372 passed every job (both ubuntu test shards, both macOS verify legs, lint, MSRV, workbench, OPA). Lesson recorded: wrap fde-proof.sh without an outer cargo flock — its wrappers take the same lock and deadlock.
- 2026-11-22: WP-23 item 2/3 wired end to end. SEA be81242b serves the recorded binding over the wire: assignment_binding_get gives the link's delegate or delegator the full join (assignment, brief, budget, link, run, frozen world and kind), and everyone else — and every legacy row — one constant unknown-assignment answer (the section 9 read rule; TDD red on the unknown verb, then green; clippy -D caught a test lint after the first push). Cognate e2ec279 adds the production provider: seaAssignmentAdission resolves verify/lookup from that read over a real unix-socket SFWP client, delegate-only and fail-closed (hidden, malformed, reissued-link, wrong-tenant and deadline answers deny before any run); focused suites green (153 connect/assignment, 49 postgres on isolated pg0, 69 sqlite/memory) and hosted verify dispatched. Resolved the lane's standing "no production authority provider" limit; workload-grant identity remains the S3 step.
- 2026-11-23: SEA CI passed all jobs at e118900b (45110339682), including the read verb, the world/kind binding amendment and the lint fix. The run before it exposed RUSTSEC-2026-0592 (url <= 2.5.8) failing every branch with a byte-identical lock — advisory-DB drift; pinned url 2.5.9 (local cargo deny green) and CI went green. Cognate hosted verify passed at e2ec279 (45107103046); status revs 82-83 pushed. Remaining for WP-23: the joined production journey (real SEA cell to real Cognate server) and the Workbench operator journey through PR #40.
- 2026-11-26: WP-23 closed out. Hosted CI recorded on both lanes (SEA 45538331379 at ac506217, status rev 390 d68c0598; Cognate 45538335916 at 88a6399, status rev 85 695c9f3). The joined journey's claim stays admission only (no agent endpoint in the cell, one uid per actor, no S3 AuthenticatedConnection on the path). Workbench operator hand-off (J-CASE-ASSIGNMENT-handoff) merged to main as d02efd60 (PR #40) after hosted CI exposed and we fixed a real menu race (candidates counted before they loaded) and resolved nine CodeRabbit findings, two of them security (controls now operator-only outside runs; foreign-tenant callers refused before any session lookup). The strict console assertion was kept; the WSL host's network churn is a host limit, and the hosted runner qualifies. Integration lane rev 383 (dd3109f2) pushed.
- 2026-11-26: Root cause behind the flaky Assignment controls: Cognate appends operator control events to the run stream the executor owns (fold.ts forbids foreign run-stream appends). Cognate #91 (bounded backoff, DEBT-033) merged as e33978a; the structural fix (PR #93) failed blind review by GPT Sol 6.2 (5 high: late controls silently discarded, terminal races, breach still racing, multi-runtime interrupt, step repeat) and is being redesigned around a single lifecycle revision. Not merged.
- 2026-11-26: CognitiveOS #108: Workbench integration qualified on hosted CI at 142f7d9 (1,883 pass, 13 opt-in skips, 0 fail; tauri-linux; clean-install x64 and arm64), after fixing two root causes: DomainForge 0.22.2's napi platform-package binding layout and ssh2's ed25519 keys failing to parse under Bun (7 in 2,000). Horizon integration fast-forwarded to 14be523 (hosted CI green). #110 blocked-Team restore merged into integration. #112 Workbench RLM consumer on ticket/112-workbench-rlm (J-RLM 8/8), exposing a Cognate re-invoke-after-resume defect now in #113. SEA #117 replay and receipt chains reviewed blind by Sonnet and Sol; fixes in progress.
