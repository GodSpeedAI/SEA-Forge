//! The one verification of settlement declarations against the authority ledger.
//!
//! The authoritative source of a settlement declaration is the hash-chained case ledger it was committed to. The
//! `settlement/declarations.jsonl` file is a derived view: nothing reads it to decide authority, promotion, or
//! correlation. A declaration counts as settled only when all of these hold:
//!
//! 1. its case ledger verifies (entry chain and stored state), and its own `declaration_hash` recomputes from its
//!    content ([`read_case_declarations`], [`read_all_declarations`]); an integrity failure is an error, never a
//!    silent skip;
//! 2. the authority ledger holds the settlement pending record it names, created by an external gate for the same
//!    case and operation;
//! 3. the declared-content hash it carries reproduces that record's approved request hash from the record's ingress
//!    hash (the approval is bound to this content and no other);
//! 4. the record is claimed, or completed by this declaration, and no other declaration names it.
//!
//! Every reconciliation, read, resume, promotion, and Thoth path calls [`verify_case_declarations`] on declarations
//! read through this module; no caller re-derives the rule.
//!
//! Trust limit: the hash chain makes a change to a stored declaration detectable by anyone who reads the ledger
//! without write access. It does not protect against a writer that can rewrite the ledger and its stored state
//! together. That protection needs an anchor held outside the writer's trust domain; see the runbook.

use super::*;
use sea_forge_core::types::SettlementDeclaration;
use std::collections::{HashMap, HashSet};

/// Why an external settlement declaration is not authorised. The codes are stable: they are written into
/// `settlement_quarantined` records and asserted by tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationRefusal {
    /// External, with no authorisation reference. Legacy declarations and bare out-of-process appends land here.
    MissingAuthorization,
    /// The authorisation reference or request hash is malformed, or does not derive from the other.
    MalformedAuthorization,
    /// External, with an authorisation reference but no declared-content hash, so it cannot be bound to the content
    /// that was approved.
    AuthorizationUnbound,
    /// The declared-content hash does not reproduce the pending record's approved request hash.
    AuthorizationContentMismatch,
    /// No settlement pending record with this reference exists in the authority ledger.
    AuthorizationMissing,
    /// The named pending record predates the case binding in the request hash. It is quarantined, never reused.
    AuthorizationUnscoped,
    /// The pending record's request hash or operation differs from the declaration.
    AuthorizationMismatch,
    /// The pending record was not created by an external gate, so it cannot authorise an external declaration.
    AuthorizationNotExternal,
    /// The approval has not been resolved.
    AuthorizationPending,
    AuthorizationRejected,
    AuthorizationExpired,
    AuthorizationFailed,
    /// The pending record was completed by a different declaration.
    AuthorizationSuperseded,
    /// Several declarations in the case name the same pending record before it is completed.
    AuthorizationReused,
    /// The approval exists but no resume attempt has claimed it. Only a claimed attempt may append a declaration.
    AuthorizationUnclaimed,
    /// A declaration presents as local, but its run has an external settlement pending record.
    ClassMismatch,
}

impl DeclarationRefusal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingAuthorization => "missing_authorization",
            Self::MalformedAuthorization => "malformed_authorization",
            Self::AuthorizationUnbound => "authorization_unbound",
            Self::AuthorizationContentMismatch => "authorization_content_mismatch",
            Self::AuthorizationMissing => "authorization_missing",
            Self::AuthorizationUnscoped => "authorization_unscoped",
            Self::AuthorizationMismatch => "authorization_mismatch",
            Self::AuthorizationNotExternal => "authorization_not_external",
            Self::AuthorizationPending => "authorization_pending",
            Self::AuthorizationRejected => "authorization_rejected",
            Self::AuthorizationExpired => "authorization_expired",
            Self::AuthorizationFailed => "authorization_failed",
            Self::AuthorizationSuperseded => "authorization_superseded",
            Self::AuthorizationReused => "authorization_reused",
            Self::AuthorizationUnclaimed => "authorization_unclaimed",
            Self::ClassMismatch => "class_mismatch",
        }
    }
}

