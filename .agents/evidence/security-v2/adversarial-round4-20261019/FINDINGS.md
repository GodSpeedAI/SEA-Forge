# Adversarial review, SEC-06 pathname Unix-socket containment (round 4)

- Reviewer: independent adversarial reviewer (no access to implementation notes)
- Date: 2026-10-19
- Target: `security/round4-20261018` worktree; commits `dcb27ad` (brokered
  supervisor) and `2b8d2fb` (kernel-security CI job)
- Scope: `crates/sea-forge-sandbox/src/socket_policy.rs`,
  `src/bin/sea-forge-workload-exec.rs`, `src/workload.rs`, `src/isolated.rs`,
  `src/syscall_filter.rs`, `crates/sea-forge-server/src/delegation.rs`,
  `tests/unix_socket_reach.rs`; normative basis ADR-014 (incl. 2026-10-19
  amendment) and root `AGENTS.md`.
- Method: full code walk of every listed file, hand verification of the BPF
  program and of the vendored seccompiler 0.5.0 codegen, kernel-man-page
  verification of the claims the design leans on, and attempted live
  execution of the charter battery.

Live-execution caveat, stated up front: this reviewer's session is itself
seccomp-sandboxed (`seccomp(2)` is intercepted: EBUSY/SIGSYS on direct
probes), so the battery could not be exercised from here. Inspection of the
test harness shows it would not have exercised the broker on this host
anyway: when `WorkloadIdentityPool::from_host()` fails, `scenario()` in
`tests/unix_socket_reach.rs` calls `std::process::exit(0)`, killing the whole
test binary with exit status 0 and **no test-result line**, which plain
`cargo test` reports as success. The author's own status record
(`.agents/CURRENT_STATUS.yaml`, round-4 entry) admits: "All claims LOCALLY
TESTED except the supervisor's end-to-end, which this agent harness
structurally cannot run." **No finding below rests on a passing test; every
verdict is from code reading plus kernel-documented semantics.**

---

## Finding 1 — The CONTINUE gate in front of the broker is itself a double-fetch; a raced family flip executes an unmediated pathname connect

- Surface: 1 (notification TOCTOU and continuation semantics)
- Severity: critical
- Verdict: BYPASS (semantics certain from kernel documentation and code; race
  win needs a live-host demonstration)

The supervisor answers `Target::Passthrough` with
`SECCOMP_USER_NOTIF_FLAG_CONTINUE` (`socket_policy.rs` `supervise`, the
`Ok(None)` arm). The classification is made from bytes the supervisor read
with `process_vm_readv` (`target_of`). Under CONTINUE the kernel does not use
the supervisor's snapshot: seccomp_unotify(2)'s documented warning is that the
target can change the pointed-to memory between the supervisor's inspection
and the kernel's own fetch, and the kernel then executes the syscall against
what *it* reads.

ADR-014 §7.1 justifies CONTINUE by claiming "Whatever the workload races its
address into, the kernel enforces the policy against the bytes it actually
fetched." That premise is false in exactly one case, and it is the case this
ADR exists to close: on a kernel with Landlock ABI < 9 the kernel has **no
policy at all** for an AF_UNIX **pathname** connect/send. Every other outcome
of the race is mediated (AF_INET/6 → Landlock TCP; abstract AF_UNIX → scope +
netns), but the outcome "raced into pathname AF_UNIX" is unmediated.

Attack narrative (two threads, standard double-fetch):

1. Thread A loops: `connect(fd, sa, 110)` where `sa` holds
   `{AF_INET, port, 127.0.0.1}`.
2. Thread B spins on the page holding `sa`: write `{AF_INET ...}`, fence,
   write `{AF_UNIX, "/tmp/canary.sock"}`, repeat (two cached stores, no
   syscalls in the loop).
