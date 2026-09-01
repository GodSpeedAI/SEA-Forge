//! One-shot cell bootstrap for the UX Journey Settlement Gauntlet
//! (`.agents/reports/ux-journey-settlement/harness/gauntlet_runner.py`).
//!
//! The gauntlet drives the real Workbench frontend against a real,
//! unmodified `sea-forge-server` binary — not a mock of the Tauri IPC
//! bridge. A `sea-forge-server` process refuses every protected verb until
//! `identity.bindings` is configured (`identity.rs`'s `Unconfigured`
//! refusal), and `case.entry_options` finds templates only under
//! `<root>/templates/` (`sfwp/case.rs`) — so a fresh root needs a
//! `server.yaml`, a template file, and a `policy.yaml` before the real
//! server binary is worth starting.
//!
//! This tool writes exactly that, using the same real types
//! (`ServerConfig`, `IdentityBindings`, `sequential_agents_template`) the
//! server itself loads — not hand-authored YAML that could drift from the
//! real schema — then exits. The harness starts the actual
//! `sea-forge-server` binary against the root this produces.
//!
//! Run: `cargo run -p sea-forge-server --example journey_gauntlet_bootstrap`
//! Env:
//!   SEA_FORGE_ROOT                    (required) empty/fresh cell root
//!   SEA_FORGE_GAUNTLET_ENDPOINT_URL   (required) base URL of a running
//!                                     OpenAI-compatible stub endpoint the
//!                                     harness keeps alive for the life of
//!                                     the run (see
//!                                     harness/stub_agent_endpoint.py)

use sea_forge_agent::{AgentConfig, AgentEndpointConfig, ProviderKind};
use sea_forge_core::types::ActorRole;
use sea_forge_planner::templates::sequential_agents_template;
use sea_forge_server::config::SOCKET_FILE_NAME;
use sea_forge_server::identity::{IdentityBinding, IdentityBindings};
use sea_forge_server::ServerConfig;
use std::path::PathBuf;

/// The one real, built-in template this gauntlet exercises end to end
/// (case.entry_options -> case.preflight -> case.commit -> dispatch ->
/// approval escalation -> execution). Proven by
/// `crates/sea-forge-server/tests/conformance_case_authoring.rs`.
const TEMPLATE_REF: &str = "sequential_agents@0.1.0";
const ENDPOINT_ID: &str = "gauntlet-stub-endpoint";

/// Actor bound to submit/commit work. Distinct actor_id (not merely a
/// distinct connection) from `APPROVER_ACTOR_ID` — separation of duty
/// (`identity.rs::IdentityRefusal::SelfApproval`) compares the approver's
/// actor_id against the submitter recorded in the ledger, so two actor_ids
/// bound to the same local uid is sufficient; it does not require two OS
/// users.
const OPERATOR_ACTOR_ID: &str = "gauntlet_operator";
/// Actor bound to decide approvals. Role choice (`SecurityOfficer`) is a
/// reasonable stand-in for "eligible approver" — verify against the real
/// approval eligibility computation (`sfwp/approvals.rs`) the first time
/// this harness actually runs CJ06 against the real server, and adjust here
/// if eligibility turns out to require a different bound role.
const APPROVER_ACTOR_ID: &str = "gauntlet_approver";

fn env_required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        eprintln!("journey_gauntlet_bootstrap: missing required env var {name}");
        std::process::exit(1);
    })
}

fn main() {
    let root = PathBuf::from(env_required("SEA_FORGE_ROOT"));
    let endpoint_url = env_required("SEA_FORGE_GAUNTLET_ENDPOINT_URL");

    if root.exists() && root.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false) {
        eprintln!(
            "journey_gauntlet_bootstrap: {} is not empty; refuse to bootstrap over an existing cell",
            root.display()
        );
        std::process::exit(1);
    }
    std::fs::create_dir_all(&root).expect("create SEA_FORGE_ROOT");

    let endpoint = AgentEndpointConfig {
        id: ENDPOINT_ID.into(),
        kind: ProviderKind::OpenAiCompatible,
        base_url: Some(endpoint_url),
        argv: vec![],
        env: vec![],
        credential_ref: None,
        default_model: Some("gauntlet-stub-model".into()),
        allow_loopback_test: true,
        max_request_bytes: 16_384,
        max_response_bytes: 16_384,
        timeout_secs: 10,
        status: None,
        transcript_retention: None,
    };

    let identity = IdentityBindings {
        bindings: [OPERATOR_ACTOR_ID, APPROVER_ACTOR_ID]
            .into_iter()
            .filter_map(|actor_id| {
                sea_forge_server::identity::current_uid().map(|uid| IdentityBinding {
                    uid,
                    actor_id: actor_id.into(),
                    roles: vec![if actor_id == OPERATOR_ACTOR_ID {
                        ActorRole::Operator
                    } else {
                        ActorRole::SecurityOfficer
                    }],
                })
            })
            .collect(),
    };

    let config = ServerConfig {
        identity,
        socket_path: root.join(SOCKET_FILE_NAME),
        root: root.clone(),
        agent: AgentConfig {
            endpoints: vec![endpoint],
            ..AgentConfig::default()
        },
        ..ServerConfig::default()
    };

    let templates_dir = root.join("templates");
    std::fs::create_dir_all(&templates_dir).expect("create templates dir");
    let template = sequential_agents_template(ENDPOINT_ID)
        .expect("sequential_agents_template must accept a valid endpoint id");
    std::fs::write(
        templates_dir.join(format!("{TEMPLATE_REF}.yaml")),
        serde_yaml::to_string(&template).expect("serialize template"),
    )
    .expect("write template file");

    // Escalate agent_task rather than allow it outright: the gauntlet's CJ06
    // ("Resolve Human Judgment and Approval") needs a real pending approval
    // to decide, and CJ07/CJ09 then observe real post-approval dispatch —
    // matching case_dispatch.rs's real escalate-on-Verdict::Escalate path
    // (case_dispatch.rs ~L807) rather than a fabricated approval record.
    let policy_yaml = "version: \"0.1\"\n\
policy_surfaces:\n\
  external_api:\n\
    mode: deny-by-default\n\
    allow_hosts: [127.0.0.1]\n\
rules:\n\
  - name: escalate-agent-task-for-human-approval\n\
    verdict: escalate\n\
    actor_role: operator\n\
    operation_kind: agent_task\n";
    std::fs::write(root.join("policy.yaml"), policy_yaml).expect("write policy.yaml");

    std::fs::write(
        root.join("server.yaml"),
        serde_yaml::to_string(&config).expect("serialize ServerConfig"),
    )
    .expect("write server.yaml");

    println!(
        "journey_gauntlet_bootstrap: wrote server.yaml, {TEMPLATE_REF}.yaml, and policy.yaml under {}",
        root.display()
    );
}