/// The verdict on one settlement declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationAuthorization {
    /// Not an external declaration. Local declarations keep their existing behaviour.
    Local,
    /// External, and verified against the pending record it names.
    Authorized { pending_id: String },
    /// External, and not authorised. It must not be correlated, promoted, or treated as authoritative.
    Quarantined(DeclarationRefusal),
}

/// A declaration is external when an external authority made it. See [`SettlementDeclaration::is_external`].
pub fn is_external_declaration(declaration: &SettlementDeclaration) -> bool {
    declaration.is_external()
}

/// The canonical hash of a declaration's content, excluding its own `declaration_hash`. This is the one definition:
/// the writer and every reader use it.
pub fn declaration_content_hash(
    declaration: &SettlementDeclaration,
) -> Result<String, CepAuthorityError> {
    let mut copy = declaration.clone();
    copy.declaration_hash.clear();
    crate::hash_canonical(&copy).map_err(|error| CepAuthorityError::Internal(error.to_string()))
}

fn integrity(message: impl std::fmt::Display) -> CepAuthorityError {
    CepAuthorityError::Evidence(format!("declaration_integrity_error: {message}"))
}

/// Read and verify one ledger stream's settlement declarations. Fails closed: a broken chain, an unparseable
/// declaration, or a declaration whose stored hash does not match its content is an error for the whole read.
fn read_stream_declarations(
    root: &Path,
    ledger_id: &str,
) -> Result<Vec<SettlementDeclaration>, CepAuthorityError> {
    let stream = LedgerStream::open(root, ledger_id.to_string(), "sea-forge-authority")?;
    stream
        .verify()
        .map_err(|error| integrity(format!("{ledger_id}: {error}")))?;
    let mut declarations = Vec::new();
    for entry in stream.read_entries()? {
        if entry.record_kind != "settlement_declaration" {
            continue;
        }
        let declaration: SettlementDeclaration = serde_json::from_value(entry.payload)
            .map_err(|error| integrity(format!("{ledger_id}: unparseable declaration: {error}")))?;
        if declaration_content_hash(&declaration)? != declaration.declaration_hash {
            return Err(integrity(format!(
                "{ledger_id}: declaration {} does not hash to its declaration_hash",
                declaration.declaration_id
            )));
        }
        declarations.push(declaration);
    }
    Ok(declarations)
}

/// The verified declarations of one case, from its hash-chained case ledger. Never reads the materialised view.
pub fn read_case_declarations(
    root: &Path,
    case_id: &str,
) -> Result<Vec<SettlementDeclaration>, CepAuthorityError> {
    read_stream_declarations(root, &format!("case-{case_id}"))
}

/// The verified declarations of every case in the state root. Declaration ids must be unique across cases: a
/// declaration id seen twice is a replay or a forged copy, and the read fails.
pub fn read_all_declarations(root: &Path) -> Result<Vec<SettlementDeclaration>, CepAuthorityError> {
    let dir = root.join("ledgers");
    let names = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .map(|entry| {
                entry
                    .map_err(|error| integrity(format!("read ledgers dir: {error}")))
                    .and_then(|entry| {
                        entry
                            .file_name()
                            .into_string()
                            .map_err(|_| integrity("ledger name is not UTF-8"))
                    })
            })
            .collect::<Result<Vec<String>, _>>()?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(integrity(format!("read ledgers dir: {error}"))),
    };
    let mut cases: Vec<String> = names
        .into_iter()
        // A stray file named like a case (a lock or temp file) is not a ledger; only case directories are read.
        .filter(|name| name.starts_with("case-") && dir.join(name).is_dir())
        .collect();
    cases.sort();
    let mut seen: HashSet<String> = HashSet::new();
    let mut all = Vec::new();
    for case in cases {
        for declaration in read_stream_declarations(root, &case)? {
            if !seen.insert(declaration.declaration_id.clone()) {
                return Err(integrity(format!(
                    "declaration {} appears more than once",
                    declaration.declaration_id
                )));
            }
            all.push(declaration);
        }
    }
    Ok(all)
}

