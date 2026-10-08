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

/// A policy carrying one `execute_command` rule with the given verdict, or —
/// for `None` — no rules at all, which denies by default.
///
/// `argv0` must be `sea-forge`: `AuthorityPolicyBundle::validate` refuses a
/// *local* allow for `execute_command` with any other argv0 as a
/// `schema_error`, so policy cannot hand out local shell execution.
fn policy(root: &Path, verdict: Option<&str>) -> PathBuf {
    let rules = match verdict {
        Some(verdict) => format!(
            "  - name: {verdict}-command\n    verdict: {verdict}\n    actor_role: operator\n    operation_kind: execute_command\n    argv0: sea-forge\n"
        ),
        None => String::new(),
    };
    let path = root.join(format!("{}.yaml", verdict.unwrap_or("deny")));
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

    /// Every record of one kind from every run directory this case produced.
    fn jsonl(&self, file: &str) -> Vec<serde_json::Value> {
        self.run_dirs()
            .into_iter()
            .filter_map(|dir| fs::read_to_string(dir.join(file)).ok())
            .flat_map(|text| {
                text.lines()
                    .map(|line| serde_json::from_str(line).expect("well-formed jsonl"))
                    .collect::<Vec<serde_json::Value>>()
            })
            .collect()
    }

    fn trace_kinds(&self) -> Vec<String> {
        self.jsonl("trace.jsonl")
            .into_iter()
            .map(|event| event["kind"].as_str().unwrap_or_default().to_owned())
            .collect()
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
    dispatch_under(args, criteria, allow.then_some("allow")).await
}

async fn dispatch_under(
    args: &[&str],
    criteria: SettlementCriteria,
    verdict: Option<&str>,
) -> Episode {
    let root = tempfile::tempdir().unwrap();
    let policy_path = policy(root.path(), verdict);
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
        // F-16: plan/policy references are cell-relative spellings.
        "plan": "plan.json",
        "policy": policy_path.strip_prefix(root.path()).unwrap().to_string_lossy(),
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

// ---------------------------------------------------------------------------
// DOM-02: the episode leaves a trace and evidence, not just a settlement
// ---------------------------------------------------------------------------

/// A settlement on its own says what was decided but not how. The CLI has
/// always written `trace.jsonl` and `evidence.jsonl` beside it
/// (`cli/src/pipeline.rs:383-394`); the dispatcher wrote neither, so a case
/// submitted over the socket produced an unauditable run that still looked
/// complete in the case view.
#[tokio::test]
async fn an_allowed_episode_records_its_trace_and_evidence() {
    let episode = dispatch(EXIT_ZERO, SettlementCriteria::default(), true).await;
    let kinds = episode.trace_kinds();
    for expected in [
        "authority_evaluated",
        "workspace_created",
        "command_started",
        "command_finished",
    ] {
        assert!(
            kinds.iter().any(|kind| kind == expected),
            "trace is missing {expected}; got {kinds:?}"
        );
    }
    // Ordering carries the invariant: authority is evaluated before the
    // workspace exists, not merely alongside it.
    let position = |needle: &str| kinds.iter().position(|kind| kind == needle);
    assert!(position("authority_evaluated") < position("workspace_created"));

    let evidence = episode.jsonl("evidence.jsonl");
    let kinds: Vec<&str> = evidence
        .iter()
        .map(|record| record["kind"].as_str().unwrap_or_default())
        .collect();
    assert!(
        kinds.contains(&"authority_decision") && kinds.contains(&"execution_result"),
        "evidence kinds {kinds:?}"
    );
    // Evidence points back at the trace event that produced it, or the two
    // files are parallel logs rather than one linked record.
    let trace_ids: Vec<String> = episode
        .jsonl("trace.jsonl")
        .into_iter()
        .map(|event| event["event_id"].as_str().unwrap_or_default().to_owned())
        .collect();
    for record in &evidence {
        let source = record["source_event_id"].as_str().unwrap_or_default();
        assert!(
            trace_ids.iter().any(|id| id == source),
            "evidence {} cites {source}, which is not in the trace {trace_ids:?}",
            record["evidence_id"]
        );
    }
}

/// A denial is still an episode. Its trace has to show that authority was
/// evaluated and the run stopped — otherwise the only durable difference
/// between "denied" and "never dispatched" is a settlement row.
#[tokio::test]
async fn a_denied_episode_records_why_it_stopped() {
    let episode = dispatch(EXIT_ZERO, SettlementCriteria::default(), false).await;
    let kinds = episode.trace_kinds();
    assert!(
        kinds.iter().any(|kind| kind == "authority_evaluated")
            && kinds.iter().any(|kind| kind == "run_halted"),
        "trace {kinds:?}"
    );
    assert!(
        !kinds.iter().any(|kind| kind == "command_started"),
        "a denied episode traced a command start: {kinds:?}"
    );
}

/// The ledger is the truth, but `run.get` and `case.get_overview` read
/// `settlement.json` and `authority.json` off the run directory. A settlement
/// that lived only in the ledger made every case-dispatched run render as
/// `"unsettled"` — including runs that had been denied *and* settled, which is
/// the one thing those views must never say. Caught by driving a real server
/// over its socket, not by any test that existed at the time.
#[tokio::test]
async fn a_settled_episode_materializes_the_records_the_views_read() {
    for allow in [true, false] {
        let episode = dispatch(EXIT_ZERO, SettlementCriteria::default(), allow).await;
        let expected = episode.settlement();

        for run_dir in episode.run_dirs() {
            let settlement: serde_json::Value = serde_json::from_slice(
                &fs::read(run_dir.join("settlement.json")).unwrap_or_else(|_| {
                    panic!(
                        "no settlement.json in {} (allow={allow})",
                        run_dir.display()
                    )
                }),
            )
            .expect("settlement.json is well-formed");
            // The projection must be the settlement, not a second opinion
            // about it.
            assert_eq!(settlement["status"], expected["status"]);
            assert_eq!(settlement["settlement_id"], expected["settlement_id"]);
            assert_eq!(settlement["basis"], expected["basis"]);

            let authority: serde_json::Value =
                serde_json::from_slice(&fs::read(run_dir.join("authority.json")).unwrap_or_else(
                    |_| panic!("no authority.json in {} (allow={allow})", run_dir.display()),
                ))
                .expect("authority.json is well-formed");
            assert_eq!(
                authority[0]["decision_id"],
                episode.decision()["decision_id"],
                "authority.json must name the decision the ledger committed"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Escalate: a question for a human, not a rejection
// ---------------------------------------------------------------------------

/// `Escalate` used to reach settlement as an ordinary rejection, which
/// discarded the review the verdict was asking for: nobody could approve
/// something that had already been recorded as refused.
#[tokio::test]
async fn an_escalated_episode_opens_an_approval_and_runs_nothing() {
    let episode = dispatch_under(EXIT_ZERO, SettlementCriteria::default(), Some("escalate")).await;
    assert_eq!(
        episode.settlement()["status"],
        "escalated",
        "decision {}",
        episode.decision()
    );
    assert_eq!(episode.settlement()["review_required"], true);

    let approvals = episode.entries("approval_request");
    assert_eq!(approvals.len(), 1, "approvals {approvals:?}");
    assert_eq!(approvals[0]["status"], "pending");
    assert_eq!(
        approvals[0]["decision_id"],
        episode.decision()["decision_id"],
        "the approval must name the decision that escalated"
    );
    assert!(
        episode.root.path().join("approvals.jsonl").exists(),
        "an escalation must reach the operator-visible approvals view"
    );

    // The inbox projection must resolve its explanation to the same committed
    // approval request and authority decision — no source file citation or
    // renderer-authored reason can stand in for this chain.
    let inbox = sea_forge_server::sfwp::approvals::list(episode.root.path(), None);
    let context = inbox.approvals[0]
        .governance
        .as_ref()
        .expect("an escalated ledger approval must project committed governance context");
    assert_eq!(context.approval_source.record_kind, "approval_request");
    assert_eq!(context.decision_source.record_kind, "authority_decision");
    assert_eq!(
        context.decision_source.record_id,
        episode.decision()["decision_id"].as_str().unwrap()
    );
    assert!(context.approval_source.digest.starts_with("sha256:"));
    assert_eq!(
        context.side_effect_standing,
        "not_executed_pending_approval"
    );
    assert_eq!(
        context
            .purpose_context
            .get("plan_item_id")
            .and_then(serde_json::Value::as_str),
        Some("task"),
        "purpose context must be the committed authority request, not a UI-authored summary"
    );
    assert!(context.eligible_actors.is_empty());
    assert_eq!(context.eligibility_standing, "resolved_per_connection");

    // Pending review means nothing ran.
    for run_dir in episode.run_dirs() {
        assert!(!run_dir.join("workspace").exists());
        assert!(!run_dir.join("artifacts").exists());
    }
}
