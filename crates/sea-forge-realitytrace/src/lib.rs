//! `sea-forge.security-trace/v1`: ledger-derived security evidence handed to
//! RealityTrace and to independent auditors.
//!
//! Ownership (ADR-012): SEA Forge owns authority, enforcement, trusted
//! resource observation and settlement, recorded in its append-only
//! hash-chained ledger. RealityTrace owns the structured comparison of a prior
//! expectation with observed consequences. The auditor owns the verdict.
//! Nothing here decides, settles, or fills in missing evidence: absent records
//! are reported as `unresolved`, never inferred.
//!
//! The bridge uses RealityTrace's existing library contract: SEA Forge's
//! `OperationalSettlement` (E6) plus a `ProofCompleted` (E7) carrying the
//! pre-committed frozen expectation are admitted by `sxr_core::proof_ingest`,
//! and `sxr_core::evidence_emit` produces the `EvidenceRecorded` (E8)
//! expected-versus-observed difference. No daemon, no second authority store.
use sea_forge_ledger::LedgerEntry;
use serde_json::{json, Value};
use sxr_core::canonical_json::sha256_canonical;
use sxr_core::evidence_emit::{emit_evidence_recorded, EvidenceEmission};
use sxr_core::proof_ingest::{ingest, Admission, IngestParams, IngestState};

pub const TRACE_SCHEMA: &str = "sea-forge.security-trace/v1";
/// Ledger record kind committed BEFORE the attempt; binds the expectation.
pub const EXPECTATION_KIND: &str = "security_frozen_expectation";
/// Who actually produced the E7 proof. RealityTrace's v1 ingest gate only
/// admits the `swe-seed` producer label, so the true producer travels here.
pub const PROOF_PRODUCER: &str = "sea-forge-security-acceptance";

/// The immutable acceptance reference an expectation is frozen against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenSuite {
    pub tag: String,
    pub commit: String,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceError {
    /// A required provenance field is missing or malformed.
    Provenance(String),
    /// Records do not correlate to one action.
    Correlation(String),
    /// RealityTrace refused the proof or the evidence emission.
    RealityTrace(String),
}

fn hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn hex40(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Payload of the `security_frozen_expectation` record. `expected_outcome`
/// is stated in the observed form the trusted observer produces, so a
/// consistent run compares `equivalent` and any divergence is visible.
pub fn expectation_record(
    suite: &FrozenSuite,
    contract_id: &str,
    case_id: &str,
    work_request_id: &str,
    expected_outcome: &Value,
) -> Result<Value, TraceError> {
    if suite.tag.trim().is_empty() || !hex40(&suite.commit) || !hex64(&suite.manifest_sha256) {
        return Err(TraceError::Provenance(
            "frozen suite reference is incomplete".into(),
        ));
    }
    if [contract_id, case_id, work_request_id]
        .iter()
        .any(|v| v.trim().is_empty())
        || expected_outcome.is_null()
    {
        return Err(TraceError::Provenance(
            "expectation identity is incomplete".into(),
        ));
    }
    Ok(json!({
        "schema": TRACE_SCHEMA,
        "suite": {"tag": suite.tag, "commit": suite.commit, "manifest_sha256": suite.manifest_sha256},
        "contract_id": contract_id,
        "case_id": case_id,
        "work_request_id": work_request_id,
        "expected_outcome": expected_outcome,
        "expected_outcome_sha256": sha256_canonical(expected_outcome),
    }))
}

fn entry_ref(entry: &LedgerEntry) -> Value {
    json!({
        "ledger_id": entry.ledger_id,
        "append_ordinal": entry.append_ordinal,
        "entry_ulid": entry.entry_ulid,
        "record_kind": entry.record_kind,
        "payload_hash": entry.payload_hash,
        "entry_hash": entry.entry_hash,
        "writer_identity_ref": entry.writer_identity_ref,
        "authority_refs": entry.authority_refs,
    })
}

fn find<'a>(entries: &'a [LedgerEntry], kind: &str) -> Option<&'a LedgerEntry> {
    entries.iter().find(|e| e.record_kind == kind)
}

