use sea_forge_core::{
    errors::ForgeError,
    types::{CasePlan, ItemKind, JobContract, Operation, OriginRef, PlanItem, SettlementCriteria},
    RECORD_VERSION,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path};

/// A plan template parameter definition.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterDef {
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// A plan template.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PlanTemplate {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parameters: BTreeMap<String, ParameterDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub origin_refs: Vec<OriginRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_contract: Option<JobContract>,
    pub plan: TemplatePlan,
}

/// The plan body inside a template, with `${param}` placeholders in
/// operation payload values and criteria values.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemplatePlan {
    pub items: Vec<TemplateItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemplateItem {
    pub plan_item_id: String,
    pub name: String,
    #[serde(default)]
    pub operations: Vec<TemplateOperation>,
    pub settlement_criteria: SettlementCriteria,
    #[serde(default)]
    pub item_kind: ItemKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_class: Option<String>,
    #[serde(default)]
    pub markers: sea_forge_core::types::ItemMarkers,
    #[serde(default = "default_max_instances")]
    pub max_instances: u32,
    /// Optional environment reference (§7.6) — projected to PlanItem.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

fn default_max_instances() -> u32 {
    1
}

/// Operation with string values that may contain `${param}` placeholders.
/// The `kind` and `argv[0]` fields are literal — no substitution allowed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TemplateOperation {
    WriteFile { path: String, content_hint: String },
    ExecuteCommand { argv: Vec<String>, cwd: String },
}

