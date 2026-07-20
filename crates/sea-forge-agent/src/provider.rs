use crate::{
    config::{EndpointSnapshot, ProviderKind},
    network::prepare_client,
};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{future::Future, pin::Pin, sync::Arc};
use zeroize::Zeroizing;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<AgentMessage>,
    pub max_tokens: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionResponse {
    pub message: AgentMessage,
    pub usage: Option<Usage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentEvent {
    Message(AgentMessage),
    Completed(CompletionResponse),
}

pub trait AgentEventStream: Send {
    fn next<'a>(&'a mut self) -> BoxFuture<'a, Result<Option<AgentEvent>, AgentError>>;
}

pub trait AgentProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn complete<'a>(
        &'a self,
        request: CompletionRequest,
        credential: Zeroizing<String>,
    ) -> BoxFuture<'a, Result<CompletionResponse, AgentError>>;
    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
        credential: Zeroizing<String>,
    ) -> BoxFuture<'a, Result<Box<dyn AgentEventStream>, AgentError>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentError {
    InvalidRequest(String),
    UnsupportedKind,
    Unreachable,
    Timeout,
    Http4xx(u16),
    Http5xx(u16),
    Redirect,
    SchemaInvalid,
    Oversize,
    Transport,
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid agent request: {message}"),
            Self::UnsupportedKind => f.write_str("unsupported agent provider kind"),
            Self::Unreachable => f.write_str("agent endpoint unreachable"),
            Self::Timeout => f.write_str("agent endpoint timeout"),
            Self::Http4xx(status) => write!(f, "agent endpoint returned HTTP {status}"),
            Self::Http5xx(status) => write!(f, "agent endpoint returned HTTP {status}"),
            Self::Redirect => f.write_str("agent endpoint redirect refused"),
            Self::SchemaInvalid => f.write_str("agent endpoint returned invalid schema"),
            Self::Oversize => f.write_str("agent endpoint response exceeded size limit"),
            Self::Transport => f.write_str("agent endpoint transport failure"),
        }
    }
}

impl std::error::Error for AgentError {}

pub struct OpenAiCompatibleProvider {
    endpoint: EndpointSnapshot,
    client: Arc<reqwest::Client>,
}

pub struct AnthropicProvider {
    endpoint: EndpointSnapshot,
    client: Arc<reqwest::Client>,
}

impl OpenAiCompatibleProvider {
    pub fn new(endpoint: EndpointSnapshot) -> Result<Self, String> {
        if endpoint.kind != ProviderKind::OpenAiCompatible {
            return Err("provider kind does not match endpoint".into());
        }
        let (client, _) = prepare_client(&endpoint)?;
        Ok(Self {
            endpoint,
            client: Arc::new(client),
        })
    }
}

impl AnthropicProvider {
    pub fn new(endpoint: EndpointSnapshot) -> Result<Self, String> {
        if endpoint.kind != ProviderKind::Anthropic {
            return Err("provider kind does not match endpoint".into());
        }
        let (client, _) = prepare_client(&endpoint)?;
        Ok(Self {
            endpoint,
            client: Arc::new(client),
        })
    }
}

impl AgentProvider for OpenAiCompatibleProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAiCompatible
    }

    fn complete<'a>(
        &'a self,
        request: CompletionRequest,
        credential: Zeroizing<String>,
    ) -> BoxFuture<'a, Result<CompletionResponse, AgentError>> {
        Box::pin(async move {
            let body = openai_request(&request)?;
            let bytes = serde_json::to_vec(&body)
                .map_err(|_| AgentError::InvalidRequest("request serialization failed".into()))?;
            if bytes.len() > self.endpoint.max_request_bytes {
                return Err(AgentError::Oversize);
            }
            let response = self
                .client
                .post(endpoint_path(
                    &self.endpoint.base_url,
                    "v1/chat/completions",
                ))
                .headers(auth_headers("Bearer", &credential, None)?)
                .json(&body)
                .send()
                .await
                .map_err(map_transport)?;
            parse_response(response, self.endpoint.max_response_bytes, parse_openai).await
        })
    }

    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
        credential: Zeroizing<String>,
    ) -> BoxFuture<'a, Result<Box<dyn AgentEventStream>, AgentError>> {
        Box::pin(async move {
            let response = self.complete(request, credential).await?;
            Ok(Box::new(OneEventStream::new(response)) as Box<dyn AgentEventStream>)
        })
    }
}

