use sea_forge_core::{
    errors::ForgeError,
    types::{
        CasePlan, EntryCriteriaMode, ItemKind, ItemMarkers, JobContract, Operation, OriginRef,
        OriginRefKind, OriginRole, PlanItem, Sentry, SentryPredicate, SentryTrigger,
        SettlementCriteria,
    },
    RECORD_VERSION,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

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
    /// Typed deterministic item expansion (§7.5 E16a). Empty by default —
    /// old templates are byte-compatible.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeated: Vec<RepeatedItem>,
}

/// A bounded hard maximum on how many items a single `RepeatedItem` group
/// may expand to.
pub const MAX_REPEATED_ENTRIES: usize = 32;

/// One entry in a `RepeatedItem` group's typed list.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepeatEntry {
    /// Stable key; the expanded item's ID is `{id_prefix}_{key}`.
    pub key: String,
    /// Entry-scoped parameter overrides, merged over the template's
    /// resolved parameters for this entry's substitution pass only.
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    /// Extra sentries appended to the shared body's `entry_criteria` for
    /// this entry only — e.g. a sequential chain references the prior
    /// entry's deterministic ID here.
    #[serde(default)]
    pub entry_criteria: Vec<Sentry>,
}

/// Typed deterministic expansion of a shared item body into N plan items
/// with stable, order/key-derived IDs (§7.5 E16a slice 6.1). `item`'s
/// `plan_item_id` must be the empty-string sentinel — expansion derives the
/// real ID from `id_prefix` + each entry's `key`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepeatedItem {
    pub id_prefix: String,
    pub item: TemplateItem,
    pub entries: Vec<RepeatEntry>,
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
    /// E8 control-flow vocabulary (§7.5): projected byte-for-byte to PlanItem.
    #[serde(default)]
    pub entry_criteria: Vec<Sentry>,
    #[serde(default)]
    pub entry_criteria_mode: EntryCriteriaMode,
    #[serde(default)]
    pub exit_criteria: Vec<Sentry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_stage: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

fn default_max_instances() -> u32 {
    1
}

/// Operation with string values that may contain `${param}` placeholders.
/// The `kind` and `argv[0]` fields are literal — no substitution allowed.
/// `AgentTask.endpoint_ref` is likewise literal: it is the delegation
/// target's identity, the same privilege class as an executable's argv[0].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TemplateOperation {
    WriteFile {
        path: String,
        content_hint: String,
    },
    ExecuteCommand {
        argv: Vec<String>,
        cwd: String,
    },
    AgentTask {
        endpoint_ref: String,
        instruction: String,
        max_turns: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token_budget: Option<u64>,
    },
}

/// Validate that no `${param}` appears in a forbidden position for one item
/// body. Forbidden: `kind` (tag), `argv[0]`, `endpoint_ref`, `plan_item_id`,
/// `name`, sentry event kinds, sentry source IDs, `item_kind` (enum tag),
/// and sandbox classes.
fn check_item_substitution_sites(item: &TemplateItem) -> Result<(), ForgeError> {
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
    for sentry in item.entry_criteria.iter().chain(&item.exit_criteria) {
        if has_param(&sentry.on.source) {
            return Err(ForgeError::Config {
                class: "schema_error",
                path: std::path::PathBuf::new(),
                message: "template parameter in sentry source is forbidden".into(),
            });
        }
        if has_param(&sentry.on.event) {
            return Err(ForgeError::Config {
                class: "schema_error",
                path: std::path::PathBuf::new(),
                message: "template parameter in sentry event kind is forbidden".into(),
            });
        }
    }
    for op in &item.operations {
        match op {
            TemplateOperation::ExecuteCommand { argv, .. } => {
                if !argv.is_empty() && has_param(&argv[0]) {
                    return Err(ForgeError::Config {
                        class: "schema_error",
                        path: std::path::PathBuf::new(),
                        message: "template parameter in argv[0] is forbidden".into(),
                    });
                }
            }
            TemplateOperation::AgentTask { endpoint_ref, .. } => {
                if has_param(endpoint_ref) {
                    return Err(ForgeError::Config {
                        class: "schema_error",
                        path: std::path::PathBuf::new(),
                        message: "template parameter in endpoint_ref is forbidden".into(),
                    });
                }
            }
            TemplateOperation::WriteFile { .. } => {}
        }
    }
    Ok(())
}

