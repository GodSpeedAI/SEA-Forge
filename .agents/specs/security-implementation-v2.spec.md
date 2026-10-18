# Security implementation increments against frozen v1

The immutable behavioral authority is
[`tests/security-v1/CONTRACT.md`](../../tests/security-v1/CONTRACT.md) at externally
pinned commit `af41ea2465e38c87dfe86471c33c6b54ea8564e1`.
Its original gap inventory must not change. Implementation ordering and all
mandatory positive/resource-side controls remain binding.

[`ADR-011`](../../docs/decisions/ADR-011-security-enforcement-boundaries.md)
records the initial production interfaces, exact action mapping, trust inputs
and unsupported boundaries. It does not relax any v1 property.

The initial increment adds transport-principal continuity, private capability
authentication, scoped Linux file
observations, and a session-scoped stdio MCP adapter through the existing
authority fabric. It preserves original server configuration and CLI behavior.
The new `mcp_tool_call` policy operation requires explicit `mcp_server` and
`mcp_tool` lists in `boundary_constraints`; both are exact-value constraints.
Its Reserved resource ID is JSON `[server, tool]`, and `request_sha256` binds the
canonical request, including arguments and correlation ID, into the exact grant.

The optional `run_authenticated` entrypoint accepts authority-issued private
connections from a trusted in-process launcher. It refuses public protected
requests. Each capability binds principal, actor and one canonical request,
expires independently at admission and records its committed issuer decision
reference before execution. It does not supply a workload launcher or protect
descriptors from hostile same-UID host processes. The normal CLI retains
host-local authentication with one primary actor per UID. These modes must not
be reported as equivalent trust boundaries.

The operator-approved legacy approval-test migration preserves positive
approval/journal assertions and adds authenticated actor-switch, fresh public
connection and separation-of-duty negative controls. Its private principals
are established by real committed authority grants, not request actor labels.

The operator subsequently authorized migration of three CEP authority controls
and one SFWP case-mutation control whose legitimate multi-actor workflows
depended on same-UID actor substitution. Preserve their original approval,
replay, transition, execution and separation-of-duty assertions. Use the same
production private-channel grant mechanism and add actual actor-switch and
fresh-public-socket attempts with independent journal observations. Reviewed
legacy regression coverage does not qualify isolated workload identity or
discharge any original v1 gap. Candidate hashes and purposes live under
`tests/security-v2/migrations`; qualification still requires a separate
supplementary contract review and freeze.

No authority decision, process exit, response stamp or unexecuted inventory gap
is native-effect evidence. File witnesses cannot establish write causality or
hostile same-UID isolation. Native PostgreSQL tests cannot establish product
credential/session mediation. Trusted-launch stdio MCP tests cannot establish
fresh-connection workload authentication. These distinctions must remain in
reports and settlement records.

Required gates: unchanged pinned v1 verification before and after work;
`python3 tests/security-v1/run.py all --output <new-security-v1-evidence-dir>`;
real PostgreSQL and stdio MCP provisioning; relevant crate regressions;
`just check`, `just test`, `just proof`, `just no-async-kernel`,
`just status-check`, and `graft build`.

Supplementary tests under `tests/security-v2` remain candidates until their
coverage, review, SHA-256 manifest and independent Git pin exist. They cannot
substitute for v1, its missing workload-authentication controls or its remaining
mandatory credential, revocation, exposure and delegation obligations.
