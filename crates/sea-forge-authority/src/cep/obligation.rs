//! Obligation settlement from RealityTrace's own ledger (Horizon HORIZON-001 1.1.0, REQ-NAV-011/012).
//!
//! An operation whose allow decision recorded `claims.modality = "obligation"` is settled here, not by the
//! evidence-item rule in `settle_with`. Everything that decides it is read from an owner's record:
//!
//! - the expectation's digest, the RealityTrace threshold (id, ledger, position T), the window deadline, the
//!   close grace and the expected verifier argv digest were frozen in the allow decision's claims;
//! - the threshold must exist in RealityTrace's ledger at exactly T;
//! - C, the RealityTrace ledger head at window close, is read by SEA-Forge itself when the closure is recorded,
//!   and the closure's `committed_at` is SEA-Forge's own clock: a closure later than deadline + grace cannot
//!   prove C on time, and the result is `unresolved` / `rejected`;
//! - the discrimination run (position B > T) must not have observed the statement before acting;
//! - execution happened no earlier than SEA-Forge received the operation's first execution evidence packet,
//!   so a verification run counts only if RealityTrace recorded it starting after that;
//! - each verification run is admissible when its ledger position is in (B, C]; it is a known result only if
//!   RealityTrace recorded a completion time at or before the deadline, an exit code, no crash and the
//!   expected argv; otherwise it is undefined. Only known results decide.
//!
//! | admissible known results | epistemic_status | settlement |
//! |---|---|---|
//! | 0 and no non-zero | supported | settled |
//! | non-zero and no 0 | contradicted | rejected |
//! | both | inconsistent | unsettled |
//! | none | unresolved | rejected (asserted occurrence unsupported) |
//!
//! The window is closed and the evidence frozen at C, so every outcome here is final.

use super::{
    allowed_operation, committed_ref, settlement_status_of, Actor, CepAuthorityError,
    CepAuthorityService, SettlementOutcome, PROFILE_VERSION, SETTLE_SUFFIX_FINAL,
};
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

/// One row of an evidence source's own ledger, read back after that ledger's chain verified.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRow {
    pub ledger_id: String,
    pub ordinal: u64,
    pub record_kind: String,
    pub payload: Value,
}

/// Read access to RealityTrace's ledger for obligation settlement.
pub trait ObligationSource {
    /// Every row of the ledger `ledger_id`, after verifying that ledger's chain. `Err` says why it cannot be read.
    fn ledger_rows(&self, ledger_id: &str) -> Result<Vec<SourceRow>, String>;
}

pub const CLOSURE_KIND: &str = "settlement_closure";
const CLOSURE_KEY: &str = "cep-closure:";
pub const DEFAULT_CLOSE_GRACE_S: i64 = 30;

/// The obligation frozen in an allow decision's claims.
#[derive(Debug, Clone, PartialEq)]
pub struct Obligation {
    pub expectation_digest: String,
    pub threshold_id: String,
    pub threshold_ledger: String,
    pub threshold_position: u64,
    pub closes_at: DateTime<Utc>,
    pub close_grace_s: i64,
    pub verifier_argv_digest: String,
}

impl Obligation {
    fn latest_closure(&self) -> DateTime<Utc> {
        self.closes_at + Duration::seconds(self.close_grace_s)
    }
}

fn claim_text<'a>(claims: &'a Map<String, Value>, key: &str) -> Result<&'a str, CepAuthorityError> {
    claims.get(key).and_then(Value::as_str).ok_or_else(|| {
        CepAuthorityError::Evidence(format!("the allow decision's claims carry no {key}"))
    })
}

fn claim_u64(claims: &Map<String, Value>, key: &str) -> Result<u64, CepAuthorityError> {
    claims.get(key).and_then(Value::as_u64).ok_or_else(|| {
        CepAuthorityError::Evidence(format!(
            "the allow decision's claims carry no non-negative integer {key}"
        ))
    })
}

