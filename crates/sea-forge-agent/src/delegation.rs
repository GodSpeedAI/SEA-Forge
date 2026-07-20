//! Delegation loop for governed `agent_task` execution (spec §16.1).
//!
//! Executes a multi-turn dialogue with an agent endpoint under turn/token
//! caps and a cancellation signal. Produces a deterministic structural
//! summary and a redacted canonical transcript hash. Never judges success —
//! that is the settlement layer's job.

use crate::provider::{
    AgentError, AgentMessage, AgentProvider, CompletionRequest, CompletionResponse, MessageRole,
};
use sea_forge_core::types::{DelegationTermination, TranscriptSummary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// Configuration for a single delegation run.
#[derive(Clone, Debug)]
pub struct DelegationConfig {
    pub model: String,
    pub instruction: String,
    pub max_turns: u32,
    pub token_budget: Option<u64>,
    /// Per-turn max output tokens (mapped to `CompletionRequest::max_tokens`).
    pub max_output_tokens: Option<u32>,
}

/// A single entry in the canonical transcript.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptEntry {
    pub role: String,
    pub content: String,
}

/// Inputs for the structural summary (used by callers that need to hash
/// independently of the outcome).
#[derive(Clone, Debug)]
pub struct TranscriptSummaryInput<'a> {
    pub entries: &'a [TranscriptEntry],
    pub turns_used: u32,
}

/// Typed result of a delegation run (spec §11).
#[derive(Clone, Debug)]
pub struct DelegationOutcome {
    pub termination: DelegationTermination,
    pub turns_used: u32,
    pub final_output: Option<String>,
    pub transcript_sha256: String,
    pub summary: TranscriptSummary,
    /// Raw transcript entries (redacted). Caller stores in `full` mode or
    /// discards after hashing in `summarized` mode.
    pub transcript: Vec<TranscriptEntry>,
    /// Typed error subcode for endpoint errors (spec §11).
    pub error_subcode: Option<String>,
}

/// Conservative token accounting: sum of usage tokens seen so far.
/// When provider metadata is missing, we estimate from content length
/// as a pessimistic upper bound (spec §10.2: never assume zero).
fn estimate_tokens(text: &str) -> u64 {
    // ponytail: ~4 chars per token is a standard heuristic; deliberately
    // pessimistic so the budget cap fires early on missing metadata.
    (text.len() as u64).div_ceil(4)
}

fn usage_tokens(resp: &CompletionResponse) -> u64 {
    resp.usage
        .as_ref()
        .and_then(|u| u.total_tokens)
        .unwrap_or_else(|| {
            let input = resp
                .usage
                .as_ref()
                .and_then(|u| u.input_tokens)
                .unwrap_or(0);
            let output = resp
                .usage
                .as_ref()
                .and_then(|u| u.output_tokens)
                .unwrap_or(0);
            if input + output > 0 {
                input + output
            } else {
                estimate_tokens(&resp.message.content)
            }
        })
}

/// Redact credential value and sentinel patterns from a string before hashing.
/// The credential is never expected in transcript content (it's in auth headers),
/// but the spec requires defense-in-depth redaction before canonicalization.
fn redact(text: &str, credential: &str) -> String {
    let mut result = text.to_owned();
    if !credential.is_empty() {
        result = result.replace(credential, "[REDACTED]");
    }
    // Redact common bearer-token patterns.
    result = result.replace("Bearer ", "");
    result
}

