# Operations and Startup

Status date: 2026-07-31. Everything below was exercised by the tests named
against it; nothing is aspirational.

## Installing the product

```sh
sudo dpkg -i sea-forge-workbench_0.1.0_amd64.deb   # or the .rpm / .AppImage
sea-forge-workbench
```

That is the whole procedure. There is no second binary to install and no
service to start first — see "Who starts the kernel" below.

Supported targets are Linux `deb`, `rpm`, and `AppImage`. macOS is **not
built** and not advertised: the packet requires its Seatbelt journey to pass
before it may be called supported, and that journey has not been run.

## Who starts the kernel (decision U-06)

The Workbench supervises its own kernel, and adopts one that is already
running rather than starting a second.

| On launch | What happens | On quit |
|---|---|---|
| Something is listening on the cell's socket | The window **adopts** it | The kernel is left running |
| Nothing is listening | The window **starts** the bundled `sea-forge-server` sidecar | The kernel is stopped |
| Nothing is listening and the sidecar cannot start | The window opens and reports the reason | — |

Consequences worth knowing before you rely on them:

* **Closing a window stops a kernel that window started**, ending any work in
  flight. If you need work to outlive the window, start the server yourself —
  the Workbench will adopt it and will not stop it. This is the supported
  escape hatch, and it is why adoption exists.
* **Three exits, three mechanisms.** Closing the window runs Tauri's
  `RunEvent::Exit`; a panic unwinds through `Drop`; `SIGTERM`/`SIGINT` are
  caught by a handler installed at startup. The third was found the hard way —
  a signalled process runs neither of the first two, so `kill <app-pid>` left a
  reparented kernel still serving the cell the operator thought they had
  closed. `SIGKILL` cannot be caught by anyone; adoption is what makes that case
  recoverable instead of corrupting, since the next launch attaches to the
  survivor rather than starting a rival.
* **Two windows on one cell are fine.** The second adopts the first's kernel.
  Closing the second does not disturb the first.
* **A crashed kernel leaves a stale socket, which is not a trap.** The server
  binds a staging path and renames it into place, atomically replacing the dead
  file, so the next launch starts cleanly.

The server binary is located, in order: `SEA_FORGE_SERVER_BIN`, then a sibling
of the application executable (where the package puts the sidecar), then
`PATH`. An override naming a missing file is an error rather than a fallback —
starting a different server than the one named would be worse than starting
none.

Pinned by `workbench/apps/desktop/src-tauri/tests/packaged_stack.rs`, which
drives the staged sidecar — the same file the `.deb` ships — through the real
supervisor and a real socket.

## Seeing it work on a fresh machine

A new cell is empty, so the surfaces have nothing to show. To populate one with
real records:

```sh
just cell-seed        # ~/.sea-forge-demo
just workbench-demo   # open the packaged Workbench against it
just cell-reset       # remove everything the seed created
```

The seeded records are not fixtures. Each one is produced by really running the
kernel — real runs, really authorized, really settled, really committed to the
append-only ledger — so following any of them to its evidence checks out
exactly as if you had done the work by hand. `scripts/reset-cell.sh` refuses
any directory without the marker `seed-cell.sh` writes, so it cannot be pointed
at a real cell.

## The cell

A *cell* is one `.sea-forge` directory holding a server's socket, its
configuration, its ledgers, and its runs. One server owns one cell for its
whole lifetime.

Resolution order (`crates/sea-forge-server/src/config.rs`):

| Priority | Source | Effect |
|---|---|---|
| 1 | `SEA_FORGE_SOCKET` | Absolute socket path. Overrides the socket only; the cell root still resolves below. |
| 2 | `SEA_FORGE_ROOT` | Cell root. |
| 3 | default | `.sea-forge` relative to the current directory (server, CLI); `$HOME/.sea-forge` (desktop). |

The socket is `<root>/server.sock`, owner-only (mode 0600).

**Unix path limit.** `sun_path` is 108 bytes on Linux and 104 on macOS, and the
server enforces a 95-byte budget to leave headroom. A cell root nested deeply
enough to overflow it is rejected at startup, rather than failing inside
`bind()` with an opaque errno:

```
Error: Input("socket path is 120 bytes but a Unix socket allows at most 95: <path>
Choose a shorter cell root (SEA_FORGE_ROOT), or point SEA_FORGE_SOCKET at a short
path such as /run/user/$UID/sea-forge.sock while keeping records where they are.")
```

This fired for real during this pass against a deep scratchpad directory; the
remedy in the message worked without consulting the source.

## Starting the server

```sh
# default cell, relative to the current directory
sea-forge-server

# explicit cell
SEA_FORGE_ROOT=/srv/forge sea-forge-server

# short socket path for a deeply nested project
SEA_FORGE_SOCKET=/tmp/forge.sock sea-forge-server
```

### Startup fails closed

`server.yaml` lives inside the cell. Startup behaviour
(`crates/sea-forge-server/src/main.rs`, `conformance_lifecycle.rs`):

