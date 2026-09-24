//! SFWP case mutation and execution verbs (T04, ADR-003 additive).
//!
//! Each verb is a thin envelope over the shared `case_ops` library — the same
//! governed implementation the CLI calls — executed on a blocking thread and
//! attributed to the actor the identity gate verified. Authority and
//! separation-of-duty therefore bind in the library, not in a server-local
//! copy: the `discretionary_task_add`, `case_reopen`, `case_terminate` and
//! `human_task_completion` reserved actions, the episode's own operation
//! authority decision, and the proposer-SoD refusal inside
//! `case_ops::resolve_approval`.
//!
//! # Event publishing (cannot-miss)
//!
//! Every append to a case's `case-events.jsonl` flows through exactly one of
//! two choke points: the `case_ops::*_with` functions (mutations) or
//! `CaseRunner::append_event` (the advance engine) — and both notify a host
//! sink *after* the append is durable. The server passes a sink that forwards
//! each event over an unbounded channel to a dedicated publisher task, which
//! appends one `case.trace.<kind>` frame to the durable events ledger and
//! fans it out on the broadcast bus. A frame therefore cannot be missed: the
//! set of published frames equals the set of appended events by construction,
//! the channel send can only fail after the receiver is gone (which happens
//! only once the last sender is dropped, i.e. after the mutation finished),
//! and ordering is preserved because the library appends sequentially and the
//! single publisher awaits each ledger append before the next.

use crate::ServerState;
use schemars::JsonSchema;
use sea_forge_case_runner::case_ops::advance::{advance_case, AdvanceOutcome, AdvanceScope};
use sea_forge_core::{
    errors::ForgeError,
    types::{ActorRole, PlanItem, TraceEvent},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Ceiling on the artifact bytes `artifact.get` will return (checked via
/// `fs::metadata` before the read). The response embeds the content, so the
/// bound also protects the NDJSON line size a client must parse.
pub(crate) const MAX_ARTIFACT_BYTES: u64 = 1024 * 1024;

/// `case.add_item` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseAddItemResult {
    pub ok: bool,
    pub case_id: String,
    pub plan_item_id: String,
    /// The actor the identity gate verified — the proposer of record. A
    /// client-supplied `proposed_by` is overwritten, never trusted.
    pub proposed_by: String,
}

/// `case.reopen` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseReopenResult {
    pub ok: bool,
    pub case_id: String,
    pub case_state: String,
}

/// `case.terminate` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseTerminateResult {
    pub ok: bool,
    pub case_id: String,
    pub case_state: String,
    pub close_reason: String,
}

/// One executed episode reported by `case.advance` / `item.execute`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct EpisodeSummary {
    pub item_id: String,
    pub run_id: String,
    /// Wire spelling of the settlement status (`accepted`, `rejected`, ...).
    pub settlement_status: String,
}

/// `case.advance` / `item.execute` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CaseAdvanceResult {
    pub ok: bool,
    pub case_id: String,
    /// Terminal standing of the pass: `completed`, `terminated`,
    /// `awaiting_approval`, `parked_human_task`, `idle`, `blocked`, or
    /// `episode_budget_reached`.
    pub state: String,
    pub episodes: Vec<EpisodeSummary>,
}

/// `human_task.complete` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct HumanTaskCompleteResult {
    pub ok: bool,
    pub case_id: String,
    pub item_id: String,
    /// `0` when the completion closed the case, `5` when work remains — the
    /// CLI's historical exit-code contract, surfaced instead of invented.
    pub exit_code: u8,
}

/// `artifact.get` result body: the content-addressed artifact bytes plus the
/// evidence-record refs the kernel's own artifact store shapes.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactView {
    pub digest: String,
    pub run_id: String,
    pub evidence_id: String,
    /// Run-relative evidence URI (e.g. `artifacts/stdout.txt`).
    pub uri: String,
    pub size_bytes: u64,
    /// The artifact content when it is valid UTF-8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Standard base64 (with padding) of the content when it is not UTF-8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
}

