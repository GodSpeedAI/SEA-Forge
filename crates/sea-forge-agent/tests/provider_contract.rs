use sea_forge_agent::{
    AgentError, AgentMessage, AgentProvider, AnthropicProvider, CompletionRequest,
    EndpointSnapshot, MessageRole, OpenAiCompatibleProvider, ProviderKind,
};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use url::Url;
use zeroize::Zeroizing;

fn endpoint(kind: ProviderKind, base_url: String) -> EndpointSnapshot {
    EndpointSnapshot {
        id: "local-test".into(),
        kind,
        base_url: Url::parse(&base_url).unwrap(),
        argv: vec![],
        env: vec![],
        credential_ref: Some("TEST_CREDENTIAL".into()),
        model: "test-model".into(),
        descriptor_config_sha256: "sha256:descriptor".into(),
        max_request_bytes: 16_384,
        max_response_bytes: 16_384,
        timeout: Duration::from_secs(5),
        allow_loopback_test: true,
    }
}

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

async fn server(
    status: &'static str,
    body: &'static str,
) -> (String, Arc<tokio::sync::Mutex<Vec<u8>>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let address = listener.local_addr().unwrap();
    let request = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let captured = Arc::clone(&request);
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut bytes = vec![0_u8; 16_384];
        let count = stream.read(&mut bytes).await.unwrap();
        captured.lock().await.extend_from_slice(&bytes[..count]);
        let body_bytes = body.as_bytes();
        let headers = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body_bytes.len()
        );
        stream.write_all(headers.as_bytes()).await.unwrap();
        stream.write_all(body_bytes).await.unwrap();
    });
    (format!("http://{address}/"), request)
}

#[tokio::test]
async fn openai_provider_sends_pinned_request_and_parses_response() {
    let (base, request_bytes) = server(
        "200 OK",
        r#"{"choices":[{"message":{"content":"done"}}],"usage":{"prompt_tokens":1,"completion_tokens":2,"total_tokens":3}}"#,
    )
    .await;
    let provider =
        OpenAiCompatibleProvider::new(endpoint(ProviderKind::OpenAiCompatible, base)).unwrap();
    let result = provider
        .complete(request(), Zeroizing::new("secret".into()))
        .await
        .unwrap();
    assert_eq!(result.message.content, "done");
    let request = String::from_utf8(request_bytes.lock().await.clone()).unwrap();
    assert!(request.starts_with("POST /v1/chat/completions HTTP/1.1"));
    assert!(request.contains(r#""model":"test-model""#));
    assert!(request.contains("authorization: Bearer secret"));
}

#[tokio::test]
async fn anthropic_provider_sends_pinned_request_and_parses_response() {
    let (base, request_bytes) = server(
        "200 OK",
        r#"{"content":[{"type":"text","text":"done"}],"usage":{"input_tokens":1,"output_tokens":2}}"#,
    )
    .await;
    let provider = AnthropicProvider::new(endpoint(ProviderKind::Anthropic, base)).unwrap();
    let result = provider
        .complete(request(), Zeroizing::new("secret".into()))
        .await
        .unwrap();
    assert_eq!(result.message.content, "done");
    let request = String::from_utf8(request_bytes.lock().await.clone()).unwrap();
    assert!(request.starts_with("POST /v1/messages HTTP/1.1"));
    assert!(request.contains("x-api-key: secret"));
    assert!(request.contains("anthropic-version: 2023-06-01"));
}

#[tokio::test]
async fn provider_maps_endpoint_failures_without_body_leakage() {
    let (base, _) = server("503 Service Unavailable", "credential=secret").await;
    let provider =
        OpenAiCompatibleProvider::new(endpoint(ProviderKind::OpenAiCompatible, base)).unwrap();
    assert_eq!(
        provider
            .complete(request(), Zeroizing::new("secret".into()))
            .await,
        Err(AgentError::Http5xx(503))
    );

    let unused = "http://127.0.0.1:9/".to_string();
    let provider =
        OpenAiCompatibleProvider::new(endpoint(ProviderKind::OpenAiCompatible, unused)).unwrap();
    assert!(matches!(
        provider
            .complete(request(), Zeroizing::new("secret".into()))
            .await,
        Err(AgentError::Unreachable | AgentError::Timeout)
    ));
}
