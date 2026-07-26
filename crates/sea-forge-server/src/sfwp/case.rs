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
}

/// A precondition ref naming a template, pinned to its current on-disk bytes.
pub fn template_precondition_ref(template_ref: &str) -> String {
    format!("template:{template_ref}")
}

/// The `RecordResolver`-compatible current value for a `template:<ref>`
/// precondition ref: the sha256 of the template file's current raw bytes,
/// wrapped as a JSON value so `sfwp::precondition::digest_of` can hash it
/// exactly the way `case.preflight` computed the expected digest.
pub fn resolve_template_digest_source(
    root: &Path,
    template_ref: &str,
) -> Option<serde_json::Value> {
    let (name, version) = template_ref.split_once('@')?;
    let path = root
        .join("templates")
        .join(format!("{name}@{version}.yaml"));
    let bytes = std::fs::read(&path).ok()?;
    use sha2::{Digest, Sha256};
    let sha = format!("sha256:{:x}", Sha256::digest(&bytes));
    Some(serde_json::json!(sha))
}

/// Load a template by ref without pinning it (pinning is a `case.commit`-time
/// concern via the existing `templates::load_pinned`, not preflight's — a
/// preflight must be freely repeatable and must not itself create a pin that
/// could make a *later*, legitimate template content fix look like tampering).
fn load_template_for_preflight(
    root: &Path,
    template_ref: &str,
) -> Result<PlanTemplate, sea_forge_core::errors::ForgeError> {
    let (name, version) = template_ref.split_once('@').ok_or_else(|| {
        sea_forge_core::errors::ForgeError::Input("template ref must be name@version".into())
    })?;
    let path = root
        .join("templates")
        .join(format!("{name}@{version}.yaml"));
    templates::load(&path)
}

/// Build the `case.preflight` view. Infallible in the sense that a bad
/// template ref or invalid params is reported as `ok: false` with `errors`,
/// never propagated as a transport error — mirroring `readiness::get`'s
/// "an inspect method always answers with a shaped view" discipline, except
/// preflight's *purpose* is exactly to report validation failure, so `ok`
/// carries that signal explicitly instead of collapsing it into `unknown`.
pub fn preflight(root: &Path, params: PreflightParams) -> PreflightResult {
    let template = match load_template_for_preflight(root, &params.template_ref) {
        Ok(template) => template,
        Err(error) => {
            return PreflightResult {
                ok: false,
                template_ref: params.template_ref,
                errors: vec![error.to_string()],
                items: Vec::new(),
                precondition: None,
            }
        }
    };

    // Placeholder ids: `case.commit` mints the real case_id/run_id inside
    // `case_dispatch::submit`; `validate_proposal` never inspects these
    // fields, only `items`.
    let mut plan: CasePlan = match templates::instantiate(
        &template,
        &params.params,
        "preflight",
        "preflight",
        "preflight",
    ) {
        Ok(plan) => plan,
        Err(error) => {
            return PreflightResult {
                ok: false,
                template_ref: params.template_ref,
                errors: vec![error.to_string()],
                items: Vec::new(),
                precondition: None,
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

    // The precondition pins the template's *current* bytes, computed the same
    // way `resolve_template_digest_source` recomputes it at commit time.
    let precondition = resolve_template_digest_source(root, &params.template_ref)
        .and_then(|value| digest_of(&value).ok())
        .map(|expected_digest| RecordDigest {
            r#ref: template_precondition_ref(&params.template_ref),
            expected_digest,
        });

    match validation {
        Ok(()) => PreflightResult {
            ok: true,
            template_ref: params.template_ref,
            errors: Vec::new(),
            items,
            precondition,
        },
        Err(error) => PreflightResult {
            ok: false,
            template_ref: params.template_ref,
            errors: vec![error.to_string()],
            items,
            precondition,
        },
    }
}
