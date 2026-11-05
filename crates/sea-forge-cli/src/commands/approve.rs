//! Thin CLI surface over the shared approval-resolution library.
//!
//! The resolution logic (ledger replay, SoD, criteria-binding cross-checks,
//! authority evaluation and the approvals journal write) lives in
//! `sea_forge_case_runner::case_ops::resolve_approval` so the SFWP server
//! resolves approvals through the same governed path in-process; these
//! wrappers render the outcome exactly as the CLI always has.

use sea_forge_authority::cep::{
    resolve_uncertain_settlement, settlement_approval_id, settlement_pending_records,
    CepAuthorityService, SettlementCriteria, UncertainResolution,
};
use sea_forge_authority::{AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_domainforge::WorldRegistry;
use std::path::Path;

/// The approval id prefix a settlement pending record is approved under.
const SETTLEMENT_APPROVAL_PREFIX: &str = "apr-settlement-pending-";

pub struct ApproveOptions<'a> {
    pub root: &'a Path,
    pub case_id: &'a str,
    pub approval_id: &'a str,
    pub actor: &'a str,
    pub note: Option<&'a str>,
    #[allow(dead_code)]
    pub policy: Option<&'a Path>,
}

pub fn approve(opts: ApproveOptions) -> Result<u8, ForgeError> {
    resolve(&opts, true)
}

pub fn reject(opts: ApproveOptions) -> Result<u8, ForgeError> {
    resolve(&opts, false)
}

fn resolve(opts: &ApproveOptions, approved: bool) -> Result<u8, ForgeError> {
    let policy = opts
        .policy
        .ok_or_else(|| ForgeError::Input("approval policy is missing".into()))?;
    if opts.approval_id.starts_with(SETTLEMENT_APPROVAL_PREFIX) {
        return resolve_settlement(opts, policy, approved);
    }
    let resolved = sea_forge_case_runner::case_ops::resolve_approval(
        opts.root,
        policy,
        opts.case_id,
        opts.approval_id,
        opts.actor,
        opts.note,
        approved,
    )?;
    println!("approval_id={}", opts.approval_id);
    println!("status={:?}", resolved.status);
    println!("resolved_by={}", opts.actor);
    Ok(0)
}

/// The operator's resolution of an uncertain settlement claim, as given on the command line.
pub enum UncertainResolutionArg {
    Declared(String),
    NotDeclared(String),
}

pub struct UncertainOptions<'a> {
    pub root: &'a Path,
    pub case_id: &'a str,
    pub approval_id: &'a str,
    pub attempt: u32,
    pub actor: &'a str,
    pub resolution: UncertainResolutionArg,
    pub policy: &'a Path,
}

/// Resolve an uncertain settlement claim (lease lapsed, no ledger declaration) through the same authority the
/// approval path uses. The approval must name a pending record of this case, and the resolver is refused by
/// policy unless it may resolve approvals (`approval_resolution`). Each resolution is audited before the attempt closes.
pub fn resolve_uncertain(opts: UncertainOptions) -> Result<u8, ForgeError> {
    let Some(pending_id) = opts.approval_id.strip_prefix("apr-") else {
        return Err(ForgeError::Input(format!(
            "approval {} is not a settlement pending approval",
            opts.approval_id
        )));
    };
    let pending = settlement_pending_records(opts.root)
        .map_err(|error| ForgeError::Input(format!("settlement resolution refused: {error}")))?;
    let record = pending
        .iter()
        .find(|record| record.pending_id == pending_id)
        .ok_or_else(|| {
            ForgeError::Input(format!(
                "approval {} is not a settlement pending approval",
                opts.approval_id
            ))
        })?;
    if record.case_id != opts.case_id {
        return Err(ForgeError::Input(format!(
            "approval {} does not belong to case {}",
            opts.approval_id, opts.case_id
        )));
    }
    // A missing or invalid policy refuses the resolution; the policy's `approval_resolution` decision is then
    // evaluated inside `resolve_uncertain_settlement`, so no caller can complete or release a claim without it.
    let bundle = AuthorityPolicyBundle::load(opts.policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let resolver = Actor {
        actor_id: opts.actor.into(),
        role: ActorRole::Operator,
    };
    let resolution = match opts.resolution {
        UncertainResolutionArg::Declared(declaration_ref) => {
            UncertainResolution::Declared { declaration_ref }
        }
        UncertainResolutionArg::NotDeclared(evidence) => {
            UncertainResolution::NotDeclared { evidence }
        }
    };
    let status = resolve_uncertain_settlement(
        &engine,
        &bundle,
        opts.root,
        pending_id,
        opts.attempt,
        resolution,
        &resolver,
    )
    .map_err(|error| ForgeError::Input(format!("settlement resolution refused: {error}")))?;
    println!("approval_id={}", opts.approval_id);
    println!("attempt={}", opts.attempt);
    println!("status={status:?}");
    println!("resolved_by={}", opts.actor);
    Ok(0)
}

/// Resolve a settlement pending approval through the CEP path the server and the m8 tests use: the requester can
/// never resolve it, and the policy's `approval_resolution` rule decides whether the resolver may. The approval
/// must name a pending record of this case, so a record of another case cannot be resolved here.
fn resolve_settlement(
    opts: &ApproveOptions,
    policy: &Path,
    approved: bool,
) -> Result<u8, ForgeError> {
    let pending = settlement_pending_records(opts.root)
        .map_err(|error| ForgeError::Input(format!("settlement approval refused: {error}")))?;
    let record = pending
        .iter()
        .find(|record| settlement_approval_id(&record.pending_id) == opts.approval_id)
        .ok_or_else(|| {
            ForgeError::Input(format!(
                "approval {} is not a settlement pending approval",
                opts.approval_id
            ))
        })?;
    if record.case_id != opts.case_id {
        return Err(ForgeError::Input(format!(
            "approval {} does not belong to case {}",
            opts.approval_id, opts.case_id
        )));
    }
    let bundle = AuthorityPolicyBundle::load(policy)?;
    let engine = PolicyAuthorityEngine::new(bundle.clone())?;
    let worlds = WorldRegistry::new();
    let service = CepAuthorityService {
        engine: &engine,
        bundle: &bundle,
        worlds: &worlds,
        root: opts.root,
        approval_ttl: chrono::Duration::hours(24),
        criteria: SettlementCriteria::default(),
    };
    let resolver = Actor {
        actor_id: opts.actor.into(),
        role: ActorRole::Operator,
    };
    service
        .resolve_approval(opts.approval_id, approved, &resolver)
        .map_err(|error| ForgeError::Input(format!("settlement approval refused: {error}")))?;
    println!("approval_id={}", opts.approval_id);
    println!("status={}", if approved { "approved" } else { "rejected" });
    println!("resolved_by={}", opts.actor);
    Ok(0)
}
