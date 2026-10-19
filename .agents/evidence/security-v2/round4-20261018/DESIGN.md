# Round 4 implementation design (locked 2026-10-18, operator-approved)

Mechanism C per ADR-014: seccomp user-notify supervisor now, Landlock
RESOLVE_UNIX when kernel ABI >= 9 (probe-gated switch, supervisor fallback,
never silent). Kernel install on this host was offered then DECLINED by the
operator — no kernel changes; current ABI-7 kernel is the target.

## Decisions (operator picks recorded in session)

1. SEC-06: C (supervisor) + ABI-9 upgrade path. 2. Federation: enable
export_bundle on the reserved surface, allowed only when destination
authority admits the union of exporting runs' exposure labels; import/adopt
classify on ingest. 3. Gitleaks: value-scope the 22 evidence exceptions.
4. Casework allowlist: remove generic-api-key auto-admission. 5. RealityTrace
E8: proposal only (bounded entries + digest of full diff set; observer
attribution signing as separate follow-up).

## Code layout

- `crates/sea-forge-sandbox/src/socket_policy.rs` (new):
  - `landlock_abi_version()` — raw syscall 444, VERSION flag.
  - `apply(grants: &[PathBuf]) -> Applied::{Landlock, Supervisor}`:
    ABI >= 9 → second raw Landlock ruleset handling only
    LANDLOCK_ACCESS_FS_RESOLVE_UNIX (bit 16, 1ULL<<16), path_beneath grants
    per socket tree, restrict_self. Errors fail closed (jail_unavailable).
    ABI < 9 (x86_64 only) → raw cBPF USER_NOTIF filter on connect/sendto/
    sendmsg (+ x32 aliases nr|0x40000000; other-arch fallback ALLOW — the
    existing deny filter already kills foreign arch, most-restrictive wins),
    installed with SECCOMP_FILTER_FLAG_NEW_LISTENER by the helper BEFORE
    exec; supervisor thread in the helper (unrestricted: spawned before
    Landlock per-thread restriction) serves SECCOMP_IOCTL_NOTIF_RECV:
    - read sockaddr via process_vm_readv (connect: arg1; sendto: arg1;
      sendmsg: msghdr.msg_name via two reads);
    - family != AF_UNIX → CONTINUE (TCP governed elsewhere); abstract
      (sun_path[0]==0) → CONTINUE (Landlock scope + netns govern);
    - relative sun_path → resolve against /proc/<pid>/cwd;
    - pathname: canonicalize in launcher view; allow iff under a granted
      tree; require final inode S_ISSOCK via O_PATH+fstat;
    - deny → SECCOMP_IOCTL_NOTIF_SEND EINVAL? no: flags=0, error=EACCES;
    - TOCTOU compensating control: after CONTINUE on a pathname, immediately
      re-resolve; inode changed or not-a-socket → SIGKILL workload, log.
    - NOTIF_ID_VALID before SEND; listener closure (helper exit) → kernel
      returns ENOSYS to workload = fail closed.
  - `validate_socket_grants(paths, readable: &[PathBuf])`: each grant
    absolute, canonical, and inside a declared readable tree or the
    workspace/artifacts roots.
- `WorkloadConfinement` += `socket_paths: Vec<PathBuf>`; helper flag
  `--socket-grant P`; all construction sites updated (delegation.rs 1417,
  isolated.rs 88, 7 test files). PG socket dir in delegation.rs moves from
  read-only reachability to an explicit --socket-grant entry.
- Install order in helper: Landlock (existing) → socket policy (either
  mechanism) → existing seccomp deny filter → spawn workload + supervisor
  thread (helper stays init as today).

## Tests (A.3) — crates/sea-forge-sandbox/tests/unix_socket_containment.rs

Canary listeners only, in test temp trees: outside-tree refused; under
declared read path refused unless socket-granted; granted PG dir allowed;
abstract canary refused; symlink alias to outside refused; rename/swap race
(tight loop, listener must stay unreached, workload killed on detection);
sendto unconnected datagram to canary refused; pre-restriction fd does not
leak (inherited audit: helper closes all but stdio+lifeline — CLOEXEC check);
workload-created socket in own tree allowed. Un-ignore
unix_socket_reach.rs. Mutation controls: (M-S1) drop RESOLVE_UNIX handling →
landlock tests fail; (M-S2) supervisor allow-all → refusal tests fail;
(M-S3) remove post-allow recheck → race test fails; (M-S4) remove --socket-grant
plumbing → PG allowed-test fails.

## Evidence dir: .agents/evidence/security-v2/round4-20261018/

kernel-probes-20261018.txt; admission-race/ (INVESTIGATION.md + 140-pass
logs + suite loop); gates/, mutations/, adversarial/ to follow.

## Status

B: not reproduced in 140 isolated runs (idle+load); suite-loop artifact was
missing CLI binary (now built); continue loop during gates; fix only on
reproduction. D: context-isolated re-attack agent after A/B/C land. Gates
then status revision 113 + commit; ask operator before push.