/// Validate substitution sites across every flat item and every repeated
/// item's shared body.
fn check_substitution_sites(template: &PlanTemplate) -> Result<(), ForgeError> {
    for item in &template.plan.items {
        check_item_substitution_sites(item)?;
    }
    for group in &template.plan.repeated {
        check_item_substitution_sites(&group.item)?;
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

/// Project one resolved `TemplateItem` body into a `PlanItem` with the given
/// final ID and extra (entry-scoped) sentries appended to its shared
/// `entry_criteria`.
fn project_item(
    ti: &TemplateItem,
    plan_item_id: String,
    resolved: &BTreeMap<String, String>,
    extra_entry_criteria: &[Sentry],
) -> PlanItem {
    PlanItem {
        plan_item_id,
        name: ti.name.clone(),
        operations: ti
            .operations
            .iter()
            .map(|op| match op {
                TemplateOperation::WriteFile { path, content_hint } => Operation::WriteFile {
                    path: substitute(path, resolved),
                    content_hint: substitute(content_hint, resolved),
                },
                TemplateOperation::ExecuteCommand { argv, cwd } => Operation::ExecuteCommand {
                    argv: argv
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            if i == 0 {
                                a.clone()
                            } else {
                                substitute(a, resolved)
                            }
                        })
                        .collect(),
                    cwd: substitute(cwd, resolved),
                },
                TemplateOperation::AgentTask {
                    endpoint_ref,
                    instruction,
                    max_turns,
                    token_budget,
                } => Operation::AgentTask {
                    endpoint_ref: endpoint_ref.clone(),
                    instruction: substitute(instruction, resolved),
                    max_turns: *max_turns,
                    token_budget: *token_budget,
                    response_schema: None,
                    transcript_retention: None,
                },
            })
            .collect(),
        entry_criteria: ti
            .entry_criteria
            .iter()
            .cloned()
            .chain(extra_entry_criteria.iter().cloned())
            .collect(),
        entry_criteria_mode: ti.entry_criteria_mode,
        exit_criteria: ti.exit_criteria.clone(),
        settlement_criteria: SettlementCriteria {
            require_exit_zero: ti.settlement_criteria.require_exit_zero,
            required_artifacts: ti
                .settlement_criteria
                .required_artifacts
                .iter()
                .map(|value| substitute(value, resolved))
                .collect(),
            stdout_must_contain: ti
                .settlement_criteria
                .stdout_must_contain
                .as_deref()
                .map(|value| substitute(value, resolved)),
            require_approval: ti.settlement_criteria.require_approval,
            ..ti.settlement_criteria.clone()
        },
        settlement_criteria_ref: None,
        item_kind: ti.item_kind.clone(),
        sandbox_class: ti.sandbox_class.clone(),
        parent_stage: ti.parent_stage.clone(),
        markers: ti.markers.clone(),
        max_instances: ti.max_instances,
        depends_on: ti.depends_on.clone(),
        environment: ti.environment.clone(),
        proposed_by: None,
    }
}

