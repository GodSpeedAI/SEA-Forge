//! RealityTrace as an evidence source: SEA-Forge reads RealityTrace's own ledger, read-only, through its
//! CLI, and checks an evidence packet item against the `evidence.created` record it names. SEA-Forge never
//! writes to RealityTrace and never takes a packet's direction or reliability on the requester's word.

use crate::config::CepRealityTraceConfig;
use sea_forge_authority::cep::{EvidenceSource, ObligationSource, SourceRow};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(60);

pub struct RealityTraceSource {
    config: CepRealityTraceConfig,
    /// Per-repo ledger rows, read once per request.
    exports: RefCell<HashMap<String, Vec<Value>>>,
    /// Repos whose chain verified in this request.
    verified: RefCell<Vec<String>>,
    /// The enrolled repos, listed once per request.
    repos: RefCell<Option<Vec<String>>>,
}

impl RealityTraceSource {
    pub fn new(config: CepRealityTraceConfig) -> Self {
        Self {
            config,
            exports: RefCell::default(),
            verified: RefCell::default(),
            repos: RefCell::default(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<Value, String> {
        let mut cmd = Command::new(&self.config.bin);
        cmd.args(args)
            .env_clear()
            .env("XDG_DATA_HOME", &self.config.data_home)
            .env("XDG_STATE_HOME", &self.config.state_home)
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("cannot run {}: {e}", self.config.bin))?;
        let mut stdout = child.stdout.take().expect("piped");
        let reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout.read_to_end(&mut buf);
            buf
        });
        let deadline = Instant::now() + TIMEOUT;
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break s,
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20))
                }
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("realitytrace {} timed out", args.join(" ")));
                }
                Err(e) => return Err(e.to_string()),
            }
        };
        let out = reader.join().unwrap_or_default();
        if !status.success() {
            return Err(format!("realitytrace {} failed ({status})", args.join(" ")));
        }
        serde_json::from_slice(&out)
            .map_err(|e| format!("realitytrace {} answered invalid JSON: {e}", args.join(" ")))
    }

    fn repos(&self) -> Result<Vec<String>, String> {
        if let Some(repos) = self.repos.borrow().as_ref() {
            return Ok(repos.clone());
        }
        let list = self.run(&["repo", "list", "--json"])?;
        let repos: Vec<String> = list
            .get("repos")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|r| r.get("repo_id").and_then(Value::as_str).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        *self.repos.borrow_mut() = Some(repos.clone());
        Ok(repos)
    }

    fn export(&self, repo: &str) -> Result<Vec<Value>, String> {
        if let Some(rows) = self.exports.borrow().get(repo) {
            return Ok(rows.clone());
        }
        let rows = match self.run(&["ledger", "export", "--repo", repo, "--json"])? {
            Value::Array(rows) => rows,
            _ => return Err("ledger export is not an array".into()),
        };
        self.exports
            .borrow_mut()
            .insert(repo.to_string(), rows.clone());
        Ok(rows)
    }

    fn verify_chain(&self, repo: &str) -> Result<(), String> {
        if self.verified.borrow().iter().any(|r| r == repo) {
            return Ok(());
        }
        let report = self.run(&["ledger", "verify", "--repo", repo, "--json"])?;
        if report.get("ok") != Some(&Value::Bool(true)) {
            return Err(format!(
                "RealityTrace's ledger for {repo} does not verify (first divergence: {})",
                report
                    .get("first_divergence")
                    .cloned()
                    .unwrap_or(Value::Null)
            ));
        }
        self.verified.borrow_mut().push(repo.to_string());
        Ok(())
    }
}

/// `sxr:<ledger_id>:<append_ordinal>` from an item's provenance.
fn position(item: &Value) -> Option<(String, u64)> {
    item.get("provenance_refs")?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .find_map(|r| {
            let rest = r.strip_prefix("sxr:")?;
            let (ledger, ordinal) = rest.rsplit_once(':')?;
            Some((ledger.to_string(), ordinal.parse().ok()?))
        })
}