| Condition | Result | Test |
|---|---|---|
| No `server.yaml` | Starts on defaults. A first run is not a failure. | `an_absent_server_yaml_is_a_first_run_not_a_failure` |
| `server.yaml` exists but does not parse | **Refuses to start**, `error_class = server_config_error` | `a_malformed_server_yaml_blocks_startup` |
| Agent endpoint invalid | **Refuses to start** | `an_invalid_agent_endpoint_blocks_startup` |
| `notify_command` argv0 not found | **Refuses to start**, naming the fix | `a_notify_command_that_cannot_run_blocks_startup` |

The last one is a preflight, not a runtime discovery: a bare program name is
resolved against `PATH`, an absolute path is checked directly. A cell whose
notify hook can never fire would otherwise run for hours looking healthy.

Falling back to defaults when a written `server.yaml` fails to parse would run
the cell under a configuration nobody wrote — the wrong agent endpoint, the
wrong concurrency, no notify hook — while every log line claimed health. That
is why it is a hard stop.

## Reload

Configuration reloads when a plan is committed
(`ServerState::reload_config`).

| Condition | Result | Test |
|---|---|---|
| Valid `server.yaml` | The live snapshot is swapped whole. A dispatch already holding a snapshot keeps it to completion. | `a_valid_reload_replaces_the_live_snapshot` |
| Invalid `server.yaml` | Last-known-good is kept; an `invalid_reload_error` event is published for the operator. | `an_invalid_reload_keeps_the_last_known_good`, `an_invalid_reload_publishes_an_operator_visible_event` |
| `server.yaml` absent | **No-op.** Nothing to re-read means nothing to change. | `a_reload_with_no_file_present_changes_nothing` |
| `root:` or `socket_path:` drifted into the file | Ignored. The cell is fixed at startup. | `a_reload_cannot_relocate_the_cell` |

The absent-file case is deliberately different from startup. At startup, no
file means "first run, use defaults." On reload it would mean "deleting
`server.yaml` on a running cell silently reverts every setting to default" —
a fail-open of exactly the kind the reload path exists to prevent.

## Request timeout

Every request except `events.subscribe` is bounded at 10 seconds
(`REQUEST_TIMEOUT`, spec §11.1). `events.subscribe` is excluded because it is
long-lived by design.

On timeout the caller receives:

```json
{
  "error": "request exceeded the 10s server timeout; the work was not cancelled",
  "error_class": "request_timeout",
  "timeout_seconds": 10,
  "request_id": "…",
  "recover_with": "request.get_status"
}
```

**The work is not cancelled.** The bound is applied to a spawned task's
*handle*, not to the future, so timing out stops waiting without abandoning
work that may already have durable effects. `a_timed_out_request_keeps_running`
asserts this directly; a naive `timeout(d, work)` would still compile and still
satisfy any test that only checked the response shape.

A panicking handler is reported as `internal_error` rather than propagating and
killing the connection (`a_panicking_handler_is_reported_not_propagated`).

## Where records live

```
<root>/
  server.yaml            configuration (optional; fail-closed if present and bad)
  server.sock            0600
  approvals.jsonl        operator-visible approval view
  ledgers/<stream>/      append-only JSONL — the truth
  runs/<run_id>/         minimum-CLI run records
  cases/<case_id>/
    case.json
    plan.json
    case-events.jsonl
    runs/<run_id>/       case-episode run records
      trace.jsonl
      evidence.jsonl
      workspace/         only after a committed Allow
      artifacts/         only after a committed Allow
```

Both run layouts resolve through `run.get` and appear in `run.list`, before and
after a restart (`conformance_run_locator`).

Append-only JSONL under `ledgers/` is the source of truth. SQLite, views, and
the generated TypeScript are rebuildable projections; deleting them loses
nothing.

## Migration from a v0.1 root

```sh
sea-forge migrate --root <root>
```

Relocates flat runs under their owning case, writes `migration.json`, and
commits one `legacy_import` ledger entry per file recording the destination
path and the source digest. Re-migration is refused. The digest recorded for
each file is verified against the bytes that landed at the destination by
`conformance_m0_migrate_imports_v01_root_losslessly_and_blocks_re_migration`.

Migration is optional: the run locator reads both layouts without it.

## Verification commands

```sh
devbox run -- just check                       # fmt, clippy, tests, no-async-kernel
devbox run -- just proof                       # spec-minimum §12.2 P1-P4b
devbox run -- just workbench-check             # Bun gates + the contracts gate
devbox run -- just workbench-contracts-gate    # generated-zone and boundary drift only
devbox run -- cargo test --workspace --all-targets --locked --no-fail-fast
```

## Known operational gaps

- **Identity is not resolved from the host.** Protected verbs attribute work to
  the request's `entity` string with `ActorRole::Operator` hardcoded. Separation
  of duty cannot be enforced. This is SF-005, blocked on decision U-07 — see
  `REMAINING_BLOCKERS.md`.
- **No packaged artifact has been produced or validated.** SF-012 (packaged
  Linux stack) and SF-013 (release gate) are downstream of that same blocker.
  Nothing in this pass was published, signed, or distributed.
