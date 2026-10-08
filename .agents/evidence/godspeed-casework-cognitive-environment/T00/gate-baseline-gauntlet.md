# T00 evidence — existing Gauntlet gate baseline

`$GAUNTLET_WORKTREE` = `/home/sprime01/projects/gauntlet-godspeed-casework` @
`fe75108d62129ffb179b6773b67f3ee92213e44d` (branch `godspeed/casework-environment`).
The operator's active Gauntlet checkout `/home/sprime01/projects/gauntlet` was **not** built,
tested, or modified; its `git status --short` md5 was identical before and after creating the
worktree (`5abbc212bfc9a43667d42295ca86a557`).

## GATE_GAUNTLET status: RESOURCE_DEFERRED (three gates), with a real narrow observation

`GATE_GAUNTLET` is `cd "$GAUNTLET_WORKTREE" && just check && just test && just ci` and activates
at **T05**, not at T00. It was **not** run and is explicitly **not** recorded as passed.

Preflight readings immediately before the observation (`/proc/meminfo`):

```
MemAvailable:    3867452 kB
SwapFree:        7380828 kB
```

Narrow single-job observation (the plan's "begin with a narrow single-job gate and observe it"):

```
$ cd /home/sprime01/projects/gauntlet-godspeed-casework
$ CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=/home/sprime01/projects/gauntlet/target \
    cargo check -p gauntlet-domain --locked
narrow check exit=0   wall_s=137
MemAvailable: 3809564 kB   SwapFree: 7381544 kB   (after)
```

Peak RSS reported by `/usr/bin/time -v` for this observation: **32028 kB (≈31 MiB)** —
see `raw-logs/gauntlet-narrow-check.log`. Read this carefully: the wrapper's reported maximum
resident set attributes the waited-for tree for the direct child, and 31 MiB is far too small to
be a rustc peak, so it did **not** capture the compiler processes' memory. The *gate* numbers for
SEA Forge (0.99 GiB, 1036928 kB, at `just test`) have the same attribution caveat; none of them is a
whole-tree peak measurement. Read them all as lower bounds, and take the `/proc/meminfo`
readings — which never degraded below ≈3.24 GiB available — as the real headroom signal.
Readings did not degrade (headroom stayed ≈ 3.63 GiB (3809564 kB), swap free unchanged), so the machine is not
memory-starved by a single-crate Gauntlet check. The reason the broad gates are deferred anyway is the plan's
own scheduling rule: **at most one heavy build or test gate runs at a time across these
worktrees**, and the SEA Forge composite baseline (`just check`, `just ci`) had to occupy that
slot in this session. 137 s for one crate across a 29-crate workspace is also the honest signal
that a full `just check`/`test`/`ci` sweep is a multi-hour slot on this 6-core host, matching the
Gauntlet repository's own recorded sweeps.

**Exact commands to run later, verbatim** (in this order, one at a time, from inside the
dedicated worktree, after `GATE_SEAFORGE` composites have finished):

```
cd /home/sprime01/projects/gauntlet-godspeed-casework && CARGO_BUILD_JOBS=1 just check
cd /home/sprime01/projects/gauntlet-godspeed-casework && CARGO_BUILD_JOBS=1 just test
cd /home/sprime01/projects/gauntlet-godspeed-casework && CARGO_BUILD_JOBS=1 just ci
```

Recorded pre-run expectations a later task must verify rather than assume: Gauntlet's `just ci`
is `check test lint deny boundaries ruler`; the `tui-*` gates are deliberately decoupled from it;
`just boundaries` (261 checks reported at `main`) must remain green; and the TUI suites
(`just tui-check` / `tui-test` / `tui-lint`) are required in addition to the Rust checks for any
task that modifies `workbench/` — including T14's removal, which must delete them as supported
entry points rather than leave them failing.

## Cross-repository toolchain fact (hot context)

Gauntlet has **no `rust-toolchain.toml`**, so it builds with the host's `stable` toolchain
(observed: `~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo`), whereas SEA Forge
pins `1.92.0` via `rust-toolchain.toml`. A Go adapter that compiles/links across both
repositories must not assume one shared toolchain.

Raw: `raw-logs/gauntlet-narrow-check.log`, `raw-logs/gauntlet-preflight.txt`.