/// The snake_case wire spelling of a `TraceKind` (`PlanMutated` →
/// `plan_mutated`) used for `case.trace.<kind>` event kinds.
fn trace_kind_snake(kind: sea_forge_core::types::TraceKind) -> String {
    format!("{kind:?}").to_lowercase()
}

/// Run a synchronous case mutation on a blocking thread, publishing one
/// durable event frame for every case trace event the mutation appends.
///
/// The work closure receives the notify sink; it must forward every append
/// through it (the `case_ops` `_with` functions and the advance engine do).
/// See the module docs for the cannot-miss argument.
async fn run_case_mutation<T, F>(
    state: &Arc<ServerState>,
    case_id: &str,
    work: F,
) -> Result<T, ForgeError>
where
    F: FnOnce(&mut dyn FnMut(&TraceEvent)) -> Result<T, ForgeError> + Send + 'static,
    T: Send + 'static,
{
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<TraceEvent>();
    let state_for_publish = Arc::clone(state);
    let case_for_publish = case_id.to_string();
    let publisher = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let event_id = event.event_id.clone();
            let kind = format!("case.trace.{}", trace_kind_snake(event.kind.clone()));
            let detail = serde_json::to_value(&event)
                .unwrap_or_else(|_| serde_json::json!({"event_id": event_id}));
            if let Err(error) = state_for_publish
                .publish_event(&kind, Some(&case_for_publish), None, detail)
                .await
            {
                // A failed events-ledger append is logged loudly and consumes
                // the frame; the case ledger and jsonl remain the source of
                // truth, and `events.get_range` reads exactly what did land.
                tracing::error!(
                    case_id = %case_for_publish,
                    event_id = %event_id,
                    "case trace event publish failed: {error}"
                );
            }
        }
    });
    let result = tokio::task::spawn_blocking(move || {
        let mut notify = |event: &TraceEvent| {
            let _ = tx.send(event.clone());
        };
        work(&mut notify)
    })
    .await
    .map_err(|error| ForgeError::Internal(format!("case mutation task panic: {error}")))
    .and_then(|result| result);
    // The blocking task's closure owned the sender; its completion closed the
    // channel, so the publisher drains the remaining frames and exits.
    let _ = publisher.await;
    result
}

/// `case.add_item`: wrap `case_ops::propose_item_with`. The verified actor is
/// the proposer of record — a client cannot attribute discretionary work to
/// someone else.
pub(crate) async fn add_item(
    state: &Arc<ServerState>,
    actor: &str,
    case_id: &str,
    mut item: PlanItem,
    policy: &str,
) -> Result<CaseAddItemResult, ForgeError> {
    let policy_path = crate::agent_probe::resolve_policy_path(&state.root, policy)?;
    let root = state.root.clone();
    let actor = actor.to_string();
    let case_id = case_id.to_string();
    let plan_item_id = item.plan_item_id.clone();
    let proposed_by = actor.clone();
    item.proposed_by = Some(actor.clone());
    let case_id_for_task = case_id.clone();
    run_case_mutation(state, &case_id, move |notify| {
        sea_forge_case_runner::case_ops::propose_item_with(
            &root,
            &policy_path,
            &actor,
            &case_id_for_task,
            item,
            notify,
        )
    })
    .await?;
    Ok(CaseAddItemResult {
        ok: true,
        case_id,
        plan_item_id,
        proposed_by,
    })
}