impl EvidenceSource for RealityTraceSource {
    fn name(&self) -> &'static str {
        "realitytrace"
    }

    fn recorded_item(&self, item: &Value) -> Result<Value, String> {
        let evidence_id = item
            .get("evidence_id")
            .and_then(Value::as_str)
            .ok_or("the item has no evidence_id")?;
        let (ledger_id, ordinal) = position(item)
            .ok_or("the item names no RealityTrace ledger position (sxr:<ledger>:<ordinal>)")?;
        for repo in self.repos()? {
            let rows = self.export(&repo)?;
            let Some(row) = rows.iter().find(|r| {
                r.get("ledger_id").and_then(Value::as_str) == Some(ledger_id.as_str())
                    && r.get("append_ordinal").and_then(Value::as_u64) == Some(ordinal)
            }) else {
                continue;
            };
            self.verify_chain(&repo)?;
            if row.get("record_kind").and_then(Value::as_str) != Some("evidence.created") {
                return Err(format!(
                    "{ledger_id}:{ordinal} is not an evidence.created record"
                ));
            }
            return row
                .pointer("/payload/evidence")
                .and_then(Value::as_array)
                .and_then(|a| {
                    a.iter()
                        .find(|e| e.get("evidence_id").and_then(Value::as_str) == Some(evidence_id))
                })
                .cloned()
                .ok_or_else(|| {
                    format!("{ledger_id}:{ordinal} does not record evidence {evidence_id}")
                });
        }
        Err(format!(
            "no RealityTrace ledger holds {ledger_id}:{ordinal}"
        ))
    }
}

