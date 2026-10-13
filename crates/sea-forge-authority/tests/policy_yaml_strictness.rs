//! Policy loading under the strict YAML boundary.
//!
//! Each rejected case asserts the error class `AuthorityPolicyBundle::load`
//! reports: `parse_error` for syntax and duplicate-key failures found while
//! checking the document, `schema_error` for failures while building the bundle.

use sea_forge_authority::{AuthorityEvaluation, AuthorityPolicyBundle, PolicyAuthorityEngine};
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::{Actor, ActorRole, AuthorityAction, Verdict};
use std::path::{Path, PathBuf};

const BUNDLE: &str = r#"version: "0.1"
default: deny
rules:
  - name: allow-recall
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
"#;

fn write_policy(dir: &Path, text: &str) -> PathBuf {
    let path = dir.join("policy.yaml");
    std::fs::write(&path, text).unwrap();
    path
}

fn load(text: &str) -> Result<AuthorityPolicyBundle, ForgeError> {
    let dir = tempfile::tempdir().unwrap();
    let path = write_policy(dir.path(), text);
    AuthorityPolicyBundle::load(&path)
}

fn load_error_class(text: &str) -> &'static str {
    match load(text) {
        Ok(_) => panic!("expected the policy to be rejected:\n{text}"),
        Err(error) => error.class(),
    }
}

fn load_error_message(text: &str) -> String {
    match load(text) {
        Ok(_) => panic!("expected the policy to be rejected:\n{text}"),
        Err(error) => error.to_string(),
    }
}

fn replace_once(from: &str, to: &str) -> String {
    assert!(BUNDLE.contains(from), "fixture does not contain {from:?}");
    BUNDLE.replacen(from, to, 1)
}

#[test]
fn unmodified_bundle_loads() {
    let bundle = load(BUNDLE).expect("valid bundle must load");
    assert_eq!(bundle.rules.len(), 1);
    assert_eq!(bundle.rules[0].name, "allow-recall");
}

/// Main loaded this as allow: serde_yaml kept the last `default` key.
#[test]
fn duplicate_default_key_is_rejected() {
    let text = format!("{BUNDLE}default: allow\n");
    assert_eq!(load_error_class(&text), "parse_error");
}

#[test]
fn duplicate_rule_verdict_is_rejected() {
    let text = replace_once(
        "    verdict: allow\n",
        "    verdict: allow\n    verdict: deny\n",
    );
    assert_eq!(load_error_class(&text), "parse_error");
}

#[test]
fn duplicate_boundary_constraint_is_rejected() {
    let text = replace_once(
        "    memory_scope: own\n",
        "    boundary_constraints:\n      workspace: [/safe]\n      workspace: [/other]\n",
    );
    assert_eq!(load_error_class(&text), "parse_error");
}

#[test]
fn merge_key_from_allow_rule_is_rejected_and_adds_no_rule() {
    let text = r#"version: "0.1"
default: deny
rules:
  - &r1
    name: allow-recall
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
  - <<: *r1
    name: copied-allow
"#;
    assert_eq!(load_error_class(text), "schema_error");
}

#[test]
fn yaml_1_1_boolean_words_for_requires_approval_are_rejected() {
    for word in ["no", "off", "yes"] {
        let text = replace_once(
            "    memory_scope: own\n",
            &format!("    memory_scope: own\n    requires_approval: {word}\n"),
        );
        assert_eq!(load_error_class(&text), "schema_error", "word {word}");
    }
}

#[test]
fn unknown_field_in_rule_is_schema_error() {
    let text = replace_once(
        "    memory_scope: own\n",
        "    memory_scope: own\n    surprise: 1\n",
    );
    assert_eq!(load_error_class(&text), "schema_error");
}

#[test]
fn unknown_top_level_field_is_schema_error() {
    let text = format!("{BUNDLE}surprise: 1\n");
    assert_eq!(load_error_class(&text), "schema_error");
}

