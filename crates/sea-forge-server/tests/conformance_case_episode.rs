//! Canonical episode pipeline conformance (SF-003, AUTH-01, §10.6).
//!
//! Three properties of a sandboxed case episode, each of which the dispatcher
//! previously got wrong in a way that looked fine from the outside:
//!
//! 1. **Authority precedes every side effect.** A denied item must leave no
//!    workspace and no artifacts directory behind. The dispatcher used to
//!    create both before it evaluated authority, so a denial still wrote to the
//!    filesystem.
//! 2. **A denial is not a spawn failure.** It used to be reported as a
//!    fabricated `ExecutionResult { status: SpawnFailed }`, describing a spawn
//!    that was attempted and failed. No spawn was ever attempted.
//! 3. **Settlement reads criteria and evidence, not exit codes.**
//!    `exit_code == Some(0)` used to be sufficient for `accepted`, so a command
//!    that exited zero while writing none of its declared artifacts settled as
//!    a success.
//!
//! Before this file, no server test drove `ItemKind::SandboxedTask` through
//! `case_dispatch` at all — which is how an exit-code-only settlement survived
//! in the dispatcher while the CLI's equivalent path was covered by `just
//! proof`.
//!
//! These run through `handle_request` with a real policy, a real ledger, and a
//! real sandboxed execution — the same path a `case.submit` over the socket
//! takes.

use sea_forge_core::types::{
    CasePlan, ItemKind, ItemMarkers, Operation, PlanItem, SettlementCriteria,
};
use sea_forge_server::{handle_request, Request, ServerConfig, ServerState};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A policy that allows `execute_command`, or denies everything.
///
/// `argv0` must be `sea-forge`: `AuthorityPolicyBundle::validate` refuses a
/// *local* allow for `execute_command` with any other argv0 as a
/// `schema_error`, so policy cannot hand out local shell execution.
fn policy(root: &Path, allow: bool) -> PathBuf {
    let rules = if allow {
        "  - name: allow-command\n    verdict: allow\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n"
    } else {
        ""
    };
    let path = root.join(if allow { "allow.yaml" } else { "deny.yaml" });
    fs::write(&path, format!("version: \"0.1\"\nrules:\n{rules}")).unwrap();
    path
}

/// An absolute path named `sea-forge` that resolves to *this* test binary.
///
/// Two independent checks constrain argv0, and only their intersection is
/// executable:
///
/// - The policy rule matches on `Path::file_name(argv[0])`, and validation
///   pins that to `sea-forge` for a local allow.
/// - `untrusted_executable` canonicalizes `argv[0]` and requires it to equal
///   `current_exe()` — SEA Forge only ever executes itself.
///
/// A symlink satisfies both without weakening either. Anything else would mean
/// changing the product to make a test pass.
fn trusted_self_executable(dir: &Path) -> PathBuf {
    let link = dir.join("sea-forge");
    std::os::unix::fs::symlink(std::env::current_exe().expect("current_exe"), &link)
        .expect("link the test binary as sea-forge");
    link
}

/// Arguments that make the libtest harness exit 0 without running anything.
const EXIT_ZERO: &[&str] = &["--exact", "__no_test_matches_this_name__"];
/// An unrecognized option; the harness exits nonzero.
const EXIT_NONZERO: &[&str] = &["--not-a-real-harness-flag"];

fn sandboxed_plan(argv: Vec<String>, criteria: SettlementCriteria) -> CasePlan {
    CasePlan {
        version: "0.2".into(),
        plan_id: "plan_episode".into(),
        case_id: "case_placeholder".into(),
        run_id: "run_placeholder".into(),
        intent_id: "int_episode".into(),
        items: vec![PlanItem {
            plan_item_id: "task".into(),
            name: "sandboxed".into(),
            operations: vec![Operation::ExecuteCommand {
                argv,
                cwd: ".".into(),
            }],
            entry_criteria: vec![],
            entry_criteria_mode: Default::default(),
            exit_criteria: vec![],
            settlement_criteria: criteria,
            settlement_criteria_ref: None,
            item_kind: ItemKind::SandboxedTask,
            sandbox_class: None,
            parent_stage: None,
            markers: ItemMarkers {
                required: true,
                ..ItemMarkers::default()
            },
            max_instances: 1,
            depends_on: vec![],
            environment: None,
            proposed_by: None,
        }],
        template_ref: None,
        job_contract_ref: None,
    }
}

