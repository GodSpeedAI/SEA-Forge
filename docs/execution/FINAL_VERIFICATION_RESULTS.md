# Final Verification Results

Run date: 2026-07-31. Branch `ultracode/sea-forge-completion`.

Toolchain: cargo/rustc 1.92.0 (edition 2021), bun 1.3.14, just 1.55.1,
devbox 0.17.5, Linux 6.18.33.2-microsoft-standard-WSL2.

## Commands and outcomes (2026-07-31, packaging pass)

| Command | Result |
|---|---|
| `cargo fmt --check` | **clean** (see "Two more gates that did not exist") |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **0 errors** |
| `cargo test --workspace --all-targets --locked --no-fail-fast` | **846 passed, 0 failed, 4 ignored** |
| `cargo clippy --all-targets --manifest-path .../src-tauri/Cargo.toml -- -D warnings` | **0 errors** (new; see below) |
| `cargo test --manifest-path .../src-tauri/Cargo.toml` | **31 passed, 0 failed** |
| `devbox run -- just proof` | **P1-P4b passed** |
| `devbox run -- just workbench-check` | **exit 0** |
| `just workbench-package` | **`.deb` and `.rpm` built** (see "The target that never built") |
| `just workbench-package-inventory` | **ok** — sidecar present, no JS runtime, no source maps |
| `python3 docs/execution/journey/drive_identity.py` | **25 live checks passed** |
| packaged app, live against a seeded cell | **started its own kernel; renderer negotiated, resolved identity, read the inbox** |

### Two more gates that did not exist

Both found by running commands the previous pass's sweep had never run.

**`cargo fmt --check` was never run locally.** The previous session verified with
`cargo test`, `cargo clippy`, `just proof`, and `just workbench-check` — none of
which check formatting. `just ci` does, and was not run. Three files
(`identity.rs`, `gen_sfwp_schema.rs`, `conformance_identity.rs`) had been sitting
unformatted since that session, and CI would have rejected the branch.

**Clippy had never run on `src-tauri` at all.** The same standalone-workspace
boundary that hid the host's tests hides its lints: `just lint` runs
`cargo clippy --workspace` from the repo root, which does not reach it. It found
dead code and an `.err().expect()` in the code added this session, immediately.
`workbench-tauri-test` now runs clippy and `fmt --check` alongside the tests, so
the host crate is linted, formatted, and tested by one gate.

The pattern is the same one this branch keeps rediscovering: **a check that
cannot see a directory reports no failures there, and that is indistinguishable
from having none.**

### The target that never built

`bundle.targets` listed `appimage`. It failed on every one of the three package
runs with `failed to run linuxdeploy`, and `just workbench-package` exited
non-zero each time.

That went unnoticed for two runs for an instructive reason: the recipe was
launched as a background command, and the *wrapper's* exit code was read rather
than the recipe's. `Bundling …AppImage` also reads like a result and is actually
a start message. A first draft of `USER_JOURNEY_EVIDENCE.md` consequently listed
a `.AppImage` among the artifacts produced — a file that has never existed.

The cause is specific and environmental: AppImage's tooling `dlopen`s
`libfuse.so.2`, and this host provides FUSE 3 only. It is one `libfuse2` install
away. It is recorded rather than waved through precisely because "environmental"
is the easiest place to hide an unmet claim, and because the artifact list is
exactly the kind of statement a reader will act on. `appimage` has been removed
from `bundle.targets`, and the route back is documented in the
`workbench-package` recipe.

The correct check is `find … -name '*.deb' -o -name '*.rpm'` — look for the
file, not for a log line that mentions it.

## Earlier run (2026-07-31, SF-005)

| Command | Result |
|---|---|
| `cargo test --workspace --all-targets --locked --no-fail-fast` | 843 passed, 0 failed, 4 ignored |
| `devbox run -- just workbench-check` | exit 0 (16 tauri, 123 renderer, 17 ui-components) |
| `python3 docs/execution/journey/drive_identity.py` | 25 live checks passed |

### The gate that did not exist

`workbench-tauri-test` is new. `workbench/apps/desktop/src-tauri` is a separate
Cargo workspace (ADR-004, K-06), so `cargo test --workspace` never compiled it
and **nothing ran the desktop host's Rust tests**. This was not hypothetical:
the SF-005 identity gate made every protected verb require an actor block,
which broke the host's two correlation-recovery tests on the day it landed, and
the entire kernel suite stayed green for a full session while they were broken.

A suite that cannot compile the code it is meant to cover reports the absence
of failures, not their absence.

## Earlier run (2026-07-30)

Retained for comparison: 819 passed, 0 failed, 4 ignored; clippy 0 errors;
`just proof` P1-P4b; `just workbench-contracts-gate` ok.

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

## Defects found by live driving (2026-07-31)

Four, in a chain, each hidden behind the one before it. The kernel suite —
839 tests at the time — passed straight through all of them, because the
escalation test asserted that an escalation *opens* an approval and never that
anyone can resolve one.

| # | Defect | Symptom in the live transcript |
|---|---|---|
| 1 | The escalation bound no settlement criteria | `approval criteria reference is missing` |
| 2 | The server never materialized `authority/active-policy.json` | `missing_config_error` on a file only the CLI wrote |
| 3 | The verified actor never reached the CLI the server shells out to | `resolved_by=operator_local` when `operator_b` approved |
| 4 | `SCHEMA_TYPES` omitted the new identity contracts | `system.get_schema` would under-report its own catalog |

Defect 3 is the one worth dwelling on: it is not a crash or a refusal, it is a
**false record**. The approval succeeded, the case advanced, and the ledger
named someone who had not made the decision. Nothing about the response
distinguished it from a correct one — only reading the transcript did.

All four are fixed in `52565b5`, and the live journey is now 17 passing checks.

## Regression found and fixed by live driving (2026-07-30)

The most important result of the previous pass also came from running the real
server rather than the test harness.

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
