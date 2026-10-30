# sea-forge-agent

## Overview
Governed agent provider adapters for SEA Forge. Two HTTP completion shapes
(OpenAI-compatible, Anthropic) plus an ACP driver for CLI-resident agents
(spec-agent-orchestration §10.4). This crate owns transport only: spawn/attach,
message pump, transcript accumulation, and permission-request surfacing.
Authority, credential authorization, evidence, and settlement remain outside
the adapter boundary, in the server layer [src/lib.rs:module docs].

## Interface
- Public surface re-exported from [src/lib.rs]:
  - ACP driver: `AcpSession`, `AcpSpawn`, `AcpOutcome`, `AcpTermination`, `AcpPermissionRequest`, `AcpPermissionMediator`, `DenyAllMediator`, `deny_once`, `PermissionDecision`, `ACP_PROTOCOL_VERSION` [src/acp.rs]
  - Endpoint configuration: `AgentConfig`, `AgentEndpointConfig`, `EndpointSnapshot`, `EndpointStatus`, `ProviderKind`, `TranscriptRetentionMode` [src/config.rs]
  - Delegation loop: `run_delegation`, `produce_transcript`, `transcript_sha256`, `DelegationConfig`, `DelegationOutcome`, `RedactedTranscript`, `TranscriptEntry`, `TranscriptSummaryInput` [src/delegation.rs]
  - Egress policy: `NetworkPolicy`, `validate_destination`, `prepare_client` [src/network.rs]
  - HTTP providers: `AgentProvider`, `AgentEventStream`, `OpenAiCompatibleProvider`, `AnthropicProvider`, `CompletionRequest`, `CompletionResponse`, `AgentMessage`, `MessageRole`, `AgentToolCall`, `AgentEvent`, `AgentError`, `Usage` [src/provider.rs]
- Source tree contract: [src/README.md]

## Invariants
- `#![forbid(unsafe_code)]` at the crate root [src/lib.rs:forbid(unsafe_code)].
- ACP protocol version pinned to 1 [src/acp.rs:ACP_PROTOCOL_VERSION].
- Production endpoints require HTTPS; private/reserved destination addresses (including cloud metadata ranges) are rejected unless loopback is explicitly allowed for tests [src/network.rs:validate_destination].
- The HTTP client pins pre-validated resolved addresses and disables redirects [src/network.rs:prepare_client].
- Delegation never judges success; settlement belongs to the server [src/delegation.rs:module docs].

## Side Effects
- Spawns CLI-resident ACP agents as child processes over NDJSON stdio and kills them via `/bin/kill` fallback [src/acp.rs:AcpSession].
- Outbound HTTPS/loopback HTTP calls to configured agent endpoints via `reqwest` [src/provider.rs], [src/network.rs:prepare_client].
- Redacts known secrets from transcripts before hashing [src/delegation.rs:redact_str].

## Verification
- Run: `cargo test -p sea-forge-agent` (repo-wide: `just test`)