/// Project one action's committed ledger records into the correlation
/// contract. Claims (what the executor said) and observations (what SEA
/// Forge's trusted observer saw) stay separate; missing records are listed.
pub fn project(entries: &[LedgerEntry], work_request_id: &str) -> Value {
    let mut unresolved = Vec::new();
    let mut section = |kind: &str, select: &dyn Fn(&LedgerEntry) -> Value| match find(entries, kind)
    {
        Some(entry) => json!({"ledger": entry_ref(entry), "record": select(entry)}),
        None => {
            unresolved.push(format!("missing:{kind}"));
            Value::Null
        }
    };
    let expectation = section(EXPECTATION_KIND, &|e| e.payload.clone());
    let authority = section("authority_decision", &|e| {
        json!({
            "decision_id": e.payload["decision_id"],
            "verdict": e.payload["verdict"],
            "actor": e.payload["actor"],
            "action_hash": e.payload["action_hash"],
            "policy_hash": e.payload["policy_hash"],
        })
    });
    let invocation = section("security_authorized_invocation", &|e| {
        json!({
            "event_id": e.payload["event_id"],
            "invocation_id": e.payload["payload"]["invocation_id"],
            "authority_decision_id": e.payload["payload"]["authority_decision_id"],
            "world_ref": e.payload["payload"]["world_ref"],
        })
    });
    let claim = section("security_submitted_observation", &|e| {
        json!({
            "claimant": e.payload["source_agent"],
            "event_id": e.payload["event_id"],
            "claimed_effects": e.payload["payload"]["observed_effects"],
        })
    });
    let workload_report = section(
        "security_workload_report",
        &|e| json!({"claimant": "workload", "report": e.payload}),
    );
    let observation = section(
        "security_resource_observation",
        &|e| json!({"observer": e.writer_identity_ref, "observation": e.payload}),
    );
    let settlement = section("security_returned_settlement", &|e| {
        json!({
            "event_id": e.payload["event_id"],
            "status": e.payload["payload"]["operational_settlement_status"],
            "observed_effects": e.payload["payload"]["observed_effects"],
            "envelope_sha256": sha256_canonical(&e.payload),
        })
    });
    if let Some(record) = expectation.get("record") {
        if record["work_request_id"] != work_request_id {
            unresolved.push("correlation:expectation_work_request_id".into());
        }
    }
    json!({
        "schema": TRACE_SCHEMA,
        "work_request_id": work_request_id,
        "expectation": expectation,
        "authority": authority,
        "invocation": invocation,
        "claims": [claim, workload_report],
        "observations": [observation],
        "settlement": settlement,
        "unresolved": unresolved,
    })
}

/// Correlation context the bridge needs; every value comes from the SEA
/// Forge records, not from the executing agent.
pub struct ProofContext<'a> {
    pub model_sha256: &'a str,
    pub world_ref: &'a str,
    pub proof_event_id: &'a str,
    pub proof_result_id: &'a str,
    pub occurred_at: &'a str,
}

