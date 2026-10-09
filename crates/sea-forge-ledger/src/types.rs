use crate::signing::{
    load_or_create_signing_key, sign_bytes, verify_signature, SigningKey, VerifyingKey,
};
use sea_forge_core::errors::ForgeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

pub const RECORD_VERSION: &str = "0.2";
pub const CANONICALIZATION: &str = "jcs-nfc-v1";
pub const HASH_ALGORITHM: &str = "sha256-v1";

const BOUNDED_READER_PAGE_BYTES: u64 = 4 * 1024 * 1024;
const BOUNDED_READER_ROW_BYTES: usize = 2 * 1024 * 1024;
const BOUNDED_READER_FORWARD_ROWS: u64 = 500;
const BOUNDED_READER_BUFFER_BYTES: usize = 8 * 1024;
const BOUNDED_READER_LOCK_WAIT: Duration = Duration::from_millis(50);

fn sha256_domain(domain: &str, bytes: &[u8]) -> String {
    let mut payload = Vec::with_capacity(domain.len() + 1 + bytes.len());
    payload.extend_from_slice(domain.as_bytes());
    payload.push(0);
    payload.extend_from_slice(bytes);
    format!("sha256:{:x}", Sha256::digest(&payload))
}

// Used by the bounded reader for every native byte-position advance.
fn checked_ledger_offset_add(start: u64, delta: u64) -> Result<u64, ForgeError> {
    start
        .checked_add(delta)
        .ok_or_else(|| reader_integrity_error("byte offset overflow"))
}

// Used by bounded-row linkage validation before accepting the next ordinal.
fn checked_ledger_ordinal_add(previous: u64) -> Result<u64, ForgeError> {
    previous
        .checked_add(1)
        .ok_or_else(|| reader_integrity_error("append ordinal overflow"))
}

fn reader_integrity_error(message: &str) -> ForgeError {
    ForgeError::Internal(format!("ledger_integrity_error: {message}"))
}

/// Canonicalize a serde value using the jcs-nfc-v1 profile.
/// - Object keys are sorted lexicographically.
/// - String values are NFC-normalized; keys are not (documented asymmetry).
/// - Arrays retain order.
///
/// SUP-09d: this is the shared `sea_forge_core::canonical` primitive — the
/// single implementation behind every hash producer in the workspace. The
/// `jcs-nfc-v1` label names *this* profile, which is not RFC 8785 JCS.
pub fn canonical_json(value: &Value) -> Result<Vec<u8>, ForgeError> {
    sea_forge_core::canonical::canonical_json(value)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
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

impl LedgerEntry {
    pub fn committed_ref(&self) -> CommittedRecordRef {
        committed_record_ref(self.clone())
    }
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

/// An independently signed receipt over a global checkpoint hash.
/// Witnesses MUST be independent from the acting entity and the local signer.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WitnessReceipt {
    pub version: String,
    pub receipt_ulid: String,
    pub global_checkpoint_hash: String,
    pub witness_signer_id: String,
    pub witness_signing_algorithm: String,
    pub witness_signature: String,
    pub receipt_hash: String,
    pub signed_at: String,
}

/// A global checkpoint committing all active stream roots.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GlobalCheckpoint {
    pub version: String,
    pub checkpoint_ulid: String,
    pub stream_roots: Vec<(String, String)>,
    pub global_root: String,
    pub previous_global_checkpoint_hash: Option<String>,
    pub global_checkpoint_hash: String,
    pub signing_algorithm: String,
    pub signing_key_id: String,
    pub signature: String,
    pub created_at: String,
}

/// Assurance level reported by verification.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AssuranceLevel {
    LocalTamperEvident,
    ExternallyVerified,
    LegacyDigestOnly,
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

/// Canonical bytes of a global checkpoint excluding `signature` and `global_checkpoint_hash`.
fn global_checkpoint_canonical_bytes(cp: &GlobalCheckpoint) -> Result<Vec<u8>, ForgeError> {
    let mut value =
        serde_json::to_value(cp).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    if let Value::Object(ref mut map) = value {
        map.remove("signature");
        map.remove("global_checkpoint_hash");
    }
    canonical_json(&value)
}

/// Canonical bytes of a witness receipt excluding `receipt_hash` and `witness_signature`.
fn witness_receipt_canonical_bytes(receipt: &WitnessReceipt) -> Result<Vec<u8>, ForgeError> {
    let mut value =
        serde_json::to_value(receipt).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    if let Value::Object(ref mut map) = value {
        map.remove("receipt_hash");
        map.remove("witness_signature");
    }
    canonical_json(&value)
}

/// Plaintext secret sentinels rejected by the ledger redaction policy
/// (spec-full §7.0c). Exposed so the transcript redactor in `sea-forge-agent`
/// redacts against the exact same corpus rather than a divergent copy.
pub const SECRET_SENTINELS: &[&str] = &[
    "-----begin private key-----", // gitleaks:allow
    "-----begin rsa private key-----",
    "-----begin ec private key-----",
    "-----begin openssh private key-----",
    "aws_secret_access_key",
    "api_key",
    "password",
    "secret_key",
    "client_secret",
];

/// Redaction policy: reject payloads containing plaintext secrets.
/// Approved ciphertext commitments (payload has `redacted_commitment` field) pass.
pub fn check_redaction(payload: &Value) -> Result<(), ForgeError> {
    // Approved ciphertext commitment: payload explicitly declares it is redacted.
    if let Some(obj) = payload.as_object() {
        if obj.contains_key("redacted_commitment") {
            return Ok(());
        }
    }
    let text = payload.to_string().to_lowercase();
    for sentinel in SECRET_SENTINELS {
        if text.contains(sentinel) {
            return Err(ForgeError::Internal(format!(
                "ledger_integrity_error: secret sentinel in payload ({sentinel})"
            )));
        }
    }
    Ok(())
}

pub struct LedgerStream {
    ledger_id: String,
    root: PathBuf,
    writer_identity_ref: String,
}

/// A candidate row position returned after ledger-integrity validation.
/// It is not, by itself, an acknowledgement or proof of event validity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerReadPosition {
    pub start_offset: u64,
    pub end_offset: u64,
    pub append_ordinal: u64,
    pub entry_ulid: String,
    pub entry_hash: String,
    pub raw_row_checksum: [u8; 32],
}

/// Resume evidence carries the original pinned head and the last acknowledged
/// candidate position together. The caller must authenticate external tokens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerReadResume {
    pub pinned_head: LedgerReadPosition,
    pub acknowledged: Option<LedgerReadPosition>,
}

fn append_reverse_segment(reversed: &mut Vec<u8>, segment: &[u8]) -> Result<(), ForgeError> {
    let new_len = reversed
        .len()
        .checked_add(segment.len())
        .ok_or_else(|| reader_integrity_error("tail row length overflow"))?;
    if new_len > BOUNDED_READER_ROW_BYTES {
        return Err(reader_integrity_error("tail row exceeds byte limit"));
    }
    reversed
        .try_reserve_exact(segment.len())
        .map_err(|error| ForgeError::Internal(format!("bounded row allocation failed: {error}")))?;
    reversed.extend(segment.iter().rev());
    Ok(())
}

fn read_exact_counted(
    file: &mut File,
    mut output: &mut [u8],
    remaining_raw_bytes: &mut u64,
) -> Result<(), ForgeError> {
    while !output.is_empty() {
        if *remaining_raw_bytes == 0 {
            return Err(reader_integrity_error("page raw-input budget exhausted"));
        }
        let request = output
            .len()
            .min(BOUNDED_READER_BUFFER_BYTES)
            .min(usize::try_from(*remaining_raw_bytes).unwrap_or(usize::MAX));
        let read = file
            .read(&mut output[..request])
            .map_err(|error| ForgeError::io("read bounded ledger row", error))?;
        if read == 0 {
            return Err(reader_integrity_error(
                "unexpected end of bounded ledger row",
            ));
        }
        *remaining_raw_bytes -= read as u64;
        output = &mut output[read..];
    }
    Ok(())
}

fn validate_bounded_entry(raw_row: &[u8], ledger_id: &str) -> Result<LedgerEntry, ForgeError> {
    if raw_row.len() > BOUNDED_READER_ROW_BYTES || raw_row.last() != Some(&b'\n') {
        return Err(reader_integrity_error(
            "invalid bounded ledger row boundary",
        ));
    }
    if raw_row[..raw_row.len() - 1].contains(&b'\n') {
        return Err(reader_integrity_error(
            "bounded ledger row contains an interior LF",
        ));
    }
    let entry: LedgerEntry = serde_json::from_slice(&raw_row[..raw_row.len() - 1])
        .map_err(|error| ForgeError::Serialization(format!("parse bounded ledger row: {error}")))?;
    if entry.ledger_id != ledger_id
        || entry.version != RECORD_VERSION
        || entry.canonicalization != CANONICALIZATION
        || entry.hash_algorithm != HASH_ALGORITHM
    {
        return Err(reader_integrity_error(
            "ledger row protocol identity mismatch",
        ));
    }
    if entry.payload_hash != payload_hash(&entry.payload)?
        || entry.entry_hash != entry_hash(&entry)?
    {
        return Err(reader_integrity_error("ledger row hash mismatch"));
    }
    if (entry.append_ordinal == 0 && entry.previous_entry_hash.is_some())
        || (entry.append_ordinal != 0 && entry.previous_entry_hash.is_none())
    {
        return Err(reader_integrity_error(
            "ledger row predecessor presence mismatch",
        ));
    }
    Ok(entry)
}

fn position_for_row(
    entry: &LedgerEntry,
    raw_row: &[u8],
    start_offset: u64,
) -> Result<LedgerReadPosition, ForgeError> {
    let row_len = u64::try_from(raw_row.len())
        .map_err(|_| reader_integrity_error("ledger row length does not fit u64"))?;
    Ok(LedgerReadPosition {
        start_offset,
        end_offset: checked_ledger_offset_add(start_offset, row_len)?,
        append_ordinal: entry.append_ordinal,
        entry_ulid: entry.entry_ulid.clone(),
        entry_hash: entry.entry_hash.clone(),
        raw_row_checksum: Sha256::digest(raw_row).into(),
    })
}

fn read_position_row(
    file: &mut File,
    ledger_id: &str,
    position: &LedgerReadPosition,
    file_len: u64,
    remaining_raw_bytes: &mut u64,
) -> Result<LedgerEntry, ForgeError> {
    let row_len = position
        .end_offset
        .checked_sub(position.start_offset)
        .ok_or_else(|| reader_integrity_error("reversed ledger row offsets"))?;
    if row_len == 0 || row_len > BOUNDED_READER_ROW_BYTES as u64 || position.end_offset > file_len {
        return Err(reader_integrity_error(
            "resume position is outside ledger bounds",
        ));
    }
    let required_raw_bytes = row_len
        .checked_add(if position.start_offset > 0 { 1 } else { 0 })
        .ok_or_else(|| reader_integrity_error("resume input byte count overflow"))?;
    if required_raw_bytes > *remaining_raw_bytes {
        return Err(reader_integrity_error(
            "resume boundary and row exceed remaining page raw-input budget",
        ));
    }
    let row_len = usize::try_from(row_len)
        .map_err(|_| reader_integrity_error("ledger row length does not fit usize"))?;

    if position.start_offset > 0 {
        file.seek(SeekFrom::Start(position.start_offset - 1))
            .map_err(|error| ForgeError::io("seek bounded ledger boundary", error))?;
        let mut boundary = [0u8; 1];
        read_exact_counted(file, &mut boundary, remaining_raw_bytes)?;
        if boundary[0] != b'\n' {
            return Err(reader_integrity_error("resume start is not a row boundary"));
        }
    }

    file.seek(SeekFrom::Start(position.start_offset))
        .map_err(|error| ForgeError::io("seek bounded ledger row", error))?;
    let mut raw_row = Vec::new();
    raw_row
        .try_reserve_exact(row_len)
        .map_err(|error| ForgeError::Internal(format!("bounded row allocation failed: {error}")))?;
    raw_row.resize(row_len, 0);
    read_exact_counted(file, &mut raw_row, remaining_raw_bytes)?;
    let entry = validate_bounded_entry(&raw_row, ledger_id)?;
    let observed = position_for_row(&entry, &raw_row, position.start_offset)?;
    if &observed != position {
        return Err(reader_integrity_error(
            "resume row does not match supplied position",
        ));
    }
    if (position.start_offset == 0) != (entry.append_ordinal == 0) {
        return Err(reader_integrity_error(
            "origin position and ordinal disagree",
        ));
    }
    Ok(entry)
}

