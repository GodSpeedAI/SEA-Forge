// Application Contract and Canonical Semantic Envelope harness.
//
// DomainForge 0.16.0 exposes `resolve_application_contract` and
// `resolve_semantic_envelope` as library APIs only; no `domainforge` CLI
// subcommand reaches them (see validation/limitations.md L4). This harness is
// the reproducible way to inspect both documents for the canonical interaction
// model. It reads .sea files, never writes, and does not modify DomainForge.
//
// Build and run:
//
//   mkdir -p /tmp/dfharness/src && cd /tmp/dfharness
//   cp <this file> src/main.rs
//   cat > Cargo.toml <<'TOML'
//   [package]
//   name = "dfharness"
//   version = "0.1.0"
//   edition = "2021"
//
//   [dependencies]
//   domainforge-core = { path = "/home/sprime01/projects/domainforge/domainforge-core" }
//   serde_json = "1"
//
//   [workspace]
//   TOML
//   cargo build --release
//   ./target/release/dfharness envelope /path/to/interaction-model.sea
//   ./target/release/dfharness contract /path/to/interaction-model.sea
//
// Extra .sea paths after the entry file are added to the in-memory source map,
// which is how a multi-module closure is inspected.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

/// Build the JSON source map {logical_id: source} the resolver expects.
/// Logical IDs are "./<file-name>" so relative imports resolve.
fn source_map(paths: &[String]) -> String {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for p in paths {
        let path = Path::new(p);
        let logical = path.file_name().unwrap().to_string_lossy().to_string();
        map.insert(
            format!("./{logical}"),
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {p}: {e}")),
        );
    }
    serde_json::to_string(&map).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: dfharness <contract|envelope> <entry.sea> [more.sea ...]");
        std::process::exit(1);
    }
    let mode = args[0].clone();
    let entry = format!(
        "./{}",
        Path::new(&args[1]).file_name().unwrap().to_string_lossy()
    );
    let sources = source_map(&args[1..]);

    match mode.as_str() {
        "contract" => {
            match domainforge_core::application::resolve::resolve_application_contract(
                &entry, &sources,
            ) {
                Ok(doc) => println!("{}", serde_json::to_string_pretty(&doc).unwrap()),
                Err(diags) => {
                    eprintln!("APPLICATION CONTRACT DIAGNOSTICS ({}):", diags.len());
                    for d in diags {
                        eprintln!("  {:?} {}", d.code, d.message);
                    }
                    std::process::exit(2);
                }
            }
        }
        "envelope" => {
            match domainforge_core::application::envelope::resolve_semantic_envelope(
                &entry, &sources,
            ) {
                Ok(doc) => println!("{}", serde_json::to_string_pretty(&doc).unwrap()),
                Err(diags) => {
                    eprintln!("ENVELOPE DIAGNOSTICS ({}):", diags.len());
                    for d in diags {
                        eprintln!("  {:?} {}", d.code, d.message);
                    }
                    std::process::exit(2);
                }
            }
        }
        other => {
            eprintln!("unknown mode {other}");
            std::process::exit(1);
        }
    }
}