/// `case.reopen`: wrap `case_ops::reopen_with`.
pub(crate) async fn reopen(
    state: &Arc<ServerState>,
    actor: &str,
    case_id: &str,
    policy: &str,
) -> Result<CaseReopenResult, ForgeError> {
    let policy_path = crate::agent_probe::resolve_policy_path(&state.root, policy)?;
    let root = state.root.clone();
    let actor = actor.to_string();
    let case_id = case_id.to_string();
    let case_id_for_task = case_id.clone();
    run_case_mutation(state, &case_id, move |notify| {
        sea_forge_case_runner::case_ops::reopen_with(
            &root,
            &policy_path,
            &actor,
            &case_id_for_task,
            notify,
        )
    })
    .await?;
    Ok(CaseReopenResult {
        ok: true,
        case_state: "active".into(),
        case_id,
    })
}

/// `case.terminate`: wrap `case_ops::terminate_with`.
pub(crate) async fn terminate(
    state: &Arc<ServerState>,
    actor: &str,
    case_id: &str,
    reason: &str,
    policy: &str,
) -> Result<CaseTerminateResult, ForgeError> {
    if reason.trim().is_empty() {
        return Err(ForgeError::Input(
            "case.terminate requires a non-empty reason".into(),
        ));
    }
    let policy_path = crate::agent_probe::resolve_policy_path(&state.root, policy)?;
    let root = state.root.clone();
    let actor = actor.to_string();
    let case_id = case_id.to_string();
    let reason = reason.to_string();
    let close_reason = format!("operator_terminated:{reason}");
    let case_id_for_task = case_id.clone();
    run_case_mutation(state, &case_id, move |notify| {
        sea_forge_case_runner::case_ops::terminate_with(
            &root,
            &policy_path,
            &actor,
            &case_id_for_task,
            &reason,
            notify,
        )
    })
    .await?;
    Ok(CaseTerminateResult {
        ok: true,
        case_state: "terminated".into(),
        close_reason,
        case_id,
    })
}

/// `human_task.complete`: wrap `case_ops::complete_human_task_with`. The
/// justification is mandatory at this boundary — a judgment with no recorded
/// reason is not a governed completion.
pub(crate) async fn human_task_complete(
    state: &Arc<ServerState>,
    actor: &str,
    case_id: &str,
    item_id: &str,
    justification: &str,
    policy: &str,
) -> Result<HumanTaskCompleteResult, ForgeError> {
    if justification.trim().is_empty() {
        return Err(ForgeError::Input(
            "human_task.complete requires a non-empty justification".into(),
        ));
    }
    let policy_path = crate::agent_probe::resolve_policy_path(&state.root, policy)?;
    let root = state.root.clone();
    let actor = actor.to_string();
    let case_id = case_id.to_string();
    let item_id = item_id.to_string();
    let note = justification.to_string();
    let item_id_for_result = item_id.clone();
    let case_id_for_task = case_id.clone();
    let exit_code = run_case_mutation(state, &case_id, move |notify| {
        sea_forge_case_runner::case_ops::complete_human_task_with(
            &root,
            &policy_path,
            &actor,
            &case_id_for_task,
            &item_id,
            Some(note.as_str()),
            notify,
        )
    })
    .await?;
    Ok(HumanTaskCompleteResult {
        ok: true,
        case_id,
        item_id: item_id_for_result,
        exit_code,
    })
}

/// Render an [`AdvanceOutcome`] for the wire.
fn advance_result(case_id: &str, outcome: AdvanceOutcome) -> CaseAdvanceResult {
    CaseAdvanceResult {
        ok: true,
        case_id: case_id.to_string(),
        state: outcome.state.to_string(),
        episodes: outcome
            .episodes
            .into_iter()
            .map(|episode| EpisodeSummary {
                item_id: episode.item_id,
                run_id: episode.run_id,
                settlement_status: serde_json::to_value(&episode.status)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_default(),
            })
            .collect(),
    }
}

