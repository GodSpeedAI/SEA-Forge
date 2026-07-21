#![forbid(unsafe_code)]

//! Governed agent provider adapters.
//!
//! Two HTTP shapes (OpenAI-compatible, Anthropic) plus an ACP driver for
//! CLI-resident agents (spec-agent-orchestration §10.4). This crate owns
//! transport only. Authority, credential authorization, evidence, and
//! settlement remain outside the adapter boundary.

pub mod acp;
mod config;
mod delegation;
mod network;
mod provider;

pub use acp::{
    deny_once, AcpOutcome, AcpPermissionMediator, AcpPermissionRequest, AcpSession, AcpSpawn,
    AcpTermination, DenyAllMediator, PermissionDecision, ACP_PROTOCOL_VERSION,
};
pub use config::{
    AgentConfig, AgentEndpointConfig, EndpointSnapshot, EndpointStatus, ProviderKind,
};
pub use delegation::{
    run_delegation, transcript_sha256, DelegationConfig, DelegationOutcome, TranscriptEntry,
    TranscriptSummaryInput,
};
pub use network::{prepare_client, validate_destination, NetworkPolicy};
pub use provider::{
    AgentError, AgentEvent, AgentEventStream, AgentMessage, AgentProvider, AgentToolCall,
    AnthropicProvider, CompletionRequest, CompletionResponse, MessageRole,
    OpenAiCompatibleProvider, Usage,
};