/// The rows of `ledger_id` in one export. A row of that ledger without a numeric `append_ordinal` or a string
/// `record_kind` fails the whole read: dropping it could hide verification evidence from settlement.
fn rows_of_ledger(export: &[Value], ledger_id: &str) -> Result<Vec<SourceRow>, String> {
    export
        .iter()
        .filter(|r| r.get("ledger_id").and_then(Value::as_str) == Some(ledger_id))
        .enumerate()
        .map(|(index, r)| {
            let ordinal = r
                .get("append_ordinal")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    format!("ledger {ledger_id} row {index} has no numeric append_ordinal")
                })?;
            let record_kind = r
                .get("record_kind")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("ledger {ledger_id}:{ordinal} has no string record_kind"))?;
            Ok(SourceRow {
                ledger_id: ledger_id.to_string(),
                ordinal,
                record_kind: record_kind.to_string(),
                payload: r.get("payload").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}

impl ObligationSource for RealityTraceSource {
    /// The rows of one RealityTrace ledger, from the repository that holds it, after its chain verified.
    fn ledger_rows(&self, ledger_id: &str) -> Result<Vec<SourceRow>, String> {
        for repo in self.repos()? {
            let rows = rows_of_ledger(&self.export(&repo)?, ledger_id)?;
            if rows.is_empty() {
                continue;
            }
            self.verify_chain(&repo)?;
            return Ok(rows);
        }
        Err(format!(
            "no enrolled RealityTrace repository holds ledger {ledger_id}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// A stand-in `realitytrace` that answers `repo list`, `ledger export` and `ledger verify` from files.
    fn fake(dir: &std::path::Path, export: &Value, verify_ok: bool) -> CepRealityTraceConfig {
        std::fs::write(
            dir.join("repos.json"),
            json!({"repos": [{"repo_id": "r1"}]}).to_string(),
        )
        .unwrap();
        std::fs::write(dir.join("export.json"), export.to_string()).unwrap();
        std::fs::write(
            dir.join("verify.json"),
            json!({"ok": verify_ok, "first_divergence": if verify_ok { Value::Null } else { json!(2) }}).to_string(),
        )
        .unwrap();
        let bin = dir.join("realitytrace");
        let d = dir.display();
        std::fs::write(
            &bin,
            format!(
                "#!/bin/sh\n[ -n \"$XDG_DATA_HOME\" ] || exit 9\ncase \"$1 $2\" in\n  \"repo list\") cat {d}/repos.json;;\n  \"ledger export\") cat {d}/export.json;;\n  \"ledger verify\") cat {d}/verify.json;;\n  *) exit 2;;\nesac\n"
            ),
        )
        .unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        CepRealityTraceConfig {
            bin: bin.display().to_string(),
            data_home: dir.display().to_string(),
            state_home: dir.display().to_string(),
            require_corroboration: true,
        }
    }

    fn ledger(kind: &str) -> Value {
        json!([
            {"ledger_id": "L", "append_ordinal": 1, "record_kind": "run.begun", "payload": {}},
            {"ledger_id": "L", "append_ordinal": 2, "record_kind": kind,
             "payload": {"evidence": [{"evidence_id": "ev-1", "direction": "contradicts", "reliability": "medium"}]}}
        ])
    }

    fn item(pos: &str) -> Value {
        json!({"evidence_id": "ev-1", "direction": "supports", "provenance_refs": [pos]})
    }

    #[test]
    fn returns_the_recorded_item_named_by_the_ledger_position() {
        let dir = tempfile::tempdir().unwrap();
        let source = RealityTraceSource::new(fake(dir.path(), &ledger("evidence.created"), true));
        let recorded = source.recorded_item(&item("sxr:L:2")).unwrap();
        // The ledger's direction, not the packet's.
        assert_eq!(recorded["direction"], "contradicts");
    }

    #[test]
    fn refuses_a_broken_chain_a_wrong_record_and_an_unknown_position() {
        let dir = tempfile::tempdir().unwrap();
        let broken = RealityTraceSource::new(fake(dir.path(), &ledger("evidence.created"), false));
        assert!(broken
            .recorded_item(&item("sxr:L:2"))
            .unwrap_err()
            .contains("does not verify"));

        let dir = tempfile::tempdir().unwrap();
        let wrong =
            RealityTraceSource::new(fake(dir.path(), &ledger("observation.recorded"), true));
        assert!(wrong
            .recorded_item(&item("sxr:L:2"))
            .unwrap_err()
            .contains("not an evidence.created"));
        assert!(wrong
            .recorded_item(&item("sxr:L:9"))
            .unwrap_err()
            .contains("no RealityTrace ledger holds"));
        assert!(wrong
            .recorded_item(&json!({"evidence_id": "ev-1"}))
            .unwrap_err()
            .contains("names no RealityTrace ledger position"));
    }

    #[test]
    fn a_missing_binary_is_an_error_not_a_pass() {
        let source = RealityTraceSource::new(CepRealityTraceConfig {
            bin: "/nonexistent/realitytrace".into(),
            data_home: "/tmp".into(),
            state_home: "/tmp".into(),
            require_corroboration: true,
        });
        assert!(source
            .recorded_item(&item("sxr:L:2"))
            .unwrap_err()
            .contains("cannot run"));
    }

    #[test]
    fn a_malformed_row_of_the_ledger_fails_the_read_instead_of_being_dropped() {
        let good = json!({"ledger_id": "L", "append_ordinal": 1, "record_kind": "run.begun", "payload": {}});
        let other = json!({"ledger_id": "M", "append_ordinal": "x"});
        let rows = rows_of_ledger(&[good.clone(), other.clone()], "L").unwrap();
        assert_eq!(
            rows.len(),
            1,
            "rows of other ledgers are ignored, even malformed ones"
        );
        let no_ordinal =
            json!({"ledger_id": "L", "append_ordinal": "2", "record_kind": "observation.recorded"});
        let err = rows_of_ledger(&[good.clone(), no_ordinal], "L").unwrap_err();
        assert!(err.contains("append_ordinal"), "{err}");
        let no_kind = json!({"ledger_id": "L", "append_ordinal": 2, "record_kind": 7});
        let err = rows_of_ledger(&[good, no_kind], "L").unwrap_err();
        assert!(err.contains("record_kind"), "{err}");
    }
}
