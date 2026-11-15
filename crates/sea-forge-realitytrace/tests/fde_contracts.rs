//! WP-11 and WP-20 conformance for the FDE contracts under `contracts/fde/`: every valid
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
    let mut out = HashMap::new();
    for f in std::fs::read_dir(dir()).unwrap() {
        let p = f.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if name.contains(".v") && name.ends_with(".json") {
            let schema = read(&p);
            out.insert(schema["$id"].as_str().unwrap().to_string(), schema);
        }
    }
    out
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
        "case-record-append-v2" => s2::append_v2(instance),
        "case-record-kinds" => s2::kinds(instance),
        "assignment" => s2::assignment(instance),
        "execution-episode-ref" => s2::exec_request(instance),
        "delegation-link" => s2::link(instance),
        "continuation" => s2::continuation(instance),
        "reserved-operation" => s2::reserved(instance),
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
    let mut n = 0;
    for (id, schema) in schemas() {
        let Some(codes) = schema["$defs"]["Error"]["properties"]["code"]["enum"].as_array() else {
            continue;
        };
        for code in codes {
            let code = code.as_str().unwrap();
            assert!(
                readme.contains(&format!("`{code}`")),
                "README lacks {code} ({id})"
            );
            n += 1;
        }
    }
    assert!(n >= 50, "expected every Error enum to be checked, saw {n}");
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
    assert_eq!(listed, 12);
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

// ---------------------------------------------------------------------------
// WP-20 (S2) semantic rules: what the S2 schemas cannot state. Each function
// returns the refusal code the README names, mirroring the rule tables; WP-21
// implements the same rules in the server.
// ---------------------------------------------------------------------------
mod s2 {
    use super::*;

