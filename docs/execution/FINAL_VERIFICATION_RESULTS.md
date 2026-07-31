# Final Verification Results

Run date: 2026-07-30. Branch `ultracode/sea-forge-completion`, HEAD after the
five commits listed in `FINAL_COMPLETION_REPORT.md`.

Toolchain: cargo/rustc 1.92.0 (edition 2021), bun 1.3.14, just 1.55.1,
devbox 0.17.5, Linux 6.18.33.2-microsoft-standard-WSL2.

## Commands and outcomes

| Command | Result |
|---|---|
| `devbox run -- cargo fmt --all` | clean, no diff |
| `devbox run -- cargo clippy --workspace --all-targets --locked -- -D warnings` | **0 errors, 0 lint warnings** |
| `devbox run -- cargo test --workspace --all-targets --locked --no-fail-fast` | **819 passed, 0 failed, 4 ignored, 84 suites** |
| `devbox run -- just proof` | **P1-P4b passed** |
| `devbox run -- just workbench-contracts-gate` | **ok** — contracts, tokens, and Tauri boundary current |

Clippy emits 14 manifest warnings of the form *"only one of `license` or
`license-file` is necessary"*. These are Cargo manifest hygiene notes present
before this pass, not lints on code, and `-D warnings` does not fail on them.

## The four ignored tests

None of them is skipped to make the suite pass. Each is gated on an external
host that is not present, and says so in its ignore reason:

```
self_invoke_noop_fail                ... ignored
self_invoke_noop_pass                ... ignored
t16_1_real_acp_host_release_gate     ... ignored, release gate:
                                         set SEA_FORGE_REAL_ACP_ARGV/SEA_FORGE_REAL_ACP_ENV
t16_6_real_swe_seed_release_gate     ... ignored, release gate:
                                         configured SWE_SEED-projected ACP host required
```

These are pre-existing release gates. This pass added no `#[ignore]` and
deleted no assertion.

## Tests added by this pass

| Suite | Tests | Covers |
|---|---|---|
| `sea-forge-settlement` (unit) | 1 | `two_settlements_are_distinguishable` |
| `conformance_case_episode` | 4 new (11 total) | trace/evidence emission, denial trace, escalate approval, record materialization |
| `conformance_run_locator` (new file) | 6 | both run layouts, restart, duplicate id, case settlement visibility, traversal |
| `conformance_m0_migrate` | strengthened | destination-digest comparison replacing a presence-only check |

## Adversarial checks

A test that cannot fail proves nothing. Each change was re-run against a
deliberately broken version of the code it covers.

### SF-004 run locator

With the case-owned probe neutralized in both `run_dir` and `run_dirs`:

```
test a_case_sees_the_settlement_of_its_own_case_owned_run ... FAILED
test run_list_reports_both_layouts_once_each              ... FAILED
test both_layouts_still_resolve_after_a_restart           ... FAILED
test both_layouts_resolve_through_one_run_get             ... FAILED
test a_duplicated_id_resolves_to_the_flat_layout          ... ok
test the_case_owned_probe_does_not_widen_the_traversal_surface ... ok
test result: FAILED. 2 passed; 4 failed
```

The two that still pass are guards — they are meant to hold in both states.

### SF-007 contracts gate

Each probe run against the real recipe; the tree was restored to clean after
every one.

| Probe | Exit | Message |
|---|---|---|
| `driftProbe` property added to `HelloResult.schema.json` | 1 | "generated TS/AJV contracts drifted from the committed schemas" |
| new `ZZDriftProbe.schema.json` | 1 | same |
| comment appended to `sea-forge.tokens.css` | 1 | "sea-forge.tokens.css has drifted" |
| `[workspace]` commented out in `src-tauri/Cargo.toml` | 1 | "restore the empty [workspace] table … (ADR-004)" |
| clean tree | 0 | "ok: generated contracts, UI tokens, and the Tauri workspace boundary are current" |

One probe deliberately does **not** fail: hand-editing a file under
`generated/` passes, because the recipe regenerates the directory before
diffing. That is the correct semantic — the gate's claim is "committed output
equals generated output," and a hand edit that regeneration erases was never in
the committed output. Source-side drift is what matters, and the first two
probes cover it.

### SF-004 migration digest guard

The guard `assert!(relocated_run_files > 0, ...)` fired on its first run and
caught a wrong path prefix in the check itself — the assertion was comparing
against `runs/<id>/…` when migrate records the *destination* path. Without the
guard the check would have silently iterated zero entries and passed.

## Regression found and fixed by live driving

The most important result of this pass came from running the real server rather
than the test harness.

First live journey (`USER_JOURNEY_EVIDENCE.md`, Journey 2), before the fix:

```
== run_get run_… ==
  settlement: "unsettled"
== case_get_overview ==
  {"settlements": []}
```

for a run that had settled as **rejected**. The settlement was in the ledger;
`run.get` and `case.get_overview` read `settlement.json`, which the server
dispatcher never wrote. Every case-dispatched run rendered as unsettled — a
false claim about the run, not a report of a missing record.

After materializing `settlement.json` and `authority.json`:

```
== run_get run_20260731T003017Z_fc0ca3 ==
  settlement: "rejected"
== case_get_overview ==
  {"settlements": [{"run_id": "run_…", "status": "rejected",
                    "basis": ["authority_deny", "legacy_unattributed_criteria"], …}]}
== filesystem ==
  ['authority.json', 'evidence.jsonl', 'settlement.json', 'trace.jsonl']
```

— and still no `workspace/` or `artifacts/`, so AUTH-01 holds. Pinned by
`a_settled_episode_materializes_the_records_the_views_read`.

This defect had been reasoned away during SF-003 as "duplicating a projection."
It survived 818 passing tests. It did not survive one live run.

## What was not verified

- **No packaged artifact was built, signed, or published.** SF-012 and SF-013
  are blocked (see `REMAINING_BLOCKERS.md`). Nothing was written to any
  external system.
- **No desktop/Workbench journey was exercised end-to-end.** Identity is
  fabricated in the router; SF-005 is blocked on decision U-07.
- **`just workbench-check`'s full Bun leg** (install/check/build/test) was not
  re-run to completion in this pass; only the contracts gate it now depends on
  was. The Bun leg is unchanged by these commits.
- **macOS and Windows** were not exercised. All results are Linux/WSL2.
