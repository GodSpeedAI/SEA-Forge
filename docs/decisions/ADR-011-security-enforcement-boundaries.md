# ADR-011: Private authentication, resource observations and MCP mediation

Status: implementation increment; broader frozen obligations remain open.

## Context

Frozen security v1 binds observable SEC-01 through SEC-12 behavior. Its baseline
is commit `af41ea2465e38c87dfe86471c33c6b54ea8564e1`; its files and gap inventory
remain immutable. Passing an executable subcase does not discharge its gaps.

## Decisions

1. Bind a direct Unix connection to the transport principal of its first
   successfully resolved protected request. A configured gateway retains one
   transport principal while speaking for different allowlisted users. Roles
   remain authority checked per request. This closes connection substitution;
   it does not authenticate distinct workloads sharing a UID on fresh sockets.
2. Project the authority decision's existing `workspace_root` context into
   authorized-invocation execution constraints. Admission opens and retains the
   workspace directory through a descriptor walk from the trusted host root,
   requiring an absolute path and refusing every symlink component. File-effect adjudication observes regular-file bytes
   through that descriptor, refusing symlink components and escapes. Linux procfs
   is required; unavailable observation and files exceeding 64 MiB reject.
   Claimed digests must match actual bytes. Nested file claims receive the same
   check. Settlement retains digest mismatch evidence and rejects such claims.
3. Add `sea-forge-mcp <trusted-config.json>` as a separate, session-scoped stdio
   adapter in the existing server edge crate. Preserve the original server CLI.
   Config declares root, workspace, policy, actor, server, upstream argv and an
   explicit upstream environment. No ambient environment is inherited upstream.
4. Extend the existing authority fabric with Reserved `mcp_tool_call` actions.
   Declarative rules require `mcp_server` and `mcp_tool` boundaries. Resource ID
   is the JSON array `[server, tool]`, avoiding delimiter ambiguity. Parameters
   carry server, tool and SHA-256 of the entire canonical request. Arguments
   are not copied into ledgers. DomainForge sees the existing resource projection;
   OPA sees the existing Reserved action projection including these parameters.
5. Parse client JSON once with recursive duplicate-key rejection, including
   escaped equivalent keys. Validate every member of a batch before dispatch.
   Discovery filtering supplements per-call enforcement. Evaluate, commit,
   mint and consume an exact ActionGrant before forwarding the same parsed
   request. Unknown methods and invalid frames do not reach upstream handlers.
6. Keep JSONL authoritative. Record decision, attempted call, forward attempt,
   response digest and settlement with linked ledger references. A transport
   failure after a forward attempt cannot claim absence of effects. An upstream
   response is labeled as such and is not presented as native resource proof.
7. Offer `run_authenticated` for a trusted in-process launcher. Its public
   listener refuses protected requests; authenticated connections arrive only
   through the launcher's private channel. `AuthenticatedConnection::new`
   consumes a committed `identity_minting` grant binding the principal, actor
   and exact canonical request. The server rejects actor substitution,
   delegation, reuse and expiry, including expiry while awaiting admission.
   Admission commits the original issuer decision reference and request hash
   before invoking the handler. Legacy host-local mode chooses one primary
   configured actor per UID; a second socket cannot select a secondary actor.
8. Migrate only the conflicting legacy approval positive control with operator
   authorization. Its old success condition let the submitter impersonate the
   approver by changing `actor_id` on one socket. The replacement uses two
   independently issued private capabilities through the production authority
   and server APIs. It preserves successful approval and journal attribution,
   verifies issuer/admission ledgers, and checks same-connection substitution,
   fresh public-socket impersonation and authenticated self-approval refusal.
   No frozen v1 test or expected result changes.

## Limits and remaining obligations

The local file witness proves bytes at observation time, not write causality.
Directory replacement, hardlinks and concurrent writers need stronger observer and
isolation contracts before claiming hostile same-UID host protection. Existing
ordinary execution paths are not all routed through this observation API.

The MCP adapter assumes the operator controls its launch/configuration and
upstream. Stdio does not separate arbitrary same-UID host processes. Workload
launcher integration, protected credentials, renewal, scoped elevation, active-session
revocation, trusted exposure propagation and bounded delegation are separate
mandatory work. Other MCP transports, progress/streaming notifications and
resource-side effect provenance beyond the recorded response are unsupported.
The shipped server entrypoint still uses host-local mode. The private API is
exercised by integration tests, but no production workload launcher delivers
its endpoints yet. A private capability authenticates its holder, not an OS
process. Descriptor theft and issuer access must be prevented independently;
the API does not establish cryptographic separation of same-UID workloads.

The PostgreSQL role fixture independently proves native permissions only; it
does not implement or prove SEA-Forge credential issuance or session mediation.
OpenShell, Landlock, capability memory and existing ledger/FTS5 ownership remain
unchanged. No new dependency or production service is introduced.

## Verification

Use the original pinned v1 verifier and runner unchanged, real unrestricted MCP
upstream and real disposable PostgreSQL. New implementation tests exercise the
scoped reader and file-effect adjudicator. Versioned supplemental contracts must
be independently reviewed, hashed and pinned before being declared frozen.
The operator approved the legacy migration while requiring genuine independent
authentication and unchanged frozen v1 behavior. Passing that migrated test is
limited to private-channel authentication and does not discharge isolation.
