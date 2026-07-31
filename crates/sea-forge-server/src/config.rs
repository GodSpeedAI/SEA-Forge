use sea_forge_agent::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// uid -> actor bindings for protected verbs (SF-005, decision U-07).
    ///
    /// Absent means *unconfigured*, and an unconfigured cell refuses every
    /// protected verb rather than deriving an actor from whoever connects.
    /// Reloadable, so an operator can bind a second actor for two-person
    /// approval without restarting a cell that has work in flight.
    #[serde(default)]
    pub identity: crate::identity::IdentityBindings,
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
        config.agent.validate().map_err(|message| {
            format!("{}: invalid agent configuration: {message}", path.display())
        })?;
        Ok(config)
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
}
