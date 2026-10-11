//! The opt-in case-advance supervisor (plan T04 step 5, operator decision
//! D-3).
//!
//! Spawned by [`crate::run`] **only** when `supervisor.enabled` is true (the
//! absent-section default is false — fail-closed off). The task polls every
//! `poll_interval_secs`, enumerates active cases, and runs one advance pass
//! per case, bounded by BOTH a dedicated supervisor semaphore
//! (`max_concurrent_cases`) and the shared run pool (`state.semaphore`) —
//! exactly the permit [`crate::advance_response`] takes for the verb.
//!
//! # One code path, no second execution path
//!
//! A pass calls the same [`sfwp::case_mutations::advance`] helper the
//! `case.advance` verb calls (with `AdvanceCaller::Supervisor`), so the
//! cannot-miss event publishing and the governed execution
//! (`case_dispatch::execute_sandbox`, F-08 authority for the evaluated role)
//! are literally the same code. Under that helper, the pass also takes the
//! case's keyed lock, so a supervisor pass and a client verb on one case
//! never interleave `case-events.jsonl` appends. Lock order is: supervisor
//! slot → shared run-pool permit → per-case lock (the verb enters at the run
//! permit), so no cycle can form.
//!
//! # Actor and role
//!
//! The pass is attributed to `supervisor.actor` from config — recorded
//! honestly in every ledger/trace record, never as an end user and never as
//! `operator_local`. The role is `ActorRole::Service`: the least-privileged
//! role meant for a non-human automated principal (`actor_type_for_role`
//! maps it to `ActorType::Service`). Because policy rules match
//! `actor_role` exactly, an operator-only policy *fails closed* for
//! supervisor episodes until the operator adds an explicit
//! `actor_role: service` rule — supervision is therefore opt-in at the
//! policy layer as well as in `server.yaml`. SoD never involves the
//! supervisor: it resolves no approvals (the engine cannot), and approval
//! resolution is not part of this loop.
//!
//! # Sandbox-class selection
//!
//! `AdvanceScope::Supervisor` restricts a pass to enabled/ready
//! `ItemKind::SandboxedTask` items. Human tasks, sign-off gates and approval
//! items are never activated or completed: the pass cannot write
//! `approvals.jsonl` or `HumanTaskCompleted` (those verbs never run here),
//! and `ParkHumanTask` is filtered inside the shared engine.
//!
//! # Quiet when idle
//!
//! A pass with nothing advanceable returns `idle`/`blocked` with **zero**
//! appends and therefore zero published events; such passes log at `debug`
//! only. Polls are skipped with a single `warn` per interval when the cell's
//! active policy file is missing, so an unattended misconfigured cell never
//! burns ready items into dispatch-error settlements.
//!
//! Settings are the startup snapshot; a config reload does not resize,
//! restart, or stop a running supervisor (restart-only).

use std::sync::Arc;
use std::time::Duration;

use sea_forge_core::types::ActorRole;
use tokio::sync::Semaphore;

use crate::config::SupervisorConfig;
use crate::sfwp::case_mutations::{advance, AdvanceCaller};
use crate::ServerState;

/// The cell's active authority policy — the same spelling the approval
/// resolution path authorizes against (`<root>/authority/active-policy.json`,
/// see `lib.rs::decide`); F-16 resolution keeps it cell-relative.
const ACTIVE_POLICY: &str = "authority/active-policy.json";

/// Episode timeout for an unattended pass: bounded and modest so a poll wave
/// can never pin a shared run-pool permit for the protocol maximum.
const EPISODE_TIMEOUT_SECS: u64 = 600;

/// Spawn the supervisor poll loop. Called by `run()` only when the operator
/// enabled it; `settings` is the startup snapshot.
pub(crate) fn spawn(state: &Arc<ServerState>, settings: SupervisorConfig) {
    let state = Arc::clone(state);
    tokio::spawn(async move { run(state, settings).await });
}

