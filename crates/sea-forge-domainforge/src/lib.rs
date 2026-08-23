//! SEA Forge first-party DomainForge semantic adapter (spec-full §7.0a).
//!
//! Public contract (synchronous, side-effect-free):
//!   load_validate(SeaSourceSet) -> DomainModel
//!   evaluate(DomainModel, CanonicalActionRequest, TrustedFacts) -> GovernanceVerdict
//!   project(DomainModel, ProjectionRequest) -> SortedMap<RelativePath, Bytes>

#![forbid(unsafe_code)]

use domainforge_core::application::envelope::{CanonicalImportEdge, CanonicalModuleRef};
use domainforge_core::application::{
    resolve_application_graph, resolve_semantic_envelope, ApplicationDiagnostic,
};
use domainforge_core::parser::parse_source;
use domainforge_core::policy::Severity;
use sea_forge_core::errors::ForgeError;
use sea_forge_core::types::ProjectionKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// A source file in a `SeaSourceSet`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceFile {
    pub uri: String,
    pub sha256: String,
    pub content: String,
}

/// One entry `.sea` source and any namespace-registry or imported `.sea` files.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeaSourceSet {
    pub entry_uri: String,
    pub files: Vec<SourceFile>,
}

/// In-memory derived view of a validated semantic model (not persisted truth).
#[derive(Clone, Debug)]
pub struct DomainModel {
    pub graph: domainforge_core::graph::Graph,
    pub model_ref: DomainModelRef,
}

/// Stable reference to a validated semantic input.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DomainModelRef {
    pub source_refs: Vec<SourceRef>,
    pub domainforge_version: String,
    pub adapter_descriptor_sha256: String,
    pub parse_options_sha256: String,
    pub semantic_model_sha256: String,
    pub concept_refs: Vec<String>,
    #[serde(default)]
    pub class_refs: Vec<String>,
    pub validation_evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRef {
    pub uri: String,
    pub sha256: String,
}

/// Normalized candidate disposition from a DomainForge authority result.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateDisposition {
    Allow,
    Deny,
    Escalate,
}

/// DomainForge authority verdict trace preserved as evidence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainForgeTrace {
    pub raw_decision: String,
    pub normalized_disposition: CandidateDisposition,
    pub reason: String,
    pub evidence_refs: Vec<String>,
}

/// Descriptor hash for this adapter (fixed for a given adapter version).
pub const ADAPTER_DESCRIPTOR_SHA256: &str =
    "sha256:0000000000000000000000000000000000000000000000000000000000000001";

// ── Finite limits (spec-full §7.0a) ──

pub const MAX_SOURCE_COUNT: usize = 64;
pub const MAX_AGGREGATE_BYTES: usize = 1_048_576; // 1 MiB
pub const MAX_IMPORT_DEPTH: usize = 16;
pub const MAX_AST_NODES: usize = 10_000;
/// Maximum lexical expression-nesting depth (bracket depth, unary `-` chain,
/// or `not` chain) accepted before parsing. Bounds the external pest parser's
/// recursion so a crafted model cannot overflow the stack and abort the
/// process (SUP-01). Well below the ~1,000-level depth that still parses, and
/// far above any legitimate model.
pub const MAX_NESTING_DEPTH: usize = 256;

/// The pinned DomainForge version this adapter expects.
pub const EXPECTED_DOMAINFORGE_VERSION: &str = "0.16.0";

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_content(content: &str) -> String {
    sha256_hex(content.as_bytes())
}

fn sha256_json(value: &Value) -> Result<String, ForgeError> {
    let bytes = serde_json::to_vec(value).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

/// Normalize an entry-spelling for identity hashing (SUP-02). DomainForge
/// resolves `model.sea` and `./model.sea` to the same canonical closure root,
/// so the semantic identity must not depend on the caller's raw spelling.
/// Strips a single leading `./` and collapses duplicate `/` segments so
/// aliases (`./entry.sea`, `entry//./x.sea`) never fork the hash. Used only
/// for the semantic-model identity — the resolved graph, not this spelling,
/// is the authority for resolution.
fn normalize_entry_uri(uri: &str) -> String {
    let stripped = uri.strip_prefix("./").unwrap_or(uri);
    let parts: Vec<&str> = stripped.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return "".into();
    }
    if uri == "." {
        return ".".into();
    }
    parts.join("/")
}