/// `None` when the decision asserted no modality (the evidence-item rule applies); an error for a modality
/// this version cannot settle, or for an obligation missing a frozen fact.
pub fn obligation_of(
    claims: Option<&Map<String, Value>>,
) -> Result<Option<Obligation>, CepAuthorityError> {
    let Some(claims) = claims else {
        return Ok(None);
    };
    let Some(modality) = claims.get("modality") else {
        return Ok(None);
    };
    if modality.as_str() != Some("obligation") {
        return Err(CepAuthorityError::Evidence(format!(
            "modality {modality} is not supported by the configured verifiers; only obligation settles"
        )));
    }
    let closes_at = DateTime::parse_from_rfc3339(claim_text(claims, "window_closes_at")?)
        .map_err(|e| CepAuthorityError::Evidence(format!("window_closes_at is not RFC 3339: {e}")))?
        .with_timezone(&Utc);
    let close_grace_s = match claims.get("close_grace_s") {
        None => DEFAULT_CLOSE_GRACE_S,
        Some(v) => v
            .as_i64()
            .filter(|g| (0..=3600).contains(g))
            .ok_or_else(|| {
                CepAuthorityError::Evidence(
                    "close_grace_s must be an integer from 0 to 3600".into(),
                )
            })?,
    };
    Ok(Some(Obligation {
        expectation_digest: claim_text(claims, "expectation_digest")?.to_string(),
        threshold_id: claim_text(claims, "threshold_id")?.to_string(),
        threshold_ledger: claim_text(claims, "threshold_ledger")?.to_string(),
        threshold_position: claim_u64(claims, "threshold_position")?,
        closes_at,
        close_grace_s,
        verifier_argv_digest: claim_text(claims, "verifier_argv_digest")?.to_string(),
    }))
}

/// `sha256:<hex>` of the argv as compact JSON: what the requester froze in `verifier_argv_digest`.
pub fn argv_digest(argv: &[Value]) -> String {
    let text = serde_json::to_string(argv).unwrap_or_default();
    format!("sha256:{}", hex::encode(Sha256::digest(text.as_bytes())))
}

fn parse_time(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

/// The observation row RealityTrace recorded for `observation_id`.
fn observation<'a>(rows: &'a [SourceRow], observation_id: &str) -> Option<&'a SourceRow> {
    rows.iter().find(|row| {
        row.record_kind == "observation.recorded"
            && row
                .payload
                .pointer("/observations/0/observation_id")
                .and_then(Value::as_str)
                == Some(observation_id)
    })
}

fn inline(row: &SourceRow) -> Option<&Value> {
    row.payload.pointer("/representations/0/inline_content")
}

/// When SEA-Forge first received execution evidence for the operation: no run before it saw the effect.
fn execution_floor(
    entries: &[sea_forge_ledger::LedgerEntry],
    operation_id: &str,
) -> Option<DateTime<Utc>> {
    entries
        .iter()
        .filter(|e| {
            e.record_kind == "evidence_packet" && e.subject_refs.iter().any(|s| s == operation_id)
        })
        .filter_map(|e| parse_time(&e.committed_at))
        .min()
}

/// The discrimination run (B) must be a completed, uncrashed run of exactly the pinned verifier argv, after the
/// threshold, with a known non-zero exit: only that proves the statement did not already hold before acting.
/// A crashed run, a run of another command, or one with no recorded result proves nothing about this verifier.
fn check_discrimination(
    row: &SourceRow,
    obligation: &Obligation,
    execution_floor: Option<DateTime<Utc>>,
) -> Result<(), CepAuthorityError> {
    let refuse = |why: &str| {
        Err(CepAuthorityError::Evidence(format!(
            "the discrimination run {why}"
        )))
    };
    if row.ledger_id != obligation.threshold_ledger || row.ordinal <= obligation.threshold_position
    {
        return refuse("is not after the threshold; the expectation was not formed first");
    }
    let Some(content) = inline(row) else {
        return refuse("has no recorded result");
    };
    if content.get("crashed").and_then(Value::as_bool) != Some(false) {
        return refuse("crashed or does not say it completed; it has no known result");
    }
    let Some(completed) = content
        .get("completed_at")
        .and_then(Value::as_str)
        .and_then(parse_time)
    else {
        return refuse("has no recorded completion; it has no known result");
    };
    // A baseline taken after the operation executed says nothing about the state before acting.
    if execution_floor.is_some_and(|floor| completed > floor) {
        return refuse(
            "completed after the execution evidence; it is not a baseline before acting",
        );
    }
    let argv = content
        .pointer("/invocation/argv")
        .and_then(Value::as_array);
    if !argv.is_some_and(|a| argv_digest(a) == obligation.verifier_argv_digest) {
        return refuse("is not a run of the pinned verifier argv");
    }
    match content.get("exit_code").and_then(Value::as_i64) {
        Some(0) => {
            refuse("observed the statement before acting; the operation should not have executed")
        }
        Some(_) => Ok(()),
        None => refuse("has no known result"),
    }
}

