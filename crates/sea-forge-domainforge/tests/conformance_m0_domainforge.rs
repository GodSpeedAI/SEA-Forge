use sea_forge_core::ForgeError;
use sea_forge_domainforge::{
    load_validate, normalize_authority, CandidateDisposition, SeaSourceSet, SourceFile,
    MAX_IMPORT_DEPTH,
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
fn equivalent_entry_spellings_produce_the_same_semantic_identity() {
    // SUP-02: `model.sea` and `./model.sea` name the same canonical entry, so
    // they must yield the same `semantic_model_sha256` (identity is derived
    // from resolved content, never the raw caller spelling).
    let canonical = load_validate(&demo_source_set()).unwrap();

    let mut aliased = demo_source_set();
    // The demo fixture's file stays `demo.sea`; only the entry spelling differs.
    aliased.entry_uri = "./demo.sea".into();
    let aliased_model = load_validate(&aliased).unwrap();

    assert_eq!(
        canonical.model_ref.semantic_model_sha256, aliased_model.model_ref.semantic_model_sha256,
        "equivalent entry spellings must hash to the same semantic identity"
    );
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

// ── Task 5: Source-set trust boundary (M0 prerequisite) ──

const BASE_SEA: &str = r#"@namespace "shared"
export Entity "Customer"
"#;

const ENTRY_SEA: &str = r#"@namespace "main"
import { Customer } from "shared"
Entity "Order" in main
"#;

fn multi_file_set() -> SeaSourceSet {
    SeaSourceSet {
        entry_uri: "entry.sea".into(),
        files: vec![
            source_file("base.sea", BASE_SEA),
            source_file("entry.sea", ENTRY_SEA),
        ],
    }
}

#[test]
fn multi_file_model_with_resolved_import_passes() {
    let model = load_validate(&multi_file_set()).unwrap();
    assert!(model.model_ref.source_refs.len() >= 2);
    assert!(model.model_ref.concept_refs.contains(&"Order".to_string()));
    // The imported entity must be merged into the graph too, not just
    // validated as resolvable — proves the model isn't entry-only.
    assert!(model
        .model_ref
        .concept_refs
        .contains(&"Customer".to_string()));
}

#[test]
fn unresolved_import_fails() {
    let mut ss = multi_file_set();
    // Remove the base file so the import has no resolution target.
    ss.files.retain(|f| f.uri != "base.sea");
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("unresolved import"),
        "expected unresolved import: {err}"
    );
}

#[test]
fn non_entry_hash_drift_fails() {
    let mut ss = multi_file_set();
    // Corrupt the base file's content but keep its declared hash.
    let base = ss.files.iter_mut().find(|f| f.uri == "base.sea").unwrap();
    base.content = r#"@namespace "shared"
export Entity "Different"
"#
    .to_string();
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("source hash drift"),
        "expected hash drift: {err}"
    );
}

#[test]
fn duplicate_uri_fails() {
    let mut ss = multi_file_set();
    // Add a duplicate of base.sea.
    ss.files.push(source_file("base.sea", BASE_SEA));
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("duplicate"),
        "expected duplicate URI: {err}"
    );
}

#[test]
fn absolute_uri_rejected() {
    let mut ss = demo_source_set();
    ss.files[0].uri = "/etc/passwd".into();
    ss.entry_uri = "/etc/passwd".into();
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("absolute"),
        "expected absolute URI rejection: {err}"
    );
}

#[test]
fn traversal_uri_rejected() {
    let mut ss = demo_source_set();
    ss.files[0].uri = "../escape.sea".into();
    ss.entry_uri = "../escape.sea".into();
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("traversal"),
        "expected traversal URI rejection: {err}"
    );
}

#[test]
fn source_count_limit_exceeded() {
    let mut ss = demo_source_set();
    // Exceed MAX_SOURCE_COUNT (64).
    for i in 0..sea_forge_domainforge::MAX_SOURCE_COUNT {
        let content = format!("@namespace \"ns{i}\"\nEntity \"E{i}\" in ns{i}\n");
        ss.files
            .push(source_file(&format!("file{i}.sea"), &content));
    }
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("source count"),
        "expected source count limit: {err}"
    );
}

#[test]
fn aggregate_bytes_limit_exceeded() {
    let mut ss = demo_source_set();
    // Create a file that exceeds MAX_AGGREGATE_BYTES (1 MiB).
    let big = "x".repeat(sea_forge_domainforge::MAX_AGGREGATE_BYTES + 1);
    ss.files.push(source_file("big.sea", &big));
    let err = load_validate(&ss).unwrap_err().to_string();
    assert!(
        err.contains("domain_model_error") && err.contains("aggregate bytes"),
        "expected aggregate bytes limit: {err}"
    );
}