/// Load and validate a `SeaSourceSet` through `domainforge-core`.
///
/// Verifies every source URI/hash, resolves imports from the supplied set,
/// enforces finite limits, and rejects unsupported DomainForge versions before
/// returning a `DomainModelRef`. No side effects.
//
// Import resolution and graph construction go through domainforge-core's
// public `application::resolve_application_graph` (0.15.0+), which takes the
// whole source set as an in-memory JSON map of uri -> content plus the entry
// uri. It resolves namespace/std/relative imports, detects cycles, checks
// named-import exports, and merges every reachable module's declarations into
// one graph — all without touching the filesystem or a CLI.
pub fn load_validate(source_set: &SeaSourceSet) -> Result<DomainModel, ForgeError> {
    // ── Version check ──
    if domainforge_core::VERSION != EXPECTED_DOMAINFORGE_VERSION {
        return Err(domain_model_error(format!(
            "unsupported DomainForge version: expected={EXPECTED_DOMAINFORGE_VERSION} linked={}",
            domainforge_core::VERSION
        )));
    }

    // ── Finite limits: source count ──
    if source_set.files.len() > MAX_SOURCE_COUNT {
        return Err(domain_model_error(format!(
            "source count {} exceeds limit {MAX_SOURCE_COUNT}",
            source_set.files.len()
        )));
    }

    // ── URI validation + duplicate detection + aggregate bytes ──
    let mut seen_uris = HashSet::new();
    let mut total_bytes = 0usize;
    for file in &source_set.files {
        if file.uri.is_empty() {
            return Err(domain_model_error("source URI must not be empty".into()));
        }
        if file.uri.starts_with('/') {
            return Err(domain_model_error(format!(
                "absolute URI rejected: {}",
                file.uri
            )));
        }
        if file.uri.split('/').any(|component| component == "..") {
            return Err(domain_model_error(format!(
                "traversal URI rejected: {}",
                file.uri
            )));
        }
        if !seen_uris.insert(&file.uri) {
            return Err(domain_model_error(format!(
                "duplicate source URI: {}",
                file.uri
            )));
        }
        total_bytes += file.content.len();
    }
    if total_bytes > MAX_AGGREGATE_BYTES {
        return Err(domain_model_error(format!(
            "aggregate bytes {total_bytes} exceeds limit {MAX_AGGREGATE_BYTES}"
        )));
    }

    // ── Hash verification for ALL files (not just entry) ──
    for file in &source_set.files {
        let computed = sha256_content(&file.content);
        if computed != file.sha256 {
            return Err(domain_model_error(format!(
                "source hash drift for {}: declared={} computed={computed}",
                file.uri, file.sha256
            )));
        }
    }

    // ── Pre-parse nesting-depth guard (SUP-01) ──
    // The external pest parser is recursive-descent with no depth cap, and the
    // AST-node budget runs *after* `parse_source`. A few KB of nested unary
    // (`-----…`) or parenthesized expressions overflows the stack and aborts
    // the whole process before the budget can reject it. Bound the lexical
    // nesting depth *before* parsing so pathological input yields a typed
    // error instead of a SIGABRT.
    for file in &source_set.files {
        if nesting_depth_exceeded(&file.content) {
            return Err(domain_model_error(format!(
                "expression nesting depth exceeds limit {MAX_NESTING_DEPTH} in {}",
                file.uri
            )));
        }
    }

    // ── Parse every file in-memory to enforce the AST node budget ──
    // Count every AST node recursively (top-level declarations, nested
    // declarations inside `export`, struct/record/enum bodies, operation
    // clauses, mapping/projection rule lists, and policy/metric expression
    // trees) so a model with few top-level declarations but deeply nested
    // expressions cannot slip past MAX_AST_NODES.
    let mut total_ast_nodes = 0usize;
    for file in &source_set.files {
        let ast = parse_source(&file.content)
            .map_err(|e| domain_model_error(format!("parse failed for {}: {e}", file.uri)))?;
        total_ast_nodes += count_ast_nodes(&ast);
    }
    if total_ast_nodes > MAX_AST_NODES {
        return Err(domain_model_error(format!(
            "AST node count {total_ast_nodes} exceeds limit {MAX_AST_NODES}"
        )));
    }

    // ── Resolve imports and enforce the canonical closure depth before graph
    // construction. The semantic envelope exposes DomainForge's own resolved
    // import graph, so SEA Forge neither reimplements namespace/relative/std
    // resolution nor trusts an authored import spelling. ──
    let sources: BTreeMap<String, String> = source_set
        .files
        .iter()
        .map(|f| (f.uri.clone(), f.content.clone()))
        .collect();
    let sources_json =
        serde_json::to_string(&sources).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let envelope = resolve_semantic_envelope(&source_set.entry_uri, &sources_json)
        .map_err(|diags| domain_model_error(format_diagnostics(&diags)))?;
    let import_depth =
        max_import_depth(&envelope.envelope.modules, &envelope.envelope.import_graph)?;
    if import_depth > MAX_IMPORT_DEPTH {
        return Err(domain_model_error(format!(
            "import depth {import_depth} exceeds limit {MAX_IMPORT_DEPTH}"
        )));
    }

    // The existing graph adapter remains the DomainModel source of truth.
    let graph = resolve_application_graph(&source_set.entry_uri, &sources_json)
        .map_err(|diags| domain_model_error(format_diagnostics(&diags)))?;

    // ── Semantic validation ──
    let validation = graph.validate();
    if validation.error_count > 0 {
        let errors: Vec<String> = validation
            .violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .map(|v| format!("{}: {}", v.policy_name, v.message))
            .collect();
        return Err(domain_model_error(format!(
            "semantic validation failed: {}",
            errors.join("; ")
        )));
    }

    // ── Build source_refs (sorted, all verified files) ──
    let mut source_refs: Vec<SourceRef> = source_set
        .files
        .iter()
        .map(|f| SourceRef {
            uri: f.uri.clone(),
            sha256: f.sha256.clone(),
        })
        .collect();
    source_refs.sort();

    // ── Build parse_options_sha256 (includes version + limits) ──
    let parse_options_value = serde_json::json!({
        "namespace_registry": null,
        // SUP-02: hash the normalized entry spelling so `x.sea` and
        // `./x.sea` share one semantic identity.
        "entry_path": normalize_entry_uri(&source_set.entry_uri),
        "domainforge_version": domainforge_core::VERSION,
        "limits": {
            "max_source_count": MAX_SOURCE_COUNT,
            "max_aggregate_bytes": MAX_AGGREGATE_BYTES,
            "max_import_depth": MAX_IMPORT_DEPTH,
            "max_ast_nodes": MAX_AST_NODES,
        }
    });
    let parse_options_sha256 = sha256_json(&parse_options_value)?;

    // ── Build semantic_model_sha256 over the canonical 4-tuple ──
    let canonical_tuple = serde_json::json!({
        "domainforge_version": domainforge_core::VERSION,
        "adapter_descriptor_sha256": ADAPTER_DESCRIPTOR_SHA256,
        "parse_options_sha256": parse_options_sha256,
        "source_refs": source_refs,
    });
    let semantic_model_sha256 = sha256_json(&canonical_tuple)?;

    // ── Extract concept refs from the graph (sorted) ──
    let mut concept_refs: Vec<String> = graph
        .all_entities()
        .iter()
        .map(|e| e.name().to_string())
        .collect();
    concept_refs.extend(graph.all_resources().iter().map(|r| r.name().to_string()));
    concept_refs.sort();
    concept_refs.dedup();
    let mut class_refs: Vec<String> = graph
        .all_resources()
        .iter()
        .map(|resource| resource.name().to_string())
        .collect();
    class_refs.sort();
    class_refs.dedup();

    let model_ref = DomainModelRef {
        source_refs,
        domainforge_version: domainforge_core::VERSION.to_string(),
        adapter_descriptor_sha256: ADAPTER_DESCRIPTOR_SHA256.into(),
        parse_options_sha256,
        semantic_model_sha256,
        concept_refs,
        class_refs,
        validation_evidence_refs: vec![format!(
            "validation:error_count={}",
            validation.error_count
        )],
    };

    Ok(DomainModel { graph, model_ref })
}

