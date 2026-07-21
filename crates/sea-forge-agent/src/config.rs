use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    OpenAiCompatible,
    Anthropic,
    Acp,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EndpointStatus {
    Declared,
    Probed,
    Demonstrated,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentEndpointConfig {
    pub id: String,
    pub kind: ProviderKind,
    pub base_url: Option<String>,
    /// Tokenized argv for `acp`-kind endpoints (spec §7.1: required for ACP,
    /// never shell-invoked). Empty/absent for HTTP kinds.
    #[serde(default)]
    pub argv: Vec<String>,
    /// Minimal explicit environment passed to the ACP child (KEY=VALUE). The
    /// parent environment is never inherited (spec §15).
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub credential_ref: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub allow_loopback_test: bool,
    #[serde(default = "default_max_request_bytes")]
    pub max_request_bytes: usize,
    #[serde(default = "default_max_response_bytes")]
    pub max_response_bytes: usize,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub status: Option<EndpointStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    #[serde(default)]
    pub endpoints: Vec<AgentEndpointConfig>,
    #[serde(default = "default_max_request_bytes")]
    pub max_request_bytes: usize,
    #[serde(default = "default_max_response_bytes")]
    pub max_response_bytes: usize,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            endpoints: Vec::new(),
            max_request_bytes: default_max_request_bytes(),
            max_response_bytes: default_max_response_bytes(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

fn default_max_request_bytes() -> usize {
    1_048_576
}

fn default_max_response_bytes() -> usize {
    4_194_304
}

fn default_timeout_secs() -> u64 {
    60
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndpointSnapshot {
    pub id: String,
    pub kind: ProviderKind,
    pub base_url: Url,
    /// Tokenized argv for ACP endpoints (empty for HTTP kinds).
    pub argv: Vec<String>,
    /// Minimal explicit env for the ACP child (empty for HTTP kinds).
    pub env: Vec<String>,
    pub credential_ref: Option<String>,
    pub model: String,
    pub descriptor_config_sha256: String,
    pub max_request_bytes: usize,
    pub max_response_bytes: usize,
    pub timeout: Duration,
    pub allow_loopback_test: bool,
}

impl AgentConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_request_bytes == 0 || self.max_response_bytes == 0 || self.timeout_secs == 0 {
            return Err("agent limits must be non-zero".into());
        }
        let mut ids = std::collections::BTreeSet::new();
        for endpoint in &self.endpoints {
            endpoint.validate(&ids)?;
            ids.insert(endpoint.id.clone());
        }
        Ok(())
    }

    pub fn endpoint(&self, id: &str) -> Option<&AgentEndpointConfig> {
        self.endpoints.iter().find(|endpoint| endpoint.id == id)
    }
}

impl AgentEndpointConfig {
    fn validate(&self, ids: &std::collections::BTreeSet<String>) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-'))
            || !ids.is_disjoint(&std::iter::once(self.id.clone()).collect())
        {
            return Err(format!(
                "invalid or duplicate agent endpoint id '{}'",
                self.id
            ));
        }
        if self.status.is_some() {
            return Err(format!("endpoint '{}' status is evidence-derived", self.id));
        }
        match self.kind {
            ProviderKind::Acp => {
                if self.argv.is_empty() || self.argv[0].is_empty() {
                    return Err(format!(
                        "endpoint '{}' of kind acp requires a non-empty argv[0]",
                        self.id
                    ));
                }
                let executable = self.argv[0]
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(&self.argv[0])
                    .to_ascii_lowercase();
                if matches!(
                    executable.as_str(),
                    "sh" | "bash"
                        | "dash"
                        | "zsh"
                        | "fish"
                        | "cmd"
                        | "cmd.exe"
                        | "powershell"
                        | "powershell.exe"
                        | "pwsh"
                        | "pwsh.exe"
                ) {
                    return Err(format!(
                        "endpoint '{}' ACP argv[0] may not be a shell",
                        self.id
                    ));
                }
                // Minimal explicit env: each entry must be KEY=VALUE.
                for entry in &self.env {
                    if entry.is_empty() || !entry.contains('=') {
                        return Err(format!(
                            "endpoint '{}' env entry must be KEY=VALUE",
                            self.id
                        ));
                    }
                }
                // base_url optional for ACP; if absent we synthesize an
                // acp:// descriptor during snapshot (no network meaning).
            }
            ProviderKind::OpenAiCompatible | ProviderKind::Anthropic => {
                let raw = self
                    .base_url
                    .as_deref()
                    .ok_or_else(|| format!("endpoint '{}' requires base_url", self.id))?;
                let url = Url::parse(raw).map_err(|e| format!("invalid endpoint URL: {e}"))?;
                if url.host_str().is_none() || url.username() != "" || url.password().is_some() {
                    return Err(format!(
                        "endpoint '{}' URL must have a host and no credentials",
                        self.id
                    ));
                }
                if url.scheme() != "https"
                    && !(self.allow_loopback_test
                        && url.scheme() == "http"
                        && url.host_str().is_some_and(is_loopback_host))
                {
                    return Err(format!("endpoint '{}' requires HTTPS", self.id));
                }
            }
        }
        if self.max_request_bytes == 0 || self.max_response_bytes == 0 || self.timeout_secs == 0 {
            return Err(format!("endpoint '{}' has invalid limits", self.id));
        }
        if self
            .credential_ref
            .as_deref()
            .is_some_and(|value| value.is_empty() || !valid_credential_ref(value))
        {
            return Err(format!("endpoint '{}' has invalid credential_ref", self.id));
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Result<EndpointSnapshot, String> {
        self.validate(&std::collections::BTreeSet::new())?;
        let base_url = match self.kind {
            ProviderKind::Acp => {
                // Synthetic descriptor: acp://argv/<argv0-basename>. Carries no
                // network meaning — ACP is a child process, not an HTTP call.
                let argv0 = self.argv[0].rsplit('/').next().unwrap_or(&self.argv[0]);
                Url::parse(&format!("acp://argv/{argv0}"))
                    .map_err(|e| format!("invalid synthesized acp URL: {e}"))?
            }
            _ => Url::parse(self.base_url.as_deref().unwrap_or_default())
                .map_err(|e| format!("invalid endpoint URL: {e}"))?,
        };
        let model = self
            .default_model
            .clone()
            .unwrap_or_else(|| "default".into());
        let descriptor_config_sha256 = descriptor_config_hash(self)?;
        Ok(EndpointSnapshot {
            id: self.id.clone(),
            kind: self.kind.clone(),
            base_url,
            argv: self.argv.clone(),
            env: self.env.clone(),
            credential_ref: self.credential_ref.clone(),
            model,
            descriptor_config_sha256,
            max_request_bytes: self.max_request_bytes,
            max_response_bytes: self.max_response_bytes,
            timeout: Duration::from_secs(self.timeout_secs),
            allow_loopback_test: self.allow_loopback_test,
        })
    }
}

fn valid_credential_ref(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn is_loopback_host(host: &str) -> bool {
    host == "localhost"
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

pub(crate) fn descriptor_config_hash(config: &AgentEndpointConfig) -> Result<String, String> {
    let mut canonical = config.clone();
    canonical.status = None;
    let bytes = serde_json::to_vec(&canonical).map_err(|e| e.to_string())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(kind: ProviderKind) -> AgentEndpointConfig {
        AgentEndpointConfig {
            id: "local-test".into(),
            kind,
            base_url: Some("http://127.0.0.1:8080/".into()),
            argv: vec!["/usr/bin/false".into()],
            env: vec![],
            credential_ref: Some("OPENAI_API_KEY".into()),
            default_model: Some("test-model".into()),
            allow_loopback_test: true,
            max_request_bytes: 1024,
            max_response_bytes: 2048,
            timeout_secs: 5,
            status: None,
        }
    }

    #[test]
    fn config_rejects_asserted_derived_status() {
        let mut endpoint = endpoint(ProviderKind::OpenAiCompatible);
        endpoint.status = Some(EndpointStatus::Probed);
        assert!(endpoint
            .snapshot()
            .unwrap_err()
            .contains("evidence-derived"));
    }

    #[test]
    fn snapshot_hash_changes_when_endpoint_config_changes() {
        let endpoint = endpoint(ProviderKind::OpenAiCompatible);
        let first = endpoint.snapshot().unwrap();
        let mut changed = endpoint;
        changed.default_model = Some("other-model".into());
        assert_ne!(
            first.descriptor_config_sha256,
            changed.snapshot().unwrap().descriptor_config_sha256
        );
    }

    #[test]
    fn acp_endpoint_requires_argv_and_synthesizes_descriptor_url() {
        let mut endpoint = endpoint(ProviderKind::Acp);
        // Missing argv rejected.
        endpoint.argv = vec![];
        let err = endpoint.snapshot().unwrap_err();
        assert!(err.contains("argv"), "{err}");

        // Valid argv snapshots with a synthetic acp:// descriptor URL.
        endpoint.argv = vec!["/usr/bin/goose".into(), "acp".into()];
        let snap = endpoint.snapshot().unwrap();
        assert_eq!(snap.base_url.scheme(), "acp");
        assert_eq!(snap.base_url.host_str(), Some("argv"));
        assert_eq!(snap.argv, vec!["/usr/bin/goose".to_string(), "acp".into()]);
    }

    #[test]
    fn acp_env_entries_must_be_key_value() {
        let mut endpoint = endpoint(ProviderKind::Acp);
        endpoint.argv = vec!["/usr/bin/goose".into()];
        endpoint.env = vec!["NOEQUALS".into()];
        let err = endpoint.snapshot().unwrap_err();
        assert!(err.contains("KEY=VALUE"), "{err}");
        endpoint.env = vec!["PATH=/usr/bin".into()];
        assert!(endpoint.snapshot().is_ok());
    }

    #[test]
    fn acp_endpoint_rejects_shell_argv0() {
        let mut endpoint = endpoint(ProviderKind::Acp);
        endpoint.argv = vec!["/bin/sh".into(), "-c".into(), "agent".into()];
        let err = endpoint.snapshot().unwrap_err();
        assert!(err.contains("may not be a shell"), "{err}");
    }
}
