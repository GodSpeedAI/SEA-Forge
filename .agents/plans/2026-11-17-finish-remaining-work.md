# Finish remaining FDE convergence work (insurance plan, 2026-11-17)

Hand this to an agent if the original session runs out. Read first:
- `.agents/reports/fde-convergence-2026-11-12/STATUS.md` (latest entries at the bottom)
- `.agents/reports/fde-convergence-2026-11-12/briefs/root-cause-classes-2026-11-17.md` (method: invariants plus property tests, ground-truth oracles, the three-round rule)
- the briefs `cwb-t4-invariants-first.md` and `wp22-invariants-first.md` (same folder)

**Do NOT:** the WP-58 cleanup (`.agents/plans/2026-11-17-wp58-remaining-branch-cleanup.md` is owner-only), the Horizon dependency graph, the real passkey (OD-5), a GitHub App, or Jetson root steps. Leave the reports dir uncommitted.

**Host rules:** 8 GB WSL; run `free -m` first; one heavy job at a time; `flock -o ~/projects/_wt/cargo.lock cargo … -j2`; `TMPDIR=~/projects/_wt/tmp`; Postgres via pg0 (`export PATH=$HOME/projects/cognate-workbench/.tmp/pg0:$PATH`). Never force-push, never `--no-verify`, never bypass CI or reviews.

## Priority order

1. **SEA-Forge WP-21B** (worktree `~/projects/_wt/fde-sea-rs-integration`, branch `fde/integration`; WIP is uncommitted, with a backup in `~/projects/_wt/tmp/wp21b-r5-wip-*`).
   - Finish the admit-and-apply `commit(case_dir, Batch)` redesign so that the server tests pass. Plain (non-Field) cases must behave exactly as before.
   - Fix the lock re-entrance deadlock in `review1_suspension_between_admission_and_write`.
   - Gates: C1–C7 proptest model, `scripts/fde-proof.sh --slice s2` with no regressed row, and the full workspace tests.
   - Commit, push, then run one invariant-targeted review (Codex `codex exec -s read-only`; a fresh Sonnet reviewer if Codex is filtered).
2. **WP-22**, per `wp22-invariants-first.md` (W1–W6). Then the WP-23 Workbench half.
3. **The HTML report.** One self-contained file at `.agents/reports/fde-convergence-2026-11-12/report.html`:
   - written for a high-school reader;
   - real screenshots of the running Workbench (Playwright or agent-browser), embedded as base64;
   - covers what was built, what is proven (with links to evidence), and what remains, including the owner-only items.
4. **Cognate Workbench T3** (PR #39, worktree `~/projects/_wt/cwb-t3`, branch `feat/capability-runtime-t3`).
   - Finish the round-7 ground-truth fixes (I1–I4 table in the root-cause brief).
   - Gates: property suites at ≥ 200 runs in CI and ≥ 2,000 locally, then one invariant review, then a squash merge.
   - **Status, 2026-11-17 PM:** ed53cea is pushed and CI is all green. Codex round 8 was content-filtered, but its probes, saved in the session scratchpad as `codex-cwb39-r8.err`, found these defects. Do NOT merge until they are fixed:
     - **I4:** after EPIPE with a stalled close, the SDK `_responseHandlers.size` is 1 after the call.
     - **I3:** the ajv compiled-schema cache retains about 537 KB of server schema JSON (66 entries), which exceeds the 262144 cap and is not counted in retained bytes.
     - **I5:** when the authorizer throws before the spawn intent, the slot stays reserved even though no process exists.
     - **Brands:** a probe minted Trusted, Sha256 or MintedId values from hostile bytes. Check whether the probe used an exported mint function that accepts arbitrary strings; if so, make it module-private or literal-only.
     - **Class fix:** per-connection resources (SDK handlers, the schema cache, queues) are owned by the connection object and released in one `dispose()` that every fault and close path runs. The cache counts toward the cap. Slot reservation is a scoped acquire that every exit before the spawn releases.
     - Also note that I2 `record_failed` with zero finished rows is the intended fail-closed outcome; document it in the invariant text.
5. **T4a, then T4b plus the T2 leftovers,** per `cwb-t4-invariants-first.md`. Then docs and enablement (the flag stays off by default).
6. **S3 WP-30–35** (an A3 passkey runbook for the owner), **S4 WP-40–44**, **S5 WP-50–57**, then the follow-ups listed in STATUS.md.

After each item, append one dated line to STATUS.md.

## Next for the SEA-Forge S2 lane (Codex round 7 of a199ed3e; full text in the session scratchpad `codex-s2-r7.md`)

W1, W5 and W6 hold. The violations fall into five classes, and each needs one fix:
1. **A verb spans several stores, non-atomically** (C5: approval consumption commits, then the ledger append fails). Fix: one `commit` batch covers every file a verb touches, through the staged-copy design (stage, verify the post-state, swap atomically).
2. **Two classifiers, plus a creation mode chosen from file presence** (C6: `field_guard.rs:55` versus the authoritative classifier; C4: delete plan.json, then `Op::Plan` re-enters creation mode). Fix: delete the second classifier. Creation mode is allowed only when NO case index and NO case files exist.
3. **The case dir is not pure** (C7: sandbox stdout and stderr artifacts under `cases/<id>/runs/.../artifacts/`, written by `isolated.rs:222`). Fix: all sandbox outputs go under `<root>/.run-workspaces/`. Then add a test that walks the case dir after an execution and compares it to the committed batch.
4. **Create-if-absent races** (W3: two evidence settlements; W4: two `pin_case` calls). Fix: atomic create (`O_EXCL` or `link()`) or the proposal lock around check-and-write, plus a concurrency proptest with real threads.
5. **Durability and the reconciler's order** (W2: no directory fsync after creating log dirs; the reconciler returns PreSaga before checking for orphan effects). Fix: fsync the parent dirs on create. The reconciler classifies from the effects first; any effect without a log is Uncertain, so it fails closed.
