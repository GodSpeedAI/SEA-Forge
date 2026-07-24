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
}

fn default_max_concurrent() -> usize {
    4
}
fn default_socket_path() -> PathBuf {
    PathBuf::from(".sea-forge/server.sock")
}
fn default_approval_ttl() -> u64 {
    24
}
fn default_root() -> PathBuf {
    PathBuf::from(".sea-forge")
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