fn domain_model_error(message: String) -> ForgeError {
    ForgeError::Plan {
        class: "domain_model_error",
        message,
    }
}

/// Return the longest resolved import path from the unique closure root,
/// measured in edges. DomainForge emits only entry-reachable modules after it
/// has normalized relative paths and rejected cycles, so the sole module with
/// no inbound edge is the canonical entry even when the caller supplied an
/// alternate spelling such as `./entry.sea`.
fn max_import_depth(
    modules: &[CanonicalModuleRef],
    edges: &[CanonicalImportEdge],
) -> Result<usize, ForgeError> {
    let mut children: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut imported = BTreeSet::new();
    for edge in edges {
        children
            .entry(edge.importer.clone())
            .or_default()
            .push(edge.imported.clone());
        imported.insert(edge.imported.as_str());
    }
    let roots: Vec<&str> = modules
        .iter()
        .map(|module| module.logical_id.as_str())
        .filter(|module| !imported.contains(module))
        .collect();
    let [entry] = roots.as_slice() else {
        return Err(domain_model_error(format!(
            "resolved import graph must have one root, found {}",
            roots.len()
        )));
    };

    fn visit(
        module: &str,
        children: &BTreeMap<String, Vec<String>>,
        depths: &mut BTreeMap<String, usize>,
        visiting: &mut BTreeSet<String>,
    ) -> Result<usize, ForgeError> {
        if let Some(depth) = depths.get(module) {
            return Ok(*depth);
        }
        if !visiting.insert(module.to_owned()) {
            return Err(domain_model_error(format!(
                "resolved import graph contains cycle at {module}"
            )));
        }

        let mut depth = 0;
        if let Some(imports) = children.get(module) {
            for imported in imports {
                depth = depth.max(visit(imported, children, depths, visiting)? + 1);
            }
        }
        visiting.remove(module);
        depths.insert(module.to_owned(), depth);
        Ok(depth)
    }

    visit(entry, &children, &mut BTreeMap::new(), &mut BTreeSet::new())
}

