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

## Limits

Planned `execute_command` runtime path has no isolated class yet; Landlock
still grants read on `/` (DAC protects 0600/0700 launcher files only); no
injectable clock; JIT granularity is a database scope role; exposure is not
enforced on workspace artifacts or the operator transcript; other MCP
transports and streaming are unqualified; the namespace owner (launcher UID)
is trusted.