3. For each of A's calls the supervisor reads the buffer, sees AF_INET,
   replies CONTINUE. The kernel wakes, re-fetches `sa`, and — whenever B's
   AF_UNIX store landed in the window — executes an AF_UNIX **pathname**
   connect. Nothing on an ABI-7 kernel mediates it: not the FS ruleset, not
   the TCP rights, not the abstract scope, not DAC against a 0666 canary.
4. The workload then `write(2)`s on the now-connected descriptor. `write` is
   not monitored at all; the data moves to the canary.

The window is the span from the supervisor's `process_vm_readv` through the
`ioctl(SECCOMP_IOCTL_NOTIF_SEND)` reply to the kernel's `copy_from_user` —
microseconds, but B re-arms the buffer millions of times per second and A can
issue thousands of connects per second; the classic win-rate arithmetic for
this race applies.

Sub-case, same surface, cheaper for the attacker: when the supervisor's read
**fails**, `target_of` returns `Target::Passthrough` and the syscall is
CONTINUEd anyway (`socket_policy.rs`: sendmsg msghdr read failure at the
`let Some(header) = ... else { return Target::Passthrough }` arm; sockaddr
read failure at the `let Some(sockaddr) = ... else { return
Target::Passthrough }` arm). Thread B can force the supervisor's read to fail
(`munmap`) and re-map the same address (`mmap MAP_FIXED`) pre-loaded with a
pathname sockaddr before the kernel's fetch. A read failure means the
supervisor decided nothing; continuing is policy-free. These two arms must
refuse and kill.

Everything the ADR amendment says about the brokered path (single fetch, no
kernel re-fetch, getpeername pin) is correct — but the Passthrough gate sits
in front of the broker and it *is* the warned pattern: a userspace decision
(read the family, choose CONTINUE) followed by a kernel re-fetch, with an
unmediated outcome available inside the window.

Minimal fix direction: there is no sound CONTINUE for *any* family on an
ABI < 9 host, because every classification is made from raceable bytes and
"raced into pathname AF_UNIX" has no kernel backstop. Either broker every
monitored syscall (the broker already performs connects itself from a single
fetch; extend that to non-UNIX families and verify with getsockname), or
restrict CONTINUE to syscalls whose every possible fetched outcome is
kernel-mediated — which on ABI < 9 is the empty set — and refuse+kill the
rest. Also: both `Target::Passthrough` fallbacks on read failure must become
refuse+kill regardless.

## Finding 2 — `sendmmsg(2)` is not trapped: deterministic datagram exfiltration to any pathname socket

- Surface: 1 (BPF trap set vs decision paths) / 3 (datagram handling)
- Severity: high
- Verdict: BYPASS (no race required; code-walk conclusive; destination set
  bounded by DAC in the workload user namespace)

The ADR states the design mediates "the only three syscalls that can target a
pathname socket." That is false: x86_64 has a fourth address-carrying socket
syscall, `sendmmsg(2)` (nr 307; x32 alias `0x40000133`).

- The supervisor BPF traps only `{42, 44, 46}` and their x32 aliases
  (`MONITORED` in `socket_policy.rs`).
- The deny filter (`syscall_filter.rs`) refuses only io_uring, setns,
  unshare/clone with namespace flags, and clone3; everything else — sendmmsg
  included — gets its mismatch action, which is `Allow` for the native
  architecture.
- On ABI < 9 no Landlock right mediates AF_UNIX pathname resolution.

Attack narrative: workload creates `socket(AF_UNIX, SOCK_DGRAM)`, builds one
`mmsghdr` whose `msg_hdr.msg_name` is `{AF_UNIX, "/dev/log"}` (or
`/run/systemd/journal/socket`, or any world-writable service datagram socket
admissible to the subordinate UID — DAC is the only barrier the ADR itself
calls insufficient), and calls `sendmmsg`. The kernel executes it
unmediated; the datagram lands. Requirement 6 of the charter
("sendto/sendmsg on an unconnected datagram socket to a canary is refused")
is evaded by an equivalent operation. The same hole also permits SCM_RIGHTS
fd passing to outside sockets (control data is dropped only on brokered
sends).

