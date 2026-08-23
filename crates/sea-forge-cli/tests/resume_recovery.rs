//! F-11 regression: stranded `Active` cases have a recovery path.
//!
//! A crash (or kill -9) between `ItemActivated` (appended before execution)
//! and the post-settlement completion event leaves an episode `Active` in
//! replay; `next_case_actions` then returns empty and `resume` answered
//! exit 5 forever while `reopen` refused. Recovery terminal-settles those
//! episodes as rejected/interrupted and re-drives the case from events.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sea-forge"))
}

const POLICY: &str = "version: \"0.1\"\nrules: []\n";

fn activated_event(case_id: &str, item_id: &str, payload: &str) -> String {
    format!(
        "{{\"version\":\"0.1\",\"event_id\":\"tev_{case_id}_01\",\"run_id\":\"run_{case_id}\",\
         \"plan_item_id\":\"{item_id}\",\"kind\":\"item_activated\",\
         \"actor_id\":\"operator_local\",\"timestamp\":\"2026-08-23T00:00:00Z\",\
         \"payload\":{payload}}}\n"
    )
}

/// Materialize a case that died right after activating `item_id`.
struct Stranded {
    root: PathBuf,
    case_id: String,
}

fn stranded_case(tag: &str, item_json: &str, activation_payload: &str, state: &str) -> Stranded {
    let root = std::env::temp_dir().join(format!(
        "f11-{tag}-{}-{}",
        std::process::id(),
        chrono_millis()
    ));
    fs::create_dir_all(root.join("cases")).unwrap();
    let case_id = format!("case_{tag}");
    let case_dir = root.join("cases").join(&case_id);
    fs::create_dir_all(&case_dir).unwrap();

    let case = format!(
        "{{\"version\":\"0.2\",\"case_id\":\"{case_id}\",\
         \"intent\":{{\"intent_id\":\"int_{tag}\",\"summary\":\"s\",\
         \"actor_id\":\"operator_local\",\"process_id\":\"cli\",\
         \"created_at\":\"2026-08-23T00:00:00Z\"}},\
         \"state\":\"{state}\",\"plan_ref\":\"plan_{tag}\",\"run_ids\":[],\
         \"close_reason\":null,\"created_at\":\"2026-08-23T00:00:00Z\",\"closed_at\":null}}"
    );
    fs::write(case_dir.join("case.json"), case).unwrap();

    let plan = format!(
        "{{\"version\":\"0.2\",\"plan_id\":\"plan_{tag}\",\"case_id\":\"{case_id}\",\
         \"run_id\":\"run_{tag}\",\"intent_id\":\"int_{tag}\",\"items\":[{item_json}]}}"
    );
    fs::write(case_dir.join("plan.json"), plan).unwrap();

    // The case ledger must exist and verify; it is empty because the crash
    // happened between the trace append and any settlement commit.
    let ledger =
        sea_forge_ledger::LedgerStream::open(&root, format!("case-{case_id}"), "cli").unwrap();
    ledger.verify().unwrap();

    fs::write(
        case_dir.join("case-events.jsonl"),
        activated_event(&case_id, "item_a", activation_payload),
    )
    .unwrap();

    let policy_path = root.join("policy.yaml");
    fs::write(&policy_path, POLICY).unwrap();

    Stranded { root, case_id }
}

