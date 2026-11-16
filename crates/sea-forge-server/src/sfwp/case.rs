//! `case.entry_options` / `case.preflight` — SFWP **inspect** methods for case
//! authoring (Task 6, ADR-003 additive).
//!
//! # Contract
//!
//! `case.entry_options` projects the templates already materialized under
//! `<root>/templates/*.yaml` — no new truth, no fabricated catalog: an empty
//! list is an honest answer when no template has been materialized yet
//! (`unknown ≠ unavailable`).
//!
//! `case.preflight` instantiates a template with caller-supplied parameters
//! (`sea_forge_planner::templates::instantiate`, pure/no I/O beyond the
//! template load itself) and runs the **same** `validate_proposal` function
//! `case_dispatch::submit` uses before ever creating a case — an authoritative
//! dry run, not a forked/duplicated validator. It is audit-only: no case,
//! run, or ledger is created. The returned `precondition` is a `RecordDigest`
//! the client should carry into `case.commit`'s `preconditions.records`; if
//! the named template's on-disk bytes have changed by the time `case.commit`
//! runs, the digest no longer matches and the commit is rejected as stale
//! (repair path: re-run `case.preflight` and retry) — reusing the existing
//! `sfwp::precondition` mechanism rather than inventing a second one.

use std::collections::BTreeMap;
use std::path::Path;

use schemars::JsonSchema;
use sea_forge_core::types::CasePlan;
use sea_forge_planner::case_engine::validate_proposal;
use sea_forge_planner::templates::{self, PlanTemplate};
use serde::{Deserialize, Serialize};

use crate::sfwp::precondition::{digest_of, RecordDigest};

/// A template parameter definition, projected from
/// `sea_forge_planner::templates::ParameterDef` (which has no `JsonSchema`
/// derive of its own — this SFWP-local shape is the frontend-facing contract,
/// mirroring the `readiness.rs` pattern of never handing raw core/planner
/// types straight to `schemars`).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TemplateParameter {
    pub param_type: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// One template an operator may instantiate a case from — projected from the
/// real on-disk `PlanTemplate`, never fabricated.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TemplateOption {
    /// `name@version`, the same form `case.preflight`/`case.commit` accept.
    pub template_ref: String,
    pub description: String,
    /// Parameter name -> definition, copied from the template.
    pub parameters: BTreeMap<String, TemplateParameter>,
}

/// `case.entry_options` result body.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct EntryOptionsResult {
    /// Empty when no template has been materialized under `templates/` yet —
    /// an honest empty list, never a fabricated built-in catalog.
    pub templates: Vec<TemplateOption>,
}

/// List every `<root>/templates/<name>@<version>.yaml` file as an entry
/// option. Malformed template files are skipped (not surfaced as a hard
/// error): entry-options is a best-effort inspect projection, and a single
/// bad file must not blank the whole list.
pub fn entry_options(root: &Path) -> EntryOptionsResult {
    let templates_dir = root.join("templates");
    let Ok(read_dir) = std::fs::read_dir(&templates_dir) else {
        return EntryOptionsResult::default();
    };
    let mut templates = Vec::new();
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let Ok(template) = templates::load(&path) else {
            continue;
        };
        templates.push(TemplateOption {
            template_ref: format!("{}@{}", template.name, template.version),
            description: template.description,
            parameters: template
                .parameters
                .into_iter()
                .map(|(name, def)| {
                    (
                        name,
                        TemplateParameter {
                            param_type: def.param_type,
                            required: def.required,
                            default: def.default,
                        },
                    )
                })
                .collect(),
        });
    }
    templates.sort_by(|a, b| a.template_ref.cmp(&b.template_ref));
    EntryOptionsResult { templates }
}

/// Request params for `case.preflight`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PreflightParams {
    pub template_ref: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

/// Non-authoritative summary of one instantiated plan item — a projection for
/// display, never source truth (`derived view ≠ source truth`).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PlanItemSummary {
    pub plan_item_id: String,
    pub name: String,
    pub item_kind: String,
}

/// `case.preflight` result body. `ok: false` means `errors` is non-empty and
/// `items`/`precondition` describe the rejected draft (still useful for the
/// UI to show what would need to change).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PreflightResult {
    pub ok: bool,
    pub template_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    pub items: Vec<PlanItemSummary>,
    /// Carry this into `case.commit`'s `preconditions.records` unchanged.
    /// `None` when the template itself could not be loaded (nothing to pin).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub precondition: Option<RecordDigest>,
    /// Pins the whole plan this commit would create: the template's bytes, the
    /// params and the instantiated items. Ref `plan:<template_ref>`. Carry it
    /// into `case.commit`'s `preconditions.records` alongside (or instead of)
    /// `precondition`. `None` when the plan could not be built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_precondition: Option<RecordDigest>,
}