fn valid_repeat_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Validate and expand one `RepeatedItem` group into its final plan items.
fn expand_repeated(
    group: &RepeatedItem,
    resolved: &BTreeMap<String, String>,
) -> Result<Vec<PlanItem>, ForgeError> {
    if !valid_repeat_token(&group.id_prefix) {
        return Err(ForgeError::Config {
            class: "schema_error",
            path: PathBuf::new(),
            message: "repeated item id_prefix must be [a-z0-9_-]+".into(),
        });
    }
    if !group.item.plan_item_id.is_empty() {
        return Err(ForgeError::Config {
            class: "schema_error",
            path: PathBuf::new(),
            message: "repeated item body plan_item_id must be the empty sentinel".into(),
        });
    }
    if group.entries.is_empty() || group.entries.len() > MAX_REPEATED_ENTRIES {
        return Err(ForgeError::Config {
            class: "schema_error",
            path: PathBuf::new(),
            message: format!(
                "repeated item entries must be 1..={MAX_REPEATED_ENTRIES}, got {}",
                group.entries.len()
            ),
        });
    }
    let mut seen_keys = std::collections::HashSet::new();
    let mut items = Vec::with_capacity(group.entries.len());
    for entry in &group.entries {
        if !valid_repeat_token(&entry.key) {
            return Err(ForgeError::Config {
                class: "schema_error",
                path: PathBuf::new(),
                message: "repeated item entry key must be [a-z0-9_-]+".into(),
            });
        }
        if !seen_keys.insert(entry.key.clone()) {
            return Err(ForgeError::Config {
                class: "schema_error",
                path: PathBuf::new(),
                message: format!("duplicate repeated item entry key: {}", entry.key),
            });
        }
        let mut entry_resolved = resolved.clone();
        entry_resolved.extend(entry.params.iter().map(|(k, v)| (k.clone(), v.clone())));
        let plan_item_id = format!("{}_{}", group.id_prefix, entry.key);
        items.push(project_item(
            &group.item,
            plan_item_id,
            &entry_resolved,
            &entry.entry_criteria,
        ));
    }
    Ok(items)
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
    let mut items: Vec<PlanItem> = template
        .plan
        .items
        .iter()
        .map(|ti| project_item(ti, ti.plan_item_id.clone(), &resolved, &[]))
        .collect();
    for group in &template.plan.repeated {
        items.extend(expand_repeated(group, &resolved)?);
    }
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
///
/// `seed_domain_model_ref`/`seed_model_sha256` are threaded into
/// `odi_adlc_case@0.1.0`'s desired-outcome origin ref (Task 12 audit
/// remediation) — callers supply the validated seed model's real identity and
/// hash (e.g. from Task 11's self-model), never a fabricated placeholder.
pub fn store_builtin(
    root: &Path,
    seed_domain_model_ref: &str,
    seed_model_sha256: &str,
    default_endpoint_ref: &str,
) -> Result<std::path::PathBuf, ForgeError> {
    let template = sea_model_demo_template();
    let path = store_template(root, &template)?;
    // Also materialize the M10 ADLC/ODI built-ins.
    store_template(root, &adlc_case_template())?;
    store_template(
        root,
        &odi_adlc_case_template(seed_domain_model_ref, seed_model_sha256),
    )?;
    // Also materialize the M14 topology built-ins.
    store_template(root, &sequential_agents_template(default_endpoint_ref)?)?;
    store_template(root, &concurrent_agents_template(default_endpoint_ref)?)?;
    Ok(path)
}

/// Names of built-in templates that can be resolved by `template_ref`
/// (`name@version`) without re-reading a pinned file — used to decide whether
/// a submitted plan's criteria should derive from the template (Task 12 audit
/// remediation step 3) rather than from intent-only provenance.
pub fn is_built_in_template_ref(template_ref: &str) -> bool {
    matches!(
        template_ref,
        "sea_model_demo@0.1.0"
            | "adlc_case@0.1.0"
            | "odi_adlc_case@0.1.0"
            | "sequential_agents@0.1.0"
            | "concurrent_agents@0.1.0"
    )
}

/// Materialize a built-in template as `<root>/templates/<name>@<version>.yaml`
/// without overwriting an existing file (§0.3 source-owned assets).
fn store_template(root: &Path, template: &PlanTemplate) -> Result<PathBuf, ForgeError> {
    let templates = root.join("templates");
    std::fs::create_dir_all(&templates)
        .map_err(|error| ForgeError::io("create templates directory", error))?;
    let path = templates.join(format!("{}@{}.yaml", template.name, template.version));
    match OpenOptions::new().create_new(true).write(true).open(&path) {
        Ok(mut file) => file
            .write_all(
                serde_yaml::to_string(template)
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
                entry_criteria: vec![],
                entry_criteria_mode: Default::default(),
                exit_criteria: vec![],
                parent_stage: None,
                depends_on: vec![],
            }],
            repeated: vec![],
        },
    }
}