/// Canonicalize transcript entries into deterministic JSONL (sorted keys, one
/// entry per line). serde_json default (no preserve_order) uses BTreeMap so
/// keys are already sorted.
fn canonical_jsonl(entries: &[TranscriptEntry]) -> String {
    entries
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Compute the canonical SHA-256 over redacted transcript entries.
/// Exported so callers can verify an artifact against a recorded digest.
pub fn transcript_sha256(entries: &[TranscriptEntry]) -> String {
    let canonical = canonical_jsonl(entries);
    let hash = Sha256::digest(canonical.as_bytes());
    format!("sha256:{hash:x}")
}

fn build_summary(entries: &[TranscriptEntry], turns_used: u32) -> TranscriptSummary {
    let tool_calls = entries.iter().filter(|e| e.role == "tool").count() as u32;
    let final_excerpt = entries
        .iter()
        .rev()
        .find(|e| e.role == "assistant")
        .map(|e| {
            // ponytail: 200-char bound on the excerpt.
            if e.content.len() <= 200 {
                e.content.clone()
            } else {
                format!("{}...", &e.content[..200])
            }
        })
        .unwrap_or_default();
    TranscriptSummary {
        turn_count: turns_used,
        tool_calls,
        final_excerpt,
    }
}

/// Execute a governed delegation dialogue.
///
/// Implements the spec §16.1 reference loop: check cancellation, check turn
/// cap, call `provider.complete()`, on error terminate with `EndpointError`,
/// append to transcript, check token budget, check finality.
///
/// - `provider`: the resolved agent provider (OpenAI-compatible or Anthropic).
/// - `credential`: the resolved API key (zeroized on drop).
/// - `config`: delegation parameters (instruction, max_turns, token_budget).
/// - `cancel`: cancellation signal; checked at the top of each turn.
///
/// Single-turn providers always return `Completed` after the first response.
/// Multi-turn tool-use continuation is structurally supported (the loop can
/// iterate when `is_final` is false) but no HTTP provider exercises it yet.
pub async fn run_delegation(
    provider: &dyn AgentProvider,
    credential: Zeroizing<String>,
    config: &DelegationConfig,
    cancel: impl Fn() -> bool,
) -> Result<DelegationOutcome, AgentError> {
    let mut transcript: Vec<TranscriptEntry> = Vec::new();
    let mut messages: Vec<AgentMessage> = vec![AgentMessage {
        role: MessageRole::User,
        content: config.instruction.clone(),
    }];
    let mut turns: u32 = 0;
    let mut total_tokens: u64 = 0;

    while turns < config.max_turns {
        if cancel() {
            return Ok(terminate(
                DelegationTermination::Cancelled,
                turns,
                None,
                &transcript,
                &credential,
                None,
            ));
        }

        let request = CompletionRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            max_tokens: config.max_output_tokens,
        };

        match provider.complete(request, credential.clone()).await {
            Ok(resp) => {
                transcript.push(TranscriptEntry {
                    role: "user".into(),
                    content: redact(
                        messages.last().map(|m| m.content.as_str()).unwrap_or(""),
                        &credential,
                    ),
                });
                transcript.push(TranscriptEntry {
                    role: "assistant".into(),
                    content: redact(&resp.message.content, &credential),
                });

                turns += 1;
                total_tokens += usage_tokens(&resp);

                // A request arriving while the provider was in flight wins
                // over completion; the response remains in transcript evidence.
                if cancel() {
                    return Ok(terminate(
                        DelegationTermination::Cancelled,
                        turns,
                        Some(resp.message.content.clone()),
                        &transcript,
                        &credential,
                        None,
                    ));
                }

                if let Some(budget) = config.token_budget {
                    if total_tokens >= budget {
                        return Ok(terminate(
                            DelegationTermination::TurnCapExceeded,
                            turns,
                            Some(resp.message.content.clone()),
                            &transcript,
                            &credential,
                            Some("token_budget_exceeded".into()),
                        ));
                    }
                }

                // ponytail: single-turn — every non-empty response is final.
                // Multi-turn tool-use (provider returns tool_call events) is
                // added in M14; the loop continues when is_final is false.
                let is_final = !resp.message.content.is_empty();
                if is_final {
                    return Ok(terminate(
                        DelegationTermination::Completed,
                        turns,
                        Some(resp.message.content.clone()),
                        &transcript,
                        &credential,
                        None,
                    ));
                }

                // Multi-turn continuation: append assistant message and loop.
                messages.push(resp.message);
            }
            Err(e) => {
                let subcode = error_subcode(&e);
                return Ok(terminate(
                    DelegationTermination::EndpointError,
                    turns,
                    None,
                    &transcript,
                    &credential,
                    Some(subcode),
                ));
            }
        }
    }

    // Turn cap exhausted without a final response.
    Ok(terminate(
        DelegationTermination::TurnCapExceeded,
        turns,
        transcript
            .iter()
            .rev()
            .find(|e| e.role == "assistant")
            .map(|e| e.content.clone()),
        &transcript,
        &credential,
        None,
    ))
}

fn terminate(
    termination: DelegationTermination,
    turns_used: u32,
    final_output: Option<String>,
    transcript: &[TranscriptEntry],
    credential: &Zeroizing<String>,
    error_subcode: Option<String>,
) -> DelegationOutcome {
    let entries = finalize_transcript(transcript, credential);
    let sha = transcript_sha256(&entries);
    let summary = build_summary(&entries, turns_used);
    DelegationOutcome {
        termination,
        turns_used,
        final_output,
        transcript_sha256: sha,
        summary,
        transcript: entries,
        error_subcode,
    }
}

