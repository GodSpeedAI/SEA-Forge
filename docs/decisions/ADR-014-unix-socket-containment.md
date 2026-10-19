# ADR-014: Containing pathname Unix-socket reachability for isolated workloads

- Status: accepted (mechanism C chosen by operator 2026-10-18); amended
  2026-10-19 after the `seccomp_unotify(2)` TOCTOU review (§7)
- Date: 2026-10-18
- Decides: how `connect(2)` / `sendto(2)` / `sendmsg(2)` to **pathname** AF_UNIX
  sockets are confined to an isolated workload's declared trees
- Does not decide: abstract-socket scoping (already enforced, see below)

## 1. Context and problem

An isolated workload (SEC-06) is launched by `workload::isolated_command`:
util-linux `unshare` creates a user namespace (one mapped subordinate UID),
pid/mount/ipc namespaces and — under `NetworkPosture::Denied` — a network
namespace; the trusted `sea-forge-workload-exec` helper then applies a
hard-requirement Landlock ruleset (FS rights at ABI V6, TCP rights,
`Scope::AbstractUnixSocket | Scope::Signal`) and a seccomp deny-list filter
(io_uring, namespace creation), then execs the workload.

Landlock confines **opening** files, but `connect(2)` to a pathname socket is
not an open, and no Landlock right through the running kernel's ABI mediates
it. Round-3 evidence
(`.agents/evidence/security-v2/round3-20261017/unix-socket-reach-before.txt`)
shows an isolated workload connecting to a canary listener **outside every
declared tree** (`outside: workload=connected listener=Some("reached")`);
`crates/sea-forge-sandbox/tests/unix_socket_reach.rs` is `#[ignore]`d on this
limit. DAC is the only remaining barrier, so any host socket whose mode/dir
permissions admit the subordinate UID (e.g. under `/tmp`) is reachable.

Already contained, unchanged by this ADR: TCP (net namespace when denied,
Landlock TCP rights otherwise) and **abstract** UNIX sockets (fresh net
namespace plus `LANDLOCK_SCOPE_ABSTRACT_UNIX_SOCKET` since round 2/3 — a
separate guarantee from pathname sockets). Inherited descriptors are an
fd-hygiene property of the launch path, orthogonal to every candidate below
and required regardless (test list item).

## 2. Host kernel facts (probed/read 2026-10-18, not from memory)

Kernel `6.18.54.1-microsoft-standard-WSL2`, x86_64,
`CONFIG_SECURITY_LANDLOCK=y`, `CONFIG_SECCOMP_FILTER=y`, `CONFIG_USER_NS=y`.

- `landlock_create_ruleset(NULL, 0, LANDLOCK_CREATE_RULESET_VERSION)` returns
  **7** on this host (`.agents/evidence/security-v2/round4-20261018/kernel-probes-20261018.txt`).
- Per current kernel documentation (docs.kernel.org, landlock(7)): through
  ABI 7 Landlock mediates filesystem rights (including `MAKE_SOCK` — creating
  a socket *inode*, not resolving one), TCP bind/connect (ABI 4), ioctl-dev
  (ABI 5), and the abstract-unix-scope/signal scopes (ABI 6; scope flags take
  **no** `landlock_add_rule` exceptions).
- **`LANDLOCK_ACCESS_FS_RESOLVE_UNIX` is ABI 9** (mainline Linux 7.1): it
  restricts `connect(2)` and `sendmsg(2)` with an explicit recipient address
  on pathname UNIX sockets, returning `EACCES`. As an FS access right it is
  grantable per path with ordinary `path_beneath` rules; sockets **created
  inside** the sandbox's Landlock domain remain reachable regardless (which is
  exactly the "workload's own socket in its run tree" allowance). It is
  **not available on this host** (ABI 7 < 9).
- The pinned `landlock` crate (0.4.7) implements up to ABI V7 and has no
  `ResolveUnix` right; using ABI 9 would require a crate upgrade (dependency
  change, operator approval) or raw syscalls.
- The pinned `seccompiler` crate (0.5.0) has no user-notification support
  (`SECCOMP_RET_USER_NOTIF` / listener ioctls absent).