// ── M10 (E12 §7.5): ADLC/ODI built-in templates ──

/// Helper: a stage item (no operations, no sentry).
fn stage(id: &str, name: &str, entry: Vec<Sentry>) -> TemplateItem {
    TemplateItem {
        plan_item_id: id.into(),
        name: name.into(),
        operations: vec![],
        settlement_criteria: SettlementCriteria::default(),
        item_kind: ItemKind::Stage,
        sandbox_class: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        environment: None,
        entry_criteria: entry,
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        parent_stage: None,
        depends_on: vec![],
    }
}

/// Helper: a milestone item within a stage.
fn milestone(id: &str, name: &str, parent: &str) -> TemplateItem {
    TemplateItem {
        plan_item_id: id.into(),
        name: name.into(),
        operations: vec![],
        settlement_criteria: SettlementCriteria::default(),
        item_kind: ItemKind::Milestone,
        sandbox_class: None,
        markers: ItemMarkers::default(),
        max_instances: 1,
        environment: None,
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        parent_stage: Some(parent.into()),
        depends_on: vec![],
    }
}

/// Helper: a sandboxed task within a stage with optional sentries.
fn task(
    id: &str,
    name: &str,
    parent: &str,
    entry: Vec<Sentry>,
    markers: ItemMarkers,
    max_instances: u32,
) -> TemplateItem {
    TemplateItem {
        plan_item_id: id.into(),
        name: name.into(),
        operations: vec![],
        settlement_criteria: SettlementCriteria {
            require_exit_zero: true,
            ..Default::default()
        },
        item_kind: ItemKind::SandboxedTask,
        sandbox_class: Some("local".into()),
        markers,
        max_instances,
        environment: None,
        entry_criteria: entry,
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        parent_stage: Some(parent.into()),
        depends_on: vec![],
    }
}

/// Source-bound reactivation sentry: fires when `src` settles with `status`.
pub(crate) fn reactivation_sentry(src: &str, status: &str) -> Sentry {
    Sentry {
        on: SentryTrigger {
            source: src.into(),
            event: "settlement_status".into(),
        },
        if_predicate: Some(SentryPredicate::SettlementStatus {
            status: status.into(),
        }),
    }
}

fn milestone_sentry(src: &str) -> Sentry {
    Sentry {
        on: SentryTrigger {
            source: src.into(),
            event: "milestone_achieved".into(),
        },
        if_predicate: None,
    }
}