/// A precondition ref naming the plan a template + params would create.
pub fn plan_precondition_ref(template_ref: &str) -> String {
    format!("plan:{template_ref}")
}

/// A template read exactly once: the sha256 of its raw bytes and the template
/// parsed from those same bytes. Everything a request shows, pins or commits
/// comes from one of these, so a template edited mid-request cannot make the
/// displayed plan, its digests and the committed plan disagree.
#[derive(Clone, Debug)]
pub struct LoadedTemplate {
    pub template_ref: String,
    /// `sha256:<hex>` of the raw template file bytes.
    pub sha256: String,
    pub template: PlanTemplate,
}

#[cfg(test)]
pub(crate) static TEST_AFTER_TEMPLATE_READ: std::sync::Mutex<Option<fn(&Path)>> =
    std::sync::Mutex::new(None);

/// Read and parse `<root>/templates/<name>@<version>.yaml` once.
pub fn load_template_once(
    root: &Path,
    template_ref: &str,
) -> Result<LoadedTemplate, sea_forge_core::errors::ForgeError> {
    use sha2::{Digest, Sha256};
    let (name, version) = templates::parse_template_ref(template_ref)?;
    let path = root
        .join("templates")
        .join(format!("{name}@{version}.yaml"));
    let bytes = std::fs::read(&path).map_err(|e| {
        sea_forge_core::errors::ForgeError::io(format!("read template {}", path.display()), e)
    })?;
    let sha256 = format!("sha256:{}", hex::encode(Sha256::digest(&bytes)));
    // Test seam: lets a unit test change the file right after the read.
    #[cfg(test)]
    if let Some(hook) = *TEST_AFTER_TEMPLATE_READ.lock().unwrap() {
        hook(&path);
    }
    let text =
        std::str::from_utf8(&bytes).map_err(|e| sea_forge_core::errors::ForgeError::Config {
            class: "schema_error",
            path: path.clone(),
            message: e.to_string(),
        })?;
    let template = templates::load_from_text(text, &path)?;
    Ok(LoadedTemplate {
        template_ref: template_ref.to_string(),
        sha256,
        template,
    })
}

/// The value whose digest a `template:<ref>` precondition pins: the sha256 of
/// the template file's bytes, as a JSON string.
pub fn template_digest_source(loaded: &LoadedTemplate) -> serde_json::Value {
    serde_json::json!(loaded.sha256)
}

/// The value whose digest a `plan:<template_ref>` precondition pins: canonical
/// JSON `{template_sha256, params, items}`, built from `loaded` alone. `None`
/// if the template cannot be instantiated or validated with `params`.
pub fn plan_digest_source(
    loaded: &LoadedTemplate,
    params: &BTreeMap<String, String>,
) -> Option<serde_json::Value> {
    let mut plan = templates::instantiate(
        &loaded.template,
        params,
        "preflight",
        "preflight",
        "preflight",
    )
    .ok()?;
    validate_proposal(&mut plan).ok()?;
    Some(serde_json::json!({
        "template_sha256": loaded.sha256,
        "params": params,
        "items": serde_json::to_value(&plan.items).ok()?,
    }))
}

/// A precondition ref naming a template, pinned to its current on-disk bytes.
pub fn template_precondition_ref(template_ref: &str) -> String {
    format!("template:{template_ref}")
}

fn record_digest(r#ref: String, value: Option<serde_json::Value>) -> Option<RecordDigest> {
    let expected_digest = digest_of(&value?).ok()?;
    Some(RecordDigest {
        r#ref,
        expected_digest,
    })
}

