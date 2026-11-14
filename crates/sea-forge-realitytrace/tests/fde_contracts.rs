//! WP-11 conformance for the FDE contracts under `contracts/fde/`: every valid
//! fixture validates, every invalid fixture fails at the pointer and for the
//! reason its manifest names, semantic rules a schema cannot state are checked
//! against their stated codes, and the DIGESTS manifest matches the schemas.
//! Schema validation only; no runtime behaviour is specified here.

use jsonschema::{Retrieve, Uri, Validator};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fde")
}

fn read(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path:?}: {e}")))
        .unwrap_or_else(|e| panic!("{path:?}: {e}"))
}

struct Schemas(HashMap<String, Value>);

impl Retrieve for Schemas {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("unknown schema {uri}").into())
    }
}

fn schemas() -> HashMap<String, Value> {
    ["frozen-reference", "anchor", "case-record-append"]
        .iter()
        .map(|n| {
            let schema = read(&dir().join(format!("{n}.v1.json")));
            (schema["$id"].as_str().unwrap().to_string(), schema)
        })
        .collect()
}

/// A validator for `$defs/<def>` of the schema `id`, or for the whole schema.
fn validator(id: &str, def: Option<&str>) -> Validator {
    let reference = match def {
        Some(d) => format!("{id}#/$defs/{d}"),
        None => id.to_string(),
    };
    jsonschema::options()
        .with_retriever(Schemas(schemas()))
        .build(&json!({ "$ref": reference }))
        .unwrap_or_else(|e| panic!("build {reference}: {e}"))
}

fn errors(v: &Validator, instance: &Value) -> Vec<(String, String)> {
    v.iter_errors(instance)
        .map(|e| (e.instance_path().to_string(), e.to_string()))
        .collect()
}

/// A deliberately small canonicaliser for the fixtures: objects, arrays,
/// ASCII strings, integers, booleans and null. Keys sort by UTF-16 code units,
/// which equals byte order only for ASCII. It REFUSES anything else
/// (non-ASCII text, floats) with an error instead of hashing it wrongly.
/// Full RFC 8785 conformance (number formatting, UTF-16 key order, escapes) is
/// tested in the CEP repository only (CEP-0008 section 16.5 vectors).
fn canonical(v: &Value) -> Result<String, String> {
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            let mut parts = Vec::new();
            for k in keys {
                if !k.is_ascii() {
                    return Err(format!("non-ASCII key {k:?}"));
                }
                parts.push(format!(
                    "{}:{}",
                    serde_json::to_string(k).unwrap(),
                    canonical(&m[k])?
                ));
            }
            Ok(format!("{{{}}}", parts.join(",")))
        }
        Value::Array(a) => {
            let parts: Result<Vec<String>, String> = a.iter().map(canonical).collect();
            Ok(format!("[{}]", parts?.join(",")))
        }
        Value::String(s) if s.is_ascii() => Ok(serde_json::to_string(s).unwrap()),
        Value::String(s) => Err(format!("non-ASCII string {s:?}")),
        Value::Number(n) if n.is_i64() || n.is_u64() => Ok(n.to_string()),
        Value::Number(n) => Err(format!("non-integer number {n}")),
        other => Ok(other.to_string()),
    }
}

fn body_digest(body: &Value) -> String {
    hex::encode(Sha256::digest(canonical(body).unwrap().as_bytes()))
}

/// episode_ref = episode:<case_id>:<plan_item_id>:<n>@sha256:<digest of body>.
fn check_episode_ref(doc: &Value) -> Result<(), &'static str> {
    let b = &doc["body"];
    let want = format!(
        "episode:{}:{}:{}@sha256:{}",
        b["case_id"].as_str().unwrap(),
        b["plan_item_id"].as_str().unwrap(),
        b["n"],
        body_digest(b)
    );
    if doc["episode_ref"] == want {
        Ok(())
    } else {
        Err("EPISODE_REF_DIGEST_MISMATCH")
    }
}

fn check_frozen_payload(p: &Value) -> Result<(), &'static str> {
    check_episode_ref(&p["reference"])?;
    let r = p["episode_ref"].as_str().unwrap();
    if p["reference"]["episode_ref"] != p["episode_ref"]
        || !r.ends_with(p["digest"].as_str().unwrap())
    {
        return Err("EPISODE_REF_DIGEST_MISMATCH");
    }
    Ok(())
}

