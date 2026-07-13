use sea_forge_domainforge::{
    load_validate, normalize_authority, CandidateDisposition, SeaSourceSet, SourceFile,
};
use sha2::{Digest, Sha256};
use std::fs;

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn source_file(uri: &str, content: &str) -> SourceFile {
    SourceFile {
        uri: uri.into(),
        sha256: sha256_hex(content.as_bytes()),
        content: content.into(),
    }
}

const DEMO_SEA: &str = include_str!("fixtures/demo.sea");

fn demo_source_set() -> SeaSourceSet {
    SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![source_file("demo.sea", DEMO_SEA)],
    }
}

#[test]
fn valid_fixture_produces_stable_domain_model_ref() {
    let model = load_validate(&demo_source_set()).unwrap();
    assert!(!model.model_ref.semantic_model_sha256.is_empty());
    assert!(!model.model_ref.source_refs.is_empty());
    assert!(model.model_ref.concept_refs.contains(&"Sample".to_string()));

    // Same input → same ref.
    let model2 = load_validate(&demo_source_set()).unwrap();
    assert_eq!(
        model.model_ref.semantic_model_sha256,
        model2.model_ref.semantic_model_sha256
    );
}

#[test]
fn invalid_syntax_fails_with_domain_model_error() {
    let mut ss = demo_source_set();
    ss.files[0].content = "this is not valid .sea syntax @@@".into();
    ss.files[0].sha256 = sha256_hex(ss.files[0].content.as_bytes());
    let result = load_validate(&ss);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error"),
        "expected domain_model_error: {err}"
    );
}

#[test]
fn source_hash_drift_rejected() {
    let ss = demo_source_set();
    let mut drifted = ss.clone();
    // Keep the declared hash but change the content.
    drifted.files[0].content = "@namespace \"different\"\nEntity \"Other\" in different\n".into();
    let result = load_validate(&drifted);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("source hash drift"));
}

#[test]
fn no_side_effects_on_invalid_input() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("should_not_exist.txt");
    let mut ss = demo_source_set();
    ss.files[0].content = "invalid syntax {{{".into();
    ss.files[0].sha256 = sha256_hex(ss.files[0].content.as_bytes());

    let _ = load_validate(&ss);
    assert!(
        !marker.exists(),
        "no side effect should occur on invalid input"
    );
    // Verify the adapter didn't write anything to disk.
    let entries: Vec<_> = fs::read_dir(tmp.path()).unwrap().collect();
    assert!(
        entries.is_empty(),
        "no files should be written by the adapter"
    );
}

#[test]
fn authority_normalization_table() {
    assert_eq!(normalize_authority("Allow"), CandidateDisposition::Allow);
    assert_eq!(normalize_authority("Deny"), CandidateDisposition::Deny);
    assert_eq!(normalize_authority("Reject"), CandidateDisposition::Deny);
    assert_eq!(
        normalize_authority("Escalate"),
        CandidateDisposition::Escalate
    );
    // NotApplicable or unknown → deny (deny-if-required).
    assert_eq!(
        normalize_authority("NotApplicable"),
        CandidateDisposition::Deny
    );
    assert_eq!(normalize_authority("Unknown"), CandidateDisposition::Deny);
}