Only the datagram path is open — stream sockets still require a brokered
`connect`, and seqpacket unaddressed sends stay pinned to a verified peer —
but datagram exfiltration/log injection is exactly the data movement the
containment claims to prevent.

Minimal fix: add 307 and `0x40000133` to `MONITORED` and handle it in
`target_of`/`broker_send` (walk the mmsghdr vector; each entry's msg_name is
classified like sendmsg's), or refuse+kill sendmmsg outright if no workload
needs vectorized sends.

## Finding 3 — The supervisor thread inherits its own USER_NOTIF filter and deadlocks on its first brokered operation: every positive control fails

- Surface: 6 (supervisor lifecycle), with direct consequences for surface 1
- Severity: critical
- Verdict: BYPASS of the positive controls (feature dead-on-arrival;
  containment-wise fail-frozen, no data movement)

Execution order in `sea-forge-workload-exec`:

1. `socket_policy::apply` installs the USER_NOTIF filter on the **main
   thread**.
2. `syscall_filter::restrict_current_thread` adds the deny filter, same
   thread.
3. `std::thread::spawn` creates the supervisor thread — seccomp filters are
   inherited by every task created after installation, threads included.
4. The workload child is spawned and execs (inherits both filters).

The supervisor's broker therefore executes `libc::connect` (in
`pin_and_verify`) on a thread that itself carries the USER_NOTIF filter. nr
42 from the supervisor's own thread matches the BPF, so the broker's connect
is trapped and a second notification is queued on the same single listener —
whose only reader is the thread now frozen inside that very connect
(USER_NOTIF syscalls sleep until answered). Permanent deadlock on the **first
granted pathname operation**: PostgreSQL connect, MCP socket connect, own-tree
connect — the exact positive controls of the charter.

The code's own documentation knows the correct shape and does not follow it:
`install_supervisor_filter`'s doc says "serve it from a thread spawned
*before* this call, which stays unfiltered"; the helper instead says "every
syscall it needs is allowed by the inherited filters" — true for ioctl,
`process_vm_readv`, poll, open, kill, and false precisely for connect/sendto/
sendmsg, the trapped set. The helper must pass the listener to a thread
spawned before `socket_policy::apply` (a channel handoff), or move the
listener into a short-lived grandchild forked before installation.

Corroboration that this never worked end-to-end: the author's status record
explicitly exempts the supervisor's end-to-end from "LOCALLY TESTED", and the
battery self-skips via `process::exit(0)` when the identity pool is
unavailable (see controls section). Had a granted connect ever executed, the
workload would have hung until the launcher timeout.

Containment-wise this fails frozen (the workload freezes; the launcher's
lifeline/timeout still reaps the namespace), so it is a availability and
claims-integrity defect rather than an exfiltration path — but it means the
central design claim ("brokered prevention of data movement") is currently
unproven *and* unoperable, and the rename-swap compensating kill in
`broker_refuse` (which correctly does NOT call connect before refusing) is
the only brokered path that can ever run.

Minimal fix: spawn the serving thread before `apply()` and hand it the
listener fd over a channel (the socket_policy docstring already prescribes
this), or fork a dedicated supervisor process pre-filter. Then add a test
that asserts a granted connect completes with data delivered — currently
missing precisely because it would have exposed this.

## Finding 4 — Rename/symlink swap races: HELD along the brokered path, with an honest zero-byte residual and three soft spots

- Surface: 2
- Verdict: HELD (for data movement), PARTIAL (behavioral)

What holds, and where:

- Pathname aliases: `target_of` canonicalizes (`absolute.canonicalize()`), so
  a symlink under a granted tree pointing outside resolves outside and is
  refused before any kernel operation. The charter test exercises this.
