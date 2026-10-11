use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use sea_forge_core::{errors::ForgeError, ids::random_id, types::*, RECORD_VERSION};
use sea_forge_ledger::{types::hash_canonical, CommittedRecordRef, LedgerStream};

/// Compute the declaration hash over canonical content excluding `declaration_hash`.
pub fn compute_declaration_hash(decl: &SettlementDeclaration) -> Result<String, ForgeError> {
    let mut copy = decl.clone();
    copy.declaration_hash.clear();
    hash_canonical(&copy)
}

/// Append a declaration to a JSONL file.
pub fn append_declaration(path: &Path, decl: &SettlementDeclaration) -> Result<(), ForgeError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut encoded = serde_json::to_vec(decl)?;
    encoded.push(b'\n');
    file.write_all(&encoded)
        .and_then(|_| file.flush())
        .map_err(|e| ForgeError::io("append declaration", e))
}

/// Commit a declaration to the M0 ledger before appending its compatibility view.
pub fn append_declaration_ledgered(
    root: &Path,
    view_path: &Path,
    actor: &str,
    decl: &SettlementDeclaration,
) -> Result<(), ForgeError> {
    if compute_declaration_hash(decl)? != decl.declaration_hash {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "declaration_hash mismatch".into(),
        });
    }
    LedgerStream::open(root, format!("case-{}", decl.case_id), actor)?.commit_typed(
        "settlement_declaration",
        vec![
            decl.case_id.clone(),
            decl.settlement_ref.clone(),
            decl.declaration_id.clone(),
        ],
        decl,
        vec![decl.declarer.authority_ref.clone()],
    )?;
    append_declaration(view_path, decl)
}

pub fn append_declaration_ledgered_once(
    root: &Path,
    view_path: &Path,
    actor: &str,
    decl: &SettlementDeclaration,
) -> Result<CommittedRecordRef, ForgeError> {
    if compute_declaration_hash(decl)? != decl.declaration_hash {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "declaration_hash mismatch".into(),
        });
    }
    let idempotency_key = hash_canonical(&(
        &decl.case_id,
        &decl.run_id,
        &decl.claim_manifest_sha256,
        &decl.declarer.authority_ref,
        &decl.settlement_ref,
        &decl.plan_item_id,
    ))?;
    let committed = LedgerStream::open(root, format!("case-{}", decl.case_id), actor)?
        .commit_typed_once(
            "settlement_declaration",
            idempotency_key,
            vec![decl.case_id.clone(), decl.run_id.clone()],
            decl,
            vec![decl.declarer.authority_ref.clone()],
        )?;
    append_declaration_view_once(view_path, decl)?;
    Ok(committed)
}

/// Atomically check-and-append a declaration view under a file lock.
/// Same declaration_id with identical content is a no-op; same ID with different
/// content is rejected. The lock covers both read and write.
fn append_declaration_view_once(
    path: &Path,
    decl: &SettlementDeclaration,
) -> Result<(), ForgeError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| ForgeError::io(format!("create {}", parent.display()), e))?;
    }
    let lock_path = path.with_extension("lock");
    let lock_file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| ForgeError::io(format!("open {}", lock_path.display()), e))?;
    lock_file
        .lock()
        .map_err(|e| ForgeError::io("lock declaration view", e))?;
    let result = (|| {
        let declarations = load_declarations(path)?;
        if let Some(existing) = declarations
            .iter()
            .find(|existing| existing.declaration_id == decl.declaration_id)
        {
            if existing.declaration_hash != decl.declaration_hash {
                return Err(ForgeError::Plan {
                    class: "settlement_integrity_error",
                    message: format!(
                        "declaration {} content conflicts with existing view entry",
                        decl.declaration_id
                    ),
                });
            }
            return Ok(());
        }
        append_declaration(path, decl)
    })();
    let unlock = lock_file
        .unlock()
        .map_err(|e| ForgeError::io("unlock declaration view", e));
    match result {
        Ok(()) => unlock,
        Err(error) => Err(error),
    }
}

