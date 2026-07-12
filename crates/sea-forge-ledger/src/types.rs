use crate::signing::{sign_bytes, verify_signature, SigningKey, VerifyingKey};
use sea_forge_core::errors::ForgeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

pub const RECORD_VERSION: &str = "0.2";
pub const CANONICALIZATION: &str = "jcs-nfc-v1";
pub const HASH_ALGORITHM: &str = "sha256-v1";

fn sha256_domain(domain: &str, bytes: &[u8]) -> String {
    let mut payload = Vec::with_capacity(domain.len() + 1 + bytes.len());
    payload.extend_from_slice(domain.as_bytes());
    payload.push(0);
    payload.extend_from_slice(bytes);
    format!("sha256:{:x}", Sha256::digest(&payload))
}

/// Canonicalize a serde value using the jcs-nfc-v1 profile.
/// - Object keys are sorted lexicographically.
/// - Strings are NFC-normalized.
/// - Arrays retain order.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>, ForgeError> {
    fn sorted(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut entries: Vec<_> = map.into_iter().collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let map: serde_json::Map<String, Value> = entries.into_iter().collect();
                Value::Object(map)
            }
            Value::Array(items) => Value::Array(items.into_iter().map(sorted).collect()),
            Value::String(s) => Value::String(
                unicode_normalization::UnicodeNormalization::nfc(s.as_str()).collect(),
            ),
            other => other,
        }
    }
    let sorted = sorted(value.clone());
    serde_json::to_vec(&sorted).map_err(|e| ForgeError::Serialization(e.to_string()))
}

/// Compute a canonical SHA-256 hash for arbitrary serializable data.
pub fn hash_canonical<T: Serialize>(value: &T) -> Result<String, ForgeError> {
    let value =
        serde_json::to_value(value).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let bytes = canonical_json(&value)?;
    Ok(format!("sha256:{:x}", Sha256::digest(&bytes)))
}
pub fn payload_hash(payload: &Value) -> Result<String, ForgeError> {
    let bytes = canonical_json(payload)?;
    Ok(sha256_domain("sea-forge/payload/v1", &bytes))
}

/// Domain-separated entry hash. Computed over the canonical entry excluding
/// the `entry_hash` field itself.
pub fn entry_hash(entry: &LedgerEntry) -> Result<String, ForgeError> {
    let mut value =
        serde_json::to_value(entry).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    if let Value::Object(ref mut map) = value {
        map.remove("entry_hash");
    }
    let bytes = canonical_json(&value)?;
    Ok(sha256_domain("sea-forge/ledger-entry/v1", &bytes))
}

/// Domain-separated MMR root hash.
pub fn stream_root(mmr_state: &MmrState) -> Result<String, ForgeError> {
    let bytes = canonical_json(
        &serde_json::to_value(mmr_state).map_err(|e| ForgeError::Serialization(e.to_string()))?,
    )?;
    Ok(sha256_domain("sea-forge/mmr-root/v1", &bytes))
}

/// Domain-separated global root hash.
pub fn global_root(stream_roots: &[(String, String)]) -> Result<String, ForgeError> {
    let mut sorted = stream_roots.to_vec();
    sorted.sort();
    let bytes = canonical_json(
        &serde_json::to_value(sorted).map_err(|e| ForgeError::Serialization(e.to_string()))?,
    )?;
    Ok(sha256_domain("sea-forge/global-root/v1", &bytes))
}

/// Crockford Base32 alphabet for ULIDs.
const CROCKFORD_ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

struct UlidState {
    last_timestamp_ms: u64,
    last_random: u128,
}

static ULID_STATE: OnceLock<Mutex<UlidState>> = OnceLock::new();