- Swap between validation and the broker's connect: the broker performs its
  own connect and then requires the kernel-recorded peer (`getpeername`) to
  sit under a grant (`pin_and_verify` → `peer_under_grant`). A socket swapped
  in from outside a grant self-reports its outside bind path and is refused
  with kill-before-reply (`broker_refuse` kills before the ENOENT-prone
  reply; the workload is frozen in the intercepted syscall and executes
  nothing past it). The residual — the wrong peer observes one empty
  connection before the kill lands — is exactly what ADR-014 §7.3 discloses.
  Data cannot follow because bytes move only after the pin verifies.
- Swap of one granted-tree listener for another passes verification, but both
  are workload-created (DAC: grants are launcher-owned 0700+ACL trees; the
  workload uid cannot create entries in `/run/postgresql`-class dirs), so no
  privilege crosses.

Soft spots:

1. **The O_PATH `(st_dev, st_ino)` check is a no-op.** `socket_inode`
   computes the inode pair "so a swapped entry cannot redirect the check"
   (comment) but the pair is never compared to anything — it cannot be;
   `getpeername` yields a path, not an inode. The actual and sole backstop is
   `getpeername`. Harmless, but the comment claims a defense that does not
   exist; the doc and any audit built on it should say so.
2. **Relative/symlink-text binds break the peer check (fail-closed).** Linux
   stores the bind address verbatim; a listener bound to `s.sock` (relative)
   or via a symlink alias makes `getpeername` return that non-canonical
   string, which fails `starts_with(canonical_grant)` → refuse+kill. A
   legitimate workload that binds its own tree socket relatively is killed on
   the client's connect. The own-tree test only uses absolute binds, so this
   is untested. Security: fail-closed. Function: a realistic positive-control
   gap.
3. **Unnamed/abstract peer records** return `None` from `peer_bound_path` and
   are refused — correct, since a pathname connect's peer always has a
   pathname bind; noted for completeness since the ADR asks.

## Finding 5 — Abstract/None classification, dissolve semantics, and the sendmsg edge cases

- Surface: 3
- Verdict: PARTIAL (mostly sound; the unsound parts are covered by Findings 1
  and 2; one behavioral lie and one misclassification noted)

- Abstract → `Passthrough`: per ADR-014 §7.1 the kernel (Landlock abstract
  scope + netns) is the authority. Sound *as a classification* — but the
  classification bytes are raceable, which is Finding 1; an abstract target
  raced into a pathname target during CONTINUE is unmediated on ABI < 9.
- `Target::None` for address-less sends: sound, because every route to a
  connected descriptor passes through the broker (all `connect`s trapped;
  io_uring denied; inherited fds closed by helper hygiene; socketpair and
  accept are in-process/by-design), so the pinned peer was verified at
  connect time. `broker_connected_send` sends copied bytes on a fresh
  `pidfd_getfd` dup, so no workload memory is dereferenced by the kernel.