/// Recursively count every AST node in a parsed module: each top-level
/// declaration, each nested declaration inside `export`, every field in a
/// record/entity body, every enum member and operation clause, every mapping
/// and projection rule, and every node in a policy/metric expression tree.
/// Leaves count as 1; containers count themselves plus their children. Used
/// by the AST node budget so nesting depth, not just declaration count,
/// determines cost.
/// Lexically bound the expression-nesting depth of a `.sea` source before it
/// reaches the recursive-descent parser. Tracks three recursion drivers in the
/// grammar — bracket depth (`(`, `[`, `{`), consecutive unary `-`, and
/// consecutive `not` — skipping string literals so a long `"-----"` string is
/// not miscounted. Returns true when any driver exceeds `MAX_NESTING_DEPTH`.
fn nesting_depth_exceeded(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut i = 0usize;
    let mut bracket_depth = 0usize;
    let mut dash_run = 0usize;
    let mut not_run = 0usize;
    let mut in_string = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            match c {
                b'\\' => i += 2,
                b'"' => {
                    in_string = false;
                    i += 1;
                }
                _ => i += 1,
            }
            continue;
        }
        match c {
            b'"' => {
                in_string = true;
                dash_run = 0;
                not_run = 0;
                i += 1;
            }
            b'(' | b'[' | b'{' => {
                bracket_depth += 1;
                if bracket_depth > MAX_NESTING_DEPTH {
                    return true;
                }
                dash_run = 0;
                not_run = 0;
                i += 1;
            }
            b')' | b']' | b'}' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                dash_run = 0;
                not_run = 0;
                i += 1;
            }
            b'-' => {
                dash_run += 1;
                if dash_run > MAX_NESTING_DEPTH {
                    return true;
                }
                not_run = 0;
                i += 1;
            }
            _ => {
                if c == b'n' && content[i..].starts_with("not") {
                    let after = i + 3;
                    let boundary = after >= bytes.len()
                        || !(bytes[after].is_ascii_alphanumeric() || bytes[after] == b'_');
                    if boundary {
                        not_run += 1;
                        if not_run > MAX_NESTING_DEPTH {
                            return true;
                        }
                        i += 3;
                        continue;
                    }
                }
                not_run = 0;
                dash_run = 0;
                i += 1;
            }
        }
    }
    false
}

