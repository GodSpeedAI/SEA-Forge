//! Workload-bound PostgreSQL sessions (SEC-05/SEC-02/SEC-08).
//!
//! The trusted control plane administers the cluster over its Unix socket
//! with peer authentication for the launcher's OS user; no administrator
//! password exists for a workload to find. Each authorized (resource, scope)
//! issuance creates one short-lived native login role for exactly one
//! workload: it inherits only the scope's group role (`SET FALSE`, so it can
//! never assume it), has no administrative attributes, a connection limit and
//! a `VALID UNTIL` expiry. PostgreSQL itself refuses everything else
//! (SQLSTATE 42501). Revocation disables the role, terminates its live
//! backends and drops it, so established sessions end too.
use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::types::{Actor, AuthorityAction, Verdict};
use sea_forge_ledger::LedgerStream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One operator-configured PostgreSQL resource.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresResourceConfig {
    pub name: String,
    /// Directory holding the `psql` client used by the control plane.
    pub psql_bindir: PathBuf,
    /// Library directory for bundled clients (pg0 layouts); optional.
    #[serde(default)]
    pub lib_dir: Option<PathBuf>,
    /// Server Unix socket directory; TCP is not used.
    pub socket_dir: PathBuf,
    pub port: u16,
    pub database: String,
    /// Administrative role reachable only by peer authentication.
    pub admin_role: String,
    /// Scope name -> native group role granting that scope's privileges.
    pub scopes: BTreeMap<String, String>,
    #[serde(default = "default_ttl")]
    pub session_ttl_secs: u64,
    /// Scopes requested on behalf of each isolated agent; authority decides
    /// each one. Issued sessions arrive as `SEA_FORGE_PG_<RESOURCE>_<SCOPE>`.
    #[serde(default)]
    pub offer: Vec<String>,
    /// Scope -> exposure label recorded on the agent's context when a session
    /// for that scope is issued (the broker cannot see query results).
    #[serde(default)]
    pub exposure: BTreeMap<String, String>,
}

fn default_ttl() -> u64 {
    900
}

#[derive(Debug)]
pub struct PostgresError(pub String);

impl std::fmt::Display for PostgresError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PostgresError {}

fn err(message: impl Into<String>) -> PostgresError {
    PostgresError(message.into())
}

fn identifier(value: &str) -> Result<&str, PostgresError> {
    let ok = !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b == b'_')
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    ok.then_some(value)
        .ok_or_else(|| err(format!("invalid PostgreSQL identifier: {value}")))
}

fn random_hex(bytes: usize) -> Result<String, PostgresError> {
    use std::io::Read;
    let mut buffer = vec![0_u8; bytes];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buffer))
        .map_err(|e| err(format!("entropy unavailable: {e}")))?;
    Ok(hex::encode(buffer))
}

/// A live, workload-bound database credential. The password is held only
/// here and in the workload's environment; it is never written to ledgers.
pub struct DatabaseSession {
    pub resource: String,
    pub scope: String,
    pub role: String,
    password: String,
    pub expires_at: String,
    expires: chrono::DateTime<chrono::Utc>,
    pub decision_id: String,
    socket_dir: PathBuf,
    port: u16,
    database: String,
}

impl std::fmt::Debug for DatabaseSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseSession")
            .field("resource", &self.resource)
            .field("scope", &self.scope)
            .field("role", &self.role)
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

impl DatabaseSession {
    /// libpq connection string handed to the workload (Unix socket only).
    /// The session password, for redacting it from workload-produced text.
    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn conninfo(&self) -> String {
        format!(
            "host={} port={} dbname={} user={} password={}",
            self.socket_dir.display(),
            self.port,
            self.database,
            self.role,
            self.password
        )
    }
}

