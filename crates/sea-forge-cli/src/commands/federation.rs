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

pub fn export(opts: ExportOptions) -> Result<u8, ForgeError> {
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