fn count_ast_nodes(ast: &domainforge_core::parser::ast::Ast) -> usize {
    ast.declarations
        .iter()
        .map(|spanned| count_ast_node(&spanned.node))
        .sum::<usize>()
        + ast.metadata.imports.len()
}

fn count_ast_node(node: &domainforge_core::parser::ast::AstNode) -> usize {
    use domainforge_core::parser::ast::AstNode;
    let base = 1usize;
    match node {
        AstNode::Export(inner) => base + count_ast_node(&inner.node),
        AstNode::Entity { body, .. } => base + body.as_ref().map(|b| b.fields.len()).unwrap_or(0),
        AstNode::Record(decl) => base + decl.fields.len(),
        AstNode::Enum(decl) => base + decl.members.len(),
        AstNode::Operation(decl) => base + decl.clauses.len(),
        AstNode::Policy { expression, .. } | AstNode::Metric { expression, .. } => {
            base + count_expression_nodes(expression)
        }
        AstNode::Instance { fields, .. } => base + fields.len(),
        AstNode::MappingDecl { rules, .. } => base + rules.len(),
        AstNode::ProjectionDecl { overrides, .. } => base + overrides.len(),
        _ => base,
    }
}

fn count_expression_nodes(expr: &domainforge_core::policy::Expression) -> usize {
    use domainforge_core::policy::Expression;
    let base = 1usize;
    match expr {
        Expression::Binary { left, right, .. } => {
            base + count_expression_nodes(left) + count_expression_nodes(right)
        }
        Expression::Unary { operand, .. } | Expression::Cast { operand, .. } => {
            base + count_expression_nodes(operand)
        }
        Expression::GroupBy {
            collection,
            filter,
            key,
            condition,
            ..
        } => {
            base + count_expression_nodes(collection)
                + filter
                    .as_ref()
                    .map(|f| count_expression_nodes(f))
                    .unwrap_or(0)
                + count_expression_nodes(key)
                + count_expression_nodes(condition)
        }
        Expression::Quantifier {
            collection,
            condition,
            ..
        } => base + count_expression_nodes(collection) + count_expression_nodes(condition),
        Expression::Aggregation {
            collection, filter, ..
        } => {
            base + count_expression_nodes(collection)
                + filter
                    .as_ref()
                    .map(|f| count_expression_nodes(f))
                    .unwrap_or(0)
        }
        Expression::AggregationComprehension {
            collection,
            predicate,
            projection,
            ..
        } => {
            base + count_expression_nodes(collection)
                + count_expression_nodes(predicate)
                + count_expression_nodes(projection)
        }
        _ => base,
    }
}

