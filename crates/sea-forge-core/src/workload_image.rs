//! The typed identity of a sandbox workload.
//!
//! `Operation::ExecuteSandboxCommand.image` stays a plain string on the wire
//! and in the ledger. This type is what a backend parses that string into, so
//! it can tell an OCI image from a native executable without guessing.
//!
//! Syntax:
//!
//! * `native:sha256:<64 lowercase hex>@<os>-<arch>`: a native executable, pinned
//!   by the sha256 of its bytes and the platform it runs on.
//! * anything else: an OCI reference, `[registry/]name[:tag][@sha256:<64 hex>]`.

use std::fmt;

const NATIVE_PREFIX: &str = "native:sha256:";

/// A parsed workload image reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkloadImage {
    /// An OCI image reference, syntactically valid and otherwise unresolved.
    Oci { reference: String },
    /// A native executable: the sha256 of its bytes and `<os>-<arch>`.
    Native { sha256: String, platform: String },
}

/// A workload image string that is neither a native identity nor an OCI reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidWorkloadImage(pub String);

impl fmt::Display for InvalidWorkloadImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InvalidWorkloadImage {}

impl WorkloadImage {
    pub fn parse(text: &str) -> Result<Self, InvalidWorkloadImage> {
        if let Some(rest) = text.strip_prefix(NATIVE_PREFIX) {
            return parse_native(text, rest);
        }
        if text.starts_with("native:") {
            return Err(invalid(
                text,
                "a native image needs `native:sha256:<64 hex>@<os>-<arch>`",
            ));
        }
        if is_oci_reference(text) {
            Ok(Self::Oci {
                reference: text.to_string(),
            })
        } else {
            Err(invalid(text, "not an OCI reference or a native identity"))
        }
    }
}

fn invalid(text: &str, why: &str) -> InvalidWorkloadImage {
    InvalidWorkloadImage(format!("workload image `{text}` is invalid: {why}"))
}

fn parse_native(text: &str, rest: &str) -> Result<WorkloadImage, InvalidWorkloadImage> {
    let (hex, platform) = rest
        .split_once('@')
        .ok_or_else(|| invalid(text, "missing `@<os>-<arch>`"))?;
    if !is_lower_hex(hex, 64) {
        return Err(invalid(
            text,
            "the digest must be 64 lowercase hex characters",
        ));
    }
    let valid_platform = platform.split_once('-').is_some_and(|(os, arch)| {
        [os, arch]
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(is_platform_byte))
    });
    if !valid_platform {
        return Err(invalid(text, "the platform must be `<os>-<arch>`"));
    }
    Ok(WorkloadImage::Native {
        sha256: hex.to_string(),
        platform: platform.to_string(),
    })
}

fn is_platform_byte(b: u8) -> bool {
    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'
}

fn is_lower_hex(text: &str, len: usize) -> bool {
    text.len() == len
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The OCI distribution grammar, checked by hand: no regex dependency.
fn is_oci_reference(text: &str) -> bool {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_graphic()) {
        return false;
    }
    let name = match text.split_once('@') {
        Some((name, digest)) => match digest.strip_prefix("sha256:") {
            Some(hex) if is_lower_hex(hex, 64) => name,
            _ => return false,
        },
        None => text,
    };
    // A tag is the part of the last path segment after its colon.
    let last_slash = name.rfind('/').map_or(0, |i| i + 1);
    let repository = match name[last_slash..].rfind(':') {
        Some(colon) => {
            if !valid_tag(&name[last_slash + colon + 1..]) {
                return false;
            }
            &name[..last_slash + colon]
        }
        None => name,
    };
    let mut parts: Vec<&str> = repository.split('/').collect();
    if parts.len() > 1 && is_registry(parts[0]) {
        parts.remove(0);
    }
    !parts.is_empty() && parts.iter().all(|part| is_path_component(part))
}

fn valid_tag(tag: &str) -> bool {
    let mut bytes = tag.bytes();
    tag.len() <= 128
        && bytes
            .next()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// A registry is `host[:port]`, recognized by a dot, a port or `localhost`.
fn is_registry(part: &str) -> bool {
    let (host, port) = match part.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (part, None),
    };
    let host_ok = !host.is_empty()
        && host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
        && (host.contains('.') || host == "localhost" || port.is_some());
    let port_ok = port.is_none_or(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    host_ok && port_ok
}

/// A repository path component per the OCI distribution grammar:
/// `[a-z0-9]+ (separator [a-z0-9]+)*` where a separator is a single `.`,
/// a single `_`, a double `__`, or a run of one or more `-`.
fn is_path_component(part: &str) -> bool {
    let alnum = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
    let separator = |b: u8| matches!(b, b'.' | b'_' | b'-');
    let bytes = part.as_bytes();
    let (Some(&first), Some(&last)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    if !alnum(first) || !alnum(last) {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        if alnum(bytes[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && separator(bytes[i]) {
            i += 1;
        }
        // Bounded by alphanumerics because the component starts and ends
        // with one. An empty run means a byte that is neither alphanumeric
        // nor a separator (`+`, `!`, uppercase): reject it, or the loop
        // would never advance.
        let run = &bytes[start..i];
        let valid = !run.is_empty()
            && (run == b"." || run == b"_" || run == b"__" || run.iter().all(|&b| b == b'-'));
        if !valid {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn oci_references_parse() {
        for text in [
            "registry.example/cell:1",
            "cell",
            "localhost:5000/team/cell",
            "ghcr.io/org/cell@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "registry.example/team/my__image:1",
            "registry.example/team/my--image:1",
            "registry.example/a-b-c:1",
            "registry.example/a.b_c:1",
        ] {
            assert!(
                matches!(WorkloadImage::parse(text), Ok(WorkloadImage::Oci { .. })),
                "{text}"
            );
        }
    }

    #[test]
    fn malformed_references_are_refused() {
        for text in [
            "",
            "has space/cell:1",
            "Registry.example/Cell:1",
            "cell:",
            "cell@sha256:short",
            "cell@md5:0123",
            "registry.example/a___b:1",
            "registry.example/_a:1",
            "registry.example/a.:1",
            "registry.example/a..b:1",
            "registry.example/a._b:1",
            "registry.example/a-.b:1",
            // Interior bytes that are neither alphanumeric nor a separator
            // (review F26: these used to loop forever).
            "a+b",
            "aBb",
            "registry.example/a!b:1",
            "registry.example/team/a%b:1",
            "native:",
            "native:sha256:abc@windows-x86_64",
            "native:sha256:0123@windows-x86_64",
        ] {
            assert!(WorkloadImage::parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn native_identity_parses() {
        let text = format!("native:sha256:{HEX}@windows-x86_64");
        assert_eq!(
            WorkloadImage::parse(&text),
            Ok(WorkloadImage::Native {
                sha256: HEX.into(),
                platform: "windows-x86_64".into(),
            })
        );
    }

    #[test]
    fn native_identity_needs_a_lowercase_digest_and_a_platform() {
        for text in [
            format!("native:sha256:{HEX}"),
            format!("native:sha256:{}@windows-x86_64", HEX.to_uppercase()),
            format!("native:sha256:{HEX}@windows"),
            format!("native:sha256:{HEX}@-x86_64"),
        ] {
            assert!(WorkloadImage::parse(&text).is_err(), "{text}");
        }
    }
}
