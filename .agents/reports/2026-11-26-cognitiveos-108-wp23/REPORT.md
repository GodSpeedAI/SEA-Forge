# CognitiveOS #108 and WP-23: interim report (2026-11-26)

The session hit its usage limit, so this work is unfinished. Every claim below has evidence in `evidence-ledger.md` next to this file.

## Done
- **WP-23 Workbench hand-off (PR #40) merged to main as d02efd60.** A person can hand an approved case assignment to a conversation, then interrupt it, resume it and report spend. Only the local operator can use these controls, and only outside a run. Hosted CI caught a menu race, which is fixed, and 9 CodeRabbit findings, which are fixed (2 were security).
- **Integration lane status revision 383** records the hosted results for both lanes and the honest limit: the joined journey proves admission only.
- **Cognate DEBT-033 (PR #91) merged.** Retries of the assignment append now back off.
- **#109: the Workbench integration passed hosted CI** at 142f7d9 (1,883 tests pass, 0 fail, 13 opt-in suites skipped). Two root causes were fixed along the way: DomainForge's new binding layout and invalid SSH keys under Bun.
- **#110 merged into the integration branch**: restoring a Team now reports when an archived member must be reactivated first.
- **#112 (Workbench recursive child runs)** is on ticket/112-workbench-rlm. Hosted CI passed (45836458373). Every refusal happens before anything is dispatched.
- **#117 replay and receipt hardening** is at 0b7a2eca: all local gates pass after two blind reviews. It is not pushed yet.
- **Cleanup (task F)** is done. Scratch files are archived in sea-rs/.tmp/evidence-archive.

## Open, in priority order
1. **Workbench main is red.** The WP-23 browser journey fails at random, 2 of 5 hosted runs. After interrupt and resume, the reconciliation card sometimes never appears. The cause is the Cognate race, where control events share the run stream. A diagnosis agent was stopped before reporting; its worktree is ~/projects/_wt/wp23-e2e-fix. Next step: gate the slow scenario so the interrupt always lands mid-call, or land the Cognate fix.
2. **Cognate PR #93 (structural control-stream fix) is not merged.** A blind Sol review found 5 high-severity issues. A redesign around one lifecycle revision was in progress when it was stopped, in ~/projects/_wt/cognate-asg-stream (uncommitted). Releasing it is authorized.
3. **#117:** Sol pass 3 on 0b7a2eca was still running. Its output is in sea-rs/.tmp/sea-117-review/pass-0b7a2eca/. Next: triage it, push, integrate and run hosted CI. Then build the peer-verifying relay, which needs a dedicated service account the owner must provision.
4. **#113 (Cognate re-invokes the provider after resume; a duplicate-spawn risk)** was stopped part-way in ~/projects/_wt/cognate-rlm113.
5. **#114 (bubblewrap contained port)** was stopped part-way in sea-rs/.tmp/cog108-114-cwb. bwrap 0.13.1 works unprivileged on this host.
6. **Not started:** #115 (metered egress), #116 (SEA allow bound to the launch-plan digest), #118–#121, and the issue comments and closure of #122.

## Update (second usage limit)
- **Main fix ready:** Workbench PR #42 makes the WP-23 interrupt land mid-call by construction, with no weakened assertions. Hosted verify passed twice on 579cfef and a third run is queued. Merge it once that run passes. It is not merged yet.
- **Cognate PR #93 (controls off the run stream):** three review passes plus a narrow delta check. The last check found 0 high and 2 medium, not yet triaged. The fallback reviews were fresh Sonnet sessions because Sol refused at OpenAI's safeguard; they count as not independent.
- **Cognate PR #94 (#113, no duplicate provider invoke):** pass 2 found 2 high. Red tests are committed (d99ac31) and the fix is in progress.
- **#117 receipts:** fixed at 38d0f79b, with every gate green and a delta check done. Not pushed.
- **#116 SEA half:** `execution_profile` binding done locally (ee0d8653), and the CEP validator accepts it. Its review was running. The Workbench host comparison is not started; it needs #114.
- **#114:** the bubblewrap port works, with every control observed from outside. Review fixes (mount pinning and others) were in progress.
- **#112:** review fixes were in progress.
- **Not started:** #115, #118–#121, and closing the issues.
