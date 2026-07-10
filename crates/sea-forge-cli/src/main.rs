//! SEA Forge CLI entry point.
//!
//! Foundation skeleton. The `run`, `validate`, `inspect`, and `recall`
//! subcommands are implemented by the minimum spec; until then the binary
//! reports that the minimum slice is not yet implemented.

#![forbid(unsafe_code)]

use std::process::ExitCode;

const NOT_IMPLEMENTED: &str = "sea-forge: minimum slice not implemented yet \
                               (see .agents/specs/spec-minimum.md)";

fn main() -> ExitCode {
    eprintln!("{NOT_IMPLEMENTED}");
    ExitCode::from(64)
}

#[cfg(test)]
mod tests {
    use super::NOT_IMPLEMENTED;

    #[test]
    fn not_implemented_message_mentions_minimum_spec() {
        assert!(NOT_IMPLEMENTED.contains("spec-minimum.md"));
    }
}