fn check_anchor(a: &Value) -> Result<(), &'static str> {
    let tag = format!(
        "fde-anchor/{}/{}",
        a["kind"].as_str().unwrap(),
        a["id"].as_str().unwrap()
    );
    if a["tag"] != tag {
        return Err("ANCHOR_TAG_MISMATCH");
    }
    if a["verifier_receipt"]["digest"] != a["digest"] {
        return Err("ANCHOR_RECEIPT_MISMATCH");
    }
    Ok(())
}

fn semantic(contract: &str, instance: &Value) -> Result<(), &'static str> {
    match contract {
        "frozen-reference" => check_episode_ref(instance),
        "anchor" => check_anchor(instance),
        "case-record-append"
            if instance["kind"] == "episode.frozen" && instance["payload"].is_object() =>
        {
            check_frozen_payload(&instance["payload"])
        }
        _ => Ok(()),
    }
}

fn manifest(contract: &str) -> Value {
    read(&dir().join("fixtures").join(contract).join("manifest.json"))
}

fn fixture(contract: &str, sub: &str, file: &str) -> Value {
    read(&dir().join("fixtures").join(contract).join(sub).join(file))
}

fn def_of(entry: &Value) -> Option<&str> {
    entry["def"].as_str()
}

fn run_contract(contract: &str, min_valid: usize, min_invalid: usize) {
    let m = manifest(contract);
    let id = m["schema"].as_str().unwrap();

    let valid = m["valid"].as_array().unwrap();
    assert!(
        valid.len() >= min_valid,
        "{contract}: need {min_valid} valid fixtures"
    );
    for e in valid {
        let file = e["file"].as_str().unwrap();
        let inst = fixture(contract, "valid", file);
        let errs = errors(&validator(id, def_of(e)), &inst);
        assert!(errs.is_empty(), "{contract}/valid/{file}: {errs:?}");
        assert_eq!(semantic(contract, &inst), Ok(()), "{contract}/valid/{file}");
    }

    let invalid = m["invalid"].as_array().unwrap();
    let semantic_invalid = m["semantic_invalid"].as_array().unwrap();
    assert!(
        invalid.len() + semantic_invalid.len() >= min_invalid,
        "{contract}: need {min_invalid} invalid fixtures"
    );
    for e in invalid {
        let file = e["file"].as_str().unwrap();
        let inst = fixture(contract, "invalid", file);
        let errs = errors(&validator(id, def_of(e)), &inst);
        assert!(
            !errs.is_empty(),
            "{contract}/invalid/{file}: schema accepted it"
        );
        let (ptr, needle) = (
            e["pointer"].as_str().unwrap(),
            e["contains"].as_str().unwrap(),
        );
        assert!(
            errs.iter().any(|(p, msg)| p == ptr && msg.contains(needle)),
            "{contract}/invalid/{file}: wanted {ptr} ~ {needle:?}, got {errs:?}"
        );
    }
    for e in semantic_invalid {
        let file = e["file"].as_str().unwrap();
        let inst = fixture(contract, "invalid", file);
        let errs = errors(&validator(id, def_of(e)), &inst);
        assert!(
            errs.is_empty(),
            "{contract}/invalid/{file} is meant to pass the schema: {errs:?}"
        );
        assert_eq!(
            semantic(contract, &inst),
            Err(e["code"].as_str().unwrap()),
            "{contract}/invalid/{file}"
        );
    }

    let mut on_disk: Vec<String> = ["valid", "invalid"]
        .iter()
        .flat_map(|s| {
            std::fs::read_dir(dir().join("fixtures").join(contract).join(s))
                .unwrap()
                .map(move |f| format!("{s}/{}", f.unwrap().file_name().to_string_lossy()))
        })
        .collect();
    let mut listed: Vec<String> = valid
        .iter()
        .map(|e| format!("valid/{}", e["file"].as_str().unwrap()))
        .chain(
            invalid
                .iter()
                .chain(semantic_invalid)
                .map(|e| format!("invalid/{}", e["file"].as_str().unwrap())),
        )
        .collect();
    on_disk.sort();
    listed.sort();
    assert_eq!(on_disk, listed, "{contract}: manifest and files differ");
}

#[test]
fn frozen_reference_fixtures() {
    run_contract("frozen-reference", 2, 3);
}

#[test]
fn anchor_fixtures() {
    run_contract("anchor", 2, 3);
}

#[test]
fn case_record_append_fixtures() {
    run_contract("case-record-append", 2, 3);
}

