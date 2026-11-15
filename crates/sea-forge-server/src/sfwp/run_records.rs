//! Run records and delegation links (WP-21 part A, `contracts/fde/README.md` §8).
//!
//! Written by the case keeper only, never by a client, in the case's store:
//! - `field-runs.jsonl`: one `RunRecord` for every execution of a Field Case
//!   episode, whatever the verb (`item_execute`, `case_advance`, `delegate`).
//! - `field-links.jsonl`: one `Link` for every accepted `delegate`, written
//!   BEFORE the run it delegates starts (the run record cites its `link_id`).
//!
//! Both writes happen in the execution gate's critical section: the caller of
//! [`record_run`] holds the case's keyed lock (`ServerState::acquire_case_lock`),
//! the same lock that covers the freeze check, the assignment record and every
//! `case.record.append` of the case. The run itself starts after the lock is
//! released, so a record never lags the run it describes.
//!
//! Settlement reads them back through [`analyse`]: the executor set, link
//! validity (fail closed: `DELEGATION_LINK_INVALID`) and the verifier context,
//! derived from run records and never taken from the payload.

use super::case_records::{
    append_line, canonical_actor, case_dir, now_rfc3339, parse_time, read_lines_strict, LINKS, RUNS,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

/// One execution to record.
pub struct RunInput<'a> {
    pub case_id: &'a str,
    pub episode_ref: &'a str,
    pub verb: &'a str,
    /// `executor`, or `verifier` for a delegation to a configured verifier endpoint.
    pub role: &'a str,
    /// The authenticated principal the run executed under; `None` when it
    /// could not be resolved (settlement is then refused).
    pub actor: Option<&'a str>,
    pub on_behalf_of: Option<&'a str>,
    pub workspace_ref: &'a str,
    /// A delegated run: the link written before it.
    pub link: Option<LinkInput<'a>>,
}

pub struct LinkInput<'a> {
    pub assignment_id: &'a str,
    pub delegator: &'a str,
    pub gateway_actor: Option<&'a str>,
    pub delegate: &'a str,
    /// Seconds the link stays valid after issue.
    pub ttl_secs: i64,
}

