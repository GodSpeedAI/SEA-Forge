#![cfg(unix)]
//! RealityTrace Phase 3: brokered MCP and exposure records as audit evidence.
//!
//! A real broker session (allowed, classified, denied and unknown-tool
//! attempts) is projected with `project_attempts`. An independent auditor
//! then reconstructs every attempt from the raw ledger lines and the
//! upstream's own invocation log alone: it recomputes each payload hash with
//! the documented algorithm, checks chain linkage, matches the native log by
//! digest, and requires that exactly the forwarded attempts reached the
//! upstream. Tampered records, a dropped settlement and a forged upstream
//! line must each be caught or reported unresolved, never smoothed over.
#![cfg(target_os = "linux")]

use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_core::types::{Actor, ActorRole};
use sea_forge_ledger::LedgerEntry;
use sea_forge_realitytrace::{project_attempts, NativeObservation, TRACE_SCHEMA};
use sea_forge_server::mcp::{serve_workload, McpServerConfig};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const POLICY: &str = "version: \"0.1\"\nrules:\n  - name: reads\n    verdict: allow\n    actor_role: operator\n    operation_kind: mcp_tool_call\n    boundary_constraints:\n      mcp_server: [exposure-upstream]\n      mcp_tool: [ticket.read, metadata.read]\n      permitted_exposure: [confidential]\n  - name: external-sinks-for-unexposed-work\n    verdict: allow\n    actor_role: operator\n    operation_kind: mcp_tool_call\n    boundary_constraints:\n      mcp_server: [exposure-upstream]\n      mcp_tool: [notify.send]\n";

struct Client(BufReader<UnixStream>, u64);

impl Client {
    fn connect(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut client = Client(BufReader::new(stream), 1);
        client.rpc(
            "initialize",
            json!({"protocolVersion":"2024-11-05","capabilities":{},
                "clientInfo":{"name":"t","version":"1"}}),
        );
        writeln!(
            client.0.get_mut(),
            "{}",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .unwrap();
        client
    }
    fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.1 += 1;
        writeln!(
            self.0.get_mut(),
            "{}",
            json!({"jsonrpc":"2.0","id":self.1,"method":method,"params":params})
        )
        .unwrap();
        let mut line = String::new();
        self.0.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
    fn call(&mut self, tool: &str, arguments: Value) -> Value {
        self.rpc("tools/call", json!({"name": tool, "arguments": arguments}))
    }
}

/// Run one brokered session; returns (root, observer log path).
fn session() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let observer = root.join("observer");
    std::fs::create_dir_all(&observer).unwrap();
    std::fs::create_dir_all(root.join("workspace")).unwrap();
    let upstream = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/security-v2/exposure_upstream.py")
        .canonicalize()
        .unwrap();
    let socket = root.join("ctx.sock");
    let guard = serve_workload(
        UnixListener::bind(&socket).unwrap(),
        std::fs::metadata("/proc/self").unwrap().uid(),
        root.clone(),
        root.join("workspace"),
        sea_forge_core::yaml::from_str::<AuthorityPolicyBundle>(POLICY).unwrap(),
        Actor {
            actor_id: "support-agent".into(),
            role: ActorRole::Operator,
        },
        McpServerConfig {
            name: "exposure-upstream".into(),
            argv: vec!["/usr/bin/python3".into(), upstream.display().to_string()],
            env: [(
                "SEA_FORGE_SECURITY_OBSERVER_DIR".to_string(),
                observer.display().to_string(),
            )]
            .into(),
            classify: [("ticket.read".to_string(), "confidential".to_string())].into(),
        },
        Some("case:trace".into()),
    )
    .unwrap();
    let mut agent = Client::connect(&socket);
    // Allowed and forwarded, then classified (exposes the context), then
    // denied by exposure, then a tool no rule names.
    assert!(agent.call("metadata.read", json!({}))["result"].is_object());
    assert!(agent.call("ticket.read", json!({"id": 7731}))["result"].is_object());
    assert_eq!(
        agent.call("notify.send", json!({"message": "x"}))["error"]["code"],
        -32001
    );
    assert!(agent.call("report.publish", json!({"body": "x"}))["error"].is_object());
    drop((agent, guard));
    (dir, observer.join("invocations.jsonl"))
}

