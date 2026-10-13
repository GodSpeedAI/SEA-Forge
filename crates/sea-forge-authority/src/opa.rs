//! OPA candidate: runs the pinned `opa` binary over a DomainForge-generated
//! Rego bundle and turns its decision into a governance candidate.
//!
//! Fail-closed contract: configuration and bundle-integrity problems are
//! `Err`. Every evaluation problem (missing binary, version drift, timeout,
//! malformed or inconsistent decision) becomes a `Deny` candidate that carries
//! a failure code, so an absent or broken decision can never resolve to Allow.

use crate::{AuthorityPolicyBundle, GovernanceDisposition, PolicySource};
use sea_forge_core::{canonical::canonical_bytes, errors::ForgeError};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    io::{ErrorKind, Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// The pinned OPA CLI version. Any other `opa version` is refused.
pub const OPA_VERSION: &str = "1.21.1";
/// The decision contract every DomainForge Rego bundle reports.
pub const REGO_DECISION_CONTRACT: &str = "domainforge.rego.decision/v1";

const PROVENANCE_SCHEMA: &str = "domainforge.rego.provenance/v1";
const QUERY: &str = "data.domainforge.policy.decision";

/// Where the `opa` binary lives and how long one invocation may run.
#[derive(Debug, Clone)]
pub struct OpaBinding {
    /// Absolute path to the `opa` executable.
    pub cli: PathBuf,
    pub timeout: Duration,
}

/// Everything needed to audit one OPA evaluation.
#[derive(Debug, Clone, Serialize)]
pub struct OpaTrace {
    pub opa_version: String,
    pub model_hash: String,
    pub projection: String,
    pub provenance_sha256: String,
    pub input_sha256: String,
    pub decision_sha256: Option<String>,
    pub outcome: Option<String>,
    pub reasons: Value,
    pub results: Value,
    pub failure: Option<String>,
}

/// One OPA evaluation, reduced to a governance disposition plus its trace.
#[derive(Debug)]
pub struct OpaCandidate {
    disposition: GovernanceDisposition,
    trace: OpaTrace,
}

impl OpaCandidate {
    /// Verifies the DomainForge Rego bundle named by the policy's
    /// `rego_provenance` source, then asks `opa` for a decision over `input`.
    pub fn evaluate(
        bundle: &AuthorityPolicyBundle,
        binding: &OpaBinding,
        input: &Value,
    ) -> Result<Self, ForgeError> {
        if !bundle
            .policy_engines
            .iter()
            .any(|engine| engine.engine == "opa")
        {
            return Err(config(
                "opa_engine_not_configured",
                Path::new("<policy>"),
                "policy does not configure the opa engine",
            ));
        }
        let source: &PolicySource = bundle
            .sources
            .iter()
            .find(|source| source.surface == "rego_provenance")
            .ok_or_else(|| {
                config(
                    "opa_bundle_missing",
                    Path::new("<policy>"),
                    "policy has no rego_provenance source",
                )
            })?;
        if !binding.cli.is_absolute() {
            return Err(config(
                "opa_binding_invalid",
                &binding.cli,
                "opa binary path must be absolute",
            ));
        }
        let base = bundle
            .source_base
            .as_deref()
            .unwrap_or_else(|| Path::new("."));
        let provenance_path = base.join(&source.path);
        let provenance_bytes = fs::read(&provenance_path)
            .map_err(|error| ForgeError::io("read rego provenance", error))?;
        if source.sha256 != sha256_hex(&provenance_bytes) {
            return Err(config(
                "source_hash_mismatch",
                &provenance_path,
                "rego_provenance source hash mismatch",
            ));
        }
        let bundle_files = verify_bundle(&provenance_path, &provenance_bytes)?;

        let input_bytes = canonical_bytes(input)?;
        let mut trace = OpaTrace {
            opa_version: OPA_VERSION.into(),
            model_hash: bundle_files.model_hash.clone(),
            projection: bundle_files.projection,
            provenance_sha256: sha256_prefixed(&provenance_bytes),
            input_sha256: sha256_prefixed(&input_bytes),
            decision_sha256: None,
            outcome: None,
            reasons: Value::Null,
            results: Value::Null,
            failure: None,
        };
        let model_hash = bundle_files.model_hash;
        let outcome = if !facts_object(input) {
            Err("opa_input_invalid")
        } else {
            query(binding, &bundle_files.dir, &input_bytes, &mut trace).and_then(|decision| {
                trace.outcome = decision
                    .get("outcome")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                trace.reasons = decision.get("reasons").cloned().unwrap_or(Value::Null);
                trace.results = decision.get("results").cloned().unwrap_or(Value::Null);
                validate(&decision, &model_hash)
            })
        };
        Ok(match outcome {
            Ok(disposition) => Self { disposition, trace },
            Err(code) => {
                trace.failure = Some(code.into());
                Self {
                    disposition: GovernanceDisposition::Deny,
                    trace,
                }
            }
        })
    }

    pub fn trace(&self) -> &OpaTrace {
        &self.trace
    }

    pub(crate) fn disposition(&self) -> GovernanceDisposition {
        self.disposition.clone()
    }

    /// The verdict reason: the decision outcome, or the failure code.
    pub(crate) fn reason(&self) -> String {
        match &self.trace.failure {
            Some(failure) => format!("opa:{failure}"),
            None => format!("opa:{}", self.trace.outcome.as_deref().unwrap_or("")),
        }
    }

    /// Evidence for the verdict. Never empty, so resolution cannot mistake it
    /// for a candidate without evidence.
    pub(crate) fn evidence_refs(&self) -> Vec<String> {
        let trace = &self.trace;
        let mut refs = vec![
            format!("rego-bundle:{}", trace.model_hash),
            format!("opa:{}", trace.opa_version),
            format!("rego-input:{}", trace.input_sha256),
        ];
        if let Some(decision) = &trace.decision_sha256 {
            refs.push(format!("rego-decision:{decision}"));
        }
        if let Some(failure) = &trace.failure {
            refs.push(format!("opa-failure:{failure}"));
        }
        refs
    }
}

struct VerifiedBundle {
    dir: PathBuf,
    model_hash: String,
    projection: String,
}

fn config(class: &'static str, path: &Path, message: impl Into<String>) -> ForgeError {
    ForgeError::Config {
        class,
        path: path.into(),
        message: message.into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// Checks the provenance manifest against the files on disk: every listed
/// file must match its digest, and `bundle/` must contain exactly the listed
/// regular files, with no symlinks, extras or omissions.
fn verify_bundle(
    provenance_path: &Path,
    provenance_bytes: &[u8],
) -> Result<VerifiedBundle, ForgeError> {
    let dir = provenance_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let tampered = |message: String| config("opa_bundle_tampered", provenance_path, message);
    let provenance: Value = serde_json::from_slice(provenance_bytes)
        .map_err(|_| tampered("provenance.json is not JSON".into()))?;
    if provenance.get("schema").and_then(Value::as_str) != Some(PROVENANCE_SCHEMA) {
        return Err(tampered("unsupported provenance schema".into()));
    }
    let model_hash = provenance
        .pointer("/model/hash")
        .and_then(Value::as_str)
        .ok_or_else(|| tampered("provenance records no model hash".into()))?
        .to_owned();
    let projection = provenance
        .pointer("/model/projection")
        .and_then(Value::as_str)
        .ok_or_else(|| tampered("provenance records no projection".into()))?
        .to_owned();
    let files = provenance
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| tampered("provenance lists no files".into()))?;

    let mut listed = BTreeSet::new();
    for (rel, expected) in files {
        if !safe_relative(rel) {
            return Err(tampered(format!("unsafe provenance path {rel}")));
        }
        let expected = expected
            .as_str()
            .and_then(|digest| digest.strip_prefix("sha256:"))
            .ok_or_else(|| tampered(format!("malformed digest for {rel}")))?;
        let path = dir.join(rel);
        let meta = fs::symlink_metadata(&path)
            .map_err(|_| tampered(format!("listed file {rel} is missing")))?;
        if !meta.is_file() {
            return Err(tampered(format!("listed file {rel} is not a regular file")));
        }
        let bytes = fs::read(&path).map_err(|_| tampered(format!("cannot read {rel}")))?;
        if sha256_hex(&bytes) != expected {
            return Err(tampered(format!("digest mismatch for {rel}")));
        }
        if rel.starts_with("bundle/") {
            listed.insert(rel.clone());
        }
    }

    let mut found = BTreeSet::new();
    walk_bundle(&dir.join("bundle"), "bundle", &mut found).map_err(tampered)?;
    if found != listed {
        return Err(tampered(
            "bundle/ contents differ from the provenance file list".into(),
        ));
    }
    let manifest: Value = fs::read(dir.join("bundle").join(".manifest"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .ok_or_else(|| tampered("bundle manifest is unreadable".into()))?;
    if manifest.get("revision").and_then(Value::as_str) != Some(model_hash.as_str()) {
        return Err(tampered(
            "manifest revision does not match the provenance model hash".into(),
        ));
    }
    Ok(VerifiedBundle {
        dir,
        model_hash,
        projection,
    })
}

fn walk_bundle(dir: &Path, rel: &str, found: &mut BTreeSet<String>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|error| format!("cannot read {rel}: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| format!("non-UTF-8 name under {rel}"))?;
        let child = format!("{rel}/{name}");
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            return Err(format!("symlink {child}"));
        } else if kind.is_dir() {
            walk_bundle(&entry.path(), &child, found)?;
        } else if kind.is_file() {
            found.insert(child);
        } else {
            return Err(format!("unsupported entry {child}"));
        }
    }
    Ok(())
}

fn facts_object(input: &Value) -> bool {
    input.get("facts").is_some_and(Value::is_object)
}

/// Runs `opa version` and then the decision query. Returns the decision
/// object, or the failure code.
fn query(
    binding: &OpaBinding,
    dir: &Path,
    input_bytes: &[u8],
    trace: &mut OpaTrace,
) -> Result<Value, &'static str> {
    let version = run(
        &binding.cli,
        &[OsStr::new("version")],
        dir,
        None,
        binding.timeout,
    )?;
    if !version.success {
        return Err("opa_exit_nonzero");
    }
    let observed = version
        .stdout
        .lines()
        .find_map(|line| line.strip_prefix("Version:"))
        .map(str::trim);
    trace.opa_version = observed.unwrap_or(OPA_VERSION).into();
    if observed != Some(OPA_VERSION) {
        return Err("opa_version_mismatch");
    }

    let bundle_dir = dir.join("bundle");
    let args = [
        OsStr::new("eval"),
        OsStr::new("--format"),
        OsStr::new("json"),
        OsStr::new("--bundle"),
        bundle_dir.as_os_str(),
        OsStr::new("--stdin-input"),
        OsStr::new(QUERY),
    ];
    let eval = run(&binding.cli, &args, dir, Some(input_bytes), binding.timeout)?;
    if !eval.success {
        return Err("opa_exit_nonzero");
    }
    let decision = decision_value(&eval.stdout)?;
    let bytes = canonical_bytes(&decision).map_err(|_| "opa_output_invalid")?;
    trace.decision_sha256 = Some(sha256_prefixed(&bytes));
    Ok(decision)
}

/// Extracts the single decision object from `opa eval --format json` output.
fn decision_value(stdout: &str) -> Result<Value, &'static str> {
    let output: Value = serde_json::from_str(stdout).map_err(|_| "opa_output_invalid")?;
    if !output.is_object() {
        return Err("opa_output_invalid");
    }
    let results = match output.get("result") {
        None => return Err("opa_decision_undefined"),
        Some(Value::Array(results)) => results,
        Some(_) => return Err("opa_output_invalid"),
    };
    let result = match results.as_slice() {
        [] => return Err("opa_decision_undefined"),
        [result] => result,
        _ => return Err("opa_output_invalid"),
    };
    let expressions = result
        .get("expressions")
        .and_then(Value::as_array)
        .ok_or("opa_output_invalid")?;
    let expression = match expressions.as_slice() {
        [] => return Err("opa_decision_undefined"),
        [expression] => expression,
        _ => return Err("opa_output_invalid"),
    };
    match expression.get("value") {
        Some(value) if value.is_object() => Ok(value.clone()),
        _ => Err("opa_output_invalid"),
    }
}

/// Checks the decision contract and maps the outcome to a disposition.
fn validate(decision: &Value, model_hash: &str) -> Result<GovernanceDisposition, &'static str> {
    if decision.get("contract").and_then(Value::as_str) != Some(REGO_DECISION_CONTRACT) {
        return Err("opa_contract_mismatch");
    }
    if decision.pointer("/model/hash").and_then(Value::as_str) != Some(model_hash) {
        return Err("opa_model_mismatch");
    }
    let invalid = "opa_decision_invalid";
    let outcome = decision
        .get("outcome")
        .and_then(Value::as_str)
        .ok_or(invalid)?;
    let results = decision
        .get("results")
        .and_then(Value::as_object)
        .ok_or(invalid)?;
    if !results
        .values()
        .all(|value| matches!(value.as_str(), Some("true" | "false" | "unknown" | "error")))
    {
        return Err(invalid);
    }
    if !decision.get("reasons").is_some_and(Value::is_array) {
        return Err(invalid);
    }
    let input_errors = decision
        .get("input_errors")
        .and_then(Value::as_array)
        .ok_or(invalid)?;
    let disposition = match outcome {
        "allow" => GovernanceDisposition::Allow,
        "deny" => GovernanceDisposition::Deny,
        "escalate" => GovernanceDisposition::Escalate,
        _ => return Err(invalid),
    };
    let inconsistent_allow = results
        .values()
        .any(|value| matches!(value.as_str(), Some("unknown" | "error")));
    if disposition == GovernanceDisposition::Allow
        && (inconsistent_allow || !input_errors.is_empty())
    {
        return Err(invalid);
    }
    Ok(disposition)
}