    pub fn strings<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
        match v {
            Value::String(s) => out.push(s),
            Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
            Value::Object(m) => m.values().for_each(|x| strings(x, out)),
            _ => {}
        }
    }

    /// A RealityTrace result id (`observation|evidence|settlement|residual:` + 26 ULID characters)
    /// quoted inside a frozen reference body.
    pub fn quotes_result_id(s: &str) -> bool {
        ["observation:", "evidence:", "settlement:", "residual:"]
            .iter()
            .any(|p| {
                s.match_indices(p).any(|(i, _)| {
                    let rest = &s[i + p.len()..];
                    rest.len() >= 26
                        && rest[..26]
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                })
            })
    }

    pub fn append_v2(i: &Value) -> Result<(), &'static str> {
        let p = &i["payload"];
        if !p.is_object() {
            return Ok(());
        }
        match i["kind"].as_str() {
            Some("episode.frozen") => {
                check_frozen_payload(p)?;
                let mut all = vec![];
                strings(&p["reference"]["body"], &mut all);
                if all.iter().any(|s| quotes_result_id(s)) {
                    return Err("REFERENCE_DERIVED_FROM_RESULT");
                }
                Ok(())
            }
            Some("gate.assessed") => {
                let refs = p["evidence_refs"].as_array().unwrap();
                for (n, r) in refs.iter().enumerate() {
                    if refs[..n].contains(r) {
                        return Err("SCHEMA_INVALID");
                    }
                }
                let hard = ["allocate_authority", "integrate_govern"];
                if hard.contains(&p["gate"].as_str().unwrap())
                    && p["state"] == "not_applicable"
                    && p.get("approval_ref").is_none()
                {
                    return Err("HARD_GATE_NOT_WAIVABLE");
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn kinds(i: &Value) -> Result<(), &'static str> {
        if !i["kinds"].is_object() {
            return Ok(());
        }
        let required: [(&str, &[&str]); 6] = [
            (
                "episode.frozen",
                &["FREEZE_KEY_EXISTS", "KIND_IMMUTABLE", "WORLD_NOT_PINNED"],
            ),
            (
                "episode.settled",
                &[
                    "SELF_SETTLEMENT_REFUSED",
                    "SETTLEMENT_EXISTS",
                    "FREEZE_MISSING",
                ],
            ),
            ("field_case.revised", &["REVISION_CONFLICT"]),
            ("strategy.recorded", &["REVISION_CONFLICT"]),
            ("gate.assessed", &["HARD_GATE_NOT_WAIVABLE"]),
            ("disposition.recorded", &["AUTHORITY_DENIED"]),
        ];
        for (k, codes) in required {
            let have: Vec<&str> = i["kinds"][k]["refusal_codes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.as_str().unwrap())
                .collect();
            if codes.iter().any(|c| !have.contains(c)) {
                return Err("REGISTRY_INCOMPLETE");
            }
        }
        for e in i["kinds"].as_object().unwrap().values() {
            if e["scope"] == "case"
                && e["natural_key"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|f| ["plan_item_id", "n", "episode_ref"].contains(&f.as_str().unwrap()))
            {
                return Err("REGISTRY_NATURAL_KEY_INVALID");
            }
        }
        Ok(())
    }

    /// `episode:<case>:<item>:<n>@sha256:<digest>` -> (case, item, n)
    pub fn parse_episode_ref(s: &str) -> Option<(String, String, u64)> {
        let (head, _) = s.split_once('@')?;
        let parts: Vec<&str> = head.strip_prefix("episode:")?.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        Some((parts[0].into(), parts[1].into(), parts[2].parse().ok()?))
    }

    /// `assign:<case>:<step>:<n>:<k>` -> (case, step, n, k)
    pub fn parse_assignment(s: &str) -> Option<(String, String, u64, u64)> {
        let parts: Vec<&str> = s.strip_prefix("assign:")?.split(':').collect();
        if parts.len() != 4 {
            return None;
        }
        let (n, k): (u64, u64) = (parts[2].parse().ok()?, parts[3].parse().ok()?);
        (n >= 1 && k >= 1).then(|| (parts[0].into(), parts[1].into(), n, k))
    }

    fn currencies_unique(v: &Value, key: &str) -> bool {
        let cs: Vec<&str> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e[key].as_str().unwrap())
            .collect();
        cs.iter().enumerate().all(|(n, c)| !cs[..n].contains(c))
    }

    pub fn assignment(i: &Value) -> Result<(), &'static str> {
        let (id, ep) = (i["assignment_id"].as_str(), i["episode_ref"].as_str());
        let (Some(id), Some(ep)) = (id, ep) else {
            return Ok(());
        };
        let (Some((c, s, n, k)), Some((ec, es, en))) =
            (parse_assignment(id), parse_episode_ref(ep))
        else {
            return Err("ASSIGNMENT_ID_INVALID");
        };
        if (&c, &s, n) != (&ec, &es, en) {
            return Err("ASSIGNMENT_ID_INVALID");
        }
        if i.get("brief").is_some() {
            if i["case_id"] != c.as_str()
                || i["step_id"] != s.as_str()
                || i["n"] != n
                || i["k"] != k
            {
                return Err("ASSIGNMENT_ID_INVALID");
            }
            if !currencies_unique(&i["budget"], "currency") {
                return Err("ASSIGNMENT_BUDGET_INVALID");
            }
        } else if !currencies_unique(&i["spend"], "currency") {
            return Err("ASSIGNMENT_BUDGET_INVALID");
        }
        Ok(())
    }

    /// Lifecycle of one assignment instance: the id and episode_ref never change; the
    /// state machine is issued -> started -> (interrupted <-> resumed)* -> terminal.
    pub fn assignment_sequence(events: &[Value]) -> Result<(), &'static str> {
        let first = &events[0];
        let mut state: Option<&str> = None;
        for e in events {
            if e["assignment_id"] != first["assignment_id"] {
                return Err("ASSIGNMENT_ID_INVALID");
            }
            if e["episode_ref"] != first["episode_ref"] {
                return Err("ASSIGNMENT_EPISODE_CHANGED");
            }
            let next = e["event"].as_str().unwrap();
            let ok = matches!(
                (state, next),
                (None, "issued")
                    | (Some("issued"), "started")
                    | (
                        Some("started" | "resumed"),
                        "interrupted" | "completed" | "failed" | "cancelled"
                    )
                    | (Some("interrupted"), "resumed" | "cancelled")
            );
            if !ok {
                return Err("ASSIGNMENT_TRANSITION_INVALID");
            }
            state = Some(next);
        }
        Ok(())
    }

    /// The execution request rules a schema cannot state (case, step and n agree).
    pub fn exec_request(i: &Value) -> Result<(), &'static str> {
        let Some(verb) = i["verb"].as_str() else {
            return Ok(());
        };
        let epr = &i["episode_ref"];
        if !epr.is_object() {
            return Ok(());
        }
        match verb {
            "item_execute"
                if i["item_id"] != epr["plan_item_id"] || i["case_id"] != epr["case_id"] =>
            {
                Err("SCHEMA_INVALID")
            }
            "case_advance" if i["case_id"] != epr["case_id"] => Err("SCHEMA_INVALID"),
            "delegate" => {
                let Some(a) = i["assignment_id"].as_str() else {
                    return Err("SCHEMA_INVALID");
                };
                match parse_assignment(a) {
                    Some((c, s, n, _))
                        if epr["case_id"] == c.as_str()
                            && epr["plan_item_id"] == s.as_str()
                            && epr["n"] == n =>
                    {
                        Ok(())
                    }
                    _ => Err("ASSIGNMENT_ID_INVALID"),
                }
            }
            _ => Ok(()),
        }
    }

    /// The rule table of execution-episode-ref.v1 (README section 5) over one scenario.
    pub fn exec_gate(s: &Value) -> Result<(), &'static str> {
        let req = &s["request"];
        let verb = s["verb"].as_str().unwrap();
        let case = &s["case"];
        if case["template_ref"] != "fde.field_case@1" {
            return Ok(()); // R9
        }
        if case["suspended"] == true {
            return Err("CASE_SUSPENDED"); // R6
        }
        let executes = !(verb == "case_advance" && s["extra"]["advance_executes"] == false);
        if verb == "case_advance"
            && case["advance_reaches"] == "Completed"
            && !case["latest_case_disposition"]
                .as_str()
                .is_some_and(|d| d.starts_with("close-"))
        {
            return Err("CLOSE_DISPOSITION_REQUIRED"); // R11
        }
        if !executes {
            return Ok(());
        }
        let f = &s["freeze_state"];
        let parts = [
            f["realitytrace"] == true,
            f["case_record"] == true,
            f["anchor"] != "absent",
        ];
        if !req["episode_ref"].is_object() || parts.iter().all(|p| !p) {
            return Err("FREEZE_MISSING"); // R1, R2
        }
        if !parts.iter().all(|p| *p) {
            return Err("FREEZE_INCOMPLETE"); // R3
        }
        if f["anchor_publisher"] != "svc:case-keeper" {
            return Err("ANCHOR_PUBLISHER_INVALID"); // R4
        }
        if req["episode_ref"]["kind"]
            != f.get("kind")
                .unwrap_or(&json!("evaluated"))
                .as_str()
                .unwrap()
        {
            return Err("KIND_IMMUTABLE"); // R5
        }
        exec_request(req)?;
        if matches!(
            s["extra"]["authority_decision"].as_str(),
            Some("denied" | "absent")
        ) {
            return Err("AUTHORITY_DENIED"); // R7
        }
        if let Some(a) = req["assignment_id"].as_str() {
            let k = parse_assignment(a).unwrap().3;
            if s["extra"]["k_used"]
                .as_array()
                .is_some_and(|u| u.iter().any(|x| *x == k))
            {
                return Err("ASSIGNMENT_ID_INVALID"); // R10
            }
        }
        Ok(())
    }

    pub fn link(i: &Value) -> Result<(), &'static str> {
        if i.get("links").is_some() {
            return settlement(
                i["episode_ref"].as_str().unwrap(),
                i["settling_principal"].as_str().unwrap(),
                &i["links"],
            );
        }
        if i.get("delegator").is_none() {
            return Ok(());
        }
        let l = i;
        if l["expires_at"].as_str() <= l["issued_at"].as_str() || l["delegator"] == l["delegate"] {
            return Err("DELEGATION_LINK_INVALID");
        }
        match (
            parse_assignment(l["assignment_id"].as_str().unwrap()),
            parse_episode_ref(l["episode_ref"].as_str().unwrap()),
        ) {
            (Some((c, s, n, _)), Some((ec, es, en)))
                if (&c, &s, n) == (&ec, &es, en) && l["case_id"] == c.as_str() =>
            {
                Ok(())
            }
            _ => Err("DELEGATION_LINK_INVALID"),
        }
    }

    /// I-2 over the delegation links: refused when the settler executed, or delegated (directly or
    /// through a chain of links) anyone who executed.
    pub fn settlement(episode_ref: &str, settler: &str, links: &Value) -> Result<(), &'static str> {
        let links = links.as_array().unwrap();
        for l in links {
            link(l)?;
            if l["episode_ref"] != episode_ref {
                return Err("DELEGATION_LINK_INVALID");
            }
        }
        let mut closure: Vec<&str> = links
            .iter()
            .map(|l| l["delegate"].as_str().unwrap())
            .collect();
        loop {
            let before = closure.len();
            for l in links {
                if closure.contains(&l["delegate"].as_str().unwrap())
                    && !closure.contains(&l["delegator"].as_str().unwrap())
                {
                    closure.push(l["delegator"].as_str().unwrap());
                }
            }
            if closure.len() == before {
                break;
            }
        }
        if closure.contains(&settler) {
            Err("SELF_SETTLEMENT_REFUSED")
        } else {
            Ok(())
        }
    }

    pub fn continuation(i: &Value) -> Result<(), &'static str> {
        if let Some(c) = i.get("continuation") {
            return continuation(c);
        }
        if i.get("document").is_some() {
            return if i["document"]["revision"] == i["revision"]
                && i["document"]["case_id"] == i["case_id"]
            {
                Ok(())
            } else {
                Err("CONTINUATION_INVALID")
            };
        }
        if i.get("gate_evidence").is_none() {
            return Ok(());
        }
        let order = [
            "choose_problem",
            "discover_reality",
            "allocate_authority",
            "prove_value",
            "prove_reliability",
            "integrate_govern",
            "earn_trust",
            "measure",
            "productize",
        ];
        let gates = i["gate_evidence"].as_array().unwrap();
        if gates
            .iter()
            .map(|g| g["gate"].as_str().unwrap())
            .collect::<Vec<_>>()
            != order
        {
            return Err("CONTINUATION_INVALID");
        }
        if gates
            .iter()
            .any(|g| g["state"] == "met" && g["evidence_refs"].as_array().unwrap().is_empty())
        {
            return Err("CONTINUATION_INVALID");
        }
        if !currencies_unique(&i["work"]["remaining_budgets"], "currency") {
            return Err("CONTINUATION_INVALID");
        }
        if i["case"]["lifecycle"] == "closed"
            && !i["work"]["active_assignments"]
                .as_array()
                .unwrap()
                .is_empty()
        {
            return Err("CONTINUATION_INVALID");
        }
        Ok(())
    }

    pub fn operation_digest(canonical_operation: &Value) -> String {
        body_digest(canonical_operation)
    }

    pub fn reserved(i: &Value) -> Result<(), &'static str> {
        if i.get("canonical_operation").is_some() {
            if operation_digest(&i["canonical_operation"])
                != i["operation_digest"].as_str().unwrap()
                || i["canonical_operation"]["operation"] != i["operation"]
            {
                return Err("RESERVED_DIGEST_MISMATCH");
            }
            if i["expires_at"].as_str() <= i["issued_at"].as_str() {
                return Err("RESERVED_EXPIRED");
            }
        }
        if i.get("binds_digest").is_some() {
            let epr = parse_episode_ref(i["episode_ref"].as_str().unwrap()).unwrap();
            let want = body_digest(&json!({
                "episode_ref": i["episode_ref"], "patch_sha256": i["patch_sha256"], "command": i["command"]
            }));
            if want != i["binds_digest"].as_str().unwrap()
                || epr.0 != i["case_id"].as_str().unwrap()
            {
                return Err("RESERVED_DIGEST_MISMATCH");
            }
            if i["expires_at"].as_str() <= i["decided_at"].as_str() {
                return Err("RESERVED_EXPIRED");
            }
        }
        if i.get("operations").is_some() {
            let ops: Vec<&str> = i["operations"]
                .as_array()
                .unwrap()
                .iter()
                .map(|o| o["operation"].as_str().unwrap())
                .collect();
            if ops.iter().enumerate().any(|(n, o)| ops[..n].contains(o)) {
                return Err("RESERVED_TABLE_INVALID");
            }
        }
        Ok(())
    }

    /// The WP-31 reserved-decision rules over one scenario: what a conforming implementation answers.
    pub fn decide(s: &Value) -> Result<String, &'static str> {
        let (p, sub, st) = (&s["pending"], &s["submission"], &s["state"]);
        if st["enforcement"] == "off" {
            return Ok("session".into());
        }
        if st["a3_channel"] == "unavailable" {
            return Err("A3_CHANNEL_UNAVAILABLE");
        }
        let Some(proof) = sub.get("proof") else {
            return Err("RESERVED_DECISION_REFUSED");
        };
        if st["now"].as_str() > p["expires_at"].as_str() {
            return Err("RESERVED_EXPIRED");
        }
        if proof["pending_id"] != p["pending_id"]
            || proof["operation_digest"] != p["operation_digest"]
        {
            return Err("RESERVED_DIGEST_MISMATCH");
        }
        let class = st["surfaces"][proof["confirmation_surface_id"].as_str().unwrap()]
            .as_str()
            .unwrap_or("none");
        if class == "none" || (class == "A3-T" && (p["class"] == "P3" || st["a3_t_policy"] != true))
        {
            return Err("A3_SURFACE_REQUIRED");
        }
        if st["challenges_used"]
            .as_array()
            .is_some_and(|u| u.contains(&proof["challenge"]))
        {
            return Err("A3_PROOF_REPLAYED");
        }
        if sub["assertion_verifies"] == false {
            return Err("A3_PROOF_INVALID");
        }
        if st["approval"] == "denied" {
            return Err("AUTHORITY_DENIED");
        }
        Ok(class.into())
    }
}

