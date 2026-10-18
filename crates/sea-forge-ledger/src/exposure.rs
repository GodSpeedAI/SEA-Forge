//! Typed exposure state for execution contexts (SEC-09/SEC-10/SEC-11).
//!
//! An execution context (an agent run, a resumed session of it, or a
//! delegated child) accumulates exposure labels only from trusted
//! observations: SEA Forge components that actually delivered sensitive
//! plaintext to it (for example the MCP broker forwarding a classified tool
//! result). Labels never come from the agent and are never removed; a child
//! context inherits every ancestor's labels and revocation. The state is a
//! fold over an append-only ledger stream, so it survives restarts and
//! resumption and stays auditable.
//!
//! Using a brokered credential is not exposure: only plaintext the context
//! received is labelled.
use crate::LedgerStream;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const STREAM: &str = "exposure";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ContextState {
    parent: Option<String>,
    labels: BTreeSet<String>,
    revoked: bool,
}

/// Exposure registry backed by the `exposure` ledger stream under `root`.
pub struct ExposureRegistry {
    ledger: LedgerStream,
}

fn valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}

impl ExposureRegistry {
    pub fn open(root: &Path, observer: &str) -> Result<Self, sea_forge_core::errors::ForgeError> {
        Ok(Self {
            ledger: LedgerStream::open(root, STREAM, observer)?,
        })
    }

    fn states(&self) -> Result<BTreeMap<String, ContextState>, Box<dyn std::error::Error>> {
        let mut states: BTreeMap<String, ContextState> = BTreeMap::new();
        for entry in self.ledger.read_entries()? {
            let context = entry.payload["context"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let state = states.entry(context).or_default();
            match entry.record_kind.as_str() {
                "exposure_observed" => {
                    if let Some(label) = entry.payload["label"].as_str() {
                        state.labels.insert(label.to_string());
                    }
                }
                "exposure_context_derived" => {
                    state.parent = entry.payload["parent"].as_str().map(str::to_string);
                }
                "exposure_context_revoked" => state.revoked = true,
                _ => {}
            }
        }
        Ok(states)
    }

    /// Ancestors-inclusive walk; a cycle or unknown parent stops the walk.
    fn lineage(states: &BTreeMap<String, ContextState>, context: &str) -> Vec<String> {
        let mut seen = Vec::new();
        let mut current = Some(context.to_string());
        while let Some(id) = current {
            if seen.contains(&id) {
                break;
            }
            current = states.get(&id).and_then(|s| s.parent.clone());
            seen.push(id);
        }
        seen
    }

    /// Record that `context` actually received plaintext labelled `label`.
    pub fn record(
        &self,
        context: &str,
        label: &str,
        source: &str,
        evidence: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !valid(context) || !valid(label) {
            return Err("invalid exposure context or label".into());
        }
        self.ledger.commit_typed(
            "exposure_observed",
            vec![context.into()],
            &json!({"context": context, "label": label, "source": source,
                "evidence_sha256": evidence}),
            vec![],
        )?;
        Ok(())
    }

    /// Create a delegated child context that inherits the parent's exposure
    /// and revocation. A context's parent can be set once.
    pub fn derive(&self, parent: &str, child: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !valid(parent) || !valid(child) || parent == child {
            return Err("invalid delegation lineage".into());
        }
        let states = self.states()?;
        if states.get(child).is_some_and(|s| s.parent.is_some()) {
            return Err("context already has a parent".into());
        }
        if Self::lineage(&states, parent).iter().any(|id| id == child) {
            return Err("delegation lineage would form a cycle".into());
        }
        self.ledger.commit_typed(
            "exposure_context_derived",
            vec![child.into(), parent.into()],
            &json!({"context": child, "parent": parent}),
            vec![],
        )?;
        Ok(())
    }

    pub fn revoke(&self, context: &str, reason: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !valid(context) {
            return Err("invalid exposure context".into());
        }
        self.ledger.commit_typed(
            "exposure_context_revoked",
            vec![context.into()],
            &json!({"context": context, "reason": reason}),
            vec![],
        )?;
        Ok(())
    }

    /// Labels of `context` and all its ancestors, sorted.
    pub fn labels(&self, context: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let states = self.states()?;
        let mut labels = BTreeSet::new();
        for id in Self::lineage(&states, context) {
            if let Some(state) = states.get(&id) {
                labels.extend(state.labels.iter().cloned());
            }
        }
        Ok(labels.into_iter().collect())
    }

    /// True when `context` is `ancestor` or descends from it, so a label
    /// recorded on `ancestor` reaches `context`.
    pub fn inherits(
        &self,
        context: &str,
        ancestor: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let states = self.states()?;
        Ok(Self::lineage(&states, context)
            .iter()
            .any(|id| id == ancestor))
    }

    /// True when `context` or any ancestor was revoked.
    pub fn revoked(&self, context: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let states = self.states()?;
        Ok(Self::lineage(&states, context)
            .iter()
            .any(|id| states.get(id).is_some_and(|s| s.revoked)))
    }
}
