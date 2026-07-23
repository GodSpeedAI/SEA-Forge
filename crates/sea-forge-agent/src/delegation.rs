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
use sea_forge_ledger::{canonical_json, SECRET_SENTINELS};
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

/// Case-insensitively redact every occurrence of `needle` in `haystack`,
/// preserving surrounding text. Empty needles are ignored. Used for the
/// sentinel corpus, which must match regardless of casing in transcript text.
fn replace_ci(haystack: &str, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return haystack.to_owned();
    }
    // ponytail: ASCII-only lowercasing preserves byte-for-byte offsets. The
    // redaction corpus (SECRET_SENTINELS + "Bearer ") is ASCII; full Unicode
    // case folding can change byte length (e.g. 'İ' → "i̇") and shift the
    // match offsets used to slice the original haystack.
    let lower_hay = haystack.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    debug_assert_eq!(lower_hay.len(), haystack.len());
    debug_assert_eq!(lower_needle.len(), needle.len());
    let mut result = String::with_capacity(haystack.len());
    let mut cursor = 0usize;
    while let Some(rel) = lower_hay[cursor..].find(&lower_needle) {
        let start = cursor + rel;
        let end = start + needle.len();
        result.push_str(&haystack[cursor..start]);
        result.push_str(replacement);
        cursor = end;
    }
    result.push_str(&haystack[cursor..]);
    result
}

/// Redact a single string against the shared secret corpus: caller-supplied
/// known secrets (the exact credential value plus any others) and the
/// ledger's [`SECRET_SENTINELS`] plaintext sentinel list. This is the ONE
/// redaction routine; HTTP and ACP transcripts both flow through it so the
/// exact same corpus is applied before any canonicalization or hashing.
///
/// Sentinel matching is case-insensitive (the ledger's `check_redaction`
/// lowercases before scanning); known-secret matching is exact so it catches
/// high-entropy tokens verbatim.
fn redact_str(text: &str, known_secrets: &[&str]) -> String {
    let mut result = text.to_owned();
    for secret in known_secrets {
        if !secret.is_empty() {
            result = result.replace(secret, "[REDACTED]");
        }
    }
    // Redact common bearer-token prefix (defense in depth against leaked
    // Authorization headers echoed into transcript content).
    result = replace_ci(&result, "Bearer ", "");
    for sentinel in SECRET_SENTINELS {
        result = replace_ci(&result, sentinel, "[REDACTED]");
    }
    result
}

/// A redacted transcript together with the exact canonical bytes used for
/// hashing and storage. The digest is computed from `canonical_bytes`, so any
/// consumer that persists `canonical_bytes` and re-hashes them reproduces
/// `sha256` bit-for-bit — no second serialization path can diverge.
#[derive(Clone, Debug)]
pub struct RedactedTranscript {
    /// Redacted transcript entries (secrets and sentinels replaced).
    pub entries: Vec<TranscriptEntry>,
    /// Canonical JSONL bytes: one [`canonical_json`]-canonicalized entry per
    /// line, `\n`-separated, no trailing newline. These are the exact bytes
    /// the digest commits to and the exact bytes callers must persist.
    pub canonical_bytes: Vec<u8>,
    /// `sha256:<hex>` over `canonical_bytes`.
    pub sha256: String,
}

/// Canonicalize one redacted entry into JCS-NFC bytes via the shared ledger
/// canonicalizer. Falls back to compact serde bytes only if the value cannot
/// be produced (unreachable for a `{role, content}` struct).
fn canonical_entry_bytes(entry: &TranscriptEntry) -> Vec<u8> {
    match serde_json::to_value(entry).and_then(|v| {
        canonical_json(&v).map_err(|e| serde_json::Error::io(std::io::Error::other(e.to_string())))
    }) {
        Ok(bytes) => bytes,
        Err(_) => serde_json::to_vec(entry).unwrap_or_default(),
    }
}

