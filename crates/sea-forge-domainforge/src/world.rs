//! Semantic-world binding (CEP/world_ref migration, Stage 4).
//!
//! SEA-Forge never trusts a sender's `world_ref`. It recomputes the identity
//! digest from the carried `DomainModelIdentity` with the pinned DomainForge
//! implementation, requires the world to be known to its registry, and fails
//! closed on anything unknown, malformed, tampered, mismatched or invalid.
//! The registry is a derived in-memory cache; it assumes no storage backend.

use crate::{domain_model_error, DomainModelRef, SeaSourceSet};
use domainforge_core::application::envelope::DomainModelIdentity;
use domainforge_core::application::resolve_semantic_envelope;
use domainforge_core::application::world::{WorldCatalog, WorldName, WorldRef, WorldRefError};
use sea_forge_core::errors::ForgeError;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;

/// Why a world could not be bound. Every variant is a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldBindingError {
    /// The envelope has no `scope.world_ref`.
    MissingWorldRef,
    /// The envelope carries no `domainforge.identity` to recompute from.
    MissingIdentity,
    /// The envelope declares its model invalid; invalid worlds are never bound.
    InvalidModel,
    /// Malformed ref, malformed identity, digest mismatch, unknown world, name conflict.
    World(WorldRefError),
    /// The presented identity hashes to the ref but differs from the registered one.
    IdentityDiffersFromRegistered(String),
    /// The model ref's own identity does not match the registered world.
    ModelRefMismatch { field: &'static str },
    /// The model ref carries no `world_ref`.
    ModelRefUnbound,
    /// The model could not be resolved by DomainForge.
    Unresolvable(String),
}

impl fmt::Display for WorldBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWorldRef => write!(f, "snapshot has no scope.world_ref"),
            Self::MissingIdentity => write!(f, "snapshot has no domainforge.identity extension"),
            Self::InvalidModel => write!(f, "snapshot declares an invalid model; refusing to bind"),
            Self::World(e) => write!(f, "{e}"),
            Self::IdentityDiffersFromRegistered(w) => {
                write!(f, "identity for {w} differs from the registered identity")
            }
            Self::ModelRefMismatch { field } => {
                write!(f, "model ref {field} does not match the bound world")
            }
            Self::ModelRefUnbound => write!(f, "model ref carries no world_ref"),
            Self::Unresolvable(m) => write!(f, "model does not resolve: {m}"),
        }
    }
}

impl std::error::Error for WorldBindingError {}

impl From<WorldRefError> for WorldBindingError {
    fn from(e: WorldRefError) -> Self {
        Self::World(e)
    }
}

impl From<WorldBindingError> for ForgeError {
    fn from(e: WorldBindingError) -> Self {
        domain_model_error(format!("world binding refused: {e}"))
    }
}

/// Verified worlds, cached by canonical `world_ref` text. Append-only: a
/// registered world is never replaced or removed.
#[derive(Debug, Default, Clone)]
pub struct WorldRegistry {
    catalog: WorldCatalog,
    by_ref: BTreeMap<String, WorldRef>,
}

