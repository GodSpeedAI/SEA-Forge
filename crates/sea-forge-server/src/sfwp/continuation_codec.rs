// This private codec is intentionally unwired until a separate integration
// release; keep the temporary dead-code allowance limited to this module.
#![allow(dead_code)]

use sea_forge_ledger::signing::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_TOKEN_BYTES: usize = 4_096;
const MAX_PAYLOAD_BYTES: usize = 2_048;
const SIGNATURE_SUFFIX_BYTES: usize = 96;
const SEPARATOR_AND_SIGNATURE_BYTES: usize = 97;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenError {
    TooLong,
    Layout,
    PayloadTooLarge,
    Signature,
    PayloadJson,
    UnsupportedVersion,
    NonCanonicalNumber,
    MalformedDigest,
    BindingMismatch,
    InvalidFilterResolution,
    InvalidPosition,
}

#[derive(Clone, Copy, Debug)]
struct ExpectedContext<'a> {
    registered_stream: &'a str,
    from_filter: Option<&'a str>,
    to_filter: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EncodedPosition {
    start_offset: String,
    end_offset: String,
    append_ordinal: String,
    entry_ulid: String,
    entry_hash: String,
    raw_row_checksum: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EncodedPayload {
    version: u8,
    stream_digest: String,
    from_digest: String,
    to_digest: String,
    pinned_head: EncodedPosition,
    acknowledged: EncodedPosition,
    resolved_from_ordinal: Option<String>,
    resolved_to_ordinal: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Position {
    start_offset: u64,
    end_offset: u64,
    append_ordinal: u64,
    entry_ulid: String,
    entry_hash: String,
    raw_row_checksum: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DecodedPayload {
    version: u8,
    stream_digest: String,
    from_digest: String,
    to_digest: String,
    pinned_head: Position,
    acknowledged: Position,
    resolved_from_ordinal: Option<u64>,
    resolved_to_ordinal: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DigestDomain {
    Stream,
    From,
    To,
}

fn encode(payload: &EncodedPayload, signing_key: &SigningKey) -> Result<String, TokenError> {
    if payload.version != 1 {
        return Err(TokenError::UnsupportedVersion);
    }
    for digest in [
        &payload.stream_digest,
        &payload.from_digest,
        &payload.to_digest,
    ] {
        parse_lower_hex_32(digest)?;
    }
    let pinned_head = decode_position(&payload.pinned_head)?;
    let acknowledged = decode_position(&payload.acknowledged)?;
    validate_frontier(&acknowledged, &pinned_head)?;
    validate_resolution(
        payload.resolved_from_ordinal.as_deref(),
        payload.resolved_from_ordinal.is_some(),
        acknowledged.append_ordinal,
    )?;
    validate_resolution(
        payload.resolved_to_ordinal.as_deref(),
        payload.resolved_to_ordinal.is_some(),
        acknowledged.append_ordinal,
    )?;

    let payload_bytes = serde_json::to_vec(payload).map_err(|_| TokenError::PayloadJson)?;
    if payload_bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(TokenError::PayloadTooLarge);
    }
    let signature = sea_forge_ledger::signing::sign_bytes(signing_key, &payload_bytes);
    if signature.len() != SIGNATURE_SUFFIX_BYTES
        || !signature.is_ascii()
        || !signature.starts_with("ed25519:")
    {
        return Err(TokenError::Signature);
    }
    let token_len = payload_bytes
        .len()
        .checked_add(SEPARATOR_AND_SIGNATURE_BYTES)
        .ok_or(TokenError::TooLong)?;
    if token_len > MAX_TOKEN_BYTES {
        return Err(TokenError::TooLong);
    }
    let payload_text = std::str::from_utf8(&payload_bytes).map_err(|_| TokenError::PayloadJson)?;
    Ok(format!("{payload_text}.{signature}"))
}

fn decode(
    token: &str,
    verifying_key: &VerifyingKey,
    expected: ExpectedContext<'_>,
) -> Result<DecodedPayload, TokenError> {
    if token.len() > MAX_TOKEN_BYTES {
        return Err(TokenError::TooLong);
    }
    let separator = token
        .len()
        .checked_sub(SEPARATOR_AND_SIGNATURE_BYTES)
        .ok_or(TokenError::Layout)?;
    if separator == 0 {
        return Err(TokenError::Layout);
    }
    if separator > MAX_PAYLOAD_BYTES {
        return Err(TokenError::PayloadTooLarge);
    }
    let separator_byte = token
        .as_bytes()
        .get(separator)
        .copied()
        .ok_or(TokenError::Layout)?;
    if separator_byte != b'.' {
        return Err(TokenError::Layout);
    }
    let payload = token.get(..separator).ok_or(TokenError::Layout)?;
    let signature = token.get(separator + 1..).ok_or(TokenError::Layout)?;
    if signature.len() != SIGNATURE_SUFFIX_BYTES
        || !signature.is_ascii()
        || !signature.starts_with("ed25519:")
    {
        return Err(TokenError::Layout);
    }
    if !is_canonical_signature_encoding(signature) {
        return Err(TokenError::Signature);
    }
    sea_forge_ledger::signing::verify_signature(verifying_key, payload.as_bytes(), signature)
        .map_err(|_| TokenError::Signature)?;

    let encoded: EncodedPayload =
        serde_json::from_str(payload).map_err(|_| TokenError::PayloadJson)?;
    if encoded.version != 1 {
        return Err(TokenError::UnsupportedVersion);
    }
    for digest in [
        &encoded.stream_digest,
        &encoded.from_digest,
        &encoded.to_digest,
    ] {
        parse_lower_hex_32(digest)?;
    }
    let expected_stream =
        continuation_digest(DigestDomain::Stream, Some(expected.registered_stream))?;
    let expected_from = continuation_digest(DigestDomain::From, expected.from_filter)?;
    let expected_to = continuation_digest(DigestDomain::To, expected.to_filter)?;
    if encoded.stream_digest != expected_stream
        || encoded.from_digest != expected_from
        || encoded.to_digest != expected_to
    {
        return Err(TokenError::BindingMismatch);
    }

    let pinned_head = decode_position(&encoded.pinned_head)?;
    let acknowledged = decode_position(&encoded.acknowledged)?;
    validate_frontier(&acknowledged, &pinned_head)?;
    let resolved_from_ordinal = validate_resolution(
        encoded.resolved_from_ordinal.as_deref(),
        expected.from_filter.is_some(),
        acknowledged.append_ordinal,
    )?;
    let resolved_to_ordinal = validate_resolution(
        encoded.resolved_to_ordinal.as_deref(),
        expected.to_filter.is_some(),
        acknowledged.append_ordinal,
    )?;

    Ok(DecodedPayload {
        version: encoded.version,
        stream_digest: encoded.stream_digest,
        from_digest: encoded.from_digest,
        to_digest: encoded.to_digest,
        pinned_head,
        acknowledged,
        resolved_from_ordinal,
        resolved_to_ordinal,
    })
}

fn is_canonical_signature_encoding(signature: &str) -> bool {
    let Some(encoded) = signature.strip_prefix("ed25519:") else {
        return false;
    };
    let bytes = encoded.as_bytes();
    bytes.len() == 88
        && bytes[..86]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/'))
        && bytes[86..] == *b"=="
        && matches!(bytes[85], b'A' | b'Q' | b'g' | b'w')
}

fn continuation_digest(domain: DigestDomain, value: Option<&str>) -> Result<String, TokenError> {
    let tag = match domain {
        DigestDomain::Stream => b"sea-forge/casework/continuation/stream/v1".as_slice(),
        DigestDomain::From => b"sea-forge/casework/continuation/from/v1".as_slice(),
        DigestDomain::To => b"sea-forge/casework/continuation/to/v1".as_slice(),
    };
    let mut hasher = Sha256::new();
    hasher.update(tag);
    match value {
        None => {
            hasher.update([0]);
            hasher.update(0u64.to_be_bytes());
        }
        Some(value) => {
            let byte_len = u64::try_from(value.len()).map_err(|_| TokenError::Layout)?;
            hasher.update([1]);
            hasher.update(byte_len.to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn parse_canonical_u64(value: &str) -> Result<u64, TokenError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || (bytes.len() > 1 && bytes[0] == b'0')
        || bytes.iter().any(|byte| !byte.is_ascii_digit())
    {
        return Err(TokenError::NonCanonicalNumber);
    }
    value
        .parse::<u64>()
        .map_err(|_| TokenError::NonCanonicalNumber)
}

fn parse_lower_hex_32(value: &str) -> Result<[u8; 32], TokenError> {
    if value.len() != 64 {
        return Err(TokenError::MalformedDigest);
    }
    let mut result = [0; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        result[index] = (lower_hex_nibble(pair[0])? << 4) | lower_hex_nibble(pair[1])?;
    }
    Ok(result)
}

fn validate_resolution(
    resolved: Option<&str>,
    filter_present: bool,
    acknowledged_ordinal: u64,
) -> Result<Option<u64>, TokenError> {
    let Some(resolved) = resolved else {
        return Ok(None);
    };
    if !filter_present {
        return Err(TokenError::InvalidFilterResolution);
    }
    let ordinal = parse_canonical_u64(resolved)?;
    if ordinal > acknowledged_ordinal {
        return Err(TokenError::InvalidFilterResolution);
    }
    Ok(Some(ordinal))
}

fn validate_frontier(acknowledged: &Position, pinned_head: &Position) -> Result<(), TokenError> {
    if acknowledged.start_offset >= acknowledged.end_offset
        || pinned_head.start_offset >= pinned_head.end_offset
        || acknowledged.append_ordinal > pinned_head.append_ordinal
    {
        return Err(TokenError::InvalidPosition);
    }
    if acknowledged.append_ordinal == pinned_head.append_ordinal {
        if acknowledged != pinned_head {
            return Err(TokenError::InvalidPosition);
        }
    } else if acknowledged.end_offset > pinned_head.start_offset {
        return Err(TokenError::InvalidPosition);
    }
    Ok(())
}

fn decode_position(encoded: &EncodedPosition) -> Result<Position, TokenError> {
    Ok(Position {
        start_offset: parse_canonical_u64(&encoded.start_offset)?,
        end_offset: parse_canonical_u64(&encoded.end_offset)?,
        append_ordinal: parse_canonical_u64(&encoded.append_ordinal)?,
        entry_ulid: encoded.entry_ulid.clone(),
        entry_hash: encoded.entry_hash.clone(),
        raw_row_checksum: parse_lower_hex_32(&encoded.raw_row_checksum)?,
    })
}

fn lower_hex_nibble(byte: u8) -> Result<u8, TokenError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(TokenError::MalformedDigest),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_forge_ledger::signing::{sign_bytes, verify_signature};

    const STREAM_DIGEST: &str = "64011d31a2ec18ac24b76124ca758e72e18e4b8221785d3a64ce65bd65a09c95";
    const FROM_DIGEST: &str = "5d68857f779e00c0bb605c37038e399172f30f034c5cc945f2cbf446a4cf6ea4";
    const ABSENT_TO_DIGEST: &str =
        "272da02c07a4e6d4beab0343b73837407144fa87a6cfde4333570c9d92a37a38";
    const EMPTY_TO_DIGEST: &str =
        "f6840dd29443b475865b437b8fb5e012e2a2d66380845fabbac5ca0a79ee87df";

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7; 32])
    }

    fn expected() -> ExpectedContext<'static> {
        ExpectedContext {
            registered_stream: "stream-A",
            from_filter: Some("filter-é"),
            to_filter: None,
        }
    }

    fn valid_payload() -> EncodedPayload {
        let checksum = "0123456789abcdef".repeat(4);
        EncodedPayload {
            version: 1,
            stream_digest: STREAM_DIGEST.into(),
            from_digest: FROM_DIGEST.into(),
            to_digest: ABSENT_TO_DIGEST.into(),
            pinned_head: EncodedPosition {
                start_offset: "100".into(),
                end_offset: "120".into(),
                append_ordinal: "3".into(),
                entry_ulid: "opaque.head.id".into(),
                entry_hash: "opaque-head-hash".into(),
                raw_row_checksum: checksum.clone(),
            },
            acknowledged: EncodedPosition {
                start_offset: "70".into(),
                end_offset: "99".into(),
                append_ordinal: "2".into(),
                entry_ulid: "opaque.ack.id".into(),
                entry_hash: "opaque-ack-hash".into(),
                raw_row_checksum: checksum,
            },
            resolved_from_ordinal: Some("1".into()),
            resolved_to_ordinal: None,
        }
    }

    fn signed_token(payload: &str, signing_key: &SigningKey) -> String {
        let signature = sign_bytes(signing_key, payload.as_bytes());
        verify_signature(&signing_key.verifying_key(), payload.as_bytes(), &signature)
            .expect("fixed local signature verifies through the public ledger API");
        format!("{payload}.{signature}")
    }

    fn decode_signed_value(
        value: &serde_json::Value,
        context: ExpectedContext<'_>,
    ) -> Result<DecodedPayload, TokenError> {
        let raw = serde_json::to_string(value).unwrap();
        let signing_key = key();
        let token = signed_token(&raw, &signing_key);
        decode(&token, &signing_key.verifying_key(), context)
    }

    fn assert_signed_value_error(
        value: &serde_json::Value,
        context: ExpectedContext<'_>,
        expected_error: TokenError,
    ) {
        assert_eq!(decode_signed_value(value, context), Err(expected_error));
    }

    fn payload_json_with_len(target: usize) -> String {
        let mut value = serde_json::to_value(valid_payload()).unwrap();
        let head = value["pinned_head"].as_object_mut().unwrap();
        head.insert(
            "entry_ulid".into(),
            serde_json::Value::String(String::new()),
        );
        let base_len = serde_json::to_vec(&value).unwrap().len();
        assert!(target >= base_len);
        value["pinned_head"]["entry_ulid"] =
            serde_json::Value::String("x".repeat(target - base_len));
        let payload = serde_json::to_string(&value).unwrap();
        assert_eq!(payload.len(), target);
        payload
    }

    fn payload_with_encoded_len(target: usize) -> EncodedPayload {
        let mut payload = valid_payload();
        payload.pinned_head.entry_ulid.clear();
        let base_len = serde_json::to_vec(&payload).unwrap().len();
        assert!(target >= base_len);
        payload.pinned_head.entry_ulid = "x".repeat(target - base_len);
        assert_eq!(serde_json::to_vec(&payload).unwrap().len(), target);
        payload
    }

    fn raw_position(start_offset: u64, end_offset: u64, append_ordinal: u64) -> Position {
        Position {
            start_offset,
            end_offset,
            append_ordinal,
            entry_ulid: "opaque.row.id".into(),
            entry_hash: "opaque.entry.hash".into(),
            raw_row_checksum: [0xabu8; 32],
        }
    }

    #[test]
    fn round_trip_uses_fixed_signature_and_payload_periods() {
        let signing_key = key();
        let token = encode(&valid_payload(), &signing_key).expect("encode valid payload");
        let decoded =
            decode(&token, &signing_key.verifying_key(), expected()).expect("decode valid payload");
        let checksum = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
            0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
            0x89, 0xab, 0xcd, 0xef,
        ];
        assert_eq!(
            decoded,
            DecodedPayload {
                version: 1,
                stream_digest: STREAM_DIGEST.into(),
                from_digest: FROM_DIGEST.into(),
                to_digest: ABSENT_TO_DIGEST.into(),
                pinned_head: Position {
                    start_offset: 100,
                    end_offset: 120,
                    append_ordinal: 3,
                    entry_ulid: "opaque.head.id".into(),
                    entry_hash: "opaque-head-hash".into(),
                    raw_row_checksum: checksum,
                },
                acknowledged: Position {
                    start_offset: 70,
                    end_offset: 99,
                    append_ordinal: 2,
                    entry_ulid: "opaque.ack.id".into(),
                    entry_hash: "opaque-ack-hash".into(),
                    raw_row_checksum: checksum,
                },
                resolved_from_ordinal: Some(1),
                resolved_to_ordinal: None,
            }
        );
        let separator = token.len() - SEPARATOR_AND_SIGNATURE_BYTES;
        assert_eq!(token.as_bytes()[separator], b'.');
        assert_eq!(token.len() - separator - 1, SIGNATURE_SUFFIX_BYTES);
        assert!(token[..separator].contains('.'));
        verify_signature(
            &signing_key.verifying_key(),
            &token.as_bytes()[..separator],
            &token[separator + 1..],
        )
        .expect("encoded token signature verifies through the public ledger API");
    }

    #[test]
    fn signature_tampering_and_wrong_key_are_rejected() {
        let raw = serde_json::to_string(&valid_payload()).unwrap();
        let signing_key = key();
        let token = signed_token(&raw, &signing_key);
        let mut tampered = token.clone();
        let last = tampered.pop().unwrap();
        tampered.push(if last == 'A' { 'B' } else { 'A' });
        assert_eq!(
            decode(&tampered, &signing_key.verifying_key(), expected()),
            Err(TokenError::Signature)
        );
        let separator = token.len() - SEPARATOR_AND_SIGNATURE_BYTES;
        let data_byte = separator + 1 + "ed25519:".len();
        let mut tampered_data = token.clone().into_bytes();
        tampered_data[data_byte] = if tampered_data[data_byte] == b'A' {
            b'B'
        } else {
            b'A'
        };
        let tampered_data = String::from_utf8(tampered_data).unwrap();
        assert_eq!(
            decode(&tampered_data, &signing_key.verifying_key(), expected()),
            Err(TokenError::Signature)
        );

        let mut noncanonical_padding_bits = token.clone().into_bytes();
        let final_data_byte = separator + 1 + "ed25519:".len() + 85;
        noncanonical_padding_bits[final_data_byte] =
            match noncanonical_padding_bits[final_data_byte] {
                b'A' => b'B',
                b'Q' => b'R',
                b'g' => b'h',
                b'w' => b'x',
                _ => unreachable!("signature encoding has canonical final base64 bits"),
            };
        let noncanonical_padding_bits = String::from_utf8(noncanonical_padding_bits).unwrap();
        assert_eq!(
            decode(
                &noncanonical_padding_bits,
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::Signature)
        );
        let other_key = SigningKey::from_bytes(&[8; 32]);
        assert_eq!(
            decode(&token, &other_key.verifying_key(), expected()),
            Err(TokenError::Signature)
        );
    }

    #[test]
    fn outer_and_payload_caps_classify_admission_stages() {
        let signing_key = key();
        let mut oversized = valid_payload();
        oversized.pinned_head.entry_ulid = "é".repeat(MAX_PAYLOAD_BYTES);
        assert_eq!(
            encode(&oversized, &signing_key),
            Err(TokenError::PayloadTooLarge)
        );
        let exact_encoded = payload_with_encoded_len(MAX_PAYLOAD_BYTES);
        let exact_encoded_token =
            encode(&exact_encoded, &signing_key).expect("exact payload cap is issuable");
        assert_eq!(
            exact_encoded_token.len(),
            MAX_PAYLOAD_BYTES + SEPARATOR_AND_SIGNATURE_BYTES
        );

        let payload_4096 = "x".repeat(4_096 - SEPARATOR_AND_SIGNATURE_BYTES);
        assert_eq!(
            signed_token(&payload_4096, &signing_key).len(),
            MAX_TOKEN_BYTES
        );
        assert_eq!(
            decode(
                &signed_token(&payload_4096, &signing_key),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::PayloadTooLarge)
        );
        let payload_4097 = "x".repeat(4_097 - SEPARATOR_AND_SIGNATURE_BYTES);
        assert_eq!(
            decode(
                &signed_token(&payload_4097, &signing_key),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::TooLong)
        );
        let exact_payload = payload_json_with_len(MAX_PAYLOAD_BYTES);
        let exact_token = signed_token(&exact_payload, &signing_key);
        assert_eq!(
            exact_token.len(),
            MAX_PAYLOAD_BYTES + SEPARATOR_AND_SIGNATURE_BYTES
        );
        assert!(decode(&exact_token, &signing_key.verifying_key(), expected()).is_ok());
        let plus_one = signed_token(&payload_json_with_len(MAX_PAYLOAD_BYTES + 1), &signing_key);
        assert_eq!(
            decode(&plus_one, &signing_key.verifying_key(), expected()),
            Err(TokenError::PayloadTooLarge)
        );
    }

    #[test]
    fn multibyte_boundary_is_rejected_without_slicing_panic() {
        let malformed = format!("{}é{}", "x".repeat(32), "x".repeat(96));
        assert_eq!(malformed.len() - SEPARATOR_AND_SIGNATURE_BYTES, 33);
        assert_eq!(
            decode(&malformed, &key().verifying_key(), expected()),
            Err(TokenError::Layout)
        );
    }

    #[test]
    fn signature_verification_precedes_json_parsing() {
        let signing_key = key();
        let other_key = SigningKey::from_bytes(&[8; 32]);
        let bad_signature = signed_token("not-json", &other_key);
        assert_eq!(
            decode(&bad_signature, &signing_key.verifying_key(), expected()),
            Err(TokenError::Signature)
        );
        let signed_invalid_json = signed_token("not-json", &signing_key);
        assert_eq!(
            decode(
                &signed_invalid_json,
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::PayloadJson)
        );
    }

    #[test]
    fn signed_noncompact_reordered_json_is_verified_as_received() {
        let signing_key = key();
        let value = serde_json::to_value(valid_payload()).unwrap();
        let raw = serde_json::to_string_pretty(&value).unwrap();
        let token = signed_token(&raw, &signing_key);
        let decoded = decode(&token, &signing_key.verifying_key(), expected())
            .expect("valid signature over exact noncompact JSON bytes is accepted");
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.pinned_head.entry_ulid, "opaque.head.id");
    }

    #[test]
    fn signature_suffix_shape_and_decoded_length_are_checked() {
        let signing_key = key();
        let raw = "{}";
        let good = sign_bytes(&signing_key, raw.as_bytes());
        for suffix in [
            "ed25519:short".to_string(),
            "wrongalg:".to_string() + &"A".repeat(87),
        ] {
            assert_eq!(
                decode(
                    &format!("{raw}.{suffix}"),
                    &signing_key.verifying_key(),
                    expected()
                ),
                Err(TokenError::Layout)
            );
        }
        let non_ascii = format!("ed25519:{}", "é".repeat(44));
        assert_eq!(non_ascii.len(), SIGNATURE_SUFFIX_BYTES);
        assert_eq!(
            decode(
                &format!("{raw}.{non_ascii}"),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::Layout)
        );
        let decoded_65_bytes = format!("ed25519:{}=", "A".repeat(87));
        assert_eq!(decoded_65_bytes.len(), SIGNATURE_SUFFIX_BYTES);
        assert_eq!(
            decode(
                &format!("{raw}.{decoded_65_bytes}"),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::Signature)
        );
        let invalid_base64 = format!("ed25519:{}", "!".repeat(88));
        assert_eq!(
            decode(
                &format!("{raw}.{invalid_base64}"),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::Signature)
        );
        assert_eq!(good.len(), SIGNATURE_SUFFIX_BYTES);
    }

    #[test]
    fn signed_unknown_payload_and_position_fields_are_rejected() {
        let signing_key = key();
        let mut unknown_payload = serde_json::to_value(valid_payload()).unwrap();
        unknown_payload["unexpected"] = serde_json::Value::Bool(true);
        let raw = serde_json::to_string(&unknown_payload).unwrap();
        assert_eq!(
            decode(
                &signed_token(&raw, &signing_key),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::PayloadJson)
        );

        let mut unknown_position = serde_json::to_value(valid_payload()).unwrap();
        unknown_position["acknowledged"]["unexpected"] = serde_json::Value::Bool(true);
        let raw = serde_json::to_string(&unknown_position).unwrap();
        assert_eq!(
            decode(
                &signed_token(&raw, &signing_key),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::PayloadJson)
        );
        let mut bad_version = valid_payload();
        bad_version.version = 2;
        let raw = serde_json::to_string(&bad_version).unwrap();
        assert_eq!(
            decode(
                &signed_token(&raw, &signing_key),
                &signing_key.verifying_key(),
                expected()
            ),
            Err(TokenError::UnsupportedVersion)
        );
    }

    #[test]
    fn signed_decode_rejects_version_number_digest_and_checksum_mutations() {
        let mut unsupported_version = serde_json::to_value(valid_payload()).unwrap();
        unsupported_version["version"] = serde_json::Value::from(2);
        assert_signed_value_error(
            &unsupported_version,
            expected(),
            TokenError::UnsupportedVersion,
        );

        let mut noncanonical_offset = serde_json::to_value(valid_payload()).unwrap();
        noncanonical_offset["pinned_head"]["start_offset"] =
            serde_json::Value::String("0100".into());
        assert_signed_value_error(
            &noncanonical_offset,
            expected(),
            TokenError::NonCanonicalNumber,
        );

        let mut overflow_ordinal = serde_json::to_value(valid_payload()).unwrap();
        overflow_ordinal["acknowledged"]["append_ordinal"] =
            serde_json::Value::String("18446744073709551616".into());
        assert_signed_value_error(
            &overflow_ordinal,
            expected(),
            TokenError::NonCanonicalNumber,
        );

        let mut malformed_digest = serde_json::to_value(valid_payload()).unwrap();
        malformed_digest["stream_digest"] =
            serde_json::Value::String("not-a-lowercase-64-byte-digest".into());
        assert_signed_value_error(&malformed_digest, expected(), TokenError::MalformedDigest);

        let mut malformed_checksum = serde_json::to_value(valid_payload()).unwrap();
        malformed_checksum["acknowledged"]["raw_row_checksum"] =
            serde_json::Value::String("g".repeat(64));
        assert_signed_value_error(&malformed_checksum, expected(), TokenError::MalformedDigest);
    }

    #[test]
    fn signed_decode_checks_filter_resolution_presence_and_bounds() {
        let mut unresolved_present_filter = serde_json::to_value(valid_payload()).unwrap();
        unresolved_present_filter["resolved_from_ordinal"] = serde_json::Value::Null;
        let decoded = decode_signed_value(&unresolved_present_filter, expected())
            .expect("present filter may be unresolved");
        assert_eq!(decoded.resolved_from_ordinal, None);

        let mut resolved_absent_filter = serde_json::to_value(valid_payload()).unwrap();
        resolved_absent_filter["resolved_to_ordinal"] = serde_json::Value::String("1".into());
        assert_signed_value_error(
            &resolved_absent_filter,
            expected(),
            TokenError::InvalidFilterResolution,
        );

        let mut resolution_beyond_acknowledged = serde_json::to_value(valid_payload()).unwrap();
        resolution_beyond_acknowledged["resolved_from_ordinal"] =
            serde_json::Value::String("3".into());
        assert_signed_value_error(
            &resolution_beyond_acknowledged,
            expected(),
            TokenError::InvalidFilterResolution,
        );

        let mut malformed_resolution = serde_json::to_value(valid_payload()).unwrap();
        malformed_resolution["resolved_from_ordinal"] = serde_json::Value::String("01".into());
        assert_signed_value_error(
            &malformed_resolution,
            expected(),
            TokenError::NonCanonicalNumber,
        );
    }

    #[test]
    fn signed_decode_rejects_invalid_frontier_offsets_and_overlap() {
        for (field, value) in [
            ("end_offset", "69"),
            ("end_offset", "101"),
            ("start_offset", "100"),
            ("append_ordinal", "4"),
        ] {
            let mut invalid = serde_json::to_value(valid_payload()).unwrap();
            invalid["acknowledged"][field] = serde_json::Value::String(value.into());
            assert_signed_value_error(&invalid, expected(), TokenError::InvalidPosition);
        }

        let mut adjacent = serde_json::to_value(valid_payload()).unwrap();
        adjacent["acknowledged"]["end_offset"] = serde_json::Value::String("100".into());
        let decoded = decode_signed_value(&adjacent, expected())
            .expect("acknowledged end may equal the pinned head start");
        assert_eq!(decoded.acknowledged.end_offset, 100);

        let mut equal_ordinal_different_position = serde_json::to_value(valid_payload()).unwrap();
        equal_ordinal_different_position["acknowledged"]["append_ordinal"] =
            serde_json::Value::String("3".into());
        assert_signed_value_error(
            &equal_ordinal_different_position,
            expected(),
            TokenError::InvalidPosition,
        );

        let mut reversed_head = serde_json::to_value(valid_payload()).unwrap();
        reversed_head["pinned_head"]["start_offset"] = serde_json::Value::String("121".into());
        reversed_head["pinned_head"]["end_offset"] = serde_json::Value::String("120".into());
        assert_signed_value_error(&reversed_head, expected(), TokenError::InvalidPosition);
    }

    #[test]
    fn canonical_unsigned_decimal_parser_covers_zero_max_and_overflow() {
        assert_eq!(parse_canonical_u64("0"), Ok(0));
        assert_eq!(parse_canonical_u64("18446744073709551615"), Ok(u64::MAX));
        for value in [
            "",
            "00",
            "01",
            "+1",
            "-1",
            " 1",
            "1 ",
            "18446744073709551616",
        ] {
            assert_eq!(
                parse_canonical_u64(value),
                Err(TokenError::NonCanonicalNumber)
            );
        }
    }

    #[test]
    fn digest_and_checksum_encoding_is_lowercase_fixed_width_hex() {
        let valid = "0123456789abcdef".repeat(4);
        assert_eq!(
            parse_lower_hex_32(&valid),
            Ok([
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
                0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
                0x89, 0xab, 0xcd, 0xef
            ])
        );
        let uppercase = "A".repeat(64);
        let non_hex = format!("{}g", "0".repeat(63));
        for invalid in ["0123456789abcdef", uppercase.as_str(), non_hex.as_str()] {
            assert_eq!(
                parse_lower_hex_32(invalid),
                Err(TokenError::MalformedDigest)
            );
        }
    }

    #[test]
    fn domain_digest_vectors_distinguish_fields_absence_and_exact_utf8() {
        const SAME_STREAM: &str =
            "1959156f875c7700c00c16875bc98e4667a21227a3f344f23a90b4097abb50a1";
        const SAME_FROM: &str = "fe2824c13bb1edd2a64a35bbdaca04816bb5cb2279a93aa6eb7163158c4223f8";
        const SAME_TO: &str = "4d962af8e82dc2042370ded4db15e27cc8b5329edc711221ace974768f956559";
        const ABSENT_STREAM: &str =
            "faf6000b409bb59292749341e8f0f8280f70dc50d835c8c877485cc68f44ca2f";
        const ABSENT_FROM: &str =
            "edc7cc7e5007c91503a3c1eca8500d17e985c4d5245b5cde9382daaea333f702";
        const EMPTY_STREAM: &str =
            "59c7e3dd771f70b705bbd64800cc0a931502eb911651c0d6d488a94844ff5e4c";
        const EMPTY_FROM: &str = "90b46ddd1b546d608cc78a92ec694b699f81ff7a423ec5e9c0307dc52f648489";
        const EMPTY_TO: &str = "f6840dd29443b475865b437b8fb5e012e2a2d66380845fabbac5ca0a79ee87df";
        assert_eq!(
            continuation_digest(DigestDomain::Stream, Some("same-é")),
            Ok(SAME_STREAM.into())
        );
        assert_eq!(
            continuation_digest(DigestDomain::From, Some("same-é")),
            Ok(SAME_FROM.into())
        );
        assert_eq!(
            continuation_digest(DigestDomain::To, Some("same-é")),
            Ok(SAME_TO.into())
        );
        assert_ne!(SAME_STREAM, SAME_FROM);
        assert_ne!(SAME_STREAM, SAME_TO);
        assert_ne!(SAME_FROM, SAME_TO);
        for (domain, absent, empty) in [
            (DigestDomain::Stream, ABSENT_STREAM, EMPTY_STREAM),
            (DigestDomain::From, ABSENT_FROM, EMPTY_FROM),
            (DigestDomain::To, ABSENT_TO_DIGEST, EMPTY_TO),
        ] {
            assert_eq!(continuation_digest(domain, None), Ok(absent.into()));
            assert_eq!(continuation_digest(domain, Some("")), Ok(empty.into()));
            assert_ne!(absent, empty);
        }
        assert_eq!(
            continuation_digest(DigestDomain::Stream, Some("stream-A")),
            Ok(STREAM_DIGEST.into())
        );
        assert_eq!(
            continuation_digest(DigestDomain::From, Some("filter-é")),
            Ok(FROM_DIGEST.into())
        );
        assert_eq!(
            continuation_digest(DigestDomain::To, None),
            Ok(ABSENT_TO_DIGEST.into())
        );
        assert_ne!(STREAM_DIGEST, FROM_DIGEST);
    }

    #[test]
    fn decoded_stream_and_filter_digests_bind_to_expected_context() {
        let signing_key = key();
        let raw = serde_json::to_string(&valid_payload()).unwrap();
        let other = ExpectedContext {
            registered_stream: "stream-B",
            ..expected()
        };
        assert_eq!(
            decode(
                &signed_token(&raw, &signing_key),
                &signing_key.verifying_key(),
                other
            ),
            Err(TokenError::BindingMismatch)
        );
        let other_filter = ExpectedContext {
            from_filter: Some("different-filter"),
            ..expected()
        };
        assert_eq!(
            decode(
                &signed_token(&raw, &signing_key),
                &signing_key.verifying_key(),
                other_filter
            ),
            Err(TokenError::BindingMismatch)
        );
        let other_to_filter = ExpectedContext {
            to_filter: Some("different-to-filter"),
            ..expected()
        };
        assert_eq!(
            decode(
                &signed_token(&raw, &signing_key),
                &signing_key.verifying_key(),
                other_to_filter
            ),
            Err(TokenError::BindingMismatch)
        );
    }

    #[test]
    fn signed_to_filter_binds_present_unresolved_and_zero_resolution() {
        const TO_FILTER: &str = "to-filter-é";
        const TO_DIGEST: &str = "d94b784a7426ce99ceb481f7767c3e872a3dbce05375f63f2a3f03b292d0a7cc";
        let signing_key = key();
        let mut value = serde_json::to_value(valid_payload()).unwrap();
        value["to_digest"] = serde_json::Value::String(TO_DIGEST.into());
        value["resolved_to_ordinal"] = serde_json::Value::String("0".into());
        let raw = serde_json::to_string(&value).unwrap();
        let token = signed_token(&raw, &signing_key);
        let context = ExpectedContext {
            to_filter: Some(TO_FILTER),
            ..expected()
        };

        let decoded = decode(&token, &signing_key.verifying_key(), context)
            .expect("present to-filter with resolution zero decodes");
        assert_eq!(decoded.to_digest, TO_DIGEST);
        assert_eq!(decoded.resolved_to_ordinal, Some(0));

        let different_present_filter = ExpectedContext {
            to_filter: Some("different-to-filter"),
            ..expected()
        };
        assert_eq!(
            decode(
                &token,
                &signing_key.verifying_key(),
                different_present_filter
            ),
            Err(TokenError::BindingMismatch)
        );

        value["resolved_to_ordinal"] = serde_json::Value::Null;
        let unresolved_token = signed_token(&serde_json::to_string(&value).unwrap(), &signing_key);
        let unresolved = decode(&unresolved_token, &signing_key.verifying_key(), context)
            .expect("present to-filter may remain unresolved");
        assert_eq!(unresolved.resolved_to_ordinal, None);

        value["resolved_to_ordinal"] = serde_json::Value::String("3".into());
        let beyond_ack = signed_token(&serde_json::to_string(&value).unwrap(), &signing_key);
        assert_eq!(
            decode(&beyond_ack, &signing_key.verifying_key(), context),
            Err(TokenError::InvalidFilterResolution)
        );
    }

    #[test]
    fn resolved_filter_presence_and_ordinal_bounds_are_validated() {
        assert_eq!(validate_resolution(None, false, 2), Ok(None));
        assert_eq!(validate_resolution(None, true, 2), Ok(None));
        assert_eq!(validate_resolution(Some("0"), true, 2), Ok(Some(0)));
        assert_eq!(validate_resolution(Some("2"), true, 2), Ok(Some(2)));
        assert_eq!(
            validate_resolution(Some("3"), true, 2),
            Err(TokenError::InvalidFilterResolution)
        );
        assert_eq!(
            validate_resolution(Some("0"), false, 2),
            Err(TokenError::InvalidFilterResolution)
        );
        assert_eq!(
            validate_resolution(Some("01"), true, 2),
            Err(TokenError::NonCanonicalNumber)
        );
    }

    #[test]
    fn acknowledged_and_pinned_positions_obey_frontier_semantics() {
        let acknowledged = raw_position(70, 99, 2);
        let pinned = raw_position(100, 120, 3);
        assert_eq!(validate_frontier(&acknowledged, &pinned), Ok(()));
        assert_eq!(
            validate_frontier(&raw_position(70, 101, 2), &pinned),
            Err(TokenError::InvalidPosition)
        );
        assert_eq!(
            validate_frontier(&raw_position(70, 99, 4), &pinned),
            Err(TokenError::InvalidPosition)
        );
        assert_eq!(
            validate_frontier(&raw_position(101, 100, 2), &pinned),
            Err(TokenError::InvalidPosition)
        );
        assert_eq!(
            validate_frontier(&raw_position(u64::MAX, u64::MAX, 2), &pinned),
            Err(TokenError::InvalidPosition)
        );
        let same = raw_position(100, 120, 3);
        assert_eq!(validate_frontier(&same, &same), Ok(()));
        let same_ordinal_different_position = raw_position(100, 121, 3);
        assert_eq!(
            validate_frontier(&same_ordinal_different_position, &same),
            Err(TokenError::InvalidPosition)
        );
    }
}
