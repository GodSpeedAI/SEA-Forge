//! Federation commands: bundle export/import + template adopt (spec-full §7.4).
//! Thin wrappers around `sea_forge_cell` with authority approval.

use sea_forge_core::{errors::ForgeError, types::AuthorityAction};
use sea_forge_ledger::LedgerStream;
use std::path::Path;

pub struct ExportOptions<'a> {
    pub root: &'a Path,
    pub policy: Option<&'a Path>,
    pub actor: &'a str,
    pub run_ids: &'a [String],
    pub templates: &'a [String],
    pub out: &'a Path,
}

/// SEC-09: the union of exposure labels of every exported run's case
/// context. A bundle carries `artifacts/`, which an isolated workload writes
/// after receiving classified plaintext, so the export states that exposure
/// and falls to default deny unless a rule permits every label. A run whose
/// case cannot be established from its control-plane plan refuses the
/// whole export.
fn exported_exposure(root: &Path, run_ids: &[String]) -> Result<Vec<String>, ForgeError> {
    let registry = sea_forge_ledger::exposure::ExposureRegistry::open(root, "sea-forge-export")?;
    let mut labels = std::collections::BTreeSet::new();
    for run_id in run_ids {
        if !sea_forge_core::path::valid_id_segment(run_id, 128) {
            return Err(ForgeError::UnsafePath(format!("invalid run id {run_id:?}")));
        }
        let plan_path = root.join("runs").join(run_id).join("plan.json");
        let plan: sea_forge_core::types::CasePlan = serde_json::from_slice(
            &std::fs::read(&plan_path).map_err(|e| ForgeError::io("read exported run plan", e))?,
        )?;
        labels.extend(
            registry
                .labels(&format!("case:{}", plan.case_id))
                .map_err(|e| ForgeError::Internal(format!("exposure state unavailable: {e}")))?,
        );
    }
    Ok(labels.into_iter().collect())
}

pub fn export(opts: ExportOptions) -> Result<u8, ForgeError> {
    let exposure = exported_exposure(opts.root, opts.run_ids)?;
    let policy = super::mediated::policy_path(opts.root, opts.policy);
    super::mediated::authorize_read(
        opts.root,
        &policy,
        opts.actor,
        &AuthorityAction::Reserved {
            resource_type: "export_bundle".into(),
            resource_id: opts
                .out
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            parameters: serde_json::json!({
                "run_ids": opts.run_ids,
                "templates": opts.templates,
                "exposure": exposure,
            }),
        },
    )?;
    let manifest = sea_forge_cell::export(opts.root, opts.run_ids, opts.templates, opts.out)?;
    // Record a ledger entry for the export side effect.
    let stream = LedgerStream::open(opts.root, "federation".to_string(), opts.actor)?;
    stream.commit_typed(
        "bundle_export",
        vec![manifest.bundle_id.clone()],
        &manifest,
        vec![],
    )?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(0)
}

pub struct ImportOptions<'a> {
    pub root: &'a Path,
    pub policy: Option<&'a Path>,
    pub actor: &'a str,
    pub bundle: &'a Path,
}

pub fn import(opts: ImportOptions) -> Result<u8, ForgeError> {
    let policy = super::mediated::policy_path(opts.root, opts.policy);
    super::mediated::authorize_read(
        opts.root,
        &policy,
        opts.actor,
        &AuthorityAction::Reserved {
            resource_type: "import_bundle".into(),
            resource_id: opts
                .bundle
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            parameters: serde_json::json!({}),
        },
    )?;
    let manifest = sea_forge_cell::import(opts.root, opts.bundle)?;
    let stream = LedgerStream::open(opts.root, "federation".to_string(), opts.actor)?;
    stream.commit_typed(
        "bundle_import",
        vec![manifest.bundle_id.clone()],
        &manifest,
        vec![],
    )?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(0)
}

pub struct AdoptOptions<'a> {
    pub root: &'a Path,
    pub policy: Option<&'a Path>,
    pub actor: &'a str,
    pub cell_id: &'a str,
    pub reference: &'a str,
}

pub fn adopt(opts: AdoptOptions) -> Result<u8, ForgeError> {
    let policy = super::mediated::policy_path(opts.root, opts.policy);
    super::mediated::authorize_read(
        opts.root,
        &policy,
        opts.actor,
        &AuthorityAction::Reserved {
            resource_type: "adopt_template".into(),
            resource_id: opts.reference.into(),
            parameters: serde_json::json!({"cell_id": opts.cell_id}),
        },
    )?;
    let dest = sea_forge_cell::adopt(opts.root, opts.cell_id, opts.reference)?;
    println!("adopted={} path={}", opts.reference, dest.display());
    Ok(0)
}