fn stamp(t: chrono::DateTime<chrono::Utc>) -> String {
    t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Write the run record (and, for a delegated run, the link first). Returns the
/// run id. The caller holds the case lock.
pub fn record_run(root: &Path, input: &RunInput) -> Result<String, String> {
    let dir = case_dir(root, input.case_id);
    let runs = read_lines_strict(&dir.join(RUNS))?;
    let links = read_lines_strict(&dir.join(LINKS))?;
    let run_id = format!("run-{:04}", runs.len() + 1);
    let issued = chrono::Utc::now();
    let mut link_id = Value::Null;
    if let Some(link) = &input.link {
        let id = format!("link-{:04}", links.len() + 1);
        append_line(
            &dir.join(LINKS),
            &json!({
                "link_id": id, "case_id": input.case_id, "episode_ref": input.episode_ref,
                "assignment_id": link.assignment_id, "delegator": link.delegator,
                "gateway_actor": link.gateway_actor, "delegate": link.delegate,
                "issued_at": stamp(issued),
                "expires_at": stamp(issued + chrono::Duration::seconds(link.ttl_secs)),
            }),
        )?;
        link_id = json!(id);
    }
    // The run starts after the link: a strictly later timestamp, never equal.
    let started = chrono::Utc::now().max(issued + chrono::Duration::milliseconds(1));
    append_line(
        &dir.join(RUNS),
        &json!({
            "run_id": run_id, "case_id": input.case_id, "episode_ref": input.episode_ref,
            "verb": input.verb, "role": input.role, "actor": input.actor,
            "on_behalf_of": input.on_behalf_of, "delegated": input.link.is_some(),
            "link_id": link_id, "workspace_ref": input.workspace_ref,
            "started_at": stamp(started), "recorded_at": now_rfc3339(),
        }),
    )?;
    Ok(run_id)
}

/// What settlement needs to know about the episode's executions.
#[derive(Debug)]
pub struct Analysis {
    /// Canonical principals of the executor set (README §8).
    pub executors: BTreeSet<String>,
    /// The episode has at least one run record.
    pub runs_exist: bool,
    /// `fresh`, `shared` or `unknown`, for the settler (README §6 "Verifier context").
    pub derived_context: &'static str,
    /// The settler shares a delegation root with an executor.
    pub sibling: bool,
}

fn canon(v: &Value) -> Option<String> {
    v.as_str().and_then(|s| canonical_actor(s).ok())
}

fn episode_prefix(episode_ref: &str) -> &str {
    episode_ref.split("@sha256:").next().unwrap_or(episode_ref)
}

fn root_of(links: &[&Value], who: &str) -> String {
    let mut current = who.to_string();
    // A cycle cannot run longer than the number of links.
    for _ in 0..=links.len() {
        match links
            .iter()
            .find(|l| canon(&l["delegate"]).as_deref() == Some(&current))
        {
            Some(l) => match canon(&l["delegator"]) {
                Some(up) if up != current => current = up,
                _ => break,
            },
            None => break,
        }
    }
    current
}

/// Read the episode's run records and links and decide, failing closed.
/// `Err(reason)` is `DELEGATION_LINK_INVALID`.
pub fn analyse(
    root: &Path,
    case_id: &str,
    episode_ref: &str,
    settler: &str,
) -> Result<Result<Analysis, String>, String> {
    let dir = case_dir(root, case_id);
    let all_runs = read_lines_strict(&dir.join(RUNS))?;
    let all_links = read_lines_strict(&dir.join(LINKS))?;
    let prefix = episode_prefix(episode_ref);
    let mine = |v: &Value| {
        v["episode_ref"]
            .as_str()
            .is_some_and(|r| episode_prefix(r) == prefix)
    };
    let runs: Vec<&Value> = all_runs.iter().filter(|r| mine(r)).collect();
    let links: Vec<&Value> = all_links.iter().filter(|l| mine(l)).collect();
    let invalid = |why: &str| Ok(Err(why.to_string()));

    for l in &links {
        let (Some(issued), Some(expires)) =
            (parse_time(&l["issued_at"]), parse_time(&l["expires_at"]))
        else {
            return invalid("a link has no usable issued_at or expires_at");
        };
        if expires <= issued {
            return invalid("a link expires at or before it was issued");
        }
        let (delegate, delegator) = (canon(&l["delegate"]), canon(&l["delegator"]));
        if delegate.is_none() || delegator.is_none() || delegate == delegator {
            return invalid("a link has no delegate or delegator, or they are the same principal");
        }
        if l["episode_ref"] != episode_ref || l["case_id"] != case_id {
            return invalid("a link names another episode or case");
        }
        let assignment_ok = l["assignment_id"].as_str().is_some_and(|a| {
            a.strip_prefix("assign:").is_some_and(|rest| {
                let parts: Vec<&str> = rest.split(':').collect();
                let key: Vec<&str> = prefix
                    .strip_prefix("episode:")
                    .unwrap_or("")
                    .split(':')
                    .collect();
                parts.len() == 4 && parts[..3] == key[..]
            })
        });
        if !assignment_ok {
            return invalid("a link's assignment id disagrees with its episode");
        }
    }
    for r in &runs {
        if r["case_id"] != case_id || r["episode_ref"] != episode_ref {
            return invalid("a run names another episode or case");
        }
        let Some(actor) = canon(&r["actor"]) else {
            return invalid("a run has no resolvable actor");
        };
        let Some(started) = parse_time(&r["started_at"]) else {
            return invalid("a run has no usable started_at");
        };
        if r["delegated"] == true {
            let link = r["link_id"]
                .as_str()
                .and_then(|id| all_links.iter().find(|l| l["link_id"] == id));
            let Some(link) = link else {
                return invalid("a delegated run has no link");
            };
            if canon(&link["delegate"]).as_deref() != Some(&actor) {
                return invalid("a delegated run's actor is not its link's delegate");
            }
            if !mine(link) || link["episode_ref"] != r["episode_ref"] {
                return invalid("a delegated run cites a link of another episode");
            }
            match parse_time(&link["issued_at"]) {
                Some(issued) if issued <= started => {}
                _ => return invalid("a link was written after its run started"),
            }
        }
    }

    // The executor set: actor and on_behalf_of of every executor run, the
    // delegate of every link (except one cited only by verifier runs), and the
    // transitive closure of their delegators.
    let mut executors: BTreeSet<String> = BTreeSet::new();
    for r in runs.iter().filter(|r| r["role"] == "executor") {
        executors.extend(canon(&r["actor"]));
        executors.extend(canon(&r["on_behalf_of"]));
    }
    for l in &links {
        let citing: Vec<&&Value> = runs
            .iter()
            .filter(|r| r["link_id"] == l["link_id"])
            .collect();
        let verifier_only = !citing.is_empty() && citing.iter().all(|r| r["role"] == "verifier");
        if !verifier_only {
            executors.extend(canon(&l["delegate"]));
        }
    }
    loop {
        let before = executors.len();
        for l in &links {
            if canon(&l["delegate"]).is_some_and(|d| executors.contains(&d)) {
                executors.extend(canon(&l["delegator"]));
            }
        }
        if executors.len() == before {
            break;
        }
    }

    let settler = canonical_actor(settler).unwrap_or_default();
    let executor_ws: Vec<&str> = runs
        .iter()
        .filter(|r| r["role"] == "executor")
        .filter_map(|r| r["workspace_ref"].as_str())
        .collect();
    let verifier_runs: Vec<&&Value> = runs
        .iter()
        .filter(|r| r["role"] == "verifier" && canon(&r["actor"]).as_deref() == Some(&settler))
        .collect();
    let derived_context = if verifier_runs.iter().any(|r| {
        r["workspace_ref"]
            .as_str()
            .is_some_and(|w| !executor_ws.contains(&w))
    }) {
        "fresh"
    } else if !verifier_runs.is_empty() {
        "shared"
    } else {
        "unknown"
    };
    let case_links: Vec<&Value> = all_links.iter().collect();
    let settler_root = root_of(&case_links, &settler);
    let sibling = settler_root != settler
        && executors
            .iter()
            .any(|e| root_of(&case_links, e) == settler_root);
    Ok(Ok(Analysis {
        executors,
        runs_exist: !runs.is_empty(),
        derived_context,
        sibling,
    }))
}