/// Build the `case.preflight` view. Infallible in the sense that a bad
/// template ref or invalid params is reported as `ok: false` with `errors`,
/// never propagated as a transport error. The template is read once: the
/// items shown and both digests describe the same bytes.
pub fn preflight(root: &Path, params: PreflightParams) -> PreflightResult {
    let loaded = match load_template_once(root, &params.template_ref) {
        Ok(loaded) => loaded,
        Err(error) => {
            return PreflightResult {
                ok: false,
                template_ref: params.template_ref,
                errors: vec![error.to_string()],
                items: Vec::new(),
                precondition: None,
                plan_precondition: None,
            }
        }
    };

    // Placeholder ids: `case.commit` mints the real case_id/run_id inside
    // `case_dispatch::submit`; `validate_proposal` never inspects these
    // fields, only `items`.
    let mut plan: CasePlan = match templates::instantiate(
        &loaded.template,
        &params.params,
        "preflight",
        "preflight",
        "preflight",
    ) {
        Ok(plan) => plan,
        Err(error) => {
            return PreflightResult {
                ok: false,
                template_ref: params.template_ref.clone(),
                errors: vec![error.to_string()],
                items: Vec::new(),
                precondition: record_digest(
                    template_precondition_ref(&params.template_ref),
                    Some(template_digest_source(&loaded)),
                ),
                plan_precondition: None,
            }
        }
    };

    let validation = validate_proposal(&mut plan);
    let items = plan
        .items
        .iter()
        .map(|item| PlanItemSummary {
            plan_item_id: item.plan_item_id.clone(),
            name: item.name.clone(),
            item_kind: format!("{:?}", item.item_kind),
        })
        .collect();

    let precondition = record_digest(
        template_precondition_ref(&params.template_ref),
        Some(template_digest_source(&loaded)),
    );
    let plan_precondition = record_digest(
        plan_precondition_ref(&params.template_ref),
        plan_digest_source(&loaded, &params.params),
    );

    match validation {
        Ok(()) => PreflightResult {
            ok: true,
            template_ref: params.template_ref,
            errors: Vec::new(),
            items,
            precondition,
            plan_precondition,
        },
        Err(error) => PreflightResult {
            ok: false,
            template_ref: params.template_ref,
            errors: vec![error.to_string()],
            items,
            precondition,
            plan_precondition,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_planner::templates::sequential_agents_template;

    fn write_template(root: &Path) -> std::path::PathBuf {
        let dir = root.join("templates");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sequential_agents@0.1.0.yaml");
        let template = sequential_agents_template("ep").unwrap();
        std::fs::write(&path, sea_forge_core::yaml::to_string(&template).unwrap()).unwrap();
        path
    }

    #[test]
    fn preflight_digests_describe_the_template_it_displayed() {
        let root = tempfile::tempdir().unwrap();
        let path = write_template(root.path());
        let original = std::fs::read(&path).unwrap();
        let params = || PreflightParams {
            template_ref: "sequential_agents@0.1.0".into(),
            params: BTreeMap::new(),
        };
        // The template changes right after preflight reads it.
        *TEST_AFTER_TEMPLATE_READ.lock().unwrap() = Some(|path| {
            let mut bytes = std::fs::read(path).unwrap();
            bytes.extend_from_slice(b"# edited mid-request\n");
            std::fs::write(path, bytes).unwrap();
        });
        let shown = preflight(root.path(), params());
        *TEST_AFTER_TEMPLATE_READ.lock().unwrap() = None;
        // Digests computed from the bytes that were displayed.
        std::fs::write(&path, &original).unwrap();
        let loaded = load_template_once(root.path(), "sequential_agents@0.1.0").unwrap();
        let expected_plan = record_digest(
            plan_precondition_ref("sequential_agents@0.1.0"),
            plan_digest_source(&loaded, &BTreeMap::new()),
        )
        .unwrap();
        assert_eq!(
            shown.plan_precondition.unwrap().expected_digest,
            expected_plan.expected_digest
        );
        assert_eq!(
            shown.precondition.unwrap().expected_digest,
            digest_of(&template_digest_source(&loaded)).unwrap()
        );
    }

    #[test]
    fn commit_resolvers_use_the_template_already_loaded() {
        let root = tempfile::tempdir().unwrap();
        let path = write_template(root.path());
        let loaded = load_template_once(root.path(), "sequential_agents@0.1.0").unwrap();
        let before = plan_digest_source(&loaded, &BTreeMap::new());
        // The file changes after the request read it; the loaded value does not.
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.extend_from_slice(b"# edited\n");
        std::fs::write(&path, bytes).unwrap();
        assert_eq!(plan_digest_source(&loaded, &BTreeMap::new()), before);
        let reread = load_template_once(root.path(), "sequential_agents@0.1.0").unwrap();
        assert_ne!(plan_digest_source(&reread, &BTreeMap::new()), before);
    }
}
