//! Strictness and behaviour pins for the `sea_forge_core::yaml` boundary.
//!
//! Rejections are asserted as errors. Positive tests prove that omission, plain
//! booleans, canonical integers and aliases keep working. `open_issue_*` tests pin
//! the two holes that the spelling pass cannot see (serde `Content` buffers used
//! by `flatten` and internally tagged enums). See `src/yaml/strict.rs`.

// Several shape structs exist only to be deserialized; their fields are checked by
// the parser, not read by the test.
#![allow(dead_code)]

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sea_forge_core::yaml::{self, YamlError};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sample {
    enabled: bool,
    count: u64,
    name: String,
    mode: Mode,
    groups: BTreeMap<String, Vec<String>>,
    items: Vec<Item>,
    shape: Shape,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Alpha,
    Beta,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Item {
    id: String,
    weight: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Shape {
    Circle { radius: u32 },
    Square { side: u32 },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Flat {
    #[serde(flatten)]
    base: Base,
    extra: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Base {
    id: String,
    n: u32,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Nested {
    value: u32,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct Loose {
    list: Vec<String>,
    map: BTreeMap<String, String>,
    nested: Nested,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Named {
    s: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Num {
    n: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bools {
    b: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pair {
    a: String,
    b: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MapPair {
    first: BTreeMap<String, String>,
    second: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Text {
    literal: String,
    folded: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Signed {
    n: i64,
}

#[derive(Debug, Deserialize)]
struct RequiredColls {
    list: Vec<String>,
    map: BTreeMap<String, String>,
    nested: Nested,
}

#[derive(Debug, Default, PartialEq, Deserialize)]
struct FieldDefaults {
    #[serde(default)]
    list: Vec<String>,
    #[serde(default)]
    map: BTreeMap<String, String>,
    #[serde(default)]
    nested: Nested,
}

#[derive(Debug, PartialEq, Deserialize)]
struct OptU32 {
    v: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Top {
    a: L1,
}

#[derive(Debug, Deserialize)]
struct L1 {
    b: L2,
}

#[derive(Debug, Deserialize)]
struct L2 {
    c: L3,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct L3 {
    flag: bool,
    n: u64,
    list: Vec<String>,
    map: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Items {
    items: Vec<Bools>,
}

#[derive(Debug, Deserialize)]
struct Mapped {
    m: BTreeMap<String, Num>,
}

#[derive(Debug, Deserialize)]
struct IntKeys {
    keys: BTreeMap<u32, String>,
}

#[derive(Debug, Deserialize)]
struct FlatBool {
    #[serde(flatten)]
    base: BoolBase,
    extra: bool,
}

#[derive(Debug, Deserialize)]
struct BoolBase {
    flag: bool,
}

#[derive(Debug, Deserialize)]
struct FlatColl {
    #[serde(flatten)]
    base: CollBase,
    extra: bool,
}

#[derive(Debug, Deserialize)]
struct CollBase {
    list: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FlatNum {
    #[serde(flatten)]
    base: NumBase,
    extra: bool,
}

#[derive(Debug, Deserialize)]
struct NumBase {
    n: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Toggle {
    On { flag: bool },
}

#[derive(Debug, Deserialize)]
struct OptBool {
    b: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct BoolRow {
    b: bool,
}

#[derive(Debug, Deserialize)]
struct BoolRows {
    items: Vec<BoolRow>,
}

#[derive(Debug, Deserialize)]
struct BoolMap {
    m: BTreeMap<String, bool>,
}

/// Spellings that are not the exact words `true` or `false`. YAML 1.2 core accepts
/// some case variants as booleans, and YAML 1.1 words are plain strings here.
const NONCANONICAL_BOOLEANS: &[&str] = &[
    "True", "TRUE", "False", "FALSE", "tRue", "fAlse", "yes", "no", "on", "off", "y", "n",
    "\"true\"", "'false'", "1", "0", "~", "",
];

const VALID: &str = "\
enabled: true
count: 42
name: demo
mode: alpha
groups:
  readers: [a, b]
items:
  - id: one
    weight: 1
shape: {radius: 5, kind: circle}
";

fn valid_sample() -> Sample {
    Sample {
        enabled: true,
        count: 42,
        name: "demo".into(),
        mode: Mode::Alpha,
        groups: BTreeMap::from([("readers".to_string(), vec!["a".into(), "b".into()])]),
        items: vec![Item {
            id: "one".into(),
            weight: 1,
        }],
        shape: Shape::Circle { radius: 5 },
    }
}

fn mutate(from: &str, to: &str) -> String {
    assert!(VALID.contains(from), "fixture does not contain {from:?}");
    VALID.replacen(from, to, 1)
}

fn parse_sample(text: &str) -> Result<Sample, YamlError> {
    yaml::from_str(text)
}

fn assert_rejected_sample(text: &str) {
    assert!(
        parse_sample(text).is_err(),
        "expected rejection for:\n{text}"
    );
}

/// Runs `work` on a worker thread and fails if it does not finish in 5 seconds.
fn finishes_within_5s<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(work());
    });
    rx.recv_timeout(Duration::from_secs(5))
        .expect("parser did not finish within 5 seconds")
}

#[test]
fn valid_fixture_loads() {
    assert_eq!(parse_sample(VALID).unwrap(), valid_sample());
}

// ---- rejected ----

#[test]
fn rejects_duplicate_key_in_struct() {
    assert_rejected_sample(&format!("{VALID}enabled: false\n"));
}

#[test]
fn rejects_duplicate_key_in_map_field() {
    assert_rejected_sample(&mutate(
        "  readers: [a, b]\n",
        "  readers: [a, b]\n  readers: [c]\n",
    ));
}

#[test]
fn rejects_merge_key_that_would_inject_fields() {
    let text = "\
enabled: true
count: 42
name: demo
mode: alpha
groups: {}
items:
  - &base
    id: one
    weight: 1
  - <<: *base
shape: {radius: 5, kind: circle}
";
    let error = parse_sample(text).expect_err("merge key must be rejected");
    assert!(
        error.to_string().contains("<<"),
        "error should name the merge key, got: {error}"
    );
}

#[test]
fn rejects_yaml_1_1_boolean_words() {
    for word in ["yes", "no", "on", "off", "y", "n"] {
        assert_rejected_sample(&mutate("enabled: true", &format!("enabled: {word}")));
    }
}

#[test]
fn rejects_unknown_field() {
    assert_rejected_sample(&format!("{VALID}extra: 1\n"));
}

#[test]
fn rejects_missing_required_field() {
    assert_rejected_sample(&mutate("name: demo\n", ""));
}

#[test]
fn rejects_wrong_case_enum_variant() {
    assert_rejected_sample(&mutate("mode: alpha", "mode: Alpha"));
}

#[test]
fn rejects_unsupported_local_tag() {
    assert_rejected_sample(&mutate("name: demo", "name: !foo x"));
}

#[test]
fn rejects_binary_tag_for_integer_field() {
    let result: Result<Num, YamlError> = yaml::from_str("n: !!binary aGk=\n");
    assert!(result.is_err());
}

/// `!!binary` into a String is not decoded: `ignore_binary_tag_for_string` keeps
/// the literal scalar text, so the base64 payload never becomes "hi".
#[test]
fn rejects_binary_tag_decoding_into_string() {
    let named: Named = yaml::from_str("s: !!binary aGk=\n").unwrap();
    assert_eq!(named.s, "aGk=");
    assert_ne!(named.s, "hi");
}

#[test]
fn rejects_second_document_in_stream() {
    assert_rejected_sample(&format!("{VALID}---\nenabled: false\n"));
}

#[test]
fn rejects_byte_order_mark_prefix() {
    let text = format!("\u{feff}{VALID}");
    let error = parse_sample(&text).expect_err("BOM must be rejected");
    assert!(
        error.to_string().contains("byte order mark"),
        "got: {error}"
    );
    assert!(yaml::check_syntax(&text).is_err());
}

#[test]
fn rejects_trailing_garbage_after_document() {
    assert_rejected_sample(&format!("{VALID}]]]\n"));
}

#[test]
fn rejects_tab_indentation() {
    assert_rejected_sample(&mutate("  readers: [a, b]", "\treaders: [a, b]"));
}

#[test]
fn rejects_unterminated_quote() {
    assert_rejected_sample(&mutate("name: demo", "name: \"demo"));
}

#[test]
fn rejects_alias_bomb_within_bounded_time() {
    // Six levels of eight references each: 8^6 expanded elements if aliases were unbounded.
    let mut text = String::from("l0: &l0 [x, x, x, x, x, x, x, x]\n");
    for level in 1..6 {
        let refs = vec![format!("*l{}", level - 1); 8].join(", ");
        text.push_str(&format!("l{level}: &l{level} [{refs}]\n"));
    }
    let is_err =
        finishes_within_5s(move || yaml::from_str::<BTreeMap<String, IgnoredAny>>(&text).is_err());
    assert!(is_err, "alias bomb must be an error");
}

#[test]
fn deep_nesting_completes_without_stack_overflow() {
    let text = format!("{}{}", "[".repeat(300), "]".repeat(300));
    let (tx, rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let _ = tx.send(yaml::from_str::<IgnoredAny>(&text).is_ok());
        })
        .unwrap();
    let outcome = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("300-level nesting did not finish within 5 seconds");
    handle.join().expect("parser thread panicked");
    // Observed: the parser returns an error for 300 nested flow sequences.
    assert!(!outcome);
}

#[test]
fn string_field_accepts_plain_number_and_boolean_scalars() {
    let number: Named = yaml::from_str("s: 123\n").unwrap();
    assert_eq!(number.s, "123");
    let boolean: Named = yaml::from_str("s: true\n").unwrap();
    assert_eq!(boolean.s, "true");
}

#[test]
fn string_field_rejects_null_scalars() {
    assert!(yaml::from_str::<Named>("s: null\n").is_err());
    assert!(yaml::from_str::<Named>("s: ~\n").is_err());
    assert!(yaml::from_str::<Named>("s:\n").is_err());
}

// ---- accepted and equal ----

#[test]
fn accepts_plain_alias_of_scalar() {
    let pair: Pair = yaml::from_str("a: &x demo\nb: *x\n").unwrap();
    assert_eq!((pair.a.as_str(), pair.b.as_str()), ("demo", "demo"));
}

#[test]
fn accepts_plain_alias_of_mapping() {
    let pair: MapPair = yaml::from_str("first: &m {k: v}\nsecond: *m\n").unwrap();
    assert_eq!(pair.first, pair.second);
    assert_eq!(pair.second.get("k").map(String::as_str), Some("v"));
}

#[test]
fn accepts_block_literal_and_folded_scalars() {
    let text: Text =
        yaml::from_str("literal: |\n  line one\n  line two\nfolded: >\n  folded\n  text\n")
            .unwrap();
    assert_eq!(text.literal, "line one\nline two\n");
    assert_eq!(text.folded, "folded text\n");
}

#[test]
fn accepts_quoted_strings_with_escapes() {
    let double: Named = yaml::from_str("s: \"a\\tb\\\"c\\\\d\"\n").unwrap();
    assert_eq!(double.s, "a\tb\"c\\d");
    let single: Named = yaml::from_str("s: 'it''s'\n").unwrap();
    assert_eq!(single.s, "it's");
}

#[test]
fn accepts_true_and_false_booleans() {
    let t: Bools = yaml::from_str("b: true\n").unwrap();
    let f: Bools = yaml::from_str("b: false\n").unwrap();
    assert!(t.b);
    assert!(!f.b);
}

#[test]
fn accepts_integers_including_u64_max() {
    let max: Num = yaml::from_str("n: 18446744073709551615\n").unwrap();
    assert_eq!(max.n, u64::MAX);
    let zero: Num = yaml::from_str("n: 0\n").unwrap();
    assert_eq!(zero.n, 0);
}

#[test]
fn accepts_internally_tagged_enum_with_kind_not_first() {
    let sample = parse_sample(VALID).unwrap();
    assert_eq!(sample.shape, Shape::Circle { radius: 5 });
}

#[test]
fn accepts_flatten_struct() {
    let flat: Flat = yaml::from_str("id: a\nn: 3\nextra: true\n").unwrap();
    assert_eq!(
        flat,
        Flat {
            base: Base {
                id: "a".into(),
                n: 3
            },
            extra: true
        }
    );
}

// ---- rejected: scalar and collection types decided by the target type ----

/// serde_yaml 0.9 rejected this: a quoted boolean.
#[test]
fn rejects_quoted_false_for_bool() {
    assert!(yaml::from_str::<Bools>("b: \"false\"\n").is_err());
}

/// serde_yaml 0.9 rejected this: a quoted boolean.
#[test]
fn rejects_quoted_true_for_bool() {
    assert!(yaml::from_str::<Bools>("b: \"true\"\n").is_err());
}

/// Null, `~` or an empty value is not a collection, with or without a default.
#[test]
fn rejects_null_collections_with_default() {
    for text in [
        "list: ~\n",
        "map: ~\n",
        "nested: ~\n",
        "list: null\n",
        "list:\n",
        "map:\n",
        "nested:\n",
    ] {
        assert!(yaml::from_str::<Loose>(text).is_err(), "for {text:?}");
        assert!(
            yaml::from_str::<FieldDefaults>(text).is_err(),
            "for {text:?}"
        );
    }
}

#[test]
fn rejects_null_collections_without_default() {
    for text in [
        "list: ~\nmap: {}\nnested: {value: 1}\n",
        "list: []\nmap: ~\nnested: {value: 1}\n",
        "list: []\nmap: {}\nnested: ~\n",
        "list: []\nmap: {}\nnested:\n",
    ] {
        assert!(
            yaml::from_str::<RequiredColls>(text).is_err(),
            "for {text:?}"
        );
    }
}

/// serde_yaml 0.9 rejected this: a quoted integer.
#[test]
fn rejects_quoted_integer_for_u64() {
    assert!(yaml::from_str::<Num>("n: \"5\"\n").is_err());
}

/// serde_yaml 0.9 rejected this: an underscore-separated integer.
#[test]
fn rejects_underscore_integer_for_u64() {
    assert!(yaml::from_str::<Num>("n: 1_000\n").is_err());
}

/// serde_yaml 0.9 rejected this: a float literal for an integer field.
#[test]
fn rejects_float_for_u64() {
    assert!(yaml::from_str::<Num>("n: 1.0\n").is_err());
}

#[test]
fn rejects_non_canonical_integer_spellings_for_u64() {
    for text in ["+5", "010", "0x10", "0o10", "1e3", "-0", "-1"] {
        assert!(
            yaml::from_str::<Num>(&format!("n: {text}\n")).is_err(),
            "for {text:?}"
        );
    }
}

#[test]
fn rejects_non_canonical_integer_spellings_for_i64() {
    for text in [
        "\"5\"", "1_000", "+5", "010", "0x10", "0o10", "1e3", "-0", "1.0",
    ] {
        assert!(
            yaml::from_str::<Signed>(&format!("n: {text}\n")).is_err(),
            "for {text:?}"
        );
    }
}

#[test]
fn rejects_non_finite_floats_for_integer_field() {
    assert!(yaml::from_str::<Num>("n: .inf\n").is_err());
    assert!(yaml::from_str::<Num>("n: .nan\n").is_err());
}

#[test]
fn rejects_integer_keys_that_are_not_canonical() {
    assert!(yaml::from_str::<IntKeys>("keys:\n  \"5\": a\n").is_err());
    assert!(yaml::from_str::<IntKeys>("keys:\n  010: a\n").is_err());
    assert!(yaml::from_str::<IntKeys>("keys:\n  1_000: a\n").is_err());
    assert!(yaml::from_str::<IntKeys>("keys:\n  5: a\n").is_ok());
}

/// `!!binary` is never accepted as an integer.
#[test]
fn rejects_binary_tag_for_signed_integer_field() {
    assert!(yaml::from_str::<Signed>("n: !!binary aGk=\n").is_err());
}

// ---- rejected at depth: the same rules two and three levels down ----

#[test]
fn rejects_quoted_bool_two_levels_down() {
    let text = "b:\n  c:\n    flag: \"false\"\n    n: 1\n    list: []\n    map: {}\n";
    assert!(yaml::from_str::<L1>(text).is_err());
}

#[test]
fn rejects_quoted_bool_three_levels_down() {
    let text =
        "a:\n  b:\n    c:\n      flag: \"true\"\n      n: 1\n      list: []\n      map: {}\n";
    assert!(yaml::from_str::<Top>(text).is_err());
}

#[test]
fn rejects_null_collection_and_quoted_integer_three_levels_down() {
    let text = "a:\n  b:\n    c:\n      flag: true\n      n: 1\n      list: ~\n      map: {}\n";
    assert!(yaml::from_str::<Top>(text).is_err());
    let text =
        "a:\n  b:\n    c:\n      flag: true\n      n: \"5\"\n      list: []\n      map: {}\n";
    assert!(yaml::from_str::<Top>(text).is_err());
}

#[test]
fn rejects_rules_inside_vec_of_structs() {
    assert!(yaml::from_str::<Items>("items:\n  - b: \"false\"\n").is_err());
    assert!(yaml::from_str::<Items>("items:\n  - b: ~\n").is_err());
    assert!(yaml::from_str::<Items>("items:\n  - b: true\n").is_ok());
}

#[test]
fn rejects_rules_inside_map_value() {
    assert!(yaml::from_str::<Mapped>("m:\n  x:\n    n: \"5\"\n").is_err());
    assert!(yaml::from_str::<Mapped>("m:\n  x:\n    n: 1_000\n").is_err());
    assert!(yaml::from_str::<Mapped>("m:\n  x:\n    n: 5\n").is_ok());
}

#[test]
fn rejects_quoted_bool_inside_flatten_struct() {
    assert!(yaml::from_str::<FlatBool>("flag: \"false\"\nextra: true\n").is_err());
    assert!(yaml::from_str::<FlatBool>("flag: true\nextra: \"true\"\n").is_err());
    assert!(yaml::from_str::<FlatBool>("flag: true\nextra: true\n").is_ok());
}

#[test]
fn rejects_null_collection_inside_flatten_struct() {
    assert!(yaml::from_str::<FlatColl>("list: ~\nextra: true\n").is_err());
    assert!(yaml::from_str::<FlatColl>("list:\nextra: true\n").is_err());
}

#[test]
fn rejects_quoted_integer_inside_flatten_struct() {
    assert!(yaml::from_str::<FlatNum>("n: \"5\"\nextra: true\n").is_err());
}

#[test]
fn rejects_rules_inside_internally_tagged_enum() {
    assert!(yaml::from_str::<Toggle>("kind: on\nflag: \"false\"\n").is_err());
    assert!(yaml::from_str::<Toggle>("kind: on\nflag: ~\n").is_err());
    assert!(yaml::from_str::<Toggle>("kind: on\nflag: true\n").is_ok());
    assert!(yaml::from_str::<Shape>("kind: circle\nradius: \"5\"\n").is_err());
    assert!(yaml::from_str::<Shape>("kind: circle\nradius: ~\n").is_err());
}

// ---- open issues: spelling is not visible through serde's Content buffer ----

/// Open issue: under `flatten` the integer is buffered as `Content::U64`, and the
/// spelling pass only sees `deserialize_any`, so `1_000` is accepted. Quoted
/// integers are still rejected by serde's `Content` deserializer.
#[test]
fn open_issue_noncanonical_integer_in_flatten_is_accepted() {
    let value: FlatNum = yaml::from_str("n: 1_000\nextra: true\n").unwrap();
    assert_eq!(value.base.n, 1000);
}

/// Open issue: same as above for an internally tagged enum.
#[test]
fn open_issue_noncanonical_integer_in_tagged_enum_is_accepted() {
    let value: Shape = yaml::from_str("kind: circle\nradius: 1_000\n").unwrap();
    assert_eq!(value, Shape::Circle { radius: 1000 });
    let hex: Shape = yaml::from_str("kind: circle\nradius: 0x10\n").unwrap();
    assert_eq!(hex, Shape::Circle { radius: 16 });
}

// ---- accepted: omission, canonical spellings and aliases keep working ----

#[test]
fn omitted_default_fields_load_as_default() {
    let defaults: FieldDefaults = yaml::from_str("{}\n").unwrap();
    assert_eq!(defaults, FieldDefaults::default());
    let loose: Loose = yaml::from_str("{}\n").unwrap();
    assert_eq!(loose, Loose::default());
}

#[test]
fn option_field_omitted_or_null_is_none() {
    let omitted: OptU32 = yaml::from_str("{}\n").unwrap();
    assert_eq!(omitted.v, None);
    let explicit: OptU32 = yaml::from_str("v: ~\n").unwrap();
    assert_eq!(explicit.v, None);
    let explicit_null: OptU32 = yaml::from_str("v: null\n").unwrap();
    assert_eq!(explicit_null.v, None);
    let set: OptU32 = yaml::from_str("v: 7\n").unwrap();
    assert_eq!(set.v, Some(7));
}

#[test]
fn accepts_canonical_integers() {
    let zero: Num = yaml::from_str("n: 0\n").unwrap();
    assert_eq!(zero.n, 0);
    let max: Num = yaml::from_str("n: 18446744073709551615\n").unwrap();
    assert_eq!(max.n, u64::MAX);
    let negative: Signed = yaml::from_str("n: -5\n").unwrap();
    assert_eq!(negative.n, -5);
    let min: Signed = yaml::from_str("n: -9223372036854775808\n").unwrap();
    assert_eq!(min.n, i64::MIN);
    let max_signed: Signed = yaml::from_str("n: 9223372036854775807\n").unwrap();
    assert_eq!(max_signed.n, i64::MAX);
}

#[test]
fn accepts_plain_false_and_lists_in_nested_structs() {
    let text =
        "a:\n  b:\n    c:\n      flag: false\n      n: 1\n      list: [x]\n      map: {k: v}\n";
    let top: Top = yaml::from_str(text).unwrap();
    assert!(!top.a.b.c.flag);
    assert_eq!(top.a.b.c.list, vec!["x".to_string()]);
}

#[test]
fn accepts_plain_alias_of_scalar_in_nested_struct() {
    let pair: Pair = yaml::from_str("a: &x demo\nb: *x\n").unwrap();
    assert_eq!(pair.b, "demo");
}

/// `!!binary` never reaches an integer, and never decodes into a String.
#[test]
fn binary_tag_never_decodes_into_string_or_integer() {
    let named: Named = yaml::from_str("s: !!binary aGk=\n").unwrap();
    assert_ne!(named.s, "hi");
    assert!(yaml::from_str::<Num>("n: !!binary AA==\n").is_err());
}

// ---- round trip and syntax checks ----

#[test]
fn round_trip_preserves_every_field_kind() {
    let sample = valid_sample();
    let text = yaml::to_string(&sample).unwrap();
    let back: Sample = yaml::from_str(&text).unwrap();
    assert_eq!(back, sample);

    let flat = Flat {
        base: Base {
            id: "a".into(),
            n: 3,
        },
        extra: true,
    };
    let text = yaml::to_string(&flat).unwrap();
    let back: Flat = yaml::from_str(&text).unwrap();
    assert_eq!(back, flat);
}

/// Case variants that serde's `Content` buffer turns into booleans (see the open
/// issues below), with the value each one produces.
const CASE_VARIANT_BOOLEANS: &[(&str, bool)] = &[
    ("True", true),
    ("TRUE", true),
    ("tRue", true),
    ("False", false),
    ("FALSE", false),
    ("fAlse", false),
];

/// Under `flatten` and internally tagged enums, the words that are not booleans
/// are still rejected: the `Content` buffer only turns case variants into booleans.
#[test]
fn rejects_noncanonical_boolean_words_in_flatten_and_tagged_enum() {
    for spelling in NONCANONICAL_BOOLEANS {
        if CASE_VARIANT_BOOLEANS.iter().any(|(s, _)| s == spelling) {
            continue;
        }
        assert!(
            yaml::from_str::<FlatBool>(&format!("flag: {spelling}\nextra: true\n")).is_err(),
            "flatten accepted {spelling:?}"
        );
        assert!(
            yaml::from_str::<Toggle>(&format!("kind: on\nflag: {spelling}\n")).is_err(),
            "tagged enum accepted {spelling:?}"
        );
    }
}

/// Open issue: under `flatten` the boolean is buffered as `Content::Bool`, and the
/// spelling pass only sees `deserialize_any`, so `True`, `TRUE`, `tRue`, `False`,
/// `FALSE` and `fAlse` are accepted with their boolean value.
#[test]
fn open_issue_case_variant_boolean_in_flatten_is_accepted() {
    for (spelling, expected) in CASE_VARIANT_BOOLEANS {
        let value: FlatBool = yaml::from_str(&format!("flag: {spelling}\nextra: true\n")).unwrap();
        assert_eq!(value.base.flag, *expected, "{spelling}");
    }
}

/// Open issue: same as above for an internally tagged enum.
#[test]
fn open_issue_case_variant_boolean_in_tagged_enum_is_accepted() {
    for (spelling, expected) in CASE_VARIANT_BOOLEANS {
        let value: Toggle = yaml::from_str(&format!("kind: on\nflag: {spelling}\n")).unwrap();
        assert_eq!(value, Toggle::On { flag: *expected }, "{spelling}");
    }
}

#[test]
fn rejects_noncanonical_boolean_spellings() {
    for spelling in NONCANONICAL_BOOLEANS {
        assert!(
            yaml::from_str::<Bools>(&format!("b: {spelling}\n")).is_err(),
            "plain bool accepted {spelling:?}"
        );
        assert!(
            yaml::from_str::<BoolRows>(&format!("items:\n  - b: {spelling}\n")).is_err(),
            "bool in Vec accepted {spelling:?}"
        );
        assert!(
            yaml::from_str::<BoolMap>(&format!("m:\n  x: {spelling}\n")).is_err(),
            "bool in map value accepted {spelling:?}"
        );
        if !matches!(*spelling, "~" | "") {
            assert!(
                yaml::from_str::<OptBool>(&format!("b: {spelling}\n")).is_err(),
                "Option<bool> accepted {spelling:?}"
            );
        }
    }
}

/// `~`, an empty value and an omitted key are all null: `Option<bool>` is `None`.
#[test]
fn option_bool_null_and_omitted_are_none() {
    for text in ["b: ~\n", "b:\n", "other: 1\n"] {
        let value: OptBool = yaml::from_str(text).unwrap();
        assert!(value.b.is_none(), "{text:?}");
    }
    let some: OptBool = yaml::from_str("b: true\n").unwrap();
    assert_eq!(some.b, Some(true));
    let none: OptBool = yaml::from_str("b: false\n").unwrap();
    assert_eq!(none.b, Some(false));
}

#[test]
fn check_syntax_accepts_what_from_str_accepts() {
    assert!(yaml::check_syntax(VALID).is_ok());
    assert!(parse_sample(VALID).is_ok());
}

#[test]
fn check_syntax_reports_location_for_syntax_error() {
    let error = yaml::check_syntax("a: [\n").expect_err("unclosed bracket");
    assert_eq!(error.line(), Some(1));
    assert_eq!(error.column(), Some(4));
}

#[test]
fn check_syntax_rejects_duplicate_keys() {
    let error = yaml::check_syntax("a: 1\na: 2\n").expect_err("duplicate key");
    assert_eq!(error.line(), Some(2));
    assert_eq!(error.column(), Some(1));
}

#[test]
fn from_slice_rejects_invalid_utf8() {
    assert!(yaml::from_slice::<IgnoredAny>(&[0x61, 0x3a, 0xff]).is_err());
}