#[test]
fn syntax_error_is_parse_error_with_line_and_column() {
    let text = replace_once("    verdict: allow\n", "    verdict: [allow\n");
    assert_eq!(load_error_class(&text), "parse_error");
    let message = load_error_message(&text);
    let tail = message
        .split("invalid YAML at line ")
        .nth(1)
        .unwrap_or_else(|| panic!("unexpected message: {message}"));
    let (line, rest) = tail.split_once(", column ").expect("column in message");
    assert!(line.parse::<usize>().is_ok(), "line not numeric: {message}");
    assert!(
        rest.chars().next().is_some_and(|c| c.is_ascii_digit()),
        "column not numeric: {message}"
    );
}

#[test]
fn tagged_verdict_is_rejected() {
    let text = replace_once("    verdict: allow\n", "    verdict: !allow x\n");
    assert!(load(&text).is_err());
}

#[test]
fn second_yaml_document_is_rejected() {
    let text = format!("{BUNDLE}---\nversion: \"0.1\"\n");
    assert!(load(&text).is_err());
}

/// Main loaded `requires_approval: "false"` as a bool. The strict boundary rejects it.
#[test]
fn quoted_boolean_is_rejected() {
    let cases = [
        (
            "requires_approval",
            replace_once(
                "    memory_scope: own\n",
                "    memory_scope: own\n    requires_approval: \"false\"\n",
            ),
        ),
        (
            "allow_degraded",
            format!("{BUNDLE}allow_degraded: \"false\"\n"),
        ),
        (
            "identity.allow_unresolved",
            format!("{BUNDLE}identity:\n  allow_unresolved: \"false\"\n"),
        ),
        (
            "integrity_ledger.required_for_side_effects",
            format!("{BUNDLE}integrity_ledger:\n  required_for_side_effects: \"false\"\n"),
        ),
        (
            "sod_rules[].allow_same_principal",
            format!("{BUNDLE}{}", sod_rule("allow_same_principal: \"false\"")),
        ),
        (
            "policy_engines[].required",
            format!("{BUNDLE}{}", policy_engine("required: \"false\"")),
        ),
        (
            "settlement_authorities[].may_issue_strong",
            format!(
                "{BUNDLE}{}",
                settlement_authority("may_issue_strong: \"false\"")
            ),
        ),
        (
            "policy_surfaces.self_disclosure.grants[].explicit_high_risk",
            format!(
                "{BUNDLE}{}",
                self_disclosure_grant("explicit_high_risk: \"false\"")
            ),
        ),
        (
            "policy_surfaces.self_disclosure.grants[].require_fresh_snapshot",
            format!(
                "{BUNDLE}{}",
                self_disclosure_grant("require_fresh_snapshot: \"true\"")
            ),
        ),
    ];
    for (field, text) in cases {
        assert!(
            load(&text).is_err(),
            "quoted bool for {field} must be rejected"
        );
    }
}

/// Bool fields in the nested structs take their canonical spelling and load.
/// This keeps the quoted-bool rejections above from passing on a broken fixture.
#[test]
fn canonical_bool_fields_in_nested_structs_load() {
    for text in [
        format!("{BUNDLE}{}", sod_rule("allow_same_principal: true")),
        format!("{BUNDLE}{}", policy_engine("required: true")),
        format!(
            "{BUNDLE}{}",
            settlement_authority("may_issue_strong: false")
        ),
        format!(
            "{BUNDLE}{}",
            self_disclosure_grant("explicit_high_risk: false")
        ),
    ] {
        if let Err(error) = sea_forge_core::yaml::from_str::<AuthorityPolicyBundle>(&text) {
            panic!("canonical fixture must load: {error}\n{text}");
        }
    }
}

fn sod_rule(extra: &str) -> String {
    format!(
        "sod_rules:\n  - name: sod\n    requester_role: operator\n    approver_role: operator\n    {extra}\n"
    )
}