/// The built-in `adlc_case@0.1.0` template: Frame → Form → Build → Activate
/// with reactivation sentries (simulation rejection re-enters design) and a
/// discretionary-item slot (spec-adlc-thoth §7.5).
pub fn adlc_case_template() -> PlanTemplate {
    PlanTemplate {
        name: "adlc_case".into(),
        version: "0.1.0".into(),
        description: "ADLC lifecycle: Frame → Form → Build → Activate with reactivation".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![
                // Stage: Frame
                stage("stage_frame", "Frame", vec![]),
                task(
                    "task_preparation",
                    "Preparation & Hypothesis",
                    "stage_frame",
                    vec![],
                    ItemMarkers::default(),
                    1,
                ),
                milestone("ms_problem_framed", "Problem Framed", "stage_frame"),
                // Stage: Form (activated when Frame milestone achieved)
                stage(
                    "stage_form",
                    "Form",
                    vec![milestone_sentry("ms_problem_framed")],
                ),
                task(
                    "task_design",
                    "Design",
                    "stage_form",
                    // Reactivation: simulation rejection re-enters design
                    vec![reactivation_sentry("task_simulation", "rejected")],
                    ItemMarkers {
                        repetition: true,
                        ..Default::default()
                    },
                    3,
                ),
                task(
                    "task_simulation",
                    "Simulation",
                    "stage_form",
                    vec![milestone_sentry("task_design")],
                    ItemMarkers::default(),
                    1,
                ),
                milestone("ms_dev_authorized", "Development Authorized", "stage_form"),
                // Stage: Build
                stage(
                    "stage_build",
                    "Build",
                    vec![milestone_sentry("ms_dev_authorized")],
                ),
                task(
                    "task_implementation",
                    "Implementation",
                    "stage_build",
                    vec![],
                    ItemMarkers::default(),
                    1,
                ),
                milestone(
                    "ms_rc_accepted",
                    "Release Candidate Accepted",
                    "stage_build",
                ),
                // Stage: Activate
                stage(
                    "stage_activate",
                    "Activate",
                    vec![milestone_sentry("ms_rc_accepted")],
                ),
                task(
                    "task_deployment",
                    "Controlled Deployment",
                    "stage_activate",
                    vec![],
                    ItemMarkers::default(),
                    1,
                ),
                milestone(
                    "ms_activation_settled",
                    "Activation Settled",
                    "stage_activate",
                ),
                // Discretionary item slot (manual activation)
                TemplateItem {
                    plan_item_id: "disc_item".into(),
                    name: "Discretionary Item".into(),
                    operations: vec![],
                    settlement_criteria: SettlementCriteria::default(),
                    item_kind: ItemKind::SandboxedTask,
                    sandbox_class: Some("local".into()),
                    markers: ItemMarkers {
                        manual_activation: true,
                        ..Default::default()
                    },
                    max_instances: 1,
                    environment: None,
                    entry_criteria: vec![],
                    entry_criteria_mode: Default::default(),
                    exit_criteria: vec![],
                    parent_stage: None,
                    depends_on: vec![],
                },
            ],
            repeated: vec![],
        },
    }
}

/// The built-in `odi_adlc_case@0.1.0` template: prepends Job Framing and
/// Outcome Discovery/Selection stages before the ADLC lifecycle, binding
/// desired-outcome concept refs into the criteria origin_refs (§7.5).
///
/// `seed_domain_model_ref`/`seed_model_sha256` name and hash the validated
/// ADLC/ODI seed model (Task 11's self-model seed) that defines the "Desired
/// Outcome Criterion" concept this template's origin ref points at — callers
/// supply real, verified values; the template never fabricates its own
/// provenance (audit remediation Task 12, §10.4).
pub fn odi_adlc_case_template(
    seed_domain_model_ref: &str,
    seed_model_sha256: &str,
) -> PlanTemplate {
    // Start from the ADLC case and prepend ODI stages + desired-outcome provenance.
    let adlc = adlc_case_template();
    let mut items = vec![
        // Stage: Job Framing
        stage("stage_job_framing", "Job Framing", vec![]),
        task(
            "task_job_discovery",
            "Job Discovery",
            "stage_job_framing",
            vec![],
            ItemMarkers::default(),
            1,
        ),
        milestone("ms_job_framed", "Job Framed", "stage_job_framing"),
        // Stage: Outcome Discovery & Selection
        stage(
            "stage_outcome_discovery",
            "Outcome Discovery",
            vec![milestone_sentry("ms_job_framed")],
        ),
        task(
            "task_outcome_discovery",
            "Outcome Discovery",
            "stage_outcome_discovery",
            vec![],
            ItemMarkers::default(),
            1,
        ),
        task(
            "task_outcome_selection",
            "Outcome Selection",
            "stage_outcome_discovery",
            vec![],
            ItemMarkers::default(),
            1,
        ),
        milestone(
            "ms_outcome_selected",
            "Outcome Selected",
            "stage_outcome_discovery",
        ),
    ];
    // The ADLC stages now activate after outcome selection.
    // Adjust stage_frame entry criteria to depend on outcome selection.
    let mut adlc_items = adlc.plan.items;
    if let Some(frame) = adlc_items
        .iter_mut()
        .find(|i| i.plan_item_id == "stage_frame")
    {
        frame.entry_criteria = vec![milestone_sentry("ms_outcome_selected")];
    }
    items.append(&mut adlc_items);
    PlanTemplate {
        name: "odi_adlc_case".into(),
        version: "0.1.0".into(),
        description:
            "ODI + ADLC: Job Framing → Outcome Discovery → Frame → Form → Build → Activate".into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![OriginRef {
            kind: OriginRefKind::DesiredOutcome,
            reference: "Desired Outcome Criterion".into(),
            sha256: seed_model_sha256.into(),
            role: OriginRole::DesiredResult,
            evidence_refs: vec![],
            domain_model_ref: Some(seed_domain_model_ref.into()),
        }],
        job_contract: None,
        plan: TemplatePlan {
            items,
            repeated: vec![],
        },
    }
}