impl AgentProvider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn complete<'a>(
        &'a self,
        request: CompletionRequest,
        credential: Zeroizing<String>,
    ) -> BoxFuture<'a, Result<CompletionResponse, AgentError>> {
        Box::pin(async move {
            let body = anthropic_request(&request)?;
            let bytes = serde_json::to_vec(&body)
                .map_err(|_| AgentError::InvalidRequest("request serialization failed".into()))?;
            if bytes.len() > self.endpoint.max_request_bytes {
                return Err(AgentError::Oversize);
            }
            let response = self
                .client
                .post(endpoint_path(&self.endpoint.base_url, "v1/messages"))
                .headers(auth_headers("", &credential, Some("2023-06-01"))?)
                .json(&body)
                .send()
                .await
                .map_err(map_transport)?;
            parse_response(response, self.endpoint.max_response_bytes, parse_anthropic).await
        })
    }

    fn stream<'a>(
        &'a self,
        request: CompletionRequest,
        credential: Zeroizing<String>,
    ) -> BoxFuture<'a, Result<Box<dyn AgentEventStream>, AgentError>> {
        Box::pin(async move {
            let response = self.complete(request, credential).await?;
            Ok(Box::new(OneEventStream::new(response)) as Box<dyn AgentEventStream>)
        })
    }
}

struct OneEventStream {
    event: Option<AgentEvent>,
}

impl OneEventStream {
    fn new(response: CompletionResponse) -> Self {
        Self {
            event: Some(AgentEvent::Completed(response)),
        }
    }
}

impl AgentEventStream for OneEventStream {
    fn next<'a>(&'a mut self) -> BoxFuture<'a, Result<Option<AgentEvent>, AgentError>> {
        Box::pin(async { Ok(self.event.take()) })
    }
}

fn openai_request(request: &CompletionRequest) -> Result<Value, AgentError> {
    Ok(json!({
        "model": request.model,
        "messages": request.messages.iter().map(|message| json!({
            "role": match message.role { MessageRole::System => "system", MessageRole::User => "user", MessageRole::Assistant => "assistant", MessageRole::Tool => "tool" },
            "content": message.content,
        })).collect::<Vec<_>>(),
        "max_tokens": request.max_tokens,
    }))
}

fn anthropic_request(request: &CompletionRequest) -> Result<Value, AgentError> {
    let system = request
        .messages
        .iter()
        .find(|message| message.role == MessageRole::System)
        .map(|message| message.content.clone());
    let messages = request
        .messages
        .iter()
        .filter(|message| message.role != MessageRole::System)
        .map(|message| {
            json!({
                "role": match message.role { MessageRole::Assistant => "assistant", _ => "user" },
                "content": message.content,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "model": request.model,
        "max_tokens": request.max_tokens.unwrap_or(1024),
        "system": system,
        "messages": messages,
    }))
}

fn auth_headers(
    prefix: &str,
    credential: &str,
    version: Option<&str>,
) -> Result<HeaderMap, AgentError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let value = if prefix.is_empty() {
        credential.to_owned()
    } else {
        format!("{prefix} {credential}")
    };
    let mut auth = HeaderValue::from_str(&value)
        .map_err(|_| AgentError::InvalidRequest("credential header is invalid".into()))?;
    auth.set_sensitive(true);
    if prefix.is_empty() {
        headers.insert("x-api-key", auth);
    } else {
        headers.insert("authorization", auth);
    }
    if let Some(version) = version {
        headers.insert(
            "anthropic-version",
            HeaderValue::from_str(version)
                .map_err(|_| AgentError::InvalidRequest("provider version is invalid".into()))?,
        );
    }
    Ok(headers)
}

