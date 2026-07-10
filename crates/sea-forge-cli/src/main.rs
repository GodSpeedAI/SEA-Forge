//! SEA Forge CLI entry point.
//!
//! Foundation skeleton. The `run`, `validate`, `inspect`, and `recall`
//! subcommands are implemented by the minimum spec; until then the binary
//! reports that the minimum slice is not yet implemented.

#![forbid(unsafe_code)]

use std::process::ExitCode;
use tracing_subscriber::EnvFilter;

const NOT_IMPLEMENTED: &str = "sea-forge: minimum slice not implemented yet \
                               (see .agents/specs/spec-minimum.md)";

fn main() -> ExitCode {
    init_diagnostics();
    tracing::error!(
        event = "startup_failed",
        run_id = "none",
        component = "sea-forge-cli",
        error_class = "not_implemented_error",
        message = NOT_IMPLEMENTED,
    );
    ExitCode::from(64)
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

#[cfg(test)]
mod tests {
    use super::NOT_IMPLEMENTED;

    #[test]
    fn not_implemented_message_mentions_minimum_spec() {
        assert!(NOT_IMPLEMENTED.contains("spec-minimum.md"));
    }
}