fn s2_manifest(contract: &str) -> Value {
    manifest(contract)
}

fn files(contract: &str, key: &str, sub: &str) -> Vec<(String, Value)> {
    s2_manifest(contract)[key]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|f| {
            (
                f.as_str().unwrap().to_string(),
                fixture(contract, sub, f.as_str().unwrap()),
            )
        })
        .collect()
}

#[test]
fn field_case_fixtures() {
    run_contract("field-case", 3, 15);
}

#[test]
fn case_record_append_v2_fixtures() {
    run_contract("case-record-append-v2", 20, 25);
}

#[test]
fn case_record_kinds_fixtures() {
    run_contract("case-record-kinds", 1, 10);
}

#[test]
fn assignment_fixtures() {
    run_contract("assignment", 5, 15);
}

#[test]
fn execution_episode_ref_fixtures() {
    run_contract("execution-episode-ref", 10, 15);
}

#[test]
fn delegation_link_fixtures() {
    run_contract("delegation-link", 4, 10);
}

#[test]
fn continuation_fixtures() {
    run_contract("continuation", 8, 15);
}

#[test]
fn reserved_operation_fixtures() {
    run_contract("reserved-operation", 10, 25);
}

#[test]
fn append_v2_scenarios_validate_and_exercise_every_v2_code() {
    let m = manifest("case-record-append-v2");
    let (id, v1id) = (
        m["schema"].as_str().unwrap(),
        "urn:sea-forge:fde:case-record-append:v1",
    );
    let mut seen: Vec<String> = vec![];
    for (name, s) in files("case-record-append-v2", "scenarios", "scenarios") {
        let v1 = s["dialect"] == "v1";
        let sid = if v1 { v1id } else { id };
        assert_eq!(
            errors(&validator(sid, Some("Request")), &s["request"]),
            vec![],
            "{name}: request"
        );
        let expect = &s["expect"];
        if expect["outcome"] == "refused" {
            assert_eq!(
                errors(&validator(sid, Some("Error")), expect),
                vec![],
                "{name}: expect"
            );
            let code = expect["code"].as_str().unwrap();
            if !v1 {
                seen.push(code.to_string());
            }
            // The semantic rules a scenario's request can already show.
            let sem = semantic("case-record-append-v2", &s["request"]);
            if ["REFERENCE_DERIVED_FROM_RESULT", "HARD_GATE_NOT_WAIVABLE"].contains(&code) {
                assert_eq!(sem, Err(code), "{name}");
            } else {
                assert_eq!(sem, Ok(()), "{name}");
            }
            if code == "PAYLOAD_TOO_LARGE" {
                assert!(
                    s["transport"]["encoded_bytes"].as_u64().unwrap() > 1_048_576,
                    "{name}"
                );
            }
            if v1 {
                assert!(
                    s["expect"]["message"].as_str().unwrap().contains(':')
                        || code == "FREEZE_KEY_EXISTS",
                    "{name}: a v1 fallback carries the v2 code in its message"
                );
            }
        } else {
            assert_eq!(
                errors(&validator(sid, Some("Response")), expect),
                vec![],
                "{name}: expect"
            );
        }
    }
    let v2_only = [
        "WORLD_NOT_PINNED",
        "SETTLEMENT_EXISTS",
        "KIND_IMMUTABLE",
        "REFERENCE_DERIVED_FROM_RESULT",
        "MODEL_ANCHOR_LATE",
        "AUTHORITY_DENIED",
        "ANCHOR_PUBLISHER_INVALID",
        "FREEZE_INCOMPLETE",
        "STORE_INTEGRITY",
        "PAYLOAD_TOO_LARGE",
        "HARD_GATE_NOT_WAIVABLE",
        "CASE_SUSPENDED",
        "REVISION_CONFLICT",
        "SELF_SETTLEMENT_REFUSED",
        "FREEZE_MISSING",
        "FREEZE_KEY_EXISTS",
    ];
    for code in v2_only {
        assert!(
            seen.iter().any(|c| c == code),
            "no v2 scenario exercises {code}"
        );
    }
}