/// Who is running the shared advance engine. This selects the engine's
/// [`AdvanceScope`] — never a second code path: the verb and the opt-in
/// supervisor both run through [`advance`], so event publishing
/// (cannot-miss) and governed execution (`execute_sandbox`) are literally the
/// same code for both callers.
#[derive(Clone, Debug)]
pub(crate) enum AdvanceCaller {
    /// `case.advance` / `item.execute`: scope follows the identity-gated
    /// request — `All`, or exactly one `Item`.
    Verb { item: Option<String> },
    /// The server-internal case-advance supervisor (T04C): the strict
    /// `SandboxedTask`-only scope the engine defines for unattended passes.
    Supervisor,
}

/// `case.advance` / `item.execute` / supervisor pass: run the shared advance
/// engine in-process, executing each sandboxed episode through the same
/// governed path `case_dispatch` uses (`execute_sandbox`), so there is no
/// second authority/execution path. The caller acquires the run-semaphore
/// permit and it is held for the duration of the episodes (bounded
/// concurrency, §11.1).
///
/// # Per-case serialization (T04C)
///
/// The whole mutation runs under this case's keyed async lock (acquired
/// here, before any append, released after the last one), so a client-driven
/// advance and a supervisor pass on the same case can never interleave
/// `case-events.jsonl` appends. Different cases lock independently; the map
/// entry is pruned on release once no pass holds or awaits it, so the key
/// map cannot grow without bound.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn advance(
    state: &Arc<ServerState>,
    actor: &str,
    actor_role: ActorRole,
    case_id: &str,
    caller: AdvanceCaller,
    policy: &str,
    timeout: u64,
    permit: tokio::sync::OwnedSemaphorePermit,
) -> Result<CaseAdvanceResult, ForgeError> {
    // Resolve once here to prove the cell-relative spelling is valid, then
    // pass the spelling itself: `execute_sandbox` resolves it again under
    // the same F-16 rule (an absolute path would fail its validation).
    let _ = crate::agent_probe::resolve_policy_path(&state.root, policy)?;
    let config = state.config();
    let root = state.root.clone();
    let actor = actor.to_string();
    let case_id = case_id.to_string();
    let scope = match caller {
        AdvanceCaller::Verb {
            item: Some(item_id),
        } => AdvanceScope::Item(item_id),
        AdvanceCaller::Verb { item: None } => AdvanceScope::All,
        AdvanceCaller::Supervisor => AdvanceScope::Supervisor,
    };
    let policy_for_task = policy.to_string();
    let case_id_for_task = case_id.clone();
    let case_lock = state.acquire_case_lock(&case_id).await;
    let result = run_case_mutation(state, &case_id, move |notify| {
        let mut execute_episode = |item: &PlanItem, run_id: &str| {
            // F-08: the episode's authority decision is evaluated for the
            // role the identity gate verified, exactly like a dispatched run.
            crate::case_dispatch::execute_sandbox(
                &config,
                item,
                &policy_for_task,
                &actor,
                "case_advance",
                &case_id_for_task,
                run_id,
                root.clone(),
                timeout,
                actor_role.clone(),
            )
        };
        advance_case(
            &root,
            &actor,
            &case_id_for_task,
            scope,
            notify,
            &mut execute_episode,
        )
    })
    .await;
    drop(case_lock);
    // Release-then-prune: the entry goes back to the map unlocked, then is
    // removed if no other pass holds or awaits it (strong_count == 1 is only
    // the map's own Arc). A task that fetched the Arc concurrently keeps the
    // count ≥ 2 and stays correctly serialized on the surviving mutex; the
    // last pass over a case prunes, so the key map tracks in-flight work,
    // not every case id ever seen.
    state.prune_case_lock(&case_id).await;
    let outcome = result?;
    drop(permit);
    Ok(advance_result(&case_id, outcome))
}