/// Verify every declaration against the authority ledger, reading the ledger once. The result is aligned with
/// `declarations`. Pass the whole set so that reuse across them is detected. Pass only declarations returned by
/// [`read_case_declarations`] or [`read_all_declarations`].
pub fn verify_case_declarations(
    root: &Path,
    declarations: &[SettlementDeclaration],
) -> Result<Vec<DeclarationAuthorization>, CepAuthorityError> {
    let entries = settlement_ledger(root)?.read_entries()?;
    let now = Utc::now();
    let mut named: HashMap<&str, usize> = HashMap::new();
    for declaration in declarations {
        if let Some(pending_id) = declaration.authorization_ref.as_deref() {
            *named.entry(pending_id).or_insert(0) += 1;
        }
    }
    Ok(declarations
        .iter()
        .map(|declaration| verify_one(&entries, now, declaration, &named))
        .collect())
}

fn verify_one(
    entries: &[LedgerEntry],
    now: DateTime<Utc>,
    declaration: &SettlementDeclaration,
    named: &HashMap<&str, usize>,
) -> DeclarationAuthorization {
    use DeclarationRefusal as R;
    if !declaration.is_external() {
        // A declaration cannot shed its external class by clearing its provenance. If its run has an external
        // settlement pending record, the relabelled declaration is refused.
        let external_gated = entries.iter().any(|entry| {
            entry.record_kind == SETTLEMENT_PENDING_KIND
                && entry.payload.get("case_id").and_then(Value::as_str)
                    == Some(declaration.case_id.as_str())
                && entry.payload.get("operation_id").and_then(Value::as_str)
                    == Some(declaration.run_id.as_str())
                && entry
                    .payload
                    .get("settlement_authority_class")
                    .and_then(Value::as_str)
                    == Some("external")
        });
        return if external_gated {
            DeclarationAuthorization::Quarantined(R::ClassMismatch)
        } else {
            DeclarationAuthorization::Local
        };
    }
    let quarantine = DeclarationAuthorization::Quarantined;
    let (pending_id, request_sha256) = match (
        declaration.authorization_ref.as_deref(),
        declaration.authorization_request_sha256.as_deref(),
    ) {
        (Some(pending_id), Some(sha)) => (pending_id, sha),
        (None, None) => return quarantine(R::MissingAuthorization),
        _ => return quarantine(R::MalformedAuthorization),
    };
    if request_sha256.len() != 64 || pending_id.is_empty() {
        return quarantine(R::MalformedAuthorization);
    }
    let content_sha256 = match declaration.authorization_content_sha256.as_deref() {
        Some(content) if !content.is_empty() => content,
        _ => return quarantine(R::AuthorizationUnbound),
    };
    let record = match pending_record_in(entries, pending_id) {
        Ok(Some(record)) => record,
        Ok(None) => {
            let legacy = entries.iter().any(|entry| {
                entry.record_kind == SETTLEMENT_PENDING_KIND
                    && entry.payload.get("pending_id").and_then(Value::as_str) == Some(pending_id)
            });
            return quarantine(if legacy {
                R::AuthorizationUnscoped
            } else {
                R::AuthorizationMissing
            });
        }
        Err(_) => return quarantine(R::MalformedAuthorization),
    };
    // The record must belong to this declaration's case and operation, not merely share its request hash.
    if record.request_sha256 != request_sha256
        || record.operation_id != declaration.run_id
        || record.case_id != declaration.case_id
    {
        return quarantine(R::AuthorizationMismatch);
    }
    if record.settlement_authority_class != SettlementAuthorityClass::External.as_str()
        || !matches!(
            record.ingress.as_str(),
            "swe_seed_declaration" | "artifact_transition_resume"
        )
    {
        return quarantine(R::AuthorizationNotExternal);
    }
    // The approval covers one content. The declared content must reproduce the approved request hash from the
    // record's ingress hash. A record without an ingress hash is legacy and cannot be bound.
    if record.ingress_sha256.is_empty() {
        return quarantine(R::AuthorizationUnscoped);
    }
    match crate::cep::bind_request_sha256(&record.ingress_sha256, content_sha256) {
        Ok(bound) if bound == record.request_sha256 => {}
        _ => return quarantine(R::AuthorizationContentMismatch),
    }
    match status_in(entries, &record, now) {
        // A completed record belongs to the one declaration it names. Any other declaration is a replay.
        SettlementStatus::Completed { declaration_ref } => {
            if declaration_ref != declaration.declaration_id {
                return quarantine(R::AuthorizationSuperseded);
            }
        }
        // Approved but unclaimed: no legitimate writer appends here, so a declaration in this state is refused.
        SettlementStatus::Approved => return quarantine(R::AuthorizationUnclaimed),
        // Claimed by an attempt with no outcome yet: the authorisation is spent by at most one declaration,
        // so any second declaration naming it is refused.
        SettlementStatus::Resuming { .. } => {
            if named.get(pending_id).copied().unwrap_or(0) > 1 {
                return quarantine(R::AuthorizationReused);
            }
        }
        SettlementStatus::Pending => return quarantine(R::AuthorizationPending),
        SettlementStatus::Rejected => return quarantine(R::AuthorizationRejected),
        SettlementStatus::Expired => return quarantine(R::AuthorizationExpired),
        SettlementStatus::Failed { .. } => return quarantine(R::AuthorizationFailed),
    }
    DeclarationAuthorization::Authorized {
        pending_id: pending_id.to_string(),
    }
}