#[test]
fn v2_error_enum_is_the_v1_enum_plus_the_new_codes() {
    let all = schemas();
    let enum_of = |id: &str| -> Vec<String> {
        all[id]["$defs"]["Error"]["properties"]["code"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap().to_string())
            .collect()
    };
    let (v1, v2) = (
        enum_of("urn:sea-forge:fde:case-record-append:v1"),
        enum_of("urn:sea-forge:fde:case-record-append:v2"),
    );
    assert!(
        v1.iter().all(|c| v2.contains(c)),
        "v2 must keep every v1 code"
    );
    assert_eq!(
        &v2[..v1.len()],
        &v1[..],
        "v1 codes keep their order at the head"
    );
    assert_eq!(v2.len() - v1.len(), 12);
}

#[test]
fn the_registry_matches_the_v2_request_and_error_schemas() {
    let all = schemas();
    let reg = fixture("case-record-kinds", "valid", "registry.json");
    let v2 = &all["urn:sea-forge:fde:case-record-append:v2"];
    let kinds: Vec<&str> = v2["$defs"]["Request"]["properties"]["kind"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k.as_str().unwrap())
        .collect();
    let mut have: Vec<&str> = reg["kinds"]
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    have.sort();
    let mut want = kinds.clone();
    want.sort();
    assert_eq!(have, want);
    let codes = v2["$defs"]["Error"]["properties"]["code"]["enum"]
        .as_array()
        .unwrap();
    for (k, e) in reg["kinds"].as_object().unwrap() {
        let target = &all[e["payload_schema"]["schema"].as_str().unwrap()]["$defs"]
            [e["payload_schema"]["def"].as_str().unwrap()];
        assert!(target.is_object(), "{k}: payload schema does not resolve");
        // each kind's Request branch references exactly this payload def
        let reference = format!(
            "{}#/$defs/{}",
            e["payload_schema"]["schema"].as_str().unwrap(),
            e["payload_schema"]["def"].as_str().unwrap()
        );
        let local = format!("#/$defs/{}", e["payload_schema"]["def"].as_str().unwrap());
        let branch = v2["$defs"]["Request"]["allOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["if"]["properties"]["kind"]["const"] == k.as_str())
            .unwrap_or_else(|| panic!("{k}: no Request branch"));
        let r = branch["then"]["properties"]["payload"]["$ref"]
            .as_str()
            .unwrap();
        assert!(
            r == reference || r == local,
            "{k}: registry says {reference}, Request says {r}"
        );
        for c in e["refusal_codes"].as_array().unwrap() {
            assert!(codes.contains(c), "{k}: {c} is not in the v2 Error enum");
        }
        let introduced_v1 = e["introduced"] == "v1";
        assert_eq!(
            introduced_v1,
            e["payload_schema"]["schema"] == "urn:sea-forge:fde:case-record-append:v1",
            "{k}"
        );
    }
}

#[test]
fn field_case_paths_are_the_revision_paths() {
    // run again the script check, which compares the schema with SOP 4.2 and the model
    let out = Command::new("sh")
        .arg(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../scripts/check-fde-field-case-schema.sh"),
        )
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn oracle_codes_are_all_defined_exactly_once() {
    let out = Command::new("sh")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/check-fde-oracle-codes.sh"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn the_oracle_check_fails_when_a_code_goes_missing() {
    let tmp = tempfile::tempdir().unwrap();
    for f in std::fs::read_dir(dir()).unwrap() {
        let f = f.unwrap();
        if f.file_type().unwrap().is_file() {
            std::fs::copy(f.path(), tmp.path().join(f.file_name())).unwrap();
        }
    }
    let target = tmp.path().join("case-record-append.v2.json");
    let text = std::fs::read_to_string(&target)
        .unwrap()
        .replace("\"ANCHOR_PUBLISHER_INVALID\",", "");
    std::fs::write(&target, text).unwrap();
    let out = Command::new("sh")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/check-fde-oracle-codes.sh"))
        .env("FDE_CONTRACTS_DIR", tmp.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("ANCHOR_PUBLISHER_INVALID"));
}

#[test]
fn assignment_lifecycles_follow_the_rules() {
    let mut codes = vec![];
    for (name, s) in files("assignment", "scenarios", "scenarios") {
        let events = s["events"].as_array().unwrap();
        for e in events {
            assert_eq!(
                errors(
                    &validator("urn:sea-forge:fde:assignment:v1", Some("Event")),
                    e
                ),
                vec![],
                "{name}"
            );
        }
        let got = s2::assignment_sequence(events);
        match s["expect"]["outcome"].as_str().unwrap() {
            "ok" => assert_eq!(got, Ok(()), "{name}"),
            _ => {
                assert_eq!(got, Err(s["expect"]["code"].as_str().unwrap()), "{name}");
                codes.push(s["expect"]["code"].as_str().unwrap().to_string());
            }
        }
    }
    for c in [
        "ASSIGNMENT_ID_INVALID",
        "ASSIGNMENT_EPISODE_CHANGED",
        "ASSIGNMENT_TRANSITION_INVALID",
    ] {
        assert!(
            codes.iter().any(|x| x == c),
            "no lifecycle scenario for {c}"
        );
    }
}

#[test]
fn the_execution_gate_rule_table_gives_each_scenario_its_answer() {
    let id = "urn:sea-forge:fde:execution-episode-ref:v1";
    let mut rules = std::collections::BTreeSet::new();
    let mut verbs_refused = std::collections::BTreeSet::new();
    for (name, s) in files("execution-episode-ref", "scenarios", "scenarios") {
        let def = match s["verb"].as_str().unwrap() {
            "item_execute" => "ItemExecuteRequest",
            "case_advance" => "CaseAdvanceRequest",
            _ => "DelegateRequest",
        };
        assert_eq!(
            errors(&validator(id, Some(def)), &s["request"]),
            vec![],
            "{name}: request"
        );
        let got = s2::exec_gate(&s);
        if s["expect"]["outcome"] == "accepted" {
            assert_eq!(got, Ok(()), "{name}");
        } else {
            assert_eq!(
                errors(&validator(id, Some("Error")), &s["expect"]),
                vec![],
                "{name}: expect"
            );
            assert_eq!(got, Err(s["expect"]["code"].as_str().unwrap()), "{name}");
            verbs_refused.insert(format!(
                "{}:{}",
                s["verb"].as_str().unwrap(),
                s["expect"]["code"].as_str().unwrap()
            ));
        }
        rules.insert(s["rule"].as_str().unwrap().to_string());
    }
    for r in (1..=11).map(|n| format!("R{n}")) {
        assert!(rules.contains(&r), "no scenario for rule {r}");
    }
    for verb in ["item_execute", "case_advance", "delegate"] {
        for code in [
            "FREEZE_MISSING",
            "FREEZE_INCOMPLETE",
            "KIND_IMMUTABLE",
            "CASE_SUSPENDED",
            "AUTHORITY_DENIED",
            "ANCHOR_PUBLISHER_INVALID",
        ] {
            assert!(
                verbs_refused.contains(&format!("{verb}:{code}")),
                "{verb} has no {code} scenario"
            );
        }
    }
}

#[test]
fn execution_frames_cover_the_three_verbs_with_and_without_the_field() {
    let id = "urn:sea-forge:fde:execution-episode-ref:v1";
    let mut combos = std::collections::BTreeSet::new();
    for (name, f) in files("execution-episode-ref", "frames", "frames") {
        assert_eq!(
            f["status"], "expected",
            "{name}: goldens are captured by WP-21"
        );
        let verb = f["verb"].as_str().unwrap();
        let def = match verb {
            "item_execute" => "ItemExecuteRequest",
            "case_advance" => "CaseAdvanceRequest",
            _ => "DelegateRequest",
        };
        assert_eq!(
            errors(&validator(id, Some(def)), &f["request"]),
            vec![],
            "{name}"
        );
        if let Some(r) = f["expect"].get("refused") {
            assert_eq!(errors(&validator(id, Some("Error")), r), vec![], "{name}");
        }
        combos.insert(format!(
            "{verb}:{}",
            f["request"].get("episode_ref").is_some()
        ));
    }
    for verb in ["item_execute", "case_advance", "delegate"] {
        for with in [true, false] {
            assert!(
                combos.contains(&format!("{verb}:{with}")),
                "{verb} with={with}"
            );
        }
    }
}

#[test]
fn read_verb_frames_validate() {
    let id = "urn:sea-forge:fde:continuation:v1";
    let mut verbs = vec![];
    for (name, f) in files("continuation", "frames", "frames") {
        assert_eq!(f["status"], "expected", "{name}");
        let req = if f["verb"] == "case_field_get" {
            "FieldGetRequest"
        } else {
            "ContinuationRequest"
        };
        assert_eq!(
            errors(&validator(id, Some(req)), &f["request"]),
            vec![],
            "{name}"
        );
        let def = f["expect"]["response_def"].as_str().unwrap();
        assert_eq!(
            errors(&validator(id, Some(def)), &f["expect"]["response"]),
            vec![],
            "{name}"
        );
        assert_eq!(
            semantic("continuation", &f["expect"]["response"]),
            Ok(()),
            "{name}"
        );
        verbs.push(f["verb"].as_str().unwrap().to_string());
    }
    assert!(
        verbs.iter().any(|v| v == "case_field_get")
            && verbs.iter().any(|v| v == "case_continuation")
    );
}

#[test]
fn delegation_link_scenarios_follow_i2() {
    for (name, s) in files("delegation-link", "scenarios", "scenarios") {
        let ep = s["links"][0]["episode_ref"].as_str().unwrap();
        let got = s2::settlement(ep, s["settling_principal"].as_str().unwrap(), &s["links"]);
        match s["expect"]["outcome"].as_str().unwrap() {
            "ok" => assert_eq!(got, Ok(()), "{name}"),
            _ => {
                assert_eq!(got, Err("SELF_SETTLEMENT_REFUSED"), "{name}");
                assert_eq!(
                    errors(
                        &validator("urn:sea-forge:fde:delegation-link:v1", Some("Error")),
                        &json!({
                    "outcome": "refused", "code": s["expect"]["code"], "message": "m", "invariant": s["expect"]["invariant"]})
                    ),
                    vec![],
                    "{name}"
                );
            }
        }
    }
}

#[test]
fn reserved_decision_scenarios_follow_the_rules() {
    let id = "urn:sea-forge:fde:reserved-operation:v1";
    let mut seen = vec![];
    for (name, s) in files("reserved-operation", "scenarios", "scenarios") {
        assert_eq!(
            errors(&validator(id, Some("PendingOperation")), &s["pending"]),
            vec![],
            "{name}"
        );
        assert_eq!(
            semantic("reserved-operation", &s["pending"]),
            Ok(()),
            "{name}"
        );
        if let Some(p) = s["submission"].get("proof") {
            assert_eq!(errors(&validator(id, Some("A3Proof")), p), vec![], "{name}");
        }
        let got = s2::decide(&s);
        if s["expect"]["outcome"] == "confirmed" {
            assert_eq!(
                errors(&validator(id, Some("Decision")), &s["expect"]),
                vec![],
                "{name}"
            );
            assert_eq!(
                got,
                Ok(s["expect"]["assurance"].as_str().unwrap().to_string()),
                "{name}"
            );
        } else {
            assert_eq!(
                errors(&validator(id, Some("Error")), &s["expect"]),
                vec![],
                "{name}"
            );
            let code = s["expect"]["code"].as_str().unwrap();
            assert_eq!(got, Err(code), "{name}");
            seen.push(code.to_string());
        }
    }
    for c in [
        "RESERVED_DECISION_REFUSED",
        "A3_SURFACE_REQUIRED",
        "A3_CHANNEL_UNAVAILABLE",
        "RESERVED_EXPIRED",
        "RESERVED_DIGEST_MISMATCH",
        "A3_PROOF_INVALID",
        "A3_PROOF_REPLAYED",
        "AUTHORITY_DENIED",
    ] {
        assert!(seen.iter().any(|x| x == c), "no reserved scenario for {c}");
    }
}

#[test]
fn the_program_execution_decision_binds_episode_patch_and_command() {
    let d = fixture(
        "reserved-operation",
        "valid",
        "program-execution-decision-allow.json",
    );
    assert_eq!(semantic("reserved-operation", &d), Ok(()));
    for (field, other) in [
        ("patch_sha256", json!("cd".repeat(32))),
        ("command", json!({"argv": ["x"]})),
    ] {
        let mut m = d.clone();
        m[field] = other;
        assert_eq!(
            semantic("reserved-operation", &m),
            Err("RESERVED_DIGEST_MISMATCH"),
            "{field}"
        );
    }
}

#[test]
fn independent_observation_events_come_from_sea_forge_emitters() {
    let m = manifest("independent-observation");
    let decision = fixture(
        "reserved-operation",
        "valid",
        "program-execution-decision-allow.json",
    );
    let (mut independent, mut executor_only) = (0, 0);
    for e in m["events"].as_array().unwrap() {
        let ev = read(
            &dir()
                .join("fixtures/independent-observation")
                .join(e["file"].as_str().unwrap()),
        );
        for key in [
            "world_ref",
            "semantic_ref",
            "declaration_key",
            "actor",
            "operation",
            "resource",
            "context",
            "correlation",
            "authority",
            "outcome",
            "source",
            "executor_provenance",
            "kind",
            "expectation_refs",
        ] {
            assert!(ev.get(key).is_some(), "{}: missing {key}", e["file"]);
        }
        assert_eq!(
            ev["authority"]["decision_id"], decision["decision_id"],
            "{}",
            e["file"]
        );
        assert_eq!(ev["context"]["episode_ref"], decision["episode_ref"]);
        let system = ev["source"]["system"].as_str().unwrap();
        assert_eq!(system, e["emitter"].as_str().unwrap());
        if e["independent_of_executor"] == true {
            assert!(system.starts_with("sea-forge-"), "{system}");
            assert_ne!(system, m["executor_emitter"].as_str().unwrap());
            independent += 1;
        } else {
            assert_eq!(system, m["executor_emitter"].as_str().unwrap());
            executor_only += 1;
        }
    }
    assert!(independent >= 2 && executor_only == 1);
}
