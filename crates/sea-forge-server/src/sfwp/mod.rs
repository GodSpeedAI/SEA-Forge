//! SFWP (SEA Forge Workbench Protocol) additive transport layer (Task 3).
//!
//! # Design seam (ADR-003)
//!
//! SFWP is layered *additively* on the existing Unix-socket NDJSON server.
//! Per ADR-003 (additive audit-remediation contracts), new capability lands as
//! new `verb`-tagged [`crate::Request`] variants — never as a reshape of the
//! existing ones, and never as a nested wrapper frame. Old clients omitting the
//! new verbs keep working unchanged; old servers reject an unknown verb cleanly
//! (serde unknown-variant), which is exactly the additive-evolution guarantee
//! ADR-003 requires. We deliberately did NOT introduce a wrapper
//! `Sfwp(SfwpFrame)` variant: flat additive verbs match the existing pattern
//! exactly and keep the negotiation/correlation/event surface legible.

pub mod approvals;
pub mod assets;
pub mod case;
pub mod case_views;
pub mod correlation;
pub mod delegation_preview;
pub mod delegations;
pub mod events;
pub mod precondition;
pub mod readiness;
pub mod run_views;
pub mod thoth;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The SFWP protocol version this server speaks. Only major version `"1"` is
/// supported; a client requesting an unsupported major is rejected with a
/// structured `unsupported_version` error (never a panic).
pub const SFWP_PROTOCOL_VERSION: &str = "1";

/// Interaction class for an SFWP method, mirroring the classes named in
/// `.claude/skills/building-sea-forge-workbench/reference/api-and-event-contracts.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InteractionClass {
    Inspect,
    Command,
    Subscribe,
}

impl InteractionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            InteractionClass::Inspect => "inspect",
            InteractionClass::Command => "command",
            InteractionClass::Subscribe => "subscribe",
        }
    }
}

/// A method in the SFWP catalog. Maintained as a manually-written table
/// because Rust has no enum-variant reflection without an extra derive
/// dependency, which is out of scope for Task 3.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MethodDescriptor {
    pub method: &'static str,
    pub class: InteractionClass,
}