/// Generate a 26-character ULID using the CSPRNG and enforcing monotonicity
/// under clock regression.
pub fn ulid() -> Result<String, ForgeError> {
    use getrandom::fill;
    let timestamp_ms = chrono::Utc::now().timestamp_millis() as u64;

    let state = ULID_STATE.get_or_init(|| {
        Mutex::new(UlidState {
            last_timestamp_ms: 0,
            last_random: 0,
        })
    });
    let mut guard = state
        .lock()
        .map_err(|e| ForgeError::Internal(format!("ULID state poisoned: {e}")))?;

    let (timestamp_ms, random) = if timestamp_ms == guard.last_timestamp_ms {
        guard.last_random += 1;
        (timestamp_ms, guard.last_random)
    } else if timestamp_ms > guard.last_timestamp_ms {
        let mut bytes = [0u8; 10];
        fill(&mut bytes).map_err(|e| ForgeError::Internal(format!("CSPRNG failed: {e}")))?;
        let mut value: u128 = 0;
        for b in bytes {
            value = (value << 8) | u128::from(b);
        }
        guard.last_timestamp_ms = timestamp_ms;
        guard.last_random = value;
        (timestamp_ms, value)
    } else {
        // Clock regressed: keep timestamp at last value and increment random.
        guard.last_random += 1;
        (guard.last_timestamp_ms, guard.last_random)
    };
    drop(guard);

    let mut out = String::with_capacity(26);
    let mut ts_chars = Vec::with_capacity(10);
    let mut t = timestamp_ms;
    for _ in 0..10 {
        ts_chars.push(CROCKFORD_ALPHABET[(t & 0x1f) as usize] as char);
        t >>= 5;
    }
    ts_chars.reverse();
    out.extend(ts_chars);

    // 16 characters for 80-bit random.
    let mut rand_chars = Vec::with_capacity(16);
    let mut r = random;
    for _ in 0..16 {
        rand_chars.push(CROCKFORD_ALPHABET[(r & 0x1f) as usize] as char);
        r >>= 5;
    }
    rand_chars.reverse();
    out.extend(rand_chars);

    Ok(out)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MmrState {
    pub peaks: Vec<String>,
    pub leaf_count: u64,
}

impl MmrState {
    pub fn empty() -> Self {
        Self {
            peaks: Vec::new(),
            leaf_count: 0,
        }
    }

    /// Append a leaf hash and return the updated MMR state.
    pub fn append(&self, leaf_hash: &str) -> Result<Self, ForgeError> {
        let mut peaks = self.peaks.clone();
        let leaf_count = self.leaf_count + 1;
        let mut current = leaf_hash.to_string();
        let mut size = leaf_count;
        while size > 1 && size.is_multiple_of(2) {
            let left = peaks
                .pop()
                .ok_or_else(|| ForgeError::Internal("missing left peak".into()))?;
            let mut hasher = Sha256::new();
            hasher.update(b"sea-forge/mmr-node/v1\0");
            hasher.update(left.as_bytes());
            hasher.update(current.as_bytes());
            current = format!("sha256:{:x}", hasher.finalize());
            size /= 2;
        }
        peaks.push(current);
        Ok(Self { peaks, leaf_count })
    }

    pub fn root(&self) -> Result<String, ForgeError> {
        stream_root(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LedgerEntry {
    pub version: String,
    pub ledger_id: String,
    pub entry_ulid: String,
    pub record_ulid: String,
    pub append_ordinal: u64,
    pub record_kind: String,
    pub subject_refs: Vec<String>,
    pub canonicalization: String,
    pub hash_algorithm: String,
    pub payload_hash: String,
    pub payload: Value,
    pub previous_entry_hash: Option<String>,
    pub entry_hash: String,
    pub mmr_leaf_index: u64,
    pub committed_at: String,
    pub writer_identity_ref: String,
    pub authority_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LedgerCheckpoint {
    pub version: String,
    pub checkpoint_ulid: String,
    pub ledger_id: String,
    pub first_ordinal: u64,
    pub last_ordinal: u64,
    pub entry_count: u64,
    pub mmr_root: String,
    pub previous_checkpoint_hash: Option<String>,
    pub checkpoint_hash: String,
    pub signing_algorithm: String,
    pub signing_key_id: String,
    pub signature: String,
    pub created_at: String,
}

/// Minimal MMR inclusion proof for a single leaf.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MerkleProof {
    pub leaf_index: u64,
    pub proof_hashes: Vec<String>,
}

/// Hash two MMR child hashes under the `sea-forge/mmr-node/v1` domain.
/// The left hash is always the earlier/left child.
fn hash_mmr_node(left: &str, right: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"sea-forge/mmr-node/v1\0");
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

/// Verify a Merkle inclusion proof against a single root hash.
/// The proof's `leaf_index` determines whether the leaf is the left or right
/// child at each level; `proof_hashes` are the sibling hashes along the path.
pub fn verify_proof(entry_hash: &str, root: &str, proof: &MerkleProof) -> bool {
    let mut current = entry_hash.to_string();
    let mut index = proof.leaf_index;
    for sibling in &proof.proof_hashes {
        current = if index.is_multiple_of(2) {
            hash_mmr_node(&current, sibling)
        } else {
            hash_mmr_node(sibling, &current)
        };
        index /= 2;
    }
    current == root
}

/// Return the peak sizes (complete binary tree leaf counts) in the MMR state,
/// ordered from largest to smallest.
fn peak_sizes(leaf_count: u64) -> Vec<u64> {
    if leaf_count == 0 {
        return Vec::new();
    }
    let mut sizes = Vec::new();
    let mut remaining = leaf_count;
    let mut size = 1u64 << (63 - remaining.leading_zeros());
    while remaining > 0 {
        if size <= remaining {
            sizes.push(size);
            remaining -= size;
        }
        size >>= 1;
    }
    sizes
}

/// Build the levels of a perfect binary tree from `leaves`.
/// `levels[0]` is the leaf layer; the last level has one element, the root.
fn build_tree(leaves: &[String]) -> Vec<Vec<String>> {
    let mut levels = vec![leaves.to_vec()];
    while levels.last().unwrap().len() > 1 {
        let level = levels.last().unwrap();
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks(2) {
            next.push(hash_mmr_node(&pair[0], &pair[1]));
        }
        levels.push(next);
    }
    levels
}

/// Canonical bytes of a checkpoint excluding `signature` and `checkpoint_hash`.
fn checkpoint_canonical_bytes(checkpoint: &LedgerCheckpoint) -> Result<Vec<u8>, ForgeError> {
    let mut value =
        serde_json::to_value(checkpoint).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    if let Value::Object(ref mut map) = value {
        map.remove("signature");
        map.remove("checkpoint_hash");
    }
    canonical_json(&value)
}

pub struct LedgerStream {
    ledger_id: String,
    root: PathBuf,
    writer_identity_ref: String,
}

impl LedgerStream {
    pub fn open(
        root: &Path,
        ledger_id: impl Into<String>,
        writer_identity_ref: impl Into<String>,
    ) -> Result<Self, ForgeError> {
        let ledger_id = ledger_id.into();
        let dir = root.join("ledgers").join(&ledger_id);
        fs::create_dir_all(&dir).map_err(|e| ForgeError::io("create ledger directory", e))?;
        fs::create_dir_all(root.join("ledgers").join("quarantine"))
            .map_err(|e| ForgeError::io("create ledger quarantine", e))?;
        Ok(Self {
            ledger_id,
            root: root.to_path_buf(),
            writer_identity_ref: writer_identity_ref.into(),
        })
    }

    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }

    pub fn entries_path(&self) -> PathBuf {
        self.root
            .join("ledgers")
            .join(&self.ledger_id)
            .join("entries.jsonl")
    }

    fn mmr_path(&self) -> PathBuf {
        self.root
            .join("ledgers")
            .join(&self.ledger_id)
            .join("mmr.json")
    }

    pub fn checkpoints_path(&self) -> PathBuf {
        self.root
            .join("ledgers")
            .join(&self.ledger_id)
            .join("checkpoints.jsonl")
    }

    pub fn load_mmr(&self) -> Result<MmrState, ForgeError> {
        let path = self.mmr_path();
        if !path.exists() {
            return Ok(MmrState::empty());
        }
        let bytes = fs::read(&path).map_err(|e| ForgeError::io("read mmr", e))?;
        serde_json::from_slice(&bytes).map_err(|e| ForgeError::Serialization(e.to_string()))
    }

    fn save_mmr(&self, mmr: &MmrState) -> Result<(), ForgeError> {
        let path = self.mmr_path();
        let tmp = path.with_extension("tmp");
        fs::write(
            &tmp,
            serde_json::to_vec(mmr).map_err(|e| ForgeError::Serialization(e.to_string()))?,
        )
        .map_err(|e| ForgeError::io("write mmr", e))?;
        fs::rename(&tmp, &path).map_err(|e| ForgeError::io("rename mmr", e))?;
        Ok(())
    }

    fn read_last_entry(&self) -> Result<Option<LedgerEntry>, ForgeError> {
        let path = self.entries_path();
        if !path.exists() {
            return Ok(None);
        }
        let file = File::open(&path).map_err(|e| ForgeError::io("open entries", e))?;
        let mut last = None;
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| ForgeError::io("read entries", e))?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LedgerEntry = serde_json::from_str(&line)
                .map_err(|e| ForgeError::Serialization(format!("parse entry: {e}")))?;
            last = Some(entry);
        }
        Ok(last)
    }

    /// Read all entries from the stream.
    pub fn read_entries(&self) -> Result<Vec<LedgerEntry>, ForgeError> {
        let path = self.entries_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path).map_err(|e| ForgeError::io("open entries", e))?;
        let mut entries = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| ForgeError::io("read entries", e))?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LedgerEntry = serde_json::from_str(&line)
                .map_err(|e| ForgeError::Serialization(format!("parse entry: {e}")))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Read all checkpoints from the stream.
    fn read_checkpoints(&self) -> Result<Vec<LedgerCheckpoint>, ForgeError> {
        let path = self.checkpoints_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path).map_err(|e| ForgeError::io("open checkpoints", e))?;
        let mut checkpoints = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| ForgeError::io("read checkpoints", e))?;
            if line.trim().is_empty() {
                continue;
            }
            let cp: LedgerCheckpoint = serde_json::from_str(&line)
                .map_err(|e| ForgeError::Serialization(format!("parse checkpoint: {e}")))?;
            checkpoints.push(cp);
        }
        Ok(checkpoints)
    }

    /// Read the last checkpoint from the stream.
    pub fn read_last_checkpoint(&self) -> Result<Option<LedgerCheckpoint>, ForgeError> {
        let checkpoints = self.read_checkpoints()?;
        Ok(checkpoints.into_iter().last())
    }

    /// Append a checkpoint to `checkpoints.jsonl` with durable fsync.
    pub fn write_checkpoint(&self, checkpoint: &LedgerCheckpoint) -> Result<(), ForgeError> {
        let path = self.checkpoints_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| ForgeError::io("open checkpoints", e))?;
        let mut bytes =
            serde_json::to_vec(checkpoint).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        bytes.push(b'\n');
        file.write_all(&bytes)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all())
            .map_err(|e| ForgeError::io("write checkpoint", e))?;
        drop(file);
        if let Some(parent) = path.parent() {
            let _ = fs::File::open(parent).and_then(|f| f.sync_all());
        }
        Ok(())
    }

    /// Create a signed checkpoint covering the current stream state.
    pub fn create_checkpoint(
        &self,
        signing_key: &SigningKey,
        signing_key_id: impl Into<String>,
    ) -> Result<LedgerCheckpoint, ForgeError> {
        let last = self.read_last_entry()?;
        let last_checkpoint = self.read_last_checkpoint()?;
        let mmr = self.load_mmr()?;
        let mmr_root = mmr.root()?;

        let first_ordinal = last_checkpoint
            .as_ref()
            .map(|cp| cp.last_ordinal + 1)
            .unwrap_or(0);
        let last_ordinal = last.as_ref().map(|e| e.append_ordinal).unwrap_or(0);
        let entry_count = if last.is_some() {
            last_ordinal - first_ordinal + 1
        } else {
            0
        };

        let mut checkpoint = LedgerCheckpoint {
            version: "0.2".into(),
            checkpoint_ulid: ulid()?,
            ledger_id: self.ledger_id.clone(),
            first_ordinal,
            last_ordinal,
            entry_count,
            mmr_root,
            previous_checkpoint_hash: last_checkpoint
                .as_ref()
                .map(|cp| cp.checkpoint_hash.clone()),
            checkpoint_hash: String::new(),
            signing_algorithm: "ed25519".into(),
            signing_key_id: signing_key_id.into(),
            signature: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let canonical_bytes = checkpoint_canonical_bytes(&checkpoint)?;
        checkpoint.checkpoint_hash = sha256_domain("sea-forge/checkpoint/v1", &canonical_bytes);
        checkpoint.signature = sign_bytes(signing_key, &canonical_bytes);
        self.write_checkpoint(&checkpoint)?;
        Ok(checkpoint)
    }

    /// Verify all checkpoint signatures and the checkpoint hash chain.
    pub fn verify_checkpoints(&self, verifying_key: &VerifyingKey) -> Result<(), ForgeError> {
        let checkpoints = self.read_checkpoints()?;
        let mut previous_hash: Option<String> = None;
        for checkpoint in &checkpoints {
            let canonical_bytes = checkpoint_canonical_bytes(checkpoint)?;
            if sha256_domain("sea-forge/checkpoint/v1", &canonical_bytes)
                != checkpoint.checkpoint_hash
            {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: checkpoint hash mismatch".into(),
                ));
            }
            verify_signature(verifying_key, &canonical_bytes, &checkpoint.signature)?;
            if checkpoint.previous_checkpoint_hash.as_ref() != previous_hash.as_ref() {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: checkpoint chain break".into(),
                ));
            }
            previous_hash = Some(checkpoint.checkpoint_hash.clone());
        }
        Ok(())
    }

    /// Generate an MMR inclusion proof for the entry with `entry_ulid`.
    pub fn prove_entry(&self, entry_ulid: &str) -> Result<MerkleProof, ForgeError> {
        let entries = self.read_entries()?;
        let mmr = self.load_mmr()?;
        if mmr.leaf_count == 0 {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: cannot prove entry in empty MMR".into(),
            ));
        }
        let leaf_index = entries
            .iter()
            .position(|e| e.entry_ulid == entry_ulid)
            .ok_or_else(|| ForgeError::Internal("entry not found".into()))?
            as u64;
        if leaf_index >= mmr.leaf_count {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: leaf index out of range".into(),
            ));
        }

        let sizes = peak_sizes(mmr.leaf_count);
        let mut start = 0u64;
        let mut containing_peak_size = 0u64;
        for size in &sizes {
            if leaf_index >= start && leaf_index < start + size {
                containing_peak_size = *size;
                break;
            }
            start += size;
        }
        if containing_peak_size == 0 {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: leaf index not in any peak".into(),
            ));
        }

        let start_usize = start as usize;
        let size_usize = containing_peak_size as usize;
        let leaves: Vec<String> = entries[start_usize..start_usize + size_usize]
            .iter()
            .map(|e| e.entry_hash.clone())
            .collect();
        let levels = build_tree(&leaves);
        let local_index = leaf_index - start;
        let mut index = local_index as usize;
        let mut proof_hashes = Vec::with_capacity(levels.len() - 1);
        for entries in levels.iter().take(levels.len() - 1) {
            let sibling = index ^ 1;
            proof_hashes.push(entries[sibling].clone());
            index >>= 1;
        }
        Ok(MerkleProof {
            leaf_index,
            proof_hashes,
        })
    }

    /// Append a payload to the stream and return the committed entry.
    pub fn append(
        &self,
        record_kind: impl Into<String>,
        subject_refs: Vec<String>,
        payload: Value,
        authority_refs: Vec<String>,
    ) -> Result<LedgerEntry, ForgeError> {
        let last = self.read_last_entry()?;
        let append_ordinal = last.as_ref().map(|e| e.append_ordinal + 1).unwrap_or(0);
        let previous_entry_hash = last.as_ref().map(hash_entry);
        let entry_ulid = ulid()?;
        let record_ulid = entry_ulid.clone();
        let mmr_state = self.load_mmr()?;
        let mmr_leaf_index = mmr_state.leaf_count;

        let mut entry = LedgerEntry {
            version: RECORD_VERSION.into(),
            ledger_id: self.ledger_id.clone(),
            entry_ulid,
            record_ulid,
            append_ordinal,
            record_kind: record_kind.into(),
            subject_refs,
            canonicalization: CANONICALIZATION.into(),
            hash_algorithm: HASH_ALGORITHM.into(),
            payload_hash: String::new(),
            payload,
            previous_entry_hash,
            entry_hash: String::new(),
            mmr_leaf_index,
            committed_at: chrono::Utc::now().to_rfc3339(),
            writer_identity_ref: self.writer_identity_ref.clone(),
            authority_refs,
        };
        let payload_hash_value = payload_hash(&entry.payload)?;
        entry.payload_hash = payload_hash_value;
        entry.entry_hash = entry_hash(&entry)?;
        let mmr_state = mmr_state.append(&entry.entry_hash)?;
        self.save_mmr(&mmr_state)?;
        self.write_entry(&entry)?;
        Ok(entry)
    }

    fn write_entry(&self, entry: &LedgerEntry) -> Result<(), ForgeError> {
        let path = self.entries_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| ForgeError::io("open entries", e))?;
        let mut bytes =
            serde_json::to_vec(entry).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        bytes.push(b'\n');
        file.write_all(&bytes)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all())
            .map_err(|e| ForgeError::io("write entry", e))?;
        drop(file);
        if let Some(parent) = path.parent() {
            let _ = fs::File::open(parent).and_then(|f| f.sync_all());
        }
        Ok(())
    }

    /// Verify the stream integrity: payload hashes, predecessor links, ordinals, ULID uniqueness, MMR.
    pub fn verify(&self) -> Result<(), ForgeError> {
        let path = self.entries_path();
        if !path.exists() {
            return Ok(());
        }
        let file = File::open(&path).map_err(|e| ForgeError::io("open entries", e))?;
        let mut seen_ulids = std::collections::HashSet::new();
        let mut expected_ordinal = 0u64;
        let mut previous_hash: Option<String> = None;
        let mut mmr = MmrState::empty();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| ForgeError::io("read entries", e))?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: LedgerEntry = serde_json::from_str(&line)
                .map_err(|e| ForgeError::Serialization(format!("parse entry: {e}")))?;
            if entry.append_ordinal != expected_ordinal {
                return Err(ForgeError::Internal(format!(
                    "ledger_integrity_error: ordinal gap at {expected_ordinal}"
                )));
            }
            if previous_hash.as_ref() != entry.previous_entry_hash.as_ref() {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: predecessor chain break".into(),
                ));
            }
            if !seen_ulids.insert(entry.entry_ulid.clone()) {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: duplicate entry_ulid".into(),
                ));
            }
            let expected_payload_hash = payload_hash(&entry.payload)?;
            if expected_payload_hash != entry.payload_hash {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: payload hash mismatch".into(),
                ));
            }
            let expected_entry_hash = entry_hash(&entry)?;
            if expected_entry_hash != entry.entry_hash {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: entry hash mismatch".into(),
                ));
            }
            mmr = mmr.append(&entry.entry_hash)?;
            if mmr.leaf_count - 1 != entry.mmr_leaf_index {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: mmr leaf index mismatch".into(),
                ));
            }
            previous_hash = Some(entry.entry_hash.clone());
            expected_ordinal += 1;
        }
        let stored_mmr = self.load_mmr()?;
        if stored_mmr != mmr {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: mmr root mismatch".into(),
            ));
        }
        Ok(())
    }
}