struct Episode {
    case_id: String,
    root: tempfile::TempDir,
}

impl Episode {
    fn entries(&self, kind: &str) -> Vec<serde_json::Value> {
        let ledger = sea_forge_ledger::LedgerStream::open(
            self.root.path(),
            format!("case-{}", self.case_id),
            "test",
        )
        .unwrap();
        ledger
            .read_entries()
            .unwrap()
            .into_iter()
            .filter(|entry| entry.record_kind == kind)
            .map(|entry| entry.payload)
            .collect()
    }

    /// The settlement this episode recorded, read back off the durable ledger
    /// rather than taken from the dispatcher's return value.
    fn settlement(&self) -> serde_json::Value {
        self.entries("settlement_event")
            .into_iter()
            .next()
            .expect("an episode must record a settlement_event")
    }

    fn basis(&self) -> Vec<String> {
        serde_json::from_value(self.settlement()["basis"].clone()).unwrap()
    }

    /// Included in failure messages: a settlement that says `authority_deny` is
    /// uninformative on its own — the reason codes say which rule refused.
    fn decision(&self) -> serde_json::Value {
        self.entries("authority_decision")
            .into_iter()
            .next()
            .unwrap_or(serde_json::Value::Null)
    }

    fn run_dirs(&self) -> Vec<PathBuf> {
        let runs = self
            .root
            .path()
            .join("cases")
            .join(&self.case_id)
            .join("runs");
        match fs::read_dir(&runs) {
            Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
            Err(_) => Vec::new(),
        }
    }
}

async fn dispatch(args: &[&str], criteria: SettlementCriteria, allow: bool) -> Episode {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), allow);
    let mut argv = vec![trusted_self_executable(root.path())
        .to_string_lossy()
        .into_owned()];
    argv.extend(args.iter().map(|arg| (*arg).to_owned()));

    let plan_path = root.path().join("plan.json");
    fs::write(
        &plan_path,
        serde_json::to_vec(&sandboxed_plan(argv, criteria)).unwrap(),
    )
    .unwrap();

    let state = Arc::new(
        ServerState::new(ServerConfig {
            socket_path: root.path().join("unused.sock"),
            root: root.path().to_path_buf(),
            ..ServerConfig::default()
        })
        .unwrap(),
    );
    let request: Request = serde_json::from_value(serde_json::json!({
        "verb": "submit",
        "plan": plan_path,
        "policy": policy_path,
        "entity": "operator_local",
        "process": "test",
        "timeout": 60,
    }))
    .unwrap();
    let response = handle_request(request, &state).await;
    let case_id = response["case_id"]
        .as_str()
        .unwrap_or_else(|| panic!("submit did not return a case_id: {response}"))
        .to_owned();
    Episode { case_id, root }
}

// ---------------------------------------------------------------------------
// AUTH-01: authority precedes every side effect
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_denied_episode_creates_no_workspace_and_no_artifacts() {
    let episode = dispatch(EXIT_ZERO, SettlementCriteria::default(), false).await;
    assert_eq!(episode.settlement()["status"], "rejected");

    // The run directory may exist as a record scaffold, but nothing the denied
    // operation would have produced may be there.
    for run_dir in episode.run_dirs() {
        assert!(
            !run_dir.join("workspace").exists(),
            "a denied episode created {}/workspace — a side effect with no committed Allow",
            run_dir.display()
        );
        assert!(
            !run_dir.join("artifacts").exists(),
            "a denied episode created {}/artifacts",
            run_dir.display()
        );
    }
}

