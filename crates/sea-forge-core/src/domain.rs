use crate::errors::ForgeError;
pub enum IntentPattern {
    Demo,
    GeneratedZone,
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
    let lower = trimmed.to_lowercase();
    if lower.contains("generate") && lower.contains(".sea model") {
        Ok(IntentPattern::Demo)
    } else {
        Err(ForgeError::UnknownIntent(
            "unknown intent; supported: generate a .sea model".into(),
        ))
    }
}
