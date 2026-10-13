//! Corpus proof for the YAML parser migration (serde_yaml 0.9 -> serde-saphyr).
//!
//! `fixtures/yaml-corpus/reference.json` holds records produced by SEA-Forge
//! main ee66154 (serde_yaml 0.9.34) through the production loaders. This test
//! re-evaluates every bundle target with the new code and compares.
//! Templates and environment specs live in the planner and sandbox siblings.
//! See `fixtures/yaml-corpus/README.md` for regeneration.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::thread;

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction, IdentityBinding};
use sea_forge_core::ForgeError;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/yaml-corpus");
const BUNDLE: &str = "AuthorityPolicyBundle";
const WORKSPACE: &str = "/yaml-corpus/workspace";

/// Adversarial inputs larger than 64 KiB, not committed. Their reference
/// records are in `reference.json`; the test asserts exactly these are skipped.
const GENERATED_IN_TEST: &[&str] = &["B0509.yaml", "B0510.yaml", "B0511.yaml", "B0512.yaml"];

/// Adversarial bundle cases the reference rejects and the new code accepts:
/// (id, canonical reference-accepted sibling id, why). The new record must
/// equal the sibling's reference record in identity and probes.
const NEWLY_ACCEPTED: &[(&str, &str, &str)] = &[];

/// Adversarial bundle cases the reference accepts and the new code rejects:
/// (id, first line of label).
const STRICTER: &[(&str, &str)] = &[("B0169.yaml", "rules Vec = ~ (list replaced)")];

/// Adversarial bundle cases labelled `dup` in the harness (results.json class "dup"): duplicate keys.
const DUPLICATE_KEY_IDS: &[&str] = &[
    "B0000.yaml",
    "B0001.yaml",
    "B0002.yaml",
    "B0003.yaml",
    "B0004.yaml",
    "B0005.yaml",
    "B0006.yaml",
    "B0007.yaml",
    "B0008.yaml",
    "B0009.yaml",
    "B0010.yaml",
    "B0011.yaml",
    "B0012.yaml",
    "B0013.yaml",
    "B0014.yaml",
    "B0015.yaml",
    "B0016.yaml",
    "B0017.yaml",
    "B0018.yaml",
];

/// Typed AuthorityPolicyBundle rows of REPORT.md's strict LOOSER and DIFFERENT tables: (table, id).
const REPORT_TYPED_BUNDLE_ROWS: &[(&str, &str)] = &[
    ("LOOSER", "B0044"),
    ("LOOSER", "B0051"),
    ("LOOSER", "B0052"),
    ("LOOSER", "B0061"),
    ("LOOSER", "B0068"),
    ("LOOSER", "B0069"),
    ("LOOSER", "B0078"),
    ("LOOSER", "B0085"),
    ("LOOSER", "B0086"),
    ("LOOSER", "B0095"),
    ("LOOSER", "B0102"),
    ("LOOSER", "B0103"),
    ("LOOSER", "B0154"),
    ("LOOSER", "B0155"),
    ("LOOSER", "B0156"),
    ("LOOSER", "B0157"),
    ("LOOSER", "B0159"),
    ("LOOSER", "B0160"),
    ("LOOSER", "B0161"),
    ("LOOSER", "B0162"),
    ("LOOSER", "B0164"),
    ("LOOSER", "B0165"),
    ("LOOSER", "B0166"),
    ("LOOSER", "B0167"),
    ("LOOSER", "B0169"),
    ("LOOSER", "B0170"),
    ("LOOSER", "B0171"),
    ("LOOSER", "B0315"),
    ("LOOSER", "B0323"),
    ("LOOSER", "B0432"),
    ("DIFFERENT", "B0374"),
    ("DIFFERENT", "B0385"),
];

fn canon(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let sorted: BTreeMap<String, Value> =
                m.iter().map(|(k, v)| (k.clone(), canon(v))).collect();
            Value::Object(sorted.into_iter().collect::<Map<String, Value>>())
        }
        Value::Array(a) => Value::Array(a.iter().map(canon).collect()),
        other => other.clone(),
    }
}