/// Load all declarations from a JSONL file. Returns empty vec if file does not exist.
pub fn load_declarations(path: &Path) -> Result<Vec<SettlementDeclaration>, ForgeError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file =
        File::open(path).map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut decls = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|e| ForgeError::io("read declarations", e))?;
        if line.trim().is_empty() {
            continue;
        }
        let decl: SettlementDeclaration =
            serde_json::from_str(&line).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        decls.push(decl);
    }
    Ok(decls)
}

/// Trait for settlement authorities (spec §7.2.1).
pub trait SettlementAuthority {
    fn declare(
        &self,
        req: &SettlementDeclarationRequest,
    ) -> Result<SettlementDeclaration, ForgeError>;
    fn adapter_ref(&self) -> &str;
}

/// Local settlement authority — strength is always `local`; cannot satisfy strong-settlement policy.
pub struct LocalSettlementAuthority {
    acting_entity_id: String,
}

impl LocalSettlementAuthority {
    pub fn new(acting_entity_id: &str) -> Self {
        Self {
            acting_entity_id: acting_entity_id.into(),
        }
    }
}

impl SettlementAuthority for LocalSettlementAuthority {
    fn adapter_ref(&self) -> &str {
        "local"
    }

    fn declare(
        &self,
        req: &SettlementDeclarationRequest,
    ) -> Result<SettlementDeclaration, ForgeError> {
        let independent = req.declarer.actor_id != req.acting_entity_id;
        let integrity = check_integrity(req);
        let status = if integrity.is_ok() {
            DeclarationStatus::Accepted
        } else {
            DeclarationStatus::Rejected
        };

        let mut decl = SettlementDeclaration {
            version: RECORD_VERSION.into(),
            declaration_id: random_id("sdec")?,
            settlement_ref: req.settlement_ref.clone(),
            run_id: req.run_id.clone(),
            case_id: req.case_id.clone(),
            plan_item_id: req.plan_item_id.clone(),
            claim_manifest_sha256: req.claim_manifest_sha256.clone(),
            status,
            strength: SettlementStrength::Local,
            qualifies_for_capability: false,
            criteria_ref: req.criteria_ref.clone(),
            criteria_sha256: req.criteria_sha256.clone(),
            criteria_record_hash: req.criteria_record_hash.clone(),
            criteria_declared_at: req.criteria_declared_at.clone(),
            job_contract_ref: req.job_contract_ref.clone(),
            origin_refs: req.origin_refs.clone(),
            verifier_ref: req.verifier_ref.clone(),
            verifier_sha256: req.verifier_sha256.clone(),
            verification_evidence_refs: req.source_evidence_refs.clone(),
            declarer: req.declarer.clone(),
            independence: DeclarationIndependence {
                acting_entity_id: self.acting_entity_id.clone(),
                independent,
                basis: if independent {
                    "declarer_differs_from_acting_entity".into()
                } else {
                    "self_declaration".into()
                },
            },
            reliability: DeclarationReliability {
                feedback_delay_ms: 0,
                attribution_confidence: "1.000000".into(),
                gaming_exposure: "0.000000".into(),
                hidden_debt_blindness: "1.000000".into(),
                weight: "0.000000".into(),
                basis: "local_adapter".into(),
            },
            variation_tags: req.variation_tags.clone(),
            disruption_tags: req.disruption_tags.clone(),
            orchestration_burden: req.orchestration_burden.clone(),
            issued_at: Utc::now().to_rfc3339(),
            source_evidence_refs: req.source_evidence_refs.clone(),
            adapter_attestation_ref: None,
            authored_by: req.authored_by.clone(),
            declaration_hash: String::new(),
        };
        decl.declaration_hash = compute_declaration_hash(&decl)?;
        Ok(decl)
    }
}

/// Response from a SWE_SEED transport, carrying the external authority's reliability assessment.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SweSeedResponse {
    pub attestation_ref: String,
    pub attribution_confidence: String,
    pub gaming_exposure: String,
    pub hidden_debt_blindness: String,
    pub feedback_delay_ms: u64,
}

/// Transport trait for SWE_SEED — real implementation submits to the external service;
/// test doubles return canned responses.
pub trait SweSeedTransport {
    fn submit(&self, req: &SettlementDeclarationRequest) -> Result<SweSeedResponse, ForgeError>;
}

