//! The frozen FDE contract as data, owned by the case-runner (WP-21 part B,
//! review round 5): the per-kind registry (README section 5, with its
//! `Suspended` and `Closed` columns) and the schemas, embedded from
//! `contracts/fde` with `include_str!`. The guard, the CLI and the server all
//! validate through this module, so no host has to install a validator.

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

const REGISTRY: &str =
    include_str!("../../../../contracts/fde/fixtures/case-record-kinds/valid/registry.json");
const SCHEMAS: [&str; 7] = [
    include_str!("../../../../contracts/fde/anchor.v1.json"),
    include_str!("../../../../contracts/fde/assignment.v1.json"),
    include_str!("../../../../contracts/fde/assignment-binding.v1.json"),
    include_str!("../../../../contracts/fde/case-record-append.v1.json"),
    include_str!("../../../../contracts/fde/case-record-append.v2.json"),
    include_str!("../../../../contracts/fde/field-case.v1.json"),
    include_str!("../../../../contracts/fde/frozen-reference.v1.json"),
];

/// What a suspended or closed case does with a kind (the registry's columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Accept,
    Refuse,
    /// Only the case-level `continue` (lifts) or `close-*` (closes).
    LiftOrCloseOnly,
}

/// One registry row: the rules and the payload schema of a record kind.
#[derive(Debug, Clone)]
pub struct KindRules {
    pub suspended: Rule,
    pub closed: Rule,
    pub payload_ref: String,
}

fn rule(text: &str) -> Rule {
    match text {
        "accept" => Rule::Accept,
        "accept_lift_or_close_only" => Rule::LiftOrCloseOnly,
        // Anything the loader does not know is a refusal: deny by default.
        _ => Rule::Refuse,
    }
}

/// The registry, loaded once from the contract fixture.
pub fn registry() -> &'static BTreeMap<String, KindRules> {
    static R: OnceLock<BTreeMap<String, KindRules>> = OnceLock::new();
    R.get_or_init(|| {
        let doc: Value = serde_json::from_str(REGISTRY).expect("the registry is JSON");
        doc["kinds"]
            .as_object()
            .expect("the registry has kinds")
            .iter()
            .map(|(name, k)| {
                let schema = k["payload_schema"]["schema"].as_str().unwrap_or("");
                let def = k["payload_schema"]["def"].as_str().unwrap_or("");
                (
                    name.clone(),
                    KindRules {
                        suspended: rule(k["suspended_rule"].as_str().unwrap_or("")),
                        closed: rule(k["closed_rule"].as_str().unwrap_or("")),
                        payload_ref: format!("{schema}#/$defs/{def}"),
                    },
                )
            })
            .collect()
    })
}

struct Docs(HashMap<String, Value>);

impl jsonschema::Retrieve for Docs {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("unknown schema {uri}").into())
    }
}

fn validator(entry: &str) -> jsonschema::Validator {
    let docs: HashMap<String, Value> = SCHEMAS
        .iter()
        .map(|text| {
            let doc: Value = serde_json::from_str(text).expect("a vendored schema is JSON");
            (doc["$id"].as_str().expect("schema $id").to_string(), doc)
        })
        .collect();
    jsonschema::options()
        .with_retriever(Docs(docs))
        .build(&json!({ "$ref": entry }))
        .expect("the vendored schemas compile")
}

fn check(v: &jsonschema::Validator, value: &Value) -> Result<(), String> {
    let errors: Vec<String> = v
        .iter_errors(value)
        .map(|e| format!("{}: {e}", e.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

/// Whether `document` is a valid `field-case.v1` document.
pub fn document_is_valid(document: &Value) -> Result<(), String> {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    check(
        V.get_or_init(|| validator("urn:sea-forge:fde:field-case:v1")),
        document,
    )
}

/// Whether `payload` satisfies the schema the registry names for `kind`.
pub fn payload_is_valid(kind: &str, payload: &Value) -> Result<(), String> {
    static V: OnceLock<Mutex<HashMap<String, std::sync::Arc<jsonschema::Validator>>>> =
        OnceLock::new();
    use std::sync::Mutex;
    let Some(rules) = registry().get(kind) else {
        return Err(format!("{kind} is not a registered kind"));
    };
    let cache = V.get_or_init(|| Mutex::new(HashMap::new()));
    let v = {
        let mut map = cache.lock().expect("validator cache");
        map.entry(kind.to_string())
            .or_insert_with(|| std::sync::Arc::new(validator(&rules.payload_ref)))
            .clone()
    };
    check(&v, payload)
}

/// Whether `approval` is a valid `assignment-binding.v1` `AssignmentApproval`
/// (what a Field Case delegate carries as `assignment`; WP-23). The five
/// currencies being each present exactly once is semantic and is checked by
/// the caller, as `assignment.v1` does for its own budget.
pub fn assignment_approval_is_valid(approval: &Value) -> Result<(), String> {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    check(
        V.get_or_init(|| {
            validator("urn:sea-forge:fde:assignment-binding:v1#/$defs/AssignmentApproval")
        }),
        approval,
    )
}

/// Whether `binding` is a valid `assignment-binding.v1` `Binding` (the versioned
/// record of the full approved assignment on the assignment row; WP-23).
pub fn assignment_binding_is_valid(binding: &Value) -> Result<(), String> {
    static V: OnceLock<jsonschema::Validator> = OnceLock::new();
    check(
        V.get_or_init(|| validator("urn:sea-forge:fde:assignment-binding:v1#/$defs/Binding")),
        binding,
    )
}