/// "No side effect" means no *mutation*, not no record. A denial that left no
/// trace would be indistinguishable from an episode that never happened.
#[tokio::test]
async fn a_denied_episode_still_records_its_authority_decision() {
    let episode = dispatch(EXIT_ZERO, SettlementCriteria::default(), false).await;
    let decisions = episode.entries("authority_decision");
    assert!(
        !decisions.is_empty(),
        "a denial must still commit its authority decision"
    );
    assert_eq!(decisions[0]["verdict"], "deny");
}

// ---------------------------------------------------------------------------
// A denial is not a spawn failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_denial_is_settled_as_a_denial_not_a_spawn_failure() {
    let episode = dispatch(EXIT_ZERO, SettlementCriteria::default(), false).await;
    let basis = episode.basis();
    assert!(
        basis.iter().any(|entry| entry == "authority_deny"),
        "a denial must say so in its basis; got {basis:?}"
    );
    assert!(
        !basis.iter().any(|entry| entry == "spawn_failed"),
        "a denial must not be reported as a spawn that was attempted and failed; got {basis:?}"
    );
}

// ---------------------------------------------------------------------------
// Settlement reads criteria and evidence, not exit codes
// ---------------------------------------------------------------------------

/// The false-success case: the command succeeds and produces none of the
/// artifacts its own plan item declared. Exit-code settlement accepted this.
#[tokio::test]
async fn a_zero_exit_without_the_declared_artifact_is_rejected() {
    let episode = dispatch(
        EXIT_ZERO,
        SettlementCriteria {
            require_exit_zero: true,
            required_artifacts: vec!["proof.txt".into()],
            ..SettlementCriteria::default()
        },
        true,
    )
    .await;

    let basis = episode.basis();
    assert_eq!(
        episode.settlement()["status"],
        "rejected",
        "a command that exited zero while writing none of its declared artifacts \
         must not settle as accepted; basis {basis:?}; decision {}",
        episode.decision()
    );
    assert!(
        basis
            .iter()
            .any(|entry| entry == "required_artifact_missing:proof.txt"),
        "the settlement must name the missing artifact; got {basis:?}"
    );
    assert!(
        basis.iter().any(|entry| entry == "exit_zero"),
        "the exit code is still evidence, just not sufficient alone; got {basis:?}"
    );
}

/// The same wiring with its criteria satisfied. Guards against a "fix" that
/// simply rejects everything.
#[tokio::test]
async fn a_satisfied_criteria_set_is_accepted() {
    let episode = dispatch(
        EXIT_ZERO,
        SettlementCriteria {
            require_exit_zero: true,
            ..SettlementCriteria::default()
        },
        true,
    )
    .await;

    let basis = episode.basis();
    assert_eq!(
        episode.settlement()["status"],
        "accepted",
        "basis {basis:?}; decision {}",
        episode.decision()
    );
    assert!(
        basis.iter().any(|entry| entry == "exit_zero"),
        "got {basis:?}"
    );
}

#[tokio::test]
async fn a_nonzero_exit_is_rejected_with_the_reason_named() {
    let episode = dispatch(
        EXIT_NONZERO,
        SettlementCriteria {
            require_exit_zero: true,
            ..SettlementCriteria::default()
        },
        true,
    )
    .await;

    let basis = episode.basis();
    assert_eq!(
        episode.settlement()["status"],
        "rejected",
        "basis {basis:?}; decision {}",
        episode.decision()
    );
    assert!(
        basis.iter().any(|entry| entry == "exit_nonzero"),
        "got {basis:?}"
    );
}

/// Every episode settles through the same `sea_forge_settlement::settle` path
/// as every other surface, so its basis always opens with the authority
/// verdict rather than a dispatcher-local string.
#[tokio::test]
async fn every_settlement_records_the_authority_verdict_first() {
    let allowed = dispatch(EXIT_ZERO, SettlementCriteria::default(), true).await;
    assert_eq!(
        allowed.basis().first().map(String::as_str),
        Some("authority_allow"),
        "basis {:?}; decision {}",
        allowed.basis(),
        allowed.decision()
    );

    let denied = dispatch(EXIT_ZERO, SettlementCriteria::default(), false).await;
    assert_eq!(
        denied.basis().first().map(String::as_str),
        Some("authority_deny"),
        "basis {:?}",
        denied.basis()
    );
}