// ── M14 (E16a §7.5): deterministic topology built-ins ──

/// Fallback endpoint reference for built-in topology templates when a
/// caller does not need real dispatch (e.g. CLI template materialization
/// for criteria derivation only, `plan_pipeline::run_plan_inner`). Any
/// caller that intends to actually dispatch generated items must supply
/// its own real, registered endpoint ID instead (audit remediation
/// Task 17) — this constant exists only to satisfy the endpoint ID
/// grammar, not to name a real agent.
pub const DEFAULT_TOPOLOGY_ENDPOINT_REF: &str = "agent_default";

/// Same grammar `AgentEndpointConfig::validate` (`sea-forge-agent`) enforces
/// for registered endpoint IDs: non-empty, <=64 chars, lowercase
/// ascii/digit/`_`/`-` only. Topology built-ins bind their endpoint_ref
/// literally at construction time (never a `${param}` placeholder — see
/// `check_item_substitution_sites`), so an invalid ID (e.g. containing `:`)
/// must fail here instead of only at dispatch-time preflight (audit
/// remediation Task 17).
fn validate_endpoint_ref(endpoint_ref: &str) -> Result<(), ForgeError> {
    let valid = !endpoint_ref.is_empty()
        && endpoint_ref.len() <= 64
        && endpoint_ref
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(ForgeError::Config {
            class: "schema_error",
            path: PathBuf::new(),
            message: format!("invalid agent endpoint_ref '{endpoint_ref}'"),
        })
    }
}

/// Shared `AgentTask` body for a repeated topology branch/step. Sentinel
/// `plan_item_id` (`""`) — expansion derives the real ID from the group's
/// `id_prefix` and each entry's key. `instruction` is entry-scoped
/// (substituted per entry, not declared as a template-level parameter).
/// `endpoint_ref` must already be validated by the caller.
fn agent_task_body(endpoint_ref: &str) -> TemplateItem {
    TemplateItem {
        plan_item_id: "".into(),
        name: "agent_branch".into(),
        operations: vec![TemplateOperation::AgentTask {
            endpoint_ref: endpoint_ref.into(),
            instruction: "${instruction}".into(),
            max_turns: 1,
            token_budget: None,
        }],
        settlement_criteria: SettlementCriteria::default(),
        item_kind: ItemKind::AgentTask,
        sandbox_class: None,
        // Required: without this, `can_auto_complete` sees zero required
        // items and completes the case before any branch ever dispatches
        // (audit remediation Task 17 — the defect that made these
        // templates unable to prove real end-to-end dispatch).
        markers: ItemMarkers {
            required: true,
            ..ItemMarkers::default()
        },
        max_instances: 1,
        environment: None,
        entry_criteria: vec![],
        entry_criteria_mode: Default::default(),
        exit_criteria: vec![],
        parent_stage: None,
        depends_on: vec![],
    }
}

fn repeated_item_id(id_prefix: &str, key: &str) -> String {
    format!("{id_prefix}_{key}")
}