fn policy_engine(extra: &str) -> String {
    format!("policy_engines:\n  - engine: eng\n    fail_mode: closed\n    {extra}\n")
}

fn settlement_authority(extra: &str) -> String {
    format!(
        "settlement_authorities:\n  - authority_ref: sa\n    adapter: local\n    trust_anchor_ref: t\n    declarer_actor_id: d\n    permitted_declarer_roles: [operator]\n    standing_basis: s\n    {extra}\n"
    )
}

fn self_disclosure_grant(extra: &str) -> String {
    format!(
        "policy_surfaces:\n  self_disclosure:\n    grants:\n      - name: g\n        actor_role: operator\n        claim_classes: []\n        {extra}\n"
    )
}

/// `~` or an empty value for a collection is an error, not an empty collection.
/// Omitting an optional collection behaves as an explicit empty one.
#[test]
fn null_deny_list_is_rejected() {
    let null_rules = "version: \"0.1\"\ndefault: deny\nrules: ~\n";
    assert!(load(null_rules).is_err(), "null rules must be rejected");
    let omitted = evaluate(load(BUNDLE).unwrap(), &deploy_action());
    for (key, empty) in [
        ("roles", "{}"),
        ("permissions", "[]"),
        ("sod_rules", "[]"),
        ("identity_bindings", "[]"),
        ("policy_engines", "[]"),
        ("settlement_authorities", "[]"),
    ] {
        assert!(load(&format!("{BUNDLE}{key}: ~\n")).is_err(), "{key}: ~");
        assert!(load(&format!("{BUNDLE}{key}:\n")).is_err(), "{key}: empty");
        let explicit_empty = evaluate(
            load(&format!("{BUNDLE}{key}: {empty}\n")).unwrap(),
            &deploy_action(),
        );
        assert_eq!(
            omitted, explicit_empty,
            "omitted {key} must behave as on main"
        );
    }
}

/// A `min_witnesses` written with a quoted, underscored, prefixed, float or
/// exponent spelling is rejected. The canonical spelling loads.
#[test]
fn noncanonical_integers_are_rejected() {
    for spelling in ["\"5\"", "1_000", "010", "0x10", "+5", "1.0", "1e3", "-0"] {
        let text = format!("{BUNDLE}integrity_ledger:\n  min_witnesses: {spelling}\n");
        assert!(
            load(&text).is_err(),
            "min_witnesses: {spelling} must be rejected"
        );
    }
    let canonical = format!("{BUNDLE}integrity_ledger:\n  min_witnesses: 1\n");
    assert!(
        load(&canonical).is_ok(),
        "canonical min_witnesses must load"
    );
}

/// Every integer field of the bundle, enumerated by `grep` over
/// `crates/sea-forge-authority/src/lib.rs` (audit 2026-10-12): `integrity_ledger.min_witnesses`
/// (usize) and `settlement_authorities[].timeout_secs` (u64). The runtime-only
/// integer fields (`AuthorityEvaluation.sequence`/`timeout_secs`, `ExecutionGrantContext`,
/// `ActionGrant.timeout_secs`) are not read from YAML. No integer in the bundle sits
/// under `#[serde(flatten)]` or an internally tagged enum, so the buffered-integer
/// gap in `yaml.rs` does not reach the policy types.
#[test]
fn noncanonical_settlement_timeout_is_rejected() {
    for spelling in ["\"60\"", "1_000", "010", "0x10", "+60", "60.0", "6e1", "-0"] {
        let text = format!(
            "{BUNDLE}{}",
            settlement_authority(&format!("timeout_secs: {spelling}"))
        );
        assert!(
            load(&text).is_err(),
            "settlement timeout_secs: {spelling} must be rejected"
        );
    }
    let canonical = format!("{BUNDLE}{}", settlement_authority("timeout_secs: 60"));
    if let Err(error) = sea_forge_core::yaml::from_str::<AuthorityPolicyBundle>(&canonical) {
        panic!("canonical timeout_secs must load: {error}");
    }
}

