# Independent adversarial review — Stages A–C at `0b25e08`

Reviewer: a fresh, context-isolated agent. It got no builder conclusions,
only the claims and artifacts. It modified no tracked file and ran no cargo.
Its harness refused to let it write this file, so the builder transcribed the
ranked findings below from its final report without editing them. The
primary record is `attempts.jsonl` (38 attempts: 20 HELD, 9 PARTIAL,
5 BYPASS, 4 INCONCLUSIVE); probes are in `probes/`.

Caveat from the reviewer: its own shell ran under an inherited seccomp filter
(`Seccomp: 2`), so its syscall-level probes are indicative only.

## Ranked BYPASS / PARTIAL findings (reviewer's ranking)

1. **BYPASS high — ADV-A19/A20.** After a launcher SIGKILL, the workload keeps running under host UID 100001. The slot lock is released, so the next episode gets the same UID and the orphan can read and write the new workspace. Root cause: `workload.rs` `--kill-child` fires only when `unshare` exits, and nothing kills `unshare` when the launcher dies. Fix: a parent-death signal on the launch chain, and refuse any slot whose UID still has live processes.
2. **BYPASS high — ADV-B07.** A PostgreSQL backend outlived `VALID UNTIL` by 22 s when no reaper ran (launcher dead). Root cause: `postgres.rs` relies on `VALID UNTIL`, which stops only new logins. Fix: per-role session timeouts plus startup reconciliation.
3. **BYPASS high — ADV-A06.** `/mnt/c` (the Windows drive) is readable from the workload. Root cause: `jail.rs` grants read on `/`. Fix: a read allowlist or a minimal pivoted root.
4. **BYPASS high — ADV-A07.** The trusted `.sea-forge` state tree is world-readable: 17 of 18 files, including `server.yaml`, ledgers and authority policy. The root is created 0755, and only workload-state/slots are 0700. ADR-013's "0600/0700 launcher files only" is not true for this tree.
5. **PARTIAL high — ADV-A23.** The SWE_SEED harvest calls `std::fs::read` before its 1 MiB check (`delegation.rs` ~1667). A sparse file costs its full apparent size (512 MB measured). The harvest follows symlinks through `is_dir`/`read_dir` (~1620, ~1633).
6. **PARTIAL medium — ADV-B09** (derived from the code). Exposure is recorded only for accepted responses (`mcp.rs` ~278–293). Classified data in an error payload carries no label.
7. **PARTIAL medium — ADV-A16/A14.** `io_uring` is enabled and nested user namespaces are allowed. Landlock still held for `io_uring` `openat` (-13). Fix: seccomp in the helper.
8. **PARTIAL medium — ADV-C02.** Workspace files, artifacts and stdout are outside exposure enforcement. This is documented, but it is a real channel.
9. **PARTIAL low/medium:**
   - ADV-A05: home files with mode 0644.
   - ADV-A22: symlink read-through.
   - ADV-B05: catalog metadata of other sessions.
   - ADV-B06: LISTEN/NOTIFY covert channel.
   - ADV-C03: database labels are set only at issuance.

## Held under attack

- Identity and DAC between workloads (A01, A02, A04, A12).
- `/proc` and ptrace (A08).
- Sockets and egress (A10, A11).
- sudo and setuid under no_new_privs (A15).
- `--kill-child` on normal exit (A18).
- PostgreSQL peer-admin path and privileges (B02, B03), TCP to PostgreSQL (B01), connection limit (B04).

The full table is in `attempts.jsonl`.

## Inconclusive

- A17: fds inherited from a real server.
- B08: MCP frame tricks (code read only).
- B11: confused-deputy MCP tools.
- C01: renewal and revocation.

## Test-correlation weaknesses (reviewer)

- Tests exercise only normal exit, never launcher death.
- The 0600 launcher-secret fixture assumes the same DAC model the implementation does.
- The PostgreSQL peer test uses its own `pg_ident` map.
- Egress holds because of `--net`, so the Landlock TCP denial is never tested on its own.
- ACL tests pin encoded bytes, not kernel enforcement.
- Symlinks are refused only when access is granted, not afterwards.
