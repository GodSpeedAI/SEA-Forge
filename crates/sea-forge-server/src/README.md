# sea-forge-server — src

## Overview

Implementation of the `sea-forge-server` crate: SEA Forge concurrent case
server (spec-full §10.3, §11.1, M3). The server listens on a Unix domain
socket (or a Windows named pipe), accepts NDJSON requests, and dispatches
case runs as subprocesses via `spawn_blocking`. Kernel logic stays
synchronous; Tokio lives only here (`src/lib.rs` module doc). This module tree
holds the request protocol, server state, configuration, identity/peer
authentication, delegation execution, governed boundary contracts (CEP),
MCP serving, Postgres session brokering, and the SFWP view layer.

## Interface

- Core server (`lib.rs`):
  - `run(config: ServerConfig)` — `lib.rs:run`;
    `run_authenticated(...)` — `lib.rs:run_authenticated`.
  - `handle_request(request, state)` — `lib.rs:handle_request`;
    `handle_request_as(...)` — `lib.rs:handle_request_as`.
  - `Request` enum (verb-tagged NDJSON requests; SFWP verbs are additive
    variants per ADR-003) — `lib.rs:Request`.
  - `ServerState`, `CaseEntry`, `ReloadError` — `lib.rs:ServerState`,
    `lib.rs:CaseEntry`, `lib.rs:ReloadError`.
  - `SubmitPayload` — `lib.rs:SubmitPayload`.
- Configuration: `config.rs` — `ServerConfig` (`config.rs:ServerConfig`),
  `GatewayConfig`, `SupervisorConfig`, `CepWorldConfig`, `CepAuthorityConfig`,
  `CepRealityTraceConfig`, `CepSettlementConfig`,
  `MAX_PRIOR_IDENTITIES_PER_WORLD`, plus `resolve_cell_root` and
  `resolve_socket_override` re-exported from `lib.rs`.
- Identity and peer authentication: `identity.rs` — `PeerIdentity`
  (Unix UID / Windows variants), `ActorClaim`, `ResolvedActor`,
  `current_principal`, `current_uid`, `role_wire_name`.
- Case dispatch: `case_dispatch.rs` — `submit` (`case_dispatch.rs:submit`),
  `execute_agent` (`case_dispatch.rs:execute_agent`),
  `execute_sandbox` (`case_dispatch.rs:execute_sandbox`),
  `commit_item_criteria`, `record_completion`.
- Delegation: `delegation.rs` — `execute`, `execute_with_control`,
  `execute_with_permission_broker` (`delegation.rs:execute`,
  `delegation.rs:execute_with_permission_broker`); types
  `DelegationRequest`, `DelegationResult`, `DelegationEpisodeContext`,
  `AcpApprovalBroker`. Audit: `delegation_audit.rs` —
  `record_delegated_request` (`delegation_audit.rs:record_delegated_request`),
  `DELEGATION_AUDIT_LEDGER`, `DELEGATED_REQUEST_RECORD_KIND`,
  `DelegatedRequestRecord`.
- Governed CEP boundary contracts:
  - `governed_work_ingress.rs` — `accept_verified_governed_work_request`,
    `accept_governed_work_request` (`governed_work_ingress.rs:accept_governed_work_request`),
    `EVENT_TYPE = "GovernedWorkRequest"`, `GovernedWorkIntent`.
  - `governed_execution_boundary.rs` — `NAMESPACE = "agentic_capability_loop"`,
    `AUTHORIZED_INVOCATION`, `EXECUTION_OBSERVATION`
    (`governed_execution_boundary.rs:AUTHORIZED_INVOCATION`).
  - `governed_settlement_return.rs` —
    `evaluate_operational_settlement`, `validate_operational_settlement_wire`,
    `OPERATIONAL_SETTLEMENT`, `REQUIRED_FIELDS`, `SettlementEvaluation`,
    `OperationalSettlementReturn`.
  - `cep_authority.rs` — `resolve`, `list`, `transitions`, `respond`,
    `submit_evidence`, `settle`, `closure` (`cep_authority.rs:resolve`,
    `cep_authority.rs:settle`).
- Agent probing: `agent_probe.rs` — `probe` (`agent_probe.rs:probe`),
  `list` (`agent_probe.rs:list`), trait `CredentialResolver`, `ProbeOutcome`,
  `ProbeRequest`, `endpoint_adapter_version`, `resolve_policy_path`.