/// The SFWP methods actually implemented by this server. This is the single
/// source of truth for `system.hello`'s `implemented_methods` and
/// `system.describe`'s class table.
pub const IMPLEMENTED_METHODS: &[MethodDescriptor] = &[
    MethodDescriptor {
        method: "system.hello",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "system.describe",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "system.get_schema",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "request.get_status",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "events.subscribe",
        class: InteractionClass::Subscribe,
    },
    MethodDescriptor {
        method: "events.unsubscribe",
        class: InteractionClass::Command,
    },
    MethodDescriptor {
        method: "events.get_range",
        class: InteractionClass::Inspect,
    },
    // --- SFWP additive inspect method (Task 5, ADR-003) ---
    // `readiness.get` is a read-only projection over already-proven kernel
    // truth (self-model validation + endpoint config). No wrapper frame; a thin
    // additive verb per the module seam above.
    MethodDescriptor {
        method: "readiness.get",
        class: InteractionClass::Inspect,
    },
    // --- SFWP additive identity method (SF-005) ---
    // Read-only: reports which actors the calling connection's uid may claim.
    // Scoped to that uid, so it discloses nothing about other principals.
    MethodDescriptor {
        method: "identity.get",
        class: InteractionClass::Inspect,
    },
    // --- Thoth (E13) ---
    // The kernel has answered `ask` since M11, but it was never listed here, so
    // `system.hello` did not advertise it and no client could discover it: an
    // implemented capability with no reachable path, which is precisely the
    // distinction the catalog exists to make (epic invariant 6). Classed as a
    // command, not an inspect: answering is disclosure-controlled and is
    // recorded, so it is governed even though it mutates no case.
    MethodDescriptor {
        method: "thoth.ask",
        class: InteractionClass::Command,
    },
    // --- SFWP additive case-authoring methods (Task 6, ADR-003) ---
    // `entry_options`/`preflight` are read-only projections/dry-runs (no case,
    // run, or ledger is created); `commit` is the one protected verb that
    // actually creates a case, over the existing `case_dispatch::submit` path.
    MethodDescriptor {
        method: "case.entry_options",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "case.preflight",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "case.commit",
        class: InteractionClass::Command,
    },
    // --- SFWP additive case-navigation methods (Task 7, ADR-003) ---
    // All three are read-only projections over committed case records; they
    // create nothing and derive item standing by folding the trace events the
    // case runner already appended (see `case_views`).
    MethodDescriptor {
        method: "case.list",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "case.get_overview",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "case.get_horizon",
        class: InteractionClass::Inspect,
    },
    // --- SFWP additive approval methods (Task 7, ADR-003) ---
    // `approval.list` is the missing half of an already-reachable capability:
    // `approve`/`reject` require ids no method could previously enumerate.
    // `approval.decide` is a thin envelope over that same governance path — it
    // forks no decision logic.
    MethodDescriptor {
        method: "approval.list",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "approval.decide",
        class: InteractionClass::Command,
    },
    // --- SFWP additive run-record methods (Task 8, ADR-003) ---
    // Read-only projections over per-run committed records. `run.get` is the
    // resolution target for the run ids every other view already emits — see
    // `run_views` for why the criteria pairing joins the settlement's own basis
    // rather than re-evaluating criteria.
    MethodDescriptor {
        method: "run.list",
        class: InteractionClass::Inspect,
    },
    MethodDescriptor {
        method: "run.get",
        class: InteractionClass::Inspect,
    },
    // --- SFWP additive asset-catalog method (Task 9, ADR-003) ---
    // `asset.list` is a read-only projection over three sources the kernel
    // already owns (materialized templates, configured agent endpoints, the
    // extension registry) plus the probe evidence that has accrued against
    // each endpoint. It creates nothing; see `assets` for why the three
    // standing vocabularies stay disjoint.
    MethodDescriptor {
        method: "asset.list",
        class: InteractionClass::Inspect,
    },
    // --- SFWP additive delegation method (Task 10, ADR-003) ---
    // `delegation.preview` projects the job contract a `delegate` would run
    // under. Inspect, not command: it commits no intent, plan, criteria, or
    // authority decision, and it deliberately does *not* evaluate authority —
    // a verdict with no ledger entry behind it would be an unrecorded grant.
    MethodDescriptor {
        method: "delegation.preview",
        class: InteractionClass::Inspect,
    },
    // --- SFWP additive delegation roster (Task 11, ADR-003) ---
    // `delegation.list` is the missing half of an already-reachable capability:
    // `cancel_delegation` has existed since M12 but nothing could enumerate
    // what there was to cancel. Inspect — it joins the server's live handles
    // with the committed run records and reports the seam between them rather
    // than guessing across it (see `delegations`).
    MethodDescriptor {
        method: "delegation.list",
        class: InteractionClass::Inspect,
    },
];

/// True if `requested` names a supported protocol major version.
pub fn version_supported(requested: &str) -> bool {
    // Accept exact "1" or a "1.x" form — compare the major component only.
    let major = requested.split('.').next().unwrap_or(requested);
    major == SFWP_PROTOCOL_VERSION
}

/// `system.hello` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct HelloResult {
    /// Echo of the version the client requested and that was accepted.
    pub protocol_version: String,
    /// The server's own supported protocol version.
    pub server_protocol_version: String,
    pub implemented_methods: Vec<String>,
}

/// `system.describe` result body: method catalog with interaction classes.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DescribeResult {
    pub protocol_version: String,
    pub methods: Vec<MethodInfo>,
}

/// A single described method.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MethodInfo {
    pub method: String,
    pub class: String,
}

/// A schema reference the generator emits (relative path under the contracts
/// schema directory).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SchemaRef {
    /// The type/shape this schema describes.
    pub name: String,
    /// Relative path to the generated JSON Schema file.
    pub path: String,
}

/// `system.get_schema` result body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetSchemaResult {
    pub schemas: Vec<SchemaRef>,
}