/// Validate that no `${param}` appears in a forbidden position.
/// Forbidden: `kind` (tag), `argv[0]`, `plan_item_id`, `name`, sentry structure.
fn check_substitution_sites(template: &PlanTemplate) -> Result<(), ForgeError> {
    for item in &template.plan.items {
        if has_param(&item.plan_item_id) || has_param(&item.name) {
            return Err(ForgeError::Config {
                class: "schema_error",
                path: std::path::PathBuf::new(),
                message: "template parameter in ID or name is forbidden".into(),
            });
        }
        if item.sandbox_class.as_deref().is_some_and(has_param) {
            return Err(ForgeError::Config {
                class: "schema_error",
                path: std::path::PathBuf::new(),
                message: "template parameter in sandbox_class is forbidden".into(),
            });
        }
        for op in &item.operations {
            if let TemplateOperation::ExecuteCommand { argv, .. } = op {
                if !argv.is_empty() && has_param(&argv[0]) {
                    return Err(ForgeError::Config {
                        class: "schema_error",
                        path: std::path::PathBuf::new(),
                        message: "template parameter in argv[0] is forbidden".into(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn has_param(s: &str) -> bool {
    s.contains("${")
}

/// Resolve parameters: apply defaults, check required, return resolved map.
fn resolve_params(
    template: &PlanTemplate,
    provided: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, ForgeError> {
    if let Some(name) = provided
        .keys()
        .find(|name| !template.parameters.contains_key(*name))
    {
        return Err(ForgeError::Input(format!(
            "unknown template parameter: {name}"
        )));
    }
    let mut resolved = BTreeMap::new();
    for (name, def) in &template.parameters {
        let value = if let Some(v) = provided.get(name) {
            v.clone()
        } else if let Some(d) = &def.default {
            d.clone()
        } else if def.required {
            return Err(ForgeError::Input(format!(
                "missing required template parameter: {name}"
            )));
        } else {
            continue;
        };
        // Basic type validation
        match def.param_type.as_str() {
            "int" => {
                value
                    .parse::<i64>()
                    .map_err(|_| ForgeError::Input(format!("parameter {name} must be int")))?;
            }
            "bool" => {
                if !matches!(value.as_str(), "true" | "false") {
                    return Err(ForgeError::Input(format!("parameter {name} must be bool")));
                }
            }
            "string" => {}
            "path" => {
                let path = Path::new(&value);
                if value.is_empty()
                    || path.is_absolute()
                    || path.components().any(|component| {
                        matches!(
                            component,
                            Component::ParentDir | Component::RootDir | Component::Prefix(_)
                        )
                    })
                {
                    return Err(ForgeError::Input(format!(
                        "parameter {name} must be a safe relative path"
                    )));
                }
            }
            other => {
                return Err(ForgeError::Input(format!(
                    "unknown parameter type: {other}"
                )))
            }
        }
        resolved.insert(name.clone(), value);
    }
    Ok(resolved)
}

fn substitute(s: &str, params: &BTreeMap<String, String>) -> String {
    let mut result = s.to_string();
    for (k, v) in params {
        result = result.replace(&format!("${{{k}}}"), v);
    }
    result
}

/// Instantiate a template into a CasePlan.
/// Same template + same params always yields an identical CasePlan.
pub fn instantiate(
    template: &PlanTemplate,
    params: &BTreeMap<String, String>,
    case_id: &str,
    run_id: &str,
    intent_id: &str,
) -> Result<CasePlan, ForgeError> {
    check_substitution_sites(template)?;
    let resolved = resolve_params(template, params)?;
    let items = template
        .plan
        .items
        .iter()
        .map(|ti| {
            Ok(PlanItem {
                plan_item_id: ti.plan_item_id.clone(),
                name: ti.name.clone(),
                operations: ti
                    .operations
                    .iter()
                    .map(|op| match op {
                        TemplateOperation::WriteFile { path, content_hint } => {
                            Operation::WriteFile {
                                path: substitute(path, &resolved),
                                content_hint: substitute(content_hint, &resolved),
                            }
                        }
                        TemplateOperation::ExecuteCommand { argv, cwd } => {
                            Operation::ExecuteCommand {
                                argv: argv
                                    .iter()
                                    .enumerate()
                                    .map(|(i, a)| {
                                        if i == 0 {
                                            a.clone()
                                        } else {
                                            substitute(a, &resolved)
                                        }
                                    })
                                    .collect(),
                                cwd: substitute(cwd, &resolved),
                            }
                        }
                    })
                    .collect(),
                entry_criteria: vec![],
                exit_criteria: vec![],
                settlement_criteria: SettlementCriteria {
                    require_exit_zero: ti.settlement_criteria.require_exit_zero,
                    required_artifacts: ti
                        .settlement_criteria
                        .required_artifacts
                        .iter()
                        .map(|value| substitute(value, &resolved))
                        .collect(),
                    stdout_must_contain: ti
                        .settlement_criteria
                        .stdout_must_contain
                        .as_deref()
                        .map(|value| substitute(value, &resolved)),
                    require_approval: ti.settlement_criteria.require_approval,
                    ..ti.settlement_criteria.clone()
                },
                settlement_criteria_ref: None,
                item_kind: ti.item_kind.clone(),
                sandbox_class: ti.sandbox_class.clone(),
                parent_stage: None,
                markers: ti.markers.clone(),
                max_instances: ti.max_instances,
                depends_on: vec![],
                environment: ti.environment.clone(),
            })
        })
        .collect::<Result<Vec<_>, ForgeError>>()?;
    Ok(CasePlan {
        version: RECORD_VERSION.into(),
        plan_id: "plan_01".into(),
        case_id: case_id.into(),
        run_id: run_id.into(),
        intent_id: intent_id.into(),
        items,
        template_ref: Some(format!("{}@{}", template.name, template.version)),
        job_contract_ref: None,
    })
}

/// Load a template from a YAML file.
pub fn load(path: &Path) -> Result<PlanTemplate, ForgeError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ForgeError::io(format!("read template {}", path.display()), e))?;
    let template: PlanTemplate = serde_yaml::from_str(&text).map_err(|e| ForgeError::Config {
        class: "schema_error",
        path: path.into(),
        message: e.to_string(),
    })?;
    // Validate name pattern
    if !template
        .name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    {
        return Err(ForgeError::Config {
            class: "schema_error",
            path: path.into(),
            message: "template name must be [a-z0-9_-]+".into(),
        });
    }
    check_substitution_sites(&template)?;
    Ok(template)
}

fn parse_template_ref(reference: &str) -> Result<(&str, &str), ForgeError> {
    let (name, version) = reference
        .split_once('@')
        .ok_or_else(|| ForgeError::Input("template ref must be name@version".into()))?;
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        || version.is_empty()
        || !version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(ForgeError::Input("invalid template ref".into()));
    }
    Ok((name, version))
}

/// Load a template from `<root>/templates/<name>@<version>.yaml` and pin its
/// first-used byte hash. A later content change under the same version fails.
pub fn load_pinned(root: &Path, reference: &str) -> Result<PlanTemplate, ForgeError> {
    let (name, version) = parse_template_ref(reference)?;
    let templates = root.join("templates");
    let path = templates.join(format!("{name}@{version}.yaml"));
    let bytes = std::fs::read(&path)
        .map_err(|error| ForgeError::io(format!("read template {}", path.display()), error))?;
    let digest = format!("sha256:{:x}", Sha256::digest(&bytes));
    let pins = templates.join(".pins");
    std::fs::create_dir_all(&pins)
        .map_err(|error| ForgeError::io("create template pins directory", error))?;
    let pin = pins.join(format!("{name}@{version}.sha256"));
    match OpenOptions::new().create_new(true).write(true).open(&pin) {
        Ok(mut file) => file
            .write_all(digest.as_bytes())
            .map_err(|error| ForgeError::io("write template pin", error))?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let expected = std::fs::read_to_string(&pin)
                .map_err(|error| ForgeError::io("read template pin", error))?;
            if expected != digest {
                return Err(ForgeError::Config {
                    class: "template_changed_error",
                    path,
                    message: "template bytes changed without a version bump".into(),
                });
            }
        }
        Err(error) => return Err(ForgeError::io("create template pin", error)),
    }
    let template = load(&path)?;
    if template.name != name || template.version != version {
        return Err(ForgeError::Config {
            class: "schema_error",
            path,
            message: "template name/version do not match its filename".into(),
        });
    }
    Ok(template)
}

/// Store the built-in demo template when absent. Existing bytes are never
/// overwritten; version changes require a new filename.
pub fn store_builtin(root: &Path) -> Result<std::path::PathBuf, ForgeError> {
    let template = sea_model_demo_template();
    let templates = root.join("templates");
    std::fs::create_dir_all(&templates)
        .map_err(|error| ForgeError::io("create templates directory", error))?;
    let path = templates.join(format!("{}@{}.yaml", template.name, template.version));
    match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(mut file) => file
            .write_all(
                serde_yaml::to_string(&template)
                    .map_err(|error| ForgeError::Serialization(error.to_string()))?
                    .as_bytes(),
            )
            .map_err(|error| ForgeError::io("write built-in template", error))?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(ForgeError::io("create built-in template", error)),
    }
    Ok(path)
}

/// The built-in `sea_model_demo@0.1.0` template from the slice demo plan.
pub fn sea_model_demo_template() -> PlanTemplate {
    PlanTemplate {
        name: "sea_model_demo".into(),
        version: "0.1.0".into(),
        description: "Built-in demo: generate and validate a .sea model".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![TemplateItem {
                plan_item_id: "item_01".into(),
                name: "generate_and_validate_sea_model".into(),
                operations: vec![
                    TemplateOperation::WriteFile {
                        path: "model.sea".into(),
                        content_hint: r#"{"domain": "demo", "entities": [{"name": "Sample"}]}"#
                            .into(),
                    },
                    TemplateOperation::ExecuteCommand {
                        argv: vec!["sea-forge".into(), "validate".into(), "model.sea".into()],
                        cwd: ".".into(),
                    },
                ],
                settlement_criteria: SettlementCriteria {
                    require_exit_zero: true,
                    required_artifacts: vec!["model.sea".into()],
                    stdout_must_contain: Some("sea-forge: model valid".into()),
                    ..Default::default()
                },
                item_kind: ItemKind::SandboxedTask,
                sandbox_class: None,
                markers: Default::default(),
                max_instances: 1,
                environment: None,
            }],
        },
    }
}
