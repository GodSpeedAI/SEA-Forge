//! World pin record: promote and re-pin over the existing approval-gated
//! `semantic_change` world transition (WP-18, E section 7 steps 4 to 6).
//!
//! There is no second approval path. A promotion is the existing transition
//! (`CepAuthorityService::decide` with an `approval_id`), preceded by checks
//! that bind the recorded decision to the proposal, and followed by one
//! durable `world_pin` record in the same append-only CEP ledger. Nothing
//! else changes: a pin is a record, so every earlier decision, settlement and
//! case record keeps the `world_ref` it was made under (NT-07). Nothing here
//! rewrites a ref.
//!
//! Fail closed: a decision that is missing, unreadable, pending or unknown is
//! never an approval. Each refusal carries one code (NT-05) and the attempt is
//! retained as a `world_pin_attempt` record (AT-12: failed attempts are kept).
//!
//! The current pin is `to_world_ref` of the last `world_pin` record, or the
//! host's baseline world before any promotion. Registering the candidate
//! world is the host's configuration (the registry is built from it; both
//! worlds must be registered or the transition itself is refused).

use super::{
    approvable, escalation_key, text_at, ApprovalState, CepAuthorityError, CepAuthorityService,
    OperationKind,
};
use chrono::Utc;
use sea_forge_core::types::{Actor, AuthorityDecision, Verdict};
use sea_forge_ledger::LedgerEntry;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Ledger record kind of a pin.
pub const PIN_KIND: &str = "world_pin";
/// Ledger record kind of a refused promotion or re-pin.
pub const PIN_ATTEMPT_KIND: &str = "world_pin_attempt";
/// Ledger record kind of an explicit non-promotion.
pub const PIN_DECLINED_KIND: &str = "world_promotion_declined";

/// Most refused attempts one caller has retained per hour (F10).
pub const ATTEMPTS_PER_ACTOR_PER_HOUR: usize = 50;

/// The claim key a promotion's escalation request must carry: the proposal
/// digest (`sha256:<hex>`). Recorded with the decision; the approval is then
/// bound to it.
pub const CLAIM_PROPOSAL_DIGEST: &str = "proposal_digest";
/// The claim key a promotion's escalation request must also carry: the
/// `matrix_id` (`matrix:<hex>`) of the regression matrix the approver saw.
/// The approver decides on the proposal digest AND this matrix; a different
/// self-consistent matrix presented at promote is `DECISION_DIGEST_MISMATCH`.
pub const CLAIM_MATRIX_ID: &str = "matrix_id";
/// The claim key carrying the held-out rule's digest (`sha256:<hex>`, the
/// proposal's `held_out_rule.sha256`), bound for the same reason.
pub const CLAIM_HELDOUT_RULE: &str = "held_out_rule_sha256";
/// The claim key a re-pin's escalation request must carry.
pub const CLAIM_REPIN_DIGEST: &str = "repin_digest";

/// One refusal: a stable oracle code and a message. No side effect other
/// than the retained attempt record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinRefusal {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for PinRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PinRefusal {}

fn refuse<T>(code: &'static str, message: impl Into<String>) -> Result<T, PinRefusal> {
    Err(PinRefusal {
        code,
        message: message.into(),
    })
}

/// One lock per ledger root, held for the whole promote or re-pin sequence
/// (read the pin, authorize and decide, commit the pin). In-process only;
/// across processes `commit_pin` still re-checks the pin and takes a
/// per-position ledger key, so a second committer loses with `PIN_STALE`.
fn pin_lock(root: &Path) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>> = OnceLock::new();
    let mut map = LOCKS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    Arc::clone(map.entry(root.to_path_buf()).or_default())
}

/// A durable pin: the cell's world moved from `from_world_ref` to `to_world_ref`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinRecord {
    pub pin_id: String,
    /// `promoted` or `repinned`.
    pub kind: String,
    pub from_world_ref: String,
    pub to_world_ref: String,
    /// `sha256:<hex>` of the proposal (promotion) or of the re-pin statement.
    pub bound_digest: String,
    pub proposal_id: Option<String>,
    pub matrix_id: Option<String>,
    pub proposer: String,
    pub requester: String,
    pub approver: String,
    pub approval_id: String,
    pub operation_id: String,
    /// Ledger entry of the allow decision (the transition record).
    pub transition_ref: String,
    /// The pin this re-pin undoes, when it does.
    pub supersedes_pin: Option<String>,
}

/// Where the cell's pin stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinState {
    pub baseline: Option<String>,
    pub current: Option<String>,
    pub history: Vec<PinRecord>,
}

