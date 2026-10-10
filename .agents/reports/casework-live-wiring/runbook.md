# Casework cell: operations runbook (T11 part A)

Covers backup, restore, restarts and known operational debts. Config and packaging are in
`config-reference.md`; the units are in `deploy/systemd/`.

## 1. What is in a cell (what to back up)

Everything under the cell root (`SEA_FORGE_ROOT`, e.g. `/var/lib/sea-forge/cell`) is durable kernel
state and is backed up as one unit:

| Path | Content |
| --- | --- |
| `server.yaml` | identity bindings, gateway delegation, limits |
| `ledgers/` | append-only ledgers incl. `delegation-audit`, authority decisions, MMR |
| `cases/<id>/` | `case.json`, `case-events.jsonl`, plans, settlements |
| `runs/`, `artifacts/`, `sealed/` | run records, governed artifacts, sealed transcripts |
| `approvals.jsonl` | approval journal |
| `requests/` | request-id correlation records (idempotency across restarts) |
| `templates/`, `authority/` | case templates and the active policy |
| `drafts/`, `self-model/`, `declarations.jsonl` | drafts and self-model snapshots |
| key material | any signing keys under the root. The continuation-token signing key is generated in process memory at startup and is not on disk (tokens do not survive a kernel restart). |

Excluded on purpose: `server.sock`, `server.sock.binding`, `server.sock.lock`, `.server.lock`
(runtime artifacts, recreated). Not part of the cell: gateway config and secrets under `/etc/sea-forge`
(back those up separately, they hold OIDC secrets), the evidence root, logs. The archive contains
keys, approvals and case data: it is written mode 0600; store it encrypted and off-host.

## 2. Consistent snapshot

A copy of ledgers while the kernel appends can tear. The kernel holds an exclusive flock on
`<root>/.server.lock` for its whole life, which gives a race-free quiesce:

1. Stop the gateway first (`systemctl stop godspeed-casework`), then the kernel
   (`systemctl stop sea-forge-server`; SIGTERM is a clean shutdown). Dev cell: `just casework-stack-down`.
2. Run `scripts/casework-cell-backup.sh <cell-root> <archive.tar.gz>`
   (`just casework-cell-backup <archive>` for `.sea-forge/casework-live/cell`).
   The script **refuses** while the lock is held ("the cell is live"), then takes the same lock itself
   for the copy, so a server cannot be started mid-snapshot (it would fail with "another sea-forge
   server already owns cell"). It writes `<archive>`, `<archive>.sha256` and `<archive>.manifest`
   (sha256 of every file).
3. Start the services again. Order: kernel, then gateway.

Downtime is the length of the copy. There is no hot-backup procedure; do not copy a live cell.

## 3. Restore

1. Stop services. Move the damaged cell aside (`mv cell cell.damaged-$(date +%F)`): restore never
   deletes and refuses a non-empty target.
2. `scripts/casework-cell-restore.sh <archive.tar.gz> <new-cell-root>`
   (`just casework-cell-restore <archive> <target>`). It verifies the archive digest first, extracts
   under the cell lock, then compares the extracted tree to the manifest; any mismatch removes
   what this run extracted and exits non-zero.
3. Fix ownership/mode if restoring as root (`chown -R sea-forge:sea-forge`, dirs 0750).
   Check `server.yaml`'s `gateway.uid` and `identity.bindings` uids match the host's service user
   (uids can differ between hosts).
4. Start the kernel, then the gateway. Check `GET /api/readyz` and `/api/healthz`.
5. Users sign in again (sessions are not restored, see section 4).

## 4. Verified for real

`TestLiveCellBackupWipeRestoreServesIdenticalRecords` (`apps/godspeed-casework-go/internal/server/cell_backup_live_test.go`,
run `go test -tags live -p 1 -run TestLiveCellBackup ./internal/server`) uses the real kernel:
commit a case, execute an item through the gateway, SIGTERM the kernel, back up, delete the cell,
restore into a new path, start a new kernel on it, and assert the case ledger, the request record
and the delegation-audit record are identical (deep-equal) and the newest case is served through the
real gateway stack; replaying the pre-backup intent id does not re-execute the item. Teeth: backup
of a live cell is refused and leaves no archive; a bit-flipped archive is refused with nothing
extracted; restore into a non-empty directory is refused; archive mode is 0600.

## 5. Restart behavior and ordering

| Event | Effect |
| --- | --- |
| Gateway restart | All browser sessions are lost (in-memory by design); users sign in again. The kernel keeps all durable state. The page stays "Reconnecting" until reloaded (DEBT CW-46). Revision history held in the gateway (SSE replay window) is rebuilt from the kernel; clients that resume with an evicted cursor get `resync_required`. |
| Kernel restart (unit restart) | Gateway unit restarts with it (`Requires=`). Intents in flight are resolved through `request.get_status` using their `request_id`; the kernel dedupes by request id (`request_id_reused` on payload drift). The continuation-token key is regenerated: outstanding continuation tokens become invalid. |
| Kernel crash | systemd restarts it (`Restart=on-failure`, max 5 in 120s). The gateway reconnects on its own (L-RECOV). |
| Both | Start kernel, wait for the socket, start gateway (the unit does this). |

Rolling a stuck pair: `systemctl restart sea-forge-server` (restarts the gateway too).

## 6. Known debts that affect operations

- **CW-45 (kernel gap):** approving an escalated settlement over SFWP records the approval
  (`approvals.jsonl`, ledger entries) but emits no case event; the case stays `awaiting_approval` and
  nothing re-runs the item. After an approval an operator must continue the lifecycle by hand (the
  ladder's L7 does this). Do not treat "approved" as "continued".
- **CW-46 (UI):** after a gateway restart the UI does not self-heal until reloaded, and the artifact
  service caches an error per artifact ref until reload. Advise users to reload after any restart or
  restore.
- **CW-44:** the ladder harness differs from a production cell (dev auth, hand-built server.yaml,
  no video/trace evidence). A production cell is not exercised by the ladder; config validation
  (`config-reference.md` section 5) is what guards the difference.
- Sessions are single-process in-memory; running two gateway replicas needs shared session storage
  (plan redesign trigger), not supported today.

## 7. Not yet done in T11 (later parts)

Load test with budgets (`just casework-load`), `/security-review` of the gateway and delegation, CI
jobs, the 50-SSE-client kernel-restart tooth, and a real systemd runtime test of the units.
