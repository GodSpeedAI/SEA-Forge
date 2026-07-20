#![forbid(unsafe_code)]

//! Governed HTTP provider adapters for OpenAI-compatible and Anthropic APIs.
//!
//! This crate owns transport only. Authority, credential authorization,
//! evidence, and settlement remain outside the adapter boundary.

mod config;
mod network;
mod provider;

pub use config::{
    AgentConfig, AgentEndpointConfig, EndpointSnapshot, EndpointStatus, ProviderKind,
};
pub use network::{prepare_client, validate_destination, NetworkPolicy};
pub use provider::{
    AgentError, AgentEvent, AgentEventStream, AgentMessage, AgentProvider, AnthropicProvider,
    CompletionRequest, CompletionResponse, MessageRole, OpenAiCompatibleProvider, Usage,
};