- MCP serving: `mcp.rs` — `run(config_path)` (`mcp.rs:run`),
  `serve_workload`, `serve_workload_gated`, `McpServerConfig`,
  `McpListenerGuard`; wire parsing in `mcp_protocol.rs` (private:
  `parse_frame`, `decode_json`, `RpcRequest`).
- Postgres session brokering: `postgres.rs` — `PostgresBroker`
  (`postgres.rs:PostgresBroker`), `DatabaseSession`, `SessionSupervisor`,
  `ExpiryReaper`, `session_deadline`, `PostgresResourceConfig`.
- World pinning and reality trace: `world_pin.rs` (private `verify`,
  `require_same`, `from_payload`); `realitytrace_source.rs` —
  `RealityTraceSource` (`realitytrace_source.rs:RealityTraceSource`).
- SWE-seed settlement reconciliation: `swe_seed_reconciliation.rs` —
  `append_and_reconcile_swe_seed_declaration`,
  `reconcile_swe_seed_declarations`, `reconcile_all_cases`,
  `verify_swe_seed_completion`, `swe_seed_claim_manifest_sha256`
  (`swe_seed_reconciliation.rs:reconcile_all_cases`).
- Transcript sealing: `transcript_seal.rs` — `seal_transcript`,
  `verify_sealed_transcript`, `SealedTranscript`
  (`transcript_seal.rs:seal_transcript`).
- Workload connections: `workload_connection.rs` —
  `authentication_action`, `AuthenticatedConnection`,
  `record_authenticated_request`
  (`workload_connection.rs:authentication_action`).
- Supervisor: `supervisor.rs` (private `spawn`) — background supervision loop
  driven by `SupervisorConfig` (`supervisor.rs:spawn`).
- Subdirectories with their own contracts:
  - [./sfwp/README.md](./sfwp/README.md) — SFWP additive transport layer.
  - [./bin/README.md](./bin/README.md) — helper binaries
    (`sea-forge-mcp`, `gen_sfwp_schema`).

## Invariants

- Kernel logic stays synchronous; Tokio lives only in this crate
  (`lib.rs` module doc; see `crates/AGENTS.md` async-edge rule and
  `just no-async-kernel`).
- The server binary has no CLI contract: configuration is supplied via
  `SEA_FORGE_ROOT`, `SEA_FORGE_SOCKET`, and `<root>/server.yaml`; any argument
  passed to the `sea-forge-server` binary is rejected before configuration
  loads (`main.rs:reject_command_line_arguments`).
- New SFWP capability is added only as flat additive `Request` verb variants —
  never a reshape of existing verbs, never a wrapper frame
  (`sfwp/mod.rs` module doc, ADR-003).
- Every protected verb refuses until `identity.bindings` is configured
  (the `Unconfigured` refusal in `identity.rs`).
- Every connection is authenticated to a peer identity (UID-based on Unix) and
  every request is bound to an `AuthorityAction` before execution
  (`workload_connection.rs:authentication_action`, `identity.rs:PeerIdentity`).
- Operational settlement returns must carry all `REQUIRED_FIELDS`
  (`governed_settlement_return.rs:REQUIRED_FIELDS`).

## Side Effects

- Listens on a Unix domain socket (or Windows named pipe) and performs the
  accept/connect-next handshake so a listening instance always exists
  (`lib.rs:Acceptor`).
- Spawns case runs as subprocesses (`sea-forge run --plan ...`) via
  `spawn_blocking` and agent/delegation processes (`lib.rs` module doc,
  `case_dispatch.rs:submit`, `delegation.rs:execute`).
- Appends to ledgers under the cell root: case events, delegation audit
  (`delegation_audit.rs:DELEGATION_AUDIT_LEDGER`), sealed transcripts
  (`transcript_seal.rs:seal_transcript`), SWE-seed declarations
  (`swe_seed_reconciliation.rs:append_and_reconcile_swe_seed_declaration`).
- Connects to external services: agent endpoints (HTTP, `agent_probe.rs:probe`)
  and Postgres (`postgres.rs:PostgresBroker`).
- Runs background loops: supervisor (`supervisor.rs:spawn`), Postgres session
  supervision and expiry reaping (`postgres.rs:SessionSupervisor`,
  `postgres.rs:ExpiryReaper`).

## Verification

- Run: `cargo test -p sea-forge-server` (repo-wide: `just test`)