fn ledger_streams(root: &Path) -> Vec<(String, Vec<String>)> {
    let mut streams: Vec<(String, Vec<String>)> = std::fs::read_dir(root.join("ledgers"))
        .unwrap()
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("mcp-") || name == "exposure"
        })
        .map(|e| {
            let text = std::fs::read_to_string(e.path().join("entries.jsonl")).unwrap();
            (
                e.file_name().to_string_lossy().to_string(),
                text.lines().map(str::to_string).collect(),
            )
        })
        .collect();
    streams.sort();
    streams
}

fn parse(streams: &[(String, Vec<String>)]) -> Vec<LedgerEntry> {
    streams
        .iter()
        .flat_map(|(_, lines)| lines.iter().map(|l| serde_json::from_str(l).unwrap()))
        .collect()
}

/// The documented payload hash: SHA-256 over `"sea-forge/payload/v1" NUL`
/// followed by the canonical JSON payload.
fn payload_hash(payload: &Value) -> String {
    let mut bytes = b"sea-forge/payload/v1\0".to_vec();
    bytes.extend(sea_forge_ledger::canonical_json(payload).unwrap());
    format!("sha256:{}", hex::encode(Sha256::digest(&bytes)))
}

#[derive(Debug, PartialEq)]
struct Reconstruction {
    forwarded_by_tool: BTreeMap<String, usize>,
    refused_by_tool: BTreeMap<String, usize>,
    exposure_linked_to_forwarded_response: bool,
}