impl WorldRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct registered worlds.
    pub fn len(&self) -> usize {
        self.by_ref.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_ref.is_empty()
    }

    /// Derive the DomainForge identity of a source set and register its world.
    pub fn register_source_set(
        &mut self,
        name: &str,
        source_set: &SeaSourceSet,
    ) -> Result<WorldRef, WorldBindingError> {
        let sources: BTreeMap<&str, &str> = source_set
            .files
            .iter()
            .map(|f| (f.uri.as_str(), f.content.as_str()))
            .collect();
        let sources_json = serde_json::to_string(&sources)
            .map_err(|e| WorldBindingError::Unresolvable(e.to_string()))?;
        let doc = resolve_semantic_envelope(&source_set.entry_uri, &sources_json)
            .map_err(|d| WorldBindingError::Unresolvable(format!("{d:?}")))?;
        self.register_identity(name, DomainModelIdentity::from_document(&doc, None))
    }

    /// Register a world from an identity (recomputes the digest).
    pub fn register_identity(
        &mut self,
        name: &str,
        identity: DomainModelIdentity,
    ) -> Result<WorldRef, WorldBindingError> {
        let name: WorldName = name.parse()?;
        let world = self.catalog.register(name, identity)?;
        self.by_ref.insert(world.to_string(), world.clone());
        Ok(world)
    }

    /// The registered identity for a canonical `world_ref`; unknown fails closed.
    pub fn require(
        &self,
        world_ref: &str,
    ) -> Result<(&WorldRef, &DomainModelIdentity), WorldBindingError> {
        let world: WorldRef = world_ref.parse()?;
        let known = self
            .by_ref
            .get(&world.to_string())
            .ok_or_else(|| WorldRefError::UnknownWorld(world.to_string()))?;
        Ok((known, self.catalog.lookup(known)?))
    }

    /// Verify a CEP `semantic_snapshot` against the registry and return the
    /// bound `WorldRef`. The digest is recomputed from the carried identity.
    pub fn verify_snapshot(&self, envelope: &Value) -> Result<WorldRef, WorldBindingError> {
        let world_ref = envelope
            .pointer("/scope/world_ref")
            .and_then(Value::as_str)
            .ok_or(WorldBindingError::MissingWorldRef)?;
        if envelope
            .pointer("/extensions/domainforge/model_validation_status")
            .and_then(Value::as_str)
            == Some("invalid")
        {
            return Err(WorldBindingError::InvalidModel);
        }
        let identity_value = envelope
            .pointer("/extensions/domainforge.identity/domain_model_identity")
            .ok_or(WorldBindingError::MissingIdentity)?;
        let presented: DomainModelIdentity = serde_json::from_value(identity_value.clone())
            .map_err(|e| WorldRefError::MalformedIdentity(e.to_string()))?;

        let (world, registered) = self.require(world_ref)?;
        world.verify_identity(&presented)?;
        if *registered != presented {
            return Err(WorldBindingError::IdentityDiffersFromRegistered(
                world.to_string(),
            ));
        }
        Ok(world.clone())
    }

    /// Bind SEA-Forge's own model identity to a registered world.
    pub fn bind(
        &self,
        model_ref: &mut DomainModelRef,
        world_ref: &str,
    ) -> Result<(), WorldBindingError> {
        let (world, _) = self.require(world_ref)?;
        let canonical = world.to_string();
        self.check_model_ref_against(model_ref, &canonical)?;
        model_ref.world_ref = Some(canonical);
        Ok(())
    }

    /// Re-verify a bound model ref: its `world_ref` must be known and the
    /// world's identity must agree with the model ref's own hashes.
    pub fn verify_model_ref(
        &self,
        model_ref: &DomainModelRef,
    ) -> Result<WorldRef, WorldBindingError> {
        let world_ref = model_ref
            .world_ref
            .as_deref()
            .ok_or(WorldBindingError::ModelRefUnbound)?;
        self.check_model_ref_against(model_ref, world_ref)?;
        Ok(self.require(world_ref)?.0.clone())
    }

    fn check_model_ref_against(
        &self,
        model_ref: &DomainModelRef,
        world_ref: &str,
    ) -> Result<(), WorldBindingError> {
        let (_, identity) = self.require(world_ref)?;
        if model_ref.d_content_hash.as_deref() != Some(identity.content_hash.as_str()) {
            return Err(WorldBindingError::ModelRefMismatch {
                field: "d_content_hash",
            });
        }
        if model_ref.semantic_closure_hash.as_deref()
            != Some(identity.semantic_closure_hash.as_str())
        {
            return Err(WorldBindingError::ModelRefMismatch {
                field: "semantic_closure_hash",
            });
        }
        Ok(())
    }
}
