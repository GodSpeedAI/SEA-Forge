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

## Journey 6 — Two actors, one approval (live, 2026-07-31)

`docs/execution/journey/drive_identity.py`, over a real Unix socket against a
cell binding `operator_a` and `operator_b`. All 17 checks pass. The uid comes
from `SO_PEERCRED`, so nothing in the script can assert it.

```
== identity.get ==
  {"available": [{"actor_id": "operator_a", "roles": ["operator"]},
                 {"actor_id": "operator_b", "roles": ["operator"]}],
   "configured": true, "uid": 1000}

== a protected verb with no actor block ==
  {"error_class": "identity_required", "no_side_effect": true}

== claiming an actor this uid does not hold ==      identity_not_bound
== verifying one actor, attributing work to another == identity_entity_mismatch

== operator_a submits work that escalates ==        apr_0001 opened
== operator_a tries to resolve their own approval ==
  {"error": "actor `operator_a` requested the work approval `apr_0001` gates
             and cannot resolve it; approval requires a different actor",
   "error_class": "separation_of_duty", "no_side_effect": true}
== the same attempt on a brand-new connection ==    still separation_of_duty
== the approval is still pending ==                 apr_0001 remains in the inbox

== operator_b resolves it ==
  {"ok": true, "output": "approval_id=apr_0001\nstatus=Approved\nresolved_by=operator_b\n"}
```

**Verified live.** This is the journey SF-005 exists for, and it is the one that
found the four defects in `52565b5` — every one of which the 839-test kernel
suite passed straight through.

## Journey 7 — What the Workbench now shows, and what it does not

The desktop client no longer fabricates its governance context. `router.tsx`'s
`mockGuardContext` is gone; the actor comes from `identity.get`, the cell from
the socket the host dialed, integrity and readiness from `readiness.get`. The
renderer cannot assert an actor at all — `SfwpCommand` has no actor field, and
the host attaches the verified one (`bridge.rs`).

**Still test-driven, not live.** The transport, the host's actor selection, and
every surface's rendering are covered by 123 renderer tests and 16 host tests,
but no transcript here shows a human completing the journey in the packaged
application — SF-012 packages it, and that has not been built. Six of the ten
guards report `indeterminate` because this kernel has no verb behind them; they
are honestly undetermined rather than fabricated, which is a smaller claim than
"the guard passed" and the correct one.

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

# 6 — two actors, one approval
CELL2=/tmp/sf-id-cell && rm -rf "$CELL2" && mkdir -p "$CELL2"
# argv0 must be named `sea-forge` AND canonicalize to the process that runs it,
# which for an in-process submit is the server, not the CLI.
ln -sf "$PWD/target/debug/sea-forge-server" "$CELL2/sea-forge"
printf 'identity:\n  bindings:\n    - uid: %s\n      actor_id: operator_a\n      roles: [operator]\n    - uid: %s\n      actor_id: operator_b\n      roles: [operator]\n' \
  "$(id -u)" "$(id -u)" > "$CELL2/server.yaml"
SEA_FORGE_ROOT="$CELL2" SEA_FORGE_SOCKET=/tmp/sf-id.sock \
  ./target/debug/sea-forge-server &
python3 docs/execution/journey/drive_identity.py "$CELL2" /tmp/sf-id.sock

# 3
devbox run -- just proof
```
