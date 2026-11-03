//! The one verification of settlement declarations against the authority ledger.
//!
//! A `settlement_declaration` in a case ledger is not authoritative by being there. An external declaration is
//! authoritative only when the authority ledger holds the settlement pending record it names, the record was
//! created by an external gate for the same request (`request_sha256`, operation), and that record is approved,
//! claimed, or completed by this declaration. Every reconciliation, read, and resume path that treats a
//! declaration as settled calls [`verify_case_declarations`]; no caller re-derives the rule.

use super::*;
use sea_forge_core::types::SettlementDeclaration;
use std::collections::HashMap;

/// Why an external settlement declaration is not authorised. The codes are stable: they are written into
/// `settlement_quarantined` records and asserted by tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationRefusal {
    /// External, with no authorisation reference. Legacy declarations and bare out-of-process appends land here.
    MissingAuthorization,
    /// The authorisation reference or request hash is malformed, or does not derive from the other.
    MalformedAuthorization,
    /// No settlement pending record with this reference exists in the authority ledger.
    AuthorizationMissing,
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
}

impl DeclarationRefusal {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingAuthorization => "missing_authorization",
            Self::MalformedAuthorization => "malformed_authorization",
            Self::AuthorizationMissing => "authorization_missing",
            Self::AuthorizationMismatch => "authorization_mismatch",
            Self::AuthorizationNotExternal => "authorization_not_external",
            Self::AuthorizationPending => "authorization_pending",
            Self::AuthorizationRejected => "authorization_rejected",
            Self::AuthorizationExpired => "authorization_expired",
            Self::AuthorizationFailed => "authorization_failed",
            Self::AuthorizationSuperseded => "authorization_superseded",
            Self::AuthorizationReused => "authorization_reused",
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

/// Verify every declaration in one case against the authority ledger, reading the ledger once. The result is
/// aligned with `declarations`. Pass the whole case's declarations so that reuse across them is detected.
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
        return DeclarationAuthorization::Local;
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
    if request_sha256.len() != 64 || pending_id != pending_id_for(request_sha256) {
        return quarantine(R::MalformedAuthorization);
    }
    let record = match pending_record_in(entries, pending_id) {
        Ok(Some(record)) => record,
        Ok(None) => return quarantine(R::AuthorizationMissing),
        Err(_) => return quarantine(R::MalformedAuthorization),
    };
    if record.request_sha256 != request_sha256 || record.operation_id != declaration.run_id {
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
    match status_in(entries, &record, now) {
        // A completed record belongs to the one declaration it names. Any other declaration is a replay.
        SettlementStatus::Completed { declaration_ref } => {
            if declaration_ref != declaration.declaration_id {
                return quarantine(R::AuthorizationSuperseded);
            }
        }
        // Approved, or claimed by an attempt with no outcome yet: the authorisation is spent by at most one
        // declaration, so any second declaration naming it is refused.
        SettlementStatus::Approved | SettlementStatus::Resuming { .. } => {
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