fn sha(v: &Value) -> String {
    let text = serde_json::to_string(&canon(v)).expect("canonical json");
    let digest = Sha256::digest(text.as_bytes());
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

fn class(e: &ForgeError) -> String {
    match e {
        ForgeError::Config { class, .. } => format!("config:{class}"),
        ForgeError::Plan { class, .. } => format!("plan:{class}"),
        ForgeError::Input(_) => "input".into(),
        ForgeError::UnknownIntent(_) => "unknown_intent".into(),
        ForgeError::UnsafePath(_) => "unsafe_path".into(),
        ForgeError::Io { .. } => "io".into(),
        ForgeError::Serialization(_) => "serialization".into(),
        ForgeError::Internal(_) => "internal".into(),
        ForgeError::SelfModel(_) => "self_model".into(),
        ForgeError::Run { .. } => "run".into(),
    }
}

/// Same probe set as the reference generator (`yaml-diff/reference/src/main.rs`).
fn actions() -> Vec<(&'static str, AuthorityAction)> {
    vec![
        (
            "write_src_lib",
            AuthorityAction::WriteFile {
                path: "src/lib.rs".into(),
                content_hint: String::new(),
            },
        ),
        (
            "write_cargo_lock",
            AuthorityAction::WriteFile {
                path: "Cargo.lock".into(),
                content_hint: String::new(),
            },
        ),
        (
            "write_git_config",
            AuthorityAction::WriteFile {
                path: ".git/config".into(),
                content_hint: String::new(),
            },
        ),
        (
            "exec_sea_forge_status",
            AuthorityAction::ExecuteCommand {
                argv: vec!["sea-forge".into(), "status".into()],
                cwd: ".".into(),
            },
        ),
        (
            "exec_sh_id",
            AuthorityAction::ExecuteCommand {
                argv: vec!["/bin/sh".into(), "-c".into(), "id".into()],
                cwd: ".".into(),
            },
        ),
        (
            "external_example",
            AuthorityAction::ExternalApi {
                host: "example.com".into(),
            },
        ),
        (
            "git_commit_src_lib",
            AuthorityAction::GitCommit {
                paths: vec!["src/lib.rs".into()],
            },
        ),
        (
            "reserved_inspect_run",
            AuthorityAction::Reserved {
                resource_type: "inspect_run".into(),
                resource_id: "fixed".into(),
                parameters: json!({}),
            },
        ),
        (
            "reserved_unknown_action",
            AuthorityAction::Reserved {
                resource_type: "unknown_action".into(),
                resource_id: "fixed".into(),
                parameters: json!({}),
            },
        ),
    ]
}

fn roles() -> Vec<(&'static str, ActorRole, &'static str)> {
    vec![
        ("Operator", ActorRole::Operator, "operator_local"),
        ("Agent", ActorRole::Agent, "agent_local"),
    ]
}

fn reject(error_class: &str) -> Value {
    json!({ "accepted": false, "error_class": error_class, "identity": null, "probes": [] })
}

fn sorted_strings(mut v: Vec<String>) -> Value {
    v.sort();
    json!(v)
}

fn evaluate_one(
    engine: &PolicyAuthorityEngine,
    actor: &Actor,
    binding: &IdentityBinding,
    action: &AuthorityAction,
) -> Result<sea_forge_core::types::AuthorityDecision, ForgeError> {
    engine.evaluate(AuthorityEvaluation {
        actor,
        binding: binding.clone(),
        run_id: "run_yaml_corpus",
        case_id: "case_yaml_corpus",
        plan_item_id: "item_yaml_corpus",
        sequence: 1,
        action,
        workspace_root: Path::new(WORKSPACE),
        evidence_refs: vec!["intent:yaml_corpus".into()],
        artifacts_root: None,
        timeout_secs: None,
        env_keys: Default::default(),
        domainforge_candidate: None,
        environment: None,
    })
}

/// Record shape identical to the reference generator.
fn bundle_record(text: &str) -> Value {
    let dir = match tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR")) {
        Ok(d) => d,
        Err(_) => return reject("tempdir"),
    };
    let path = dir.path().join("policy.yaml");
    if fs::write(&path, text).is_err() {
        return reject("tempfile");
    }
    let bundle = match AuthorityPolicyBundle::load(&path) {
        Ok(b) => b,
        Err(e) => return reject(&class(&e)),
    };
    let canonical = sha(&serde_json::to_value(&bundle).expect("bundle to json"));
    let bindings: Vec<(&str, Actor, IdentityBinding)> = roles()
        .into_iter()
        .map(|(name, role, id)| {
            let binding = bundle.resolve_identity(id, role.clone());
            (
                name,
                Actor {
                    actor_id: id.into(),
                    role,
                },
                binding,
            )
        })
        .collect();
    let engine = match PolicyAuthorityEngine::new(bundle) {
        Ok(e) => e,
        Err(e) => return reject(&class(&e)),
    };

    let (_, actor, binding) = &bindings[0];
    let fixed = AuthorityAction::WriteFile {
        path: "src/lib.rs".into(),
        content_hint: String::new(),
    };
    let identity = match evaluate_one(&engine, actor, binding, &fixed) {
        Ok(d) => json!({
            "canonical_sha256": canonical,
            "determinism_policy_bundle_hash": d.determinism.policy_bundle_hash,
            "determinism_identity_binding_hash": d.determinism.identity_binding_hash,
            "policy_refs": sorted_strings(d.policy_refs.clone()),
            "identity_binding": serde_json::to_value(&d.identity_binding).expect("binding json"),
        }),
        Err(e) => json!({ "canonical_sha256": canonical, "identity_error": class(&e) }),
    };

    let mut probes = Vec::new();
    for (role_name, actor, binding) in &bindings {
        for (name, action) in actions() {
            let rec = match evaluate_one(&engine, actor, binding, &action) {
                Ok(d) => json!({
                    "action": name,
                    "role": role_name,
                    "result": "decision",
                    "outcome": serde_json::to_value(&d.outcome).expect("outcome"),
                    "verdict": serde_json::to_value(&d.verdict).expect("verdict"),
                    "normalized_disposition": serde_json::to_value(&d.normalized_disposition).expect("disposition"),
                    "matched_rule": d.matched_rule,
                    "sandbox_class_granted": d.sandbox_class_granted,
                    "reasons": sorted_strings(d.reason_codes.clone()),
                    "boundary_constraints": d.boundary_constraints,
                }),
                Err(e) => {
                    json!({ "action": name, "role": role_name, "result": "error", "error_class": class(&e) })
                }
            };
            probes.push(rec);
        }
    }
    json!({ "accepted": true, "error_class": null, "identity": identity, "probes": probes })
}

