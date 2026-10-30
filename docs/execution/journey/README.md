# Journey Drivers (docs/execution/journey)

## Overview
Live journey drivers used by the execution campaign to exercise a real, booted
SEA Forge cell end to end over its SFWP Unix socket. Unlike unit or socket
tests, these scripts make assertions against a running server whose identity
and authority decisions are the production ones, and they print a PASS/FAIL
line for every expectation so the output can serve as journey evidence
(referenced by `docs/execution/USER_JOURNEY_EVIDENCE.md`).

## Interface
- `drive.py` — default-deny journey driver. Takes the cell root as `argv[1]`
  ([drive.py:SOCK] binds the hardcoded socket path `/tmp/sf-journey.sock`).
  Writes a policy with no rules (`deny.yaml`) and a one-item sandboxed-task
  plan (`plan.json`) into the cell, then exercises the SFWP verbs
  `system_hello`, `submit`, `run_list`, `run_get`, and `case_get_overview`
  ([drive.py:C.call]), printing the settlement state and which records are
  present after a denial.
- `drive_identity.py` — identity and separation-of-duty journey. Usage:
  `python3 drive_identity.py <cell-root> <socket-path>`
  ([drive_identity.py:module docstring]). Exercises `identity_get`,
  `system_hello`, `ask` (thoth.ask), `submit`, `approval_list`, and
  `approval_decide`; realizes the Genesis self-model by running
  `sea-forge-cli self-model --root <cell> rebuild` via `subprocess.run`
  ([drive_identity.py:subprocess.run]); collects failures via `expect()`
  ([drive_identity.py:expect]) and exits 1 with the failure list if any
  expectation fails.

## Invariants
- The actor identity comes exclusively from the kernel's `SO_PEERCRED` read of
  the socket peer; nothing the script asserts can change it
  ([drive_identity.py:module docstring]).
- One uid maps to exactly one actor: a uid-bound cell exposes only its own
  actor to `identity_get`, and `operator_b` is reachable only through the
  gateway's `on_behalf_of` ([drive_identity.py:expect] assertions).
- A protected `submit` without an actor block is refused as
  `identity_required` with `no_side_effect: true`; an unheld actor is refused
  as `identity_not_bound`; a verified actor attributed to a different entity
  is refused as `identity_entity_mismatch` ([drive_identity.py:expect]).
- Self-approval is refused as `separation_of_duty`, including on a brand-new
  connection, and a refused approval remains pending in the inbox
  ([drive_identity.py:expect]).
- Policy matching on `argv0` requires an absolute path named `sea-forge` whose
  canonical path equals the running executable
  ([drive_identity.py:plan construction comments]).
- Before the self-model exists, an `ask` must fail with `error_class`
  `self_model_error` and name the command that fixes it
  ([drive_identity.py:ungrounded ask expectations]).

## Side Effects
- Connects to the cell's real Unix socket and sends governed requests, which
  can create cases, runs, ledger records, and approvals on the server
  ([drive.py:C], [drive_identity.py:C]).
- Writes `deny.yaml` and `plan.json` (or `escalate.yaml` and `plan.json`)
  into the cell directory ([drive.py:plan construction],
  [drive_identity.py:policy/plan writes]).
- Executes the `sea-forge-cli self-model rebuild` subprocess against the cell
  ([drive_identity.py:subprocess.run]).
- Resolving the approval via `approval_decide` makes the server shell out to
  the real CLI ([drive_identity.py:expect on `resolved_by`]).
- `drive.py` lists cell filesystem state under `<cell>/cases/<case_id>/runs`
  ([drive.py:filesystem listing]).

## Verification
- Run: `python3 -m py_compile docs/execution/journey/drive.py docs/execution/journey/drive_identity.py`
- The real proof is running the drivers against a booted cell (see
  `docs/execution/OPERATIONS_AND_STARTUP.md` for cell startup); no `just`
  gate covers these scripts directly.
