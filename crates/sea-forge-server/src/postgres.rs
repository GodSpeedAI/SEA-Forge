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
        let role = format!("sfw_{workload_uid}_{}", random_hex(6)?);
        let password = random_hex(32)?;
        let expires_at = (chrono::Utc::now()
            + chrono::Duration::seconds(self.config.session_ttl_secs as i64))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        // Identifiers are validated; the password and timestamp are hex/RFC3339.
        self.admin_sql(&format!(
            "CREATE ROLE {role} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION \
             NOBYPASSRLS CONNECTION LIMIT 4 PASSWORD '{password}' VALID UNTIL '{expires_at}'; \
             GRANT {group} TO {role} WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;"
        ))?;
        ledger.commit_typed(
            "database_session_issued",
            vec![run_id.into()],
            &json!({"resource": self.config.name, "scope": scope, "native_role": role,
                "group_role": group, "workload_uid": workload_uid, "expires_at": expires_at,
                "decision_id": decision.decision_id}),
            vec![committed.entry_ulid().to_string()],
        )?;
        Ok(Some(DatabaseSession {
            resource: self.config.name.clone(),
            scope: scope.into(),
            role,
            password,
            expires_at,
            decision_id: decision.decision_id,
            socket_dir: self.config.socket_dir.clone(),
            port: self.config.port,
            database: self.config.database.clone(),
        }))
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

    /// Revoke: refuse new logins, end live backends, then drop the role.
    pub fn revoke(
        &self,
        session: &DatabaseSession,
        ledger: &LedgerStream,
        run_id: &str,
        reason: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let role = identifier(&session.role)?;
        self.admin_sql(&format!("ALTER ROLE {role} NOLOGIN"))?;
        let terminated = self.admin_sql(&format!(
            "SELECT count(*) FROM (SELECT pg_terminate_backend(pid, 5000) FROM pg_stat_activity \
             WHERE usename = '{role}') t"
        ))?;
        // Verify while the role still names its backends: once dropped,
        // pg_stat_activity no longer attributes survivors to it.
        let remaining = self.active_backends(role)?.len();
        let record = json!({"resource": session.resource, "native_role": role,
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
