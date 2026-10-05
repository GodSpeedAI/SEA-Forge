use sea_forge_agent::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Upper bounds prevent a malformed operator file from turning startup into
/// unbounded semaphore allocation or approvals that effectively never expire.
const MAX_CONCURRENT_RUNS: usize = 64;
const MAX_APPROVAL_TTL_HOURS: u64 = 8_760;

/// Bounds for the opt-in case-advance supervisor (plan T04 step 5, decision
/// D-3). The poll ceiling keeps a malformed interval from starving the loop
/// into uselessness; the concurrency ceiling exists because every supervisor
/// pass also holds one permit from the shared run pool for its duration, so
/// beyond roughly twice the pool (`max_concurrent_runs`, itself capped at 64)
/// extra passes would only queue — 8 bounds task and permit allocation while
/// staying a sane parallelism for any realistic pool.
const MIN_SUPERVISOR_POLL_INTERVAL_SECS: u64 = 1;
const MAX_SUPERVISOR_POLL_INTERVAL_SECS: u64 = 3600;
const MAX_SUPERVISOR_CONCURRENT_CASES: usize = 8;
const MAX_SUPERVISOR_ACTOR_LEN: usize = 128;

/// Bounds for the gateway delegation section (plan T02, decision D-2).
/// Actor ids ride into ledger/trace records, so they share the
/// supervisor-actor identifier grammar; the allowlist length cap keeps a
/// malformed operator file from turning the per-request membership scan
/// into an unbounded loop.
const MAX_GATEWAY_ACTOR_LEN: usize = 128;
const MAX_GATEWAY_DELEGABLE_ACTORS: usize = 256;

/// Which OS uid is the multi-user gateway, and which end-user actors it
/// may speak for (plan T02, operator decision D-2).
///
/// Absent (`None`) means delegation is *unconfigured*, and unconfigured
/// refuses every `on_behalf_of` block rather than letting any uid assert
/// any actor. There is deliberately no uid default: this section names one
/// concrete principal, and a default would be a fail-open that let
/// whatever connected first claim the gateway's standing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayConfig {
    /// The OS uid of the gateway process, as `SO_PEERCRED` reports it.
    pub uid: u32,
    /// The gateway principal's own actor id (bound to `uid` with the
    /// gateway role). Defaults to `"gateway"`.
    #[serde(default = "default_gateway_actor")]
    pub actor: String,
    /// End-user actor ids the gateway may present `on_behalf_of`. Empty by
    /// default — a gateway with no allowlist delegates nothing.
    #[serde(default)]
    pub delegable_actors: Vec<String>,
}

fn default_gateway_actor() -> String {
    "gateway".into()
}

///
/// Every field carries a serde default so an absent `supervisor:` section
/// parses to [`SupervisorConfig::default`] — which is *disabled*. The
/// supervisor is fail-closed off: only an operator who writes
/// `supervisor: {enabled: true}` gets a background task that advances cases.
///
/// These settings are read once at server startup (the task is spawned from
/// `run()`); a config reload does not resize, restart, or stop a running
/// supervisor — that takes a restart, consistent with `server.yaml` living
/// inside the cell it configures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupervisorConfig {
    /// Whether the supervisor task is spawned at startup. Default `false`.
    #[serde(default)]
    pub enabled: bool,
    /// Seconds between poll passes. Valid range 1..=3600. Default 5.
    #[serde(default = "default_supervisor_poll_interval_secs")]
    pub poll_interval_secs: u64,
    /// How many case passes may run concurrently inside one poll wave, on top
    /// of (not instead of) a permit from the shared run pool. Valid range
    /// 1..=8. Default 2.
    #[serde(default = "default_supervisor_max_concurrent_cases")]
    pub max_concurrent_cases: usize,
    /// The supervisor's attributed effective actor — the actor id recorded
    /// honestly in every ledger/trace record a supervisor pass produces.
    /// Never an end user's identity. Default `"supervisor"`.
    #[serde(default = "default_supervisor_actor")]
    pub actor: String,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            poll_interval_secs: default_supervisor_poll_interval_secs(),
            max_concurrent_cases: default_supervisor_max_concurrent_cases(),
            actor: default_supervisor_actor(),
        }
    }
}