fn read_tail_position(
    file: &mut File,
    ledger_id: &str,
    file_len: u64,
    remaining_raw_bytes: &mut u64,
) -> Result<(LedgerReadPosition, LedgerEntry), ForgeError> {
    let mut cursor = file_len;
    let mut reversed_row = Vec::new();
    let mut saw_terminal_lf = false;
    let mut block = [0u8; BOUNDED_READER_BUFFER_BYTES];

    while cursor > 0 {
        if *remaining_raw_bytes == 0 {
            return Err(reader_integrity_error(
                "page raw-input budget exhausted pinning tail",
            ));
        }
        let request_u64 = cursor
            .min(BOUNDED_READER_BUFFER_BYTES as u64)
            .min(*remaining_raw_bytes)
            .min(
                u64::try_from(BOUNDED_READER_ROW_BYTES - reversed_row.len() + 1)
                    .unwrap_or(u64::MAX),
            );
        let request = usize::try_from(request_u64)
            .map_err(|_| reader_integrity_error("tail read size does not fit usize"))?;
        let start = cursor - request_u64;
        file.seek(SeekFrom::Start(start))
            .map_err(|error| ForgeError::io("seek bounded ledger tail", error))?;
        let mut filled = 0usize;
        while filled < request {
            if *remaining_raw_bytes == 0 {
                return Err(reader_integrity_error(
                    "page raw-input budget exhausted pinning tail",
                ));
            }
            let read_request =
                (request - filled).min(usize::try_from(*remaining_raw_bytes).unwrap_or(usize::MAX));
            let read = file
                .read(&mut block[filled..filled + read_request])
                .map_err(|error| ForgeError::io("read bounded ledger tail", error))?;
            if read == 0 {
                return Err(reader_integrity_error(
                    "unexpected end while pinning ledger tail",
                ));
            }
            *remaining_raw_bytes -= read as u64;
            filled += read;
        }

        if !saw_terminal_lf {
            if block[filled - 1] != b'\n' {
                return Err(reader_integrity_error("ledger tail is not LF terminated"));
            }
            append_reverse_segment(&mut reversed_row, b"\n")?;
            saw_terminal_lf = true;
        }
        let scan_end = if file_len - start == filled as u64 {
            filled - 1
        } else {
            filled
        };
        if let Some(boundary) = block[..scan_end].iter().rposition(|byte| *byte == b'\n') {
            append_reverse_segment(&mut reversed_row, &block[boundary + 1..scan_end])?;
            break;
        }
        append_reverse_segment(&mut reversed_row, &block[..scan_end])?;
        cursor = start;
        if cursor == 0 {
            break;
        }
    }

    if !saw_terminal_lf {
        return Err(reader_integrity_error("ledger tail is empty or torn"));
    }
    reversed_row.reverse();
    let row_len = u64::try_from(reversed_row.len())
        .map_err(|_| reader_integrity_error("tail row length does not fit u64"))?;
    let start_offset = file_len
        .checked_sub(row_len)
        .ok_or_else(|| reader_integrity_error("tail row starts before ledger origin"))?;
    let entry = validate_bounded_entry(&reversed_row, ledger_id)?;
    let position = position_for_row(&entry, &reversed_row, start_offset)?;
    if position.end_offset != file_len {
        return Err(reader_integrity_error(
            "tail row does not end at sampled file end",
        ));
    }
    if (position.start_offset == 0) != (entry.append_ordinal == 0) {
        return Err(reader_integrity_error("tail origin and ordinal disagree"));
    }
    Ok((position, entry))
}

#[derive(Clone, Debug, PartialEq)]
pub struct LedgerReadRow {
    pub entry: LedgerEntry,
    pub position: LedgerReadPosition,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LedgerReadStep {
    Row(Box<LedgerReadRow>),
    Complete,
    LimitReached,
}

/// A bounded scan over one immutable pinned prefix of a registered stream.
pub struct LedgerReadSession {
    _file: Option<File>,
    ledger_id: String,
    pinned_head: Option<LedgerReadPosition>,
    remaining_raw_bytes: u64,
    forward_rows: u64,
    validated_position: Option<LedgerReadPosition>,
    next_offset: u64,
    read_offset: u64,
    read_buffer: [u8; BOUNDED_READER_BUFFER_BYTES],
    read_buffer_pos: usize,
    read_buffer_len: usize,
    previous_entry_hash: Option<String>,
    complete: bool,
    failed: bool,
}

impl LedgerReadSession {
    /// The immutable pinned-head candidate, or `None` for proven empty input.
    pub fn pinned_head(&self) -> Option<&LedgerReadPosition> {
        self.pinned_head.as_ref()
    }

