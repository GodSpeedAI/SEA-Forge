use sea_forge_core::types::ProjectionKind;
use sea_forge_domainforge::{load_validate, project, SeaSourceSet, SourceFile};
use sha2::{Digest, Sha256};

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
fn calm_projection_produces_valid_json() {
    let model = load_validate(&demo_source_set()).unwrap();
    let calm = project(&model, &ProjectionKind::Calm).unwrap();
    assert!(
        calm.contains_key("calm.json"),
        "CALM output should have calm.json"
    );
    let content = &calm["calm.json"];
    let value: serde_json::Value = serde_json::from_str(content).unwrap();
    assert!(value.is_object(), "CALM output should be a JSON object");
}

#[test]
fn rdf_projection_produces_turtle() {
    let model = load_validate(&demo_source_set()).unwrap();
    let rdf = project(&model, &ProjectionKind::Rdf).unwrap();
    assert!(
        rdf.contains_key("model.ttl"),
        "RDF output should have model.ttl"
    );
    let turtle = &rdf["model.ttl"];
    assert!(!turtle.is_empty(), "Turtle output should not be empty");
    // Turtle should contain triples (look for typical Turtle syntax).
    assert!(
        turtle.contains("@prefix") || turtle.contains("a ") || turtle.contains("<"),
        "Turtle should contain RDF triples"
    );
}

#[test]
fn projection_is_deterministic() {
    let model = load_validate(&demo_source_set()).unwrap();
    let calm1 = project(&model, &ProjectionKind::Calm).unwrap();
    let calm2 = project(&model, &ProjectionKind::Calm).unwrap();
    assert_eq!(
        calm1["calm.json"], calm2["calm.json"],
        "CALM projection must be byte-identical for same model"
    );
    let rdf1 = project(&model, &ProjectionKind::Rdf).unwrap();
    let rdf2 = project(&model, &ProjectionKind::Rdf).unwrap();
    assert_eq!(
        rdf1["model.ttl"], rdf2["model.ttl"],
        "RDF projection must be byte-identical for same model"
    );
}

#[test]
fn projection_different_models_produce_different_output() {
    let model1 = load_validate(&demo_source_set()).unwrap();
    let modified_sea = r#"@namespace "demo"
@version "1.0.0"

Entity "Different" in demo
Resource "Thing" units in demo
Flow "Thing" from "Different" to "Different" quantity 1
"#;
    let model2 = load_validate(&SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![source_file("demo.sea", modified_sea)],
    })
    .unwrap();
    let calm1 = project(&model1, &ProjectionKind::Calm).unwrap();
    let calm2 = project(&model2, &ProjectionKind::Calm).unwrap();
    assert_ne!(
        calm1["calm.json"], calm2["calm.json"],
        "Different models should produce different CALM output"
    );
}

#[test]
fn unsupported_kind_rejected() {
    let model = load_validate(&demo_source_set()).unwrap();
    let result = project(&model, &ProjectionKind::Manifest);
    assert!(result.is_err(), "Manifest projection should be unsupported");
}
