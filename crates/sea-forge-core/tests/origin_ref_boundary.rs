use sea_forge_core::types::{OriginRef, OriginRefKind, OriginRole};

/// M10 (E12 §7.7): `OriginRefKind::DesiredOutcome` is a closed-serde-enum
/// addition. This test proves an old reader (simulated by deserializing into a
/// restricted enum) gets a clean serde `Err`, never a panic. Old readers are
/// structurally shielded: `desired_outcome` refs only appear in new ADLC/ODI
/// template-derived criteria records, which old code never encounters.
#[test]
fn old_reader_rejects_desired_outcome_variant_cleanly() {
    // Serialize a DesiredOutcome origin ref (new variant).
    let new_ref = OriginRef {
        kind: OriginRefKind::DesiredOutcome,
        reference: "outcome:faster_builds".into(),
        sha256: "sha256:abc".into(),
        role: OriginRole::DesiredResult,
        evidence_refs: vec![],
        domain_model_ref: Some("godspeed.adlc_odi_case".into()),
    };
    let json = serde_json::to_string(&new_ref).unwrap();

    // The serialized form uses "desired_outcome" as the kind tag.
    assert!(json.contains("\"desired_outcome\""));

    // An old reader that doesn't know the variant gets a clean error.
    #[derive(serde::Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    enum LegacyOriginRefKind {
        Intent,
        PlanTemplate,
        SpecPipelineStage,
        PolicyRequirement,
        Issue,
        ExternalRequirement,
        JobContract,
        ImplementationDefined,
    }

    #[derive(serde::Deserialize, Debug)]
    struct LegacyRef {
        kind: LegacyOriginRefKind,
        reference: String,
        sha256: String,
    }

    let result: Result<LegacyRef, _> = serde_json::from_str(&json);
    assert!(result.is_err(), "old reader must reject desired_outcome");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("desired_outcome") || err.contains("unknown variant"),
        "error should name the unknown variant: {err}"
    );
}

/// Legacy origin refs (without domain_model_ref) still round-trip.
#[test]
fn legacy_origin_ref_without_domain_model_ref_round_trips() {
    let legacy = OriginRef {
        kind: OriginRefKind::Intent,
        reference: "int_01".into(),
        sha256: "sha256:xyz".into(),
        role: OriginRole::DesiredResult,
        evidence_refs: vec![],
        domain_model_ref: None,
    };
    let json = serde_json::to_string(&legacy).unwrap();
    // domain_model_ref should be absent (skip_serializing_if = "Option::is_none")
    assert!(!json.contains("domain_model_ref"));
    let back: OriginRef = serde_json::from_str(&json).unwrap();
    assert_eq!(legacy, back);
}