- AF_UNSPEC dissolve: `unix_dgram_connect` handles `sun_family == AF_UNSPEC`
  by disconnecting on current kernels; the broker's 2-byte `AF_UNSPEC`
  connect is sufficient and works. Even where it failed, the surviving pin
  points only at a grant-verified peer, so later unaddressed sends remain
  in-policy (the code's comment is accurate).
- **`connect(fd, NULL, 0)` is misclassified.** The NULL-address connect (the
  portable dgram dissolve) yields `Target::None` (`address == 0`), which
  routes to `broker_connected_send`: `broker_payload` sees base 0 → empty
  payload → the broker *sends a zero-length datagram* to whatever peer the
  socket has (or returns EACCES → kill if unconnected) and replies success.
  The dissolve never happens. Bounded (any peer was broker-verified), but it
  is a semantic lie and a kill hazard for legitimate C code.
- **`iovlen > 1024` in a brokered sendmsg silently sends an empty datagram
  and reports success** (`broker_payload` returns `Ok(Vec::new())`).
  Fail-safe (nothing of the payload moves, destination still pin-verified)
  but the workload is told its data was sent. Should refuse+kill like the
  oversize path does.
- `BROKER_MAX_PAYLOAD` (1 MiB) on the sendto path: refuse+kill, fail-closed,
  as documented. Payload partial reads can shorten a datagram; shorter to a
  verified destination is harmless.

## Finding 6 — Descriptor hygiene and transfer: HELD

- Surface: 4
- Verdict: HELD

- `close_range(3, u32::MAX, CLOSE_RANGE_CLOEXEC)` runs before any
  restriction, threading, or parsing; the lifeline is deliberately re-opened
  by path afterwards with CLOEXEC (re-opened description is cloexec by
  Rust's `File::open`). The preconnected-descriptor charter test covers the
  invariant end-to-end.
- The listener fd itself is created *after* that sweep, so the helper relies
  on the kernel setting close-on-exec on NEW_LISTENER fds; seccomp(2)
  guarantees "The close-on-exec flag is set for the file descriptor." Held —
  but relying on a kernel default for the single fd whose theft would let the
  workload answer its own notifications (self-CONTINUE = total bypass) with
  no explicit close in the child is thinner than it should be; an explicit
  `close` (or `CommandExt` fd closure) pre-exec would make the invariant
  local rather than inherited.
- `pidfd_getfd` fd-number races: the notification freezes the *issuing*
  thread, but sibling threads can close/dup2 the slot between RECV and
  `pidfd_getfd`, so the supervisor may duplicate a different file. This is
  bounded: the broker never sends on an unverified dup for addressed
  operations — it *connects the dup itself* to the validated path and
  requires `getpeername` before any byte moves, so a swapped-in dup can only
  (a) fail to connect (EISCONN/ENOTSOCK → refuse+kill), or (b) be re-pinned
  to the granted path and carry the copied payload there. Data to an outside
  destination is unreachable through this race. For `Target::None` sends the
  dup is used without re-verification, but the only connected descriptors in
  the process are broker-created or in-process (Finding 5), so the worst case
  is confusion between two workload-owned, policy-legal peers.
- A shared-open-file-description side effect: the broker flips O_NONBLOCK on
  the dup (same OFD as the workload's fd) and restores the saved flags; a
  concurrent flag change by a sibling thread can be clobbered in the window.
  Low, behavioral.
- `BROKER_MAX_PAYLOAD`: covered in Finding 5 — fail-closed on the sendto
  path, silent-empty on the sendmsg iovlen path (fix that one).

## Finding 7 — Namespace and filesystem-boundary bypass: HELD via getpeername, but the mount surface is real and the doc claims are false

- Surface: 5
- Verdict: HELD (for data movement), with corrections the record needs

- Shared mount namespace: verified. `unshare --mount --mount-proc` happens in
  the launcher chain before the helper starts; helper and exec'd workload
  share that namespace, so the supervisor's `canonicalize`/
  `read_link(/proc/<pid>/cwd)`/O_PATH views are the workload's views. The
  pidns-relative pid in the notification matches the `/proc` the supervisor
  reads.
- The workload **can** change mount topology: `unshare -r` leaves the helper
  (and thus the workload) holding CAP_SYS_ADMIN in the new user/mount
  namespace, and the deny filter refuses namespace *creation* and `setns`,
  not `mount(2)`/`move_mount(2)`/`open_tree(2)`/`fsopen(2)`. The claim in
  `syscall_filter.rs`'s module doc — "Landlock ... refuses mount-topology
  changes even inside a namespace the workload creates" — is false: through
  ABI 7 Landlock has no mount hook (landlock(7) lists mount among the
  unrestricted actions). So bind-mount aliasing (e.g. mounting a `/tmp`
  canary socket over `workspace/x.sock`, or over the postgres socket) is
  available to a determined workload.
- When it is tried, `getpeername` is what catches it: the broker's connect
  resolves through the mount to the real listener, whose kernel-recorded bind
  path is outside every grant → refuse+kill before any byte. The O_PATH
  fstat does not help (Finding 4.1 — the inode pair is unused). Verdict for
  data movement: held, by exactly one mechanism, kill-before-reply.
- The canonicalize → O_PATH → broker-connect chain is a multi-step TOCTOU,
  and the code comment overstates its protection; the design is honest only
  when described as: prefix/policy gate (canonicalize) + kernel-verified
  peer pin (getpeername). Recommending the record say that, and that
  `socket_inode`'s job is only "is it a socket inode at all".
- Relative `sun_path`: resolved against a `/proc/<pid>/cwd` snapshot and
  then connected as an absolute path by the broker; the frozen thread's
  syscall is never executed, so the kernel's notion of the workload's cwd
  never races the decision. The snapshot's staleness is bounded by the
  getpeername pin. Sound.

## Finding 8 — Grant construction and consistency: HELD, two notes

- Surface: 8
- Verdict: HELD, with one medium design note and one unexercised invariant

- Production grants (`delegation.rs`): `runs/<run>/mcp` (only when MCP
  servers exist) plus each PostgreSQL resource's `socket_dir`, all passed
  through `validate_socket_grants(&granted, &[root])` — absolute, canonical,
  and not containing the control-plane root. `isolated.rs` grants
  `socket_paths: []` (default-deny). Read grants and socket grants are
  independent dimensions, as required. Consistent with the kernel/supervised
  reachability: a granted dir is exactly reachable, a read-only declared tree
  is not.
- Helper-side implicit grants (workspace+artifacts appended to
  `socket_reach` unconditionally): sound for outbound reach — any socket
  under those trees was created by the workload itself (the trees are
  launcher-owned 0700+ACL, workload is the only other ACL principal), so the
  grant conveys no new source of sockets. Connecting to its own sockets is
  the charter's allowance. No escalation found from "bind your own, then...":
  self-delivered data stays inside; peers must still self-report a granted
  bind path.
- Medium note — **reverse impersonation via the MCP dir**: the workload is
  granted a *write* ACL on `runs/<run>/mcp` (`grant_exclusive_access`), so it
  can unlink/rename the broker's `<server>.sock` entry and bind its own
  listener at the same path. The host-side `sea-forge-mcp-connect` bridges by
  path; a subsequent connect lands in workload-controlled code, exposing
  whatever the bridge sends (request context, upstream credentials if the
  bridge carries any) and allowing forged tool responses into the run. The
  forward direction (workload → broker) is gated by `serve_workload_gated`,
  but no reciprocal workload-side verification of the broker's identity was
  found. If the bridge does not pin the broker (e.g. via `SO_PEERCRED` check
  against the launcher uid, or binding the broker socket before the ACL grant
  and dropping workload write on the dir), this is a workload→control-plane
  integrity crossing enabled by the grant plumbing. Needs a design decision,
  not necessarily code in this module.

---

## Positive controls (must-keep-working) — present in code and tests, none ever executed

| Control | Test | Status |
| --- | --- | --- |
| Granted-tree connect with payload (`requirement 2` reachable case) | `pathname_sockets_outside_declared_trees_are_unreachable`, granted leg asserts `connected` + `reached` | Never executed on this host; per Finding 3 it would deadlock at the broker's first connect |
| Workload's own-tree bind/connect (`requirement 8`) | `workload_socket_in_own_tree_stays_usable` | Same deadlock exposure; also only exercises absolute binds — a relative bind hits Finding 4.2's kill |
| Run-tree MCP socket reach | no dedicated test; production path `delegation.rs` | Unexercised; same broker path |
| Abstract canary connect is refused (`requirement 3`) | `abstract_namespace_canary_is_refused` (CONTINUE path — no broker connect) | Never executed here; would exercise the CONTINUE arm that Finding 1 shows is raceable |

## Negative controls (must-fail) — present and correctly asserted, same execution caveat

Outside-tree connect (req 1), declared-read-but-ungranted connect (req 2),
symlink alias to outside canary (req 4), rename-swap race with payload
delivery asserted impossible (req 5), datagram sendto to outside canary
(req 6), preconnected smuggled descriptor (req 7) — all present with
listener-side receipt assertions, not just client-side error strings. Gaps:

- **No `sendmmsg` negative test** (Finding 2 — would fail today).
- **No granted-connect-with-data test that must complete** — the single most
  important test for the broker (Finding 3 — would hang today).
- No bind-mount-alias scenario (Finding 7; the workload *can* mount, and the
  compensating kill should be pinned by test).
- No relative-bind own-tree scenario (Finding 4.2).
- Rename-swap test's assertion set allows the workload to be killed
  (`said.is_empty()` accepted) — correct — but on this design the granted leg
  deadlocks rather than proceeding, so the test could never have passed
  against the shipped helper.

## Test-integrity note

`tests/unix_socket_reach.rs` `scenario()` calls `std::process::exit(0)` when
`WorkloadIdentityPool::from_host()` fails (guard: only when
`SEA_FORGE_REQUIRE_WORKLOAD_ISOLATION` is unset). This kills the whole test
binary mid-run with status 0 and no result summary; plain `cargo test`
reports success. Combined with the per-host availability of the identity
pool, the charter battery can silently degrade to a no-op on any host. The
CI `kernel-security.yml` job's NOT-YET-EXECUTABLE discipline is the right
pattern; the local battery should use libtest's skip/report path (return
early per test) so an incapable host yields explicit, visible NOT-EXECUTABLE
outcomes rather than a green-looking silence.

## Overall verdict

Does the brokered supervisor prevent unauthorized pathname-socket data
movement? **As designed on paper for the brokered path: yes; as shipped: no.**
The broker's core — `pidfd_getfd` duplication, a single `process_vm_readv`
fetch the kernel never re-reads, a self-performed connect, a kernel-recorded
`getpeername` pin required before any byte moves, and kill-before-reply on
disagreement — survived every direct attack this review could construct: I
found no way to move data to an outside pathname socket through a monitored
and brokered syscall, and the races reachable against it (fd-slot swaps,
validate-to-connect renames, mount aliases, shared-OFD flag flips) are all
bounded to outcomes short of data delivery — an empty connection observed by
the wrong peer, or a killed workload — exactly the residual ADR-014 §7.3
discloses.

But the guarantee is not what runs. Three defects break it in operation:
(1) the supervisor thread inherits its own filter and deadlocks on the first
brokered operation, so the positive controls cannot work at all and the
mechanism has never executed end-to-end anywhere; (2) `sendmmsg(2)` is
simply not trapped, giving a race-free datagram exfiltration path to any
DAC-admissible pathname socket today; (3) the CONTINUE classification gate is
itself a double-fetch whose raced outcome — a pathname AF_UNIX target on an
ABI < 9 kernel — has no kernel backstop, so the amendment's central
justification ("whatever the workload races its address into, the kernel
enforces its policy") is wrong for precisely the case this ADR exists to
close. Against those, residual races *along the brokered path* are indeed
bounded to consequences short of data delivery; the residual along the
CONTINUE path is not (the kernel executes the raced syscall in full), and
that distinction must replace the current blanket claim in the ADR before
the next round.

Recommended order of repair: fix the helper's supervisor-thread ordering and
add the granted-connect-completes-with-payload test (Finding 3); trap and
broker-or-refuse `sendmmsg` (Finding 2); eliminate CONTINUE for monitored
syscalls on ABI < 9 hosts or narrow it to provably-kernel-mediated outcomes
and convert both read-failure fallbacks to refuse+kill (Finding 1); then the
record corrections (no-op inode check, false mount claim, relative-bind
behavior) and the mcp_dir reverse-impersonation decision.
