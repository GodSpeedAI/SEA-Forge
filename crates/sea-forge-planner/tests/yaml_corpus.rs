//! Corpus proof for the YAML parser migration: `PlanTemplate` targets.
//!
//! Reads the shared fixtures in `sea-forge-authority/tests/fixtures/yaml-corpus` (see the README there)
//! and compares `sea_forge_core::yaml` with the SEA-Forge main ee66154 reference records for the
//! `PlanTemplate` target. The bundle target is covered by `sea-forge-authority/tests/yaml_corpus.rs`.

use std::collections::BTreeMap;
use std::fs;
use std::thread;

use sea_forge_planner::templates::PlanTemplate;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../sea-forge-authority/tests/fixtures/yaml-corpus"
);
const TARGET: &str = "PlanTemplate";

/// Adversarial cases the reference rejects and the new code accepts: (id, sibling id, why).
/// Must stay empty. B0634 and B0651 (`tRue`) were removed when the strict boundary
/// began to require the exact spellings `true` and `false` for booleans.
const NEWLY_ACCEPTED: &[(&str, &str, &str)] = &[];

/// Adversarial cases the reference accepts and the new code rejects: (id, first line of label).
const STRICTER: &[(&str, &str)] = &[
    ("B0584.yaml", "version = null"),
    ("B0624.yaml", "manual_activation = \"True\""),
    ("B0625.yaml", "manual_activation = \"TRUE\""),
    ("B0632.yaml", "manual_activation = \"FALSE\""),
    ("B0641.yaml", "markers.required = \"True\""),
    ("B0642.yaml", "markers.required = \"TRUE\""),
    ("B0649.yaml", "markers.required = \"FALSE\""),
    ("B0652.yaml", "name = \"~\""),
    ("B0655.yaml", "name = \"null\""),
    ("B0658.yaml", "name = \"Null\""),
    ("B0661.yaml", "name = \"NULL\""),
    ("B0664.yaml", "name = \"\""),
    ("B0666.yaml", "operations = \"\""),
    ("B0673.yaml", "plan item tagged !foo"),
    ("B0686.yaml", "BOM prefix"),
];

/// Typed `PlanTemplate` rows of REPORT.md's strict LOOSER and DIFFERENT tables: (table, id).
const REPORT_TYPED_ROWS: &[(&str, &str)] = &[
    ("LOOSER", "B0626"),
    ("LOOSER", "B0633"),
    ("LOOSER", "B0634"),
    ("LOOSER", "B0643"),
    ("LOOSER", "B0650"),
    ("LOOSER", "B0651"),
    ("LOOSER", "B0654"),
    ("LOOSER", "B0657"),
    ("LOOSER", "B0660"),
    ("LOOSER", "B0663"),
    ("DIFFERENT", "B0681"),
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

/// Record shape identical to the reference generator (class-only errors).
fn record(text: &str) -> Value {
    match sea_forge_core::yaml::from_str::<PlanTemplate>(text) {
        Ok(t) => json!({
            "accepted": true,
            "error_class": null,
            "identity": { "canonical_sha256": sha(&serde_json::to_value(&t).expect("json")) },
            "probes": [],
        }),
        Err(_) => {
            json!({ "accepted": false, "error_class": "parse", "identity": null, "probes": [] })
        }
    }
}

/// Runs on a 64 MiB stack, as the generator does, so deep nesting cannot abort the test binary.
fn on_big_stack(text: String) -> Value {
    thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || record(&text))
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
fn yaml_corpus_template_proof() {
    let reference = read_reference();
    let mut failures: Vec<String> = Vec::new();

    let mut corpus_of: BTreeMap<String, String> = BTreeMap::new();
    let mut input_of: BTreeMap<String, String> = BTreeMap::new();
    let mut new_recs: BTreeMap<String, Value> = BTreeMap::new();
    for (corpus, id, target) in read_targets() {
        if target != TARGET {
            continue;
        }
        let text = fs::read_to_string(format!("{FIXTURES}/{corpus}/{id}")).expect("fixture");
        new_recs.insert(id.clone(), on_big_stack(text.clone()));
        corpus_of.insert(id.clone(), corpus);
        input_of.insert(id, text);
    }
    let reference_count = reference.keys().filter(|(_, t)| t == TARGET).count();
    assert_eq!(
        reference_count,
        new_recs.len(),
        "every reference record for {TARGET} has a fixture"
    );

    let (mut real_n, mut real_acc) = (0usize, 0usize);
    let (mut both_rej, mut both_acc, mut newly, mut stricter_ids) =
        (0usize, 0usize, Vec::new(), Vec::new());
    let mut panics = 0usize;

    for (id, new) in &new_recs {
        let r = &reference[&(id.clone(), TARGET.to_string())];
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

    for id in &newly {
        let new = &new_recs[id];
        match NEWLY_ACCEPTED.iter().find(|(nid, _, _)| nid == id) {
            None => failures.push(format!(
                "UNEXPLAINED NEWLY ACCEPTED {}",
                describe(
                    id,
                    &input_of[id],
                    &reference[&(id.clone(), TARGET.to_string())],
                    new
                )
            )),
            Some((_, sibling, _)) => {
                match reference.get(&(sibling.to_string(), TARGET.to_string())) {
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

    stricter_ids.sort();
    let documented: Vec<String> = STRICTER.iter().map(|(id, _)| id.to_string()).collect();
    if stricter_ids != documented {
        failures.push(format!(
            "STRICTER constant drift. computed ({}): {:?}",
            stricter_ids.len(),
            stricter_ids
        ));
    }

    let (mut typed_rej, mut typed_eq, mut typed_loose, mut typed_diff) =
        (0usize, 0usize, 0usize, 0usize);
    for (table, report_id) in REPORT_TYPED_ROWS {
        let id = &format!("{report_id}.yaml");
        if *table == "LOOSER" {
            typed_loose += 1;
        } else {
            typed_diff += 1;
        }
        let Some(new) = new_recs.get(id) else {
            println!("row {table} {id}: skipped");
            continue;
        };
        let r = &reference[&(id.clone(), TARGET.to_string())];
        if new["accepted"] != true {
            typed_rej += 1;
            println!("row {table} {id}: rejected");
        } else if r["accepted"] == true && same_proof(r, new) {
            typed_eq += 1;
            println!("row {table} {id}: equal");
        } else {
            println!("row {table} {id}: UNRESOLVED");
            failures.push(format!(
                "REPORT typed row unresolved: {}",
                describe(id, &input_of[id], r, new)
            ));
        }
    }

    assert_eq!(panics, 0, "new code must not panic on any corpus document");
    println!(
        "summary {TARGET} real: documents={real_n} accepted={real_acc} rejected={} (accepted: identity equal reference)",
        real_n - real_acc
    );
    println!(
        "summary {TARGET} adversarial: both-rejected={both_rej} both-accepted={both_acc} newly-accepted={} stricter={}",
        newly.len(),
        stricter_ids.len()
    );
    println!("summary {TARGET} REPORT typed rows: LOOSER={typed_loose} DIFFERENT={typed_diff} rejected={typed_rej} equal={typed_eq}");
    for id in &stricter_ids {
        println!("stricter {id}: new error {}", new_recs[id]["error_class"]);
    }
    assert!(
        failures.is_empty(),
        "{} yaml corpus failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