/// When a session issued at `now` must end: the session TTL, cut short by
/// an approval window when the session exists only because of an approval
/// (SEC-07). An approval window that has already closed (`window <= now`)
/// is refused rather than minting an already-expired credential. Whole
/// seconds, rounded down, because `VALID UNTIL` is written at second
/// precision: access never outlives the computed deadline.
pub fn session_deadline(
    now: chrono::DateTime<chrono::Utc>,
    ttl_secs: u64,
    approval_window: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<chrono::DateTime<chrono::Utc>, PostgresError> {
    use chrono::{DurationRound, TimeDelta};
    if ttl_secs == 0 || ttl_secs > i64::MAX as u64 / 1000 {
        return Err(err("session TTL out of range"));
    }
    if approval_window.is_some_and(|window| window <= now) {
        return Err(err("approval window has closed"));
    }
    let ttl = now + chrono::Duration::seconds(ttl_secs as i64);
    let deadline = approval_window.map_or(ttl, |window| ttl.min(window));
    deadline
        .duration_trunc(TimeDelta::seconds(1))
        .map_err(|e| err(format!("deadline: {e}")))
}

pub struct PostgresBroker {
    config: PostgresResourceConfig,
}

impl PostgresBroker {
    pub fn new(config: PostgresResourceConfig) -> Result<Self, PostgresError> {
        identifier(&config.name.replace('-', "_"))?;
        identifier(&config.database)?;
        identifier(&config.admin_role)?;
        for (scope, role) in &config.scopes {
            identifier(&scope.replace('-', "_"))?;
            identifier(role)?;
        }
        if config.session_ttl_secs == 0 {
            return Err(err("session TTL must be positive"));
        }
        Ok(Self { config })
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn scope_exposure(&self, scope: &str) -> Option<&str> {
        self.config.exposure.get(scope).map(String::as_str)
    }

    pub fn offered_scopes(&self) -> &[String] {
        &self.config.offer
    }

    /// Environment variable carrying a session's connection string.
    pub fn env_key(&self, scope: &str) -> String {
        format!("SEA_FORGE_PG_{}_{}", self.config.name, scope)
            .to_ascii_uppercase()
            .replace('-', "_")
    }

    /// Run SQL as the administrative role through peer authentication.
    pub fn admin_sql(&self, sql: &str) -> Result<String, PostgresError> {
        let mut command = Command::new(self.config.psql_bindir.join("psql"));
        command
            .env_clear()
            .args(["-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1"])
            .arg("-h")
            .arg(&self.config.socket_dir)
            .arg("-p")
            .arg(self.config.port.to_string())
            .arg("-U")
            .arg(&self.config.admin_role)
            .arg("-d")
            .arg(&self.config.database)
            .arg("-c")
            .arg(sql);
        if let Some(lib) = &self.config.lib_dir {
            command.env("LD_LIBRARY_PATH", lib);
        }
        let output = command
            .output()
            .map_err(|e| err(format!("psql unavailable: {e}")))?;
        if !output.status.success() {
            return Err(err(format!(
                "administrative statement failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Authorize and issue one session for `workload_uid`. A non-Allow
    /// decision issues nothing; both outcomes are recorded.
    #[allow(clippy::too_many_arguments)]
    pub fn issue(
        &self,
        engine: &PolicyAuthorityEngine,
        bundle: &AuthorityPolicyBundle,
        actor: &Actor,
        ledger: &LedgerStream,
        run_id: &str,
        workspace: &Path,
        scope: &str,
        workload_uid: u32,
    ) -> Result<Option<DatabaseSession>, Box<dyn std::error::Error>> {
        let group = self
            .config
            .scopes
            .get(scope)
            .ok_or_else(|| err(format!("unknown database scope {scope}")))?;
        // A launcher that died left its reapers with it: end every expired
        // session first, so no expired role outlives the next issuance.
        self.reconcile_expired(ledger, run_id)?;
        let action = AuthorityAction::Reserved {
            resource_type: "database_session".into(),
            resource_id: json!([self.config.name, scope]).to_string(),
            parameters: json!({"resource": self.config.name, "scope": scope,
                "workload_uid": workload_uid, "ttl_secs": self.config.session_ttl_secs}),
        };
        let decision = engine.evaluate(AuthorityEvaluation {
            actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
            run_id,
            case_id: run_id,
            plan_item_id: "database-session",
            sequence: 1,
            action: &action,
            workspace_root: workspace,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })?;
        let committed =
            ledger.commit_typed("authority_decision", vec![run_id.into()], &decision, vec![])?;
        if decision.verdict != Verdict::Allow {
            ledger.commit_typed(
                "database_session_refused",
                vec![run_id.into()],
                &json!({"resource": self.config.name, "scope": scope,
                    "workload_uid": workload_uid, "decision_id": decision.decision_id}),
                vec![committed.entry_ulid().to_string()],
            )?;
            return Ok(None);
        }
        let grant = engine.grant(&decision, &committed, &action, None)?;
        grant.authorize(&action, run_id, "database-session", workspace)?;
        let expires = session_deadline(chrono::Utc::now(), self.config.session_ttl_secs, None)?;
        let session = self.mint(scope, group, workload_uid, expires, &decision.decision_id)?;
        ledger.commit_typed(
            "database_session_issued",
            vec![run_id.into()],
            &json!({"resource": self.config.name, "scope": scope, "native_role": session.role,
                "group_role": group, "workload_uid": workload_uid, "expires_at": session.expires_at,
                "decision_id": decision.decision_id}),
            vec![committed.entry_ulid().to_string()],
        )?;
        Ok(Some(session))
    }

    /// The exact authority action for one (scope, workload) session.
    pub fn session_action(&self, scope: &str, workload_uid: u32) -> AuthorityAction {
        AuthorityAction::Reserved {
            resource_type: "database_session".into(),
            resource_id: json!([self.config.name, scope]).to_string(),
            parameters: json!({"resource": self.config.name, "scope": scope,
                "workload_uid": workload_uid, "ttl_secs": self.config.session_ttl_secs}),
        }
    }

    fn mint(
        &self,
        scope: &str,
        group: &str,
        workload_uid: u32,
        expires: chrono::DateTime<chrono::Utc>,
        decision_id: &str,
    ) -> Result<DatabaseSession, PostgresError> {
        let role = format!("sfw_{workload_uid}_{}", random_hex(6)?);
        let password = random_hex(32)?;
        let expires_at = expires.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        // Identifiers are validated; the password and timestamp are hex/RFC3339.
        self.admin_sql(&format!(
            "CREATE ROLE {role} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION \
             NOBYPASSRLS CONNECTION LIMIT 4 PASSWORD '{password}' VALID UNTIL '{expires_at}'; \
             ALTER ROLE {role} SET idle_session_timeout = '{idle_ms}'; \
             GRANT {group} TO {role} WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;",
            idle_ms = self.config.session_ttl_secs.saturating_mul(1000)
        ))?;
        Ok(DatabaseSession {
            resource: self.config.name.clone(),
            scope: scope.into(),
            role,
            password,
            expires_at,
            expires,
            decision_id: decision_id.into(),
            socket_dir: self.config.socket_dir.clone(),
            port: self.config.port,
            database: self.config.database.clone(),
        })
    }

    /// SEC-07 just-in-time elevation: issue a scope whose policy escalates,
    /// only through the existing approval grant (independent resolver,
    /// separation of duties, exact action, one use). The session expires at
    /// the earlier of the approval window and the session TTL; pair it with
    /// [`PostgresBroker::expiry_reaper`] so live backends end at expiry.
    #[allow(clippy::too_many_arguments)]
    pub fn issue_with_approval(
        &self,
        engine: &PolicyAuthorityEngine,
        decision: &sea_forge_core::types::AuthorityDecision,
        decision_ref: &sea_forge_ledger::CommittedRecordRef,
        approval: &sea_forge_core::types::ApprovalRequest,
        approval_ref: &sea_forge_ledger::CommittedRecordRef,
        criteria: &sea_forge_core::types::SettlementCriteriaRecord,
        criteria_ref: &sea_forge_ledger::CommittedRecordRef,
        ledger: &LedgerStream,
        workspace: &Path,
        scope: &str,
        workload_uid: u32,
    ) -> Result<DatabaseSession, Box<dyn std::error::Error>> {
        let group = self
            .config
            .scopes
            .get(scope)
            .ok_or_else(|| err(format!("unknown database scope {scope}")))?;
        let action = self.session_action(scope, workload_uid);
        let grant = engine.grant_after_approval(
            decision,
            decision_ref,
            &action,
            approval,
            approval_ref,
            criteria,
            criteria_ref,
            None,
            ledger,
        )?;
        grant.authorize(&action, &decision.run_id, &decision.plan_item_id, workspace)?;
        let window = chrono::DateTime::parse_from_rfc3339(&approval.expires_at)
            .map_err(|_| err("approval expiry is not RFC 3339"))?
            .with_timezone(&chrono::Utc);
        let expires = session_deadline(
            chrono::Utc::now(),
            self.config.session_ttl_secs,
            Some(window),
        )?;
        let session = self.mint(scope, group, workload_uid, expires, &decision.decision_id)?;
        ledger.commit_typed(
            "database_elevation_issued",
            vec![decision.run_id.clone()],
            &json!({"resource": self.config.name, "scope": scope, "native_role": session.role,
                "group_role": group, "workload_uid": workload_uid, "expires_at": session.expires_at,
                "decision_id": decision.decision_id, "approval_id": approval.approval_id,
                "resolved_by": approval.resolved_by}),
            vec![approval_ref.entry_ulid().to_string()],
        )?;
        Ok(session)
    }

    /// SEC-02 renewal: re-decide under current policy. Anything but Allow
    /// revokes the session (no fallback credential); a revoked or disabled
    /// role cannot be renewed.
    #[allow(clippy::too_many_arguments)]
    pub fn renew(
        &self,
        engine: &PolicyAuthorityEngine,
        bundle: &AuthorityPolicyBundle,
        actor: &Actor,
        ledger: &LedgerStream,
        run_id: &str,
        workspace: &Path,
        session: &mut DatabaseSession,
        workload_uid: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let role = identifier(&session.role)?.to_string();
        if self.admin_sql(&format!(
            "SELECT rolcanlogin FROM pg_roles WHERE rolname = '{role}'"
        ))? != "t"
        {
            return Err(err("session is revoked and cannot be renewed").into());
        }
        let action = self.session_action(&session.scope, workload_uid);
        let decision = engine.evaluate(AuthorityEvaluation {
            actor,
            binding: bundle.resolve_identity(&actor.actor_id, actor.role.clone()),
            run_id,
            case_id: run_id,
            plan_item_id: "database-session",
            sequence: 1,
            action: &action,
            workspace_root: workspace,
            evidence_refs: vec![],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            opa_candidate: None,
            environment: None,
        })?;
        let committed =
            ledger.commit_typed("authority_decision", vec![run_id.into()], &decision, vec![])?;
        if decision.verdict != Verdict::Allow {
            self.revoke(session, ledger, run_id, "renewal_refused")?;
            return Err(err("renewal refused by authority; session revoked").into());
        }
        engine
            .grant(&decision, &committed, &action, None)?
            .authorize(&action, run_id, "database-session", workspace)?;
        let expires = session_deadline(chrono::Utc::now(), self.config.session_ttl_secs, None)?;
        let expires_at = expires.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.admin_sql(&format!("ALTER ROLE {role} VALID UNTIL '{expires_at}'"))?;
        session.expires = expires;
        session.expires_at = expires_at.clone();
        ledger.commit_typed(
            "database_session_renewed",
            vec![run_id.into()],
            &json!({"resource": self.config.name, "native_role": role,
                "expires_at": expires_at, "decision_id": decision.decision_id}),
            vec![committed.entry_ulid().to_string()],
        )?;
        Ok(())
    }

    /// End a session's live access when it expires (`VALID UNTIL` alone only
    /// stops new logins). Extend the deadline after renewal; dropping the
    /// guard stops the reaper without revoking.
    pub fn expiry_reaper(
        &self,
        session: &DatabaseSession,
        ledger_root: PathBuf,
        ledger_stream: String,
        run_id: String,
    ) -> ExpiryReaper {
        let (tx, rx) = std::sync::mpsc::channel::<chrono::DateTime<chrono::Utc>>();
        let broker = PostgresBroker {
            config: self.config.clone(),
        };
        let role = session.role.clone();
        let mut deadline = session.expires;
        let thread = std::thread::spawn(move || loop {
            let wait = (deadline - chrono::Utc::now()).to_std().unwrap_or_default();
            match rx.recv_timeout(wait) {
                Ok(next) => deadline = next,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Ok(ledger) = LedgerStream::open(
                        &ledger_root,
                        ledger_stream.as_str(),
                        "sea-forge-control-plane",
                    ) {
                        let _ = broker.revoke_role(&role, &ledger, &run_id, "expired");
                    }
                    return;
                }
            }
        });
        ExpiryReaper {
            extend: Some(tx),
            thread: Some(thread),
        }
    }

    /// Backends currently attributed to `role` (pid, application_name).
    pub fn active_backends(&self, role: &str) -> Result<Vec<(u32, String)>, PostgresError> {
        let role = identifier(role)?;
        let rows = self.admin_sql(&format!(
            "SELECT pid || '|' || coalesce(application_name, '') FROM pg_stat_activity \
             WHERE usename = '{role}' ORDER BY pid"
        ))?;
        Ok(rows
            .lines()
            .filter_map(|line| {
                let (pid, app) = line.split_once('|')?;
                Some((pid.parse().ok()?, app.to_string()))
            })
            .collect())
    }

    /// Revoke every broker-minted role whose `VALID UNTIL` has passed
    /// (sessions of a launcher that died before its reaper fired). Returns the
    /// revoked role names.
    pub fn reconcile_expired(
        &self,
        ledger: &LedgerStream,
        run_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let expired = self.admin_sql(
            "SELECT rolname FROM pg_roles WHERE rolname LIKE 'sfw\\_%' \
             AND rolvaliduntil IS NOT NULL AND rolvaliduntil <= now() ORDER BY rolname",
        )?;
        let mut revoked = Vec::new();
        for role in expired.lines().filter(|line| !line.is_empty()) {
            self.revoke_role(role, ledger, run_id, "expired_unreaped")?;
            revoked.push(role.to_string());
        }
        Ok(revoked)
    }

    /// Revoke: refuse new logins, end live backends, then drop the role.
    pub fn revoke(
        &self,
        session: &DatabaseSession,
        ledger: &LedgerStream,
        run_id: &str,
        reason: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.revoke_role(&session.role, ledger, run_id, reason)
    }

    fn revoke_role(
        &self,
        role: &str,
        ledger: &LedgerStream,
        run_id: &str,
        reason: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let role = identifier(role)?;
        if self.admin_sql(&format!(
            "SELECT count(*) FROM pg_roles WHERE rolname = '{role}'"
        ))? == "0"
        {
            // Already revoked (for example by the expiry reaper).
            return Ok(json!({"resource": self.config.name, "native_role": role,
                "already_revoked": true, "reason": reason}));
        }
        self.admin_sql(&format!("ALTER ROLE {role} NOLOGIN"))?;
        let terminated = self.admin_sql(&format!(
            "SELECT count(*) FROM (SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity \
             WHERE usename = '{role}') t"
        ))?;
        // Verify while the role still names its backends: once dropped,
        // pg_stat_activity no longer attributes survivors to it.
        let remaining = self.active_backends(role)?.len();
        let record = json!({"resource": self.config.name, "native_role": role,
            "terminated_backends": terminated.parse::<u64>().unwrap_or(0),
            "remaining_backends": remaining, "reason": reason});
        ledger.commit_typed(
            "database_session_revoked",
            vec![run_id.into()],
            &record,
            vec![],
        )?;
        if remaining != 0 {
            // NOLOGIN stays in force; the role is kept for investigation.
            return Err(err("backends survived revocation").into());
        }
        self.admin_sql(&format!("DROP ROLE {role}"))?;
        Ok(record)
    }
}

/// Revokes its session at expiry unless dropped first.
pub struct ExpiryReaper {
    extend: Option<std::sync::mpsc::Sender<chrono::DateTime<chrono::Utc>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl ExpiryReaper {
    /// Move the deadline after a successful renewal.
    pub fn extend_to(&self, session: &DatabaseSession) {
        if let Some(tx) = &self.extend {
            let _ = tx.send(session.expires);
        }
    }

    /// Wait for the reaper to fire (tests and shutdown ordering).
    pub fn join(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ExpiryReaper {
    fn drop(&mut self) {
        self.extend.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::session_deadline;
    use chrono::{DateTime, Duration, Utc};

    fn t(secs: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-17T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            + Duration::seconds(secs)
    }

    /// Injected clock: an approval window closing at T+10 bounds a 300 s
    /// session issued at T+9; at T+10 and T+11 the window is closed and no
    /// credential is minted.
    #[test]
    fn approval_window_bounds_and_then_refuses_elevated_sessions() {
        assert_eq!(session_deadline(t(0), 300, Some(t(10))).unwrap(), t(10));
        assert_eq!(session_deadline(t(9), 300, Some(t(10))).unwrap(), t(10));
        assert!(session_deadline(t(10), 300, Some(t(10))).is_err());
        assert!(session_deadline(t(11), 300, Some(t(10))).is_err());
        // The TTL wins when it ends first; without approval it alone applies.
        assert_eq!(session_deadline(t(0), 5, Some(t(10))).unwrap(), t(5));
        assert_eq!(session_deadline(t(0), 300, None).unwrap(), t(300));
        // Sub-second instants round down, never up.
        let fractional = t(0) + Duration::milliseconds(900);
        assert_eq!(session_deadline(fractional, 2, None).unwrap(), t(2));
        assert!(session_deadline(t(0), 0, None).is_err());
    }
}