/// Build the E7 `ProofCompleted` for one settled action. Refuses when the
/// expectation was not committed before the attempt or lacks a frozen-suite
/// reference, or when the settlement does not belong to the same action.
pub fn proof_completed(
    entries: &[LedgerEntry],
    settlement: &Value,
    ctx: &ProofContext<'_>,
) -> Result<Value, TraceError> {
    let expectation = find(entries, EXPECTATION_KIND)
        .ok_or_else(|| TraceError::Provenance("no committed frozen expectation".into()))?;
    let attempt = find(entries, "security_authorized_invocation")
        .ok_or_else(|| TraceError::Correlation("no authorized invocation".into()))?;
    if expectation.append_ordinal >= attempt.append_ordinal {
        return Err(TraceError::Provenance(
            "expectation was not frozen before the attempt".into(),
        ));
    }
    let record = &expectation.payload;
    let suite = FrozenSuite {
        tag: record["suite"]["tag"].as_str().unwrap_or_default().into(),
        commit: record["suite"]["commit"]
            .as_str()
            .unwrap_or_default()
            .into(),
        manifest_sha256: record["suite"]["manifest_sha256"]
            .as_str()
            .unwrap_or_default()
            .into(),
    };
    let expected = &record["expected_outcome"];
    let rebuilt = expectation_record(
        &suite,
        record["contract_id"].as_str().unwrap_or_default(),
        record["case_id"].as_str().unwrap_or_default(),
        record["work_request_id"].as_str().unwrap_or_default(),
        expected,
    )?;
    if &rebuilt != record {
        return Err(TraceError::Provenance(
            "expectation record is not canonical".into(),
        ));
    }
    let work_request_id = record["work_request_id"].as_str().unwrap_or_default();
    if settlement["payload"]["work_request_id"] != work_request_id {
        return Err(TraceError::Correlation(
            "settlement belongs to another work request".into(),
        ));
    }
    let settled = find(entries, "security_returned_settlement").ok_or_else(|| {
        TraceError::Correlation("settlement is not in the SEA Forge ledger".into())
    })?;
    if &settled.payload != settlement {
        return Err(TraceError::Correlation(
            "settlement differs from the ledger record".into(),
        ));
    }
    let settlement_event_id = settlement["event_id"].as_str().unwrap_or_default();
    Ok(json!({
        "schema_version": "v1",
        "event_id": ctx.proof_event_id,
        "source_agent": "swe-seed",
        "event_type": "ProofCompleted",
        "occurred_at": ctx.occurred_at,
        "idempotency_key": sha256_canonical(&json!([ctx.proof_event_id, work_request_id])),
        "payload": {
            "domain_model_hash": ctx.model_sha256,
            "world_ref": ctx.world_ref,
            "namespace": "agentic_capability_loop",
            "domain_model_ref": {"namespace": "agentic_capability_loop", "model_hash": ctx.model_sha256},
            "work_request_id": work_request_id,
            "proof_result_id": ctx.proof_result_id,
            "proof_status": if settlement["payload"]["operational_settlement_status"] == "rejected" { "failed" } else { "passed" },
            "proof_contract": {
                "producer": PROOF_PRODUCER,
                "proof_type": "frozen_security_acceptance",
                "criterion": format!("{}:{}", record["contract_id"].as_str().unwrap_or_default(), record["case_id"].as_str().unwrap_or_default()),
                "suite": record["suite"],
                "contract_id": record["contract_id"],
                "case_id": record["case_id"],
                "expectation_ledger_ref": entry_ref(expectation),
            },
            "expected_outcome": expected,
            "operational_settlement_ref": sha256_canonical(settlement),
        },
        "provenance": {"origin": PROOF_PRODUCER, "chain": [format!("caused_by:{settlement_event_id}")]},
    }))
}

/// Admit E7 against E6 through RealityTrace and emit its E8 comparison.
pub fn record_evidence(
    proof: &Value,
    settlement: &Value,
    state: &mut IngestState,
    ctx: &ProofContext<'_>,
    evidence_event_id: &str,
    affordance_id: &str,
) -> Result<Value, TraceError> {
    let work_request_id = settlement["payload"]["work_request_id"]
        .as_str()
        .unwrap_or_default();
    let settlement_event_id = settlement["event_id"].as_str().unwrap_or_default();
    let params = IngestParams {
        local_model_sha256: ctx.model_sha256,
        local_world_ref: ctx.world_ref,
        expected_work_request_id: work_request_id,
        expected_settlement_event_id: settlement_event_id,
    };
    let comparison = match ingest(proof, Some(settlement), &params, state) {
        Ok(Admission::First(comparison)) => comparison,
        Ok(Admission::DuplicateDelivery) => {
            return Err(TraceError::RealityTrace("duplicate delivery".into()))
        }
        Err(refusal) => return Err(TraceError::RealityTrace(format!("{refusal:?}"))),
    };
    emit_evidence_recorded(
        &comparison,
        &EvidenceEmission {
            local_model_sha256: ctx.model_sha256,
            local_world_ref: ctx.world_ref,
            work_request_id,
            affordance_id,
            proof_event_id: ctx.proof_event_id,
            event_id: evidence_event_id,
            occurred_at: ctx.occurred_at,
            context_bundle_ref: None,
        },
    )
    .map_err(|refusal| TraceError::RealityTrace(format!("{refusal:?}")))
}

