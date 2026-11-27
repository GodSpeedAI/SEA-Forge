//! Shared by the CEP validator tests (`mod common;`).

/// Whether `REQUIRE_CEP` demands the external validator: anything but unset, empty, `0` or `false`.
pub fn require_cep(value: Option<&str>) -> bool {
    value.is_some_and(|v| {
        let v = v.trim();
        !(v.is_empty() || v == "0" || v.eq_ignore_ascii_case("false"))
    })
}

/// The CEP checkout for the validator tests. Unset, they skip, unless `REQUIRE_CEP` is set, which makes a
/// missing checkout a failure so a gate cannot pass by silently skipping the external validator.
pub fn cep_repo() -> Option<String> {
    match std::env::var("CEP_REPO") {
        Ok(path) if !path.is_empty() => Some(path),
        _ => {
            assert!(
                !require_cep(std::env::var("REQUIRE_CEP").ok().as_deref()),
                "REQUIRE_CEP is set but CEP_REPO is not"
            );
            eprintln!("CEP_REPO not set: skipping profile conformance");
            None
        }
    }
}

#[test]
fn require_cep_accepts_one_and_true_and_refuses_off_values() {
    for on in ["1", "true", "TRUE", "yes"] {
        assert!(require_cep(Some(on)), "{on}");
    }
    for off in ["", "0", "false", "False", " "] {
        assert!(!require_cep(Some(off)), "{off:?}");
    }
    assert!(!require_cep(None));
}