    /// Return one ledger-integrity candidate, completion, or an explicit
    /// bounded-limit outcome. Candidate positions do not advance ACK state.
    pub fn next_step(&mut self) -> Result<LedgerReadStep, ForgeError> {
        if self.failed {
            return Err(reader_integrity_error(
                "bounded ledger session already failed",
            ));
        }
        let result = self.next_step_inner();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn next_step_inner(&mut self) -> Result<LedgerReadStep, ForgeError> {
        if self.complete {
            return Ok(LedgerReadStep::Complete);
        }
        let Some(pinned_head_end) = self
            .pinned_head
            .as_ref()
            .map(|position| position.end_offset)
        else {
            self.complete = true;
            return Ok(LedgerReadStep::Complete);
        };
        if self.next_offset == pinned_head_end {
            if self.validated_position.as_ref() != self.pinned_head.as_ref() {
                return Err(reader_integrity_error(
                    "scan ended without validating pinned head",
                ));
            }
            self.complete = true;
            return Ok(LedgerReadStep::Complete);
        }
        if self.next_offset > pinned_head_end {
            return Err(reader_integrity_error("scan advanced beyond pinned head"));
        }
        if self.forward_rows >= BOUNDED_READER_FORWARD_ROWS {
            return Ok(LedgerReadStep::LimitReached);
        }

        let Some(raw_row) = self.read_forward_row(pinned_head_end)? else {
            return Ok(LedgerReadStep::LimitReached);
        };
        let start_offset = self.next_offset;
        let entry = validate_bounded_entry(&raw_row, &self.ledger_id)?;
        let expected_ordinal = match self.validated_position.as_ref() {
            Some(previous) => checked_ledger_ordinal_add(previous.append_ordinal)?,
            None => 0,
        };
        if entry.append_ordinal != expected_ordinal
            || entry.previous_entry_hash != self.previous_entry_hash
        {
            return Err(reader_integrity_error(
                "ledger ordinal or predecessor linkage mismatch",
            ));
        }
        let position = position_for_row(&entry, &raw_row, start_offset)?;
        if position.end_offset > pinned_head_end {
            return Err(reader_integrity_error("row extends beyond pinned head"));
        }
        if position.end_offset == pinned_head_end && self.pinned_head.as_ref() != Some(&position) {
            return Err(reader_integrity_error(
                "scanned tail does not match pinned head",
            ));
        }

        self.next_offset = position.end_offset;
        self.previous_entry_hash = Some(entry.entry_hash.clone());
        self.forward_rows += 1;
        self.validated_position = Some(position.clone());
        Ok(LedgerReadStep::Row(Box::new(LedgerReadRow {
            entry,
            position,
        })))
    }

    fn read_forward_row(&mut self, pinned_end: u64) -> Result<Option<Vec<u8>>, ForgeError> {
        let mut row = Vec::new();
        loop {
            if self.read_buffer_pos < self.read_buffer_len {
                let remaining = &self.read_buffer[self.read_buffer_pos..self.read_buffer_len];
                if let Some(line_feed) = remaining.iter().position(|byte| *byte == b'\n') {
                    let count = line_feed + 1;
                    let next_len = row
                        .len()
                        .checked_add(count)
                        .ok_or_else(|| reader_integrity_error("row length overflow"))?;
                    if next_len > BOUNDED_READER_ROW_BYTES {
                        return Err(reader_integrity_error("ledger row exceeds byte limit"));
                    }
                    row.try_reserve_exact(count).map_err(|error| {
                        ForgeError::Internal(format!("bounded row allocation failed: {error}"))
                    })?;
                    row.extend_from_slice(&remaining[..count]);
                    self.read_buffer_pos += count;
                    return Ok(Some(row));
                }

                let count = remaining.len();
                let next_len = row
                    .len()
                    .checked_add(count)
                    .ok_or_else(|| reader_integrity_error("row length overflow"))?;
                if next_len > BOUNDED_READER_ROW_BYTES {
                    return Err(reader_integrity_error("ledger row exceeds byte limit"));
                }
                row.try_reserve_exact(count).map_err(|error| {
                    ForgeError::Internal(format!("bounded row allocation failed: {error}"))
                })?;
                row.extend_from_slice(remaining);
                self.read_buffer_pos = self.read_buffer_len;
                continue;
            }

            self.read_buffer_pos = 0;
            self.read_buffer_len = 0;
            if self.remaining_raw_bytes == 0 {
                return Ok(None);
            }
            if self.read_offset >= pinned_end {
                return Err(reader_integrity_error(
                    "pinned head ends inside a ledger row",
                ));
            }
            let row_remaining = BOUNDED_READER_ROW_BYTES - row.len();
            if row_remaining == 0 {
                return Err(reader_integrity_error("ledger row exceeds byte limit"));
            }
            let pin_remaining = pinned_end - self.read_offset;
            let request = BOUNDED_READER_BUFFER_BYTES
                .min(row_remaining)
                .min(usize::try_from(self.remaining_raw_bytes).unwrap_or(usize::MAX))
                .min(usize::try_from(pin_remaining).unwrap_or(usize::MAX));
            if request == 0 {
                return Ok(None);
            }
            let file = self
                ._file
                .as_mut()
                .ok_or_else(|| reader_integrity_error("pinned ledger file is unavailable"))?;
            let read = file
                .read(&mut self.read_buffer[..request])
                .map_err(|error| ForgeError::io("read bounded ledger row", error))?;
            if read == 0 {
                return Err(reader_integrity_error("ledger ended before pinned head"));
            }
            self.remaining_raw_bytes -= read as u64;
            self.read_offset = checked_ledger_offset_add(self.read_offset, read as u64)?;
            self.read_buffer_len = read;
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CommittedRecordRef {
    ledger_id: String,
    record_kind: String,
    entry_ulid: String,
    record_ulid: String,
    append_ordinal: u64,
    entry_hash: String,
    payload_hash: String,
}

pub struct PreActionAssurance {
    global_checkpoint_hash: String,
    covered_entries: std::collections::BTreeSet<String>,
}

impl PreActionAssurance {
    pub fn global_checkpoint_hash(&self) -> &str {
        &self.global_checkpoint_hash
    }

    pub fn covers(&self, committed: &CommittedRecordRef) -> bool {
        self.covered_entries.contains(committed.entry_ulid())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewStatus {
    Current,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityViewState {
    pub source_entry_ulid: String,
    #[serde(default)]
    pub source_entry_ulids: Vec<String>,
    pub source_payload_hash: String,
    pub view_path: String,
    pub view_sha256: String,
    pub status: ViewStatus,
    pub failure_class: Option<String>,
}

impl CommittedRecordRef {
    pub fn ledger_id(&self) -> &str {
        &self.ledger_id
    }

    pub fn record_kind(&self) -> &str {
        &self.record_kind
    }

    pub fn entry_ulid(&self) -> &str {
        &self.entry_ulid
    }

    pub fn payload_hash(&self) -> &str {
        &self.payload_hash
    }
}

impl LedgerStream {
    pub fn open(
        root: &Path,
        ledger_id: impl Into<String>,
        writer_identity_ref: impl Into<String>,
    ) -> Result<Self, ForgeError> {
        let ledger_id = ledger_id.into();
        // The ledger id is joined into the filesystem path; validate it as a
        // single safe segment before any mutation (F-14). Server-minted ids
        // (`case-<id>`, `run-<id>`) and internal names (`self-model`) satisfy
        // this grammar; traversal-shaped ids are rejected with a typed error.
        if !sea_forge_core::path::valid_id_segment(&ledger_id, 128) {
            return Err(ForgeError::Input(format!("unsafe ledger id: {ledger_id}")));
        }
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

    fn acquire_bounded_reader_lock(&self) -> Result<File, ForgeError> {
        let lock_file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())
            .map_err(|error| ForgeError::io("open bounded ledger lock", error))?;
        let deadline = Instant::now() + BOUNDED_READER_LOCK_WAIT;
        loop {
            if Instant::now() >= deadline {
                return Err(ForgeError::io(
                    "bounded ledger lock wait expired",
                    std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "ledger stream lock remained held",
                    ),
                ));
            }
            match lock_file.try_lock() {
                Ok(()) if Instant::now() < deadline => return Ok(lock_file),
                Ok(()) => {
                    return Err(ForgeError::io(
                        "bounded ledger lock wait expired",
                        std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "ledger stream lock remained held",
                        ),
                    ));
                }
                Err(std::fs::TryLockError::WouldBlock) => thread::yield_now(),
                Err(std::fs::TryLockError::Error(error)) => {
                    return Err(ForgeError::io("acquire bounded ledger lock", error));
                }
            }
        }
    }

    fn open_bounded_entries(&self) -> Result<Option<File>, ForgeError> {
        let path = self.entries_path();
        let link_metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(ForgeError::io("inspect bounded ledger entries", error)),
        };
        let target_metadata = if link_metadata.file_type().is_symlink() {
            fs::metadata(&path).map_err(|error| ForgeError::io("inspect ledger target", error))?
        } else {
            link_metadata
        };
        if !target_metadata.file_type().is_file() {
            return Err(reader_integrity_error(
                "registered ledger entries are not a regular file",
            ));
        }
        let file = File::open(&path)
            .map_err(|error| ForgeError::io("open bounded ledger entries", error))?;
        if !file
            .metadata()
            .map_err(|error| ForgeError::io("inspect opened ledger entries", error))?
            .file_type()
            .is_file()
        {
            return Err(reader_integrity_error(
                "opened ledger entries are not a regular file",
            ));
        }
        Ok(Some(file))
    }

    /// Start a bounded scan pinned to this registered stream's current tail.
    pub fn bounded_reader(&self) -> Result<LedgerReadSession, ForgeError> {
        let lock = self.acquire_bounded_reader_lock()?;
        let mut remaining_raw_bytes = BOUNDED_READER_PAGE_BYTES;
        let mut file = self.open_bounded_entries()?;
        let pinned_head = if let Some(entries) = file.as_mut() {
            let file_len = entries
                .metadata()
                .map_err(|error| ForgeError::io("inspect bounded ledger length", error))?
                .len();
            if file_len == 0 {
                None
            } else {
                Some(
                    read_tail_position(
                        entries,
                        &self.ledger_id,
                        file_len,
                        &mut remaining_raw_bytes,
                    )?
                    .0,
                )
            }
        } else {
            None
        };
        if let Some(entries) = file.as_mut() {
            entries
                .seek(SeekFrom::Start(0))
                .map_err(|error| ForgeError::io("seek bounded ledger origin", error))?;
        }
        drop(lock);
        Ok(LedgerReadSession {
            _file: file,
            ledger_id: self.ledger_id.clone(),
            pinned_head,
            remaining_raw_bytes,
            forward_rows: 0,
            validated_position: None,
            next_offset: 0,
            read_offset: 0,
            read_buffer: [0; BOUNDED_READER_BUFFER_BYTES],
            read_buffer_pos: 0,
            read_buffer_len: 0,
            previous_entry_hash: None,
            complete: false,
            failed: false,
        })
    }

    /// Resume a bounded scan after validating the original pin and the
    /// acknowledged predecessor against this stream's current bytes.
    pub fn resume_bounded_reader(
        &self,
        resume: LedgerReadResume,
    ) -> Result<LedgerReadSession, ForgeError> {
        let lock = self.acquire_bounded_reader_lock()?;
        let mut remaining_raw_bytes = BOUNDED_READER_PAGE_BYTES;
        let mut file = self
            .open_bounded_entries()?
            .ok_or_else(|| reader_integrity_error("registered ledger file is absent on resume"))?;
        let file_len = file
            .metadata()
            .map_err(|error| ForgeError::io("inspect bounded ledger length", error))?
            .len();
        if file_len == 0 {
            return Err(reader_integrity_error(
                "registered ledger file is empty on resume",
            ));
        }
        let pinned_entry = read_position_row(
            &mut file,
            &self.ledger_id,
            &resume.pinned_head,
            file_len,
            &mut remaining_raw_bytes,
        )?;
        let (next_offset, previous_entry_hash, validated_position) =
            if let Some(acknowledged) = resume.acknowledged.as_ref() {
                if acknowledged.end_offset > resume.pinned_head.end_offset
                    || acknowledged.append_ordinal > resume.pinned_head.append_ordinal
                    || (acknowledged.end_offset == resume.pinned_head.end_offset
                        && acknowledged != &resume.pinned_head)
                    || (acknowledged.end_offset < resume.pinned_head.end_offset
                        && acknowledged.append_ordinal >= resume.pinned_head.append_ordinal)
                {
                    return Err(reader_integrity_error(
                        "acknowledged row is beyond pinned head",
                    ));
                }
                let acknowledged_entry = if acknowledged == &resume.pinned_head {
                    pinned_entry
                } else {
                    read_position_row(
                        &mut file,
                        &self.ledger_id,
                        acknowledged,
                        file_len,
                        &mut remaining_raw_bytes,
                    )?
                };
                (
                    acknowledged.end_offset,
                    Some(acknowledged_entry.entry_hash),
                    Some(acknowledged.clone()),
                )
            } else {
                (0, None, None)
            };
        file.seek(SeekFrom::Start(next_offset))
            .map_err(|error| ForgeError::io("seek bounded ledger resume", error))?;
        drop(lock);
        Ok(LedgerReadSession {
            _file: Some(file),
            ledger_id: self.ledger_id.clone(),
            pinned_head: Some(resume.pinned_head),
            remaining_raw_bytes,
            forward_rows: 0,
            validated_position,
            next_offset,
            read_offset: next_offset,
            read_buffer: [0; BOUNDED_READER_BUFFER_BYTES],
            read_buffer_pos: 0,
            read_buffer_len: 0,
            previous_entry_hash,
            complete: false,
            failed: false,
        })
    }

    pub fn commit_typed<T: Serialize>(
        &self,
        record_kind: impl Into<String>,
        subject_refs: Vec<String>,
        record: &T,
        authority_refs: Vec<String>,
    ) -> Result<CommittedRecordRef, ForgeError> {
        let payload = serde_json::to_value(record)?;
        let entry = self.append(record_kind, subject_refs, payload, authority_refs)?;
        Ok(CommittedRecordRef {
            ledger_id: entry.ledger_id,
            record_kind: entry.record_kind,
            entry_ulid: entry.entry_ulid,
            record_ulid: entry.record_ulid,
            append_ordinal: entry.append_ordinal,
            entry_hash: entry.entry_hash,
            payload_hash: entry.payload_hash,
        })
    }

    pub fn commit_typed_once<T: Serialize>(
        &self,
        record_kind: impl Into<String>,
        idempotency_key: impl Into<String>,
        subject_refs: Vec<String>,
        record: &T,
        authority_refs: Vec<String>,
    ) -> Result<CommittedRecordRef, ForgeError> {
        let record_kind = record_kind.into();
        let idempotency_key = idempotency_key.into();
        let payload = serde_json::to_value(record)?;
        check_redaction(&payload)?;
        let expected_payload_hash = payload_hash(&payload)?;

        self.with_exclusive_lock(|| {
            if let Some(entry) = self.read_entries()?.into_iter().find(|entry| {
                entry.record_kind == record_kind
                    && entry.idempotency_key.as_deref() == Some(idempotency_key.as_str())
            }) {
                if entry.payload_hash != expected_payload_hash
                    || entry.subject_refs != subject_refs
                    || entry.authority_refs != authority_refs
                {
                    return Err(ForgeError::Internal(
                        "ledger_integrity_error: idempotency key payload conflict".into(),
                    ));
                }
                return Ok(committed_record_ref(entry));
            }

            self.append_under_lock(
                record_kind,
                Some(idempotency_key),
                subject_refs,
                payload,
                authority_refs,
            )
            .map(committed_record_ref)
        })
    }

    /// Append a typed record under an idempotency key, failing if the key already exists.
    /// Unlike [`commit_typed_once`], matching keys never return Ok — they always conflict.
    pub fn commit_typed_new<T: Serialize>(
        &self,
        record_kind: impl Into<String>,
        idempotency_key: impl Into<String>,
        subject_refs: Vec<String>,
        record: &T,
        authority_refs: Vec<String>,
    ) -> Result<CommittedRecordRef, ForgeError> {
        let record_kind = record_kind.into();
        let idempotency_key = idempotency_key.into();
        let payload = serde_json::to_value(record)?;
        check_redaction(&payload)?;

        self.with_exclusive_lock(|| {
            if self.read_entries()?.into_iter().any(|entry| {
                entry.record_kind == record_kind
                    && entry.idempotency_key.as_deref() == Some(idempotency_key.as_str())
            }) {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: idempotency key already committed".into(),
                ));
            }

            self.append_under_lock(
                record_kind,
                Some(idempotency_key),
                subject_refs,
                payload,
                authority_refs,
            )
            .map(committed_record_ref)
        })
    }

    pub fn materialize_view(
        &self,
        committed: &CommittedRecordRef,
        path: &Path,
        bytes: &[u8],
    ) -> Result<CompatibilityViewState, ForgeError> {
        self.materialize_aggregate_view(committed, vec![committed.entry_ulid.clone()], path, bytes)
    }

    pub fn materialize_aggregate_view(
        &self,
        committed: &CommittedRecordRef,
        source_entry_ulids: Vec<String>,
        path: &Path,
        bytes: &[u8],
    ) -> Result<CompatibilityViewState, ForgeError> {
        let write_result = (|| -> Result<(), ForgeError> {
            let parent = path
                .parent()
                .ok_or_else(|| ForgeError::Input("view path has no parent".into()))?;
            fs::create_dir_all(parent)
                .map_err(|error| ForgeError::io("create view parent", error))?;
            let temporary = path.with_extension("tmp");
            fs::write(&temporary, bytes).map_err(|error| ForgeError::io("write view", error))?;
            fs::rename(&temporary, path).map_err(|error| ForgeError::io("replace view", error))?;
            Ok(())
        })();
        let state = CompatibilityViewState {
            source_entry_ulid: committed.entry_ulid.clone(),
            source_entry_ulids,
            source_payload_hash: committed.payload_hash.clone(),
            view_path: path.to_string_lossy().into_owned(),
            view_sha256: format!("sha256:{:x}", Sha256::digest(bytes)),
            status: if write_result.is_ok() {
                ViewStatus::Current
            } else {
                ViewStatus::Failed
            },
            failure_class: write_result
                .as_ref()
                .err()
                .map(|error| error.class().into()),
        };
        let state_dir = self
            .root
            .join("ledgers")
            .join(&self.ledger_id)
            .join("views");
        fs::create_dir_all(&state_dir)
            .map_err(|error| ForgeError::io("create view state", error))?;
        fs::write(
            state_dir.join(format!("{}.json", committed.entry_ulid)),
            serde_json::to_vec_pretty(&state)?,
        )
        .map_err(|error| ForgeError::io("write view state", error))?;
        write_result.map(|_| state)
    }

    pub fn rebuild_record_view(
        &self,
        committed: &CommittedRecordRef,
        path: &Path,
    ) -> Result<CompatibilityViewState, ForgeError> {
        let entry = self
            .read_entries()?
            .into_iter()
            .find(|entry| entry.entry_ulid == committed.entry_ulid)
            .ok_or_else(|| ForgeError::Input("committed record not found in ledger".into()))?;
        if entry.payload_hash != committed.payload_hash {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: committed reference payload mismatch".into(),
            ));
        }
        self.materialize_view(committed, path, &serde_json::to_vec_pretty(&entry.payload)?)
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

    fn lock_path(&self) -> PathBuf {
        self.root
            .join("ledgers")
            .join(&self.ledger_id)
            .join("stream.lock")
    }

    fn with_exclusive_lock<T>(
        &self,
        operation: impl FnOnce() -> Result<T, ForgeError>,
    ) -> Result<T, ForgeError> {
        let lock_file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())
            .map_err(|error| ForgeError::io("open ledger stream lock", error))?;
        lock_file
            .lock()
            .map_err(|error| ForgeError::io("lock ledger stream", error))?;
        let result = operation();
        let unlock_result = lock_file
            .unlock()
            .map_err(|error| ForgeError::io("unlock ledger stream", error));
        match result {
            Ok(value) => unlock_result.map(|()| value),
            Err(error) => Err(error),
        }
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

    /// Rebuild the derived MMR cache from the entries on disk and persist it.
    /// `entries.jsonl` is the single source of truth; this repairs the crash
    /// state where `mmr.json` is ahead of (or behind) the surviving entries.
    fn rebuild_mmr(&self) -> Result<(), ForgeError> {
        let entries = self.read_entries()?;
        let mut mmr = MmrState::empty();
        for entry in &entries {
            mmr = mmr.append(&entry.entry_hash)?;
        }
        self.save_mmr(&mmr)
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
        // F-05: the MMR is a derived cache and may be ahead of `entries.jsonl`
        // after a crash. Validate the entry count against the MMR leaf count
        // *before* slicing so a desynced cache yields a typed error, never a
        // slice-out-of-range panic.
        if entries.len() as u64 != mmr.leaf_count {
            return Err(ForgeError::Internal(format!(
                "ledger_integrity_error: entries ({}) and mmr leaves ({}) are desynced",
                entries.len(),
                mmr.leaf_count
            )));
        }
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
    /// Rejects payloads containing plaintext secret sentinels (§7.0c redaction).
    pub fn append(
        &self,
        record_kind: impl Into<String>,
        subject_refs: Vec<String>,
        payload: Value,
        authority_refs: Vec<String>,
    ) -> Result<LedgerEntry, ForgeError> {
        let record_kind = record_kind.into();
        self.with_exclusive_lock(|| {
            self.append_under_lock(record_kind, None, subject_refs, payload, authority_refs)
        })
    }

    fn append_under_lock(
        &self,
        record_kind: String,
        idempotency_key: Option<String>,
        subject_refs: Vec<String>,
        payload: Value,
        authority_refs: Vec<String>,
    ) -> Result<LedgerEntry, ForgeError> {
        check_redaction(&payload)?;
        let last = self.read_last_entry()?;
        // F-25.q: refuse to chain onto an unverified predecessor. Recompute
        // the tail's content hash and require it to equal the stored
        // `entry_hash`, or the file was tampered with between appends and
        // building on it would silently legitimize forged history. (A
        // *consistently* rewritten tail is undetectable here by definition;
        // that class is what verify() and checkpoint signatures exist for.)
        if let Some(tail) = &last {
            if tail.entry_hash != entry_hash(tail)? {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: predecessor entry hash mismatch; run verify() before appending"
                        .into(),
                ));
            }
        }
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
            record_kind,
            idempotency_key,
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
        // F-04: `entries.jsonl` is the single source of truth; `mmr.json` is a
        // derived cache. Writing the entry first makes the crash window leave
        // `entries` ahead of `mmr` — the recoverable direction (rebuild the
        // cache) — rather than `mmr` ahead of `entries`, which had no repair
        // path. `mmr.json` is rebuilt from entries by
        // `quarantine_incomplete_tail` and `verify`'s mismatch message.
        self.write_entry(&entry)?;
        self.save_mmr(&mmr_state)?;
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
        self.verify_legacy_files()?;
        Ok(())
    }