/// Render `resolve_application_graph`'s diagnostics as one adapter-facing
/// message. Reason slugs are translated to the wording SEA Forge callers and
/// tests key on ("unresolved import", "circular import", ...).
fn format_diagnostics(diags: &[ApplicationDiagnostic]) -> String {
    diags
        .iter()
        .map(|d| {
            let prefix = match d.context.reason.as_deref() {
                Some("unresolved_specifier") => "unresolved import",
                Some("import_cycle") => "circular import",
                Some("not_exported") | Some("unresolved_alias") => "import error",
                Some("symbol_collision") => "duplicate declaration",
                _ => "import resolution error",
            };
            format!("{prefix}: {}", d.message)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// Normalize a DomainForge authority decision to a SEA Forge candidate disposition.
///
/// | DomainForge result | SEA Forge candidate disposition |
/// |---|---|
/// | `Reject` or `Deny` | `deny` |
/// | `Escalate` | `escalate` |
/// | `Allow` | `allow` |
/// | `NotApplicable` | no candidate; `deny` if policy requires DomainForge |
pub fn normalize_authority(raw_decision: &str) -> CandidateDisposition {
    match raw_decision {
        "Allow" => CandidateDisposition::Allow,
        "Reject" | "Deny" => CandidateDisposition::Deny,
        "Escalate" => CandidateDisposition::Escalate,
        _ => CandidateDisposition::Deny, // NotApplicable or unknown → deny-if-required
    }
}

pub fn evaluate_authority(
    model: &DomainModel,
    operation_kind: &str,
    resource_id: &str,
    evidence_refs: Vec<String>,
) -> Result<DomainForgeTrace, ForgeError> {
    if evidence_refs.is_empty() || model.model_ref.validation_evidence_refs.is_empty() {
        return Err(ForgeError::Input(
            "DomainForge authority evaluation requires validation evidence".into(),
        ));
    }
    let semantic_target = std::path::Path::new(resource_id)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|target| {
            model
                .graph
                .all_entities()
                .iter()
                .any(|entity| entity.name().eq_ignore_ascii_case(target))
                || model
                    .graph
                    .all_resources()
                    .iter()
                    .any(|resource| resource.name().eq_ignore_ascii_case(target))
        });
    let raw_decision = match (operation_kind, semantic_target) {
        ("write_file", true) => "Allow",
        ("write_file", false) => "Reject",
        ("execute_command", _) => "NotApplicable",
        _ => "NotApplicable",
    };
    Ok(DomainForgeTrace {
        raw_decision: raw_decision.into(),
        normalized_disposition: normalize_authority(raw_decision),
        // SUP-09h: this is a filename-stem approximation, not the real
        // domainforge-core authority subsystem. The evidence string must
        // describe what actually ran — an ASCII-case-insensitive stem match
        // of `resource_id` against declared entity/resource names — so a
        // durable record never asserts "evaluated validated model against
        // canonical action" until that evaluator is wired.
        reason: "DomainForge stem-heuristic authority approximation: write_file allowed iff \
                 the resource_id file stem ASCII-case-insensitively matches a declared \
                 entity/resource name; full policy evaluation not yet wired"
            .into(),
        evidence_refs,
    })
}

/// In-memory projection of a DomainModel into CALM or RDF.
///
/// Pure: no filesystem writes, no network calls, no CLI invocations.
/// Returns a sorted map of relative_path → content (§7.0a, §10.4a).
pub fn project(
    model: &DomainModel,
    kind: &ProjectionKind,
) -> Result<BTreeMap<String, String>, ForgeError> {
    match kind {
        ProjectionKind::Calm => {
            let mut calm_value = domainforge_core::calm::export(&model.graph).map_err(|e| {
                ForgeError::Internal(format!("domain_model_error: CALM projection failed: {e}"))
            })?;
            // Strip non-deterministic timestamp for byte-identical regeneration (§10.7).
            // domainforge-core adds sea:timestamp to metadata; remove it so the same
            // graph produces identical bytes across calls.
            if let Some(metadata) = calm_value
                .get_mut("metadata")
                .and_then(|m| m.as_object_mut())
            {
                metadata.remove("sea:timestamp");
            }
            let mut map = BTreeMap::new();
            let encoded = serde_json::to_vec_pretty(&calm_value)
                .map_err(|e| ForgeError::Serialization(e.to_string()))?;
            map.insert(
                "calm.json".into(),
                String::from_utf8_lossy(&encoded).into_owned(),
            );
            Ok(map)
        }
        ProjectionKind::Rdf => {
            let kg =
                domainforge_core::kg::KnowledgeGraph::from_graph(&model.graph).map_err(|e| {
                    ForgeError::Internal(format!("domain_model_error: KG build failed: {e}"))
                })?;
            let turtle = kg.to_turtle();
            let rdf_xml = kg.to_rdf_xml();
            let mut map = BTreeMap::new();
            map.insert("model.ttl".into(), turtle);
            map.insert("model.rdf".into(), rdf_xml);
            Ok(map)
        }
        ProjectionKind::Kg => {
            // M9 (E11): the self-model knowledge-graph projection. The KG is the
            // canonical Turtle serialization of the validated graph.
            let kg =
                domainforge_core::kg::KnowledgeGraph::from_graph(&model.graph).map_err(|e| {
                    ForgeError::Internal(format!("domain_model_error: KG build failed: {e}"))
                })?;
            let turtle = kg.to_turtle();
            let mut map = BTreeMap::new();
            map.insert("model.ttl".into(), turtle);
            Ok(map)
        }
        _ => Err(ForgeError::Input(format!(
            "unsupported projection kind for DomainForge adapter: {kind:?}"
        ))),
    }
}