/// Three-valued result of one verification run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunResult {
    Known0,
    KnownNonZero,
    Undefined,
}

/// The admissibility frame of one obligation.
pub struct Frame<'a> {
    pub obligation: &'a Obligation,
    pub b: u64,
    pub c: u64,
    pub execution_floor: DateTime<Utc>,
}

/// `None` when the run is not admissible (outside (B, C], or started before execution); otherwise its result.
pub fn classify(row: Option<&SourceRow>, frame: &Frame<'_>) -> Option<RunResult> {
    // A claimed run RealityTrace never recorded asserts an occurrence nothing supports.
    let Some(row) = row else {
        return Some(RunResult::Undefined);
    };
    if row.ledger_id != frame.obligation.threshold_ledger
        || row.ordinal <= frame.b
        || row.ordinal > frame.c
    {
        return None;
    }
    let Some(content) = inline(row) else {
        return Some(RunResult::Undefined);
    };
    let started = content
        .get("started_at")
        .and_then(Value::as_str)
        .and_then(parse_time);
    match started {
        Some(t) if t > frame.execution_floor => {}
        Some(_) => return None,
        None => return Some(RunResult::Undefined),
    }
    let completed = content
        .get("completed_at")
        .and_then(Value::as_str)
        .and_then(parse_time);
    if !completed.is_some_and(|t| t <= frame.obligation.closes_at) {
        return Some(RunResult::Undefined);
    }
    if content.get("crashed").and_then(Value::as_bool) != Some(false) {
        return Some(RunResult::Undefined);
    }
    let argv = content
        .pointer("/invocation/argv")
        .and_then(Value::as_array);
    if !argv.is_some_and(|a| argv_digest(a) == frame.obligation.verifier_argv_digest) {
        return Some(RunResult::Undefined);
    }
    match content.get("exit_code").and_then(Value::as_i64) {
        Some(0) => Some(RunResult::Known0),
        Some(_) => Some(RunResult::KnownNonZero),
        None => Some(RunResult::Undefined),
    }
}

/// The REQ-NAV-012 table: `(epistemic_status, settlement_status)`.
pub fn judge(results: &[RunResult]) -> (&'static str, &'static str) {
    let pass = results.contains(&RunResult::Known0);
    let fail = results.contains(&RunResult::KnownNonZero);
    match (pass, fail) {
        (true, true) => ("inconsistent", "unsettled"),
        (true, false) => ("supported", "settled"),
        (false, true) => ("contradicted", "rejected"),
        (false, false) => ("unresolved", "rejected"),
    }
}

/// What `authority_closure` recorded.
#[derive(Debug)]
pub struct ClosureAck {
    pub b: u64,
    pub c: u64,
    pub committed_at: String,
    pub on_time: bool,
    pub replayed: bool,
    pub entry_ulid: String,
}