async fn run(state: Arc<ServerState>, settings: SupervisorConfig) {
    let interval = Duration::from_secs(settings.poll_interval_secs);
    let slots = Arc::new(Semaphore::new(settings.max_concurrent_cases));
    tracing::info!(
        actor = %settings.actor,
        poll_interval_secs = settings.poll_interval_secs,
        max_concurrent_cases = settings.max_concurrent_cases,
        "case-advance supervisor enabled"
    );
    loop {
        // Fail-closed on a missing active policy: skip the whole wave with
        // one log line per interval rather than letting ready items burn into
        // dispatch-error settlements on a misconfigured cell. (A *corrupt*
        // policy still behaves exactly like the verb would — the episode's
        // error class settles durably and loudly, bounded by max_instances.)
        if !state.root.join(ACTIVE_POLICY).exists() {
            tracing::warn!(
                policy = ACTIVE_POLICY,
                "supervisor poll skipped: cell active policy missing"
            );
            tokio::time::sleep(interval).await;
            continue;
        }

        // Enumerate off the async worker: this is the same disk read
        // `case.list` performs (one source of truth for case discovery).
        let root = state.root.clone();
        let listed =
            tokio::task::spawn_blocking(move || crate::sfwp::case_views::list(&root)).await;
        let mut passes = Vec::new();
        match listed {
            Ok(result) => {
                if !result.unreadable.is_empty() {
                    tracing::debug!(
                        count = result.unreadable.len(),
                        "supervisor: skipping unreadable cases"
                    );
                }
                for row in result.cases {
                    // Terminal and approval-parked cases cannot advance; only
                    // active cases are worth a slot and a run-pool permit.
                    if row.case_state != "active" {
                        continue;
                    }
                    let Ok(slot) = Arc::clone(&slots).acquire_owned().await else {
                        tracing::warn!("supervisor slot semaphore closed; stopping");
                        return;
                    };
                    let state = Arc::clone(&state);
                    let case_id = row.case_id;
                    let actor = settings.actor.clone();
                    passes.push(tokio::spawn(async move {
                        pass(state, case_id, actor, slot).await;
                    }));
                }
            }
            Err(error) => tracing::warn!("supervisor: case enumeration failed: {error}"),
        }
        for handle in passes {
            let _ = handle.await;
        }
        tokio::time::sleep(interval).await;
    }
}

/// One supervisor advance pass for one case: hold the supervisor slot, then
/// one shared run-pool permit for the duration — the same bounded-concurrency
/// contract `advance_response` applies to the verb — then the shared helper
/// (which serializes per case). Results log at `info` only when the case
/// actually progressed; idle passes are `debug` so an idle cell produces no
/// per-poll noise.
async fn pass(
    state: Arc<ServerState>,
    case_id: String,
    actor: String,
    _slot: tokio::sync::OwnedSemaphorePermit,
) {
    let permit = match state.semaphore.clone().acquire_owned().await {
        Ok(permit) => permit,
        Err(_) => {
            tracing::warn!(case_id = %case_id, "supervisor pass skipped: run pool unavailable");
            return;
        }
    };
    match advance(
        &state,
        &actor,
        // F-08: the role the supervisor *is*, evaluated for authority —
        // never an assumed operator role (see the module docs).
        ActorRole::Service,
        &case_id,
        AdvanceCaller::Supervisor,
        ACTIVE_POLICY,
        EPISODE_TIMEOUT_SECS,
        permit,
    )
    .await
    {
        Ok(result) => {
            if result.episodes.is_empty() && result.state == "idle" {
                tracing::debug!(case_id = %case_id, "supervisor pass idle");
            } else {
                tracing::info!(
                    case_id = %case_id,
                    state = %result.state,
                    episodes = result.episodes.len(),
                    "supervisor advance pass finished"
                );
            }
        }
        Err(error) => tracing::warn!(case_id = %case_id, "supervisor advance pass failed: {error}"),
    }
}