struct Output {
    success: bool,
    stdout: String,
}

/// Runs `opa` with a clean environment, an optional stdin payload and a
/// deadline. The child is killed when the deadline passes.
fn run(
    cli: &Path,
    args: &[&OsStr],
    cwd: &Path,
    stdin: Option<&[u8]>,
    timeout: Duration,
) -> Result<Output, &'static str> {
    let mut command = Command::new(cli);
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // ETXTBSY: another thread forked while the executable was still open for
    // writing (a just-installed binary). It clears as soon as that fork execs.
    let mut attempts = 0;
    let mut child = loop {
        match command.spawn() {
            Ok(child) => break child,
            Err(error) if error.kind() == ErrorKind::ExecutableFileBusy && attempts < 20 => {
                attempts += 1;
                thread::sleep(Duration::from_millis(25));
            }
            Err(_) => return Err("opa_unavailable"),
        }
    };
    if let (Some(bytes), Some(mut pipe)) = (stdin, child.stdin.take()) {
        let bytes = bytes.to_vec();
        // Written from its own thread so a large input cannot deadlock against
        // a child that is still writing its output.
        thread::spawn(move || {
            let _ = pipe.write_all(&bytes);
        });
    }
    let stdout = child.stdout.take().map(drain);
    let _stderr = child.stderr.take().map(drain);
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("opa_timeout");
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("opa_unavailable");
            }
        }
    };
    let stdout = stdout
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    Ok(Output {
        success: status.success(),
        stdout,
    })
}

fn drain<R: Read + Send + 'static>(mut pipe: R) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    })
}