impl PinState {
    pub fn to_json(&self) -> Value {
        json!({"baseline": self.baseline, "current": self.current, "history": self.history})
    }
}

/// A promotion request: the proposal and matrix documents (RealityTrace
/// `revision-proposal.v1`, `regression-matrix.v1`) and the CEP request that
/// names the approval (`approval_id`) and carries the transition.
#[derive(Debug, Clone)]
pub struct PromoteRequest {
    pub proposal: Value,
    pub matrix: Value,
    pub envelope: Value,
}

/// A re-pin request: back to a world this cell was pinned to before.
#[derive(Debug, Clone)]
pub struct RepinRequest {
    pub to_world_ref: String,
    pub reason: String,
    pub envelope: Value,
}

// ------------------------------------------------------------------ digests

/// RFC 8785 canonical JSON for objects, arrays, strings, integers, booleans and
/// null (a float is refused). The same profile RealityTrace and
/// `sea-forge-server`'s `case_records::jcs` use; `sea_forge_core::canonical`
/// is a different profile (`jcs-nfc-v1`) and is not used for these digests.
fn jcs(value: &Value) -> Option<String> {
    Some(match value {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if n.is_f64() {
                return None;
            }
            n.to_string()
        }
        Value::String(s) => serde_json::to_string(s).ok()?,
        Value::Array(items) => {
            let parts: Option<Vec<String>> = items.iter().map(jcs).collect();
            format!("[{}]", parts?.join(","))
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            let mut parts = Vec::with_capacity(keys.len());
            for key in keys {
                parts.push(format!(
                    "{}:{}",
                    serde_json::to_string(key).ok()?,
                    jcs(&map[key])?
                ));
            }
            format!("{{{}}}", parts.join(","))
        }
    })
}

fn digest_hex(value: &Value) -> Option<String> {
    Some(hex::encode(Sha256::digest(jcs(value)?.as_bytes())))
}

/// The digest a document names itself by: sha256(JCS(document without `field`)).
fn self_digest(doc: &Value, field: &str) -> Option<String> {
    let mut body = doc.clone();
    body.as_object_mut()?.remove(field);
    digest_hex(&body)
}

/// The digest a re-pin's escalation request must carry in `repin_digest`.
pub fn repin_digest(from: &str, to: &str, reason: &str, of_pin: &str) -> String {
    let v = json!({"from": from, "to": to, "reason": reason, "of_pin": of_pin});
    format!("sha256:{}", digest_hex(&v).unwrap_or_default())
}

// ----------------------------------------------------------------- the store

fn pins_in(entries: &[LedgerEntry]) -> Vec<PinRecord> {
    entries
        .iter()
        .filter(|e| e.record_kind == PIN_KIND)
        .filter_map(|e| serde_json::from_value(e.payload.clone()).ok())
        .collect()
}

struct Authorized {
    operation_id: String,
    approval_id: String,
    requester: String,
    approver: String,
    transition_ref: String,
}

fn str_at<'a>(v: &'a Value, p: &str) -> &'a str {
    v.pointer(p).and_then(Value::as_str).unwrap_or_default()
}