/// One semantic world this cell can bind CEP authority requests to (migration
/// Stage 6). `base` is a directory relative to the cell root; `entry` and
/// `files` are paths relative to `base` and are the world's *logical* URIs,
/// exactly as DomainForge's CLI derives them from an entry's directory. The
/// `world_ref` is derived from the DomainForge identity, never configured.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CepWorldConfig {
    pub name: String,
    pub base: String,
    pub entry: String,
    pub files: Vec<String>,
}

/// The CEP authority loop (`authority_request` verb). Fail-closed off: absent
/// means disabled, and a disabled cell refuses every request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CepAuthorityConfig {
    #[serde(default)]
    pub enabled: bool,
    /// Authority policy bundle, relative to the cell root.
    #[serde(default = "default_cep_policy")]
    pub policy: String,
    #[serde(default)]
    pub worlds: Vec<CepWorldConfig>,
}

fn default_cep_policy() -> String {
    "sea-forge-policy.yaml".into()
}

impl Default for CepAuthorityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            policy: default_cep_policy(),
            worlds: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_runs: usize,
    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,
    #[serde(default)]
    pub notify_command: Option<Vec<String>>,
    #[serde(default = "default_approval_ttl")]
    pub approval_ttl_hours: u64,
    #[serde(default = "default_root")]
    pub root: PathBuf,
    #[serde(default)]
    pub agent: AgentConfig,
    /// The opt-in case-advance supervisor (plan T04 step 5, D-3).
    ///
    /// Absent means [`SupervisorConfig::default`], whose `enabled` is `false`:
    /// existing configs keep parsing (this section's key is new, and
    /// `deny_unknown_fields` above rejects only *unknown* keys) and the
    /// supervisor stays fail-closed off until an operator opts in.
    #[serde(default)]
    pub supervisor: SupervisorConfig,
    /// The gateway delegation principal (plan T02, operator decision D-2).
    ///
    /// Absent means [`GatewayConfig`] is `None`, which refuses every
    /// `on_behalf_of` block: existing configs keep parsing (this section's
    /// key is new, and `deny_unknown_fields` above rejects only *unknown*
    /// keys) and delegation stays fail-closed off until an operator binds a
    /// gateway uid with a delegable allowlist.
    #[serde(default)]
    pub gateway: Option<GatewayConfig>,
    /// uid -> actor bindings for protected verbs (SF-005, decision U-07).
    ///
    /// Absent means *unconfigured*, and an unconfigured cell refuses every
    /// protected verb rather than deriving an actor from whoever connects.
    /// Reloadable, so an operator can bind a second actor for two-person
    /// approval without restarting a cell that has work in flight.
    #[serde(default)]
    pub identity: crate::identity::IdentityBindings,
    /// The CEP authority loop for Cognate (migration Stage 6). Off unless enabled.
    #[serde(default)]
    pub cep_authority: CepAuthorityConfig,
}

/// The socket file name inside a cell root. Every surface (server, CLI,
/// desktop host) composes this under the resolved root, so one root names one
/// socket. See `docs/CELL_CONTRACT.md`.
pub const SOCKET_FILE_NAME: &str = "server.sock";

/// The conventional cell root, relative to the current directory when
/// `SEA_FORGE_ROOT` is unset. Matches the CLI's `--root` default.
pub const DEFAULT_ROOT_DIR: &str = ".sea-forge";

fn default_max_concurrent() -> usize {
    4
}
fn default_socket_path() -> PathBuf {
    PathBuf::from(SOCKET_FILE_NAME)
}
fn default_approval_ttl() -> u64 {
    24
}
fn default_root() -> PathBuf {
    PathBuf::from(DEFAULT_ROOT_DIR)
}
fn default_supervisor_poll_interval_secs() -> u64 {
    5
}
fn default_supervisor_max_concurrent_cases() -> usize {
    2
}
fn default_supervisor_actor() -> String {
    "supervisor".into()
}

