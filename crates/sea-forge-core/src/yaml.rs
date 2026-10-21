//! Strict YAML boundary for the workspace.
//!
//! This is the only module that names `serde_saphyr`. Every parse entry point
//! uses the same options, so a call site cannot forget one:
//!
//! - duplicate mapping keys are rejected (serde_yaml silently kept the last one);
//! - merge keys (`<<`) are ordinary keys and are never expanded, so an alias
//!   cannot inject fields into a rule;
//! - YAML 1.1 boolean words (`yes`, `no`, `on`, `off`, `y`, `n`) are not booleans;
//! - unsupported explicit tags are rejected;
//! - input that starts with a UTF-8 byte order mark is rejected;
//! - input larger than [`MAX_INPUT_BYTES`] (4 MiB) is rejected before parsing;
//! - `!!binary` into a `String` yields the literal text, never decoded bytes;
//! - non-finite floats (`.inf`, `.nan`) are rejected in typeless positions.
//!
//! Typed parsing ([`from_str`], [`from_slice`]) also applies the strict adapter
//! in `strict.rs`: the target type decides the meaning of a scalar, and a value
//! that does not fit is an error rather than a silent coercion. Quoted booleans,
//! quoted integers, `~`/`null` for a collection or a required string, and
//! non-canonical integer spellings are rejected at every depth. [`check_syntax`]
//! uses the bare options and is not a typed check.
//!
//! Typeless parsing (into a generic value) is deliberately not offered: it
//! re-types scalars (`no` becomes `false`, `010` becomes `10.0`), so every
//! caller parses into a concrete type.
//!
//! # What the boundary does not guarantee
//!
//! - Integers and booleans reached through `#[serde(flatten)]` or an internally
//!   tagged enum (`#[serde(tag = ...)]`) are buffered by serde before the field
//!   visitor runs, so the spelling pass cannot see their source text. In that
//!   path `1_000` and `0x10` are accepted with their numeric value. Fields of this
//!   kind in the YAML-loaded types: `planner::templates::TemplateOperation::AgentTask`
//!   `max_turns` and `token_budget`. The policy bundle has no flatten or tagged
//!   type, so the authority policy types are not affected (audit 2026-10-12).
//! - Explicit core tags (`!!int`, `!!bool`, `!!str`, `!!seq`, `!!map`, `!!null`,
//!   `!!float`, `!!binary`, `!!timestamp`) are accepted by serde-saphyr even with
//!   `reject_unsupported_tags`, which rejects only unknown tags. Their values are
//!   what the library produces for that tag. Only unknown tags, `!include`
//!   and the robotics tags are rejected. No tracked YAML file in the repository
//!   and no `real` corpus document uses a tag.
//! - Typed positions that accept any scalar text (`String`) take the scalar text
//!   as written, so `s: 123` and `s: true` load as the strings `"123"` and `"true"`.
//!
//! # Effective resource limits
//!
//! Every limit below is the serde-saphyr 2.0.0 `Budget` / `AliasLimits` field
//! value set in [`budget`] and [`options`]. Exceeding a budget is a parse error.
//!
//! | limit | value | meaning |
//! |-------|-------|---------|
//! | input size | 4 MiB | checked before parsing; also `max_reader_input_bytes` |
//! | max depth | 64 | nesting of collections |
//! | max events | 100 000 | parser events for the document |
//! | max nodes | 50 000 | sequences, maps and scalars |
//! | max documents | 1 | a YAML stream holds one document |
//! | max aliases | 1 000 | `*alias` references |
//! | max anchors | 1 000 | `&anchor` definitions |
//! | max alias replayed events | 100 000 | events injected from aliases, across the parse |
//! | max alias expansions per anchor | 10 000 | expansions of one anchor (default was unlimited) |
//! | alias replay stack depth | 64 | nested alias expansion depth (library default) |
//! | total scalar bytes | 4 MiB | summed scalar text |
//! | max merge keys | 10 000 | library default; merge keys are not expanded |
//!
//! The largest real policy document in the corpus is about 3.9 KB, so these
//! limits leave a factor of more than 20 of headroom for real input.

mod strict;

use serde::{de::DeserializeOwned, de::IgnoredAny, Serialize};
use serde_saphyr::{Budget, Options};
use std::fmt;

