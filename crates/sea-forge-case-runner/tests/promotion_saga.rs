//! WP-22 invariants W1 to W6, written before the saga. The oracles read ground
//! truth: the bytes on disk and the step log, never a return value alone.

use proptest::prelude::*;
use sea_forge_case_runner::promotion::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn cfg() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

/// Every file under `root` (relative path to bytes), minus `skip` prefixes.
fn snapshot(root: &Path, skip: &[&str]) -> BTreeMap<String, Vec<u8>> {
    fn walk(base: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        let Ok(rd) = fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(base, &p, out);
            } else {
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel, fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out.retain(|k, _| !skip.iter().any(|s| k.starts_with(s)));
    out
}

const SKIP: [&str; 2] = ["promotions/", "locks/"];

fn log_steps(root: &Path, key: &Key) -> Vec<String> {
    let text = fs::read_to_string(root.join("promotions").join(key.id()).join("steps.jsonl"))
        .unwrap_or_default();
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|v| v["step"].as_str().map(str::to_owned))
        .collect()
}

fn store(base: &Path, name: &str) -> PathBuf {
    let root = base.join(name);
    init_store(&root, "w0").unwrap();
    root
}

fn promotion(proposal: &str, cand: &str, base: &str, keys: &[String]) -> Promotion {
    Promotion {
        key: Key {
            proposal_id: proposal.into(),
            candidate_world_ref: cand.into(),
        },
        base_world_ref: base.into(),
        changed_keys: keys.to_vec(),
        mapping: keys
            .iter()
            .map(|k| (k.clone(), format!("{k}-v2")))
            .collect(),
    }
}

fn arb_keys() -> impl Strategy<Value = Vec<String>> {
    proptest::collection::btree_set("[a-d][0-9]", 0..5).prop_map(|s| s.into_iter().collect())
}

fn arb_fault() -> impl Strategy<Value = FaultPoint> {
    let all = FaultPoint::all();
    (0..all.len()).prop_map(move |i| all[i])
}

fn arb_req(base: &'static str) -> impl Strategy<Value = Promotion> {
    ("[pq][0-3]", "w[1-4]", arb_keys()).prop_map(move |(p, c, k)| promotion(&p, &c, base, &k))
}

/// The three allowed final states, judged from bytes (W2 oracle).
fn assert_not_mixed(
    root: &Path,
    initial: &BTreeMap<String, Vec<u8>>,
    req: &Promotion,
    out: &Outcome,
) {
    match out {
        Outcome::Promoted => {
            let pins = fs::read_to_string(root.join("pins.jsonl")).unwrap();
            let last: Value = serde_json::from_str(pins.lines().last().unwrap()).unwrap();
            assert_eq!(last["to"], req.key.candidate_world_ref.as_str());
            let id = req.key.id();
            assert!(root.join("frozen").join(format!("{id}.json")).exists());
            assert!(root.join("stamps").join(format!("{id}.json")).exists());
            let steps = log_steps(root, &req.key);
            for s in ["begin", "freeze", "pin", "stamp", "complete"] {
                assert!(steps.iter().any(|x| x == s), "missing {s} in {steps:?}");
            }
            assert!(!steps.iter().any(|x| x == "rollback"));
            assert!(require_promoted(root, &req.key).is_ok());
        }
        Outcome::PreSaga => {
            assert_eq!(&snapshot(root, &SKIP), initial, "pre-saga bytes differ");
            assert!(require_promoted(root, &req.key).is_err());
        }
        Outcome::Uncertain(_) => {
            assert!(
                require_promoted(root, &req.key).is_err(),
                "uncertain used as promoted"
            );
        }
        Outcome::InProgress => panic!("a reconciled saga is never in progress"),
    }
}

// --- the seam ---------------------------------------------------------------

#[test]
fn fault_points_are_generated_from_step_all() {
    let ordinals: Vec<usize> = Step::ALL.iter().map(|s| s.ordinal()).collect();
    assert_eq!(ordinals, (0..Step::ALL.len()).collect::<Vec<_>>());
    let points = FaultPoint::all();
    assert_eq!(points.len(), Step::ALL.len() * Phase::ALL.len());
    for s in Step::ALL {
        assert!(
            points.iter().any(|p| p.step == s),
            "{s:?} has no fault point"
        );
    }
}