/// Runs on a 64 MiB stack, as the generator does, so deep nesting cannot abort the test binary.
fn on_big_stack(text: String) -> Value {
    thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || bundle_record(&text))
        .expect("spawn")
        .join()
        .unwrap_or_else(|_| json!({ "accepted": false, "error_class": "panic", "identity": null, "probes": [] }))
}

fn read_targets() -> Vec<(String, String, String)> {
    let text = fs::read_to_string(format!("{FIXTURES}/targets.tsv")).expect("targets.tsv");
    text.lines()
        .skip(1)
        .map(|l| {
            let p: Vec<&str> = l.split('\t').collect();
            (p[0].to_string(), p[1].to_string(), p[2].to_string())
        })
        .collect()
}

fn read_reference() -> BTreeMap<(String, String), Value> {
    let text = fs::read_to_string(format!("{FIXTURES}/reference.json")).expect("reference.json");
    let v: Value = serde_json::from_str(&text).expect("reference json");
    v.as_array()
        .expect("array")
        .iter()
        .map(|r| {
            (
                (
                    r["id"].as_str().unwrap().into(),
                    r["target"].as_str().unwrap().into(),
                ),
                r.clone(),
            )
        })
        .collect()
}

fn same_proof(a: &Value, b: &Value) -> bool {
    a["identity"] == b["identity"] && a["probes"] == b["probes"]
}