/// Evidence category of a SEA Forge security record. The categories never
/// merge: a workload's request is a claim, the authority verdict a decision,
/// what the broker did an enforcement act, what a trusted observer saw an
/// observation, and the broker's closing record a settlement.
pub fn record_category(kind: &str) -> Option<&'static str> {
    Some(match kind {
        EXPECTATION_KIND => "expectation",
        "authority_decision" | "mcp_discovery_decision" => "decision",
        "mcp_attempt" => "claim",
        "mcp_forward_attempt"
        | "mcp_peer_refused"
        | "mcp_protocol_refusal"
        | "mcp_session_revoked"
        | "database_session_issued"
        | "database_elevation_issued"
        | "database_session_renewed"
        | "database_session_refused"
        | "database_session_revoked"
        | "exposure_context_derived"
        | "exposure_context_revoked" => "enforcement",
        "exposure_observed" => "observation",
        "mcp_settlement" => "settlement",
        _ => return None,
    })
}

/// A trusted observer's own artifact (an upstream's invocation log, a
/// database catalog digest), attached to a trace by content digest. The
/// observer, not the workload or the broker, produced it.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeObservation {
    pub observer: String,
    pub source: String,
    pub bytes: Vec<u8>,
}

/// Project every brokered attempt (successful and failed) in `entries` into
/// the security-trace contract for an auditor. Each record keeps its
/// category, ledger reference and hash, and its causal parents; each
/// attempt (`mcp_attempt`, session issuance or refusal, exposure
/// observation, peer or protocol refusal) lists the records that follow from
/// it. An MCP attempt with no settlement is reported `unresolved`, never
/// inferred. Native observations travel with their SHA-256 so the auditor
/// can check them against the observer's original bytes.
pub fn project_attempts(entries: &[LedgerEntry], native: &[NativeObservation]) -> Value {
    use sha2::{Digest, Sha256};
    let by_ulid: std::collections::BTreeMap<&str, &LedgerEntry> =
        entries.iter().map(|e| (e.entry_ulid.as_str(), e)).collect();
    let mut unresolved = Vec::new();
    let records: Vec<Value> = entries
        .iter()
        .map(|entry| {
            let category = record_category(&entry.record_kind);
            if category.is_none() {
                unresolved.push(format!("uncategorized:{}", entry.record_kind));
            }
            json!({
                "category": category,
                "ledger": entry_ref(entry),
                "parents": entry.authority_refs,
                "record": entry.payload,
            })
        })
        .collect();
    let roots = [
        "mcp_attempt",
        "mcp_peer_refused",
        "mcp_protocol_refusal",
        "database_session_issued",
        "database_session_refused",
        "database_elevation_issued",
        "exposure_observed",
    ];
    let attempts: Vec<Value> = entries
        .iter()
        .filter(|e| roots.contains(&e.record_kind.as_str()))
        .map(|root| {
            // Everything caused (transitively) by the root, in ledger order.
            let mut chain = vec![root.entry_ulid.as_str()];
            for entry in entries {
                if entry
                    .authority_refs
                    .iter()
                    .any(|parent| chain.contains(&parent.as_str()))
                    && !chain.contains(&entry.entry_ulid.as_str())
                {
                    chain.push(entry.entry_ulid.as_str());
                }
            }
            let decision = root
                .authority_refs
                .iter()
                .filter_map(|r| by_ulid.get(r.as_str()))
                .find(|e| record_category(&e.record_kind) == Some("decision"))
                .map(|e| json!({"entry_ulid": e.entry_ulid, "verdict": e.payload["verdict"]}));
            let settlement = chain
                .iter()
                .filter_map(|u| by_ulid.get(u))
                .find(|e| e.record_kind == "mcp_settlement")
                .map(|e| {
                    json!({"entry_ulid": e.entry_ulid, "status": e.payload["status"],
                    "forwarded": e.payload["forwarded"]})
                });
            if root.record_kind == "mcp_attempt" && settlement.is_none() {
                unresolved.push(format!("unsettled:{}", root.entry_ulid));
            }
            json!({
                "root": {"kind": root.record_kind, "entry_ulid": root.entry_ulid},
                "decision": decision,
                "chain": chain,
                "settlement": settlement,
            })
        })
        .collect();
    let native: Vec<Value> = native
        .iter()
        .map(|o| {
            json!({
                "category": "native_observation",
                "observer": o.observer,
                "source": o.source,
                "sha256": format!("sha256:{}", hex::encode(Sha256::digest(&o.bytes))),
                "bytes": o.bytes.len(),
            })
        })
        .collect();
    json!({
        "schema": TRACE_SCHEMA,
        "records": records,
        "attempts": attempts,
        "native_observations": native,
        "unresolved": unresolved,
    })
}
