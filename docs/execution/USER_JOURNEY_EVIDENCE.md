# User Journey Evidence

Status date: 2026-07-31. Every transcript below is from a real binary in a real
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

## Journey 6b — Asking the cell what it can do (live, 2026-07-31)

Part of the same driver. `thoth.ask` was implemented in the kernel from M11 but
absent from the method catalog, so `system.hello` never advertised it and no
client could discover it — an implemented capability with no reachable path.

```
== the catalog advertises what a client can actually call ==
  PASS  identity.get is advertised
  PASS  thoth.ask is advertised (it was implemented but undiscoverable)

== asking before the self-model is realized ==
  {"error": "no self-model snapshot; run 'sea-forge self-model rebuild'",
   "error_class": "self_model_error"}
  PASS  the refusal names the command that fixes it

== realizing the Genesis self-model ==   (epic 1.5)
  PASS  self-model rebuild succeeds

== thoth.ask answers, and discloses its own standing ==
  {"answer_id": "tha_3f4e38", "disposition": "denied", "claims": [],
   "omitted_claim_classes": ["identity", "architecture", "declared_capability"],
   "assurance": "local_tamper_evident", "freshness": "current",
   "snapshot_ref": "smsnap_20260731T072602Z_c70a06",
   "authority_notice": "This answer confers no execution authority."}
```

**Verified live.** The answer is a governed *denial*, not an error: this cell's
policy grants none of the three claim classes the question needed, and the
answer says which three. That is the shape epic story 3.9 asks for, and it is
what the surface renders.

Worth noting what the refusal before it did: it named its own remedy. An
operator reading `no self-model snapshot; run 'sea-forge self-model rebuild'`
does not have to find the fix.

## Journey 7 — What the Workbench now shows, and what it does not

The desktop client no longer fabricates its governance context. `router.tsx`'s
`mockGuardContext` is gone; the actor comes from `identity.get`, the cell from
the socket the host dialed, integrity and readiness from `readiness.get`. The
renderer cannot assert an actor at all — `SfwpCommand` has no actor field, and
the host attaches the verified one (`bridge.rs`).

The last two fabricated surfaces are gone. `ThothPage` rendered a written-in
answer citing "UX epic §4.6" as its evidence — a claim about the system sourced
from a design document — and `ModelsPage` rendered a `.sea` model this cell has
never held with three invented validation verdicts. Both were watermarked and
forced their pills to `unknown`, which made them honest about their *states*
while the content stayed fiction. Thoth now asks; the domain workbench declares
the method it is waiting on and resolves that standing from `system.hello`.

**Still test-driven, not live.** The transport, the host's actor selection, and
every surface's rendering are covered by 123 renderer tests and 16 host tests,
but no transcript here shows a human completing the journey in the packaged
application — SF-012 packages it, and that has not been built. Six of the ten
guards report `indeterminate` because this kernel has no verb behind them; they
are honestly undetermined rather than fabricated, which is a smaller claim than
"the guard passed" and the correct one.

## Journey 8 — The packaged product (live, 2026-07-31)

`just workbench-package` produced installable Linux artifacts, and everything
below was driven against **those** artifacts rather than a source tree.

```
target/release/bundle/deb/sea-forge-workbench_0.1.0_amd64.deb        10609060 bytes
target/release/bundle/rpm/sea-forge-workbench-0.1.0-1.x86_64.rpm     10609258 bytes
```

**`appimage` was listed as a target and never once built.** Three package runs
reported `failed to run linuxdeploy`, and the recipe exited non-zero every
time — which was missed because the surrounding background command's exit code
was read instead of the recipe's, and because `Bundling …AppImage` is a *start*
message. The cause is real and specific: AppImage's tooling `dlopen`s
`libfuse.so.2`, and this host has FUSE 3 only.

That is an environmental gap with a one-line fix (`libfuse2`), which is exactly
why it is recorded rather than waved through: an unbuilt target stayed in
`bundle.targets`, and a first draft of this document listed the `.AppImage`
among the artifacts produced. It has been removed from the targets, and the
route back is in the `workbench-package` recipe.

