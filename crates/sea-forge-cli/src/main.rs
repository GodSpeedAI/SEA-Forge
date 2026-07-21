#![forbid(unsafe_code)]

mod approvals;
mod commands;
mod pipeline;
mod plan_pipeline;

use clap::{Parser, Subcommand, ValueEnum};
use sea_forge_core::types::SettlementStatus;
use std::{path::PathBuf, process::ExitCode};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "sea-forge",
    version,
    about = "Governed capability-execution kernel"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        #[arg(long, required_unless_present = "plan", conflicts_with = "plan")]
        intent: Option<String>,
        #[arg(long, required_unless_present = "intent", conflicts_with = "intent")]
        plan: Option<PathBuf>,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: PathBuf,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long, default_value_t = 60)]
        timeout: u64,
        #[arg(long, default_value = "operator_local")]
        entity: String,
        #[arg(long, default_value = "cli")]
        process: String,
    },
    Runs {
        #[arg(long)]
        unsettled: bool,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
    #[command(hide = true)]
    Validate {
        file: PathBuf,
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
    #[command(hide = true)]
    InternalTestSleep { seconds: u64 },
    #[command(hide = true)]
    InternalTestSweSeed {
        #[arg(long)]
        fail: bool,
        #[arg(long)]
        delay: Option<u64>,
        #[arg(long)]
        outage_file: Option<PathBuf>,
    },
    Recall {
        query: String,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        entity: Option<String>,
        #[arg(long)]
        process: Option<String>,
        #[arg(long)]
        result: Option<ResultArg>,
        /// When set, search memory/items.jsonl (MemoryItem) instead of
        /// capabilities.jsonl (SemanticEnvelope). §10.5 M4b upgrade.
        #[arg(long)]
        kind: Option<MemoryKindArg>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    Inspect {
        run_id: String,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        entity: String,
    },
    Migrate {
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long)]
        key_dir: Option<PathBuf>,
        #[arg(long, default_value = "migration")]
        key_id: String,
    },
    Case {
        #[command(subcommand)]
        action: CaseCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    Task {
        #[command(subcommand)]
        action: TaskCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    Ledger {
        #[command(subcommand)]
        action: LedgerAction,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
    Approve {
        case_id: String,
        approval_id: String,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        actor: String,
        #[arg(long)]
        note: Option<String>,
    },
    Reject {
        case_id: String,
        approval_id: String,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        actor: String,
        #[arg(long)]
        note: Option<String>,
    },
    Resume {
        case_id: String,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: PathBuf,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long, default_value_t = 60)]
        timeout: u64,
        #[arg(long, default_value = "operator_local")]
        entity: String,
        #[arg(long, default_value = "cli")]
        process: String,
    },
    Memory {
        #[command(subcommand)]
        action: MemoryCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
    /// List or show EnvironmentSpecs (§7.6).
    Env {
        #[command(subcommand)]
        action: EnvCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
    /// Export selected runs (and optionally templates) to a federation bundle.
    Export {
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: PathBuf,
        #[arg(long, default_value = "operator_local")]
        actor: String,
        #[arg(long, num_args = 1.., required = true)]
        run_ids: Vec<String>,
        #[arg(long, num_args = 0..)]
        templates: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },
    /// Import a federation bundle.
    Import {
        bundle: PathBuf,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: PathBuf,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    /// Adopt an imported template (§10.6).
    Adopt {
        cell_id: String,
        reference: String,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: PathBuf,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    Artifact {
        #[command(subcommand)]
        action: ArtifactCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    /// Genesis self-model (spec-adlc-thoth E11): validate, rebuild, show.
    SelfModel {
        #[command(subcommand)]
        action: SelfModelCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
    /// Ask Thoth a typed question about the self-model (spec-adlc-thoth E13).
    Ask {
        /// Question kind (§7.4).
        kind: String,
        /// Subject (capability name, concept ref, etc.).
        subject: String,
        #[arg(long, default_value = "planning")]
        purpose: String,
        #[arg(long)]
        case: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
        #[arg(long, default_value = "operator_local")]
        actor: String,
    },
    /// Governed HTTP agent connectivity (M12).
    Agent {
        #[command(subcommand)]
        action: AgentCommand,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
}

#[derive(Subcommand)]
enum LedgerAction {
    Verify {
        ledger_id: String,
    },
    Prove {
        ledger_id: String,
        entry_ulid: String,
    },
    /// Reproduce the persisted dispatch/settlement order for a case
    /// (spec §17.2 T13.2). Reads `case-events.jsonl` only; never re-executes.
    Replay {
        #[arg(long)]
        case: String,
    },
}

#[derive(Subcommand)]
enum CaseCommand {
    Reopen {
        case_id: String,
    },
    AddTask {
        case_id: String,
        #[arg(long)]
        item: PathBuf,
    },
}

#[derive(Subcommand)]
enum TaskCommand {
    Complete {
        case_id: String,
        item_id: String,
        #[arg(long)]
        note: Option<String>,
    },
}

#[derive(Subcommand)]
enum ArtifactCommand {
    Synthesize {
        input: PathBuf,
    },
    Productize {
        input: PathBuf,
    },
    Capitalize {
        input: PathBuf,
    },
    Attest {
        artifact_id: String,
        #[arg(long, default_value = "ifl")]
        ledger: String,
        #[arg(long, default_value = "forbidden")]
        degraded_mode: String,
        #[arg(long, default_value = "operator")]
        requester_role: String,
        #[arg(long, default_value = "R-SO")]
        approver_role: String,
        #[arg(long = "degraded-control")]
        degraded_controls: Vec<String>,
    },
}

#[derive(Subcommand)]
enum MemoryCommand {
    /// Rebuild the memory FTS index from items.jsonl (§10.5).
    Rebuild,
}

#[derive(Subcommand)]
enum SelfModelCommand {
    /// Verify bundled models + current snapshot + projections.
    Validate,
    /// Build a new self-model snapshot (init-or-upgrade, then rebuild).
    Rebuild {
        #[arg(long)]
        probe: bool,
        #[arg(long, default_value = "sha256:capability-projection")]
        capability_hash: String,
    },
    /// Show the newest snapshot (composed-view summary, or full JSON).
    Show {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum EnvCommand {
    /// List available EnvironmentSpecs.
    List,
    /// Show a specific EnvironmentSpec.
    Show { reference: String },
}

#[derive(Subcommand)]
enum AgentCommand {
    List,
    Probe {
        endpoint: String,
        prompt: String,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: String,
        #[arg(long, default_value = "operator_local")]
        entity: String,
        #[arg(long, default_value = "cli")]
        process: String,
    },
    /// Delegate a task to an agent endpoint (M13).
    Delegate {
        endpoint: String,
        instruction: String,
        /// Stable run identity required to cancel this delegation while in flight.
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 1)]
        max_turns: u32,
        #[arg(long)]
        token_budget: Option<u64>,
        /// Literal the agent's final response must contain for acceptance.
        #[arg(long)]
        agent_output_must_contain: Option<String>,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: String,
        #[arg(long, default_value = "operator_local")]
        entity: String,
        #[arg(long, default_value = "cli")]
        process: String,
    },
    /// Request cancellation of an active delegation.
    Cancel {
        run_id: String,
        #[arg(long, default_value = "sea-forge-policy.yaml")]
        policy: String,
        #[arg(long, default_value = "operator_local")]
        entity: String,
        #[arg(long, default_value = "cli")]
        process: String,
    },
}

#[derive(Clone, ValueEnum)]
enum MemoryKindArg {
    Fact,
    Decision,
    Outcome,
    Preference,
}

impl From<MemoryKindArg> for sea_forge_core::types::MemoryKind {
    fn from(v: MemoryKindArg) -> Self {
        match v {
            MemoryKindArg::Fact => Self::Fact,
            MemoryKindArg::Decision => Self::Decision,
            MemoryKindArg::Outcome => Self::Outcome,
            MemoryKindArg::Preference => Self::Preference,
        }
    }
}

#[derive(Clone, ValueEnum)]
enum ResultArg {
    Accepted,
    Rejected,
    Escalated,
}
impl From<ResultArg> for SettlementStatus {
    fn from(v: ResultArg) -> Self {
        match v {
            ResultArg::Accepted => Self::Accepted,
            ResultArg::Rejected => Self::Rejected,
            ResultArg::Escalated => Self::Escalated,
        }
    }
}

fn main() -> ExitCode {
    init_diagnostics();
    match dispatch(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err((code, error)) => {
            let run_id = error.run_id().unwrap_or("none");
            tracing::error!(event="command_failed",run_id=run_id,component="sea-forge-cli",error_class=error.class(),message=%error);
            ExitCode::from(code)
        }
    }
}
fn dispatch(cli: Cli) -> Result<u8, (u8, sea_forge_core::ForgeError)> {
    match cli.command {
        Command::Validate { file, root, policy } => {
            let authority_root = root.unwrap_or_else(|| {
                file.parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join(".sea-forge")
            });
            commands::validate::execute(&file, &authority_root, policy.as_deref())
                .map_err(|error| (1, error))
        }
        Command::InternalTestSleep { seconds } => {
            std::thread::sleep(std::time::Duration::from_secs(seconds));
            Ok(0)
        }
        Command::InternalTestSweSeed {
            fail,
            delay,
            outage_file,
        } => {
            use std::io::Read;
            if let Some(seconds) = delay {
                std::thread::sleep(std::time::Duration::from_secs(seconds));
            }
            let mut request = Vec::new();
            std::io::stdin()
                .read_to_end(&mut request)
                .map_err(|error| {
                    (
                        1,
                        sea_forge_core::ForgeError::io("read SWE_SEED request", error),
                    )
                })?;
            serde_json::from_slice::<sea_forge_core::types::SettlementDeclarationRequest>(&request)
                .map_err(|error| {
                    (
                        1,
                        sea_forge_core::ForgeError::Serialization(error.to_string()),
                    )
                })?;
            if fail || outage_file.is_some_and(|path| path.exists()) {
                return Ok(1);
            }
            println!(
                "{}",
                serde_json::to_string(&sea_forge_settlement::SweSeedResponse {
                    attestation_ref: "attestation:test-swe-seed".into(),
                    attribution_confidence: "1.000000".into(),
                    gaming_exposure: "0.000000".into(),
                    hidden_debt_blindness: "0.000000".into(),
                    feedback_delay_ms: 1,
                })
                .map_err(|error| (
                    1,
                    sea_forge_core::ForgeError::Serialization(error.to_string())
                ))?
            );
            Ok(0)
        }
        Command::Run {
            intent,
            plan,
            policy,
            root,
            timeout,
            entity,
            process,
        } => commands::run::execute(intent, plan, policy, root, timeout, entity, process).map_err(
            |e| {
                let code = if matches!(
                    e,
                    sea_forge_core::ForgeError::Input(_)
                        | sea_forge_core::ForgeError::UnknownIntent(_)
                        | sea_forge_core::ForgeError::Plan { .. }
                ) {
                    2
                } else {
                    1
                };
                (code, e)
            },
        ),
        Command::Recall {
            query,
            root,
            entity,
            process,
            result,
            kind,
            limit,
            policy,
            actor,
        } => commands::recall::execute(commands::recall::RecallOptions {
            root: &root,
            policy: policy.as_deref(),
            actor_id: &actor,
            query: &query,
            entity: entity.as_deref(),
            process: process.as_deref(),
            result: result.map(Into::into),
            kind: kind.map(Into::into),
            limit,
        })
        .map_err(|e| (1, e)),
        Command::Inspect {
            run_id,
            root,
            policy,
            entity,
        } => commands::inspect::execute(&root, policy.as_deref(), &entity, &run_id)
            .map_err(|e| (1, e)),
        Command::Migrate {
            root,
            policy,
            key_dir,
            key_id,
        } => commands::migrate::execute(commands::migrate::MigrateOptions {
            root: &root,
            policy: policy.as_deref(),
            key_dir: key_dir.as_deref(),
            key_id: &key_id,
        })
        .map_err(|e| (1, e)),
        Command::Case {
            action,
            root,
            policy,
            actor,
        } => {
            let policy = commands::mediated::policy_path(&root, policy.as_deref());
            match action {
                CaseCommand::Reopen { case_id } => {
                    commands::case::reopen(&root, &policy, &actor, &case_id)
                }
                CaseCommand::AddTask { case_id, item } => {
                    commands::case::add_task(&root, &policy, &actor, &case_id, &item)
                }
            }
            .map_err(|error| (1, error))
        }
        Command::Task {
            action,
            root,
            policy,
            actor,
        } => {
            let policy = commands::mediated::policy_path(&root, policy.as_deref());
            match action {
                TaskCommand::Complete {
                    case_id,
                    item_id,
                    note,
                } => commands::task::complete(
                    &root,
                    &policy,
                    &actor,
                    &case_id,
                    &item_id,
                    note.as_deref(),
                ),
            }
            .map_err(|error| (1, error))
        }
        Command::Ledger { action, root } => {
            commands::ledger::execute(action, &root).map_err(|e| (1, e))
        }
        Command::Approve {
            case_id,
            approval_id,
            root,
            policy,
            actor,
            note,
        } => {
            let policy = commands::mediated::policy_path(&root, policy.as_deref());
            commands::approve::approve(commands::approve::ApproveOptions {
                root: &root,
                case_id: &case_id,
                approval_id: &approval_id,
                actor: &actor,
                note: note.as_deref(),
                policy: Some(&policy),
            })
            .map_err(|e| (1, e))
        }
        Command::Reject {
            case_id,
            approval_id,
            root,
            policy,
            actor,
            note,
        } => {
            let policy = commands::mediated::policy_path(&root, policy.as_deref());
            commands::approve::reject(commands::approve::ApproveOptions {
                root: &root,
                case_id: &case_id,
                approval_id: &approval_id,
                actor: &actor,
                note: note.as_deref(),
                policy: Some(&policy),
            })
            .map_err(|e| (1, e))
        }
        Command::Resume {
            case_id,
            policy,
            root,
            timeout,
            entity,
            process,
        } => commands::resume::resume(commands::resume::ResumeOptions {
            root,
            case_id,
            policy,
            timeout_secs: timeout,
            entity,
            process,
        })
        .map(|outcome| {
            println!("case_id={}", outcome.case_id);
            println!("case_state={}", outcome.state);
            outcome.exit_code
        })
        .map_err(|e| (1, e)),
        Command::Runs { unsettled, root } => {
            commands::runs::list(&root, unsettled).map_err(|e| (1, e))
        }
        Command::Memory { action, root } => match action {
            MemoryCommand::Rebuild => commands::memory::rebuild(&root)
                .map(|_| 0)
                .map_err(|e| (1, e)),
        },
        Command::Env { action, root } => match action {
            EnvCommand::List => commands::env::list(&root).map_err(|e| (1, e)),
            EnvCommand::Show { reference } => {
                commands::env::show(&root, &reference).map_err(|e| (1, e))
            }
        },
        Command::Export {
            root,
            policy,
            actor,
            run_ids,
            templates,
            out,
        } => commands::federation::export(commands::federation::ExportOptions {
            root: &root,
            policy: Some(&policy),
            actor: &actor,
            run_ids: &run_ids,
            templates: &templates,
            out: &out,
        })
        .map_err(|e| (1, e)),
        Command::Import {
            bundle,
            root,
            policy,
            actor,
        } => commands::federation::import(commands::federation::ImportOptions {
            root: &root,
            policy: Some(&policy),
            actor: &actor,
            bundle: &bundle,
        })
        .map_err(|e| (1, e)),
        Command::Adopt {
            cell_id,
            reference,
            root,
            policy,
            actor,
        } => commands::federation::adopt(commands::federation::AdoptOptions {
            root: &root,
            policy: Some(&policy),
            actor: &actor,
            cell_id: &cell_id,
            reference: &reference,
        })
        .map_err(|e| (1, e)),
        Command::Artifact {
            action,
            root,
            policy,
            actor,
        } => {
            let policy = commands::mediated::policy_path(&root, policy.as_deref());
            match action {
                ArtifactCommand::Synthesize { input } => commands::artifact::transition_command(
                    &root,
                    &policy,
                    &actor,
                    sea_forge_artifact_ip::TransitionKind::Synthesize,
                    &input,
                ),
                ArtifactCommand::Productize { input } => commands::artifact::transition_command(
                    &root,
                    &policy,
                    &actor,
                    sea_forge_artifact_ip::TransitionKind::Productize,
                    &input,
                ),
                ArtifactCommand::Capitalize { input } => commands::artifact::transition_command(
                    &root,
                    &policy,
                    &actor,
                    sea_forge_artifact_ip::TransitionKind::Capitalize,
                    &input,
                ),
                ArtifactCommand::Attest {
                    artifact_id,
                    ledger,
                    degraded_mode,
                    requester_role,
                    approver_role,
                    degraded_controls,
                } => commands::artifact::attest_command(commands::artifact::AttestOptions {
                    root: &root,
                    policy: &policy,
                    actor: &actor,
                    artifact_id: &artifact_id,
                    ledger: &ledger,
                    degraded_mode: &degraded_mode,
                    requester_role: &requester_role,
                    approver_role: &approver_role,
                    degraded_controls,
                }),
            }
            .map_err(|e| (1, e))
        }
        Command::SelfModel { action, root } => match action {
            SelfModelCommand::Validate => commands::self_model::validate(&root),
            SelfModelCommand::Rebuild {
                probe,
                capability_hash,
            } => commands::self_model::rebuild(&root, probe, &capability_hash),
            SelfModelCommand::Show { json } => commands::self_model::show(&root, json),
        }
        .map_err(|e| (1, e)),
        Command::Ask {
            kind,
            subject,
            purpose,
            case,
            json,
            root,
            actor,
        } => commands::ask::run(
            &kind,
            &subject,
            &purpose,
            case.as_deref(),
            json,
            &root,
            &actor,
        )
        .map_err(|e| (1, e)),
        Command::Agent { action, root } => match action {
            AgentCommand::List => commands::agent::list(&root).map(|_| 0),
            AgentCommand::Probe {
                endpoint,
                prompt,
                model,
                policy,
                entity,
                process,
            } => commands::agent::probe(
                &root,
                &endpoint,
                &prompt,
                model.as_deref(),
                &policy,
                &entity,
                &process,
            ),
            AgentCommand::Delegate {
                endpoint,
                instruction,
                run_id,
                model,
                max_turns,
                token_budget,
                agent_output_must_contain,
                policy,
                entity,
                process,
            } => commands::agent::delegate(
                &root,
                &endpoint,
                &instruction,
                run_id.as_deref(),
                model.as_deref(),
                max_turns,
                token_budget,
                agent_output_must_contain.as_deref(),
                &policy,
                &entity,
                &process,
            ),
            AgentCommand::Cancel {
                run_id,
                policy,
                entity,
                process,
            } => commands::agent::cancel(&root, &run_id, &policy, &entity, &process),
        }
        .map_err(|e| (1, e)),
    }
}
fn init_diagnostics() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .with_current_span(false)
        .with_span_list(false)
        .init();
}