/// Largest input accepted by any entry point, in bytes.
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;

/// Error returned by every YAML entry point in this module.
#[derive(Debug)]
pub struct YamlError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
}

impl YamlError {
    /// 1-based line of the error, when the parser reports a location.
    pub fn line(&self) -> Option<usize> {
        self.line
    }

    /// 1-based column of the error, when the parser reports a location.
    pub fn column(&self) -> Option<usize> {
        self.column
    }

    fn without_location(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    fn from_parse(error: serde_saphyr::Error) -> Self {
        let location = error.location();
        Self {
            message: error.to_string(),
            line: location.map(|at| at.line() as usize),
            column: location.map(|at| at.column() as usize),
        }
    }
}

impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for YamlError {}

fn budget() -> Budget {
    let mut budget = Budget::default();
    budget.max_reader_input_bytes = Some(MAX_INPUT_BYTES);
    budget.max_depth = 64;
    budget.max_documents = 1;
    budget.max_events = 100_000;
    budget.max_nodes = 50_000;
    budget.max_aliases = 1_000;
    budget.max_anchors = 1_000;
    budget.max_recorded_anchor_events = 100_000;
    budget.max_total_scalar_bytes = MAX_INPUT_BYTES;
    budget
}

fn options() -> Options {
    let mut options = Options::default();
    options.budget = Some(budget());
    options.alias_limits.max_total_replayed_events = 100_000;
    options.alias_limits.max_alias_expansions_per_anchor = 10_000;
    options.duplicate_keys = serde_saphyr::DuplicateKeyPolicy::Error;
    options.merge_keys = serde_saphyr::MergeKeyPolicy::AsOrdinary;
    options.strict_booleans = true;
    options.reject_unsupported_tags = true;
    options.legacy_octal_numbers = false;
    options.angle_conversions = false;
    options.non_finite_float_policy = serde_saphyr::NonFiniteFloatPolicy::Reject;
    // `!!binary aGk=` into a String stays the literal text instead of decoding.
    options.ignore_binary_tag_for_string = true;
    // The `include` and `properties` crate features are not enabled, so there is
    // no include resolver and no property interpolation to configure.
    options
}

fn reject_bom(text: &str) -> Result<(), YamlError> {
    if text.starts_with('\u{feff}') {
        return Err(YamlError::without_location(
            "input starts with a UTF-8 byte order mark",
        ));
    }
    Ok(())
}

fn reject_oversize(text: &str) -> Result<(), YamlError> {
    if text.len() > MAX_INPUT_BYTES {
        return Err(YamlError::without_location(format!(
            "input is {} bytes, larger than the {MAX_INPUT_BYTES} byte limit",
            text.len()
        )));
    }
    Ok(())
}

/// Parses `text` into a concrete type with the strict typing rules.
///
/// The document is parsed twice: once for types and shape, once for canonical
/// integer spelling. Both passes must succeed; the value comes from the first.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, YamlError> {
    reject_bom(text)?;
    reject_oversize(text)?;
    let value = serde_saphyr::with_deserializer_from_str_with_options(text, options(), |de| {
        T::deserialize(strict::Strict::new(de, strict::Mode::Shape))
    })
    .map_err(YamlError::from_parse)?;
    serde_saphyr::with_deserializer_from_str_with_options(text, options(), |de| {
        T::deserialize(strict::Strict::new(de, strict::Mode::Spelling)).map(|_: T| ())
    })
    .map_err(YamlError::from_parse)?;
    Ok(value)
}

/// Parses UTF-8 `bytes` into a concrete type. Invalid UTF-8 is an error.
pub fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, YamlError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| YamlError::without_location(format!("invalid UTF-8: {error}")))?;
    from_str(text)
}

/// Checks that `text` is valid under the strict options without building a value.
pub fn check_syntax(text: &str) -> Result<(), YamlError> {
    reject_bom(text)?;
    reject_oversize(text)?;
    serde_saphyr::from_str_with_options::<IgnoredAny>(text, options())
        .map(|_| ())
        .map_err(YamlError::from_parse)
}

/// Serializes `value` to YAML.
pub fn to_string<T: Serialize>(value: &T) -> Result<String, YamlError> {
    serde_saphyr::to_string(value).map_err(|error| YamlError::without_location(error.to_string()))
}