impl CepAuthorityService<'_> {
    fn obligation_for(
        &self,
        entries: &[sea_forge_ledger::LedgerEntry],
        operation_id: &str,
    ) -> Result<(super::AllowedOperation, Obligation), CepAuthorityError> {
        let allowed = allowed_operation(entries, operation_id).ok_or_else(|| {
            CepAuthorityError::Evidence(format!("{operation_id} has no committed allow decision"))
        })?;
        let obligation = obligation_of(allowed.claims.as_ref())?.ok_or_else(|| {
            CepAuthorityError::Evidence(format!("{operation_id} asserted no obligation"))
        })?;
        Ok((allowed, obligation))
    }

    /// Whether `operation_id`'s allow decision asserted a modality, so `settle_obligation` decides it.
    pub fn asserts_obligation(&self, operation_id: &str) -> Result<bool, CepAuthorityError> {
        let entries = self.ledger()?.read_entries()?;
        Ok(allowed_operation(&entries, operation_id)
            .and_then(|a| a.claims)
            .is_some_and(|c| c.contains_key("modality")))
    }

    /// Record the closure of an obligation's window (`authority_closure`). SEA-Forge reads RealityTrace's
    /// ledger head itself (C) and stamps the record with its own clock. The first closure wins.
    pub fn record_closure(
        &self,
        operation_id: &str,
        discrimination_observation_id: &str,
        verification_observation_ids: &[String],
        caller: &Actor,
        source: &dyn ObligationSource,
    ) -> Result<ClosureAck, CepAuthorityError> {
        if verification_observation_ids.len() > 256 {
            return Err(CepAuthorityError::Evidence(
                "at most 256 verification runs per closure".into(),
            ));
        }
        let ledger = self.ledger()?;
        let entries = ledger.read_entries()?;
        let (_, obligation) = self.obligation_for(&entries, operation_id)?;
        let key = format!("{CLOSURE_KEY}{operation_id}");
        if let Some(done) = entries.iter().find(|e| {
            e.record_kind == CLOSURE_KIND && e.idempotency_key.as_deref() == Some(key.as_str())
        }) {
            return Ok(closure_ack(done, &obligation, true));
        }
        if Utc::now() < obligation.closes_at {
            return Err(CepAuthorityError::Evidence(
                "the window is still open; a closure is recorded once it closes".into(),
            ));
        }
        self.gate(&ledger, "settlement_declaration", operation_id, caller)?;
        let rows = source
            .ledger_rows(&obligation.threshold_ledger)
            .map_err(|why| {
                CepAuthorityError::Evidence(format!("RealityTrace's ledger cannot be read: {why}"))
            })?;
        let c = rows.iter().map(|r| r.ordinal).max().unwrap_or(0);
        let discrimination =
            observation(&rows, discrimination_observation_id).ok_or_else(|| {
                CepAuthorityError::Evidence(format!(
                    "RealityTrace recorded no observation {discrimination_observation_id}"
                ))
            })?;
        // A bad discrimination run is refused before any closure is frozen.
        check_discrimination(
            discrimination,
            &obligation,
            execution_floor(&entries, operation_id),
        )?;
        let record = json!({
            "operation_id": operation_id,
            "ledger_id": obligation.threshold_ledger,
            "b": discrimination.ordinal,
            "c": c,
            "discrimination_observation_id": discrimination_observation_id,
            "verification_observation_ids": verification_observation_ids,
            "window_closes_at": obligation.closes_at.to_rfc3339(),
            "recorded_by": caller.actor_id,
        });
        let subjects = vec!["cognate".to_string(), operation_id.to_string()];
        // A failure here means a concurrent closure won (first wins); either way the committed one is read back.
        let _ = ledger.commit_typed_new(CLOSURE_KIND, key.clone(), subjects, &record, vec![]);
        let entries = ledger.read_entries()?;
        let done = entries
            .iter()
            .find(|e| {
                e.record_kind == CLOSURE_KIND && e.idempotency_key.as_deref() == Some(key.as_str())
            })
            .ok_or_else(|| CepAuthorityError::Internal("closure was not committed".into()))?;
        Ok(closure_ack(done, &obligation, false))
    }

    /// Settle an obligation from RealityTrace's ledger (see the module documentation for the rule).
    pub fn settle_obligation(
        &self,
        operation_id: &str,
        caller: &Actor,
        source: &dyn ObligationSource,
    ) -> Result<SettlementOutcome, CepAuthorityError> {
        let ledger = self.ledger()?;
        let entries = ledger.read_entries()?;
        let (allowed, obligation) = self.obligation_for(&entries, operation_id)?;
        let final_key = format!("{SETTLE_SUFFIX_FINAL}{operation_id}");
        if let Some(done) = entries
            .iter()
            .find(|e| e.idempotency_key.as_deref() == Some(final_key.as_str()))
        {
            return Ok(SettlementOutcome {
                status: settlement_status_of(&done.payload),
                final_status: true,
                envelope: done.payload.clone(),
                committed: committed_ref(done),
                replayed: true,
            });
        }
        let now = Utc::now();
        if now < obligation.closes_at {
            return Err(CepAuthorityError::SettlementPending(
                "the window is still open; settlement is pending".into(),
            ));
        }
        let closure = entries.iter().find(|e| {
            e.record_kind == CLOSURE_KIND
                && e.idempotency_key.as_deref()
                    == Some(format!("{CLOSURE_KEY}{operation_id}").as_str())
        });
        if closure.is_none() && now <= obligation.latest_closure() {
            return Err(CepAuthorityError::SettlementPending(
                "the closure has not been recorded yet; settlement is pending".into(),
            ));
        }
        self.gate(&ledger, "settlement_declaration", operation_id, caller)?;
        let rows = source
            .ledger_rows(&obligation.threshold_ledger)
            .map_err(|why| {
                CepAuthorityError::Evidence(format!("RealityTrace's ledger cannot be read: {why}"))
            })?;
        // The claims must equal RealityTrace's own record of the threshold.
        let threshold_ok = rows.iter().any(|r| {
            r.ordinal == obligation.threshold_position
                && r.record_kind == "threshold.declared"
                && r.payload
                    .pointer("/states/0/state_id")
                    .and_then(Value::as_str)
                    == Some(obligation.threshold_id.as_str())
        });
        if !threshold_ok {
            return Err(CepAuthorityError::Evidence(format!(
                "RealityTrace has no threshold {} at {}:{}; the claims do not match its record",
                obligation.threshold_id, obligation.threshold_ledger, obligation.threshold_position
            )));
        }
        let execution_floor = execution_floor(&entries, operation_id).ok_or_else(|| {
            CepAuthorityError::Evidence(format!(
                "no execution evidence was submitted for {operation_id}"
            ))
        })?;

        let mut basis = json!({
            "expectation_digest": obligation.expectation_digest,
            "threshold": {"id": obligation.threshold_id, "ledger_id": obligation.threshold_ledger, "position": obligation.threshold_position},
            "window_closes_at": obligation.closes_at.to_rfc3339(),
            "close_grace_s": obligation.close_grace_s,
            "execution_floor": execution_floor.to_rfc3339(),
        });
        let (epistemic, status, admissible, reason) = match closure {
            Some(c)
                if parse_time(&c.committed_at)
                    .is_some_and(|t| t <= obligation.latest_closure()) =>
            {
                let b = c
                    .payload
                    .get("b")
                    .and_then(Value::as_u64)
                    .unwrap_or(u64::MAX);
                let c_pos = c.payload.get("c").and_then(Value::as_u64).unwrap_or(0);
                let discrimination_id = c
                    .payload
                    .get("discrimination_observation_id")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let before = observation(&rows, discrimination_id).ok_or_else(|| {
                    CepAuthorityError::Evidence("the discrimination run has no known result".into())
                })?;
                check_discrimination(before, &obligation, Some(execution_floor))?;
                let frame = Frame {
                    obligation: &obligation,
                    b,
                    c: c_pos,
                    execution_floor,
                };
                let ids: Vec<&str> = c
                    .payload
                    .get("verification_observation_ids")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default();
                let results: Vec<RunResult> = ids
                    .iter()
                    .filter_map(|id| classify(observation(&rows, id), &frame))
                    .collect();
                let (e, s) = judge(&results);
                basis["closure"] = json!({"b": b, "c": c_pos, "committed_at": c.committed_at, "entry": c.entry_ulid});
                basis["results"] =
                    json!(results.iter().map(|r| format!("{r:?}")).collect::<Vec<_>>());
                (e, s, results.len(), Value::Null)
            }
            Some(c) => {
                basis["closure"] = json!({"committed_at": c.committed_at, "entry": c.entry_ulid});
                ("unresolved", "rejected", 0, json!("the closure was recorded after deadline + grace: C cannot be proven on time"))
            }
            None => (
                "unresolved",
                "rejected",
                0,
                json!("no closure was recorded by deadline + grace: C cannot be proven on time"),
            ),
        };

        let at = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let envelope = json!({
            "envelope_id": format!("env-settlement_packet-{operation_id}-obligation"),
            "cep_version": "1.0.0",
            "envelope_version": "1.0",
            "envelope_kind": "settlement_packet",
            "created_at": at,
            "created_by": "sea-forge-authority",
            "scope": {"evaluation_context": "godspeed-settlement_packet", "world_ref": allowed.world_ref},
            "boundary_record": {
                "scope": "obligation settlement of one Cognate operation from RealityTrace's ledger",
                "included_sections": ["settlements"],
                "excluded_sections": [],
                "known_omissions": ["runs not named in the closure are not considered"],
                "unknowns": [], "redactions": [], "compression_notes": [], "out_of_scope_entities": [],
                "classification": "internal", "limitations": [],
            },
            "completeness_status": "partial",
            "omission_status": "marked",
            "provenance_refs": [format!("prov-settle-{operation_id}-obligation")],
            "validation_status": "valid",
            "lineage_refs": [allowed.decision_envelope_id],
            "provenance": [{
                "provenance_id": format!("prov-settle-{operation_id}-obligation"),
                "source_system_refs": ["sea-forge", "realitytrace"],
                "producer_refs": ["sea-forge-authority"],
                "production_method": "obligation_settlement",
                "created_at": at,
            }],
            "settlements": [{
                "settlement_id": format!("set-{operation_id}-obligation"),
                "settlement_status": status,
                "transformation_ref": format!("tr-{operation_id}"),
                "epistemic_status": epistemic,
                "admissible_runs": admissible,
                "reason": reason,
                "threshold_basis": basis,
            }],
            "extensions": {
                "cep.profile": {"profile_id": "godspeed.settlement_packet", "profile_version": PROFILE_VERSION},
                "godspeed.settlement_packet": {
                    "operation_id": operation_id,
                    "authority_decision_ref": allowed.decision_envelope_id,
                    "modality": "obligation",
                    "epistemic_status": epistemic,
                    "overall_status": status,
                    "final": true,
                    "declared_by": caller.actor_id,
                },
            },
        });
        let subjects = vec!["cognate".to_string(), operation_id.to_string()];
        let committed = match ledger.commit_typed_new(
            "settlement_packet",
            final_key.clone(),
            subjects,
            &envelope,
            vec![],
        ) {
            Ok(c) => c,
            Err(_) => {
                // A concurrent settlement won: report it, never a second one.
                let entries = ledger.read_entries()?;
                let done = entries
                    .iter()
                    .find(|e| e.idempotency_key.as_deref() == Some(final_key.as_str()))
                    .ok_or_else(|| {
                        CepAuthorityError::Internal("settlement was not committed".into())
                    })?;
                return Ok(SettlementOutcome {
                    status: settlement_status_of(&done.payload),
                    final_status: true,
                    envelope: done.payload.clone(),
                    committed: committed_ref(done),
                    replayed: true,
                });
            }
        };
        Ok(SettlementOutcome {
            status,
            final_status: true,
            envelope,
            committed,
            replayed: false,
        })
    }
}

fn closure_ack(
    entry: &sea_forge_ledger::LedgerEntry,
    obligation: &Obligation,
    replayed: bool,
) -> ClosureAck {
    ClosureAck {
        b: entry.payload.get("b").and_then(Value::as_u64).unwrap_or(0),
        c: entry.payload.get("c").and_then(Value::as_u64).unwrap_or(0),
        committed_at: entry.committed_at.clone(),
        on_time: parse_time(&entry.committed_at).is_some_and(|t| t <= obligation.latest_closure()),
        replayed,
        entry_ulid: entry.entry_ulid.clone(),
    }
}
