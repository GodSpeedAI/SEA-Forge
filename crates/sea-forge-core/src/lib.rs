//! SEA Forge core kernel.
//!
//! Foundation skeleton only. Minimum-spec lifecycle modules (domain, planner,
//! authority, sandbox, runtime, trace, evidence, settlement, capability) are
//! implemented by `.agents/specs/spec-minimum.md` and are not part of the
//! Shell-SPEC foundation.

#![forbid(unsafe_code)]

/// Minimum supported kernel record schema version (spec-minimum §3.1).
pub const RECORD_VERSION: &str = "0.1";

#[cfg(test)]
mod tests {
    use super::RECORD_VERSION;

    #[test]
    fn record_version_is_stable() {
        assert_eq!(RECORD_VERSION, "0.1");
    }
}
