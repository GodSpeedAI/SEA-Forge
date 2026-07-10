#![forbid(unsafe_code)]

mod commands;

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
        #[arg(long)]
        intent: String,
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
    Validate { file: PathBuf },
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
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    Inspect {
        run_id: String,
        #[arg(long, default_value = ".sea-forge")]
        root: PathBuf,
    },
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
            tracing::error!(event="command_failed",run_id="none",component="sea-forge-cli",error_class=error.class(),message=%error);
            ExitCode::from(code)
        }
    }
}
fn dispatch(cli: Cli) -> Result<u8, (u8, sea_forge_core::ForgeError)> {
    match cli.command {
        Command::Validate { file } => Ok(commands::validate::execute(&file)),
        Command::Run {
            intent,
            policy,
            root,
            timeout,
            entity,
            process,
        } => commands::run::execute(intent, policy, root, timeout, entity, process).map_err(|e| {
            let code = if matches!(
                e,
                sea_forge_core::ForgeError::Input(_) | sea_forge_core::ForgeError::UnknownIntent(_)
            ) {
                2
            } else {
                1
            };
            (code, e)
        }),
        Command::Recall {
            query,
            root,
            entity,
            process,
            result,
            limit,
        } => commands::recall::execute(
            &root,
            &query,
            entity.as_deref(),
            process.as_deref(),
            result.map(Into::into),
            limit,
        )
        .map_err(|e| (1, e)),
        Command::Inspect { run_id, root } => {
            commands::inspect::execute(&root, &run_id).map_err(|e| (1, e))
        }
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
