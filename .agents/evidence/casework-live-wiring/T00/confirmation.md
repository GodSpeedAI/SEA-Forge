# T00 independent confirmation — live-stack recipes

Verdict: **APPROVE**

Date: 2026-09-23. Branch: casework/live-wiring. Verifier: independent critic subagent
(did not build the unit). Every command below was executed by the verifier against the
uncommitted working tree; outputs quoted are the verifier's own observations, not the
builder's claims.

Unit under review: the appended justfile block after `casework-demo-up` (+247 lines),
new `apps/godspeed-casework-go/configs/live-serve.json`, and evidence under
`.agents/evidence/casework-live-wiring/T00/gate-teeth/`.

## Findings (all personally observed)

1. **Append-only justfile change; fixture/UI recipes untouched.**
   `git diff --numstat justfile` -> `247	0`; `git diff justfile | grep -c '^@@'` -> `1`
   (single hunk, `@@ -1398,3 +1398,250 @@`, i.e. appended directly after
   `casework-demo-up`). Zero pre-existing lines deleted, so no fixture/UI recipe changed
   semantics. `just --list` still shows `casework-go-up/down/status`,
   `casework-ui-up/down/status`, `casework-demo-up` unchanged alongside the six new
   recipes, all under `[casework]`.

2. **All six required recipes present with required behaviors.**
   `just --list` shows: `casework-cell-init`, `casework-server-up`,
   `casework-server-down`, `casework-live-go-up addr="127.0.0.1:4179"`,
   `casework-live-go-down addr="127.0.0.1:4179"`, `casework-stack-down`. Each carries
   `[group('casework')]`; `casework_live_dir := ".sea-forge/casework-live"` variable
   present; bodies use `{{set}}` shebang style, pidfile+log discipline, readiness polls
   (server: 120x0.5s = 60s for `test -S socket`; gateway: 40x0.5s on /api/healthz),
   log tails on failure, `setsid nohup` starts. `cargo build --locked -p sea-forge-server
   --bin sea-forge-server` matches the pre-existing recipe style at justfile:436.

3. **live-serve.json matches DELIVERABLE 1 exactly and loads clean.**
   File has version "1", `evidence_root: ".sea-forge/casework-live/evidence"`, four
   capabilities (authority/execution/repository/artifact) all `required:false` with
   endpoint `fixture:in-process`, and NO `cell_root` key. Validated against the real
   loader: `cd apps/godspeed-casework-go && GODSPEED_CELL_ROOT=/tmp/critic-t00-cell go
   run ./cmd/godspeed-casework -config configs/live-serve.json` -> exit 0,
   `configuration loaded (version 1, cell_root "/tmp/critic-t00-cell", ... 4
   capability/ies, 0 resolved secret(s))`, `preflight clear (no required capability
   blocking)`. Strict `DisallowUnknownFields` decoding accepted the file (no unknown
   keys); env injection of cell_root worked; temp cell was not created on disk
   (non-serve mode is side-effect free). Honesty notes are in justfile comments plus the
   start-up echo `(FIXTURE-LABELED providers until T05)`, and healthz itself reports
   `"provenance":"go:fixture:northstar"` — honest.

4. **casework-cell-init is idempotent, both branches.**
   Existing-cell branch: run twice, sha256 of `cell/server.yaml` identical before/between/
   after: `9922dd80dc28c4d90680213a0a99a56ba1fce6624747349cf0d68345425e2e9c`, exit 0 both
   times, prints resolved absolute cell path, never modifies the file. First-run write
   branch: moved `server.yaml` away, ran the recipe -> "wrote ... (comment-only ...)",
   and the newly written file is byte-identical (same sha256) to the original; file
   restored. Content is comment-only; `ServerConfig::load` yields defaults (verified in
   crates/sea-forge-server/src/config.rs:112-127 — absent/comment-only file path, and the
   server ran successfully against it in every cycle below, with empty identity bindings
   = fail-closed).