    fn verify_legacy_files(&self) -> Result<(), ForgeError> {
        let entries = self.read_entries()?;
        for entry in entries {
            if entry.record_kind != "legacy_import" {
                continue;
            }
            let path = entry
                .payload
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ForgeError::Internal(
                        "ledger_integrity_error: legacy_import missing path".into(),
                    )
                })?;
            let expected_sha256 = entry
                .payload
                .get("sha256")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ForgeError::Internal(
                        "ledger_integrity_error: legacy_import missing sha256".into(),
                    )
                })?;
            let expected_size = entry
                .payload
                .get("size_bytes")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| {
                    ForgeError::Internal(
                        "ledger_integrity_error: legacy_import missing size_bytes".into(),
                    )
                })?;
            let file_path = self.root.join(path);
            let metadata = fs::metadata(&file_path).map_err(|e| {
                ForgeError::io(
                    format!("legacy_import missing file {}", file_path.display()),
                    e,
                )
            })?;
            if metadata.len() != expected_size {
                return Err(ForgeError::Internal(format!(
                    "ledger_integrity_error: legacy_import size mismatch for {path}"
                )));
            }
            let bytes = fs::read(&file_path).map_err(|e| {
                ForgeError::io(
                    format!("read legacy_import file {}", file_path.display()),
                    e,
                )
            })?;
            let actual_sha256 = format!("sha256:{:x}", Sha256::digest(&bytes));
            if actual_sha256 != expected_sha256 {
                return Err(ForgeError::Internal(format!(
                    "ledger_integrity_error: legacy_import sha256 mismatch for {path}"
                )));
            }
        }
        Ok(())
    }

    // ---- Witness receipts ----

    fn witness_receipts_path(&self) -> PathBuf {
        self.root.join("ledgers").join("witness-receipts.jsonl")
    }

    /// Read all witness receipts from the global witness log.
    pub fn read_witness_receipts(&self) -> Result<Vec<WitnessReceipt>, ForgeError> {
        let path = self.witness_receipts_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path).map_err(|e| ForgeError::io("open witness receipts", e))?;
        let mut receipts = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| ForgeError::io("read witness receipts", e))?;
            if line.trim().is_empty() {
                continue;
            }
            let receipt: WitnessReceipt = serde_json::from_str(&line)
                .map_err(|e| ForgeError::Serialization(format!("parse receipt: {e}")))?;
            receipts.push(receipt);
        }
        Ok(receipts)
    }

    /// Append a witness receipt for a global checkpoint hash.
    /// The witness signs with its own key (independent from the checkpoint signer).
    pub fn append_witness_receipt(
        &self,
        global_checkpoint_hash: &str,
        witness_signer_id: impl Into<String>,
        witness_signing_key: &SigningKey,
    ) -> Result<WitnessReceipt, ForgeError> {
        let receipt_ulid = ulid()?;
        let signed_at = chrono::Utc::now().to_rfc3339();
        let mut receipt = WitnessReceipt {
            version: RECORD_VERSION.into(),
            receipt_ulid,
            global_checkpoint_hash: global_checkpoint_hash.into(),
            witness_signer_id: witness_signer_id.into(),
            witness_signing_algorithm: "ed25519".into(),
            witness_signature: String::new(),
            receipt_hash: String::new(),
            signed_at,
        };
        let canonical_bytes = witness_receipt_canonical_bytes(&receipt)?;
        receipt.receipt_hash = sha256_domain("sea-forge/witness-receipt/v1", &canonical_bytes);
        receipt.witness_signature = sign_bytes(witness_signing_key, &canonical_bytes);

        let path = self.witness_receipts_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| ForgeError::io("open witness receipts", e))?;
        let mut bytes =
            serde_json::to_vec(&receipt).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        bytes.push(b'\n');
        file.write_all(&bytes)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all())
            .map_err(|e| ForgeError::io("write witness receipt", e))?;
        Ok(receipt)
    }

    /// Verify witness receipts for a given global checkpoint hash.
    /// `min_witnesses` receipts from independent witnesses are required.
    /// A witness == acting_entity is rejected (not independent).
    pub fn verify_witness_receipts(
        &self,
        global_checkpoint_hash: &str,
        witness_verifying_keys: &[(String, VerifyingKey)],
        acting_entity_id: &str,
        min_witnesses: usize,
    ) -> Result<AssuranceLevel, ForgeError> {
        let receipts = self.read_witness_receipts()?;
        let matching: Vec<&WitnessReceipt> = receipts
            .iter()
            .filter(|r| r.global_checkpoint_hash == global_checkpoint_hash)
            .collect();
        let mut verified = 0;
        let mut verified_witness_ids = std::collections::HashSet::new();
        for receipt in &matching {
            // Independence: witness must not be the acting entity.
            if receipt.witness_signer_id == acting_entity_id {
                continue;
            }
            // Find the verifying key for this witness.
            let key_entry = witness_verifying_keys
                .iter()
                .find(|(id, _)| id == &receipt.witness_signer_id);
            let Some((_, vk)) = key_entry else {
                continue;
            };
            // Verify receipt hash.
            let canonical_bytes = witness_receipt_canonical_bytes(receipt)?;
            let expected_hash = sha256_domain("sea-forge/witness-receipt/v1", &canonical_bytes);
            if expected_hash != receipt.receipt_hash {
                continue;
            }
            // Verify signature.
            if verify_signature(vk, &canonical_bytes, &receipt.witness_signature).is_err() {
                continue;
            }
            if verified_witness_ids.insert(receipt.witness_signer_id.clone()) {
                verified += 1;
            }
        }
        if verified >= min_witnesses && min_witnesses > 0 {
            Ok(AssuranceLevel::ExternallyVerified)
        } else {
            Ok(AssuranceLevel::LocalTamperEvident)
        }
    }

    // ---- Crash recovery ----

    /// Move an incomplete uncheckpointed tail to quarantine.
    /// Returns the number of quarantined lines.
    pub fn quarantine_incomplete_tail(&self) -> Result<usize, ForgeError> {
        let path = self.entries_path();
        if !path.exists() {
            return Ok(0);
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| ForgeError::io("read entries for quarantine", e))?;
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        let mut last_valid = 0usize;
        for (i, line) in lines.iter().enumerate() {
            if serde_json::from_str::<LedgerEntry>(line).is_err() {
                break;
            }
            last_valid = i + 1;
        }
        let quarantined = lines.len() - last_valid;
        if quarantined > 0 {
            let quarantined_lines = &lines[last_valid..];
            let q_path = self.root.join("ledgers").join("quarantine").join(format!(
                "{}-{}.jsonl",
                self.ledger_id,
                ulid()?
            ));
            let q_content = quarantined_lines.join("\n");
            fs::write(&q_path, q_content.as_bytes())
                .map_err(|e| ForgeError::io("write quarantine", e))?;
            // Truncate entries file to valid lines only.
            let valid_content = lines[..last_valid].join("\n");
            if valid_content.is_empty() {
                fs::write(&path, b"").map_err(|e| ForgeError::io("truncate entries", e))?;
            } else {
                fs::write(&path, format!("{valid_content}\n").as_bytes())
                    .map_err(|e| ForgeError::io("truncate entries", e))?;
            }
        }
        // F-04: always reconcile the derived MMR cache with the surviving
        // entries, so a crash that left `mmr.json` ahead of `entries.jsonl`
        // (the pre-fix commit order) is repaired rather than failing `verify()`
        // forever.
        self.rebuild_mmr()?;
        Ok(quarantined)
    }

    // ---- Key rotation ----

    /// Verify checkpoints using a map of key_id → VerifyingKey.
    /// Old checkpoints verify under their snapshotted key refs;
    /// new checkpoints reject keys not in the map.
    pub fn verify_checkpoints_with_keys(
        &self,
        keys: &[(String, VerifyingKey)],
    ) -> Result<(), ForgeError> {
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
            let vk = keys
                .iter()
                .find(|(id, _)| id == &checkpoint.signing_key_id)
                .map(|(_, k)| k)
                .ok_or_else(|| {
                    ForgeError::Internal(format!(
                        "ledger_integrity_error: signing key {} not in key set",
                        checkpoint.signing_key_id
                    ))
                })?;
            verify_signature(vk, &canonical_bytes, &checkpoint.signature)?;
            if checkpoint.previous_checkpoint_hash.as_ref() != previous_hash.as_ref() {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: checkpoint chain break".into(),
                ));
            }
            previous_hash = Some(checkpoint.checkpoint_hash.clone());
        }
        Ok(())
    }
}

