use sea_forge_core::errors::ForgeError;
pub enum IntentPattern {
    Demo,
    GeneratedZone,
    FalseSuccess,
    Nonzero,
    Timeout,
}
pub fn interpret(summary: &str) -> Result<IntentPattern, ForgeError> {
    let trimmed = summary.trim();
    if trimmed.is_empty() {
        return Err(ForgeError::Input("intent must not be empty".into()));
    }
    if trimmed.chars().count() > 500 {
        return Err(ForgeError::Input(
            "intent must be at most 500 characters".into(),
        ));
    }
    if trimmed == "TEST_ONLY: write generated zone" {
        return Ok(IntentPattern::GeneratedZone);
    }
    if trimmed == "TEST_ONLY: false success" {
        return Ok(IntentPattern::FalseSuccess);
    }
    if trimmed == "TEST_ONLY: nonzero" {
        return Ok(IntentPattern::Nonzero);
    }
    if trimmed == "TEST_ONLY: timeout" {
        return Ok(IntentPattern::Timeout);
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("generate") && lower.contains(".sea model") {
        Ok(IntentPattern::Demo)
    } else {
        Err(ForgeError::UnknownIntent(
            "unknown intent; supported: generate a .sea model".into(),
        ))
    }
}