5. **Full gate chain passes, twice, with clean teardown.**
   Cycle 1: `just casework-cell-init && just casework-server-up` -> "listening (pid
   839598, socket .sea-forge/casework-live/cell/server.sock)"; `just casework-live-go-up`
   -> "answering at http://127.0.0.1:4179/api/healthz (pid 841464 ...)";
   `curl -sS http://127.0.0.1:4179/api/healthz` ->
   `{"status":"ok","provenance":"go:fixture:northstar","liveCursor":1150}`.
   `just casework-stack-down` -> gateway first then server, both reported ok, exit 0.
   Post-checks: /proc scan by exe (`*/sea-forge-server`, `*casework-live*`) found 0
   processes; both pidfiles gone; cell data intact (server.yaml same sha256, ledgers/,
   requests/, .server.lock preserved). Cycle 2 after additional abuse (see finding 8):
   full chain re-ran end to end, exit 0, same clean end state.

6. **TEETH (a) double server-start:** second `just casework-server-up` -> exit 1,
   `casework-server-up: server already running (pid 839598, socket ...)`. /proc exe scan
   showed exactly one `sea-forge-server` process; `ss -xa` showed exactly one LISTEN
   unix socket for the cell. (Note: ss renders the listener path as `server.sock.binding`
   because the kernel retains the bind-time address while the server publishes via
   bind-then-rename — crates/sea-forge-server/src/lib.rs:1071-1084; an AF_UNIX connect
   to `server.sock` succeeded, proving the published socket is live. Not a recipe
   defect.)

7. **TEETH (b) foreign gateway:** with the live stack down, `just casework-go-up`
   (fixture) started on 127.0.0.1:4179; `just casework-live-go-up` -> exit 1 with
   `a foreign process is answering /api/healthz at 127.0.0.1:4179 (possibly the fixture
   casework-go-up); stop it first (just casework-go-down)`. No silent reuse. Fixture
   `casework-go-down` then stopped cleanly.

8. **TEETH (c) stale pidfile + ungraceful kill (socket ownership):**
   - `echo 999999 > .sea-forge/casework-live/server.pid; just casework-server-up` ->
     "removing stale pidfile (pid 999999 not alive or not the server)", then built and
     started (pid 842942); `casework-server-down` stopped it cleanly.
   - kill -9 the running server (pid 843311): socket lingered on disk, the kernel
     released the socket flock (verifier's own `flock -n server.sock.lock true`
     succeeded), `casework-server-down` reported "not running (stale pidfile removed)"
     + "removed lingering socket ... (socket lock free; no server owns it)", exit 0, no
     data loss; a subsequent `casework-server-up` started and served normally.
   - Live-owner safety proof: started the server, deleted the pidfile, ran
     `casework-server-down` -> "not running (no pidfile)" + "socket ... is still owned
     by a live server (socket lock held); leaving it", exit 0; the server was unharmed
     and its socket still accepted an AF_UNIX connect ("STILL-ACCEPTING"). The recipe
     provably does NOT delete a socket a live server owns.

9. **Foreign-pid refusal in both down recipes:** wrote a live `sleep` pid into
   `server.pid` -> `casework-server-down` exit 1, "does not look like sea-forge-server;
   not killing it" (sleep survived). Same for `go.pid` -> `casework-live-go-down` exit 1,
   "is not the live casework gateway; refusing to kill it". No foreign process was
   killed by either recipe.

10. **casework-stack-down failure propagation (both directions):** foreign pid in
    `server.pid` -> live-go-down ok, server-down FAILED reported, stack-down exit 1.
    Foreign pid in `go.pid` -> live-go-down FAILED reported, server-down STILL RAN
    ("not running (no pidfile)") and was reported ok, stack-down exit 1. Both downs run
    unconditionally, each is reported, gateway-first order held, cell data preserved.

11. **casework-live-go-up guard for missing cell:** with `cell/server.yaml` moved away,
    `just casework-live-go-up` -> exit 1, "no cell at .sea-forge/casework-live/cell -
    run just casework-cell-init first". File restored (sha256 unchanged).

12. **Reuse branch (deviation 3):** with our gateway running, a second
    `just casework-live-go-up` -> exit 0, "our gateway already answers at ... - reusing
    it" — the reuse is announced, gated on live pidfile + cmdline containing
    `casework-live/go-bin`, and is the fixture-recipe `casework-go-up` behavior; every
    other healthz-answering process triggers the mandated foreign error (finding 7).

