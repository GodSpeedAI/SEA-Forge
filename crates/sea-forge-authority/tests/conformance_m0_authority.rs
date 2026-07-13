use sea_forge_authority::{
    resolve_candidates, GovernanceDisposition as D, GovernanceVerdict, ResolutionPolicy,
};
use std::collections::{BTreeMap, BTreeSet};

fn candidate(disposition: D) -> GovernanceVerdict {
    GovernanceVerdict {
        engine: format!("{disposition:?}"),
        disposition: disposition.clone(),
        boundaries: if disposition == D::Boundary {
            BTreeMap::from([("workspace".into(), BTreeSet::from(["/safe".into()]))])
        } else {
            BTreeMap::new()
        },
        compensating_controls: if disposition == D::Degraded {
            BTreeSet::from(["audit".into()])
        } else {
            BTreeSet::new()
        },
        reason: "fixture".into(),
        evidence_refs: vec!["evi_fixture".into()],
    }
}

fn expected(left: &D, right: &D) -> D {
    if left == &D::Deny || right == &D::Deny {
        D::Deny
    } else if left == &D::Escalate || right == &D::Escalate {
        D::Escalate
    } else if left == &D::Boundary || right == &D::Boundary {
        D::Boundary
    } else if left == &D::Degraded || right == &D::Degraded {
        D::Degraded
    } else {
        D::Allow
    }
}

#[test]
fn complete_pairwise_matrix_is_commutative() {
    let dispositions = [D::Allow, D::Deny, D::Escalate, D::Boundary, D::Degraded];
    let policy = ResolutionPolicy {
        allow_degraded: true,
    };
    for left in &dispositions {
        for right in &dispositions {
            let forward = resolve_candidates(
                &[candidate(left.clone()), candidate(right.clone())],
                &policy,
            );
            let reverse = resolve_candidates(
                &[candidate(right.clone()), candidate(left.clone())],
                &policy,
            );
            assert_eq!(forward, reverse, "{left:?} with {right:?}");
            assert_eq!(forward.disposition, expected(left, right));
        }
    }
}

#[test]
fn resolution_is_idempotent_and_group_order_independent() {
    let policy = ResolutionPolicy {
        allow_degraded: true,
    };
    for disposition in [D::Allow, D::Deny, D::Escalate, D::Boundary, D::Degraded] {
        assert_eq!(
            resolve_candidates(&[candidate(disposition.clone())], &policy),
            resolve_candidates(
                &[candidate(disposition.clone()), candidate(disposition)],
                &policy,
            )
        );
    }
    let first = resolve_candidates(
        &[
            candidate(D::Allow),
            candidate(D::Boundary),
            candidate(D::Escalate),
        ],
        &policy,
    );
    let regrouped = resolve_candidates(
        &[
            candidate(D::Escalate),
            candidate(D::Allow),
            candidate(D::Boundary),
        ],
        &policy,
    );
    assert_eq!(first, regrouped);
}

#[test]
fn domainforge_authority_is_model_semantic() {
    let source = include_str!("../../sea-forge-domainforge/tests/fixtures/demo.sea");
    let source_hash = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(source.as_bytes()))
    };
    let model = sea_forge_domainforge::load_validate(&sea_forge_domainforge::SeaSourceSet {
        entry_uri: "demo.sea".into(),
        files: vec![sea_forge_domainforge::SourceFile {
            uri: "demo.sea".into(),
            sha256: source_hash,
            content: source.into(),
        }],
    })
    .unwrap();
    let matching = sea_forge_domainforge::evaluate_authority(
        &model,
        "write_file",
        "Sample.sea",
        vec!["evi_model".into()],
    )
    .unwrap();
    let unrelated = sea_forge_domainforge::evaluate_authority(
        &model,
        "write_file",
        "Unknown.sea",
        vec!["evi_model".into()],
    )
    .unwrap();
    assert_eq!(
        matching.normalized_disposition,
        sea_forge_domainforge::CandidateDisposition::Allow
    );
    assert_eq!(
        unrelated.normalized_disposition,
        sea_forge_domainforge::CandidateDisposition::Deny
    );
}
