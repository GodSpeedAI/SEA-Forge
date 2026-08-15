use sea_forge_domainforge::{load_validate, project, evaluate_authority, SeaSourceSet, SourceFile};
use sea_forge_core::types::ProjectionKind;
use sha2::{Digest, Sha256};
use std::panic;
use std::os::unix::process::ExitStatusExt;

fn sha(content: &str) -> String { format!("{:x}", Sha256::digest(content.as_bytes())) }

fn source(entry: &str, content: &str) -> SeaSourceSet {
    SeaSourceSet { entry_uri: entry.into(), files: vec![SourceFile { uri: entry.trim_start_matches("./").into(), sha256: sha(content), content: content.into() }] }
}

const TINY: &str = "@namespace \"t\"\n@version \"1.0.0\"\n\nEntity \"Sample\" in t\n";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "nest" {
        let depth: usize = args[2].parse().unwrap();
        let content = format!("@namespace \"t\"\n@version \"1.0.0\"\npolicy p as: {}1\n", "-".repeat(depth));
        let set = source("model.sea", &content);
        let result = load_validate(&set);
        println!("depth={depth} => {:?}", result.as_ref().map(|m| m.model_ref.semantic_model_sha256.clone()).map_err(|e| e.to_string()));
        return;
    }

    // S3: entry spelling aliasing
    println!("== S3: entry spelling aliasing ==");
    let a = load_validate(&source("model.sea", TINY)).unwrap();
    let b = load_validate(&source("./model.sea", TINY));
    match b {
        Ok(b) => {
            println!("  both spellings accepted; identity equal: {}", a.model_ref.semantic_model_sha256 == b.model_ref.semantic_model_sha256);
            println!("  A: {}", a.model_ref.semantic_model_sha256);
            println!("  B: {}", b.model_ref.semantic_model_sha256);
        }
        Err(e) => println!("  ./model.sea rejected: {e}"),
    }

    // S7: projection determinism (repo fixture)
    println!("\n== S7: projection determinism (demo.sea) ==");
    let content = std::fs::read_to_string("/home/sprime01/projects/sea-rs/crates/sea-forge-domainforge/tests/fixtures/demo.sea").unwrap();
    let m = load_validate(&source("demo.sea", &content)).unwrap();
    for kind in [ProjectionKind::Calm, ProjectionKind::Rdf, ProjectionKind::Kg] {
        let p1 = project(&m, &kind).unwrap();
        let p2 = project(&m, &kind).unwrap();
        let det = p1 == p2;
        println!("  {kind:?}: deterministic={det} files={:?}", p1.keys().collect::<Vec<_>>());
    }

    // S8: authority case-insensitive semantic target
    println!("\n== S8: evaluate_authority case-insensitivity ==");
    for rid in ["docs/Sample.md", "docs/SAMPLE.md", "docs/sample.md", "docs/other.md"] {
        let t = evaluate_authority(&m, "write_file", rid, vec!["ev".into()]).unwrap();
        println!("  {rid:<18} => {} ({})", t.raw_decision, serde_json::to_string(&t.normalized_disposition).unwrap());
    }

    // S5: generated-zone guard spellings
    println!("\n== S5: is_generated_zone spellings ==");
    for p in ["src/gen/model.rs", "src//gen//model.rs", "src/./gen/model.rs", "src/gen", "mysrc/gen/x.rs", "fixtures/semantic/m.semantic.fixture.yaml", "docs/x.ast.json"] {
        println!("  {p:<42} generated={} edit_denied={}", sea_forge_spec_pipeline::is_generated_zone(p), sea_forge_spec_pipeline::check_generated_zone_edit(p).is_err());
    }

    // S6: build_projection_record unconditional validation
    println!("\n== S6: projection record 'validated' without validation ==");
    let mut outputs = std::collections::BTreeMap::new();
    outputs.insert("model.ttl".to_string(), "garbage not validated at all".to_string());
    let rec = sea_forge_spec_pipeline::build_projection_record("proj_1", ProjectionKind::Kg, "adapter", "case", "run", None, vec![], "ih", &outputs, "2026-01-01T00:00:00Z").unwrap();
    println!("  status={:?} basis={:?} validator_ref={}", rec.validation.status, rec.validation.basis, rec.validation.validator_ref);

    // S1/S2: parse_fixed
    println!("\n== S1/S2: parse_fixed ==");
    for s in ["1.5", "1.ab\u{2200}\u{2200}", "9300000000000", "-0.5"] {
        let s = s.to_string();
        let d = s.clone();
        let r = panic::catch_unwind(move || sea_forge_capability::promotion::parse_fixed(&s));
        match r {
            Ok(v) => println!("  {:?} => {v}", d),
            Err(_) => println!("  {:?} => PANIC", d),
        }
    }
    // stack-exhaustion child runs
    println!("\n== S4: deep nesting (child processes) ==");
    let exe = std::env::current_exe().unwrap();
    for depth in [1_000usize, 10_000, 50_000, 100_000, 300_000] {
        let out = std::process::Command::new(&exe)
            .args(["nest", &depth.to_string()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .unwrap();
        let code = out.status.code();
        let sig = out.status.signal();
        let tail = String::from_utf8_lossy(&out.stdout);
        println!("  depth={depth:>7} exit_code={code:?} signal={sig:?} stdout={:?}", tail.trim());
    }
}
