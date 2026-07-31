# User Journey Evidence

Status date: 2026-07-30. Every transcript below is from a real binary in a real
cell, not from a test harness. Test-only evidence is labelled as such.

## Journey 1 — Operator starts a cell (live)

Built binaries: `devbox run -- cargo build -p sea-forge-server -p sea-forge-cli --bins`.

### 1a. A broken `server.yaml` refuses to start

```
$ printf 'agent: [this is not valid\n' > $CELL/server.yaml
$ SEA_FORGE_ROOT=$CELL ./target/debug/sea-forge-server
exit=1
... "error_class":"server_config_error" ...
```

The cell does not come up under a configuration nobody wrote. **Verified live.**

### 1b. A cell root too deep for a Unix socket is rejected with the remedy

Booting with a 120-byte socket path:

```
Error: Input("socket path is 120 bytes but a Unix socket allows at most 95: /tmp/.../journey/cell/server.sock
Choose a shorter cell root (SEA_FORGE_ROOT), or point SEA_FORGE_SOCKET at a short path such as
/run/user/$UID/sea-forge.sock while keeping records where they are.")
```

This fired for real during this pass — the scratchpad path happened to exceed
the limit — and the message named both remedies without needing the source.
**Verified live, unplanned.**

### 1c. Following the remedy starts the cell

```
$ SEA_FORGE_ROOT=$CELL SEA_FORGE_SOCKET=/tmp/sf-journey.sock ./target/debug/sea-forge-server &
socket: srw-------   alive=yes
```

Owner-only (0600), as the contract requires. No `server.yaml` present, and a
first run is not a failure. **Verified live.**

## Journey 2 — Submit a case that policy denies (live)

Driven over the real Unix socket with a plain NDJSON client
(`scratchpad/journey/drive.py`), against a policy with no rules — default deny.

```
== submit (deny policy) ==
{"case_id": "case_20260731T003017Z_86f02a", "exit_code": 3, "state": "terminated"}

== run_list ==
1 run(s): ['run_20260731T003017Z_fc0ca3']

== run_get run_20260731T003017Z_fc0ca3 ==
  settlement: "rejected"

== case_get_overview ==
{"case_id": "case_20260731T003017Z_86f02a",
 "settlements": [{"run_id": "run_20260731T003017Z_fc0ca3",
                  "status": "rejected",
                  "basis": ["authority_deny", "legacy_unattributed_criteria"],
                  "review_required": false,
                  "settled_at": "2026-07-31T00:30:17.768365444+00:00"}]}

== filesystem ==
  run_20260731T003017Z_fc0ca3: ['authority.json', 'evidence.jsonl',
                                'settlement.json', 'trace.jsonl']
```

What this proves, live:

| Claim | Evidence in the transcript |
|---|---|
| AUTH-01: a denial creates no workspace | the run directory holds four record files and **no `workspace/`, no `artifacts/`** |
| A denial is still recorded | `trace.jsonl`, `evidence.jsonl`, `authority.json`, and a settlement all exist |
| SF-004: a case-owned run resolves | `run_list` returned it and `run_get` opened it — this list was empty before the locator change |
| Settlement is criteria-based, not exit-code-based | `basis` opens with `authority_deny`; no process ever ran, so there was no exit code to read |
| A denied required item terminates the case | `state: terminated`, `exit_code: 3` |

### What this journey caught

The **first** run of it reported `settlement: "unsettled"` from `run.get` and
an empty `settlements` array from `case.get_overview`, for a run that had in
fact settled as rejected. The settlement was committed to the ledger; the views
read `settlement.json`, which the server dispatcher never wrote.

That is a false statement in the exact operator-facing view SF-004 exists to
make reachable — "unsettled" is a claim about the *run*, not a report that a
record is missing. It had been reasoned away during SF-003 as "duplicating a
projection." Driving the real socket disproved that.

Fixed by materializing `settlement.json` and `authority.json` from the values
already decided, and pinned by
`conformance_case_episode::a_settled_episode_materializes_the_records_the_views_read`.
The transcript above is the post-fix run.

## Journey 3 — Minimum CLI lifecycle (live, via `just proof`)

```
$ devbox run -- just proof
[proof] running spec-minimum §12.2 P1-P4b
[proof] P1-P4b passed
```

P1-P4b drive the real `sea-forge` binary through the minimum lifecycle,
including a denied run whose flat `workspace/` must exist and be empty. That
assertion is why the CLI's pre-authority scaffold was left alone while the
server path was tightened.

## Journey 4 — Server lifecycle under bad input (test-driven, real binary)

`crates/sea-forge-server/tests/conformance_lifecycle.rs` spawns the actual
`sea-forge-server` binary via `env!("CARGO_BIN_EXE_sea-forge-server")`:

| Scenario | Outcome |
|---|---|
| Malformed `server.yaml` | startup blocked |
| Invalid agent endpoint | startup blocked |
| Absent `server.yaml` | starts on defaults |
| `notify_command` that cannot run | startup blocked, naming the fix |
| Valid reload | live snapshot swapped |
| Invalid reload | last-known-good kept, `invalid_reload_error` published |
| Reload with no file | no-op |
| Reload naming a different root | cell not relocated |
| Hung endpoint | bounded at 10s, server keeps serving, work not cancelled |

10 tests, all green.

## Journey 5 — Run resolution across layouts and restart (test-driven, real socket)

`crates/sea-forge-server/tests/conformance_run_locator.rs` boots a real server
over a real Unix socket, then boots a *second* server against the same cell:

- A case-owned run and a flat CLI run both resolve through `run_get`.
- Both still resolve after the restart.
- `run_list` reports each exactly once.
- A duplicated id resolves to the flat layout in both `run_get` and `run_list`.
- A case sees the settlement of its own case-owned run.
- `../../etc`, `..`, `case-owned/runs/run-cased`, and `run cased` are all
  refused as `not_found` — the second lookup place did not widen traversal.

6 tests, all green. With the case-owned probe neutralized, 4 of the 6 fail.

## Journey NOT demonstrated — the Workbench operator journey

**The desktop journey does not work end-to-end and is not claimed to.**

`workbench/apps/desktop/src/router.tsx:38-51` supplies a fabricated
`mockGuardContext` — `actor_op_01`, `sha256:policy_v1`, `verified`. No resolved
identity reaches the server, the server hardcodes `ActorRole::Operator`, and
separation of duty cannot be enforced or demonstrated.

Making it real is SF-005, which is blocked on decision U-07 (the exact public
SFWP identity/session contract). See `REMAINING_BLOCKERS.md`. Nothing in this
pass papered over that with a plausible-looking default.

## Reproducing these journeys

```sh
devbox run -- cargo build -p sea-forge-server -p sea-forge-cli --bins
CELL=$(mktemp -d)/cell && mkdir -p "$CELL"

# 1a
printf 'agent: [this is not valid\n' > "$CELL/server.yaml"
SEA_FORGE_ROOT="$CELL" ./target/debug/sea-forge-server ; echo "exit=$?"
rm "$CELL/server.yaml"

# 1c + 2
SEA_FORGE_ROOT="$CELL" SEA_FORGE_SOCKET=/tmp/sf-journey.sock \
  ./target/debug/sea-forge-server &
python3 docs/execution/journey/drive.py "$CELL"   # see the script in this repo's scratchpad

# 3
devbox run -- just proof
```