fn finalize_transcript(
    transcript: &[TranscriptEntry],
    credential: &Zeroizing<String>,
) -> Vec<TranscriptEntry> {
    transcript
        .iter()
        .map(|e| TranscriptEntry {
            role: e.role.clone(),
            content: redact(&e.content, credential),
        })
        .collect()
}

fn error_subcode(e: &AgentError) -> String {
    match e {
        AgentError::Unreachable => "unreachable",
        AgentError::Timeout => "timeout",
        AgentError::Http4xx(_) => "http_4xx",
        AgentError::Http5xx(_) => "http_5xx",
        AgentError::Redirect => "redirect",
        AgentError::SchemaInvalid => "schema_invalid",
        AgentError::Oversize => "oversize",
        AgentError::Transport => "transport",
        AgentError::InvalidRequest(_) => "invalid_request",
        AgentError::UnsupportedKind => "unsupported_kind",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{AgentProvider, CompletionRequest, CompletionResponse, Usage};
    use crate::MessageRole;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    struct StubProvider {
        responses: Vec<String>,
        call_count: Arc<AtomicU32>,
    }

    impl StubProvider {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                call_count: Arc::new(AtomicU32::new(0)),
            }
        }
    }

    impl AgentProvider for StubProvider {
        fn kind(&self) -> crate::ProviderKind {
            crate::ProviderKind::OpenAiCompatible
        }

        fn complete<'a>(
            &'a self,
            _request: CompletionRequest,
            _credential: Zeroizing<String>,
        ) -> crate::provider::BoxFuture<'a, Result<CompletionResponse, AgentError>> {
            let idx = self.call_count.fetch_add(1, Ordering::SeqCst) as usize;
            let resp = self.responses.get(idx).cloned().unwrap_or_default();
            Box::pin(async move {
                Ok(CompletionResponse {
                    message: AgentMessage {
                        role: MessageRole::Assistant,
                        content: resp,
                    },
                    usage: Some(Usage {
                        input_tokens: Some(10),
                        output_tokens: Some(20),
                        total_tokens: Some(30),
                    }),
                })
            })
        }

        fn stream<'a>(
            &'a self,
            _request: CompletionRequest,
            _credential: Zeroizing<String>,
        ) -> crate::provider::BoxFuture<'a, Result<Box<dyn crate::AgentEventStream>, AgentError>>
        {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn single_turn_completes() {
        let provider = StubProvider::new(vec!["Hello!".into()]);
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: "Say hello".into(),
            max_turns: 5,
            token_budget: None,
            max_output_tokens: Some(100),
        };
        let outcome = run_delegation(
            &provider,
            Zeroizing::new("secret-key".into()),
            &config,
            || false,
        )
        .await
        .unwrap();

        assert_eq!(outcome.termination, DelegationTermination::Completed);
        assert_eq!(outcome.turns_used, 1);
        assert_eq!(outcome.final_output.as_deref(), Some("Hello!"));
        assert!(!outcome.transcript_sha256.is_empty());
        assert_eq!(outcome.summary.turn_count, 1);
    }

    #[tokio::test]
    async fn cancellation_before_first_turn() {
        let provider = StubProvider::new(vec!["Hello!".into()]);
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: "Say hello".into(),
            max_turns: 5,
            token_budget: None,
            max_output_tokens: None,
        };
        let outcome = run_delegation(
            &provider,
            Zeroizing::new("key".into()),
            &config,
            || true, // cancel immediately
        )
        .await
        .unwrap();

        assert_eq!(outcome.termination, DelegationTermination::Cancelled);
        assert_eq!(outcome.turns_used, 0);
        assert!(outcome.final_output.is_none());
    }

    #[tokio::test]
    async fn cancellation_after_provider_response_wins_over_completion() {
        struct CancellingProvider(Arc<std::sync::atomic::AtomicBool>);

        impl AgentProvider for CancellingProvider {
            fn kind(&self) -> crate::ProviderKind {
                crate::ProviderKind::OpenAiCompatible
            }

            fn complete<'a>(
                &'a self,
                _: CompletionRequest,
                _: Zeroizing<String>,
            ) -> crate::provider::BoxFuture<'a, Result<CompletionResponse, AgentError>>
            {
                self.0.store(true, Ordering::SeqCst);
                Box::pin(async {
                    Ok(CompletionResponse {
                        message: AgentMessage {
                            role: MessageRole::Assistant,
                            content: "response raced with cancellation".into(),
                        },
                        usage: None,
                    })
                })
            }

            fn stream<'a>(
                &'a self,
                _: CompletionRequest,
                _: Zeroizing<String>,
            ) -> crate::provider::BoxFuture<'a, Result<Box<dyn crate::AgentEventStream>, AgentError>>
            {
                unimplemented!()
            }
        }

        let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let provider = CancellingProvider(Arc::clone(&cancelled));
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: "test".into(),
            max_turns: 1,
            token_budget: None,
            max_output_tokens: None,
        };
        let outcome = run_delegation(
            &provider,
            Zeroizing::new("key".into()),
            &config,
            move || cancelled.load(Ordering::SeqCst),
        )
        .await
        .unwrap();

        assert_eq!(outcome.termination, DelegationTermination::Cancelled);
        assert_eq!(outcome.turns_used, 1);
        assert_eq!(
            outcome.final_output.as_deref(),
            Some("response raced with cancellation")
        );
        assert_eq!(outcome.transcript.len(), 2);
    }

    #[tokio::test]
    async fn token_budget_terminates() {
        let provider = StubProvider::new(vec!["Long response".into()]);
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: "Say something".into(),
            max_turns: 5,
            token_budget: Some(25), // provider reports 30 tokens per call
            max_output_tokens: None,
        };
        let outcome = run_delegation(&provider, Zeroizing::new("key".into()), &config, || false)
            .await
            .unwrap();

        assert_eq!(outcome.termination, DelegationTermination::TurnCapExceeded);
        assert_eq!(
            outcome.error_subcode.as_deref(),
            Some("token_budget_exceeded")
        );
    }

    #[tokio::test]
    async fn endpoint_error_terminates_with_transcript() {
        struct ErrorProvider;
        impl AgentProvider for ErrorProvider {
            fn kind(&self) -> crate::ProviderKind {
                crate::ProviderKind::OpenAiCompatible
            }
            fn complete<'a>(
                &'a self,
                _: CompletionRequest,
                _: Zeroizing<String>,
            ) -> crate::provider::BoxFuture<'a, Result<CompletionResponse, AgentError>>
            {
                Box::pin(async { Err(AgentError::Http5xx(503)) })
            }
            fn stream<'a>(
                &'a self,
                _: CompletionRequest,
                _: Zeroizing<String>,
            ) -> crate::provider::BoxFuture<'a, Result<Box<dyn crate::AgentEventStream>, AgentError>>
            {
                unimplemented!()
            }
        }

        let provider = ErrorProvider;
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: "Say hello".into(),
            max_turns: 5,
            token_budget: None,
            max_output_tokens: None,
        };
        let outcome = run_delegation(&provider, Zeroizing::new("key".into()), &config, || false)
            .await
            .unwrap();

        assert_eq!(outcome.termination, DelegationTermination::EndpointError);
        assert_eq!(outcome.error_subcode.as_deref(), Some("http_5xx"));
        assert!(!outcome.transcript_sha256.is_empty());
    }

    #[tokio::test]
    async fn credential_redacted_from_transcript() {
        let secret = "sk-super-secret-key-12345";
        let provider = StubProvider::new(vec![format!("The key is {secret}")]);
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: format!("Echo: {secret}"),
            max_turns: 1,
            token_budget: None,
            max_output_tokens: None,
        };
        let outcome = run_delegation(&provider, Zeroizing::new(secret.into()), &config, || false)
            .await
            .unwrap();

        // Credential must not appear in any transcript entry.
        for entry in &outcome.transcript {
            assert!(
                !entry.content.contains(secret),
                "credential leaked in transcript entry: {}",
                entry.role
            );
        }
        // Hash commits to the redacted form.
        assert!(outcome.transcript_sha256.starts_with("sha256:"));
    }

    #[test]
    fn canonical_jsonl_is_deterministic() {
        let entries = vec![
            TranscriptEntry {
                role: "user".into(),
                content: "hello".into(),
            },
            TranscriptEntry {
                role: "assistant".into(),
                content: "world".into(),
            },
        ];
        let a = canonical_jsonl(&entries);
        let b = canonical_jsonl(&entries);
        assert_eq!(a, b);
        assert!(a.contains("\n"));
    }
}