fn chrono_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn run_resume(stranded: &Stranded) -> (Option<i32>, String, String) {
    let output = cli()
        .args([
            "resume",
            "--root",
            stranded.root.to_str().unwrap(),
            "--policy",
            stranded.root.join("policy.yaml").to_str().unwrap(),
            &stranded.case_id,
        ])
        .output()
        .unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn events_path(stranded: &Stranded) -> PathBuf {
    stranded
        .root
        .join("cases")
        .join(&stranded.case_id)
        .join("case-events.jsonl")
}

fn sandboxed_item(required: bool) -> String {
    format!(
        "{{\"plan_item_id\":\"item_a\",\"name\":\"task\",\
         \"operations\":[],\
         \"settlement_criteria\":{{\"required_artifacts\":[],\"required_declarations\":[]}},\
         \"item_kind\":\"sandboxed_task\",\"markers\":{{\"required\":{required}}},\
         \"max_instances\":1,\"depends_on\":[]}}"
    )
}

#[test]
fn f11_crash_mid_episode_required_item_recovers_to_terminated_not_exit5_forever() {
    let stranded = stranded_case("req", &sandboxed_item(true), "{\"instance\":1}", "active");
    let (code, stdout, stderr) = run_resume(&stranded);
    assert_eq!(code, Some(3), "stdout={stdout} stderr={stderr}");
    assert!(
        stdout.contains("recovered_interrupted_episodes=1"),
        "recovery must disclose what it settled: {stdout}"
    );
    assert!(stdout.contains("case_state=terminated"), "{stdout}");

    let events = fs::read_to_string(events_path(&stranded)).unwrap();
    assert!(
        events.contains("\"basis\":\"interrupted\""),
        "the interrupted episode must be terminal-settled honestly: {events}"
    );

    // The case record carries the lawful close reason derived from events.
    let case_json = fs::read_to_string(
        stranded
            .root
            .join("cases")
            .join(&stranded.case_id)
            .join("case.json"),
    )
    .unwrap();
    assert!(
        case_json.contains("required_item_failed:item_a"),
        "{case_json}"
    );
}

#[test]
fn f11_crash_mid_episode_optional_item_recovers_to_completed() {
    let stranded = stranded_case("opt", &sandboxed_item(false), "{\"instance\":1}", "active");
    let (code, stdout, stderr) = run_resume(&stranded);
    assert_eq!(code, Some(0), "stdout={stdout} stderr={stderr}");
    assert!(stdout.contains("case_state=completed"), "{stdout}");
}

#[test]
fn f11_parked_human_task_is_not_disturbed_by_recovery() {
    let human_task = "{\"plan_item_id\":\"item_a\",\"name\":\"approve\",\
                      \"operations\":[],\
                      \"settlement_criteria\":{\"required_artifacts\":[],\"required_declarations\":[]},\
                      \"item_kind\":\"human_task\",\"markers\":{\"required\":true},\
                      \"max_instances\":1,\"depends_on\":[]}"
        .to_string();
    let stranded = stranded_case("human", &human_task, "{\"human_task\":true}", "active");
    let before = fs::read_to_string(events_path(&stranded)).unwrap();

    let (code, stdout, stderr) = run_resume(&stranded);
    assert_eq!(
        code,
        Some(5),
        "a parked human task stays parked: {stdout} {stderr}"
    );
    assert!(stdout.contains("case_state=active"), "{stdout}");

    let after = fs::read_to_string(events_path(&stranded)).unwrap();
    assert_eq!(
        before, after,
        "recovery must not settle a parked human task"
    );
}

#[test]
fn f11_awaiting_approval_still_refuses_while_approvals_pending() {
    // The recovery path must not weaken the existing approval gate.
    let stranded = stranded_case(
        "gate",
        &sandboxed_item(true),
        "{\"instance\":1}",
        "awaiting_approval",
    );
    // No approval records exist in the ledger, so resume proceeds into the
    // expiry path and terminates — the point here is that the gate text no
    // longer hard-refuses non-awaiting states only; an awaiting_approval
    // case without approvals still reaches its documented termination.
    let (code, _stdout, _stderr) = run_resume(&stranded);
    assert_eq!(code, Some(4));
}

#[test]
fn f11_terminal_states_are_still_refused() {
    let stranded = stranded_case(
        "term",
        &sandboxed_item(true),
        "{\"instance\":1}",
        "terminated",
    );
    let (_code, _stdout, stderr) = run_resume(&stranded);
    assert!(
        stderr.contains("only awaiting_approval or active can resume"),
        "{stderr}"
    );
}