/// A merge key cannot copy an allow rule into a rule meant to deny, or add
/// `requires_approval: false`. Merge keys are ordinary keys, so the load fails.
#[test]
fn merge_key_cannot_override_policy() {
    let merge_copies_allow = r#"version: "0.1"
default: deny
rules:
  - &r1
    name: allow-recall
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
  - <<: *r1
    name: intended-deny
    verdict: deny
"#;
    assert!(load(merge_copies_allow).is_err());
    let merge_adds_approval_false = r#"version: "0.1"
default: deny
rules:
  - &r1
    name: needs-approval
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
    requires_approval: true
  - <<: *r1
    name: copied
    requires_approval: false
"#;
    assert!(load(merge_adds_approval_false).is_err());
}

/// Repeating an allow rule, literally or through an alias, is rejected by the
/// unique-rule-name check, so an alias cannot add a second copy of a rule. A
/// recursive alias must fail.
#[test]
fn aliases_cannot_duplicate_or_widen_rules() {
    let literal = r#"version: "0.1"
default: deny
rules:
  - name: allow-recall
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
  - name: allow-recall
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
"#;
    let aliased = r#"version: "0.1"
default: deny
rules:
  - &r1
    name: allow-recall
    verdict: allow
    actor_role: operator
    operation_kind: recall_memory
    memory_scope: own
  - *r1
"#;
    assert!(
        load(literal).is_err(),
        "literal duplicate rule name must be rejected"
    );
    assert!(
        load(aliased).is_err(),
        "aliased duplicate rule name must be rejected"
    );
    let recursive = "version: \"0.1\"\ndefault: deny\nrules: &a [*a]\n";
    assert!(load(recursive).is_err(), "recursive alias must be rejected");
}

/// Tags: `!!str` and `!!binary` on a bool field cannot load. A local tag is
/// rejected. `!!bool "true"` keeps its literal value and is never read as false.
#[test]
fn unsupported_tags_are_rejected() {
    let str_tag = replace_once(
        "    memory_scope: own\n",
        "    memory_scope: own\n    requires_approval: !!str false\n",
    );
    assert!(load(&str_tag).is_err(), "!!str on a bool must be rejected");
    let local = replace_once("    verdict: allow\n", "    verdict: !allow x\n");
    assert!(load(&local).is_err());
    let binary = replace_once(
        "    memory_scope: own\n",
        "    memory_scope: own\n    requires_approval: !!binary aGk=\n",
    );
    assert!(
        load(&binary).is_err(),
        "!!binary on a bool must be rejected"
    );
    let explicit_true = replace_once(
        "    memory_scope: own\n",
        "    memory_scope: own\n    requires_approval: !!bool \"true\"\n",
    );
    if let Ok(bundle) = load(&explicit_true) {
        assert_ne!(bundle.rules[0].requires_approval, Some(false));
    }
}

/// A write that a rule allows only after approval, with the SOD rule that names the
/// approver. `requires` and `approver` are whole YAML lines (or empty).
fn approval_policy(requires: &str, approver: &str) -> String {
    format!(
        "version: \"0.1\"\ndefault: deny\nrules:\n  - name: gated-write\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"reviewed\"\n{requires}sod_rules:\n  - name: write-sod\n    requester_role: operator\n{approver}    operation_kind: write_file\n"
    )
}

fn write_action(path: &str) -> AuthorityAction {
    AuthorityAction::WriteFile {
        path: path.into(),
        content_hint: "fixed".into(),
    }
}

/// Verdict, and whether the non-approval `grant` path mints a grant, for one
/// evaluation of a loaded bundle.
struct Outcome {
    verdict: Verdict,
    grant_ok: bool,
}