fn endpoint_path(base: &url::Url, path: &str) -> url::Url {
    let mut url = base.clone();
    let base_path = url.path().trim_end_matches('/');
    url.set_path(&format!("{base_path}/{path}"));
    url.set_query(None);
    url.set_fragment(None);
    url
}

fn map_transport(error: reqwest::Error) -> AgentError {
    if error.is_timeout() {
        AgentError::Timeout
    } else if error.is_redirect() {
        AgentError::Redirect
    } else if error.is_connect() {
        AgentError::Unreachable
    } else {
        AgentError::Transport
    }
}

async fn parse_response<T>(
    response: reqwest::Response,
    max_bytes: usize,
    parser: fn(&[u8]) -> Result<T, AgentError>,
) -> Result<T, AgentError> {
    let status = response.status();
    if status.is_redirection() {
        return Err(AgentError::Redirect);
    }
    if status.is_client_error() {
        return Err(AgentError::Http4xx(status.as_u16()));
    }
    if status.is_server_error() {
        return Err(AgentError::Http5xx(status.as_u16()));
    }
    if response
        .content_length()
        .is_some_and(|length| length as usize > max_bytes)
    {
        return Err(AgentError::Oversize);
    }
    let body = response.bytes().await.map_err(map_transport)?;
    if body.len() > max_bytes {
        return Err(AgentError::Oversize);
    }
    parser(&body)
}

fn parse_openai(bytes: &[u8]) -> Result<CompletionResponse, AgentError> {
    let value: OpenAiResponse =
        serde_json::from_slice(bytes).map_err(|_| AgentError::SchemaInvalid)?;
    let choice = value
        .choices
        .into_iter()
        .next()
        .ok_or(AgentError::SchemaInvalid)?;
    Ok(CompletionResponse {
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: choice.message.content,
        },
        usage: value.usage.map(|usage| Usage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }),
    })
}

fn parse_anthropic(bytes: &[u8]) -> Result<CompletionResponse, AgentError> {
    let value: AnthropicResponse =
        serde_json::from_slice(bytes).map_err(|_| AgentError::SchemaInvalid)?;
    let content = value
        .content
        .into_iter()
        .find_map(|block| (block.kind == "text").then_some(block.text))
        .flatten()
        .ok_or(AgentError::SchemaInvalid)?;
    Ok(CompletionResponse {
        message: AgentMessage {
            role: MessageRole::Assistant,
            content,
        },
        usage: value.usage.map(|usage| Usage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            total_tokens: None,
        }),
    })
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}
#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}
#[derive(Deserialize)]
struct OpenAiMessage {
    content: String,
}
#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}
#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicBlock>,
    usage: Option<AnthropicUsage>,
}
#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}
#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> CompletionRequest {
        CompletionRequest {
            model: "test-model".into(),
            messages: vec![AgentMessage {
                role: MessageRole::User,
                content: "hello".into(),
            }],
            max_tokens: Some(32),
        }
    }

    #[test]
    fn openai_request_shape_is_pinned() {
        assert_eq!(
            openai_request(&request()).unwrap(),
            json!({
                "model": "test-model",
                "messages": [{"role": "user", "content": "hello"}],
                "max_tokens": 32,
            })
        );
    }

    #[test]
    fn anthropic_request_shape_is_pinned() {
        assert_eq!(
            anthropic_request(&request()).unwrap(),
            json!({
                "model": "test-model",
                "max_tokens": 32,
                "system": null,
                "messages": [{"role": "user", "content": "hello"}],
            })
        );
    }

    #[test]
    fn malformed_provider_response_is_typed() {
        assert_eq!(parse_openai(b"{}"), Err(AgentError::SchemaInvalid));
    }
}