/// THE transcript producer: redact `entries` against `known_secrets` + the
/// shared sentinel corpus, canonicalize with the repository's canonical-JSON
/// rules, and hash the exact canonical bytes.
///
/// Every path that hashes, summarizes, stores, or verifies a transcript MUST
/// route through this function (HTTP and ACP alike) so redaction always
/// precedes canonicalization and no divergent serialization exists.
pub fn produce_transcript(
    entries: &[TranscriptEntry],
    known_secrets: &[&str],
) -> RedactedTranscript {
    let redacted: Vec<TranscriptEntry> = entries
        .iter()
        .map(|e| TranscriptEntry {
            role: e.role.clone(),
            content: redact_str(&e.content, known_secrets),
        })
        .collect();
    let mut canonical_bytes = Vec::new();
    for (idx, entry) in redacted.iter().enumerate() {
        if idx > 0 {
            canonical_bytes.push(b'\n');
        }
        canonical_bytes.extend_from_slice(&canonical_entry_bytes(entry));
    }
    let hash = Sha256::digest(&canonical_bytes);
    RedactedTranscript {
        entries: redacted,
        canonical_bytes,
        sha256: format!("sha256:{hash:x}"),
    }
}

/// Compute the canonical SHA-256 over already-redacted transcript entries.
/// Exported so callers can verify an artifact against a recorded digest.
/// Passes no additional known secrets — entries are assumed pre-redacted;
/// the sentinel corpus is still applied idempotently.
pub fn transcript_sha256(entries: &[TranscriptEntry]) -> String {
    produce_transcript(entries, &[]).sha256
}