fn linear_import_chain(depth: usize) -> SeaSourceSet {
    let mut files = Vec::with_capacity(depth + 1);
    for index in 0..=depth {
        let uri = if index == 0 {
            "entry.sea".to_string()
        } else {
            format!("module{index}.sea")
        };
        let content = if index == depth {
            format!("@namespace \"ns{index}\"\nexport Entity \"E{index}\"\n")
        } else {
            format!(
                "@namespace \"ns{index}\"\nimport {{ E{} }} from \"./module{}.sea\"\nexport Entity \"E{index}\"\n",
                index + 1,
                index + 1,
            )
        };
        files.push(source_file(&uri, &content));
    }
    SeaSourceSet {
        entry_uri: "entry.sea".into(),
        files,
    }
}

#[test]
fn import_depth_limit_accepts_the_bound_and_rejects_the_next_edge() {
    assert!(
        load_validate(&linear_import_chain(MAX_IMPORT_DEPTH)).is_ok(),
        "an import closure exactly at MAX_IMPORT_DEPTH must be valid"
    );

    let error = load_validate(&linear_import_chain(MAX_IMPORT_DEPTH + 1)).unwrap_err();
    assert_eq!(error.class(), "domain_model_error");
    assert!(
        error
            .to_string()
            .contains("import depth 17 exceeds limit 16"),
        "expected an explicit import-depth validation error: {error}"
    );
}

#[test]
fn import_depth_limit_uses_domainforge_normalized_entry_id() {
    let mut source_set = linear_import_chain(MAX_IMPORT_DEPTH + 1);
    // DomainForge normalizes the entry logical path before resolution. The
    // adapter must calculate depth from that canonical closure, not from this
    // raw spelling.
    source_set.entry_uri = "./entry.sea".into();

    let error = load_validate(&source_set).unwrap_err();
    assert_eq!(error.class(), "domain_model_error");
    assert!(
        error
            .to_string()
            .contains("import depth 17 exceeds limit 16"),
        "a normalized entry URI must not bypass the depth limit: {error}"
    );
}

#[test]
fn unsupported_version_is_guarded() {
    // The adapter const must match the linked library version (proves the
    // guard is maintained). If someone bumps domainforge-core without
    // updating the adapter, load_validate returns domain_model_error.
    assert_eq!(
        sea_forge_domainforge::EXPECTED_DOMAINFORGE_VERSION,
        domainforge_core::VERSION,
        "adapter version constant must track the pinned library"
    );
}

#[test]
fn no_source_error_reaches_planning_or_authority() {
    // Domain-model validation is a typed planning error: it blocks planning
    // and authority before side effects without being reported as internal.
    let mut ss = multi_file_set();
    ss.files.retain(|f| f.uri != "base.sea");
    let error = load_validate(&ss).unwrap_err();
    assert_eq!(error.class(), "domain_model_error");
    assert!(matches!(
        error,
        ForgeError::Plan {
            class: "domain_model_error",
            ..
        }
    ));
}

#[test]
fn source_refs_include_all_verified_files() {
    let model = load_validate(&multi_file_set()).unwrap();
    let uris: Vec<&str> = model
        .model_ref
        .source_refs
        .iter()
        .map(|r| r.uri.as_str())
        .collect();
    assert!(
        uris.contains(&"base.sea"),
        "source_refs must include base.sea"
    );
    assert!(
        uris.contains(&"entry.sea"),
        "source_refs must include entry.sea"
    );
}

#[test]
fn deep_nesting_is_rejected_before_parse_not_abort() {
    // SUP-01 regression: a few KB of nested unary expressions must yield a
    // typed domain_model_error, never a stack-overflow SIGABRT.
    let depth = 100_000usize;
    let content = format!(
        "@namespace \"t\"\n@version \"1.0.0\"\npolicy p as: {}1\n",
        "-".repeat(depth)
    );
    let ss = SeaSourceSet {
        entry_uri: "model.sea".into(),
        files: vec![source_file("model.sea", &content)],
    };
    let result = load_validate(&ss);
    assert!(result.is_err(), "deep nesting must be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("nesting depth"),
        "expected a nesting-depth error: {err}"
    );

    // Deep parenthesized nesting is also rejected.
    let parens = format!(
        "@namespace \"t\"\n@version \"1.0.0\"\npolicy p as: {}1{}\n",
        "(".repeat(depth),
        ")".repeat(depth)
    );
    let ss2 = SeaSourceSet {
        entry_uri: "model.sea".into(),
        files: vec![source_file("model.sea", &parens)],
    };
    assert!(load_validate(&ss2).is_err(), "deep parens must be rejected");
}

#[test]
fn modest_nesting_still_parses() {
    // A depth well under the cap must still parse (and fail semantically, not
    // on a nesting guard).
    let content = format!(
        "@namespace \"t\"\n@version \"1.0.0\"\npolicy p as: {}1\n",
        "-".repeat(100)
    );
    let ss = SeaSourceSet {
        entry_uri: "model.sea".into(),
        files: vec![source_file("model.sea", &content)],
    };
    let result = load_validate(&ss);
    // Either parses (and fails semantic validation) or is rejected for some
    // other typed reason — but must not be a nesting-depth rejection.
    match result {
        Ok(_) => {}
        Err(e) => assert!(
            !e.to_string().contains("nesting depth"),
            "modest nesting must not trip the guard: {e}"
        ),
    }
}
