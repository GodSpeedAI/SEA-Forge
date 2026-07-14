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
enum MemoryCommand {
    /// Rebuild the memory FTS index from items.jsonl (§10.5).
    Rebuild,
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
            policy: _,
            actor,
            note,
        } => {
            let policy = commands::mediated::policy_path(&root, None);
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
            policy: _,
            actor,
            note,
        } => {
            let policy = commands::mediated::policy_path(&root, None);
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
        Command::Memory { action, root } => match action {
            MemoryCommand::Rebuild => commands::memory::rebuild(&root)
                .map(|_| 0)
                .map_err(|e| (1, e)),
        },
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