/// `unsupported_version` structured error body.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UnsupportedVersion {
    /// Always `"unsupported_version"`.
    pub error_class: String,
    pub requested: String,
    pub supported: String,
    pub error: String,
}

/// The set of generated schema types, paired with their emitted filenames.
/// The generator binary and `system.get_schema` share this list so they never
/// drift.
pub const SCHEMA_TYPES: &[&str] = &[
    "EventFrame",
    "Precondition",
    "RecordDigest",
    "ChangedRecord",
    "RejectedAsStale",
    "RequestRecord",
    "HelloResult",
    "DescribeResult",
    "GetSchemaResult",
    "UnsupportedVersion",
    "MethodDescriptor",
    "ReadinessView",
    "ReadinessItem",
    "SourceRecordRef",
    "ReadinessGetParams",
    "IdentityView",
    "AvailableActor",
    "RefusalView",
    "ThothAnswerView",
    "ClaimView",
    "EntryOptionsResult",
    "TemplateOption",
    "TemplateParameter",
    "PreflightParams",
    "PreflightResult",
    "PlanItemSummary",
    "CaseListResult",
    "CaseSummary",
    "CaseOverview",
    "CaseHorizon",
    "HorizonItem",
    "RunSettlement",
    "ApprovalListResult",
    "PendingApproval",
    "ApprovalGovernanceContext",
    "RunListResult",
    "RunSummary",
    "RunRecord",
    "RunTermination",
    "AuthorityProjection",
    "CriterionCheck",
    "SettlementDetail",
    "DeclarationRow",
    "EvidenceRow",
    "TraceRow",
    "RecordPresence",
    "AssetListResult",
    "AssetRow",
    "AssetKind",
    "DelegationPreviewParams",
    "DelegationPreviewResult",
    "JobContractPreview",
    "ResolvedValue",
    "ValueSource",
    "DelegationListResult",
    "DelegationRow",
    "DelegationStanding",
];

/// Build the `system.hello` result, or an `unsupported_version` error if the
/// requested version's major is not supported.
pub fn hello(requested_version: &str) -> Result<HelloResult, UnsupportedVersion> {
    if !version_supported(requested_version) {
        return Err(UnsupportedVersion {
            error_class: "unsupported_version".into(),
            requested: requested_version.to_string(),
            supported: SFWP_PROTOCOL_VERSION.to_string(),
            error: format!(
                "unsupported SFWP protocol version {requested_version}; server supports major {SFWP_PROTOCOL_VERSION}"
            ),
        });
    }
    Ok(HelloResult {
        protocol_version: requested_version.to_string(),
        server_protocol_version: SFWP_PROTOCOL_VERSION.to_string(),
        implemented_methods: IMPLEMENTED_METHODS
            .iter()
            .map(|descriptor| descriptor.method.to_string())
            .collect(),
    })
}

/// Build the `system.describe` result.
pub fn describe() -> DescribeResult {
    DescribeResult {
        protocol_version: SFWP_PROTOCOL_VERSION.to_string(),
        methods: IMPLEMENTED_METHODS
            .iter()
            .map(|descriptor| MethodInfo {
                method: descriptor.method.to_string(),
                class: descriptor.class.as_str().to_string(),
            })
            .collect(),
    }
}

/// Build the `system.get_schema` result. When `method` is `Some`, only that
/// method's referenced schemas are returned; otherwise all generated schemas.
pub fn get_schema(method: Option<&str>) -> GetSchemaResult {
    // All SFWP methods share the same generated transport contracts today, so
    // a per-method filter selects the whole set unless the method is unknown.
    let known = method
        .map(|m| IMPLEMENTED_METHODS.iter().any(|d| d.method == m))
        .unwrap_or(true);
    let schemas = if known {
        SCHEMA_TYPES
            .iter()
            .map(|name| SchemaRef {
                name: (*name).to_string(),
                path: format!("workbench/packages/contracts/schema/{name}.schema.json"),
            })
            .collect()
    } else {
        Vec::new()
    };
    GetSchemaResult { schemas }
}
