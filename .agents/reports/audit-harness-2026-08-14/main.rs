// Audit verification harness — executes suspected defects against the real crates.
// Research only; runs entirely in tempdirs.
use std::path::Path;
use std::panic;

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, ActorRole, ActorType, AuthorityAction, BindingResolution, IdentityBinding};

use sea_forge_ledger::signing::{load_or_create_signing_key, verify_signature};
use sea_forge_ledger::types::LedgerStream;

fn actor() -> Actor {
    Actor { actor_id: "operator_local".into(), role: ActorRole::Operator }
}

fn binding() -> IdentityBinding {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("policy.yaml");
    std::fs::write(&p, "version: \"0.1\"\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n").unwrap();
    let bundle = AuthorityPolicyBundle::load(&p).unwrap();
    bundle.resolve_identity("operator_local", ActorRole::Operator)
}

fn engine() -> PolicyAuthorityEngine {
    let bundle: AuthorityPolicyBundle = serde_yaml_from_str(
        "version: \"0.1\"\nrules:\n  - name: allow-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"\"\n",
    );
    PolicyAuthorityEngine::new(bundle).unwrap()
}

fn serde_yaml_from_str(s: &str) -> AuthorityPolicyBundle {
    serde_yaml_parse(s)
}
fn serde_yaml_parse(s: &str) -> AuthorityPolicyBundle {
    // serde_yaml is a transitive dep of the authority crate; use JSON fallback if needed.
    // Simplest: ask the authority crate's loader by writing to a temp file.
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("policy.yaml");
    std::fs::write(&p, s).unwrap();
    AuthorityPolicyBundle::load(&p).unwrap()
}