/// Make `path` absolute without requiring it to exist yet.
///
/// `std::fs::canonicalize` cannot be used here: the root and socket are
/// resolved *before* the cell directory is created on a first run.
fn absolutize(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(path),
        // ponytail: a process with no readable CWD cannot resolve a relative
        // root at all; returning it unchanged lets the caller fail on the real
        // filesystem error rather than on a fabricated one.
        Err(_) => path,
    }
}

/// Resolve the absolute cell root: `SEA_FORGE_ROOT` when set, otherwise
/// `.sea-forge` under the current directory.
///
/// The result is always absolute so that a server, a CLI invocation, and a
/// desktop host name the same cell regardless of their working directories.
pub fn resolve_cell_root() -> PathBuf {
    let root = std::env::var_os("SEA_FORGE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(default_root);
    absolutize(root)
}

/// The `SEA_FORGE_SOCKET` override, absolutized, when the operator set one.
///
/// This outranks both the configured `socket_path` and the root-composed
/// default, and is the single documented escape hatch for pointing a surface at
/// a socket outside its own cell.
pub fn resolve_socket_override() -> Option<PathBuf> {
    std::env::var_os("SEA_FORGE_SOCKET").map(|value| absolutize(PathBuf::from(value)))
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_runs: default_max_concurrent(),
            socket_path: default_socket_path(),
            notify_command: None,
            approval_ttl_hours: default_approval_ttl(),
            root: default_root(),
            agent: AgentConfig::default(),
            // Empty by default, and empty refuses every protected verb. A cell
            // that has not said who may act does not get to guess.
            identity: crate::identity::IdentityBindings::default(),
            // Disabled by default: the supervisor is opt-in (fail-closed).
            supervisor: SupervisorConfig::default(),
            // Absent by default: no gateway principal, no delegation.
            gateway: None,
            // Disabled by default: no CEP authority loop until an operator opts in.
            cep_authority: CepAuthorityConfig::default(),
        }
    }
}