13. **Worktree scope:** final `git status --porcelain` shows exactly: ` M .agents/
    current_status.yml`, ` M .agents/plans/2026-09-23-casework-live-wiring-production.
    plan.yaml`, ` M .agents/reports/2026-09-23-case-engine-frontend-e2e-journey-mapping.md`,
    ` M justfile`, ` D target`, `?? .agents/evidence/casework-live-wiring/`,
    `?? .agents/reports/casework-live-wiring/`, `??
    apps/godspeed-casework-go/configs/live-serve.json` — the expected set (the three
    modifications and the `target` symlink deletion were pre-existing and out-of-unit
    per the handoff). `.sea-forge/` is gitignored (.gitignore:21). Nothing committed.
    Builder's gate-teeth logs (01..08 + summary.md) are authentic raw transcripts that
    corroborate the same behaviors, including an honest in-log correction of a pgrep
    self-match artifact.

## Material deviations from the original instructions (judged)

1. **casework-server-down uses `flock -n <socket>.lock` instead of a pgrep-style
   liveness check for lingering-socket ownership.** SOUND, and strictly better than the
   suggested heuristic: the server takes an exclusive process-lifetime lock on
   `<socket>.lock` (crates/sea-forge-server/src/lib.rs `lock_socket_path`, lines
   1027-1051), which is exactly the flock(2) domain `flock -n` probes, so success
   proves no live owner. The builder's own log 04 demonstrates a real pgrep false
   positive (the harness's own bash -c text matched). The verifier proved both
   directions live: dead owner -> lock free -> socket removed and the next up works;
   live owner -> lock held -> socket left and the server kept accepting connections
   (finding 8).

2. **casework-stack-down runs both downs unconditionally and reports each, exiting
   non-zero if either fails.** This is what the instructions themselves specify
   ("run both downs (gateway first), report each, exit non-zero if either fails");
   verified in both failure directions (finding 10). Not a defect; listed for
   completeness because the builder flagged it.

3. **casework-live-go-up reuses OUR OWN gateway (live pidfile + go-bin cmdline match)
   when it answers healthz, instead of erroring on any answering process.** SOUND:
   "foreign" in the instructions means not-ours; the reuse branch is announced (not
   silent), narrower than the fixture recipe's (cmdline must contain
   `casework-live/go-bin`), and the fixture-gateway case still errors exactly as
   mandated (findings 7 and 12).

## Minor observations (no action required; not approval blockers)

- On foreign-pid refusal, `casework-server-down` removes the pidfile before exiting 1,
  whereas `casework-go-down` and the new `casework-live-go-down` keep it. Both
  behaviors are defensible (a pidfile pointing at a foreign pid is stale by
  definition); the refusal itself — the safety property — is identical (exit 1, no
  kill). Slight inconsistency between the two new recipes, within the instruction's
  "mirroring" latitude.
- `casework-live-go-up` kills an alive-but-not-answering pid from its own pidfile
  without a cmdline check. This is inherited verbatim from the fixture
  `casework-go-up` (justfile:1321-1327), which the instructions told the builder to
  mirror; the theoretical pid-reuse exposure pre-exists in the fixture recipe and is
  not new in this unit.
- Provenance note: `.agents/reports/casework-live-wiring/decision-log.yaml` (mtime
  21:55) and `.agents/evidence/casework-live-wiring/T00/baseline/` (21:41-21:54)
  predate the bounded unit (live-serve.json 22:00:57; gate-teeth 22:08) and are
  mandated by plan T00 steps 1 and 3; the bounded instructions already incorporate the
  plan amendment v0.2.1 that the decision log records. Consistent with the builder's
  "left as found" claim; flagged per protocol since they are untracked paths outside
  the three deliverables.
- `just casework-server-up`'s socket-exists guard fires before stale-pidfile cleanup,
  so after a crash a bare re-up asks the operator to run down (or remove the socket)
  first. This matches the instructed guard order ("alive pidfile => error; existing
  socket => error; stale pidfile => clean") and the fixed down unblocks it (finding 8).

## Verdict

APPROVE. The implementation satisfies every deliverable and verification requirement of
the original instructions; all three reported deviations are sound and were re-proven
live by the verifier; the minor observations above are within the instructed latitude
and do not affect safety or correctness. No gate was weakened; no fixture/UI recipe
changed; nothing was committed.