impl CepAuthorityService<'_> {
    /// The current pin and the pin history, read from the ledger.
    pub fn pin_state(&self, baseline: Option<&str>) -> Result<PinState, CepAuthorityError> {
        let entries = self.ledger()?.read_entries()?;
        let history = pins_in(&entries);
        let current = history
            .last()
            .map(|p| p.to_world_ref.clone())
            .or_else(|| baseline.map(str::to_owned));
        Ok(PinState {
            baseline: baseline.map(str::to_owned),
            current,
            history,
        })
    }

    /// Retain a refused attempt. Best effort: a ledger that cannot be written
    /// still yields the refusal. Growth is bounded: fields are size-capped,
    /// and one caller keeps at most [`ATTEMPTS_PER_ACTOR_PER_HOUR`] attempts an
    /// hour; the attempt that reaches the cap is recorded as a rate-limit
    /// marker and later ones in the window are refused without a record.
    fn retain_attempt(
        &self,
        kind: &str,
        r: &PinRefusal,
        digest: Option<&str>,
        target: &str,
        who: &Actor,
    ) {
        let Ok(ledger) = self.ledger() else { return };
        let cap = |s: &str, n: usize| s.chars().take(n).collect::<String>();
        let actor = cap(&who.actor_id, 128);
        let now = Utc::now();
        let recent = ledger
            .read_entries()
            .map(|entries| {
                entries
                    .iter()
                    .filter(|e| e.record_kind == PIN_ATTEMPT_KIND)
                    .filter(|e| e.payload["actor"] == json!(actor))
                    .filter(|e| {
                        chrono::DateTime::parse_from_rfc3339(&e.committed_at).is_ok_and(|t| {
                            now.signed_duration_since(t) < chrono::Duration::hours(1)
                        })
                    })
                    .count()
            })
            .unwrap_or(0);
        let payload = if recent + 1 < ATTEMPTS_PER_ACTOR_PER_HOUR {
            json!({"attempt": kind, "outcome": "refused", "code": r.code,
                   "message": cap(&r.message, 512), "bound_digest": digest,
                   "target_world_ref": cap(target, 256), "actor": actor})
        } else if recent + 1 == ATTEMPTS_PER_ACTOR_PER_HOUR {
            json!({"attempt": kind, "outcome": "refused", "code": "ATTEMPTS_RATE_LIMITED",
                   "message": format!("further refused attempts by this caller are not retained for an hour (limit {ATTEMPTS_PER_ACTOR_PER_HOUR})"),
                   "bound_digest": null, "target_world_ref": cap(target, 256), "actor": actor})
        } else {
            return;
        };
        let _ = ledger.commit_typed(
            PIN_ATTEMPT_KIND,
            vec!["world-pin".into(), cap(target, 256)],
            &payload,
            vec![],
        );
    }

    /// The world a recorded decision was made under, read from the decision
    /// itself. A pin never changes it: evidence stays with its world (NT-07).
    pub fn evidence_world(&self, entry_ulid: &str) -> Result<Option<String>, CepAuthorityError> {
        let entries = self.ledger()?.read_entries()?;
        Ok(entries
            .iter()
            .find(|e| e.entry_ulid == entry_ulid)
            .and_then(|e| {
                let decision: AuthorityDecision = serde_json::from_value(e.payload.clone()).ok()?;
                let action = serde_json::to_value(&decision.action_request.action).ok()?;
                Some(text_at(&action, "/parameters/world_ref")).filter(|w| !w.is_empty())
            }))
    }

    /// Every retained refused attempt, oldest first.
    pub fn pin_attempts(&self) -> Result<Vec<Value>, CepAuthorityError> {
        Ok(self
            .ledger()?
            .read_entries()?
            .into_iter()
            .filter(|e| e.record_kind == PIN_ATTEMPT_KIND)
            .map(|e| e.payload)
            .collect())
    }

    /// Promote `proposal.candidate_world_ref` to be the cell's pinned world.
    ///
    /// Order: the proposal and the regression matrix (a failing or absent
    /// matrix is `REGRESSION_FAILED`, or the matrix's own more specific code),
    /// the pin is current (`PIN_STALE`), the recorded decision (denied
    /// `PROMOTION_DENIED`, unknown or pending `AUTHORITY_UNAVAILABLE`, expired
    /// `DECISION_EXPIRED`, bound to another proposal `DECISION_DIGEST_MISMATCH`),
    /// proposer is not approver, then the existing gated transition is decided
    /// and the pin recorded.
    pub fn promote(
        &self,
        req: &PromoteRequest,
        caller: &Actor,
        baseline: Option<&str>,
    ) -> Result<PinRecord, PinRefusal> {
        let target = str_at(&req.proposal, "/candidate_world_ref").to_owned();
        let digest = proposal_digest(&req.proposal).ok();
        let lock = pin_lock(self.root);
        let _held = lock.lock().unwrap_or_else(|p| p.into_inner());
        let out = self.promote_inner(req, caller, baseline);
        if let Err(r) = &out {
            self.retain_attempt("promote", r, digest.as_deref(), &target, caller);
        }
        out
    }

    fn promote_inner(
        &self,
        req: &PromoteRequest,
        caller: &Actor,
        baseline: Option<&str>,
    ) -> Result<PinRecord, PinRefusal> {
        let digest = proposal_digest(&req.proposal)?;
        let proposal = &req.proposal;
        let (old, cand) = (
            str_at(proposal, "/old_world_ref").to_owned(),
            str_at(proposal, "/candidate_world_ref").to_owned(),
        );
        let proposer = str_at(proposal, "/proposer").to_owned();
        check_matrix(&req.matrix, proposal, &digest, &old, &cand)?;
        if proposer != caller.actor_id {
            return refuse(
                "PROPOSER_MISMATCH",
                format!(
                    "the proposal names {proposer} as proposer; the request is made by {}",
                    caller.actor_id
                ),
            );
        }
        let state = self.pin_state(baseline).map_err(|e| PinRefusal {
            code: "AUTHORITY_UNAVAILABLE",
            message: e.to_string(),
        })?;
        if state.current.as_deref() != Some(old.as_str()) {
            return refuse(
                "PIN_STALE",
                format!(
                    "the proposal is against {old}; the pinned world is {}",
                    state.current.as_deref().unwrap_or("none")
                ),
            );
        }
        let matrix_id = str_at(&req.matrix, "/matrix_id").to_owned();
        let rule = str_at(proposal, "/held_out_rule/sha256").to_owned();
        if rule.is_empty() {
            return refuse("PROPOSAL_INVALID", "held_out_rule.sha256 is missing");
        }
        let auth = self.authorize(
            &req.envelope,
            caller,
            CLAIM_PROPOSAL_DIGEST,
            &digest,
            &[(CLAIM_MATRIX_ID, &matrix_id), (CLAIM_HELDOUT_RULE, &rule)],
            &old,
            &cand,
            &proposer,
        )?;
        let record = PinRecord {
            pin_id: format!("pin-{}", auth.operation_id),
            kind: "promoted".into(),
            from_world_ref: old,
            to_world_ref: cand,
            bound_digest: digest,
            proposal_id: Some(str_at(proposal, "/proposal_id").to_owned()),
            matrix_id: Some(str_at(&req.matrix, "/matrix_id").to_owned()),
            proposer,
            requester: auth.requester,
            approver: auth.approver,
            approval_id: auth.approval_id,
            operation_id: auth.operation_id,
            transition_ref: auth.transition_ref,
            supersedes_pin: None,
        };
        self.commit_pin(record, baseline)
    }

    /// Re-pin the cell to a world it was pinned to before, through the same
    /// gated transition. The recovery path for a promotion that proved wrong.
    pub fn repin(
        &self,
        req: &RepinRequest,
        caller: &Actor,
        baseline: Option<&str>,
    ) -> Result<PinRecord, PinRefusal> {
        let lock = pin_lock(self.root);
        let _held = lock.lock().unwrap_or_else(|p| p.into_inner());
        let out = self.repin_inner(req, caller, baseline);
        if let Err(r) = &out {
            self.retain_attempt("repin", r, None, &req.to_world_ref, caller);
        }
        out
    }

    fn repin_inner(
        &self,
        req: &RepinRequest,
        caller: &Actor,
        baseline: Option<&str>,
    ) -> Result<PinRecord, PinRefusal> {
        if req.reason.trim().is_empty() {
            return refuse("REPIN_REASON_REQUIRED", "a re-pin states its reason");
        }
        let state = self.pin_state(baseline).map_err(|e| PinRefusal {
            code: "AUTHORITY_UNAVAILABLE",
            message: e.to_string(),
        })?;
        let Some(current) = state.current.clone() else {
            return refuse("NO_PIN", "the cell has no pinned world to move from");
        };
        let known = state
            .baseline
            .iter()
            .cloned()
            .chain(
                state
                    .history
                    .iter()
                    .flat_map(|p| [p.from_world_ref.clone(), p.to_world_ref.clone()]),
            )
            .any(|w| w == req.to_world_ref);
        if !known || req.to_world_ref == current {
            return refuse(
                "REPIN_TARGET_UNKNOWN",
                format!(
                    "{} is not a world this cell was pinned to before, or is the current pin",
                    req.to_world_ref
                ),
            );
        }
        let of_pin = state
            .history
            .last()
            .map(|p| p.pin_id.clone())
            .unwrap_or_default();
        let digest = repin_digest(&current, &req.to_world_ref, &req.reason, &of_pin);
        let auth = self.authorize(
            &req.envelope,
            caller,
            CLAIM_REPIN_DIGEST,
            &digest,
            &[],
            &current,
            &req.to_world_ref,
            &caller.actor_id,
        )?;
        let record = PinRecord {
            pin_id: format!("pin-{}", auth.operation_id),
            kind: "repinned".into(),
            from_world_ref: current,
            to_world_ref: req.to_world_ref.clone(),
            bound_digest: digest,
            proposal_id: None,
            matrix_id: None,
            proposer: caller.actor_id.clone(),
            requester: auth.requester,
            approver: auth.approver,
            approval_id: auth.approval_id,
            operation_id: auth.operation_id,
            transition_ref: auth.transition_ref,
            supersedes_pin: state.history.last().map(|p| p.pin_id.clone()),
        };
        self.commit_pin(record, baseline)
    }

    /// Record that a candidate was not promoted, with the divergence (the
    /// residuals that motivated it) and the rationale. The non-promotion
    /// variant of AT-12.
    pub fn decline(
        &self,
        proposal: &Value,
        rationale: &str,
        caller: &Actor,
    ) -> Result<Value, PinRefusal> {
        let digest = proposal_digest(proposal)?;
        if rationale.trim().is_empty() {
            return refuse("RATIONALE_REQUIRED", "a non-promotion keeps its rationale");
        }
        let record = json!({
            "proposal_id": proposal["proposal_id"], "proposal_digest": digest,
            "candidate_world_ref": proposal["candidate_world_ref"],
            "divergence": {"diagnosis_residuals": proposal["diagnosis"]["residual_ids"],
                           "held_out_residuals": proposal["held_out"]["residual_ids"],
                           "intended_changes": proposal["intended_changes"]},
            "rationale": rationale, "declined_by": caller.actor_id,
        });
        let ledger = self.ledger().map_err(|e| PinRefusal {
            code: "AUTHORITY_UNAVAILABLE",
            message: e.to_string(),
        })?;
        ledger
            .commit_typed(
                PIN_DECLINED_KIND,
                vec![
                    "world-pin".into(),
                    str_at(proposal, "/candidate_world_ref").to_owned(),
                ],
                &record,
                vec![],
            )
            .map_err(|e| PinRefusal {
                code: "AUTHORITY_UNAVAILABLE",
                message: e.to_string(),
            })?;
        Ok(record)
    }

    /// Append the pin. Called under the pin lock; it still re-checks that the
    /// pin it moves from is the current one, and takes a per-position ledger
    /// key (`world-pin-slot:<n>`, n = pins so far) so that even a committer in
    /// another process loses (`PIN_STALE`) rather than forking the history.
    fn commit_pin(
        &self,
        record: PinRecord,
        baseline: Option<&str>,
    ) -> Result<PinRecord, PinRefusal> {
        let unavailable = |e: String| PinRefusal {
            code: "AUTHORITY_UNAVAILABLE",
            message: e,
        };
        let ledger = self.ledger().map_err(|e| unavailable(e.to_string()))?;
        let existing = ledger
            .read_entries()
            .map_err(|e| unavailable(e.to_string()))?;
        let pins = pins_in(&existing);
        // Replay or crash recovery: the same operation has one pin. It is a
        // success only while that pin is still the current one; once a later
        // pin superseded it, the answer says so instead of claiming success.
        if let Some(prior) = pins.iter().find(|p| p.operation_id == record.operation_id) {
            if pins.last().map(|p| p.pin_id.as_str()) == Some(prior.pin_id.as_str()) {
                return Ok(prior.clone());
            }
            return refuse(
                "PIN_SUPERSEDED",
                format!(
                    "{} already pinned this operation and a later pin superseded it; nothing was pinned now",
                    prior.pin_id
                ),
            );
        }
        let current = pins.last().map(|p| p.to_world_ref.as_str()).or(baseline);
        if current != Some(record.from_world_ref.as_str()) {
            return refuse(
                "PIN_STALE",
                format!(
                    "the pin moved from {} to {} while this operation was being decided",
                    record.from_world_ref,
                    current.unwrap_or("none")
                ),
            );
        }
        let key = format!("world-pin-slot:{}", pins.len());
        ledger
            .commit_typed_new(
                PIN_KIND,
                key,
                vec![
                    "world-pin".into(),
                    record.to_world_ref.clone(),
                    record.from_world_ref.clone(),
                ],
                &record,
                vec![record.transition_ref.clone()],
            )
            .map_err(|e| {
                let m = e.to_string();
                if m.contains("already committed") {
                    PinRefusal {
                        code: "PIN_STALE",
                        message: "another promotion took this position in the pin history".into(),
                    }
                } else {
                    unavailable(m)
                }
            })?;
        Ok(record)
    }

    /// The recorded decision, bound to `digest`, then the existing gated
    /// transition. Returns who asked, who approved and the transition record.
    #[allow(clippy::too_many_arguments)]
    fn authorize(
        &self,
        envelope: &Value,
        caller: &Actor,
        claim_key: &str,
        digest: &str,
        extra_claims: &[(&str, &str)],
        source: &str,
        target: &str,
        proposer: &str,
    ) -> Result<Authorized, PinRefusal> {
        let unavailable = |m: String| PinRefusal {
            code: "AUTHORITY_UNAVAILABLE",
            message: m,
        };
        let request = super::parse_request(envelope, self.worlds).map_err(|e| match e {
            CepAuthorityError::Transition(_) | CepAuthorityError::World(_) => PinRefusal {
                code: "TRANSITION_REFUSED",
                message: e.to_string(),
            },
            other => PinRefusal {
                code: "REQUEST_INVALID",
                message: other.to_string(),
            },
        })?;
        let Some(t) = request.transition.as_ref() else {
            return refuse(
                "TRANSITION_REFUSED",
                "the request is not a world_transition",
            );
        };
        if request.kind != OperationKind::WorldTransition
            || t.transition_kind != "semantic_change"
            || t.source_world_ref != source
            || t.target_world_ref != target
        {
            return refuse(
                "TRANSITION_REFUSED",
                format!("the request must be a semantic_change from {source} to {target}"),
            );
        }
        // Without a named approval there is no recorded decision to rely on.
        let Some(approval_id) = request.approval_ref.clone() else {
            return refuse(
                "AUTHORITY_UNAVAILABLE",
                "the request names no approval: no recorded decision",
            );
        };
        let entries = self
            .ledger()
            .and_then(|l| Ok(l.read_entries()?))
            .map_err(|e| unavailable(e.to_string()))?;
        let Some(op) = approval_id.strip_prefix("apr-") else {
            return refuse(
                "AUTHORITY_UNAVAILABLE",
                format!("{approval_id} is not an approval id"),
            );
        };
        // The escalation this approval resolves.
        let Some(escalation) = entries
            .iter()
            .find(|e| e.idempotency_key.as_deref() == Some(&escalation_key(op)))
        else {
            return refuse(
                "AUTHORITY_UNAVAILABLE",
                format!("no decision is recorded for {approval_id}: unknown is never approved"),
            );
        };
        let decision: AuthorityDecision = serde_json::from_value(escalation.payload.clone())
            .map_err(|e| unavailable(format!("the recorded decision is unreadable: {e}")))?;
        match decision.verdict {
            Verdict::Deny => {
                return refuse(
                    "PROMOTION_DENIED",
                    format!("authority denied the transition ({})", decision.reason),
                );
            }
            Verdict::Allow => {
                return refuse(
                    "AUTHORITY_UNAVAILABLE",
                    "the recorded decision is not an escalation to approve",
                );
            }
            Verdict::Escalate => {}
        }
        if !approvable(&decision) {
            return refuse(
                "AUTHORITY_UNAVAILABLE",
                "the escalation cannot be satisfied by an approval",
            );
        }
        let action = serde_json::to_value(&decision.action_request.action)
            .map_err(|e| unavailable(e.to_string()))?;
        let Some(standing) = self.standing_in(&entries, &approval_id, Utc::now()) else {
            return refuse(
                "AUTHORITY_UNAVAILABLE",
                format!("{approval_id} has no standing"),
            );
        };
        match standing.state {
            ApprovalState::Pending => {
                return refuse(
                    "AUTHORITY_UNAVAILABLE",
                    format!("{approval_id} is still pending: no decision yet"),
                );
            }
            ApprovalState::Rejected => {
                return refuse(
                    "PROMOTION_DENIED",
                    format!(
                        "{approval_id} was rejected by {}",
                        standing.resolved_by.clone().unwrap_or_default()
                    ),
                );
            }
            ApprovalState::Expired => {
                return refuse(
                    "DECISION_EXPIRED",
                    format!(
                        "{approval_id} expired at {}",
                        standing.expires_at.to_rfc3339()
                    ),
                );
            }
            ApprovalState::Approved | ApprovalState::Used => {}
        }
        // The decision is about this candidate and this proposal, not another.
        let claimed = action
            .pointer(&format!("/parameters/claims/{claim_key}"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        // The extra claims (the matrix the approver saw, the held-out rule)
        // must equal what is presented now, in the decision AND the request.
        for (key, want) in extra_claims {
            let decided = action
                .pointer(&format!("/parameters/claims/{key}"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let requested = request
                .claims
                .as_ref()
                .and_then(|c| c.get(*key))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if decided != *want || requested != *want {
                return refuse(
                    "DECISION_DIGEST_MISMATCH",
                    format!(
                        "{approval_id} was decided with {key} {decided:?}; this request presents {want:?}"
                    ),
                );
            }
        }
        if text_at(&action, "/resource_id") != target || claimed != digest {
            return refuse(
                "DECISION_DIGEST_MISMATCH",
                format!(
                    "{approval_id} was decided for {} with {claim_key} {:?}; this request is for {target} with {digest}",
                    text_at(&action, "/resource_id"),
                    claimed
                ),
            );
        }
        if request
            .claims
            .as_ref()
            .and_then(|c| c.get(claim_key))
            .and_then(Value::as_str)
            != Some(digest)
        {
            return refuse(
                "DECISION_DIGEST_MISMATCH",
                format!("the request's own {claim_key} differs from the decision's"),
            );
        }
        let approver = standing.resolved_by.clone().unwrap_or_default();
        if approver.is_empty() {
            return refuse(
                "AUTHORITY_UNAVAILABLE",
                format!("{approval_id} names no approver"),
            );
        }
        // Proposer is not the approver (the same rule family as I-2); the requester cannot
        // resolve its own request either (enforced when the approval was resolved).
        if approver == proposer || approver == standing.requester || approver == caller.actor_id {
            return refuse(
                "PROPOSER_IS_APPROVER",
                format!("{approver} approved a transition it proposed or requested"),
            );
        }
        if standing.requester != caller.actor_id {
            return refuse(
                "PROPOSER_MISMATCH",
                "the approval was requested by another principal",
            );
        }
        let transition_ref = if standing.state == ApprovalState::Used {
            // A crash after the transition was decided and before the pin was recorded:
            // finish from the recorded transition, never decide twice.
            let facts = self.transitions().map_err(|e| unavailable(e.to_string()))?;
            let Some(done) = facts.into_iter().find(|f| {
                f.approval_id.as_deref() == Some(approval_id.as_str())
                    && text_at(&f.record, "/target_world_ref") == target
                    && text_at(&f.record, "/source_world_ref") == source
                    && f.requester == caller.actor_id
            }) else {
                return refuse(
                    "DECISION_ALREADY_USED",
                    format!("{approval_id} was spent on another operation"),
                );
            };
            return Ok(Authorized {
                operation_id: done.operation_id,
                approval_id,
                requester: done.requester,
                approver,
                transition_ref: done.ledger_entry,
            });
        } else {
            let decided = self.decide(envelope, caller).map_err(|e| PinRefusal {
                code: match e {
                    CepAuthorityError::Approval(_) => "PROMOTION_DENIED",
                    CepAuthorityError::Transition(_) | CepAuthorityError::World(_) => {
                        "TRANSITION_REFUSED"
                    }
                    _ => "AUTHORITY_UNAVAILABLE",
                },
                message: e.to_string(),
            })?;
            if decided.decision.verdict != Verdict::Allow {
                return refuse(
                    "PROMOTION_DENIED",
                    format!(
                        "the transition was not allowed: {}",
                        decided.decision.reason
                    ),
                );
            }
            decided.committed.entry_ulid().to_owned()
        };
        Ok(Authorized {
            operation_id: request.operation_id,
            approval_id,
            requester: caller.actor_id.clone(),
            approver,
            transition_ref,
        })
    }
}

/// The proposal's digest (`sha256:<hex>`), after checking that its id is the
/// digest of its own content and that it names what a promotion needs.
pub fn proposal_digest(proposal: &Value) -> Result<String, PinRefusal> {
    let bad = |m: &str| PinRefusal {
        code: "PROPOSAL_INVALID",
        message: m.to_string(),
    };
    if proposal["contract"] != json!("revision-proposal.v1") {
        return Err(bad("not a revision-proposal.v1 document"));
    }
    for f in [
        "proposer",
        "old_world_ref",
        "candidate_world_ref",
        "semantic_diff_sha256",
    ] {
        if str_at(proposal, &format!("/{f}")).is_empty() {
            return Err(bad(&format!("{f} is missing")));
        }
    }
    if proposal["old_world_ref"] == proposal["candidate_world_ref"] {
        return Err(bad("the candidate world is the old world"));
    }
    if !proposal["held_out"]["episodes"].is_array()
        || proposal["held_out"]["episodes"]
            .as_array()
            .is_some_and(Vec::is_empty)
        || proposal["intended_changes"]
            .as_array()
            .is_none_or(Vec::is_empty)
    {
        return Err(bad(
            "the held-out set and the intended changes are required",
        ));
    }
    let want = self_digest(proposal, "proposal_id").ok_or_else(|| bad("not canonicalisable"))?;
    if str_at(proposal, "/proposal_id") != format!("proposal:{want}") {
        return Err(bad("proposal_id is not the digest of the document"));
    }
    Ok(format!("sha256:{want}"))
}

/// The regression matrix must be present, be this proposal's, agree with
/// itself, and pass. Absent or failing: `REGRESSION_FAILED` (or the matrix's
/// own more specific refusal code, which NT-21 expects). An escalated matrix
/// (a listed weakening) needs an approval class this path does not have:
/// refused, not downgraded.
fn check_matrix(
    matrix: &Value,
    proposal: &Value,
    digest: &str,
    old: &str,
    cand: &str,
) -> Result<(), PinRefusal> {
    let reg = |m: &str| PinRefusal {
        code: "REGRESSION_FAILED",
        message: m.to_string(),
    };
    if !matrix.is_object() || matrix["contract"] != json!("regression-matrix.v1") {
        return Err(reg(
            "no regression matrix: skipping the regression is not an option",
        ));
    }
    let want =
        self_digest(matrix, "matrix_id").ok_or_else(|| reg("matrix is not canonicalisable"))?;
    if str_at(matrix, "/matrix_id") != format!("matrix:{want}") {
        return Err(reg("matrix_id is not the digest of the matrix"));
    }
    if str_at(matrix, "/proposal_id") != str_at(proposal, "/proposal_id")
        || str_at(matrix, "/proposal_digest") != digest
        || str_at(matrix, "/old_world_ref") != old
        || str_at(matrix, "/candidate_world_ref") != cand
    {
        return Err(reg("the matrix is for another proposal or other worlds"));
    }
    let rows = matrix["rows"]
        .as_array()
        .ok_or_else(|| reg("matrix has no rows"))?;
    if rows.is_empty() {
        return Err(reg(
            "the matrix has no rows: an empty matrix proves no regression check ran",
        ));
    }
    let failing: Vec<&Value> = rows
        .iter()
        .filter(|r| r["result"] != json!("pass"))
        .collect();
    let verdict_fail = matrix["verdict"] != json!("pass");
    if verdict_fail != !failing.is_empty() {
        return Err(reg("the matrix verdict disagrees with its rows"));
    }
    if verdict_fail {
        let code = match matrix["refusal_code"].as_str() {
            Some("LOST_CAPABILITY") => "LOST_CAPABILITY",
            Some("UNLISTED_WEAKENING") => "UNLISTED_WEAKENING",
            _ => "REGRESSION_FAILED",
        };
        return Err(PinRefusal {
            code,
            message: format!("{} matrix row(s) fail", failing.len()),
        });
    }
    if matrix["escalated"] == json!(true) || rows.iter().any(|r| r["escalated"] == json!(true)) {
        return Err(PinRefusal {
            code: "ESCALATED_APPROVAL_REQUIRED",
            message: "the matrix lists a weakening, which needs the escalated approval class; this path has none".into(),
        });
    }
    check_coverage(rows, proposal)
}

/// Structural completeness of a passing matrix against the proposal that
/// names it. RealityTrace's rows carry `old_expectation` / `new_expectation`
/// (the expectation ids of the old and candidate worlds) and `episode_ref`;
/// a proposal's `intended_changes[].element` is a candidate-world expectation
/// id and `supersedes` lists old-world ids. So: every intended element is the
/// `new_expectation` (or, for a removal, the `old_expectation`) of some row,
/// every superseded id is the `old_expectation` of some row, and every episode
/// the proposal names (diagnosis and held-out) is the `episode_ref` of some
/// row. Rows for further compared episodes are allowed (RealityTrace compares
/// a superset).
///
/// LIMIT, stated plainly: SEA-Forge cannot recompute the transitions without
/// RealityTrace. The rows' classes and results are the proposer-supplied
/// matrix's word; what binds them is the approver's `matrix_id` claim
/// ([`CLAIM_MATRIX_ID`]), not a recomputation here. Completeness narrows what a
/// forged matrix can omit; it does not make the rows true, and it cannot tell
/// whether a row's element exists in either world (that needs RealityTrace).
fn check_coverage(rows: &[Value], proposal: &Value) -> Result<(), PinRefusal> {
    let reg = |m: String| PinRefusal {
        code: "REGRESSION_FAILED",
        message: m,
    };
    let base = |e: &str| e.split('@').next().unwrap_or(e).to_owned();
    let has = |field: &str, id: &str| rows.iter().any(|r| r[field].as_str() == Some(id));
    for change in proposal["intended_changes"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if let Some(element) = change["element"].as_str() {
            if !has("new_expectation", element) && !has("old_expectation", element) {
                return Err(reg(format!(
                    "the matrix has no row for the proposal's element {element}"
                )));
            }
        }
        for old in change["supersedes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !has("old_expectation", old) {
                return Err(reg(format!(
                    "the matrix has no row for the superseded element {old}"
                )));
            }
        }
    }
    let row_episodes: Vec<String> = rows
        .iter()
        .filter_map(|r| r["episode_ref"].as_str().map(base))
        .collect();
    for set in ["diagnosis", "held_out"] {
        for ep in proposal[set]["episodes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !row_episodes.contains(&base(ep)) {
                return Err(reg(format!(
                    "the matrix has no row for the proposal's episode {ep}"
                )));
            }
        }
    }
    Ok(())
}