fn hash_entry(entry: &LedgerEntry) -> String {
    entry.entry_hash.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ulid_is_26_chars_and_sortable_prefix() {
        let a = ulid().unwrap();
        let b = ulid().unwrap();
        assert_eq!(a.len(), 26);
        assert_eq!(b.len(), 26);
        assert!(a < b, "ULIDs should be monotonic in process");
    }

    #[test]
    fn payload_hash_is_domain_separated() {
        let value = serde_json::json!({"hello": "world"});
        let h = payload_hash(&value).unwrap();
        assert!(h.starts_with("sha256:"));
        assert_eq!(h.len(), 7 + 64);
    }

    #[test]
    fn append_creates_chained_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(tmp.path(), "case_demo", "writer_01").unwrap();
        let entry = stream
            .append(
                "case_event",
                vec!["case_abc".into()],
                serde_json::json!({"state": "active"}),
                vec![],
            )
            .unwrap();
        assert_eq!(entry.append_ordinal, 0);
        assert_eq!(entry.previous_entry_hash, None);
        assert!(entry.entry_hash.starts_with("sha256:"));
        stream.verify().unwrap();
    }

    #[test]
    fn detect_one_byte_alteration() {
        let tmp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(tmp.path(), "case_demo", "writer_01").unwrap();
        stream
            .append(
                "case_event",
                vec!["case_abc".into()],
                serde_json::json!({"state": "active"}),
                vec![],
            )
            .unwrap();
        let path = stream.entries_path();
        let mut content = fs::read_to_string(&path).unwrap();
        content = content.replace("active", "ACTIVE");
        fs::write(&path, content).unwrap();
        let result = stream.verify();
        assert!(result.is_err());
    }

    #[test]
    fn detect_duplicate_ordinal() {
        let tmp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(tmp.path(), "case_demo", "writer_01").unwrap();
        let e1 = stream
            .append(
                "case_event",
                vec!["case_abc".into()],
                serde_json::json!({"state": "active"}),
                vec![],
            )
            .unwrap();
        let mut e2 = e1.clone();
        e2.append_ordinal = 0; // duplicate
        e2.entry_hash = entry_hash(&e2).unwrap();
        // append by writing directly
        let mut file = OpenOptions::new()
            .append(true)
            .open(stream.entries_path())
            .unwrap();
        let mut bytes = serde_json::to_vec(&e2).unwrap();
        bytes.push(b'\n');
        file.write_all(&bytes).unwrap();
        drop(file);
        let result = stream.verify();
        assert!(result.is_err());
    }
}