#[test]
fn case_record_append_scenarios_name_the_refusal_they_exercise() {
    let m = manifest("case-record-append");
    let id = m["schema"].as_str().unwrap();
    let (request, response, error) = (
        validator(id, Some("Request")),
        validator(id, Some("Response")),
        validator(id, Some("Error")),
    );
    let mut seen: Vec<String> = vec![];
    for f in m["scenarios"].as_array().unwrap() {
        let s = fixture("case-record-append", "scenarios", f.as_str().unwrap());
        let name = s["id"].as_str().unwrap();
        assert_eq!(errors(&request, &s["request"]), vec![], "{name}: request");
        assert_eq!(
            semantic("case-record-append", &s["request"]),
            Ok(()),
            "{name}"
        );
        let expect = &s["expect"];
        let rule = s["rule"].as_str().unwrap();
        if expect["outcome"] == "refused" {
            assert_eq!(errors(&error, expect), vec![], "{name}: expect");
            if rule.starts_with("I-") {
                assert_eq!(expect["invariant"], rule, "{name}");
            }
            seen.push(expect["code"].as_str().unwrap().to_string());
        } else {
            assert_eq!(errors(&response, expect), vec![], "{name}: expect");
        }
        if rule == "I-3" && s["request"]["kind"] == "episode.frozen" {
            let r = s["request"]["payload"]["reference"]["body"].clone();
            let first = &s["ledger_before"][0];
            if expect["outcome"] == "refused" {
                assert_eq!(first["episode_key"]["case_id"], r["case_id"], "{name}");
                assert_eq!(first["episode_key"]["n"], r["n"], "{name}");
            } else {
                assert_ne!(first["episode_key"]["n"], r["n"], "{name}: new n");
            }
        }
    }
    for code in [
        "EXPLORATORY_NOT_SETTLEABLE",
        "FREEZE_KEY_EXISTS",
        "FREEZE_MISSING",
        "FREEZE_INCOMPLETE",
        "SELF_SETTLEMENT_REFUSED",
        "IDEMPOTENCY_CONFLICT",
    ] {
        assert!(
            seen.iter().any(|c| c == code),
            "no scenario exercises {code}"
        );
    }
}

#[test]
fn every_error_code_is_documented_in_the_readme() {
    let readme = std::fs::read_to_string(dir().join("README.md")).unwrap();
    let schema = &schemas()["urn:sea-forge:fde:case-record-append:v1"];
    for code in schema["$defs"]["Error"]["properties"]["code"]["enum"]
        .as_array()
        .unwrap()
    {
        let code = code.as_str().unwrap();
        assert!(readme.contains(&format!("`{code}`")), "README lacks {code}");
    }
}

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/check-fde-contract-digests.sh")
}

#[test]
fn digests_manifest_matches_the_schemas() {
    let manifest = std::fs::read_to_string(dir().join("DIGESTS")).unwrap();
    let mut listed = 0;
    for line in manifest.lines() {
        let (digest, name) = line.split_once("  ").unwrap();
        let bytes = std::fs::read(dir().join(name)).unwrap();
        assert_eq!(hex::encode(Sha256::digest(&bytes)), digest, "{name}");
        listed += 1;
    }
    assert_eq!(listed, 3);
    let out = Command::new("sh").arg(script()).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn digest_check_fails_when_a_schema_drifts() {
    let tmp = tempfile::tempdir().unwrap();
    for f in std::fs::read_dir(dir()).unwrap() {
        let f = f.unwrap();
        if f.file_type().unwrap().is_file() {
            std::fs::copy(f.path(), tmp.path().join(f.file_name())).unwrap();
        }
    }
    let target = tmp.path().join("anchor.v1.json");
    let mut text = std::fs::read_to_string(&target).unwrap();
    text.push(' ');
    std::fs::write(&target, text).unwrap();
    let out = Command::new("sh")
        .arg(script())
        .env("FDE_CONTRACTS_DIR", tmp.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("DIGEST MISMATCH anchor.v1.json"));
}

#[test]
fn the_test_canonicaliser_refuses_input_it_cannot_hash_faithfully() {
    assert!(canonical(&json!({"a": "caf\u{e9}"})).is_err());
    assert!(canonical(&json!({"caf\u{e9}": 1})).is_err());
    assert!(canonical(&json!({"a": 1.5})).is_err());
    assert!(canonical(&json!({"a": [1e30]})).is_err());
    assert_eq!(
        canonical(&json!({"b": [1, null, true], "a": "x"})).unwrap(),
        r#"{"a":"x","b":[1,null,true]}"#
    );
}
