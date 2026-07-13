use sea_forge_authority::AuthorityPolicyBundle;
use sea_forge_core::{errors::ForgeError, types::AuthorityAction};
use std::{fs, path::Path};
pub fn execute(path: &Path, root: &Path, policy: Option<&Path>) -> Result<u8, ForgeError> {
    let action = AuthorityAction::Reserved {
        resource_type: "validate_model".into(),
        resource_id: path.to_string_lossy().into_owned(),
        parameters: serde_json::json!({}),
    };
    if let Some(policy) = policy {
        super::mediated::authorize_read(root, policy, "operator_local", &action)?;
    } else {
        let bundle: AuthorityPolicyBundle = serde_yaml::from_str(
            "version: \"0.2\"\nrules:\n  - name: validate-model\n    verdict: allow\n    actor_role: operator\n    operation_kind: validate_model\n",
        )
        .map_err(|error| ForgeError::Internal(error.to_string()))?;
        super::mediated::authorize_with_bundle(root, "operator_local", &action, bundle)?;
    }
    match fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| e.to_string())
        })
        .and_then(validate)
    {
        Ok(()) => {
            println!("sea-forge: model valid");
            Ok(0)
        }
        Err(reason) => {
            eprintln!("sea-forge: model invalid: {reason}");
            Ok(1)
        }
    }
}
fn validate(value: serde_json::Value) -> Result<(), String> {
    let domain = value
        .get("domain")
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .ok_or("domain must be a non-empty string")?;
    let _ = domain;
    let entities = value
        .get("entities")
        .and_then(|v| v.as_array())
        .filter(|v| !v.is_empty())
        .ok_or("entities must be a non-empty array")?;
    if entities.iter().all(|e| {
        e.get("name")
            .and_then(|v| v.as_str())
            .is_some_and(|v| !v.is_empty())
    }) {
        Ok(())
    } else {
        Err("every entity must have a non-empty name".into())
    }
}