fn build_summary(entries: &[TranscriptEntry], turns_used: u32) -> TranscriptSummary {
    let tool_calls = entries.iter().filter(|e| e.role == "tool").count() as u32;
    let final_excerpt = entries
        .iter()
        .rev()
        .find(|e| e.role == "assistant")
        .map(|e| {
            // ponytail: 200-byte bound on the excerpt, respecting UTF-8 char
            // boundaries so redacted multi-byte content never panics.
            if e.content.len() <= 200 {
                e.content.clone()
            } else {
                let mut end = 200;
                while !e.content.is_char_boundary(end) {
                    end -= 1;
                }
                format!("{}...", &e.content[..end])
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
                // Attribute the user turn to the most recent user message in
                // the request, not `messages.last()` — on a multi-turn
                // continuation the last entry is the prior assistant message.
                let user_content = messages
                    .iter()
                    .rev()
                    .find(|m| m.role == MessageRole::User)
                    .map(|m| m.content.clone())
                    .unwrap_or_default();
                transcript.push(TranscriptEntry {
                    role: "user".into(),
                    content: user_content,
                });
                transcript.push(TranscriptEntry {
                    role: "assistant".into(),
                    content: resp.message.content.clone(),
                });

                turns += 1;
                total_tokens += usage_tokens(&resp);

                // Every tool/function call requested by the agent re-enters
                // authority. In single-turn mode there is no tool channel,
                // so every request is unconditionally denied and recorded
                // (spec §10.2, T13.7). No side effect occurs.
                for tool_call in &resp.tool_calls {
                    transcript.push(TranscriptEntry {
                        role: "system".into(),
                        content: format!("tool_request_denied:{}", tool_call.operation),
                    });
                }

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
    // Single choke point: redact against the credential + shared sentinel
    // corpus and hash the exact canonical bytes.
    let produced = produce_transcript(transcript, &[credential.as_str()]);
    let summary = build_summary(&produced.entries, turns_used);
    DelegationOutcome {
        termination,
        turns_used,
        final_output,
        transcript_sha256: produced.sha256,
        summary,
        transcript: produced.entries,
        error_subcode,
    }
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
                    tool_calls: Vec::new(),
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
                        tool_calls: Vec::new(),
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
    fn produce_transcript_is_deterministic() {
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
        let a = produce_transcript(&entries, &[]);
        let b = produce_transcript(&entries, &[]);
        assert_eq!(a.canonical_bytes, b.canonical_bytes);
        assert_eq!(a.sha256, b.sha256);
        assert!(a.canonical_bytes.contains(&b'\n'));
    }

    /// The digest MUST be computed from the exact returned canonical bytes:
    /// re-hashing `canonical_bytes` reproduces `sha256` bit-for-bit, so a
    /// consumer that stores those bytes and re-verifies cannot diverge.
    #[test]
    fn produce_transcript_digest_matches_returned_bytes() {
        let entries = vec![TranscriptEntry {
            role: "assistant".into(),
            content: "some content".into(),
        }];
        let produced = produce_transcript(&entries, &[]);
        let rehash = format!("sha256:{:x}", Sha256::digest(&produced.canonical_bytes));
        assert_eq!(produced.sha256, rehash);
    }

    /// Every ledger sentinel value must be redacted out of transcript content,
    /// case-insensitively (private-key headers, password/token markers, etc.).
    #[test]
    fn sentinels_are_redacted() {
        for sentinel in SECRET_SENTINELS {
            let leaked = format!("prefix {sentinel} suffix");
            let entries = vec![TranscriptEntry {
                role: "assistant".into(),
                content: leaked.clone(),
            }];
            let produced = produce_transcript(&entries, &[]);
            let out = &produced.entries[0].content;
            assert!(
                !out.to_lowercase().contains(&sentinel.to_lowercase()),
                "sentinel `{sentinel}` leaked: {out}"
            );
            // And it must be absent from the canonical bytes that get hashed.
            let bytes_lower = String::from_utf8_lossy(&produced.canonical_bytes).to_lowercase();
            assert!(
                !bytes_lower.contains(&sentinel.to_lowercase()),
                "sentinel `{sentinel}` present in hashed bytes"
            );
        }
    }

    /// Case-insensitive sentinel matching: an uppercased sentinel still redacts.
    #[test]
    fn sentinel_redaction_is_case_insensitive() {
        let entries = vec![TranscriptEntry {
            role: "assistant".into(),
            content: "here is my PASSWORD=hunter2 and API_KEY=abc".into(),
        }];
        let produced = produce_transcript(&entries, &[]);
        let out = produced.entries[0].content.to_lowercase();
        assert!(!out.contains("password"), "PASSWORD not redacted: {out}");
        assert!(!out.contains("api_key"), "API_KEY not redacted: {out}");
    }

    /// A caller-supplied high-entropy token (e.g. a bearer credential) is
    /// redacted verbatim even though it is not in the sentinel corpus.
    #[test]
    fn known_secret_token_is_redacted() {
        let token = "sk-live-8f3a2b1c9d7e6f5a4b3c2d1e0f9a8b7c";
        let entries = vec![TranscriptEntry {
            role: "assistant".into(),
            content: format!("the token is {token} okay"),
        }];
        let produced = produce_transcript(&entries, &[token]);
        assert!(
            !produced.entries[0].content.contains(token),
            "known secret leaked: {}",
            produced.entries[0].content
        );
    }

    /// Unrelated content that merely resembles a secret must NOT be
    /// over-redacted: benign words that do not match any sentinel survive.
    #[test]
    fn unrelated_secrets_are_not_over_redacted() {
        let benign = "The passwordless flow uses a keychain and secretary notes.";
        // "passwordless" contains "password" as a substring — the ledger's own
        // policy treats that as a hit, so we only assert that clearly-unrelated
        // tokens with no sentinel substring survive untouched.
        let clean = "The login flow uses a keychain and secretary notes.";
        let entries = vec![TranscriptEntry {
            role: "user".into(),
            content: clean.into(),
        }];
        let produced = produce_transcript(&entries, &[]);
        assert_eq!(
            produced.entries[0].content, clean,
            "benign content was over-redacted"
        );
        // Sanity: the passwordy variant DOES trip redaction (substring policy).
        let entries2 = vec![TranscriptEntry {
            role: "user".into(),
            content: benign.into(),
        }];
        let produced2 = produce_transcript(&entries2, &[]);
        assert!(produced2.entries[0].content.contains("[REDACTED]"));
    }

    /// Redaction preserves UTF-8 correctness around multi-byte characters:
    /// a sentinel embedded next to emoji/CJK text is removed without corrupting
    /// neighbouring code points, and the canonical bytes remain valid UTF-8.
    #[test]
    fn redaction_preserves_utf8() {
        let entries = vec![TranscriptEntry {
            role: "assistant".into(),
            content: "日本語 password=秘密 🔐 done".into(),
        }];
        let produced = produce_transcript(&entries, &[]);
        let out = &produced.entries[0].content;
        assert!(!out.to_lowercase().contains("password"));
        assert!(out.contains("日本語"));
        assert!(out.contains("🔐"));
        // Canonical bytes must round-trip as valid UTF-8.
        assert!(std::str::from_utf8(&produced.canonical_bytes).is_ok());
    }

    /// Field ordering in the canonical bytes is stable regardless of the order
    /// in which fields appear on the wire (JCS sorts keys). Two logically
    /// equal entries produce identical canonical bytes.
    #[test]
    fn canonical_field_ordering_is_stable() {
        let a = produce_transcript(
            &[TranscriptEntry {
                role: "user".into(),
                content: "x".into(),
            }],
            &[],
        );
        // A hand-built object with reversed key order must canonicalize equal.
        let manual = canonical_json(&serde_json::json!({"content": "x", "role": "user"})).unwrap();
        assert_eq!(a.canonical_bytes, manual);
    }

    /// Streaming chunk-boundary reconstruction: a secret split across two
    /// streamed chunks is NOT caught while the chunks are separate entries,
    /// but IS caught once the chunks are reassembled into a single logical
    /// message before `produce_transcript` runs. This documents that redaction
    /// must run on the reconstructed message, not per-chunk.
    #[test]
    fn streaming_chunk_boundary_reconstruction() {
        // Two raw chunks that split the sentinel "password" across a boundary.
        let chunk_a = "the secret is pass";
        let chunk_b = "word=hunter2 done";

        // Per-chunk redaction misses the split secret (neither chunk contains
        // the full sentinel) — this is the failure mode we guard against.
        let split = produce_transcript(
            &[
                TranscriptEntry {
                    role: "assistant".into(),
                    content: chunk_a.into(),
                },
                TranscriptEntry {
                    role: "assistant".into(),
                    content: chunk_b.into(),
                },
            ],
            &[],
        );
        let split_joined = split
            .entries
            .iter()
            .map(|e| e.content.as_str())
            .collect::<String>()
            .to_lowercase();
        assert!(
            split_joined.contains("password"),
            "sanity: split chunks individually do not contain the sentinel"
        );

        // Reassembling the chunks into one logical message before producing the
        // transcript catches the reconstructed secret.
        let reassembled = format!("{chunk_a}{chunk_b}");
        let produced = produce_transcript(
            &[TranscriptEntry {
                role: "assistant".into(),
                content: reassembled,
            }],
            &[],
        );
        assert!(
            !produced.entries[0]
                .content
                .to_lowercase()
                .contains("password"),
            "reassembled secret leaked: {}",
            produced.entries[0].content
        );
    }

    /// T13.7: an out-of-grant tool request from the agent is recorded as
    /// denied in the transcript, produces no side effect, and the dialogue
    /// settles normally.
    #[tokio::test]
    async fn hostile_tool_request_is_denied_without_side_effect() {
        struct HostileProvider;
        impl AgentProvider for HostileProvider {
            fn kind(&self) -> crate::ProviderKind {
                crate::ProviderKind::OpenAiCompatible
            }
            fn complete<'a>(
                &'a self,
                _: CompletionRequest,
                _: Zeroizing<String>,
            ) -> crate::provider::BoxFuture<'a, Result<CompletionResponse, AgentError>>
            {
                Box::pin(async {
                    Ok(CompletionResponse {
                        message: AgentMessage {
                            role: MessageRole::Assistant,
                            content: "done".into(),
                        },
                        usage: None,
                        tool_calls: vec![crate::AgentToolCall {
                            operation: "execute_command".into(),
                            parameters: serde_json::json!({"argv": ["rm", "-rf", "/"]}),
                        }],
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

        let provider = HostileProvider;
        let config = DelegationConfig {
            model: "test-model".into(),
            instruction: "test".into(),
            max_turns: 1,
            token_budget: None,
            max_output_tokens: None,
        };
        let outcome = run_delegation(&provider, Zeroizing::new("key".into()), &config, || false)
            .await
            .unwrap();

        assert_eq!(outcome.termination, DelegationTermination::Completed);
        assert_eq!(outcome.final_output.as_deref(), Some("done"));
        let denial = outcome
            .transcript
            .iter()
            .find(|e| e.role == "system" && e.content.starts_with("tool_request_denied:"));
        assert!(
            denial.is_some(),
            "transcript must record the tool-request denial"
        );
        assert!(
            denial.unwrap().content.contains("execute_command"),
            "denial must name the refused operation"
        );
    }
}