/// Settle every open claim for one operation (run) that has no recorded outcome. A claim completes only on an
/// authorised declaration of this case that names that claim; otherwise it fails. Never declares.
pub fn reconcile_operation_claims(
    root: &Path,
    case_id: &str,
    operation_id: &str,
) -> Result<(), CepAuthorityError> {
    let declarations = read_case_declarations(root, case_id)?;
    let verdicts = verify_case_declarations(root, &declarations)?;
    let mut authorised: HashMap<String, String> = HashMap::new();
    for (declaration, verdict) in declarations.iter().zip(&verdicts) {
        if let DeclarationAuthorization::Authorized { pending_id } = verdict {
            if declaration.run_id == operation_id {
                authorised.insert(pending_id.clone(), declaration.declaration_id.clone());
            }
        }
    }
    for record in settlement_pending_records(root)?
        .iter()
        .filter(|record| record.case_id == case_id && record.operation_id == operation_id)
    {
        if let SettlementStatus::Resuming { .. } = settlement_status(root, &record.pending_id)? {
            reconcile_settlement_resume(
                root,
                &record.pending_id,
                authorised.get(&record.pending_id).map(String::as_str),
            )?;
        }
    }
    Ok(())
}

/// What settling a failed resume attempt's open claim did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptSettlement {
    /// An authorised declaration names this claim: the attempt completed on that declaration.
    Completed,
    /// No declaration in the case names this claim: it was released and a later resume may claim it again.
    Released,
    /// A declaration names this claim but is not authorised for it. The claim stays open for reconciliation.
    Open,
}

/// Settle the open claim of a resume attempt that returned an error. The claim is released only when no
/// settlement declaration in the case names it, so a declaration appended before the failure is never joined by
/// a second one. The check reads the raw payloads, so a declaration that does not parse still blocks release.
/// Never declares.
pub fn settle_failed_resume_attempt(
    root: &Path,
    case_id: &str,
    pending_id: &str,
    attempt: u32,
) -> Result<AttemptSettlement, CepAuthorityError> {
    let raw = LedgerStream::open(root, format!("case-{case_id}"), "sea-forge-authority")?
        .read_entries()?;
    let named = raw.iter().any(|entry| {
        entry.record_kind == "settlement_declaration"
            && entry
                .payload
                .get("authorization_ref")
                .and_then(Value::as_str)
                == Some(pending_id)
    });
    if !named {
        release_settlement_resume(root, pending_id, attempt)?;
        return Ok(AttemptSettlement::Released);
    }
    let declarations = read_case_declarations(root, case_id)?;
    let verdicts = verify_case_declarations(root, &declarations)?;
    let authorised = declarations
        .iter()
        .zip(&verdicts)
        .find_map(|(declaration, verdict)| match verdict {
            DeclarationAuthorization::Authorized { pending_id: named } if named == pending_id => {
                Some(declaration.declaration_id.clone())
            }
            _ => None,
        });
    match authorised {
        Some(declaration_id) => {
            complete_settlement_resume(root, pending_id, attempt, &declaration_id)?;
            Ok(AttemptSettlement::Completed)
        }
        None => Ok(AttemptSettlement::Open),
    }
}