### 8a. What actually ships

```
$ just workbench-package-inventory
19693960 usr/bin/sea-forge-server
10809704 usr/bin/sea-forge-workbench
   23137 usr/share/icons/hicolor/256x256@2/apps/sea-forge-workbench.png
     221 usr/share/applications/sea-forge-workbench.desktop
[inventory] ok: sidecar present, no JS runtime, no source maps
```

The kernel ships *beside* the application in `/usr/bin`, which is exactly where
the supervisor's sibling lookup expects it. No Bun, no Node, no source maps.
`bundle.targets` was narrowed to `deb`/`rpm`/`appimage`; the macOS targets that
were previously listed have never been built and are no longer advertised.

### 8b. Install, open, and there is a cell (decision U-06)

`workbench/apps/desktop/src-tauri/tests/packaged_stack.rs` drives the **staged
sidecar** — byte-for-byte the file the `.deb` copies in — through the real
`CellSupervisor` and a real Unix socket. Four tests, all green:

| Claim | What it does |
|---|---|
| A cold cell serves SFWP with no operator step | starts the sidecar, negotiates `system.hello`, asserts the socket is 0600 |
| A second window adopts rather than restarts | asserts `Adopted`, then that closing it leaves the first window's kernel alive |
| Records survive a stop and start | commits a denied run, stops the kernel, restarts, requires `run.get` to return an identical record and `run_list` to still hold it |
| A cell that cannot start says why | an over-long socket path, refused with the remedy named, well inside the deadline |

Then the real thing, against the packaged binary on a seeded cell:

```
$ SEA_FORGE_ROOT=/tmp/sf-demo SEA_FORGE_SOCKET=/tmp/sea-forge-demo.sock \
    target/release/sea-forge-workbench &
srw------- 1 sprime01 sprime01 0 /tmp/sea-forge-demo.sock

  protocol: 1 | methods: 23
  identity: {"available": [{"actor_id": "operator_a", ...}, {"actor_id": "operator_b", ...}],
             "configured": true, "uid": 1000}
  cases: 2   runs: 4
```

**Verified live.** Nothing was running before launch; the window started its own
kernel, and that kernel served the cell.

And from the installed layout itself — the `.deb` unpacked, then run out of it,
with no `SEA_FORGE_SERVER_BIN` and no source tree in the picture:

```
$ dpkg-deb -x sea-forge-workbench_0.1.0_amd64.deb /tmp/sf-install
-rwxr-xr-x 19694824 /tmp/sf-install/usr/bin/sea-forge-server
-rwxr-xr-x 10903160 /tmp/sf-install/usr/bin/sea-forge-workbench

$ SEA_FORGE_ROOT=$CELL /tmp/sf-install/usr/bin/sea-forge-workbench &
app=1405136 kernel=1405192
kernel exe: /tmp/sf-install/usr/bin/sea-forge-server
  protocol 1 | methods 23
  identity: {"available": [{"actor_id": "operator_a", "roles": ["operator"]}],
             "configured": true, "uid": 1000}

$ kill -TERM 1405136
  kernel stopped with the app
```

The kernel's `exe` link is the decisive line: the application located the
sidecar as a sibling of itself inside the installed tree, which is the path that
only exists once something is actually packaged.

### 8c. The packaged renderer really runs, under the bundle's CSP

`tauri.conf.json` had `"csp": null` — no content-security policy at all, which
the mission forbids. It is now a real policy: `default-src 'self'`, no
`object-src`, no inline scripts, `connect-src` limited to `'self'` and Tauri's
own IPC origins.

Proving the frontend still works under it needed evidence, not reasoning, and
this host has no screenshot tool. So the renderer was observed instead: a
logging proxy was placed on the cell's socket, the packaged application was
pointed at it, and every line the application sent was recorded. The renderer
is the only thing that issues these, so their presence is the proof.