#[test]
fn crash_after_every_step_and_mid_write_then_reconcile() {
    let keys = vec!["a1".to_string(), "b2".to_string()];
    for point in FaultPoint::all() {
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "s");
        let initial = snapshot(&root, &SKIP);
        let req = promotion("p0", "w1", "w0", &keys);
        let crashed = run(&root, &req, Faults(Some(point)));
        if point.step != Step::Rollback {
            assert_eq!(
                crashed,
                Err(SagaError::Crash(point)),
                "{point:?} unreachable"
            );
        }
        let out = reconcile(&root, &req.key, Faults::default()).unwrap();
        assert_not_mixed(&root, &initial, &req, &out);
        assert!(
            matches!(out, Outcome::Promoted | Outcome::PreSaga),
            "{point:?} -> {out:?}"
        );
        // and the saga is finishable by re-running, whatever the crash left
        let again = run(&root, &req, Faults::default()).unwrap();
        assert!(
            matches!(again, Outcome::Promoted | Outcome::PreSaga),
            "{point:?}"
        );
        if out == Outcome::Promoted {
            assert_eq!(again, out, "{point:?}");
        }
    }
}

// --- W1 idempotence -----------------------------------------------------------

proptest! {
    #![proptest_config(cfg())]

    #[test]
    fn w1_n_runs_equal_one_run(req in arb_req("w0"), n in 1usize..5, fault in arb_fault(), stale in any::<bool>()) {
        let req = if stale { Promotion { base_world_ref: "w9".into(), ..req } } else { req };
        let dir = tempfile::tempdir().unwrap();
        let once = store(dir.path(), "once");
        let many = store(dir.path(), "many");
        let _ = run(&once, &req, Faults::default());
        for _ in 0..n { let _ = run(&many, &req, Faults::default()); }
        let _ = reconcile(&many, &req.key, Faults::default());
        prop_assert_eq!(snapshot(&once, &["locks/"]), snapshot(&many, &["locks/"]));
        // a crash at any point followed by plain re-runs lands on the same bytes
        let crashy = store(dir.path(), "crashy");
        let _ = run(&crashy, &req, Faults(Some(fault)));
        for _ in 0..n { let _ = run(&crashy, &req, Faults::default()); }
        prop_assert_eq!(snapshot(&once, &["locks/", "promotions/"]), snapshot(&crashy, &["locks/", "promotions/"]));
    }

    // --- W2 crash consistency ----------------------------------------------------
    #[test]
    fn w2_crash_then_reconcile_is_never_mixed(
        req in arb_req("w0"), first in arb_fault(), second in proptest::option::of(arb_fault()),
        repeats in 0usize..3
    ) {
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "s");
        let initial = snapshot(&root, &SKIP);
        let _ = run(&root, &req, Faults(Some(first)));
        if let Some(p) = second { let _ = reconcile(&root, &req.key, Faults(Some(p))); }
        let mut out = reconcile(&root, &req.key, Faults::default()).unwrap();
        for _ in 0..repeats { out = reconcile(&root, &req.key, Faults::default()).unwrap(); }
        assert_not_mixed(&root, &initial, &req, &out);
        prop_assert!(!matches!(out, Outcome::Uncertain(_)), "a plain crash must not be uncertain: {out:?}");
    }

    #[test]
    fn w2_corrupted_effects_fail_closed(req in arb_req("w0"), which in 0usize..3) {
        prop_assume!(!req.changed_keys.is_empty());
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "s");
        prop_assert_eq!(run(&root, &req, Faults::default()).unwrap(), Outcome::Promoted);
        let id = req.key.id();
        match which {
            0 => fs::write(root.join("frozen").join(format!("{id}.json")), b"{}").unwrap(),
            1 => fs::write(root.join("stamps").join(format!("{id}.json")), b"{}").unwrap(),
            _ => {
                let p = root.join("pins.jsonl");
                let t = fs::read_to_string(&p).unwrap().replace(&format!("\"to\":\"{}\"", req.key.candidate_world_ref), "\"to\":\"wX\"");
                fs::write(&p, t).unwrap();
            }
        }
        let out = reconcile(&root, &req.key, Faults::default()).unwrap();
        prop_assert!(matches!(out, Outcome::Uncertain(_)), "{out:?}");
        prop_assert!(require_promoted(&root, &req.key).is_err());
        prop_assert!(matches!(classify(&root, &req.key).unwrap(), Outcome::Uncertain(_)));
    }

    // --- W3 NT-07 ---------------------------------------------------------------
    #[test]
    fn w3_settled_evidence_keeps_its_world(
        req in arb_req("w0"), fault in proptest::option::of(arb_fault()), do_repin in any::<bool>(),
        body in "[a-z]{0,8}"
    ) {
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "s");
        settle_evidence(&root, "e1", "w0", &json!({"b": body})).unwrap();
        let before = fs::read(root.join("evidence/e1.json")).unwrap();
        let (w_before, d_before) = read_evidence(&root, "e1").unwrap();
        let _ = run(&root, &req, Faults(fault));
        let _ = reconcile(&root, &req.key, Faults::default());
        if do_repin { repin(&root, "w0", "wrong").unwrap(); repin(&root, "w0", "wrong").unwrap(); }
        prop_assert_eq!(fs::read(root.join("evidence/e1.json")).unwrap(), before);
        let (w, d) = read_evidence(&root, "e1").unwrap();
        prop_assert_eq!((w.as_str(), d.as_str()), (w_before.as_str(), d_before.as_str()));
        prop_assert_eq!(w, "w0");
        // the evidence cannot be re-bound to another world
        assert!(settle_evidence(&root, "e1", &req.key.candidate_world_ref, &json!({"b": body})).is_err());
    }

    // --- W4 pins ----------------------------------------------------------------
    #[test]
    fn w4_in_flight_case_keeps_its_pin(req in arb_req("w0"), fault in proptest::option::of(arb_fault())) {
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "world");
        let cases = dir.path().join("cases");
        let a = cases.join("A");
        prop_assert_eq!(pin_case(&root, &a).unwrap(), "w0");
        let a_bytes = fs::read(a.join("runs/pin.json")).unwrap();
        let _ = run(&root, &req, Faults(fault));
        let out = reconcile(&root, &req.key, Faults::default()).unwrap();
        prop_assert_eq!(pin_case(&root, &a).unwrap(), "w0"); // re-pinning an open case keeps it
        prop_assert_eq!(fs::read(a.join("runs/pin.json")).unwrap(), a_bytes);
        let b = cases.join("B");
        let want = if out == Outcome::Promoted { req.key.candidate_world_ref.clone() } else { "w0".to_string() };
        prop_assert_eq!(pin_case(&root, &b).unwrap(), want);
    }

    // --- W5 key mapping ---------------------------------------------------------
    #[test]
    fn w5_mapping_total_injective_or_refused_before_any_write(
        changed in arb_keys(),
        mapping in proptest::collection::vec(("[a-d][0-9]", "[a-d][0-9]"), 0..6),
    ) {
        let mut by_old: BTreeMap<&String, std::collections::BTreeSet<&String>> = BTreeMap::new();
        for (o, n) in &mapping { by_old.entry(o).or_default().insert(n); }
        let unambiguous = by_old.values().all(|v| v.len() == 1);
        let total = changed.iter().all(|k| by_old.contains_key(k));
        let mut images = std::collections::BTreeSet::new();
        let injective = by_old.values().all(|v| images.insert(*v.iter().next().unwrap()));
        let expect_ok = unambiguous && total && injective;
        let got = map_keys(&changed, &mapping);
        prop_assert_eq!(got.is_ok(), expect_ok, "{:?}", got);
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "s");
        let before = snapshot(&root, &[]);
        let req = Promotion {
            key: Key { proposal_id: "p".into(), candidate_world_ref: "w1".into() },
            base_world_ref: "w0".into(), changed_keys: changed.clone(), mapping: mapping.clone(),
        };
        let out = run(&root, &req, Faults::default());
        if expect_ok {
            prop_assert_eq!(out.unwrap(), Outcome::Promoted);
        } else {
            assert!(matches!(out, Err(SagaError::Refused { .. })));
            prop_assert_eq!(snapshot(&root, &[]), before, "a refused mapping wrote bytes");
        }
    }
}

