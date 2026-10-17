# ADR-013: Workload identity, brokered resources, lifecycle and exposure

Status: implemented as supplementary-qualified increments; frozen v1 rows
remain NOT-YET-EXECUTABLE until independently re-qualified.

## Context

ADR-011 left workload isolation, production MCP integration, database
mediation, credential lifecycle, revocation, exposure and delegation open.
The host now provides setuid `newuidmap`/`newgidmap` (uidmap 4.17.4) and a
`sprime01:100000:65536` subordinate delegation.

## Decisions

1. **Isolated class.** Untrusted ACP agents run under one exclusive
   subordinate host UID/GID (single-ID map via util-linux `unshare`, new
   user/pid/mount/ipc namespaces, net namespace when network is denied),
   with the jail Landlock posture applied by the trusted
   `sea-forge-workload-exec` helper before exec. Workspaces get identity-
   exclusive POSIX ACLs. No shared-UID fallback; `local`/`jail` remain for
   trusted work. Rejected: shared-UID bwrap (fd theft demonstrated),
   `--map-auto` ranges, a new privileged daemon.
2. **Brokered MCP only.** Isolated agents are offered MCP solely as
   `sea-forge-mcp-connect <socket>`; the trusted server serves the socket,
   checks `SO_PEERCRED` against the workload UID, runs the existing broker,
   and keeps upstream processes and credentials outside the workload.
   Configured MCP or database resources with a non-isolated class fail
   closed.
3. **Workload-bound PostgreSQL.** Authority action `database_session`
   (explicit `db_resource`/`db_scope`). Control plane: psql over the Unix
   socket, admin via peer auth only. Per-issuance native role: scope role
   `INHERIT TRUE, SET FALSE`, no admin attributes, connection limit,
   `VALID UNTIL`. PostgreSQL enforces privileges.
4. **Lifecycle.** Renewal re-decides; refusal revokes. Expiry reaper ends
   live backends. Revocation = NOLOGIN, terminate, verify none survive,
   drop (fail closed otherwise). JIT elevation only through the existing
   `grant_after_approval` (SoD, exact action, one use), bounded by the
   approval window. MCP revocation is an acknowledged barrier.
5. **Exposure.** Append-only exposure ledger with lineage; labels only from
   trusted observations (broker-forwarded classified results; labelled DB
   scopes at issuance). The existing matcher requires `permitted_exposure`
   to cover every label of the acting context. Production context:
   `case:<case>`.
6. **Evidence.** Brokered MCP attempts and settlements carry workload UID,
   context, exposure, decision and request digests. RealityTrace bridge per
   ADR-012.

## Round 2 decisions (after the 2026-10-17 adversarial review)

7. **Read posture.** Isolated workloads read only the system trees
   (`/usr`, `/bin`, `/sbin`, `/lib*`, `/etc`, `/proc`, `/dev`) plus
   operator-declared `workload_read_paths`. Each declared path must be
   absolute and canonical. It may not be `/`, a whole home directory, or an
   ancestor of the control-plane root. Every Landlock right of ABI 6 is a
   hard requirement, and abstract UNIX sockets and signals are scoped.
   Rejected: read on `/` with DAC as the only guard (ADV-A05/A06/A07).
8. **Lifetime.** The launcher holds the only write end of a lifeline pipe.
   The helper stays as init of the PID namespace and exits on EOF, and the
   kernel then kills the namespace. The pipe inode is 0444, so a workload
   cannot reopen a second write end. Rejected: a parent-death signal, which
   the user-namespace credential change clears.
9. **Slots.** Lock directory per user and host (`/run/user/<uid>` when it is
   private). A slot is issued only after every process under its UID is
   reaped. Its grants are masked (mode 0700) when the episode ends.
10. **Planned commands.** `execute_command` with class `isolated` runs
    through the same chain. Authority admits only the trusted executable,
    whose directory is the one extra read path.
11. **Expired sessions.** Every issuance first revokes expired broker roles.
    Each role carries `idle_session_timeout`. Approval-bound deadlines come
    from a pure function, and a window that has already closed is refused.
12. **MCP transports.** Only stdio upstreams are representable. HTTP, SSE
    and streamable HTTP cannot be configured, and isolated agents have no
    network to reach one directly.
13. **Exposure.** Any forwarded response of a classified tool labels the
    context, including error payloads. Issued database credentials are
    redacted from persisted transcripts.

## Limits

Remaining after round 2 (candidate revisions, not hidden):

- **State tree DAC.** The control-plane tree is still created 0755/0644.
  Only Landlock keeps it from workloads. Making it 0700 with traverse ACLs
  affects other principals.
- **Pathname UNIX sockets.** Reachability falls to DAC; Landlock ABI 7 has
  no connect right.
- **Database sessions with no launcher.** A busy backend whose launcher is
  dead lasts until the next issuance. Bounding it needs an out-of-process
  supervisor.
- **Kernel surface.** io_uring and nested user namespaces are reachable; no
  seccomp filter yet.
- **Exposure coverage.** Workspace files, artifacts and stdout are not
  exposure-gated.
- **PostgreSQL side channels.** Catalog metadata and LISTEN/NOTIFY remain
  visible.
- **Trust and granularity.** JIT granularity is a database scope role. The
  namespace owner (launcher UID) is trusted.