fn hash_entry(entry: &LedgerEntry) -> String {
    entry.entry_hash.clone()
}

fn committed_record_ref(entry: LedgerEntry) -> CommittedRecordRef {
    CommittedRecordRef {
        ledger_id: entry.ledger_id,
        record_kind: entry.record_kind,
        entry_ulid: entry.entry_ulid,
        record_ulid: entry.record_ulid,
        append_ordinal: entry.append_ordinal,
        entry_hash: entry.entry_hash,
        payload_hash: entry.payload_hash,
    }
}

/// Manages multiple ledger streams and creates/verifies global checkpoints.
pub struct LedgerManager {
    root: PathBuf,
}

impl LedgerManager {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, ForgeError> {
        let root = root.into();
        fs::create_dir_all(root.join("ledgers"))
            .map_err(|e| ForgeError::io("create ledgers root", e))?;
        Ok(Self { root })
    }

    pub fn create_pre_action_assurance(
        &self,
        key_dir: &Path,
        key_id: &str,
        acting_entity_id: &str,
        witnesses: &[(String, PathBuf, String)],
        min_witnesses: usize,
        required_refs: &[&CommittedRecordRef],
    ) -> Result<PreActionAssurance, ForgeError> {
        if witnesses.len() < min_witnesses {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: independent witness adapter unavailable".into(),
            ));
        }
        let witness_ids = witnesses
            .iter()
            .map(|(witness_id, _, _)| witness_id)
            .collect::<std::collections::BTreeSet<_>>();
        if witness_ids.len() != witnesses.len() {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: duplicate witness identity".into(),
            ));
        }
        if !key_dir.is_absolute() || key_dir.starts_with(&self.root) {
            return Err(ForgeError::Input(
                "signing key directory must be absolute and outside ledger root".into(),
            ));
        }
        let key = load_or_create_signing_key(key_dir, key_id)?;
        if self.global_checkpoints_path().exists() {
            self.verify_global_checkpoints(&key.verifying_key())?;
        }
        for stream_id in self.list_stream_ids()? {
            let stream = self.open_stream(&stream_id, "pre_action")?;
            stream.verify()?;
            stream.verify_checkpoints(&key.verifying_key())?;
            let last_entry = stream.read_last_entry()?;
            let last_checkpoint = stream.read_last_checkpoint()?;
            if last_entry.as_ref().is_some_and(|entry| {
                last_checkpoint
                    .as_ref()
                    .is_none_or(|checkpoint| checkpoint.last_ordinal < entry.append_ordinal)
            }) {
                stream.create_checkpoint(&key, key_id)?;
            }
        }
        let checkpoint = self.create_global_checkpoint(&key, key_id)?;
        self.verify_global_checkpoints(&key.verifying_key())?;
        let mut witness_keys = Vec::new();
        for (witness_id, witness_key_dir, witness_key_id) in witnesses {
            if witness_id == acting_entity_id {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: witness must be independent".into(),
                ));
            }
            let witness_key = load_or_create_signing_key(witness_key_dir, witness_key_id)?;
            let receipt = self.witness_global_checkpoint(&checkpoint, witness_id, &witness_key)?;
            self.open_stream("witness", "witness_stream")?
                .commit_typed(
                    "witness_receipt",
                    vec![checkpoint.global_checkpoint_hash.clone()],
                    &receipt,
                    vec![],
                )?;
            witness_keys.push((witness_id.clone(), witness_key.verifying_key()));
        }
        let witness_assurance = self.verify_witness_receipts(
            &checkpoint.global_checkpoint_hash,
            &witness_keys,
            acting_entity_id,
            min_witnesses,
        )?;
        if min_witnesses > 0 && witness_assurance != AssuranceLevel::ExternallyVerified {
            return Err(ForgeError::Internal(
                "ledger_integrity_error: insufficient independent witnesses".into(),
            ));
        }
        let final_checkpoint = if witnesses.is_empty() {
            checkpoint.clone()
        } else {
            self.open_stream("witness", "witness_stream")?
                .create_checkpoint(&key, key_id)?;
            self.create_global_checkpoint(&key, key_id)?
        };
        self.verify_global_checkpoints(&key.verifying_key())?;
        for required in required_refs {
            let stream = self.open_stream(&required.ledger_id, "assurance-check")?;
            if !stream
                .read_entries()?
                .iter()
                .any(|entry| entry.entry_ulid == required.entry_ulid)
            {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: required record is not checkpointed".into(),
                ));
            }
        }
        Ok(PreActionAssurance {
            global_checkpoint_hash: final_checkpoint.global_checkpoint_hash,
            covered_entries: required_refs
                .iter()
                .map(|reference| reference.entry_ulid.clone())
                .collect(),
        })
    }

    pub fn open_stream(
        &self,
        ledger_id: impl Into<String>,
        writer_identity_ref: impl Into<String>,
    ) -> Result<LedgerStream, ForgeError> {
        LedgerStream::open(&self.root, ledger_id, writer_identity_ref)
    }

    fn global_checkpoints_path(&self) -> PathBuf {
        self.root.join("ledgers").join("global-checkpoints.jsonl")
    }

    /// Enumerate all stream directories under ledgers/.
    pub fn list_stream_ids(&self) -> Result<Vec<String>, ForgeError> {
        let ledgers_dir = self.root.join("ledgers");
        if !ledgers_dir.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in
            fs::read_dir(&ledgers_dir).map_err(|e| ForgeError::io("read ledgers dir", e))?
        {
            let entry = entry.map_err(|e| ForgeError::io("read dir entry", e))?;
            if entry
                .file_type()
                .map_err(|e| ForgeError::io("file type", e))?
                .is_dir()
            {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name != "quarantine" {
                    ids.push(name);
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    /// Create a signed global checkpoint committing all active stream roots.
    pub fn create_global_checkpoint(
        &self,
        signing_key: &SigningKey,
        signing_key_id: impl Into<String>,
    ) -> Result<GlobalCheckpoint, ForgeError> {
        let stream_ids = self.list_stream_ids()?;
        let mut stream_roots = Vec::new();
        for id in &stream_ids {
            let stream = self.open_stream(id, "global_cp")?;
            let mmr = stream.load_mmr()?;
            let root = mmr.root()?;
            stream_roots.push((id.clone(), root));
        }
        let global_root = global_root(&stream_roots)?;

        // Read previous global checkpoint hash.
        let previous = self.read_last_global_checkpoint()?;
        let previous_hash = previous
            .as_ref()
            .map(|cp| cp.global_checkpoint_hash.clone());

        let mut cp = GlobalCheckpoint {
            version: RECORD_VERSION.into(),
            checkpoint_ulid: ulid()?,
            stream_roots,
            global_root,
            previous_global_checkpoint_hash: previous_hash,
            global_checkpoint_hash: String::new(),
            signing_algorithm: "ed25519".into(),
            signing_key_id: signing_key_id.into(),
            signature: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let canonical_bytes = global_checkpoint_canonical_bytes(&cp)?;
        cp.global_checkpoint_hash =
            sha256_domain("sea-forge/global-checkpoint/v1", &canonical_bytes);
        cp.signature = sign_bytes(signing_key, &canonical_bytes);

        // Persist.
        let path = self.global_checkpoints_path();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| ForgeError::io("open global checkpoints", e))?;
        let mut bytes =
            serde_json::to_vec(&cp).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        bytes.push(b'\n');
        file.write_all(&bytes)
            .and_then(|_| file.flush())
            .and_then(|_| file.sync_all())
            .map_err(|e| ForgeError::io("write global checkpoint", e))?;
        Ok(cp)
    }

    /// Read all global checkpoints.
    pub fn read_global_checkpoints(&self) -> Result<Vec<GlobalCheckpoint>, ForgeError> {
        let path = self.global_checkpoints_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path).map_err(|e| ForgeError::io("open global checkpoints", e))?;
        let mut checkpoints = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| ForgeError::io("read global checkpoints", e))?;
            if line.trim().is_empty() {
                continue;
            }
            let cp: GlobalCheckpoint = serde_json::from_str(&line)
                .map_err(|e| ForgeError::Serialization(format!("parse global cp: {e}")))?;
            checkpoints.push(cp);
        }
        Ok(checkpoints)
    }

    /// Read the last global checkpoint.
    pub fn read_last_global_checkpoint(&self) -> Result<Option<GlobalCheckpoint>, ForgeError> {
        let cps = self.read_global_checkpoints()?;
        Ok(cps.into_iter().last())
    }

    /// Verify the global checkpoint chain, signatures, and stream root coverage.
    pub fn verify_global_checkpoints(
        &self,
        verifying_key: &VerifyingKey,
    ) -> Result<(), ForgeError> {
        let checkpoints = self.read_global_checkpoints()?;
        let mut previous_hash: Option<String> = None;
        for cp in &checkpoints {
            let canonical_bytes = global_checkpoint_canonical_bytes(cp)?;
            if sha256_domain("sea-forge/global-checkpoint/v1", &canonical_bytes)
                != cp.global_checkpoint_hash
            {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: global checkpoint hash mismatch".into(),
                ));
            }
            verify_signature(verifying_key, &canonical_bytes, &cp.signature)?;
            // Verify global_root recomputes from stream_roots.
            let recomputed = global_root(&cp.stream_roots)?;
            if recomputed != cp.global_root {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: global root mismatch".into(),
                ));
            }
            if cp.previous_global_checkpoint_hash.as_ref() != previous_hash.as_ref() {
                return Err(ForgeError::Internal(
                    "ledger_integrity_error: global checkpoint chain break".into(),
                ));
            }
            previous_hash = Some(cp.global_checkpoint_hash.clone());
        }
        Ok(())
    }

    pub fn global_checkpoint_covers_record<T: Serialize>(
        &self,
        global_checkpoint_hash: &str,
        record_kind: &str,
        record: &T,
    ) -> Result<bool, ForgeError> {
        let expected_payload = serde_json::to_value(record)?;
        let checkpoint = self
            .read_global_checkpoints()?
            .into_iter()
            .find(|checkpoint| checkpoint.global_checkpoint_hash == global_checkpoint_hash)
            .ok_or_else(|| ForgeError::Input("global checkpoint not found".into()))?;
        for (stream_id, expected_root) in checkpoint.stream_roots {
            let stream = self.open_stream(&stream_id, "checkpoint-coverage")?;
            stream.verify()?;
            let mut state = MmrState::empty();
            let mut target_ordinal = None;
            for entry in stream.read_entries()? {
                state = state.append(&entry.entry_hash)?;
                if entry.record_kind == record_kind && entry.payload == expected_payload {
                    target_ordinal = Some(entry.append_ordinal);
                }
                if state.root()? == expected_root {
                    if target_ordinal.is_some_and(|ordinal| ordinal <= entry.append_ordinal) {
                        return Ok(true);
                    }
                    break;
                }
            }
        }
        Ok(false)
    }

    /// Witness a global checkpoint: create a receipt signed by an independent witness.
    pub fn witness_global_checkpoint(
        &self,
        global_checkpoint: &GlobalCheckpoint,
        witness_signer_id: impl Into<String>,
        witness_signing_key: &SigningKey,
    ) -> Result<WitnessReceipt, ForgeError> {
        let stream = self.open_stream("witness", "witness_stream")?;
        stream.append_witness_receipt(
            &global_checkpoint.global_checkpoint_hash,
            witness_signer_id,
            witness_signing_key,
        )
    }

    /// Verify witness receipts for a global checkpoint hash.
    pub fn verify_witness_receipts(
        &self,
        global_checkpoint_hash: &str,
        witness_verifying_keys: &[(String, VerifyingKey)],
        acting_entity_id: &str,
        min_witnesses: usize,
    ) -> Result<AssuranceLevel, ForgeError> {
        let stream = self.open_stream("witness", "verify_witness")?;
        stream.verify_witness_receipts(
            global_checkpoint_hash,
            witness_verifying_keys,
            acting_entity_id,
            min_witnesses,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::OpenOptions,
        panic::{catch_unwind, AssertUnwindSafe},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    const READER_ROW_BYTES: usize = 2 * 1024 * 1024;

    fn reader_stream() -> (tempfile::TempDir, LedgerStream) {
        let temp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(temp.path(), "case_reader", "reader-test").unwrap();
        (temp, stream)
    }

    fn replace_rows(stream: &LedgerStream, rows: &[LedgerEntry]) {
        let mut bytes = Vec::new();
        for entry in rows {
            bytes.extend(serde_json::to_vec(entry).unwrap());
            bytes.push(b'\n');
        }
        fs::write(stream.entries_path(), bytes).unwrap();
    }

    fn value_object_row(entry: &LedgerEntry) -> Vec<u8> {
        let mut bytes = serde_json::to_vec(&serde_json::to_value(entry).unwrap()).unwrap();
        bytes.push(b'\n');
        bytes
    }

    fn append_sized_row(
        stream: &LedgerStream,
        prior_rows: &mut Vec<LedgerEntry>,
        raw_bytes_including_lf: usize,
    ) -> LedgerEntry {
        let prior_file = fs::read(stream.entries_path()).unwrap_or_default();
        let mut entry = stream
            .append(
                "reader_record",
                vec![],
                serde_json::json!({"pad": ""}),
                vec![],
            )
            .unwrap();
        let empty_len = serde_json::to_vec(&entry).unwrap().len();
        assert!(raw_bytes_including_lf > empty_len + 1);
        let padding = raw_bytes_including_lf - empty_len - 1;
        entry.payload = serde_json::json!({"pad": "x".repeat(padding)});
        entry.payload_hash = payload_hash(&entry.payload).unwrap();
        entry.entry_hash = entry_hash(&entry).unwrap();
        let mut row = serde_json::to_vec(&entry).unwrap();
        row.push(b'\n');
        assert_eq!(row.len(), raw_bytes_including_lf);
        let mut file = prior_file;
        file.extend(row);
        fs::write(stream.entries_path(), file).unwrap();
        prior_rows.push(entry.clone());
        entry
    }

    fn next_row(session: &mut LedgerReadSession) -> LedgerReadRow {
        match session.next_step().unwrap() {
            LedgerReadStep::Row(row) => *row,
            step => panic!("expected candidate row, got {step:?}"),
        }
    }

    fn drain_reader(stream: &LedgerStream) -> Result<Vec<LedgerReadRow>, ForgeError> {
        let mut session = stream.bounded_reader()?;
        let mut rows = Vec::new();
        loop {
            match session.next_step()? {
                LedgerReadStep::Row(row) => rows.push(*row),
                LedgerReadStep::Complete => return Ok(rows),
                LedgerReadStep::LimitReached => {
                    return Err(ForgeError::Internal("bounded reader limit reached".into()))
                }
            }
        }
    }

    fn raw_checksum(row_including_lf: &[u8]) -> [u8; 32] {
        Sha256::digest(row_including_lf).into()
    }

    #[test]
    fn bounded_reader_returns_exact_typed_positions_and_distinct_completion() {
        let (_temp, stream) = reader_stream();
        let first = stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let second = stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();
        let expected_bytes = fs::read(stream.entries_path()).unwrap();
        let mut session = stream.bounded_reader().unwrap();
        let pin = session.pinned_head().unwrap();
        assert_eq!(pin.entry_ulid, second.entry_ulid);
        assert_eq!(pin.append_ordinal, 1);
        let row0 = next_row(&mut session);
        let row1 = next_row(&mut session);
        assert_eq!(row0.entry, first);
        assert_eq!(row1.entry, second);
        assert_eq!(row0.entry.append_ordinal, 0);
        assert_eq!(row0.entry.previous_entry_hash, None);
        let first_len = serde_json::to_vec(&row0.entry).unwrap().len() + 1;
        let second_len = serde_json::to_vec(&row1.entry).unwrap().len() + 1;
        assert_eq!(
            (row0.position.start_offset, row0.position.end_offset),
            (0, first_len as u64)
        );
        assert_eq!(row1.position.start_offset, first_len as u64);
        assert_eq!(row1.position.end_offset, (first_len + second_len) as u64);
        assert_eq!(row0.position.entry_hash, row0.entry.entry_hash);
        assert_eq!(row1.position.entry_hash, row1.entry.entry_hash);
        assert_eq!(
            row0.position.raw_row_checksum,
            raw_checksum(&expected_bytes[..first_len])
        );
        assert_eq!(
            row1.position.raw_row_checksum,
            raw_checksum(&expected_bytes[first_len..first_len + second_len])
        );
        assert_eq!(session.next_step().unwrap(), LedgerReadStep::Complete);
    }

    #[test]
    fn bounded_reader_preserves_unknown_top_level_and_hashed_payload_fields() {
        let (_temp, stream) = reader_stream();
        let mut entry = stream
            .append(
                "sfwp_event",
                vec![],
                serde_json::json!({"kind":"", "detail":null, "future_payload":{"v":1}}),
                vec![],
            )
            .unwrap();
        entry.payload["future_payload"]["v"] = serde_json::json!(2);
        entry.payload_hash = payload_hash(&entry.payload).unwrap();
        entry.entry_hash = entry_hash(&entry).unwrap();
        let mut row: Value = serde_json::to_value(&entry).unwrap();
        row["future_top_level"] = serde_json::json!({"accepted": true});
        let bytes = serde_json::to_vec(&row).unwrap();
        let mut bytes = bytes;
        bytes.push(b'\n');
        fs::write(stream.entries_path(), bytes).unwrap();
        let rows = drain_reader(&stream).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].entry.payload["future_payload"]["v"], 2);

        row["payload"]["future_payload"]["v"] = serde_json::json!(3);
        let mut tampered = serde_json::to_vec(&row).unwrap();
        tampered.push(b'\n');
        fs::write(stream.entries_path(), tampered).unwrap();
        assert!(drain_reader(&stream).is_err());
    }

    #[test]
    fn bounded_reader_proves_absent_and_zero_byte_streams_empty() {
        let (_temp, stream) = reader_stream();
        let mut absent = stream.bounded_reader().unwrap();
        assert!(absent.pinned_head().is_none());
        assert_eq!(absent.next_step().unwrap(), LedgerReadStep::Complete);
        fs::write(stream.entries_path(), b"").unwrap();
        let mut zero = stream.bounded_reader().unwrap();
        assert!(zero.pinned_head().is_none());
        assert_eq!(zero.next_step().unwrap(), LedgerReadStep::Complete);
    }

    #[test]
    fn bounded_reader_rejects_whitespace_torn_and_malformed_history() {
        let (_temp, stream) = reader_stream();
        for bytes in [
            b" \n".as_slice(),
            b"{\"partial\":".as_slice(),
            b"not-json\n".as_slice(),
        ] {
            fs::write(stream.entries_path(), bytes).unwrap();
            assert!(drain_reader(&stream).is_err());
        }
    }

    #[test]
    fn bounded_reader_rejects_nonregular_and_path_io_failures() {
        let (_temp, stream) = reader_stream();
        fs::create_dir(stream.entries_path()).unwrap();
        assert!(stream.bounded_reader().is_err());
        fs::remove_dir(stream.entries_path()).unwrap();
        fs::remove_dir_all(stream.entries_path().parent().unwrap()).unwrap();
        assert!(stream.bounded_reader().is_err());
    }

    #[test]
    fn bounded_reader_fails_closed_when_cooperative_lock_is_held() {
        let (_temp, stream) = reader_stream();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let lock_path = stream.lock_path();
        let (locked_tx, locked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let holder = thread::spawn(move || {
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .open(lock_path)
                .unwrap();
            lock.lock().unwrap();
            let _ = locked_tx.send(());
            let _ = release_rx.recv();
        });
        let ready = locked_rx.recv_timeout(Duration::from_secs(1));
        if ready.is_err() {
            let released = release_tx.send(());
            let holder_joined = holder.join();
            assert!(released.is_ok());
            assert!(holder_joined.is_ok());
            assert!(ready.is_ok(), "lock holder did not acquire the lock");
            return;
        }

        let (outcome, released, holder_joined, worker_joined) = thread::scope(|scope| {
            let (result_tx, result_rx) = mpsc::channel();
            let stream_ref = &stream;
            let worker = scope.spawn(move || {
                let started = Instant::now();
                let result =
                    catch_unwind(AssertUnwindSafe(|| stream_ref.bounded_reader().is_err())).ok();
                let _ = result_tx.send((started.elapsed(), result));
            });
            let outcome = result_rx.recv_timeout(Duration::from_secs(2));
            let released = release_tx.send(());
            let holder_joined = holder.join();
            let worker_joined = worker.join();
            (outcome, released, holder_joined, worker_joined)
        });
        assert!(released.is_ok());
        assert!(holder_joined.is_ok());
        assert!(worker_joined.is_ok());
        assert!(ready.is_ok());
        let (elapsed, failed) = outcome.expect("bounded lock attempt must return");
        assert_eq!(failed, Some(true), "lock contention must fail closed");
        assert!(
            elapsed >= Duration::from_millis(40),
            "reader returned before the 50 ms lock-wait deadline: {elapsed:?}"
        );
        assert!(
            elapsed <= Duration::from_millis(150),
            "reader exceeded a tolerant lock-wait bound: {elapsed:?}"
        );
    }

    #[test]
    fn bounded_reader_accepts_exact_row_cap_and_rejects_cap_plus_one() {
        let (_temp, stream) = reader_stream();
        let mut rows = Vec::new();
        append_sized_row(&stream, &mut rows, READER_ROW_BYTES);
        let mut session = stream.bounded_reader().unwrap();
        assert_eq!(next_row(&mut session).entry.append_ordinal, 0);
        assert_eq!(session.remaining_raw_bytes, 0);
        assert_eq!(session.next_step().unwrap(), LedgerReadStep::Complete);

        let (_temp, oversized) = reader_stream();
        let mut rows = Vec::new();
        append_sized_row(&oversized, &mut rows, READER_ROW_BYTES + 1);
        assert!(drain_reader(&oversized).is_err());
    }

    #[test]
    fn bounded_reader_charges_pin_and_forward_reads_to_one_exact_page_budget() {
        let (_temp, stream) = reader_stream();
        let mut rows = Vec::new();
        append_sized_row(&stream, &mut rows, READER_ROW_BYTES);
        let mut session = stream.bounded_reader().unwrap();
        let head = next_row(&mut session);
        assert_eq!(head.entry.append_ordinal, 0);
        assert_eq!(session.remaining_raw_bytes, 0);
        assert_eq!(session.next_step().unwrap(), LedgerReadStep::Complete);
    }

    #[test]
    fn bounded_reader_stops_before_aggregate_budget_cap_plus_one_without_partial_ack() {
        let (_temp, stream) = reader_stream();
        let mut rows = Vec::new();
        let first_size = 1_048_577;
        let head_size = 1_572_864;
        append_sized_row(&stream, &mut rows, first_size);
        append_sized_row(&stream, &mut rows, head_size);
        let mut session = stream.bounded_reader().unwrap();
        assert_eq!(next_row(&mut session).entry.append_ordinal, 0);
        let last_complete = session.validated_position.clone().unwrap();
        assert!(matches!(
            session.next_step(),
            Err(_) | Ok(LedgerReadStep::LimitReached)
        ));
        assert_eq!(session.validated_position.as_ref(), Some(&last_complete));
    }

    #[test]
    fn bounded_reader_scans_at_most_500_global_rows_including_non_events() {
        let (_exact_temp, exact_head) = reader_stream();
        for ordinal in 0..500 {
            exact_head
                .append(
                    "non_event",
                    vec![],
                    serde_json::json!({"n": ordinal}),
                    vec![],
                )
                .unwrap();
        }
        assert_eq!(drain_reader(&exact_head).unwrap().len(), 500);

        let (_temp, stream) = reader_stream();
        for ordinal in 0..501 {
            stream
                .append(
                    "non_event",
                    vec![],
                    serde_json::json!({"n": ordinal}),
                    vec![],
                )
                .unwrap();
        }
        let mut session = stream.bounded_reader().unwrap();
        for ordinal in 0..500 {
            assert_eq!(next_row(&mut session).entry.append_ordinal, ordinal);
        }
        assert_eq!(session.next_step().unwrap(), LedgerReadStep::LimitReached);
        assert_eq!(
            session.validated_position.as_ref().unwrap().append_ordinal,
            499
        );
    }

    #[test]
    fn bounded_reader_keeps_pin_immutable_when_a_supported_append_follows_pin() {
        let (_temp, stream) = reader_stream();
        let first = stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let mut session = stream.bounded_reader().unwrap();
        let pin = session.pinned_head().unwrap().clone();
        let appended = stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();
        assert_ne!(pin.entry_ulid, appended.entry_ulid);
        assert_eq!(next_row(&mut session).entry, first);
        assert_eq!(session.next_step().unwrap(), LedgerReadStep::Complete);
    }

    #[test]
    fn bounded_reader_resumes_from_original_pin_and_acknowledged_predecessor() {
        let (_temp, stream) = reader_stream();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let second = stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();
        let mut initial = stream.bounded_reader().unwrap();
        let pinned_head = initial.pinned_head().unwrap().clone();
        let acknowledged = next_row(&mut initial).position;
        let appended = stream
            .append("reader_record", vec![], serde_json::json!({"n": 2}), vec![])
            .unwrap();
        let mut resumed = stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: pinned_head.clone(),
                acknowledged: Some(acknowledged),
            })
            .unwrap();
        assert_eq!(next_row(&mut resumed).entry, second);
        assert_eq!(resumed.next_step().unwrap(), LedgerReadStep::Complete);
        assert_ne!(pinned_head.entry_ulid, appended.entry_ulid);
    }

    #[test]
    fn bounded_reader_rejects_resume_after_predecessor_bytes_change() {
        let (_temp, stream) = reader_stream();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();
        let mut initial = stream.bounded_reader().unwrap();
        let pinned_head = initial.pinned_head().unwrap().clone();
        let acknowledged = next_row(&mut initial).position;
        let path = stream.entries_path();
        let mut bytes = fs::read(&path).unwrap();
        bytes[0] ^= 1;
        fs::write(path, bytes).unwrap();
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head,
                acknowledged: Some(acknowledged),
            })
            .is_err());
    }

    #[test]
    fn bounded_reader_rejects_valid_typed_resume_byte_changes() {
        let (_temp, predecessor_stream) = reader_stream();
        predecessor_stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        predecessor_stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();
        let mut initial = predecessor_stream.bounded_reader().unwrap();
        let pinned_head = initial.pinned_head().unwrap().clone();
        let acknowledged = next_row(&mut initial).position;
        let path = predecessor_stream.entries_path();
        let original = fs::read(&path).unwrap();
        let row_end = acknowledged.end_offset as usize;
        let entry: LedgerEntry = serde_json::from_slice(&original[..row_end - 1]).unwrap();
        let reordered = value_object_row(&entry);
        assert_eq!(reordered.len(), row_end);
        assert_eq!(
            serde_json::from_slice::<LedgerEntry>(&reordered[..reordered.len() - 1]).unwrap(),
            entry
        );
        assert_eq!(entry.entry_hash, acknowledged.entry_hash);
        assert_ne!(
            raw_checksum(&original[..row_end]),
            raw_checksum(&reordered),
            "fixture must change raw bytes while preserving typed row and boundary"
        );
        let mut changed = reordered;
        changed.extend_from_slice(&original[row_end..]);
        fs::write(path, changed).unwrap();
        assert!(predecessor_stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head,
                acknowledged: Some(acknowledged),
            })
            .is_err());

        let (_temp, pin_stream) = reader_stream();
        pin_stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let initial = pin_stream.bounded_reader().unwrap();
        let pinned_head = initial.pinned_head().unwrap().clone();
        let original = fs::read(pin_stream.entries_path()).unwrap();
        let entry: LedgerEntry = serde_json::from_slice(&original[..original.len() - 1]).unwrap();
        let reordered = value_object_row(&entry);
        assert_eq!(reordered.len(), original.len());
        assert_eq!(
            serde_json::from_slice::<LedgerEntry>(&reordered[..reordered.len() - 1]).unwrap(),
            entry
        );
        assert_eq!(entry.entry_hash, pinned_head.entry_hash);
        assert_ne!(raw_checksum(&original), raw_checksum(&reordered));
        fs::write(pin_stream.entries_path(), reordered).unwrap();
        assert!(pin_stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head,
                acknowledged: None,
            })
            .is_err());
    }

    #[test]
    fn bounded_reader_regression_rejects_pretty_json_resume_position() {
        let (_temp, stream) = reader_stream();
        let entry = stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let mut pretty_row = serde_json::to_vec_pretty(&entry).unwrap();
        pretty_row.push(b'\n');
        assert!(pretty_row[..pretty_row.len() - 1].contains(&b'\n'));
        assert_eq!(
            serde_json::from_slice::<LedgerEntry>(&pretty_row[..pretty_row.len() - 1]).unwrap(),
            entry
        );
        let position = position_for_row(&entry, &pretty_row, 0).unwrap();
        assert_eq!(position.raw_row_checksum, raw_checksum(&pretty_row));
        assert_eq!(position.end_offset, pretty_row.len() as u64);
        fs::write(stream.entries_path(), &pretty_row).unwrap();

        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: position.clone(),
                acknowledged: Some(position),
            })
            .is_err());
    }

    #[test]
    fn bounded_reader_regression_errors_remain_terminal_after_bad_physical_row() {
        let (_temp, stream) = reader_stream();
        let first = stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();
        let path = stream.entries_path();
        let original = fs::read(&path).unwrap();
        let first_len = serde_json::to_vec(&first).unwrap().len() + 1;
        let mut with_garbage = original[..first_len].to_vec();
        with_garbage.extend_from_slice(b"garbage\n");
        with_garbage.extend_from_slice(&original[first_len..]);
        fs::write(path, with_garbage).unwrap();

        let mut session = stream.bounded_reader().unwrap();
        assert_eq!(next_row(&mut session).entry, first);
        assert!(session.next_step().is_err());
        assert!(session.next_step().is_err());
        assert!(session.next_step().is_err());
    }

    #[test]
    fn bounded_reader_rejects_corrupt_pin_and_resume_offset_arithmetic() {
        let (_temp, stream) = reader_stream();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let mut initial = stream.bounded_reader().unwrap();
        let original_pin = initial.pinned_head().unwrap().clone();
        let mut bad_pin = original_pin.clone();
        bad_pin.start_offset += 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: bad_pin,
                acknowledged: None,
            })
            .is_err());
        let mut bad_pin = original_pin.clone();
        bad_pin.end_offset += 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: bad_pin,
                acknowledged: None,
            })
            .is_err());
        let mut bad_pin = original_pin.clone();
        bad_pin.append_ordinal += 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: bad_pin,
                acknowledged: None,
            })
            .is_err());
        let mut bad_pin = original_pin.clone();
        bad_pin.entry_ulid.push('X');
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: bad_pin,
                acknowledged: None,
            })
            .is_err());
        let mut bad_pin = original_pin.clone();
        bad_pin.entry_hash.push_str("corrupt");
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: bad_pin,
                acknowledged: None,
            })
            .is_err());
        let mut bad_pin = original_pin.clone();
        bad_pin.raw_row_checksum[0] ^= 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: bad_pin,
                acknowledged: None,
            })
            .is_err());
        let mut overflow_pin = original_pin.clone();
        overflow_pin.start_offset = u64::MAX;
        overflow_pin.end_offset = u64::MAX;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: overflow_pin,
                acknowledged: None,
            })
            .is_err());

        let acknowledged = next_row(&mut initial).position;
        let mut bad_ack = acknowledged.clone();
        bad_ack.start_offset += 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin.clone(),
                acknowledged: Some(bad_ack),
            })
            .is_err());
        let mut overflow_ack = acknowledged.clone();
        overflow_ack.end_offset = u64::MAX;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin.clone(),
                acknowledged: Some(overflow_ack),
            })
            .is_err());
        let mut bad_ack = acknowledged.clone();
        bad_ack.end_offset += 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin.clone(),
                acknowledged: Some(bad_ack),
            })
            .is_err());
        let mut bad_ack = acknowledged.clone();
        bad_ack.append_ordinal += 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin.clone(),
                acknowledged: Some(bad_ack),
            })
            .is_err());
        let mut bad_ack = acknowledged.clone();
        bad_ack.entry_ulid.push('X');
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin.clone(),
                acknowledged: Some(bad_ack),
            })
            .is_err());
        let mut bad_ack = acknowledged.clone();
        bad_ack.entry_hash.push_str("corrupt");
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin.clone(),
                acknowledged: Some(bad_ack),
            })
            .is_err());
        let mut bad_ack = acknowledged;
        bad_ack.raw_row_checksum[0] ^= 1;
        assert!(stream
            .resume_bounded_reader(LedgerReadResume {
                pinned_head: original_pin,
                acknowledged: Some(bad_ack),
            })
            .is_err());
    }

    #[test]
    fn bounded_reader_offset_addition_is_checked_at_native_u64_boundary() {
        assert_eq!(checked_ledger_offset_add(17, 25).unwrap(), 42);
        assert!(checked_ledger_offset_add(u64::MAX, 1).is_err());
        assert_eq!(checked_ledger_ordinal_add(0).unwrap(), 1);
        assert!(checked_ledger_ordinal_add(u64::MAX).is_err());
    }

    #[test]
    fn bounded_reader_rejects_broken_ordinals_linkage_and_typed_hashes() {
        let (_temp, stream) = reader_stream();
        let first = stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let second = stream
            .append("reader_record", vec![], serde_json::json!({"n": 1}), vec![])
            .unwrap();

        let mut bad_origin_ordinal = first.clone();
        bad_origin_ordinal.append_ordinal = 1;
        bad_origin_ordinal.entry_hash = entry_hash(&bad_origin_ordinal).unwrap();
        replace_rows(&stream, &[bad_origin_ordinal]);
        assert!(drain_reader(&stream).is_err());

        let mut bad_origin_link = first.clone();
        bad_origin_link.previous_entry_hash = Some("sha256:invented-origin".into());
        bad_origin_link.entry_hash = entry_hash(&bad_origin_link).unwrap();
        replace_rows(&stream, &[bad_origin_link]);
        assert!(drain_reader(&stream).is_err());

        for invalid in [
            ("ledger_id", "other_ledger"),
            ("version", "other-version"),
            ("canonicalization", "other-canonicalization"),
            ("hash_algorithm", "other-hash-algorithm"),
        ] {
            let mut bad_constant = first.clone();
            match invalid.0 {
                "ledger_id" => bad_constant.ledger_id = invalid.1.into(),
                "version" => bad_constant.version = invalid.1.into(),
                "canonicalization" => bad_constant.canonicalization = invalid.1.into(),
                "hash_algorithm" => bad_constant.hash_algorithm = invalid.1.into(),
                _ => unreachable!(),
            }
            bad_constant.entry_hash = entry_hash(&bad_constant).unwrap();
            replace_rows(&stream, &[bad_constant]);
            assert!(
                drain_reader(&stream).is_err(),
                "accepted invalid {}",
                invalid.0
            );
        }

        let mut bad_ordinal = second.clone();
        bad_ordinal.append_ordinal = 4;
        bad_ordinal.entry_hash = entry_hash(&bad_ordinal).unwrap();
        replace_rows(&stream, &[first.clone(), bad_ordinal]);
        assert!(drain_reader(&stream).is_err());

        let mut bad_link = second.clone();
        bad_link.previous_entry_hash = Some("sha256:wrong".into());
        bad_link.entry_hash = entry_hash(&bad_link).unwrap();
        replace_rows(&stream, &[first.clone(), bad_link]);
        assert!(drain_reader(&stream).is_err());

        let mut bad_payload_hash = second.clone();
        bad_payload_hash.payload_hash = "sha256:wrong".into();
        bad_payload_hash.entry_hash = entry_hash(&bad_payload_hash).unwrap();
        replace_rows(&stream, &[first.clone(), bad_payload_hash]);
        assert!(drain_reader(&stream).is_err());

        let mut bad_entry_hash = second;
        bad_entry_hash.entry_hash = "sha256:wrong-entry-hash".into();
        replace_rows(&stream, &[first, bad_entry_hash]);
        assert!(drain_reader(&stream).is_err());
    }

    #[test]
    fn bounded_reader_never_returns_partial_row_or_treats_limit_as_complete() {
        let (_temp, stream) = reader_stream();
        stream
            .append("reader_record", vec![], serde_json::json!({"n": 0}), vec![])
            .unwrap();
        let path = stream.entries_path();
        let mut bytes = fs::read(&path).unwrap();
        bytes.pop();
        fs::write(path, bytes).unwrap();
        assert!(drain_reader(&stream).is_err());

        let (_temp, limited) = reader_stream();
        for ordinal in 0..501 {
            limited
                .append(
                    "non_event",
                    vec![],
                    serde_json::json!({"n": ordinal}),
                    vec![],
                )
                .unwrap();
        }
        let mut session = limited.bounded_reader().unwrap();
        for _ in 0..500 {
            let _ = next_row(&mut session);
        }
        assert_ne!(session.next_step().unwrap(), LedgerReadStep::Complete);
    }

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
    fn open_rejects_traversal_ledger_id() {
        // F-14 regression: a traversal-shaped ledger id must be rejected before
        // any directory is created.
        let tmp = tempfile::tempdir().unwrap();
        let result = LedgerStream::open(tmp.path(), "esc/../../outside_ledger", "audit");
        assert!(result.is_err(), "traversal ledger id must be rejected");
        assert!(
            !tmp.path().join("outside_ledger").exists(),
            "no directory may be created outside the ledgers dir"
        );
        // A normal server-minted style id still works.
        LedgerStream::open(tmp.path(), "case-case_20260710T120000Z_ab12cd34", "audit").unwrap();
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

    #[test]
    fn committed_truth_survives_failed_view_materialization() {
        let tmp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(tmp.path(), "case_demo", "writer_01").unwrap();
        let committed = stream
            .commit_typed("demo", vec![], &serde_json::json!({"value": 1}), vec![])
            .unwrap();
        let blocked_parent = tmp.path().join("blocked");
        fs::write(&blocked_parent, b"not a directory").unwrap();
        assert!(stream
            .materialize_view(&committed, &blocked_parent.join("view.json"), b"view")
            .is_err());
        stream.verify().unwrap();
        let state_path = tmp
            .path()
            .join("ledgers/case_demo/views")
            .join(format!("{}.json", committed.entry_ulid()));
        let state: CompatibilityViewState =
            serde_json::from_slice(&fs::read(state_path).unwrap()).unwrap();
        assert_eq!(state.status, ViewStatus::Failed);
        assert_eq!(state.source_entry_ulid, committed.entry_ulid());
    }

    #[test]
    fn rebuild_record_view_is_byte_identical() {
        let tmp = tempfile::tempdir().unwrap();
        let stream = LedgerStream::open(tmp.path(), "case_demo", "writer_01").unwrap();
        let record = serde_json::json!({"alpha": 1, "beta": [2, 3]});
        let committed = stream
            .commit_typed("demo", vec![], &record, vec![])
            .unwrap();
        let path = tmp.path().join("view.json");
        stream.rebuild_record_view(&committed, &path).unwrap();
        assert_eq!(
            fs::read(&path).unwrap(),
            serde_json::to_vec_pretty(&record).unwrap()
        );
    }

    #[test]
    fn pre_action_assurance_rejects_duplicate_witnesses() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("state");
        let manager = LedgerManager::new(&root).unwrap();
        let duplicate = (
            "witness_one".into(),
            tmp.path().join("witness-key"),
            "key".into(),
        );
        assert!(manager
            .create_pre_action_assurance(
                &tmp.path().join("signing-key"),
                "signer",
                "actor",
                &[duplicate.clone(), duplicate],
                2,
                &[],
            )
            .is_err());
    }

    #[test]
    fn witnessed_assurance_commits_receipt_before_final_checkpoint() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("state");
        let manager = LedgerManager::new(&root).unwrap();
        let stream = manager.open_stream("case_demo", "actor").unwrap();
        let committed = stream
            .commit_typed(
                "authority_decision",
                vec![],
                &serde_json::json!({"ok": true}),
                vec![],
            )
            .unwrap();
        let assurance = manager
            .create_pre_action_assurance(
                &tmp.path().join("signing-key"),
                "signer",
                "actor",
                &[(
                    "witness_one".into(),
                    tmp.path().join("witness-key"),
                    "key".into(),
                )],
                1,
                &[&committed],
            )
            .unwrap();
        assert!(assurance.covers(&committed));
        assert!(manager
            .open_stream("witness", "verify")
            .unwrap()
            .read_entries()
            .unwrap()
            .iter()
            .any(|entry| entry.record_kind == "witness_receipt"));
        assert_eq!(manager.read_global_checkpoints().unwrap().len(), 2);
    }

    #[test]
    fn checkpoint_coverage_rejects_post_checkpoint_record() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("state");
        let manager = LedgerManager::new(&root).unwrap();
        let stream = manager.open_stream("case_demo", "actor").unwrap();
        stream
            .commit_typed(
                "capability_envelope",
                vec![],
                &serde_json::json!({"id": 1}),
                vec![],
            )
            .unwrap();
        let key = load_or_create_signing_key(&tmp.path().join("keys"), "signer").unwrap();
        let checkpoint = manager.create_global_checkpoint(&key, "signer").unwrap();
        let later = serde_json::json!({"id": 2});
        stream
            .commit_typed("capability_envelope", vec![], &later, vec![])
            .unwrap();
        assert!(!manager
            .global_checkpoint_covers_record(
                &checkpoint.global_checkpoint_hash,
                "capability_envelope",
                &later,
            )
            .unwrap());
    }
}