/// Independent auditor: raw ledger lines + the projection + the upstream's
/// raw log bytes. Trusts none of the producer's conclusions.
fn audit(
    streams: &[(String, Vec<String>)],
    projection: &Value,
    native_log: &[u8],
) -> Result<Reconstruction, String> {
    if projection["schema"] != TRACE_SCHEMA {
        return Err("unknown trace schema".into());
    }
    if let Some(open) = projection["unresolved"]
        .as_array()
        .filter(|u| !u.is_empty())
    {
        return Err(format!("unresolved: {open:?}"));
    }
    // 1. Every stream: hash-linked, every payload hash recomputed.
    let mut entries = Vec::new();
    for (name, lines) in streams {
        let mut previous: Option<String> = None;
        for line in lines {
            let entry: LedgerEntry = serde_json::from_str(line).map_err(|e| e.to_string())?;
            if entry.previous_entry_hash != previous {
                return Err(format!("{name}: chain broken at {}", entry.append_ordinal));
            }
            if entry.payload_hash != payload_hash(&entry.payload) {
                return Err(format!(
                    "{name}: payload altered at {}",
                    entry.append_ordinal
                ));
            }
            previous = Some(entry.entry_hash.clone());
            entries.push(entry);
        }
    }
    // 2. The projection must describe exactly these records.
    let projected: Vec<&Value> = projection["records"].as_array().unwrap().iter().collect();
    if projected.len() != entries.len()
        || projected
            .iter()
            .zip(&entries)
            .any(|(p, e)| p["ledger"]["payload_hash"] != e.payload_hash.as_str())
    {
        return Err("projection does not match the ledger".into());
    }
    // 3. Native observation: the bytes the auditor holds are the bytes cited.
    let digest = format!("sha256:{}", hex::encode(Sha256::digest(native_log)));
    if !projection["native_observations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["sha256"] == digest.as_str() && o["category"] == "native_observation")
    {
        return Err("native observation digest mismatch".into());
    }
    // 4. Each MCP attempt: its own decision and settlement, by ledger ref.
    let by_ulid: BTreeMap<&str, &LedgerEntry> =
        entries.iter().map(|e| (e.entry_ulid.as_str(), e)).collect();
    let mut forwarded_by_tool = BTreeMap::new();
    let mut refused_by_tool = BTreeMap::new();
    let mut forwarded_responses = Vec::new();
    for attempt in projection["attempts"].as_array().unwrap() {
        if attempt["root"]["kind"] != "mcp_attempt" {
            continue;
        }
        let root = by_ulid
            .get(attempt["root"]["entry_ulid"].as_str().unwrap_or_default())
            .ok_or("attempt root not in ledger")?;
        let tool = root.payload["tool"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let settlement = by_ulid
            .get(
                attempt["settlement"]["entry_ulid"]
                    .as_str()
                    .unwrap_or_default(),
            )
            .ok_or("attempt without settlement")?;
        let decided = root
            .authority_refs
            .iter()
            .filter_map(|r| by_ulid.get(r.as_str()))
            .find(|e| e.record_kind == "authority_decision")
            .ok_or("attempt without its decision")?;
        let allowed = decided.payload["verdict"] == "allow";
        let forwarded = settlement.payload["forwarded"] == true;
        if forwarded && !allowed {
            return Err(format!("{tool}: forwarded without an allow decision"));
        }
        if forwarded {
            *forwarded_by_tool.entry(tool).or_insert(0) += 1;
            forwarded_responses.push(settlement.payload["response_sha256"].clone());
        } else {
            *refused_by_tool.entry(tool).or_insert(0) += 1;
        }
    }
    // 5. The upstream saw exactly the forwarded calls, nothing else.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for line in String::from_utf8_lossy(native_log).lines() {
        let call: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
        *seen
            .entry(call["tool"].as_str().unwrap_or_default().to_string())
            .or_insert(0) += 1;
    }
    if seen != forwarded_by_tool {
        return Err(format!(
            "upstream saw {seen:?}, broker forwarded {forwarded_by_tool:?}"
        ));
    }
    // 6. Each exposure observation cites a response the broker forwarded.
    let exposure_linked_to_forwarded_response = entries
        .iter()
        .filter(|e| e.record_kind == "exposure_observed")
        .all(|e| {
            forwarded_responses
                .iter()
                .any(|r| r.as_str() == e.payload["evidence_sha256"].as_str())
        });
    Ok(Reconstruction {
        forwarded_by_tool,
        refused_by_tool,
        exposure_linked_to_forwarded_response,
    })
}

fn native(log: &Path) -> Vec<NativeObservation> {
    vec![NativeObservation {
        observer: "exposure-upstream".into(),
        source: "invocations.jsonl".into(),
        bytes: std::fs::read(log).unwrap(),
    }]
}

#[test]
fn auditor_reconstructs_every_brokered_attempt_from_native_evidence() {
    let (dir, log) = session();
    let streams = ledger_streams(dir.path());
    let projection = project_attempts(&parse(&streams), &native(&log));
    let native_bytes = std::fs::read(&log).unwrap();
    let got = audit(&streams, &projection, &native_bytes).unwrap();
    assert_eq!(
        got,
        Reconstruction {
            forwarded_by_tool: [("metadata.read".into(), 1), ("ticket.read".into(), 1)].into(),
            refused_by_tool: [("notify.send".into(), 1), ("report.publish".into(), 1)].into(),
            exposure_linked_to_forwarded_response: true,
        }
    );
    // Categories stay distinct: the workload's request is a claim, not
    // evidence of what happened.
    let categories: Vec<&str> = projection["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["category"].as_str())
        .collect();
    for category in [
        "claim",
        "decision",
        "enforcement",
        "settlement",
        "observation",
    ] {
        assert!(categories.contains(&category), "{category}: {categories:?}");
    }

    // Mutation 1: a settlement rewritten to "forwarded" is caught by hash.
    let mut tampered = streams.clone();
    for (_, lines) in tampered.iter_mut() {
        for line in lines.iter_mut() {
            if line.contains("\"authority_not_granted\"") {
                *line = line.replace("\"forwarded\":false", "\"forwarded\":true");
            }
        }
    }
    let err = audit(&tampered, &projection, &native_bytes).unwrap_err();
    assert!(err.contains("payload altered"), "{err}");

    // Mutation 2: a dropped settlement is unresolved, not inferred.
    let mut entries = parse(&streams);
    let dropped = entries
        .iter()
        .position(|e| e.record_kind == "mcp_settlement")
        .unwrap();
    entries.remove(dropped);
    let partial = project_attempts(&entries, &native(&log));
    assert!(partial["unresolved"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u.as_str().unwrap().starts_with("unsettled:")));
    assert!(audit(&streams, &partial, &native_bytes).is_err());

    // Mutation 3: an upstream log claiming a denied call ran is caught,
    // both by digest and by the forwarded-versus-seen reconciliation.
    let mut forged = native_bytes.clone();
    forged.extend(br#"{"tool": "notify.send", "carries_secret": true}"#);
    forged.push(b'\n');
    let err = audit(&streams, &projection, &forged).unwrap_err();
    assert!(err.contains("native observation digest mismatch"), "{err}");
    let reprojected = project_attempts(
        &parse(&streams),
        &[NativeObservation {
            observer: "exposure-upstream".into(),
            source: "invocations.jsonl".into(),
            bytes: forged.clone(),
        }],
    );
    let err = audit(&streams, &reprojected, &forged).unwrap_err();
    assert!(err.contains("upstream saw"), "{err}");
}