- Empirical probe (same evidence file): `connect(2)` to a socket on a
  **read-only bind mount succeeds**. `MS_RDONLY` does not express "readable
  but not connectable".

## 3. Requirements (from the round-4 charter)

1. A pathname socket outside every declared tree must be refused.
2. A socket under a declared **read** path must be refused unless that tree is
   **explicitly socket-granted**; the declared PostgreSQL socket directory is
   reachable exactly when granted.
3. An abstract-namespace canary stays refused (already enforced; regression
   coverage only).
4. Symlink/bind-mount aliases to a socket outside the trees are refused.
5. If a supervisor is used, a rename/swap race between check and connect is
   refused or detected fail-closed.
6. `sendto`/`sendmsg` on an unconnected datagram socket to a canary is refused.
7. A descriptor obtained before restriction must not leak into the workload.
8. A socket created by the workload inside its own run tree works (legit IPC).
9. Everything runs behind the workload-isolation availability probe; on hosts
   lacking the mechanism, fail closed with NOT-EXECUTABLE/unavailable — never
   a silent degrade.
10. Testing uses only canary listeners the tests create in their own temp
    trees; no enumeration or probing of host sockets.

## 4. Candidates

### A. Landlock `LANDLOCK_ACCESS_FS_RESOLVE_UNIX` (ABI 9)