/// `artifact.get`: content-addressed fetch by SHA-256 digest over the
/// evidence records the kernel's artifact store already commits — no new
/// store, no new truth. Read-only. The returned bytes are re-hashed and must
/// equal the requested digest, or the read is an integrity error.
pub(crate) fn artifact_get(root: &Path, digest: &str) -> Result<ArtifactView, ForgeError> {
    if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ForgeError::Input(format!(
            "malformed artifact digest {digest:?}: expected a 64-character hex sha256"
        )));
    }
    for run_dir in candidate_run_dirs(root) {
        let evidence_path = run_dir.join("evidence.jsonl");
        if !crate::sfwp::size_within_cap(&evidence_path, crate::sfwp::MAX_JOURNAL_BYTES) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&evidence_path) else {
            continue;
        };
        for line in text.lines().filter(|line| !line.trim().is_empty()) {
            let Ok(record) = serde_json::from_str::<sea_forge_core::types::EvidenceRecord>(line)
            else {
                continue;
            };
            if record.sha256.as_deref() != Some(digest) {
                continue;
            }
            sea_forge_core::path::validate_relative_path(&record.uri)?;
            let path = run_dir.join(&record.uri);
            let metadata =
                std::fs::metadata(&path).map_err(|error| ForgeError::io("stat artifact", error))?;
            if metadata.len() > MAX_ARTIFACT_BYTES {
                return Err(ForgeError::Input(format!(
                    "artifact {} is {} bytes, over the {}-byte artifact.get ceiling",
                    record.uri,
                    metadata.len(),
                    MAX_ARTIFACT_BYTES
                )));
            }
            let bytes =
                std::fs::read(&path).map_err(|error| ForgeError::io("read artifact", error))?;
            let actual = sea_forge_evidence::sha256_bytes(&bytes);
            if actual != digest {
                return Err(ForgeError::Internal(format!(
                    "artifact_integrity_error: content at {} hashes to {actual}, not {digest}",
                    record.uri
                )));
            }
            let (content, content_base64) = match String::from_utf8(bytes.clone()) {
                Ok(text) => (Some(text), None),
                Err(_) => (None, Some(base64_encode(&bytes))),
            };
            return Ok(ArtifactView {
                digest: digest.to_string(),
                run_id: record.run_id,
                evidence_id: record.evidence_id,
                uri: record.uri,
                size_bytes: metadata.len(),
                content,
                content_base64,
            });
        }
    }
    Err(ForgeError::Input(format!(
        "artifact_not_found: no evidence record in this cell carries digest {digest}"
    )))
}

/// Every run directory this cell owns, deterministically ordered: standalone
/// `<root>/runs/*` and case-scoped `<root>/cases/*/runs/*`.
fn candidate_run_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let collect = |runs: &Path, out: &mut Vec<PathBuf>| {
        if let Ok(entries) = std::fs::read_dir(runs) {
            let mut found: Vec<PathBuf> = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect();
            found.sort();
            out.extend(found);
        }
    };
    collect(&root.join("runs"), &mut dirs);
    if let Ok(cases) = std::fs::read_dir(root.join("cases")) {
        let mut case_dirs: Vec<PathBuf> = cases
            .flatten()
            .map(|entry| entry.path().join("runs"))
            .filter(|path| path.is_dir())
            .collect();
        case_dirs.sort();
        for case_runs in case_dirs {
            collect(&case_runs, &mut dirs);
        }
    }
    dirs
}

/// Standard base64 (RFC 4648, with padding). Hand-rolled because adding the
/// `base64` crate to the server would be a new dependency for twenty lines of
/// encoding-only code, and dependency changes are ask-first here.
fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(triple >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(triple >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(triple >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[triple as usize & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn a_malformed_digest_is_refused_without_touching_disk() {
        let error = artifact_get(Path::new("/nonexistent"), "not-a-digest").unwrap_err();
        assert!(error.to_string().contains("malformed artifact digest"));
    }

    #[test]
    fn an_unknown_digest_is_a_typed_not_found() {
        let error = artifact_get(Path::new("/nonexistent"), &"a".repeat(64)).unwrap_err();
        assert!(error.to_string().contains("artifact_not_found"));
    }
}
