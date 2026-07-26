//! SFWP JSON Schema generator (Task 3, Rust half).
//!
//! Emits one `<TypeName>.schema.json` per SFWP-layer contract type into
//! `workbench/packages/contracts/schema/`. This is the Rust-authored
//! intermediate the follow-up TS task's `bun run generate:contracts` consumes.
//! Running `cargo run -p sea-forge-server --bin gen_sfwp_schema` regenerates
//! these deterministically; a drift test fails if a contract type changes
//! without regeneration.

use schemars::schema_for;
use sea_forge_server::sfwp::case::{
    EntryOptionsResult, PlanItemSummary, PreflightParams, PreflightResult, TemplateOption,
    TemplateParameter,
};
use sea_forge_server::sfwp::correlation::RequestRecord;
use sea_forge_server::sfwp::events::EventFrame;
use sea_forge_server::sfwp::precondition::{
    ChangedRecord, Precondition, RecordDigest, RejectedAsStale,
};
use sea_forge_server::sfwp::readiness::{ReadinessGetParams, ReadinessItem, ReadinessView};
use sea_forge_server::sfwp::{
    DescribeResult, GetSchemaResult, HelloResult, MethodDescriptor, UnsupportedVersion,
};
use std::path::PathBuf;

/// Serialize each schema to pretty JSON with a trailing newline, keyed by the
/// exact filename it must be written to. Shared by the binary and the drift
/// test so they never diverge.
pub fn generated_schemas() -> Vec<(String, String)> {
    macro_rules! schema {
        ($ty:ty, $name:literal) => {{
            let schema = schema_for!($ty);
            let mut json =
                serde_json::to_string_pretty(&schema).expect("schema serialization must not fail");
            json.push('\n');
            (format!("{}.schema.json", $name), json)
        }};
    }
    vec![
        schema!(EventFrame, "EventFrame"),
        schema!(Precondition, "Precondition"),
        schema!(RecordDigest, "RecordDigest"),
        schema!(ChangedRecord, "ChangedRecord"),
        schema!(RejectedAsStale, "RejectedAsStale"),
        schema!(RequestRecord, "RequestRecord"),
        schema!(HelloResult, "HelloResult"),
        schema!(DescribeResult, "DescribeResult"),
        schema!(GetSchemaResult, "GetSchemaResult"),
        schema!(UnsupportedVersion, "UnsupportedVersion"),
        schema!(MethodDescriptor, "MethodDescriptor"),
        schema!(ReadinessView, "ReadinessView"),
        schema!(ReadinessItem, "ReadinessItem"),
        schema!(ReadinessGetParams, "ReadinessGetParams"),
        schema!(EntryOptionsResult, "EntryOptionsResult"),
        schema!(TemplateOption, "TemplateOption"),
        schema!(TemplateParameter, "TemplateParameter"),
        schema!(PreflightParams, "PreflightParams"),
        schema!(PreflightResult, "PreflightResult"),
        schema!(PlanItemSummary, "PlanItemSummary"),
    ]
}

/// The contracts schema directory, resolved relative to the workspace root
/// (the crate manifest dir is `crates/sea-forge-server`). Honors
/// `SFWP_SCHEMA_OUT_DIR` so the drift test can regenerate into a temp dir and
/// diff against the committed tree without mutating it.
pub fn schema_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SFWP_SCHEMA_OUT_DIR") {
        return PathBuf::from(dir);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .expect("crate lives two levels under workspace root")
        .join("workbench/packages/contracts/schema")
}

fn main() -> std::io::Result<()> {
    let dir = schema_dir();
    std::fs::create_dir_all(&dir)?;
    for (filename, json) in generated_schemas() {
        std::fs::write(dir.join(&filename), json)?;
    }
    println!(
        "wrote {} SFWP schema files to {}",
        generated_schemas().len(),
        dir.display()
    );
    Ok(())
}
