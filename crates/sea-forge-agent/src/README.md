# sea-forge-agent — src

## Overview
Source tree for the governed agent provider adapters. The crate owns transport
only (HTTP completion endpoints and an ACP stdio driver); authority, credential
authorization, evidence, and settlement remain outside the adapter boundary
([lib.rs] crate docs). The crate is `#![forbid(unsafe_code)]`
[lib.rs:forbid(unsafe_code)].

## Interface
- Module tree, declared in [lib.rs]:
  - `acp` ([acp.rs]) — minimal ACP (Agent Client Protocol) v1 client driver for CLI-resident agents; JSON-RPC 2.0 over NDJSON stdio.
  - `config` ([config.rs], private module) — endpoint configuration types re-exported below.
  - `delegation` ([delegation.rs], private module) — governed multi-turn delegation loop.
  - `network` ([network.rs], private module) — egress validation and HTTP client construction.
  - `provider` ([provider.rs], private module) — HTTP provider abstraction and the two adapters.
- Public surface (re-exports from [lib.rs]):
  - ACP: `AcpSession`, `AcpSpawn`, `AcpOutcome`, `AcpTermination`, `AcpPermissionRequest`, `AcpPermissionMediator`, `DenyAllMediator`, `deny_once`, `PermissionDecision`, `ACP_PROTOCOL_VERSION` [acp.rs]
  - Config: `AgentConfig`, `AgentEndpointConfig`, `EndpointSnapshot`, `EndpointStatus`, `ProviderKind`, `TranscriptRetentionMode` [config.rs]
  - Delegation: `run_delegation`, `produce_transcript`, `transcript_sha256`, `DelegationConfig`, `DelegationOutcome`, `RedactedTranscript`, `TranscriptEntry`, `TranscriptSummaryInput` [delegation.rs]
  - Network: `NetworkPolicy`, `validate_destination`, `prepare_client` [network.rs]
  - Provider: `AgentProvider`, `AgentEventStream`, `OpenAiCompatibleProvider`, `AnthropicProvider`, `CompletionRequest`, `CompletionResponse`, `AgentMessage`, `MessageRole`, `AgentToolCall`, `AgentEvent`, `AgentError`, `Usage` [provider.rs]

## Invariants
- ACP protocol version is pinned to 1: `pub const ACP_PROTOCOL_VERSION: u32 = 1` [acp.rs:ACP_PROTOCOL_VERSION].
- Production agent endpoints require HTTPS; plain HTTP is only allowed for loopback hosts under an explicit `allow_loopback_test` policy [network.rs:validate_destination].
- Endpoints must resolve only to public addresses; loopback/private/reserved ranges (including 169.254.169.254 metadata) are rejected unless loopback is explicitly allowed for tests [network.rs:validate_destination], [network.rs:is_public].
- The HTTP client is built with redirects disabled and resolved to the pre-validated addresses, preventing DNS-rebinding between validation and request [network.rs:prepare_client].
- Delegation never judges success; it produces a deterministic structural summary and a redacted transcript hash, leaving settlement to the server layer [delegation.rs:module docs].

## Side Effects
- Spawns child processes for ACP agents via `Command::new(&spawn.argv[0])` with NDJSON stdio pipes, and drains their stderr [acp.rs:AcpSession], [acp.rs:drain_stderr].
- Terminates ACP agents with an external `/bin/kill` process as a fallback [acp.rs:kill fallback].
- Performs outbound HTTP requests to agent endpoints through `reqwest::Client` [provider.rs:OpenAiCompatibleProvider], [provider.rs:AnthropicProvider], [network.rs:prepare_client].
- Redacts known secrets from transcripts before hashing/transmission [delegation.rs:redact_str].

## Verification
- Run: `cargo test -p sea-forge-agent` (repo-wide: `just test`)
