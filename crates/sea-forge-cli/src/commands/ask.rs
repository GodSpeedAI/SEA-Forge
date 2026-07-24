//! Thoth `ask` command (spec-adlc-thoth §11.1, §8.6).
//!
//! `sea-forge ask <kind> <subject> [--purpose] [--case] [--json]`
//! Exit codes: 0 answered, 4 denied (governance success), 2 usage, 1 error.
//!
//! A thin adapter: all governance (real snapshot join, disclosure policy,
//! ledgered question/decision/answer chain) lives in
//! `sea_forge_thoth::service::ask` (T14A), so this module only parses CLI
//! input and formats output (T14B) — it never constructs its own
//! `SnapshotView` or loads authority policy directly.

use sea_forge_core::errors::ForgeError;
use sea_forge_thoth::protocol::*;
use sea_forge_thoth::service::ask;
use std::path::Path;

/// Parse a string into a QuestionKind. Returns Err on unknown kind.
fn parse_kind(s: &str) -> Result<QuestionKind, ForgeError> {
    parse_question_kind(s).ok_or_else(|| ForgeError::Input(format!("unknown question kind: {s}")))
}

/// Run the `ask` command. Returns exit code.
pub fn run(
    kind_str: &str,
    subject: &str,
    purpose: &str,
    case_id: Option<&str>,
    json: bool,
    root: &Path,
    actor: &str,
) -> Result<u8, ForgeError> {
    let kind = parse_kind(kind_str).map_err(|e| {
        eprintln!("sea-forge ask: {e}");
        e
    })?;

    let answer = ask(root, actor, kind, subject, purpose, case_id)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&answer)
                .map_err(|e| ForgeError::Serialization(e.to_string()))?
        );
    } else {
        print_human(&answer);
    }

    Ok(match answer.disposition {
        Disposition::Denied => 4,
        Disposition::Answered | Disposition::Partial => 0,
    })
}

fn print_human(answer: &ThothAnswer) {
    println!("disposition: {:?}", answer.disposition);
    println!("freshness: {:?}", answer.freshness);
    println!("assurance: {}", answer.assurance);
    println!("snapshot: {}", answer.snapshot_ref);
    for claim in &answer.claims {
        println!(
            "  {}: {:?} ({}) — {} [{}]",
            claim.subject,
            claim.status,
            claim_class_to_surface_str(&claim.claim_class),
            claim.statement,
            claim.evidence_refs.join(", ")
        );
    }
    if !answer.omitted_claim_classes.is_empty() {
        print!("omitted: ");
        for c in &answer.omitted_claim_classes {
            print!("{c:?} ");
        }
        println!();
    }
    println!("notice: {}", answer.authority_notice);
}
