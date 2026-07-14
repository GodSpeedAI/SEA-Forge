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
        Ok(config)
    }
}