Handle the right in the isolated-workload ruleset; grant it via `path_beneath`
rules on exactly the socket-granted trees (the PostgreSQL socket directory),
and on nothing else by default. Sockets created inside the domain (the
workload's own, requirement 8) stay reachable by design.

- **Guarantee:** kernel-enforced, race-free, no TOCTOU; semantics match
  requirements 1/2/6 exactly (connect + addressed sendmsg/sendto-on-
  unconnected-datagram are all resolution events).
- **Availability:** unavailable on this host (ABI 7). A pure-ABI-9 design
  fails closed everywhere here — every containment test NOT-EXECUTABLE, which
  proves nothing locally this round. On future kernels (or a WSL kernel
  upgrade) it is the clean mechanism. Crate 0.4.7 lacks it → dependency
  decision or raw-syscall ruleset construction.
- **Residual risk:** none specific; the Landlock model itself (per-thread,
  inherited) already governs the stack.

### B. Minimal mount view (build the workload's mount namespace)

After `unshare --mount`, replace the copied host tree with a scaffold: bind
only `WORKLOAD_SYSTEM_READ`, declared read trees, workspace/artifacts, proc,
`/dev` minimal set; everything else does not exist in the namespace
(`ENOENT`), so requirement 1 holds by non-existence; symlink aliases resolve
inside the view and die (requirement 4). No pathname TOCTOU: mounts are
kernel-atomic and the workload cannot mount (seccomp denies namespace
creation; Landlock denies topology change). Implementation needs the helper
(or a pre-exec setup stage) to hold `CAP_SYS_ADMIN` **in the workload user
namespace** during setup — i.e. mapping a second inner ID (0) alongside 1000,
or a keep-caps path through the uid switch; the workload itself still runs as
unprivileged inner 1000 after setup.

- **Guarantee:** kernel-enforced non-existence for everything not mounted;
  strong for requirement 1.
- **Fatal gap for this charter:** requirement 2. A socket inside a mounted
  declared read tree is connectable — read-only bind mounts do not stop
  `connect(2)` (probed, §2). The mount view cannot express "readable but not
  socket-granted" without layering candidate A or C on top, and grant-by-
  non-mounting contradicts "declared read path must be readable".
- **Availability:** works on this host and wherever mapped user namespaces
  are allowed; NOT-EXECUTABLE on GitHub Ubuntu runners (same as today).
- **Residual risk:** largest blast radius of the candidates — rewrites the
  launch contract (uid mapping, mount sequence, `/dev` and proc provisioning);
  regression risk for legitimate workloads (interpreters, `/dev/null`).
  Inherited descriptors (requirement 7) unaffected — fd hygiene still needed.

### C. Seccomp user-notification supervisor

Extend the workload filter with `SECCOMP_RET_USER_NOTIF` on `connect`,
`sendto`, `sendmsg` (x32 aliases like the existing rules; other architectures
kill as today). A trusted launcher-side supervisor owns the listener fd,
reads each notification, fetches the `sockaddr_un`, and allows only paths
that resolve inside **socket-granted** trees (a new confinement field
alongside `read_paths`); everything else is denied with `EACCES`. Non-UNIX
families (AF_INET/6 under `NetworkPosture::AllowTcpPorts`) pass through to
the existing Landlock TCP rights — the supervisor only mediates `AF_UNIX`
pathname targets; abstract targets (`sun_path[0]==0`) stay with the Landlock
scope + net namespace.

- **Guarantee:** mediation at the only three syscalls that can target a
  pathname socket, with the exact policy granularity of requirement 2
  (read-grant ≠ socket-grant). Works on this host today. `seccompiler` 0.5.0
  has no user-notify support → raw BPF + `SECCOMP_IOCTL_NOTIF_*` via existing
  deps (rustix/libc), or a new dependency (operator approval).
- **TOCTOU (requirements 4/5), two distinct races, both must be addressed:**
  1. *Pathname double-fetch:* the supervisor reads `sun_path` from workload
     memory; the workload can rewrite the buffer before the kernel performs
     its own fetch. Mitigation: fetch once via `process_vm_readv`, validate,
     respond — the window shrinks to supervisor-vs-kernel fetch ordering but
     cannot be eliminated for memory-passed pathnames by any supervisor.
     Compensating control: after allowing, the supervisor re-resolves the
     path (`openat2` with `RESOLVE_*` mirroring connect's walk), `fstat`s
     `S_ISSOCK`, and on mismatch kills the workload — detection with fail-
     closed consequence, not prevention.
  2. *Rename/swap:* the validated socket file is replaced between check and
     connect. Mitigation: validate through an `O_PATH` fd on the final
     component and compare `(st_dev, st_ino)` immediately before responding;
     residual window remains (milliseconds). Compensating control: DAC —
     swapping a file inside a launcher-owned declared tree requires write
     permission the workload UID lacks (trees are granted 0700+ACL); swaps in
     world-writable dirs (`/tmp`) remain possible and are exactly what the
     post-connect re-check catches.
  - Supervisor lifecycle must fail closed: if the listener fd closes (or the
    supervisor dies), the kernel's documented behaviour for outstanding
    notifications must be verified at implementation and pinned by test;
    the supervisor is bounded by the workload lifeline.
- **Residual risk:** the double-fetch window above — honest bound: this
  candidate is *policy-precise but not race-free*; a determined workload in a
  tight loop may attempt the race, with the re-check + kill as the backstop.
- **Upgrade path:** behind the same availability probe, prefer ABI 9
  (candidate A) when the host kernel reports ≥ 9 and fall back to the
  supervisor otherwise, with the fallback logged/asserted, never silent.

### D. DAC / ownership hardening (defence in depth only)

The subordinate-UID model already blocks host sockets that are 0600/0700 or
in 0700 directories. It does not bind requirement 1: world/group-writable
service sockets and socket directories (the `/tmp` class) remain reachable by
any UID. Useful as the layer that makes candidate C's rename/swap race hard
in launcher-owned trees, and as a general backstop; never the guarantee.

## 5. Comparison

| Requirement | A: ABI 9 | B: mount view | C: supervisor | D: DAC |
| --- | --- | --- | --- | --- |
| 1 outside-tree refused | yes | yes (ENOENT) | yes | no |
| 2 read≠socket-grant | yes | **no** (ro-mount probe) | yes | partial |
| 4 symlink/bind alias | yes | yes (inside view) | yes + race caveat | no |
| 5 rename/swap race | n/a (race-free) | n/a (atomic mounts) | shrunk + detect/kill | hardens |
| 6 datagram sendto | yes | per-req-2 gap | yes | no |
| 7 inherited fds | fd hygiene, separate | same | same | same |
| 8 own socket works | yes (in-domain) | yes | yes (own tree granted) | yes |
| 9 fail closed off-host | NOT-EXECUTABLE here | works here, CI NOT-EXEC | works here | works |
| Cost | crate/ABI dependency | launch rewrite | filter+supervisor, raw BPF | minimal |

## 6. Recommendation (operator decides)

**C now, A when available, D always.** The supervisor is the only candidate
that satisfies requirement 2 on this host; its TOCTOU residuals are bounded,
instrumented (re-check + kill) and layered over DAC. Landlock ABI 9 becomes
the preferred enforcement the day the host kernel provides it (probe-gated,
fail-closed switch, keeping the supervisor as logged fallback). The mount
view is rejected as the primary mechanism — its one real strength
(non-existence) is already delivered by Landlock FS rights for everything
except sockets, and it cannot express the read/socket-grant distinction this
charter requires.

Regardless of choice: requirement 7 needs an explicit inherited-fd audit of
the launch path (close-on-exec discipline for everything except stdio and
the lifeline reopen), and requirement 3 gets regression coverage from the
existing net-namespace + abstract-scope posture.

Probes and citations: `.agents/evidence/security-v2/round4-20261018/
kernel-probes-20261018.txt`; docs.kernel.org `userspace-api/landlock.html`
(ABI table, RESOLVE_UNIX, scope semantics); landlock(7).

## 7. Amendment 2026-10-19: the unotify authority model

`seccomp_unotify(2)` warns against userspace policy verdicts followed by
`SECCOMP_USER_NOTIF_FLAG_CONTINUE`: under CONTINUE the kernel re-fetches the
syscall arguments from workload memory and executes the workload's syscall,
so a workload that rewrites its buffer between the supervisor's check and
the kernel's fetch changes what was approved. A post-allow kill cannot undo
an irreversible consequence (a datagram delivered, a connection used), so a
check-then-continue design proves detection, not prevention.

Reviewing the round-4 supervisor against that warning found two defects in
the shipped draft: the allow reply (`error=0, flags=0`) synthesized fake
success without any connection ever existing, and the planned allow path
(CONTINUE) would have been exactly the warned pattern with a userspace
pathname check in front. The amendment replaces the authority model — the
notification mechanism stays as the interception point (nothing else exists
on ABI < 9), but it is no longer the security authority over a raceable
syscall:

1. **Kernel-authoritative targets CONTINUE.** A non-UNIX family falls to the
   Landlock network rights and an abstract AF_UNIX target to the Landlock
   abstract-socket scope; in both cases the kernel enforces its policy
   against the bytes *it* fetches. Whatever the workload races its address
   into, the kernel grants or refuses what actually arrives. This is the
   sanctioned CONTINUE use: the decision is kernel-made, not ours.
2. **Pathname targets and address-less sends are brokered, never
   continued.** The supervisor duplicates the workload's descriptor with
   `pidfd_getfd` (kernel-verified identity: the helper is the parent in the
   same user namespace), performs the operation itself from memory it read
   once (`process_vm_readv` — no kernel re-fetch of workload bytes), pins
   the peer with `connect`, and requires the *kernel-recorded* peer name
   (`getpeername(2)`) to sit under a granted tree before any byte moves.
   The workload is frozen inside the intercepted syscall the whole time.
3. **The residual race is bounded to an empty connection.** A rename raced
   into the check-to-connect window can make the broker reach the wrong
   socket; `getpeername` then disagrees with the validated path, and the
   supervisor kills the workload *before* replying — the workload dies
   without executing anything past the refused call, so no payload can
   follow. The outside peer receives an empty connection, never data; a
   datagram is sent only after the pin verifies. This is prevention for
   data movement, with the honest residual that a raced connection itself
   (zero bytes) is observable to the wrong peer before the kill lands.
4. **Fail closed everywhere.** Oversize or unreadable send payloads refuse
   and kill (1 MiB broker cap); a connect that neither completes nor fails
   within 5 s denies; supervisor death closes the listener and the kernel
   answers `ENOSYS`; the identity pool refuses to launch when no listener
   can be installed (inherited-listener EBUSY).

With this model the answer to "is the fallback secure despite unotify's
warning" is: the warning's failure mode (raced re-fetch changing the
approved operation) no longer applies to the decided targets, because the
kernel never re-fetches — there is no continuation of workload syscalls
carrying userspace verdicts. On ABI ≥ 9 hosts the pure-Landlock path has no
notification at all and remains the preferred enforcement.