/// The built-in `sequential_agents@0.1.0` template: three `AgentTask` steps
/// chained by settlement-accepted sentries, each step gated on the prior
/// step's acceptance (§7.5 E16a slice 6.3, T14.1). `endpoint_ref` must be a
/// real, registry-resolvable endpoint ID (`^[a-z0-9_-]{1,64}$`) — an invalid
/// ID (e.g. containing `:`) is rejected here rather than reaching dispatch
/// preflight (audit remediation Task 17).
pub fn sequential_agents_template(endpoint_ref: &str) -> Result<PlanTemplate, ForgeError> {
    validate_endpoint_ref(endpoint_ref)?;
    let id_prefix = "step";
    let keys = ["1", "2", "3"];
    let entries = keys
        .iter()
        .enumerate()
        .map(|(i, key)| RepeatEntry {
            key: (*key).into(),
            params: BTreeMap::from([(
                "instruction".into(),
                format!("perform sequential step {key}"),
            )]),
            entry_criteria: if i == 0 {
                vec![]
            } else {
                vec![reactivation_sentry(
                    &repeated_item_id(id_prefix, keys[i - 1]),
                    "accepted",
                )]
            },
        })
        .collect();
    Ok(PlanTemplate {
        name: "sequential_agents".into(),
        version: "0.1.0".into(),
        description:
            "Deterministic chain of agent-task steps, each gated on the prior step's acceptance"
                .into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![],
            repeated: vec![RepeatedItem {
                id_prefix: id_prefix.into(),
                item: agent_task_body(endpoint_ref),
                entries,
            }],
        },
    })
}

/// The built-in `concurrent_agents@0.1.0` template: N independent
/// `AgentTask` branches plus a flat rollup `Milestone` with
/// `entry_criteria_mode: All`, naming every branch — the rollup fires only
/// when every branch settles accepted (§7.5 E16a slice 6.3, T14.2/T14.3).
/// `endpoint_ref` must be a real, registry-resolvable endpoint ID
/// (`^[a-z0-9_-]{1,64}$`) — see `sequential_agents_template`.
pub fn concurrent_agents_template(endpoint_ref: &str) -> Result<PlanTemplate, ForgeError> {
    validate_endpoint_ref(endpoint_ref)?;
    let id_prefix = "branch";
    let keys = ["a", "b", "c"];
    let entries: Vec<RepeatEntry> = keys
        .iter()
        .map(|key| RepeatEntry {
            key: (*key).into(),
            params: BTreeMap::from([(
                "instruction".into(),
                format!("perform concurrent branch {key}"),
            )]),
            entry_criteria: vec![],
        })
        .collect();
    let rollup = TemplateItem {
        plan_item_id: "ms_all_branches_accepted".into(),
        name: "All Branches Accepted".into(),
        operations: vec![],
        settlement_criteria: SettlementCriteria::default(),
        item_kind: ItemKind::Milestone,
        sandbox_class: None,
        // Required: the rollup is the topology's completion gate, not an
        // optional side milestone (audit remediation Task 17).
        markers: ItemMarkers {
            required: true,
            ..ItemMarkers::default()
        },
        max_instances: 1,
        environment: None,
        entry_criteria: keys
            .iter()
            .map(|key| reactivation_sentry(&repeated_item_id(id_prefix, key), "accepted"))
            .collect(),
        entry_criteria_mode: EntryCriteriaMode::All,
        exit_criteria: vec![],
        parent_stage: None,
        depends_on: vec![],
    };
    Ok(PlanTemplate {
        name: "concurrent_agents".into(),
        version: "0.1.0".into(),
        description:
            "Deterministic fan-out of independent agent-task branches with an all-of success rollup"
                .into(),
        parameters: BTreeMap::new(),
        origin_refs: vec![],
        job_contract: None,
        plan: TemplatePlan {
            items: vec![rollup],
            repeated: vec![RepeatedItem {
                id_prefix: id_prefix.into(),
                item: agent_task_body(endpoint_ref),
                entries,
            }],
        },
    })
}