// --- W6 concurrency -------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn w6_same_key_commits_once_and_same_proposal_serializes(
        reqs in proptest::collection::vec(arb_req("w0"), 2..5)
    ) {
        let dir = tempfile::tempdir().unwrap();
        let root = store(dir.path(), "s");
        let handles: Vec<_> = reqs.iter().cloned().map(|r| {
            let root = root.clone();
            std::thread::spawn(move || run(&root, &r, Faults::default()))
        }).collect();
        for h in handles { let _ = h.join().unwrap(); }
        let pins = fs::read_to_string(root.join("pins.jsonl")).unwrap();
        let lines: Vec<Value> = pins.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        // a linear chain: each pin starts where the last ended
        for w in lines.windows(2) { prop_assert_eq!(&w[1]["from"], &w[0]["to"]); }
        // never two commits of one key, never two of one proposal off one base
        let mut keys = std::collections::BTreeSet::new();
        for l in &lines[1..] { assert!(keys.insert(l["key"].as_str().unwrap().to_string())); }
        for r in &reqs {
            let id = r.key.id();
            let n = lines.iter().filter(|l| l["key"] == id.as_str()).count();
            prop_assert!(n <= 1);
            let out = classify(&root, &r.key).unwrap();
            prop_assert!(matches!(out, Outcome::Promoted | Outcome::PreSaga), "{out:?}");
            prop_assert_eq!(out == Outcome::Promoted, n == 1);
        }
    }
}