impl ServerConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        let config: Self =
            serde_yaml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
        config.validate().map_err(|message| {
            format!(
                "{}: invalid server configuration: {message}",
                path.display()
            )
        })?;
        Ok(config)
    }

    /// Validate every operator-controlled value before startup creates cell
    /// records or binds a socket.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent_runs == 0 || self.max_concurrent_runs > MAX_CONCURRENT_RUNS {
            return Err(format!(
                "max_concurrent_runs must be between 1 and {MAX_CONCURRENT_RUNS}"
            ));
        }
        if self.approval_ttl_hours == 0 || self.approval_ttl_hours > MAX_APPROVAL_TTL_HOURS {
            return Err(format!(
                "approval_ttl_hours must be between 1 and {MAX_APPROVAL_TTL_HOURS}"
            ));
        }
        // Supervisor bounds (T04C). Validated even while disabled: a section
        // an operator wrote must be coherent before the first side effect, and
        // an out-of-range value must never silently re-enable or reshape a
        // task that a later `enabled: true` would spawn.
        if !(MIN_SUPERVISOR_POLL_INTERVAL_SECS..=MAX_SUPERVISOR_POLL_INTERVAL_SECS)
            .contains(&self.supervisor.poll_interval_secs)
        {
            return Err(format!(
                "supervisor.poll_interval_secs must be between {MIN_SUPERVISOR_POLL_INTERVAL_SECS} and {MAX_SUPERVISOR_POLL_INTERVAL_SECS} (got {})",
                self.supervisor.poll_interval_secs
            ));
        }
        if self.supervisor.max_concurrent_cases == 0
            || self.supervisor.max_concurrent_cases > MAX_SUPERVISOR_CONCURRENT_CASES
        {
            return Err(format!(
                "supervisor.max_concurrent_cases must be between 1 and {MAX_SUPERVISOR_CONCURRENT_CASES} (got {})",
                self.supervisor.max_concurrent_cases
            ));
        }
        // The actor id is attributed in ledgers, traces and authority
        // evaluations, so it must be a bounded identifier — never empty, never
        // free text that could masquerade as a sentence or smuggle control
        // characters into a record.
        if !sea_forge_core::path::valid_id_segment(&self.supervisor.actor, MAX_SUPERVISOR_ACTOR_LEN)
        {
            return Err(format!(
                "supervisor.actor must be a non-empty identifier of at most {MAX_SUPERVISOR_ACTOR_LEN} characters using only [A-Za-z0-9_-] (got {:?})",
                self.supervisor.actor
            ));
        }
        // Gateway delegation bounds (T02, D-2). Validated even with an empty
        // allowlist: a section an operator wrote must be coherent before the
        // first delegated side effect, and duplicates are refused (not
        // silently deduped) so the operator sees exactly the allowlist that
        // will be enforced.
        if let Some(gateway) = &self.gateway {
            if !sea_forge_core::path::valid_id_segment(&gateway.actor, MAX_GATEWAY_ACTOR_LEN) {
                return Err(format!(
                    "gateway.actor must be a non-empty identifier of at most {MAX_GATEWAY_ACTOR_LEN} characters using only [A-Za-z0-9_-] (got {:?})",
                    gateway.actor
                ));
            }
            if gateway.delegable_actors.len() > MAX_GATEWAY_DELEGABLE_ACTORS {
                return Err(format!(
                    "gateway.delegable_actors must hold at most {MAX_GATEWAY_DELEGABLE_ACTORS} actors (got {})",
                    gateway.delegable_actors.len()
                ));
            }
            let mut seen = std::collections::HashSet::new();
            for actor in &gateway.delegable_actors {
                if !sea_forge_core::path::valid_id_segment(actor, MAX_GATEWAY_ACTOR_LEN) {
                    return Err(format!(
                        "gateway.delegable_actors must be non-empty identifiers of at most {MAX_GATEWAY_ACTOR_LEN} characters using only [A-Za-z0-9_-] (got {actor:?})",
                    ));
                }
                if !seen.insert(actor) {
                    return Err(format!(
                        "gateway.delegable_actors must not contain duplicates (got {actor:?} twice)"
                    ));
                }
                if *actor == gateway.actor {
                    return Err(format!(
                        "gateway.delegable_actors must not contain the gateway actor itself (got {actor:?})"
                    ));
                }
                if *actor == self.supervisor.actor {
                    return Err(format!(
                        "gateway.delegable_actors must not contain the supervisor actor (got {actor:?}); supervisor passes are never end-user work"
                    ));
                }
            }
            // Both halves of a delegation must be resolvable, or the section is
            // a promise the cell cannot keep: the gateway could never present
            // its own claim, or an allowlisted actor could never resolve to any
            // standing. Refused at load, where the operator can fix the file,
            // rather than discovered as a runtime refusal at the first
            // delegated request in production.
            if !self
                .identity
                .bindings
                .iter()
                .any(|binding| binding.uid == gateway.uid && binding.actor_id == gateway.actor)
            {
                return Err(format!(
                    "gateway.uid {} must be bound to the gateway actor {:?} in `identity.bindings` \
                     (expected `- uid: {}, actor_id: {}, roles: [...]`), or the gateway can never \
                     present its own claim",
                    gateway.uid, gateway.actor, gateway.uid, gateway.actor
                ));
            }
            for actor in &gateway.delegable_actors {
                if !self
                    .identity
                    .bindings
                    .iter()
                    .any(|binding| binding.actor_id == *actor && !binding.roles.is_empty())
                {
                    return Err(format!(
                        "gateway.delegable_actors names {actor:?}, which no `identity.bindings` \
                         entry binds to a role, so a delegation to it could never resolve"
                    ));
                }
            }
        }
        self.agent
            .validate()
            .map_err(|message| format!("invalid agent configuration: {message}"))
    }

    /// The one socket path this configuration names.
    ///
    /// A relative `socket_path` composes under `root`, so relocating the root
    /// relocates the socket with it — records and socket never split across two
    /// cells. An absolute `socket_path` is an explicit operator override and is
    /// honored verbatim.
    ///
    /// This reads no environment; `SEA_FORGE_SOCKET` is applied by the binary
    /// entry point (see `resolve_socket_override`) so that in-process tests
    /// cannot leak one run's override into another's.
    pub fn resolved_socket_path(&self) -> PathBuf {
        if self.socket_path.is_absolute() {
            return self.socket_path.clone();
        }
        absolutize(self.root.join(&self.socket_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_rejects_invalid_agent_endpoint_with_asserted_status() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "agent:\n  endpoints:\n    - id: bad\n      kind: open_ai_compatible\n      base_url: https://example.com\n      status: probed\n",
        )
        .unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("evidence-derived"), "{error}");
    }

    #[test]
    fn load_rejects_non_loopback_http_endpoint_without_test_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "agent:\n  endpoints:\n    - id: bad\n      kind: open_ai_compatible\n      base_url: http://example.com\n",
        )
        .unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("HTTPS"), "{error}");
    }

    #[test]
    fn load_rejects_unknown_top_level_field_before_startup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(&path, "max_concurrent_rnns: 4\n").unwrap();

        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("unknown field"), "{error}");
    }

    #[test]
    fn load_rejects_unbounded_startup_limits() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(&path, "max_concurrent_runs: 0\n").unwrap();
        let zero_error = ServerConfig::load(&path).unwrap_err();
        assert!(zero_error.contains("between 1"), "{zero_error}");

        std::fs::write(&path, "approval_ttl_hours: 8761\n").unwrap();
        let ttl_error = ServerConfig::load(&path).unwrap_err();
        assert!(ttl_error.contains("between 1"), "{ttl_error}");
    }

    #[test]
    fn relative_socket_path_composes_under_root() {
        let config = ServerConfig {
            root: PathBuf::from("/tmp/cell"),
            ..Default::default()
        };
        assert_eq!(
            config.resolved_socket_path(),
            PathBuf::from("/tmp/cell/server.sock"),
            "relocating the root must relocate the socket with it"
        );
    }

    #[test]
    fn absolute_socket_path_is_honored_verbatim() {
        let config = ServerConfig {
            root: PathBuf::from("/tmp/cell"),
            socket_path: PathBuf::from("/run/user/1000/sea-forge.sock"),
            ..Default::default()
        };
        assert_eq!(
            config.resolved_socket_path(),
            PathBuf::from("/run/user/1000/sea-forge.sock")
        );
    }

    #[test]
    fn resolved_socket_path_is_always_absolute() {
        let config = ServerConfig::default();
        assert!(
            config.resolved_socket_path().is_absolute(),
            "every surface must name the same socket regardless of its CWD"
        );
    }

    #[test]
    fn default_layout_matches_the_conventional_cell() {
        // The pre-contract default was a standalone `.sea-forge/server.sock`.
        // Composition under the default root must land on the same file so no
        // existing cell moves underneath an operator.
        let config = ServerConfig::default();
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(
            config.resolved_socket_path(),
            cwd.join(".sea-forge").join("server.sock")
        );
    }

    #[test]
    fn load_accepts_explicit_loopback_test_endpoint() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "agent:\n  endpoints:\n    - id: local\n      kind: open_ai_compatible\n      base_url: http://127.0.0.1:11434/v1\n      allow_loopback_test: true\n",
        )
        .unwrap();

        let config = ServerConfig::load(&path).unwrap();
        assert_eq!(config.agent.endpoints.len(), 1);
        assert!(config.agent.endpoints[0].allow_loopback_test);
    }

    #[test]
    fn load_accepts_valid_agent_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "agent:\n  endpoints:\n    - id: prod\n      kind: open_ai_compatible\n      base_url: https://api.example.com\n      credential_ref: OPENAI_API_KEY\n",
        )
        .unwrap();
        let config = ServerConfig::load(&path).unwrap();
        assert_eq!(config.agent.endpoints.len(), 1);
    }

    // -----------------------------------------------------------------------
    // The opt-in case-advance supervisor section (T04C, requirement 7a/7f).
    // -----------------------------------------------------------------------

    #[test]
    fn an_absent_supervisor_section_parses_to_disabled_bounded_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(&path, "max_concurrent_runs: 4\n").unwrap();

        let config = ServerConfig::load(&path).unwrap();
        // Opt-in, fail-closed: no section means no supervisor task.
        assert!(!config.supervisor.enabled);
        assert_eq!(config.supervisor.poll_interval_secs, 5);
        assert_eq!(config.supervisor.max_concurrent_cases, 2);
        assert_eq!(config.supervisor.actor, "supervisor");
        // The parse default and the struct default must be the same value, or
        // a missing file and an empty file would disagree.
        let default = ServerConfig::default();
        assert!(!default.supervisor.enabled);
        assert_eq!(default.supervisor, config.supervisor);
    }

    #[test]
    fn an_explicit_supervisor_section_parses_field_by_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "supervisor:\n  enabled: true\n  poll_interval_secs: 60\n  max_concurrent_cases: 4\n  actor: cell_supervisor\n",
        )
        .unwrap();

        let config = ServerConfig::load(&path).unwrap();
        assert!(config.supervisor.enabled);
        assert_eq!(config.supervisor.poll_interval_secs, 60);
        assert_eq!(config.supervisor.max_concurrent_cases, 4);
        assert_eq!(config.supervisor.actor, "cell_supervisor");
    }

    #[test]
    fn an_unknown_supervisor_field_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "supervisor:\n  enabled: true\n  pol_interval_secs: 5\n",
        )
        .unwrap();

        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("unknown field"), "{error}");
    }

    #[test]
    fn load_rejects_out_of_range_supervisor_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");

        std::fs::write(&path, "supervisor:\n  poll_interval_secs: 0\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("supervisor.poll_interval_secs"), "{error}");
        assert!(error.contains("between 1 and 3600"), "{error}");

        std::fs::write(&path, "supervisor:\n  poll_interval_secs: 3601\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("between 1 and 3600"), "{error}");

        std::fs::write(&path, "supervisor:\n  max_concurrent_cases: 0\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("supervisor.max_concurrent_cases"), "{error}");
        assert!(error.contains("between 1 and 8"), "{error}");

        std::fs::write(&path, "supervisor:\n  max_concurrent_cases: 9\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("between 1 and 8"), "{error}");

        // The actor id is attributed in durable records: empty and free text
        // are both refused.
        std::fs::write(&path, "supervisor:\n  actor: \"\"\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("supervisor.actor"), "{error}");

        std::fs::write(&path, "supervisor:\n  actor: \"two words\"\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("supervisor.actor"), "{error}");

        std::fs::write(&path, "supervisor:\n  actor: \"a;rm -rf\"\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("supervisor.actor"), "{error}");
    }

    // -----------------------------------------------------------------------
    // The gateway delegation section (T02, decision D-2).
    // -----------------------------------------------------------------------

    #[test]
    fn an_absent_gateway_section_parses_to_none_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(&path, "max_concurrent_runs: 4\n").unwrap();

        let config = ServerConfig::load(&path).unwrap();
        // Fail-closed: no section means no delegation principal.
        assert!(config.gateway.is_none());
        let default = ServerConfig::default();
        assert!(default.gateway.is_none());
    }

    #[test]
    fn an_explicit_gateway_section_parses_field_by_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "identity:\n  bindings:\n    - uid: 2000\n      actor_id: cell_gateway\n      roles: [\"service\"]\n    - uid: 3101\n      actor_id: operator_a\n      roles: [\"operator\"]\n    - uid: 3102\n      actor_id: operator_b\n      roles: [\"operator\"]\ngateway:\n  uid: 2000\n  actor: cell_gateway\n  delegable_actors: [operator_a, operator_b]\n",
        )
        .unwrap();

        let config = ServerConfig::load(&path).unwrap();
        let gateway = config.gateway.expect("gateway section must parse");
        assert_eq!(gateway.uid, 2000);
        assert_eq!(gateway.actor, "cell_gateway");
        assert_eq!(gateway.delegable_actors, vec!["operator_a", "operator_b"]);
    }

    #[test]
    fn a_gateway_actor_defaults_to_gateway() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "identity:\n  bindings:\n    - uid: 2000\n      actor_id: gateway\n      roles: [\"service\"]\ngateway:\n  uid: 2000\n",
        )
        .unwrap();

        let config = ServerConfig::load(&path).unwrap();
        let gateway = config.gateway.expect("gateway section must parse");
        assert_eq!(gateway.actor, "gateway");
        assert!(gateway.delegable_actors.is_empty());
    }

    /// A gateway section that names a uid no binding covers is a cell where
    /// delegation could never resolve. Refused at load: an operator who wrote
    /// the section meant it to work.
    #[test]
    fn load_rejects_a_gateway_uid_with_no_binding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "identity:\n  bindings:\n    - uid: 3101\n      actor_id: operator_a\n      roles: [\"operator\"]\ngateway:\n  uid: 2000\n  actor: gateway\n  delegable_actors: [operator_a]\n",
        )
        .unwrap();

        let error = ServerConfig::load(&path).unwrap_err();
        assert!(
            error.contains("gateway.uid 2000 must be bound to the gateway actor"),
            "{error}"
        );
    }

    /// ... and an allowlisted actor no binding covers, for the same reason on
    /// the other side of the delegation.
    #[test]
    fn load_rejects_a_delegable_actor_with_no_binding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(
            &path,
            "identity:\n  bindings:\n    - uid: 2000\n      actor_id: gateway\n      roles: [\"service\"]\ngateway:\n  uid: 2000\n  actor: gateway\n  delegable_actors: [operator_a]\n",
        )
        .unwrap();

        let error = ServerConfig::load(&path).unwrap_err();
        assert!(
            error.contains("gateway.delegable_actors names \"operator_a\""),
            "{error}"
        );

        // A binding that exists but declares no role is not standing either:
        // the actor could claim nothing, so the allowlist entry would be a
        // promise the cell cannot keep.
        std::fs::write(
            &path,
            "identity:\n  bindings:\n    - uid: 2000\n      actor_id: gateway\n      roles: [\"service\"]\n    - uid: 3101\n      actor_id: operator_a\n      roles: []\ngateway:\n  uid: 2000\n  actor: gateway\n  delegable_actors: [operator_a]\n",
        )
        .unwrap();

        let error = ServerConfig::load(&path).unwrap_err();
        assert!(
            error.contains("gateway.delegable_actors names \"operator_a\""),
            "{error}"
        );
    }

    #[test]
    fn an_unknown_gateway_field_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");
        std::fs::write(&path, "gateway:\n  uid: 2000\n  uidd: 2000\n").unwrap();

        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("unknown field"), "{error}");
    }

    #[test]
    fn load_rejects_invalid_gateway_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("server.yaml");

        std::fs::write(&path, "gateway:\n  uid: 2000\n  actor: \"\"\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("gateway.actor"), "{error}");

        std::fs::write(&path, "gateway:\n  uid: 2000\n  actor: \"two words\"\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("gateway.actor"), "{error}");

        std::fs::write(&path, "gateway:\n  uid: 2000\n  delegable_actors: [\"\"]\n").unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("gateway.delegable_actors"), "{error}");

        std::fs::write(
            &path,
            "gateway:\n  uid: 2000\n  delegable_actors: [\"two words\"]\n",
        )
        .unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("gateway.delegable_actors"), "{error}");

        // Duplicates are refused, not silently deduped: the operator sees
        // exactly the allowlist that will be enforced.
        std::fs::write(
            &path,
            "gateway:\n  uid: 2000\n  delegable_actors: [operator_a, operator_a]\n",
        )
        .unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("duplicates"), "{error}");

        // The gateway may never speak as itself through delegation.
        std::fs::write(
            &path,
            "gateway:\n  uid: 2000\n  delegable_actors: [gateway]\n",
        )
        .unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("gateway actor itself"), "{error}");

        // The supervisor actor is never delegable (D-2 / D-3).
        std::fs::write(
            &path,
            "gateway:\n  uid: 2000\n  delegable_actors: [supervisor]\n",
        )
        .unwrap();
        let error = ServerConfig::load(&path).unwrap_err();
        assert!(error.contains("supervisor actor"), "{error}");
    }
}
