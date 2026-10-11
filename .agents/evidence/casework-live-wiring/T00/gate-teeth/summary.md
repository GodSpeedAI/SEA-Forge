# T00 gate-teeth — live-stack recipes (casework-live-wiring-production v0.2.1)

Date: 2026-09-23. Branch: casework/live-wiring. Runner: builder subagent (T00).

Deliverables under test:

- `apps/godspeed-casework-go/configs/live-serve.json` (new): version 1, evidence_root
  `.sea-forge/casework-live/evidence`, four `required: false` capabilities with honest
  `fixture:in-process` endpoints (real SFWP adapters land in T05). `cell_root` deliberately
  absent — injected by `casework-live-go-up` via `GODSPEED_CELL_ROOT`.
- `justfile` (append-only, +247 lines after `casework-demo-up`): `casework-cell-init`,
  `casework-server-up`, `casework-server-down`, `casework-live-go-up`,
  `casework-live-go-down`, `casework-stack-down`, plus the `casework_live_dir` variable.
- Config-load validation: `GODSPEED_CELL_ROOT=/tmp/just-t00-check-cell go run
  ./cmd/godspeed-casework -config configs/live-serve.json` (non-serve) exited 0 with
  "preflight clear (no required capability blocking)" (degraded set = all four non-required
  kinds, no adapter registered in non-serve mode). The temp cell was not created on disk.

## Commands run and outcomes

| # | Log | Command | Outcome |
|---|-----|---------|---------|
| 1 | 01-just-list.log | `just --list` | PASS — all six new recipes parse under `[casework]` |
| 2 | 02-cell-init-idempotent.log | `just casework-cell-init` ×2 + sha256 before/after | PASS — second run left server.yaml untouched (sha256 `9922dd80…e2e9c`); file is comment-only, verified to parse via the real server binary (defaults, fail-closed identity) |
| 3 | 03-server-up-first.log | `just casework-server-up` | PASS — cargo build 1m18s, server listening (pid 824476, socket `.sea-forge/casework-live/cell/server.sock`) |
| 4 | 04-server-up-teeth.log | `just casework-server-up` (again) + process count | PASS — exit 1, "server already running (pid 824476, socket …)"; /proc argv0 scan count == 1 (initial grep count of 5 matched the harness's own bash -c text; corrected in-log) |
| 5 | 05-live-go-up.log | `just casework-live-go-up` + `curl -sS http://127.0.0.1:4179/api/healthz` | PASS — healthz `{"status":"ok","provenance":"go:fixture:northstar","liveCursor":1150}` |
| 6 | 06-stack-down.log | `just casework-stack-down` + post-checks | PASS — both processes gone (argv0 scan 0/0), pidfiles removed, cell data preserved. Note: socket lingered because the first teardown's ownership check (`pgrep -f sea-forge-server`) matched the harness shell text; fixed same-session (see deviation) |
| 7 | 07-cycle2-full-chain.log | full chain re-run (cell-init → server-up ×2 teeth → live-go-up → healthz → stack-down) | PASS — idempotent cycle; teeth held again (exit 1, "already running (pid 827073…)"); fixed down removed the lingering stale socket ("socket lock free; no server owns it") and unblocked server-up; final state: 0 processes, cell data intact |
| 8 | 08-live-go-up-foreign-teeth.log | `just casework-go-up` then `just casework-live-go-up` | PASS — live up refused: "a foreign process is answering /api/healthz at 127.0.0.1:4179 (possibly the fixture casework-go-up); stop it first (just casework-go-down)"; fixture gateway then stopped cleanly |

## Teeth outcome (verbatim)

Server double-start (both cycles):

    casework-server-up: server already running (pid 824476, socket .sea-forge/casework-live/cell/server.sock)
    error: recipe `casework-server-up` failed with exit code 1

Foreign gateway on the addr:

    casework-live-go-up: a foreign process is answering /api/healthz at 127.0.0.1:4179 (possibly the fixture casework-go-up); stop it first (just casework-go-down)
    error: recipe `casework-live-go-up` failed with exit code 1

Stale-socket guard (exercised between cycles):

    casework-server-up: socket .sea-forge/casework-live/cell/server.sock already exists; if no server owns it, remove it manually

## Deviations from the task spec (with reasons)

1. `casework-server-down` lingering-socket ownership test: spec suggested removing the socket
   once "no server owns it". The natural `pgrep -f sea-forge-server` heuristic false-positives on
   any command line that merely mentions the binary (including test harnesses and editors), which
   left a stale socket after cycle 1 and would have blocked the next `casework-server-up`.
   Replaced with the server's own ownership signal: `flock -n <socket>.lock` succeeds only when
   no live server holds the socket lock (`crates/sea-forge-server/src/lib.rs lock_socket_path`
   holds an exclusive try_lock for the server's lifetime). Proven in logs 06/07.
2. `casework-stack-down` runs BOTH downs unconditionally and reports each (spec said
   `live-go-down && server-down`, then "runs both and reports each"); it still exits non-zero if
   either failed, and preserves the gateway-first order.
3. `casework-live-go-up`: when OUR OWN gateway already answers healthz (live pidfile + cmdline
   match on `casework-live/go-bin`), it reuses it (fixture-recipe behavior); every other
   healthz-answering case is the mandated foreign-process error.

## Environment / scope notes

- No cargo target-lock contention was encountered (no `cargo test` running during the gate).
- `server.yaml` for the live cell is comment-only; empirically verified that the real
  `sea-forge-server` binary parses it to defaults (fail-closed identity bindings) before it was
  used by the recipes.
- Worktree state not touched by this task and left as found: modified
  `.agents/current_status.yml`, `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml`,
  `.agents/reports/2026-09-23-case-engine-frontend-e2e-journey-mapping.md`, deleted tracked
  `target` symlink (already a real directory before this session), and untracked
  `.agents/reports/casework-live-wiring/`.
- Per instructions: no `cargo test` / `just check` / `just ci` / other gates were run; nothing
  committed; `.sea-forge/casework-live/` runtime output left in place (gitignored).
