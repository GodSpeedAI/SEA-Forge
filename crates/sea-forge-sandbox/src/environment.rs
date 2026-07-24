//! EnvironmentSpec contracts (spec-full §7.6): declarative sandbox content
//! orthogonal to isolation and permission. Hash-pinned immutable; mismatch ⇒
//! `environment_unavailable`. Three orthogonal axes:
//!   - **content** — `provides.commands` declares what the env supplies.
//!   - **permission** — policy `execute_command` rules decide allow/deny.
//!   - **isolation** — `sandbox_class` decides jail/local/microvm.
//!
//! Field semantics stay compatible with DomainForge ADR-012 `Cell` declarations
//! (Mount→base files, Endpoint/NetworkFlow→deny-by-default, Credential→authority
//! contract) so a future `cell→EnvironmentSpec` projection adapter needs no
//! schema break. Building that projection is NOT in scope.

use sea_forge_core::{errors::ForgeError, types::ExecutionResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;

/// An environment specification (YAML, `.sea-forge/environments/<name>@<version>.yaml`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnvironmentSpec {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// Sandbox template inputs materialized into the workspace at prepare time.
    #[serde(default)]
    pub base: Vec<BaseFile>,
    pub provides: Provides,
    #[serde(default)]
    pub evaluators: BTreeMap<String, Evaluator>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Provides {
    /// argv0 basenames this environment supplies.
    pub commands: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BaseFile {
    pub path: String,
    pub content: String,
}

/// Evaluator: runs inside the same sandbox/episode under the same authority.
/// Its stdout/exit becomes a score in `[0,1]`.
// ponytail: Predicate variant deferred — M7 proof only needs command form.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evaluator {
    Command {
        argv: Vec<String>,
        score_from: ScoreFrom,
    },
}

/// How to derive a score from a command evaluator's result.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScoreFrom {
    Exit,
    StdoutFloat,
}

/// Parse `name@version` into its parts.
pub fn parse_env_ref(reference: &str) -> Result<(&str, &str), ForgeError> {
    let (name, version) = reference
        .split_once('@')
        .ok_or_else(|| ForgeError::Input(format!("invalid environment reference {reference}")))?;
    if name.is_empty() || version.is_empty() {
        return Err(ForgeError::Input(format!(
            "invalid environment reference {reference}"
        )));
    }
    Ok((name, version))
}

/// Load an EnvironmentSpec from `<state-root>/environments/<name>@<version>.yaml`
/// with hash-pinning. First use pins the SHA-256; subsequent loads reject on
/// content change (§7.6: `environment_unavailable`).
pub fn load_pinned_environment(
    root: &Path,
    reference: &str,
) -> Result<EnvironmentSpec, ForgeError> {
    let (name, version) = parse_env_ref(reference)?;
    let envs_dir = root.join("environments");
    let path = envs_dir.join(format!("{name}@{version}.yaml"));
    let bytes = fs::read(&path).map_err(|_| ForgeError::Config {
        class: "environment_unavailable",
        path: path.clone(),
        message: format!("environment {reference} not found"),
    })?;
    let digest = format!("sha256:{:x}", Sha256::digest(&bytes));
    let envs_dir_clone = envs_dir.clone();
    let pins = envs_dir_clone.join(".pins");
    fs::create_dir_all(&pins).map_err(|e| ForgeError::io("create env pins directory", e))?;
    let pin = pins.join(format!("{name}@{version}.sha256"));
    match fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&pin)
    {
        Ok(mut file) => {
            use std::io::Write;
            file.write_all(digest.as_bytes())
                .map_err(|e| ForgeError::io("write env pin", e))?;
        }
        Err(_) => {
            let expected =
                fs::read_to_string(&pin).map_err(|e| ForgeError::io("read env pin", e))?;
            if expected != digest {
                return Err(ForgeError::Config {
                    class: "environment_unavailable",
                    path,
                    message: "environment bytes changed without a version bump".into(),
                });
            }
        }
    }
    let spec: EnvironmentSpec = serde_yaml::from_slice(&bytes).map_err(|e| ForgeError::Config {
        class: "environment_unavailable",
        path: path.clone(),
        message: format!("invalid environment yaml: {e}"),
    })?;
    if spec.name != name || spec.version != version {
        return Err(ForgeError::Config {
            class: "environment_unavailable",
            path,
            message: "environment name/version do not match its filename".into(),
        });
    }
    Ok(spec)
}

/// Store the built-in `demo_env@0.1.0` if absent (mirrors planner templates).
pub fn store_builtin_environment(root: &Path) -> Result<std::path::PathBuf, ForgeError> {
    let envs_dir = root.join("environments");
    fs::create_dir_all(&envs_dir).map_err(|e| ForgeError::io("create envs dir", e))?;
    let path = envs_dir.join("demo_env@0.1.0.yaml");
    if !path.exists() {
        let spec = demo_environment();
        let yaml = serde_yaml::to_string(&spec)
            .map_err(|e| ForgeError::Internal(format!("serialize demo_env: {e}")))?;
        fs::write(&path, yaml).map_err(|e| ForgeError::io("write demo_env", e))?;
    }
    Ok(path)
}

/// The built-in demo environment fixture (§12 M7 proof scenario).
pub fn demo_environment() -> EnvironmentSpec {
    EnvironmentSpec {
        name: "demo_env".into(),
        version: "0.1.0".into(),
        description: "Built-in demo environment for M7 proofs".into(),
        base: vec![],
        provides: Provides {
            commands: vec!["sea-forge".into()],
        },
        evaluators: [(
            "score_eval".into(),
            Evaluator::Command {
                argv: vec!["sea-forge".into(), "internal-test-sleep".into(), "0".into()],
                score_from: ScoreFrom::Exit,
            },
        )]
        .into_iter()
        .collect(),
    }
}

/// Materialize `base` files into the workspace via safe_join.
pub fn materialize_base(spec: &EnvironmentSpec, workspace: &Path) -> Result<(), ForgeError> {
    for file in &spec.base {
        let dest = crate::safe_join(workspace, &file.path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| ForgeError::io("create base parent", e))?;
        }
        fs::write(&dest, &file.content)
            .map_err(|e| ForgeError::io(format!("write base file {}", file.path), e))?;
    }
    Ok(())
}

/// Parse an evaluator score from an execution result.
/// `Exit` → 1.0 if exit_code == 0, else 0.0.
/// `StdoutFloat` → parse stdout as f64, clamp to [0,1].
pub fn parse_evaluator_score(
    score_from: &ScoreFrom,
    result: &ExecutionResult,
    artifacts: &Path,
) -> Result<f64, ForgeError> {
    match score_from {
        ScoreFrom::Exit => Ok(if result.exit_code == Some(0) {
            1.0
        } else {
            0.0
        }),
        ScoreFrom::StdoutFloat => {
            let stdout_path = crate::safe_existing(artifacts, &result.stdout_path)?;
            let mut content = String::new();
            fs::File::open(&stdout_path)
                .and_then(|mut f| f.read_to_string(&mut content))
                .map_err(|e| ForgeError::io("read evaluator stdout", e))?;
            let value: f64 = content.trim().parse().map_err(|_| ForgeError::Config {
                class: "evaluator_score_error",
                path: stdout_path,
                message: format!("stdout is not a float: {content:?}"),
            })?;
            Ok(value.clamp(0.0, 1.0))
        }
    }
}