```
-> {"protocol_version":"1","verb":"system_hello"}                     <- host event loop
-> {"from_cursor":"01KYW0MFHA9RR86W4K4MTCB1A5","verb":"events_subscribe"}
-> {"client":"workbench","protocol_version":"1","verb":"system_hello"}  <- the renderer
-> {"verb":"system_describe"}
-> {"verb":"identity_get"}
-> {"intended_operation":{"method":"agent_run.start"},"verb":"readiness_get"}
-> {"verb":"approval_list"}
```

The renderer negotiated the method catalog, resolved its actor from
`identity.get`, read readiness, and read the approval inbox — through real Tauri
IPC, the real host bridge, a real Unix socket, and a real kernel. It also
confirms adoption: the proxy was already listening, so the application attached
to it and started no rival.

An earlier reading of this same test appeared to show the CSP blocking the
renderer. It did not: the window simply had not finished first paint within the
15-second sample, under software rendering. Worth recording because the wrong
conclusion was one step away, and "the security control I just added broke the
product" is exactly the claim that deserves a second measurement.

### 8d. Demonstration data that is not fixture data

```
$ just cell-seed
[seed]   accepted run: run_20260731T113252Z_61fc5c
[seed]   denied run:   run_20260731T113252Z_a845fd (no workspace contents — AUTH-01)
  case ...174a3b: apr_0001 is waiting in the inbox
  operator_a was refused their own approval (separation_of_duty)
  case ...cb457e: apr_0001 resolved by operator_b
```

Every record is produced by really running the kernel. Checked afterwards:

```
accepted: {"status":"accepted","basis":["authority_allow","exit_zero",
                                        "required_artifact_present:model.sea","stdout_match"]}
denied:   {"status":"rejected","basis":["authority_deny"]}
denied workspace file count: 0
artifacts/stdout.txt: OK   artifacts/stderr.txt: OK   artifacts/model.sea: OK   (sha256 -c)
```

`just cell-reset` removes it, and refuses any directory without the marker the
seed writes.

## What driving the package found

Two defects, neither of which the 843-test suite or any prior journey caught.

### A pending approval vanished from the inbox

The seeded cell had one approval waiting and one resolved. The running kernel
reported an **empty inbox**, scoped or unscoped, while the ledger plainly held
the committed `approval_request`.

Cause: `sea_forge_core::approvals::latest_by_id` folded the journal on
`approval_id` alone. Approval ids are per-case ordinals — every case's first
escalation is `apr_0001` — so the `approved` record written for one case's
`apr_0001` superseded the still-pending `apr_0001` of the other. The request
stayed committed, but nothing could enumerate it, and `approval.decide` requires
identifiers only the inbox can supply. **The work was stranded, and nothing
reported a problem.**

Fixed by keying the fold on `(case_id, approval_id)`, and scoping
`latest_status`/`check_expiry` the same way. Pinned by
`approvals::tests::resolving_one_case_does_not_clear_the_same_ordinal_in_another`
and `sfwp::approvals::tests::one_cases_resolved_approval_does_not_empty_another_cases_inbox`.
Neutralizing the fold key fails exactly those two and nothing else.

Every test in that module used a single case. The bug needed two, which is what
seeding a demonstration cell produced for the first time.

### Signalling the window orphaned its kernel

`kill <app-pid>` left `sea-forge-server` running, reparented, still holding the
socket and still serving the cell the operator had just closed.

Cause: the shutdown path was `RunEvent::Exit`, which Tauri emits from its event
loop, plus `Drop` as a backstop. A signalled process reaches neither — the
default disposition for `SIGTERM` terminates without unwinding.

Fixed with a handler for `SIGTERM`/`SIGINT` that stops the supervised kernel
before exiting, using tokio's `signal` feature (tokio was already a direct
dependency; no new one was added). `SIGKILL` remains uncatchable by anyone —
adoption is what makes that case recoverable rather than corrupting, since the
next launch attaches to the survivor instead of starting a rival.

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