pub struct CommandSweSeedTransport {
    argv: Vec<String>,
    timeout: Duration,
}

impl CommandSweSeedTransport {
    pub fn new(argv: Vec<String>, timeout_secs: u64) -> Result<Self, ForgeError> {
        if argv.first().is_none_or(String::is_empty) || timeout_secs == 0 {
            return Err(ForgeError::Config {
                class: "settlement_authority_config_error",
                path: "<policy>".into(),
                message: "SWE_SEED command and positive timeout are required".into(),
            });
        }
        Ok(Self {
            argv,
            timeout: Duration::from_secs(timeout_secs),
        })
    }
}

impl SweSeedTransport for CommandSweSeedTransport {
    fn submit(&self, req: &SettlementDeclarationRequest) -> Result<SweSeedResponse, ForgeError> {
        use std::os::unix::process::CommandExt;

        let mut command = Command::new(&self.argv[0]);
        command
            .args(&self.argv[1..])
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Own process group so timeout can terminate the full tree.
        command.process_group(0);
        let mut child = command.spawn().map_err(settlement_authority_unavailable)?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| settlement_authority_unavailable("SWE_SEED stdin was not piped"))?;
        let request = serde_json::to_vec(req)?;
        let stdin_done = Arc::new(AtomicBool::new(false));
        let stdin_flag = Arc::clone(&stdin_done);
        let stdin_writer = thread::spawn(move || {
            let result = stdin.write_all(&request).and_then(|_| stdin.flush());
            stdin_flag.store(true, Ordering::SeqCst);
            result
        });
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| settlement_authority_unavailable("SWE_SEED stdout was not piped"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| settlement_authority_unavailable("SWE_SEED stderr was not piped"))?;
        let stdout_reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout
                .take(1_048_577)
                .read_to_end(&mut bytes)
                .map(|_| bytes)
        });
        let stderr_reader = thread::spawn(move || {
            let mut bytes = Vec::new();
            stderr.take(65_537).read_to_end(&mut bytes).map(|_| bytes)
        });
        let started = Instant::now();
        let deadline = started + self.timeout;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(settlement_authority_unavailable)? {
                break status;
            }
            if Instant::now() >= deadline {
                terminate_process_tree(&child)?;
                let _ = child.wait();
                return Err(settlement_authority_unavailable(
                    "SWE_SEED command timed out",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        };
        // Require stdin completion within remaining timeout before accepting response.
        while !stdin_done.load(Ordering::SeqCst) {
            if Instant::now() >= deadline {
                terminate_process_tree(&child)?;
                let _ = child.wait();
                return Err(settlement_authority_unavailable(
                    "SWE_SEED command timed out",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
        stdin_writer
            .join()
            .map_err(|_| settlement_authority_unavailable("SWE_SEED stdin writer failed"))?
            .map_err(settlement_authority_unavailable)?;
        let stdout = join_reader_within(stdout_reader, deadline, "stdout")?;
        let stderr = join_reader_within(stderr_reader, deadline, "stderr")?;
        if !status.success() || stdout.len() > 1_048_576 || stderr.len() > 65_536 {
            return Err(settlement_authority_unavailable(
                "SWE_SEED command failed or exceeded output limits",
            ));
        }
        serde_json::from_slice(&stdout).map_err(|_| {
            settlement_authority_unavailable("SWE_SEED returned an invalid JSON response")
        })
    }
}

fn terminate_process_tree(child: &std::process::Child) -> Result<(), ForgeError> {
    let group = format!("-{}", child.id());
    let group_kill = Command::new("/bin/kill")
        .args(["-KILL", "--", &group])
        .env_clear()
        .output()
        .map_err(settlement_authority_unavailable)?;
    if !group_kill.status.success() {
        let group_still_exists = Command::new("/bin/kill")
            .args(["-0", "--", &group])
            .env_clear()
            .output()
            .map_err(settlement_authority_unavailable)?
            .status
            .success();
        if group_still_exists {
            return Err(settlement_authority_unavailable(
                "SWE_SEED process group survived termination",
            ));
        }
    }
    Ok(())
}

fn join_reader_within(
    handle: thread::JoinHandle<std::io::Result<Vec<u8>>>,
    deadline: Instant,
    label: &str,
) -> Result<Vec<u8>, ForgeError> {
    loop {
        if handle.is_finished() {
            return handle
                .join()
                .map_err(|_| {
                    settlement_authority_unavailable(format!("SWE_SEED {label} reader failed"))
                })?
                .map_err(settlement_authority_unavailable);
        }
        if Instant::now() >= deadline {
            return Err(settlement_authority_unavailable(
                "SWE_SEED command timed out",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn settlement_authority_unavailable(error: impl std::fmt::Display) -> ForgeError {
    ForgeError::Plan {
        class: "settlement_authority_unavailable",
        message: error.to_string(),
    }
}

/// SWE_SEED settlement authority — issues strong declarations backed by external verification.
pub struct SweSeedSettlementAuthority<T: SweSeedTransport> {
    transport: T,
    acting_entity_id: String,
    adapter_ref: String,
}

impl<T: SweSeedTransport> SweSeedSettlementAuthority<T> {
    pub fn new(transport: T, acting_entity_id: &str) -> Self {
        Self {
            transport,
            acting_entity_id: acting_entity_id.into(),
            adapter_ref: "swe_seed".into(),
        }
    }

    pub fn with_adapter_ref(mut self, adapter_ref: &str) -> Self {
        self.adapter_ref = adapter_ref.into();
        self
    }
}

impl<T: SweSeedTransport> SettlementAuthority for SweSeedSettlementAuthority<T> {
    fn adapter_ref(&self) -> &str {
        &self.adapter_ref
    }

    fn declare(
        &self,
        req: &SettlementDeclarationRequest,
    ) -> Result<SettlementDeclaration, ForgeError> {
        let independent = req.declarer.actor_id != req.acting_entity_id;
        let integrity = check_integrity(req);
        let swe_resp = self.transport.submit(req)?;

        let weight = compute_weight(
            &swe_resp.attribution_confidence,
            &swe_resp.gaming_exposure,
            &swe_resp.hidden_debt_blindness,
        );

        let qualifies = integrity.is_ok()
            && req.requested_strength == SettlementStrength::Strong
            && independent;

        let status = if integrity.is_ok() {
            DeclarationStatus::Accepted
        } else {
            DeclarationStatus::Rejected
        };

        let mut decl = SettlementDeclaration {
            version: RECORD_VERSION.into(),
            declaration_id: random_id("sdec")?,
            settlement_ref: req.settlement_ref.clone(),
            run_id: req.run_id.clone(),
            case_id: req.case_id.clone(),
            plan_item_id: req.plan_item_id.clone(),
            claim_manifest_sha256: req.claim_manifest_sha256.clone(),
            status,
            strength: SettlementStrength::Strong,
            qualifies_for_capability: qualifies,
            criteria_ref: req.criteria_ref.clone(),
            criteria_sha256: req.criteria_sha256.clone(),
            criteria_record_hash: req.criteria_record_hash.clone(),
            criteria_declared_at: req.criteria_declared_at.clone(),
            job_contract_ref: req.job_contract_ref.clone(),
            origin_refs: req.origin_refs.clone(),
            verifier_ref: req.verifier_ref.clone(),
            verifier_sha256: req.verifier_sha256.clone(),
            verification_evidence_refs: req.source_evidence_refs.clone(),
            declarer: req.declarer.clone(),
            independence: DeclarationIndependence {
                acting_entity_id: self.acting_entity_id.clone(),
                independent,
                basis: if independent {
                    "swe_seed_independent_verification".into()
                } else {
                    "self_declaration".into()
                },
            },
            reliability: DeclarationReliability {
                feedback_delay_ms: swe_resp.feedback_delay_ms,
                attribution_confidence: swe_resp.attribution_confidence,
                gaming_exposure: swe_resp.gaming_exposure,
                hidden_debt_blindness: swe_resp.hidden_debt_blindness,
                weight,
                basis: "swe_seed_adapter".into(),
            },
            variation_tags: req.variation_tags.clone(),
            disruption_tags: req.disruption_tags.clone(),
            orchestration_burden: req.orchestration_burden.clone(),
            issued_at: Utc::now().to_rfc3339(),
            source_evidence_refs: req.source_evidence_refs.clone(),
            adapter_attestation_ref: Some(swe_resp.attestation_ref),
            authored_by: req.authored_by.clone(),
            declaration_hash: String::new(),
        };
        decl.declaration_hash = compute_declaration_hash(&decl)?;
        Ok(decl)
    }
}

/// Integrity checks shared by all adapters (spec §7.2.1).
fn check_integrity(req: &SettlementDeclarationRequest) -> Result<(), ForgeError> {
    // SoD boundary (spec-adlc-thoth §10.3, T13B): the actor that authored the
    // claim being settled cannot itself be the declarer.
    sea_forge_core::types::validate_claim_authorship_sod(
        req.authored_by.as_deref(),
        &req.declarer.actor_id,
    )?;
    if req.criteria_declared_at > req.execution_started_at {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "criteria_declared_at follows execution_started_at (post-hoc criteria)".into(),
        });
    }
    if req.declarer.actor_id == req.acting_entity_id {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "declarer is the acting entity (self-declaration)".into(),
        });
    }
    if req.criteria_ref.is_empty() {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "missing criteria_ref (legacy_unattributed_criteria)".into(),
        });
    }
    if req.origin_refs.is_empty() {
        return Err(ForgeError::Plan {
            class: "settlement_integrity_error",
            message: "missing origin_refs".into(),
        });
    }
    Ok(())
}

/// Fixed-point weight: attribution_confidence * (1 - gaming_exposure) * (1 - hidden_debt_blindness).
/// Uses i64 millionths internally, clamped to [0, 1_000_000].
fn compute_weight(conf: &str, exposure: &str, blindness: &str) -> String {
    let c = parse_fixed(conf).unwrap_or(0);
    let e = parse_fixed(exposure).unwrap_or(0);
    let b = parse_fixed(blindness).unwrap_or(0);
    let weight = c * (1_000_000 - e) / 1_000_000 * (1_000_000 - b) / 1_000_000;
    format_fixed(weight.clamp(0, 1_000_000))
}

/// Parse a fixed-scale decimal string ("0.800000") into millionths (800000).
fn parse_fixed(s: &str) -> Result<i64, ()> {
    let s = s.trim();
    let (sign, rest) = if let Some(stripped) = s.strip_prefix('-') {
        (-1_i64, stripped)
    } else {
        (1_i64, s)
    };
    let parts: Vec<&str> = rest.split('.').collect();
    let int_part: i64 = parts
        .first()
        .filter(|p| !p.is_empty())
        .and_then(|p| p.parse::<i64>().ok())
        .unwrap_or(0);
    let frac_part = parts.get(1).unwrap_or(&"");
    // SUP-05: char-boundary-safe truncation and checked arithmetic — a
    // pathological magnitude is a typed parse failure, not a panic or wrap.
    let frac_chars: String = frac_part.chars().take(6).collect();
    let frac_padded = format!("{frac_chars:0<6}");
    let frac: i64 = frac_padded.parse().unwrap_or(0);
    let scaled = int_part
        .checked_mul(1_000_000)
        .and_then(|v| v.checked_add(frac))
        .and_then(|v| v.checked_mul(sign))
        .ok_or(())?;
    Ok(scaled)
}

/// Format millionths (800000) as a fixed-scale decimal string ("0.800000").
fn format_fixed(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let abs = v.unsigned_abs();
    format!("{}{}.{:06}", sign, abs / 1_000_000, abs % 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_point_roundtrip() {
        assert_eq!(parse_fixed("0.800000").unwrap(), 800_000);
        assert_eq!(parse_fixed("1.000000").unwrap(), 1_000_000);
        assert_eq!(parse_fixed("0.000000").unwrap(), 0);
        assert_eq!(format_fixed(800_000), "0.800000");
        assert_eq!(format_fixed(1_000_000), "1.000000");
    }

    #[test]
    fn weight_high_confidence_low_exposure() {
        let w = compute_weight("0.950000", "0.050000", "0.100000");
        let v = parse_fixed(&w).unwrap();
        // 0.95 * 0.95 * 0.90 = 0.812250 → 812250
        assert!(v >= 800_000, "expected weight >= 0.8, got {w}");
    }

    #[test]
    fn weight_high_exposure_reduces() {
        let w = compute_weight("0.950000", "0.800000", "0.100000");
        let v = parse_fixed(&w).unwrap();
        // 0.95 * 0.20 * 0.90 = 0.171 → 171000
        assert!(v < 200_000, "expected weight < 0.2, got {w}");
    }

    struct CannedSweSeedTransport;

    impl SweSeedTransport for CannedSweSeedTransport {
        fn submit(
            &self,
            _req: &SettlementDeclarationRequest,
        ) -> Result<SweSeedResponse, ForgeError> {
            Ok(SweSeedResponse {
                attestation_ref: "attestation:test".into(),
                attribution_confidence: "1.000000".into(),
                gaming_exposure: "0.000000".into(),
                hidden_debt_blindness: "0.000000".into(),
                feedback_delay_ms: 0,
            })
        }
    }

    fn swe_seed_declaration_request() -> SettlementDeclarationRequest {
        SettlementDeclarationRequest {
            settlement_ref: "set_x".into(),
            run_id: "run_x".into(),
            case_id: "case_x".into(),
            plan_item_id: "item_x".into(),
            claim_manifest_sha256: "sha256:claim".into(),
            criteria_ref: "crit_x".into(),
            criteria_sha256: "sha256:crit".into(),
            criteria_record_hash: "sha256:crit-record".into(),
            criteria_declared_at: "2026-01-01T00:00:00Z".into(),
            execution_started_at: "2026-01-01T00:00:01Z".into(),
            job_contract_ref: None,
            origin_refs: vec![OriginRef {
                kind: OriginRefKind::Intent,
                reference: "int_x".into(),
                sha256: "sha256:intent".into(),
                role: OriginRole::DesiredResult,
                evidence_refs: vec![],
                domain_model_ref: None,
            }],
            verifier_ref: "swe_seed_harness".into(),
            verifier_sha256: "sha256:verifier".into(),
            acting_entity_id: "operator_local".into(),
            requested_strength: SettlementStrength::Strong,
            declarer: Declarer {
                actor_id: "swe_seed_verifier".into(),
                authority_ref: "swe_seed_test".into(),
                role: "R-AA".into(),
                standing_basis: "test".into(),
            },
            variation_tags: Default::default(),
            disruption_tags: vec![],
            orchestration_burden: None,
            source_evidence_refs: vec!["evi_x".into()],
            authored_by: None,
        }
    }

    /// Two independent `declare()` calls for the exact same underlying claim
    /// (same case/run/claim-manifest/declarer/settlement/plan-item) mint two
    /// different `declaration_id`s — `declare()` itself has no memory of
    /// prior calls. The append-time idempotency key in
    /// `append_declaration_ledgered_once` is keyed on the claim's identity,
    /// not the minted id, so appending the second one must conflict rather
    /// than silently duplicate the same logical declaration in the ledger
    /// (spec-agent-orchestration M16 T18: "matching declarations appear
    /// exactly once").
    #[test]
    fn swe_seed_duplicate_declare_for_same_claim_conflicts_on_append() {
        let dir = tempfile::tempdir().unwrap();
        let authority = SweSeedSettlementAuthority::new(CannedSweSeedTransport, "operator_local")
            .with_adapter_ref("swe_seed_test");
        let request = swe_seed_declaration_request();
        let first = authority.declare(&request).unwrap();
        let second = authority.declare(&request).unwrap();
        assert_ne!(first.declaration_id, second.declaration_id);

        append_declaration_ledgered_once(
            dir.path(),
            &dir.path().join("declarations.jsonl"),
            "test",
            &first,
        )
        .unwrap();
        let conflict = append_declaration_ledgered_once(
            dir.path(),
            &dir.path().join("declarations.jsonl"),
            "test",
            &second,
        );
        assert!(
            conflict.is_err(),
            "a second declaration for the same claim must not append as a distinct record"
        );
    }
}
