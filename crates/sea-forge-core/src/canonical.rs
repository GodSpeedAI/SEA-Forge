//! One canonical-JSON primitive for every hash producer (SUP-09d).
//!
//! Four crates previously carried byte-equivalent copies of this algorithm
//! (ledger, self-model, evidence, authority); the evidence and authority
//! copies relied implicitly on `serde_json::Map`'s sorted iteration instead
//! of sorting explicitly. All copies now delegate here.
//!
//! # The profile (`jcs-nfc-v1`) — what it is and is not
//!
//! - Object keys are sorted lexicographically by UTF-8 bytes (explicitly,
//!   not via map-order coincidence).
//! - String *values* are NFC-normalized. Object **keys are not** NFC'd — a
//!   deliberate, conservative asymmetry: it cannot cause collisions because
//!   distinct NFC spellings remain distinct identities.
//! - Arrays keep their order; numbers serialize via serde_json's default
//!   formatter.
//!
//! This is **not RFC 8785 JCS**: no UTF-16-based key ordering and no
//! ECMAScript number canonicalization. External JCS verifiers will compute
//! different hashes for the same value. The `jcs-nfc-v1` label persists in
//! committed ledger records as an identifier for *this* profile.

use crate::errors::ForgeError;

/// Canonical JSON bytes for a JSON value: explicit key sort + NFC values.
pub fn canonical_json(value: &serde_json::Value) -> Result<Vec<u8>, ForgeError> {
    fn sorted(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => {
                // Recurse into every value: NFC normalization applies at all
                // depths. (Historically two of the four pre-consolidation
                // copies normalized only top-level strings — divergence the
                // consolidation exposed; see the plan's SUP-09d entry.)
                let mut entries: Vec<(String, serde_json::Value)> = map.into_iter().collect();
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                let map: serde_json::Map<String, serde_json::Value> =
                    entries.into_iter().map(|(k, v)| (k, sorted(v))).collect();
                serde_json::Value::Object(map)
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.into_iter().map(sorted).collect())
            }
            serde_json::Value::String(s) => serde_json::Value::String(
                unicode_normalization::UnicodeNormalization::nfc(s.as_str()).collect(),
            ),
            other => other,
        }
    }
    let sorted = sorted(value.clone());
    serde_json::to_vec(&sorted).map_err(|e| ForgeError::Serialization(e.to_string()))
}

/// Canonical JSON bytes for any serializable value.
pub fn canonical_bytes<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, ForgeError> {
    let json = serde_json::to_value(value).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    canonical_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use sha2::{Digest, Sha256};

    // SUP-09d golden vectors: the exact bytes every hash producer must agree
    // on. Any drift here shifts every payload/entry/rebuild hash in the
    // system, so these assertions are the consolidation contract.
    #[test]
    fn keys_are_sorted_and_values_nfc_normalized() {
        // "A" + combining diaeresis composes under NFC to U+00C4 at ANY
        // depth; keys sort lexicographically.
        let value = json!({
            "b": 2,
            "a": format!("A{}ngstrom", '\u{0308}'),
            "z": {"m": format!("A{}", '\u{0308}')},
        });
        let bytes = canonical_json(&value).unwrap();
        let expected = format!(
            r#"{{"a":"{}ngstrom","b":2,"z":{{"m":"{}"}}}}"#,
            '\u{00C4}', '\u{00C4}'
        );
        assert_eq!(String::from_utf8(bytes).unwrap(), expected);
    }

    #[test]
    fn arrays_keep_order_and_nested_objects_sort() {
        let value = json!({"z": [{"y": 1, "x": [3, 1, 2]}]});
        let bytes = canonical_json(&value).unwrap();
        let expected = br#"{"z":[{"x":[3,1,2],"y":1}]}"#;
        assert_eq!(bytes, &expected[..]);
    }

    #[test]
    fn nfc_equivalent_spellings_hash_identically() {
        // Precomposed U+00C4 vs "A" + combining U+0308: identical canonical
        // bytes, therefore identical hashes.
        let composed = json!({"name": '\u{00C4}'});
        let decomposed = json!({"name": format!("A{}", '\u{0308}')});
        let h = |v: &serde_json::Value| {
            let bytes = canonical_json(v).unwrap();
            format!("sha256:{:x}", Sha256::digest(&bytes))
        };
        assert_eq!(h(&composed), h(&decomposed));
    }

    #[test]
    fn keys_are_not_nfc_normalized_documented_asymmetry() {
        // Distinct key spellings stay distinct — conservative by design.
        let composed = json!({"\u{00C5}": 1});
        let decomposed = json!({"A\u{0308}": 1});
        assert_ne!(
            canonical_json(&composed).unwrap(),
            canonical_json(&decomposed).unwrap()
        );
    }
}