fn describe(id: &str, input: &str, reference: &Value, new: &Value) -> String {
    format!(
        "{id}\n  input:\n{input}\n  reference: {}\n  new: {}",
        serde_json::to_string(reference).unwrap(),
        serde_json::to_string(new).unwrap()
    )
}

#[test]
fn yaml_corpus_bundle_proof() {
    let reference = read_reference();
    let mut failures: Vec<String> = Vec::new();

    // New records for every committed bundle target, keyed by id.
    let mut corpus_of: BTreeMap<String, String> = BTreeMap::new();
    let mut input_of: BTreeMap<String, String> = BTreeMap::new();
    let mut new_recs: BTreeMap<String, Value> = BTreeMap::new();
    for (corpus, id, target) in read_targets() {
        if target != BUNDLE {
            continue;
        }
        let text = fs::read_to_string(format!("{FIXTURES}/{corpus}/{id}")).expect("fixture");
        let new = on_big_stack(text.clone());
        corpus_of.insert(id.clone(), corpus);
        input_of.insert(id.clone(), text);
        new_recs.insert(id, new);
    }

    // The skipped set must be exactly GENERATED_IN_TEST.
    let mut skipped: Vec<String> = reference
        .iter()
        .filter(|((id, t), _)| t == BUNDLE && !new_recs.contains_key(id))
        .map(|((id, _), _)| id.clone())
        .collect();
    skipped.sort();
    let mut expected_skip: Vec<String> = GENERATED_IN_TEST.iter().map(|s| s.to_string()).collect();
    expected_skip.sort();
    assert_eq!(
        skipped, expected_skip,
        "skipped bundle ids must equal GENERATED_IN_TEST"
    );
    assert_eq!(skipped.len(), 4, "explicit skip count");

    let (mut real_n, mut real_acc) = (0usize, 0usize);
    let (mut both_rej, mut both_acc, mut newly, mut stricter_ids) =
        (0usize, 0usize, Vec::new(), Vec::new());
    let mut panics = 0usize;

    for (id, new) in &new_recs {
        let r = reference
            .get(&(id.clone(), BUNDLE.to_string()))
            .expect("reference record");
        let input = &input_of[id];
        if new["error_class"] == "panic" {
            panics += 1;
            failures.push(describe(id, input, r, new));
        }
        let (r_acc, n_acc) = (r["accepted"] == true, new["accepted"] == true);
        if corpus_of[id] == "real" {
            real_n += 1;
            if n_acc {
                real_acc += 1;
            }
            if r_acc != n_acc || (n_acc && !same_proof(r, new)) {
                failures.push(format!("REAL {}", describe(id, input, r, new)));
            }
            continue;
        }
        match (r_acc, n_acc) {
            (false, false) => both_rej += 1,
            (false, true) => newly.push(id.clone()),
            (true, true) => {
                both_acc += 1;
                if !same_proof(r, new) {
                    failures.push(format!("BOTH-ACCEPT {}", describe(id, input, r, new)));
                }
            }
            (true, false) => stricter_ids.push(id.clone()),
        }
    }

    // (b) newly accepted: only through the explicit table, equal to the sibling's reference record.
    for id in &newly {
        let new = &new_recs[id];
        match NEWLY_ACCEPTED.iter().find(|(nid, _, _)| nid == id) {
            None => failures.push(format!(
                "UNEXPLAINED NEWLY ACCEPTED {}",
                describe(
                    id,
                    &input_of[id],
                    &reference[&(id.clone(), BUNDLE.to_string())],
                    new
                )
            )),
            Some((_, sibling, _)) => {
                let sib = reference.get(&(sibling.to_string(), BUNDLE.to_string()));
                match sib {
                    Some(s) if s["accepted"] == true && same_proof(s, new) => {}
                    _ => failures.push(format!(
                        "NEWLY ACCEPTED {id} differs from sibling {sibling}"
                    )),
                }
            }
        }
    }
    for (nid, _, _) in NEWLY_ACCEPTED {
        if !newly.iter().any(|i| i == nid) {
            failures.push(format!("stale NEWLY_ACCEPTED entry {nid}"));
        }
    }

    // (d) stricter: computed list must equal the documented constant.
    stricter_ids.sort();
    let documented: Vec<String> = STRICTER.iter().map(|(id, _)| id.to_string()).collect();
    if stricter_ids != documented {
        failures.push(format!(
            "STRICTER constant drift. computed ({}): {:?}",
            stricter_ids.len(),
            stricter_ids
        ));
    }
    // Duplicate-key cases are NOT in the stricter bucket: the reference already rejects them
    // (serde_yaml 0.9 errors on duplicate keys), so the premise "stricter contains duplicate keys"
    // does not hold. The check that holds is that both reject every duplicate-key case.
    for id in DUPLICATE_KEY_IDS {
        let ref_acc = reference
            .get(&(id.to_string(), BUNDLE.to_string()))
            .map(|r| r["accepted"] == true);
        let new_acc = new_recs.get(*id).map(|n| n["accepted"] == true);
        if ref_acc != Some(false) || new_acc != Some(false) {
            failures.push(format!(
                "duplicate-key case {id} accepted (reference {ref_acc:?}, new {new_acc:?})"
            ));
        }
    }

    // (e) REPORT typed rows: rejected, or accepted and equal to the reference.
    let (mut typed_rej, mut typed_eq, mut typed_loose, mut typed_diff) =
        (0usize, 0usize, 0usize, 0usize);
    let mut resolutions = Vec::new();
    for (table, report_id) in REPORT_TYPED_BUNDLE_ROWS {
        let id = &format!("{report_id}.yaml");
        if *table == "LOOSER" {
            typed_loose += 1;
        } else {
            typed_diff += 1;
        }
        let Some(new) = new_recs.get(id) else {
            resolutions.push(format!("{table} {id}: skipped"));
            continue;
        };
        let r = &reference[&(id.clone(), BUNDLE.to_string())];
        if new["accepted"] != true {
            typed_rej += 1;
            resolutions.push(format!("{table} {id}: rejected"));
        } else if r["accepted"] == true && same_proof(r, new) {
            typed_eq += 1;
            resolutions.push(format!("{table} {id}: equal"));
        } else {
            resolutions.push(format!("{table} {id}: UNRESOLVED"));
            failures.push(format!(
                "REPORT typed row unresolved: {}",
                describe(id, &input_of[id], r, new)
            ));
        }
    }
    for line in &resolutions {
        println!("row {line}");
    }

    assert_eq!(panics, 0, "new code must not panic on any corpus document");
    println!(
        "summary real: documents={real_n} accepted={real_acc} rejected={} (accepted: identity and probes equal reference)",
        real_n - real_acc
    );
    println!(
        "summary adversarial: both-rejected={both_rej} both-accepted={both_acc} newly-accepted={} stricter={} skipped-oversize={}",
        newly.len(),
        stricter_ids.len(),
        skipped.len()
    );
    println!(
        "summary REPORT typed bundle rows: LOOSER={typed_loose} DIFFERENT={typed_diff} rejected={typed_rej} equal={typed_eq}"
    );
    for id in &stricter_ids {
        println!("stricter {id}: new error {}", new_recs[id]["error_class"]);
    }
    println!("stricter ids: {}", stricter_ids.join(","));
    assert!(
        failures.is_empty(),
        "{} yaml corpus failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
