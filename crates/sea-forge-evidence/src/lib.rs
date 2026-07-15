use sea_forge_core::{
    errors::ForgeError,
    ids::{random_id, seq_id},
    types::*,
    RECORD_VERSION,
};
use sea_forge_sandbox::safe_join;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read, Write},
    path::Path,
};
use unicode_normalization::UnicodeNormalization;

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn sha256_file(path: &Path) -> Result<String, ForgeError> {
    let mut file =
        File::open(path).map_err(|e| ForgeError::io(format!("open {}", path.display()), e))?;
    let mut hash = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| ForgeError::io("hash artifact", e))?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ForgeError> {
    fn sorted(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                Value::Object(map.into_iter().map(|(k, v)| (k, sorted(v))).collect())
            }
            Value::Array(items) => Value::Array(items.into_iter().map(sorted).collect()),
            Value::String(s) => Value::String(s.nfc().collect()),
            other => other,
        }
    }
    Ok(serde_json::to_vec(&sorted(serde_json::to_value(value)?))?)
}
pub fn hash_canonical<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    Ok(format!("sha256:{}", sha256_bytes(&canonical_json(value)?)))
}

pub struct JsonlEvidenceWriter {
    writer: BufWriter<File>,
    run_id: String,
    sequence: usize,
    refs: Vec<String>,
}
impl JsonlEvidenceWriter {
    pub fn create(path: &Path, run_id: &str) -> Result<Self, ForgeError> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .map_err(|e| ForgeError::io(format!("create {}", path.display()), e))?;
        Ok(Self {
            writer: BufWriter::new(file),
            run_id: run_id.into(),
            sequence: 0,
            refs: Vec::new(),
        })
    }
    pub fn append(
        &mut self,
        kind: EvidenceKind,
        uri: String,
        sha256: Option<String>,
        source_event_id: String,
        metadata: BTreeMap<String, Value>,
    ) -> Result<EvidenceRecord, ForgeError> {
        let record = self.prepare(kind, uri, sha256, source_event_id, metadata);
        self.append_prepared(record)
    }
    pub fn prepare(
        &self,
        kind: EvidenceKind,
        uri: String,
        sha256: Option<String>,
        source_event_id: String,
        metadata: BTreeMap<String, Value>,
    ) -> EvidenceRecord {
        EvidenceRecord {
            version: RECORD_VERSION.into(),
            evidence_id: seq_id("evi", 4, self.sequence + 1),
            run_id: self.run_id.clone(),
            kind,
            uri,
            sha256,
            source_event_id,
            metadata,
            cell_id: None,
        }
    }
    pub fn append_prepared(
        &mut self,
        record: EvidenceRecord,
    ) -> Result<EvidenceRecord, ForgeError> {
        let expected = seq_id("evi", 4, self.sequence + 1);
        if record.run_id != self.run_id || record.evidence_id != expected {
            return Err(ForgeError::Input(
                "prepared evidence does not match writer sequence".into(),
            ));
        }
        self.sequence += 1;
        serde_json::to_writer(&mut self.writer, &record)?;
        self.writer
            .write_all(b"\n")
            .and_then(|_| self.writer.flush())
            .map_err(|e| ForgeError::io("flush evidence", e))?;
        self.refs.push(record.evidence_id.clone());
        Ok(record)
    }
    pub fn refs(&self) -> Vec<String> {
        self.refs.clone()
    }
}

pub fn capture_file(
    source: &Path,
    artifacts: &Path,
    name: &str,
    writer: &mut JsonlEvidenceWriter,
    source_event_id: String,
    descriptor: Option<(&Intent, &str)>,
) -> Result<(EvidenceRecord, Option<ArtifactDescriptor>), ForgeError> {
    fs::create_dir_all(artifacts).map_err(|e| ForgeError::io("create artifacts directory", e))?;
    let destination = safe_join(artifacts, name)?;
    if source != destination {
        fs::copy(source, &destination)
            .map_err(|e| ForgeError::io(format!("copy artifact {name}"), e))?;
    }
    let digest = sha256_file(&destination)?;
    let mut metadata = BTreeMap::new();
    let mut output_descriptor = None;
    if let Some((intent, plan_item_id)) = descriptor {
        let identity_input = json!({"artifact_type":"sea_model","stage":"intellectual","owner":intent.actor_id,"license":"internal-unlicensed","review_status":"draft","content_sha256":digest,"source_refs":[]});
        let artifact = ArtifactDescriptor {
            artifact_id: random_id("art")?,
            artifact_type: ArtifactType::SeaModel,
            stage: Some(ArtifactStage::Intellectual),
            producer: ArtifactProducer {
                entity_id: intent.actor_id.clone(),
                process_id: intent.process_id.clone(),
                run_id: writer.run_id.clone(),
                plan_item_id: plan_item_id.into(),
            },
            owner: intent.actor_id.clone(),
            license: "internal-unlicensed".into(),
            review_status: ReviewStatus::Draft,
            source_refs: vec![],
            content_sha256: digest.clone(),
            pre_mint_identity: format!(
                "ifl:hash:{}",
                sha256_bytes(&canonical_json(&identity_input)?)
            ),
        };
        metadata.insert("artifact".into(), serde_json::to_value(&artifact)?);
        output_descriptor = Some(artifact);
    } else {
        metadata.insert("artifact_role".into(), json!("execution_evidence"));
    }
    let record = writer.append(
        EvidenceKind::Artifact,
        format!("artifacts/{name}"),
        Some(digest),
        source_event_id,
        metadata,
    )?;
    Ok((record, output_descriptor))
}