fn eval_write(engine: &PolicyAuthorityEngine, path: &str) -> String {
    let action = AuthorityAction::WriteFile { path: path.into(), content_hint: "x".into() };
    let a = actor();
    let d = engine
        .evaluate(AuthorityEvaluation {
            actor: &a,
            binding: binding(),
            run_id: "run_20260710T120000Z_abcdef",
            case_id: "case_test",
            plan_item_id: "item_01",
            sequence: 1,
            action: &action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    format!("{:?}", d.outcome)
}

fn main() {
    // (sha2 used inside binding())
    let mut failures = 0usize;

    // ---- TEST 1: authority deny_write glob bypass via non-canonical spellings ----
    let e = engine();
    let cases = [
        ("src/gen/model.rs", "control: canonical spelling"),
        ("src//gen//model.rs", "double-slash spelling"),
        ("src/./gen/model.rs", "dot-segment spelling"),
        ("subdir/.env", "nested .env (built-in .env*)"),
        ("sub/.git/config", "nested .git (built-in .git/**)"),
    ];
    println!("== T1: authority hard-boundary evaluation ==");
    for (path, why) in cases {
        let verdict = eval_write(&e, path);
        let bypass = verdict != "Deny";
        println!("  {path:<22} => {verdict:<8} [{why}] {}", if bypass { "<< BYPASS" } else { "" });
        if bypass {
            failures += 1;
        }
    }

    // ---- TEST 2: sandbox safe_join accepts the same spellings and lands in gen zone ----
    println!("\n== T2: safe_join/materialize accepts bypass spellings ==");
    let root = tempfile::tempdir().unwrap();
    let joined = sea_forge_sandbox::safe_join(root.path(), "src//gen//model.rs");
    match joined {
        Ok(p) => {
            std::fs::write(&p, b"x").unwrap();
            let landed = root.path().join("src").join("gen").join("model.rs");
            let ok = landed.exists();
            println!("  safe_join OK: {} -> landed at src/gen/model.rs: {}", p.display(), ok);
            if ok {
                failures += 1;
            }
        }
        Err(err) => println!("  safe_join rejected: {err}"),
    }

    // ---- TEST 3: base64_decode panic on >U+00FF char in signature ----
    println!("\n== T3: signature verification panic on non-ASCII signature ==");
    let kd = tempfile::tempdir().unwrap();
    let key = load_or_create_signing_key(kd.path(), "audit").unwrap();
    let vk = key.verifying_key();
    let bad_sigs = ["ed25519:\u{4E2D}", "ed25519:abc\u{E9}", "ed25519:AAAA\u{100}"]; // CJK char, é (in range), U+0100
    for sig in bad_sigs {
        let sig = sig.to_string();
        let display = sig.clone();
        let result = panic::catch_unwind(move || {
            let _ = verify_signature(&vk, b"msg", &sig);
        });
        match result {
            Ok(()) => println!("  sig={:?} => returned (no panic)", display),
            Err(_) => {
                println!("  sig={:?} => PANIC (index out of bounds)", display);
                failures += 1;
            }
        }
    }

    // ---- TEST 4: prove_entry panic on mmr/entries desync ----
    println!("\n== T4: prove_entry under mmr/entries desync (crash-window state) ==");
    {
        let dir = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(dir.path(), "audit", "audit").unwrap();
        stream.append("test", vec![], serde_json::json!({"n": 1}), vec![]).unwrap();
        let e2 = stream.append("test", vec![], serde_json::json!({"n": 2}), vec![]).unwrap();
        // Simulate the crash window: mmr.json ahead of entries.jsonl.
        let mmr_path = dir.path().join("ledgers").join("audit").join("mmr.json");
        let mut mmr: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&mmr_path).unwrap()).unwrap();
        mmr["leaf_count"] = serde_json::json!(7);
        mmr["peaks"] = serde_json::json!(["sha256:aa", "sha256:bb", "sha256:cc"]);
        std::fs::write(&mmr_path, serde_json::to_string(&mmr).unwrap()).unwrap();
        let ulid = e2.entry_ulid.clone();
        let stream2 = LedgerStream::open(dir.path(), "audit", "audit").unwrap();
        let result = panic::catch_unwind(move || {
            let _ = stream2.prove_entry(&ulid);
        });
        match result {
            Ok(_) => println!("  prove_entry returned Ok"),
            Err(_) => {
                println!("  prove_entry PANICKED (slice out of range) under desync");
                failures += 1;
            }
        }
    }

    // ---- TEST 5: mmr-ahead desync is unrecoverable (verify fails, quarantine doesn't fix) ----
    println!("\n== T5: desync recovery attempt ==");
    {
        let dir = tempfile::tempdir().unwrap();
        let a = LedgerStream::open(dir.path(), "aaa", "audit").unwrap();
        for i in 0..3 {
            a.append("test", vec![], serde_json::json!({"n": i}), vec![]).unwrap();
        }
        let b = LedgerStream::open(dir.path(), "bbb", "audit").unwrap();
        for i in 0..2 {
            b.append("test", vec![], serde_json::json!({"n": i}), vec![]).unwrap();
        }
        // Crash-window state for stream b: mmr from 3-entry stream, entries has 2.
        std::fs::copy(
            dir.path().join("ledgers/aaa/mmr.json"),
            dir.path().join("ledgers/bbb/mmr.json"),
        )
        .unwrap();
        let v1 = b.verify();
        println!("  verify() after desync: {:?}", v1.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));
        let q = b.quarantine_incomplete_tail();
        println!("  quarantine_incomplete_tail: {:?}", q);
        let v2 = b.verify();
        println!("  verify() after quarantine: {:?}", v2.as_ref().map(|_| "Ok").map_err(|e| e.to_string()));
        if v2.is_err() {
            println!("  => desync state NOT recoverable via quarantine (permanent verify failure)");
            failures += 1;
        }
    }

    // ---- TEST 6: LedgerStream::open traversal creates dirs outside root ----
    println!("\n== T6: LedgerStream::open traversal ==");
    {
        let dir = tempfile::tempdir().unwrap();
        let _ = LedgerStream::open(dir.path(), "esc/../../outside_ledger", "audit");
        let escaped = dir.path().join("outside_ledger").exists();
        println!("  created {} outside root: {}", dir.path().join("outside_ledger").display(), escaped);
        if escaped {
            failures += 1;
        }
    }

    // ---- TEST 7: settlement quarantine write traversal via plan_item_id ----
    println!("\n== T7: settlement quarantine path traversal (latent) ==");
    {
        use sea_forge_core::types::*;
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join("run");
        let ws = run_dir.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();
        let claim = SettlementClaim {
            run_id: "run_x".into(),
            plan_item_id: "../../evil_item".into(),
            criteria_ref: None,
            criteria: SettlementCriteria {
                records: Some("records.jsonl".into()),
                per_record_evaluator: Some("demo.score".into()),
                min_pass_ratio: Some(1.0),
                ..Default::default()
            },
            execution: Some(sea_forge_core::types::ExecutionResult {
                status: sea_forge_core::types::ExecutionStatus::Completed,
                exit_code: Some(0),
                stdout_path: "artifacts/stdout.txt".into(),
                stderr_path: "artifacts/stderr.txt".into(),
                started_at: "2026-01-01T00:00:00Z".into(),
                finished_at: "2026-01-01T00:00:01Z".into(),
            }),
            authority_verdicts: vec![Verdict::Allow],
            evaluator_scores: Default::default(),
            batch: Some(BatchEvaluationResult {
                total: 1,
                passed: 0,
                pass_ratio: 0.0,
                min_pass_ratio: 1.0,
                failures: vec![BatchFailure {
                    record: serde_json::json!({"bad": true}),
                    score: 0.0,
                    evidence_ref: "ev".into(),
                }],
            }),
        };
        match sea_forge_settlement::settle(&claim, &ws, &run_dir) {
            Ok(ev) => {
                let escaped = dir.path().join("evil_item.jsonl").exists();
                println!(
                    "  settled {:?}; ../evil_item.jsonl escaped run_dir: {}",
                    ev.status, escaped
                );
                if escaped {
                    failures += 1;
                }
            }
            Err(e) => println!("  settle returned Err: {e}"),
        }
    }

    println!("\n==== {} confirming observations ({} expected-defect confirmations) ====", failures, failures);
}