fn evaluate_and_grant(bundle: AuthorityPolicyBundle, action: &AuthorityAction) -> Outcome {
    let binding = bundle.resolve_identity("operator_local", ActorRole::Operator);
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    let decision = engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_approvers",
            case_id: "case_approvers",
            plan_item_id: "item_approvers",
            sequence: 1,
            action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec!["intent:approvers".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    let stream =
        sea_forge_ledger::LedgerStream::open(root.path(), "policy-yaml-strictness", "test")
            .unwrap();
    let committed = stream
        .commit_typed("authority_decision", vec![], &decision, vec![])
        .unwrap();
    let grant_ok = engine.grant(&decision, &committed, action, None).is_ok();
    Outcome {
        verdict: decision.verdict,
        grant_ok,
    }
}

/// `requires_approval` and the approver role are the approval gate. A malformed
/// value either fails the load or leaves the write escalated, never allowed.
#[test]
fn missing_required_approvers_is_rejected_or_never_allows() {
    const REQUIRES: &str = "    requires_approval: true\n";
    const APPROVER: &str = "    approver_role: reviewer\n";
    let action = write_action("reviewed/doc.md");

    // The SOD rule names `write_file`, so every such write escalates to approval
    // whatever `requires_approval` says. The pins record that observed behaviour.
    let omitted = evaluate_and_grant(load(&approval_policy("", APPROVER)).unwrap(), &action);
    assert_eq!(
        omitted.verdict,
        Verdict::Escalate,
        "requires_approval omitted"
    );
    assert!(
        !omitted.grant_ok,
        "omitted: grant must not mint without approval"
    );
    let explicit_false = evaluate_and_grant(
        load(&approval_policy("    requires_approval: false\n", APPROVER)).unwrap(),
        &action,
    );
    assert_eq!(
        explicit_false.verdict,
        Verdict::Escalate,
        "requires_approval: false"
    );
    assert!(
        !explicit_false.grant_ok,
        "false: grant must not mint without approval"
    );
    let explicit_true =
        evaluate_and_grant(load(&approval_policy(REQUIRES, APPROVER)).unwrap(), &action);
    assert_eq!(
        explicit_true.verdict,
        Verdict::Escalate,
        "requires_approval: true"
    );
    assert!(
        !explicit_true.grant_ok,
        "grant must not mint without approval"
    );

    let mut cases: Vec<(String, String)> = Vec::new();
    for (name, approver) in [
        ("omitted", ""),
        ("null", "    approver_role: ~\n"),
        ("empty", "    approver_role: \"\"\n"),
    ] {
        cases.push((
            format!("approver_role {name}"),
            approval_policy(REQUIRES, approver),
        ));
    }
    for (name, requires) in [
        ("quoted \"true\"", "    requires_approval: \"true\"\n"),
        ("True", "    requires_approval: True\n"),
        ("yes", "    requires_approval: yes\n"),
        ("null", "    requires_approval: ~\n"),
    ] {
        cases.push((
            format!("requires_approval {name}"),
            approval_policy(requires, APPROVER),
        ));
    }
    for (label, text) in cases {
        if let Ok(bundle) = load(&text) {
            let outcome = evaluate_and_grant(bundle, &action);
            assert!(
                matches!(outcome.verdict, Verdict::Escalate | Verdict::Deny),
                "{label}: loaded and the write is {:?}",
                outcome.verdict
            );
            assert!(!outcome.grant_ok, "{label}: grant minted without approval");
        }
    }
}

/// A base bundle with three probes: P1 is denied by default, P2 needs approval,
/// P3 is allowed, and P4 is an explicit deny rule.
const MUTATION_BASE: &str = r#"version: "0.1"
default: deny
rules:
  - name: deny-secret
    verdict: deny
    actor_role: operator
    operation_kind: write_file
    path_prefix: "secret"
  - name: allow-notes
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: "notes"
  - name: gated-write
    verdict: allow
    actor_role: operator
    operation_kind: write_file
    path_prefix: "reviewed"
    requires_approval: true
  - name: allow-deployment
    verdict: allow
    actor_role: operator
    operation_kind: deployment
sod_rules:
  - name: write-sod
    requester_role: operator
    approver_role: reviewer
    operation_kind: write_file
integrity_ledger:
  min_witnesses: 1
"#;

/// Applies each replacement once, in order. A replacement that does not match the
/// current text is a bug in the table, so it panics rather than testing nothing.
fn subst(pairs: &[(&str, &str)]) -> String {
    let mut text = MUTATION_BASE.to_string();
    for (from, to) in pairs {
        assert!(text.contains(from), "mutation base lacks {from:?}");
        text = text.replacen(from, to, 1);
    }
    text
}

/// Malformed or ambiguous spellings of the base bundle. None of them may make P1
/// (default deny), P2 (escalate) or P4 (explicit deny) allowed.
#[test]
fn malformed_input_never_turns_deny_or_escalate_into_allow() {
    let p1 = write_action("other/x.md");
    let p2 = write_action("reviewed/doc.md");
    let p3 = AuthorityAction::Reserved {
        resource_type: "deployment".into(),
        resource_id: "prod".into(),
        parameters: serde_json::json!({}),
    };
    let p4 = write_action("secret/key");
    let base = load(MUTATION_BASE).expect("base bundle must load");
    assert_eq!(evaluate(base, &p1), Verdict::Deny, "base P1");
    let base = load(MUTATION_BASE).unwrap();
    assert_eq!(evaluate(base, &p2), Verdict::Escalate, "base P2");
    let base = load(MUTATION_BASE).unwrap();
    assert_eq!(
        evaluate(base, &p3),
        Verdict::Allow,
        "base P3 proves the bundle is live"
    );

    const SOD_BLOCK: &str = "sod_rules:\n  - name: write-sod\n    requester_role: operator\n    approver_role: reviewer\n    operation_kind: write_file\n";
    const REQUIRES: &str = "    requires_approval: true\n";
    let mutations: Vec<(&str, String)> = vec![
        ("duplicate default, allow then deny", subst(&[("default: deny\n", "default: deny\ndefault: allow\n")])),
        ("duplicate default, deny then allow", subst(&[("default: deny\n", "default: allow\ndefault: deny\n")])),
        ("default !!str allow", subst(&[("default: deny\n", "default: !!str allow\n")])),
        ("default quoted allow", subst(&[("default: deny\n", "default: \"allow\"\n")])),
        ("default null", subst(&[("default: deny\n", "default: ~\n")])),
        ("default uppercase ALLOW", subst(&[("default: deny\n", "default: ALLOW\n")])),
        ("deny rule tagged verdict !allow", subst(&[("    verdict: deny\n", "    verdict: !allow deny\n")])),
        ("deny rule duplicate verdict, allow last", subst(&[("    verdict: deny\n", "    verdict: deny\n    verdict: allow\n")])),
        ("deny rule duplicate verdict, allow first", subst(&[("    verdict: deny\n", "    verdict: allow\n    verdict: deny\n")])),
        ("deny rule flow-style duplicate verdict", subst(&[(
            "  - name: deny-secret\n    verdict: deny\n    actor_role: operator\n    operation_kind: write_file\n    path_prefix: \"secret\"\n",
            "  - {name: deny-secret, verdict: deny, verdict: allow, actor_role: operator, operation_kind: write_file, path_prefix: \"secret\"}\n",
        )])),
        ("duplicate requires_approval, true then false", subst(&[(REQUIRES, "    requires_approval: true\n    requires_approval: false\n")])),
        ("duplicate requires_approval, false then true", subst(&[(REQUIRES, "    requires_approval: false\n    requires_approval: true\n")])),
        ("merge key pulls allow rule into deny rule", subst(&[
            ("  - name: allow-notes\n", "  - &allow_rule\n    name: allow-notes\n"),
            ("  - name: deny-secret\n    verdict: deny\n", "  - name: deny-secret\n    <<: *allow_rule\n    verdict: deny\n"),
        ])),
        ("merge key pulls allow rule into gated rule", subst(&[
            ("  - name: allow-notes\n", "  - &allow_rule\n    name: allow-notes\n"),
            ("  - name: gated-write\n    verdict: allow\n", "  - name: gated-write\n    <<: *allow_rule\n    verdict: allow\n"),
        ])),
        ("alias replaces deny verdict with anchored allow", subst(&[
            ("  - name: allow-notes\n    verdict: allow\n", "  - name: allow-notes\n    verdict: &v allow\n"),
            ("    verdict: deny\n", "    verdict: *v\n"),
        ])),
        ("alias before anchor replaces deny verdict", subst(&[
            ("    verdict: deny\n", "    verdict: *v\n"),
            ("  - name: allow-notes\n    verdict: allow\n", "  - name: allow-notes\n    verdict: &v allow\n"),
        ])),
        ("rules null", "version: \"0.1\"\ndefault: deny\nrules: ~\n".to_string()),
        ("rules empty value", "version: \"0.1\"\ndefault: deny\nrules:\n".to_string()),
        ("sod_rules null", subst(&[(SOD_BLOCK, "sod_rules: ~\n")])),
        ("sod_rules empty value", subst(&[(SOD_BLOCK, "sod_rules:\n")])),
        ("boundary_constraints null on gated rule", subst(&[(REQUIRES, "    requires_approval: true\n    boundary_constraints: ~\n")])),
        ("boundary_constraints null on deny rule", subst(&[("    verdict: deny\n", "    verdict: deny\n    boundary_constraints: ~\n")])),
        ("requires_approval quoted \"true\"", subst(&[(REQUIRES, "    requires_approval: \"true\"\n")])),
        ("requires_approval single-quoted 'true'", subst(&[(REQUIRES, "    requires_approval: 'true'\n")])),
        ("requires_approval True", subst(&[(REQUIRES, "    requires_approval: True\n")])),
        ("requires_approval yes", subst(&[(REQUIRES, "    requires_approval: yes\n")])),
        ("requires_approval 0", subst(&[(REQUIRES, "    requires_approval: 0\n")])),
        ("requires_approval null", subst(&[(REQUIRES, "    requires_approval: ~\n")])),
        ("requires_approval empty value", subst(&[(REQUIRES, "    requires_approval:\n")])),
        ("requires_approval !!str true", subst(&[(REQUIRES, "    requires_approval: !!str true\n")])),
        ("allow_degraded quoted \"true\"", format!("{MUTATION_BASE}allow_degraded: \"true\"\n")),
        ("allow_degraded yes", format!("{MUTATION_BASE}allow_degraded: yes\n")),
        ("allow_degraded True", format!("{MUTATION_BASE}allow_degraded: True\n")),
        ("policy_engines null", format!("{MUTATION_BASE}policy_engines: ~\n")),
        ("second document, default allow", format!("{MUTATION_BASE}---\nversion: \"0.1\"\ndefault: allow\n")),
        ("second document, allow-all rule", format!("{MUTATION_BASE}---\nversion: \"0.1\"\ndefault: deny\nrules:\n  - name: any\n    verdict: allow\n    actor_role: operator\n    operation_kind: write_file\n")),
        ("document end, then permissive document", format!("{MUTATION_BASE}...\n---\nversion: \"0.1\"\ndefault: allow\n")),
        ("document end, then permissive bare key", format!("{MUTATION_BASE}...\ndefault: allow\n")),
        ("UTF-8 BOM prefix", format!("\u{feff}{MUTATION_BASE}")),
        ("tab indentation on deny verdict", subst(&[("    verdict: deny\n", "\tverdict: deny\n")])),
        ("root tag !foo", format!("!foo\n{MUTATION_BASE}")),
        ("root tag !!map", format!("!!map\n{MUTATION_BASE}")),
        ("min_witnesses quoted \"1\"", subst(&[("  min_witnesses: 1\n", "  min_witnesses: \"1\"\n")])),
        ("min_witnesses 1_000", subst(&[("  min_witnesses: 1\n", "  min_witnesses: 1_000\n")])),
        ("min_witnesses 01", subst(&[("  min_witnesses: 1\n", "  min_witnesses: 01\n")])),
        ("min_witnesses 0x1", subst(&[("  min_witnesses: 1\n", "  min_witnesses: 0x1\n")])),
        ("min_witnesses +1", subst(&[("  min_witnesses: 1\n", "  min_witnesses: +1\n")])),
        ("min_witnesses 1.0", subst(&[("  min_witnesses: 1\n", "  min_witnesses: 1.0\n")])),
        ("min_witnesses 1e0", subst(&[("  min_witnesses: 1\n", "  min_witnesses: 1e0\n")])),
        ("unknown field on gated rule", subst(&[(REQUIRES, "    requires_approval: true\n    approve_all: true\n")])),
        ("unknown top-level field", format!("{MUTATION_BASE}approval_bypass: true\n")),
        ("duplicate approver_role", subst(&[("    approver_role: reviewer\n", "    approver_role: reviewer\n    approver_role: operator\n")])),
        ("approver_role null", subst(&[("    approver_role: reviewer\n", "    approver_role: ~\n")])),
        ("approver_role omitted", subst(&[("    approver_role: reviewer\n", "")])),
        ("approver_role empty quoted", subst(&[("    approver_role: reviewer\n", "    approver_role: \"\"\n")])),
    ];

    let total = mutations.len();
    let mut rejected = Vec::new();
    let mut loaded = Vec::new();
    let mut failures = Vec::new();
    for (label, text) in &mutations {
        match load(text) {
            Err(error) => rejected.push(format!("{label} [{}]", error.class())),
            Ok(_) => {
                let v1 = evaluate(load(text).unwrap(), &p1);
                let v2 = evaluate(load(text).unwrap(), &p2);
                let v4 = evaluate(load(text).unwrap(), &p4);
                println!("LOADED {label}: P1={v1:?} P2={v2:?} P4={v4:?}");
                loaded.push(label.to_string());
                if v1 != Verdict::Deny || v2 != Verdict::Escalate || v4 != Verdict::Deny {
                    failures.push(format!(
                        "{label}: P1={v1:?} (want Deny) P2={v2:?} (want Escalate) P4={v4:?} (want Deny)"
                    ));
                }
            }
        }
    }
    println!(
        "mutations run={total} rejected_at_load={} loaded={}",
        rejected.len(),
        loaded.len()
    );
    println!("loaded: {loaded:?}");
    assert!(
        failures.is_empty(),
        "verdicts changed:\n{}",
        failures.join("\n")
    );
}

fn deploy_action() -> AuthorityAction {
    AuthorityAction::Reserved {
        resource_type: "deploy_production".into(),
        resource_id: "prod".into(),
        parameters: serde_json::json!({}),
    }
}

fn evaluate(bundle: AuthorityPolicyBundle, action: &AuthorityAction) -> Verdict {
    let binding = bundle.resolve_identity("operator_local", ActorRole::Operator);
    let engine = PolicyAuthorityEngine::new(bundle).unwrap();
    let actor = Actor {
        actor_id: "operator_local".into(),
        role: ActorRole::Operator,
    };
    engine
        .evaluate(AuthorityEvaluation {
            actor: &actor,
            binding,
            run_id: "run_null_rules",
            case_id: "case_null_rules",
            plan_item_id: "item_null_rules",
            sequence: 1,
            action,
            workspace_root: Path::new("/tmp/workspace"),
            evidence_refs: vec!["intent:null_rules".into()],
            artifacts_root: None,
            timeout_secs: None,
            env_keys: Default::default(),
            domainforge_candidate: None,
            environment: None,
        })
        .unwrap()
        .verdict
}
