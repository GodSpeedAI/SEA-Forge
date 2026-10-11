# DomainForge Limitation Remediation Implementation Plan

**Source:** `.sea/interaction/validation/limitation-remediation-plan.md` (2026-08-12).
**Goal:** Implement all ten remediations that plan identifies, in the priority
order it establishes, with tests, in the actual DomainForge repository.

**Track progress with checkboxes.** Each phase is independently committable
and independently verifiable — do not batch phases into one commit.

## Scope and repository note

This plan's work happens almost entirely in
**`/home/sprime01/projects/domainforge`**, a separate git repository from
`sea-rs` with its own remote (`origin` = `https://github.com/GodSpeedAI/DomainForge.git`).
Only Phase 8 touches `sea-rs`.

**This reverses a constraint from the prior interaction-model mission.** That
mission's brief said "do not modify DomainForge grammar or application code" —
correct for a re-canonicalization task, and it must still hold for anyone
re-touching `.sea/interaction/`. It does not apply here: the user has now
explicitly asked for DomainForge itself to be fixed. Do not let that older
constraint block this plan.

## Global constraints

- Do not modify SEA Forge frontend, backend, API, CLI, or runtime code, and do
  not touch the 12 canonical journeys, `canonicalization-matrix.csv`, or the
  declared meaning of `interaction-model.sea`. Only its validation tooling and
  `limitations.md` change, in Phase 8.
- Every DomainForge change ships with a test in the existing
  `domainforge-core/tests/` convention (see the `application_*_tests.rs` and
  `cli_module_closure_tests.rs` files for naming and structure) and must pass
  `cargo test` before the phase is considered done.
- One phase, one commit (or a small tight series). Do not mix phases.
- **The DomainForge working tree currently holds unrelated staged changes**
  (35 files under `.claude/skills/neatcode/**`, edits to `.agents/current_state.md`
  and `.agents/next_steps.md`) alongside the RDF work this plan needs in Phase
  0. Separate them; do not fold them into this plan's commits. Ask the user
  whether those unrelated files should be committed at all — that decision
  belongs to whoever is doing that other work, not to this plan.
- **Pushing to `origin` and opening a PR against `GodSpeedAI/DomainForge` are
  visible, hard-to-reverse actions.** This plan authorizes local commits on
  local branches. It does **not** authorize push or PR creation — confirm with
  the user at that point in each phase, every time, even though this plan as a
  whole was requested.
- After any phase that touches the evaluator, CLI, or grammar, re-run this
  model's own four-check recipe from `.sea/interaction/README.md` → "Validate
  and Inspect" against `interaction-model.sea`, using a `domainforge` binary
  rebuilt from the changed worktree. A DomainForge change must not silently
  break the model it was fixed for.
- Follow existing conventions exactly: one file per subcommand in
  `domainforge-core/src/cli/`, `anyhow::Result` at the CLI boundary,
  `Result<_, String>` inside policy evaluation, `clap::Parser` derive structs
  matching `validate.rs`'s shape.

## Sequencing

Matches the remediation plan's priority order. Phases 0–1 are P0, 2–3 are P1,
4–5 are P2, 6 is P3, 7 is explicitly deferred (not implemented by this plan),
8 is SEA-Forge-side follow-through gated on the phases above.

```
0 (L1 commit)  ──┐
1 (L5 stage A) ──┼─► 2 (L4 CLI) ──► 3 (L9 partition) ──► 4 (L6 collection) ──► 5 (L3 role syntax) ──► 6 (L2 array + L5 stage B) ──► 8 (SEA-Forge follow-through)
                 │
                 └─ independent of 2-6, do first regardless
```

Phase 0 has no code dependency on anything else — do it immediately regardless
of where the rest of the plan stands. Phases 2 and 3 share `application/`
files and are easiest done in that order (2 first) since 3 needs the graph
validator to already distinguish operation-bound policies, which touches the
same call path `--application` exercises.

---

## Phase 0: Secure the RDF projection work (L1)

**Why first:** this work is complete, staged, and one `git reset --hard` away
from gone. Zero commits on the branch, no upstream. This is a data-loss risk,
not a feature request.

- [ ] **Step 1.** `cd /home/sprime01/projects/domainforge && git status` and
  confirm the working tree still matches the recorded state: `kg.rs`,
  `projection/rdf/ontology.rs`, `docs/rdf-projections.md`,
  `docs/specs/SDS-005-knowledge-graph-module.md` modified, plus
  `domainforge-core/tests/turtle_instance_export_tests.rs` new/staged.
- [ ] **Step 2.** Unstage everything with `git reset` (soft/mixed — this
  unstages, it does not discard working-tree content). Re-stage only the five
  RDF-relevant paths above with `git add <path>` individually.
- [ ] **Step 3.** Run the project's test command (check `Justfile` /
  `CONTRIBUTING.md` "Running Tests" for the exact invocation used by this
  repo) scoped to the new test file, e.g.
  `cargo test --package domainforge-core turtle_instance_export`. Confirm it
  passes before committing — do not commit a red test.
- [ ] **Step 4.** Commit on the current branch
  (`feat/rdf-instance-policy-projection` — already correctly named) with a
  message describing what the projection now preserves (instances and
  policies, not just the ontology spine).
- [ ] **Step 5.** Ask the user explicitly: push this branch to `origin` and
  open a PR now, or leave it committed locally for later? Do not push without
  that answer.
- [ ] **Step 6.** Ask the user separately whether the unrelated staged
  `.claude/skills/neatcode/**` and `.agents/*.md` changes should be committed,
  and if so, in what commit(s). This plan does not decide that.

**Done when:** the RDF work is a real commit reachable by SHA, independent of
the working tree's future state.

---

## Phase 1: L5 stage A — fail loudly on filter-evaluation errors

**Root cause**, confirmed by direct read:
`domainforge-core/src/policy/quantifier.rs:540-551`. The aggregation filter
closure discards two distinct errors:

```rust
.filter(|item| {
    let substituted = filter_expr
        .substitute(variable_name, item)
        .unwrap_or_else(|_| filter_expr.as_ref().clone());   // substitution error silently ignored
    if let Ok(expanded) = substituted.expand(graph) {
        Self::is_true_literal(&expanded)
    } else {
        false                                                 // expansion error silently becomes "no match"
    }
})
```

Arithmetic in a `where` predicate causes `expand` to fail with
`Err("Arithmetic operations not supported in boolean context")`
(`domainforge-core/src/policy/core.rs:327` and `:510`). The `else { false }`
branch turns that failure into "this row doesn't match," so every row is
silently dropped and a `count(...) = 0` invariant reports true regardless of
the actual data.

- [ ] **Step 1.** Read `evaluate_aggregation` in full
  (`quantifier.rs:512-680`) and its caller, to determine the right error type
  to propagate — likely the existing policy evaluation error type used
  elsewhere in the file (check what `evaluate_aggregation`'s `Result` already
  carries).
- [ ] **Step 2.** Change the filter step to collect into
  `Result<Vec<_>, EvalError>` (or whatever the file's existing error type is)
  instead of `Iterator::filter`, propagating both the substitution error and
  the expansion error with `?` instead of swallowing them.
- [ ] **Step 3.** Confirm this does not regress the working case: a `where`
  clause doing plain field-to-field or field-to-literal comparison (the
  documented-working case in `limitations.md` L5) must still evaluate
  normally and must not now error.
- [ ] **Step 4.** Add a regression test in a new or existing
  `domainforge-core/tests/` file — model it on
  `sea_forge_interaction_fixture_tests.rs`'s structure — asserting that a
  policy with arithmetic in its `where` predicate now **fails validation with
  an explicit error**, not a silent pass, using data that would violate the
  intended (but unevaluable) invariant.
- [ ] **Step 5.** Run the full DomainForge test suite (`cargo test`) to catch
  any existing fixture that happened to rely on the old silent-drop behavior.
  If one exists, that fixture was itself asserting a false positive — fix the
  fixture, not this change.
- [ ] **Step 6.** Rebuild `domainforge` and re-run this model's four-check
  recipe against `interaction-model.sea`. None of its sixteen policies use
  arithmetic in a `where` predicate (confirmed during the original
  remediation-plan review), so this should be a no-op for the model, and
  confirming that is the point.
- [ ] **Step 7.** Commit with a message naming this a correctness fix (silent
  false-positive on policy evaluation), not a feature.

**Done when:** the new test proves an arithmetic `where` predicate now raises
instead of silently passing, the full suite is green, and the interaction
model still validates clean.

---

## Phase 2: L4 — CLI reach into the Application Contract and Semantic Envelope

**Why:** `domainforge-core/src/application/{contract,envelope,canonical,resolve,validate,diagnostic,policy_context}.rs`
is a complete library. Zero of the eleven existing CLI subcommands
(`Parse`, `Validate`, `Import`, `Project`, `Format`, `Test`, `ValidateKg`,
`Normalize`, `Registry`, `Authority`, `Pack` — `domainforge-core/src/cli/mod.rs`)
call `resolve_application_contract` or `resolve_semantic_envelope`. This phase
is CLI wiring over existing logic, following the one-file-per-subcommand
pattern every existing subcommand already uses (`validate.rs` is the closest
model: it takes a target path, resolves a graph via
`resolve_filesystem_graph`, and reports).

**Preferred shape, in priority order:**

1. An `--application` flag on the existing `validate` subcommand that also
   resolves the Application Contract and surfaces `APP001`–`APP015`
   diagnostics through the same reporting path `validate` already uses. This
   is the one that matters most: it's what makes the operation reachable by
   the command an ordinary editor runs without learning a new subcommand.
2. New `Contract` and `Envelope` subcommands that print the resolved
   `ApplicationContractDocument` / `CanonicalSemanticEnvelopeDocument` as JSON
   — a direct CLI replacement for `application-contract-harness.rs`.

- [ ] **Step 1.** Read `application/resolve.rs`'s
  `resolve_application_contract` and `resolve_semantic_envelope` signatures
  and error types in full, and read `application/diagnostic.rs` for the
  `APP001`–`APP015` diagnostic shape, so the new CLI code reports them in the
  same `OutputFormat::{Human,Json,Lsp}` styles `validate.rs` already supports
  — do not invent a fourth output convention.
- [ ] **Step 2.** Add `--application: bool` to `ValidateArgs` in
  `cli/validate.rs`. When set, after the existing graph validation succeeds,
  additionally call `resolve_application_contract` and report any `APP*`
  diagnostics through the existing reporter, respecting `--format` and
  `--no-color`.
- [ ] **Step 3.** Create `domainforge-core/src/cli/contract.rs` and
  `domainforge-core/src/cli/envelope.rs`, each following `validate.rs`'s
  shape (a `*Args` struct with `target: PathBuf` and `--format`, a `run`
  function that resolves the filesystem graph then calls the corresponding
  `application::resolve` function and prints the canonical JSON document,
  including its hash fields).
- [ ] **Step 4.** Wire both into `cli/mod.rs`: add `pub mod contract;` /
  `pub mod envelope;`, and two new `Commands` variants
  (`Contract(contract::ContractArgs)`, `Envelope(envelope::EnvelopeArgs)`),
  matching the doc-comment style already used for each existing variant.
- [ ] **Step 5.** Add tests in `domainforge-core/tests/` — likely a new
  `cli_application_tests.rs` modeled on `cli_module_closure_tests.rs` —
  covering: (a) `validate --application` on a file with a valid operation
  passes clean; (b) `validate --application` on a file with a broken operation
  (e.g., an output field that doesn't project a compatible state field, per
  APP011) fails with the specific `APP` code; (c) `contract` and `envelope`
  subcommands produce parseable JSON matching the library's own hash
  computation.
- [ ] **Step 6.** Run `cargo test`, then rebuild and run
  `domainforge validate --application` and `domainforge contract` /
  `domainforge envelope` against `interaction-model.sea` directly, confirming
  they report the same `get_canonical_journey` operation and the same
  `semantic_closure_hash` currently recorded in
  `.sea/interaction/validation/diagnostics.md`.
- [ ] **Step 7.** Commit.

**Done when:** `domainforge validate --application` on
`interaction-model.sea` reports the operation clean through the standard
validate path, and `domainforge contract` / `domainforge envelope` reproduce
the harness's output without a hand-built binary.

---

## Phase 3: L9 — exclude operation-bound policies from graph evaluation

**Why:** DomainForge's own flagship fixture
`fixtures/application_generation/flagship/command-write.sea` currently fails
`domainforge validate` with
`Policy 'order_total_within_limit' evaluation is UNKNOWN (NULL)`, because a
policy written for an operation's typed precondition context gets evaluated
against the bare semantic graph, where its terms don't resolve. This blocks
any write operation with a precondition — which is a real gap for future
Application Contract users, though not for this model (its one operation is
`access public`, `read_only`).

- [ ] **Step 1.** Find where a policy's binding context is determined —
  whether a policy is graph-scoped versus operation-scoped. Check
  `application/policy_context.rs` first; it likely already has the concept
  this phase needs, given its name.
- [ ] **Step 2.** Determine how a policy currently declares (or should
  declare) that it belongs to an operation's precondition rather than the
  graph — this may already be implicit from how `operation_decl`'s
  precondition clauses parse into policy references
  (`sea.pest` operation grammar, ~lines 282-321), or it may need an explicit
  marker. Read before assuming either.
- [ ] **Step 3.** In the graph validator's policy-evaluation loop (find it via
  `grep -rn "Policy evaluation is UNKNOWN\|fn validate_policies\|graph.policies"`),
  skip policies identified as operation-bound. In the Application Contract
  validator (`application/validate.rs`), evaluate operation-bound policies in
  the operation's typed context instead.
- [ ] **Step 4.** Confirm DomainForge's own flagship fixture
  `fixtures/application_generation/flagship/command-write.sea` passes
  `domainforge validate` after this change — that fixture is the natural,
  pre-existing acceptance test; do not write a new one if this one now passes
  for the right reason.
- [ ] **Step 5.** Add a regression test asserting a graph-evaluable policy and
  an operation-bound policy can coexist in one file (the case that is
  currently impossible) and that each is checked in its correct context —
  i.e., corrupting the graph-evaluable one still fails validation, and
  corrupting the operation-bound one still fails contract resolution.
- [ ] **Step 6.** Run `cargo test`, rebuild, re-run this model's four checks
  (this model declares `access public` specifically to avoid needing this fix
  — confirm the fix doesn't change its behavior).
- [ ] **Step 7.** Commit.

**Done when:** `command-write.sea` validates clean, and a file mixing
graph-evaluable and operation-bound policies validates both correctly.

---

## Phase 4: L6 — typed `entity_instances of "X"` collection

**Why:** `forall`/`exists` over the untyped `entity_instances` collection
returns `UNKNOWN (NULL)` whenever a field is absent on some other entity type
in the collection, because DomainForge does not narrow the type before
projecting the field. The workaround (aggregation with a `where` filter)
works today and is not going away — this phase is a convenience for
expressing genuine universal quantification, not a correctness fix.

- [ ] **Step 1.** Read `collection` in `sea.pest` (~line "collection = { ^"entity_instances" | ... }")
  and its handling in `quantifier.rs` around the `entity_instances` branch
  noted in the earlier change review (`policy/quantifier.rs:377-407`).
- [ ] **Step 2.** Extend the grammar: `entity_instances ~ ("of" ~ string_literal)?`
  or equivalent, so `entity_instances of "CanonicalJourney"` parses.
  Match the existing quoting convention used elsewhere for entity-type name
  literals in this grammar (check how `i.entity = "CanonicalJourney"`
  currently compares against a string, and reuse that literal form rather
  than inventing an identifier form).
- [ ] **Step 3.** In the quantifier/aggregation evaluator, when the optional
  type filter is present, pre-filter the collection to instances of that
  entity type before substitution — this is exactly the filtering
  `count(... where i.entity = "X": ...)` already does, just moved earlier in
  the pipeline so a bare `forall`/`exists` (no aggregation) can use it safely
  without hitting an absent-field NULL.
- [ ] **Step 4.** Add tests confirming: (a) `forall i in entity_instances of "X": (i.field > 0)`
  evaluates correctly (not NULL) in a model with multiple entity types; (b)
  the existing untyped `entity_instances` form still works unchanged
  (backward compatibility — this is additive grammar, not a breaking change).
- [ ] **Step 5.** Run `cargo test`, rebuild, re-run this model's four checks.
  Note: do **not** rewrite any of this model's sixteen existing policies to
  use the new form as part of this phase — that's the model's call, tracked
  separately in Phase 8, not an automatic consequence of the grammar existing.
- [ ] **Step 6.** Commit.

**Done when:** typed universal/existential quantification over one entity
type parses, evaluates without spurious NULL, and the untyped form is
unaffected.

---

## Phase 5: L3 — role-to-entity binding syntax

**Why:** The graph model already supports this —
`domainforge-core/src/graph/mod.rs` has
`entity_roles: IndexMap<ConceptId, Vec<ConceptId>>` with working accessors
(`:225`, `:233`, `:237`) and serialization (`:833`, `:873`). Only the text
surface can't author it: the existing `in <identifier>` clause on an entity
declaration sets a namespace/domain, not a role binding, so `entity_roles` is
always empty coming out of the parser. This is parser-and-lowering work over
an existing data structure, not new graph design — but the syntax itself
needs deliberate design so it carries cardinality and responsibility kind, or
it won't be useful for what motivated it (CMMN `performerRef`-style
projections).

- [ ] **Step 1.** Read every existing call site of `entity_roles` in
  `graph/mod.rs` and any consumer (grep the whole tree for
  `entity_roles` and `add_entity_role` or equivalent) to understand exactly
  what shape of data the graph expects — cardinality, direction (role owns
  entities vs. entity claims roles), and whether a "responsibility kind" field
  already exists anywhere nearby that this syntax should populate.
- [ ] **Step 2.** Design the concrete syntax before writing grammar. Candidate,
  subject to what Step 1 reveals: a clause on the entity declaration,
  `entity "Case" { ... } performed_by role<Approver> as "reviewer"` or a
  standalone `role_binding` declaration alongside `relation` — prefer whichever
  shape lets responsibility kind and cardinality be expressed without
  overloading `in`. Do not reuse `in <identifier>` for this; that clause's
  meaning (namespace) is settled and should not become overloaded.
- [ ] **Step 3.** Add the grammar rule to `sea.pest`, following the structure
  of `role_decl` (`sea.pest:164`) and `relation` declarations as the nearest
  precedents.
- [ ] **Step 4.** Lower the new syntax into `graph.entity_roles` in the
  parser/graph-builder path (find where `role_decl` and `relation_decl`
  currently populate the graph, and add the equivalent call for the new
  construct).
- [ ] **Step 5.** Add tests: parsing produces a non-empty `entity_roles` map
  with the right cardinality; an entity bound to a role that doesn't exist is
  rejected (mirroring the dangling-reference rejection pattern already
  established for `ref<Entity>` in `entity_validation.rs`).
- [ ] **Step 6.** Run `cargo test`, rebuild, re-run this model's four checks.
  This model currently expresses role responsibility only through prose and
  three role-to-role `relation` declarations (`README.md`, L3) — do not
  retrofit the model to use the new syntax as part of this phase; that's a
  Phase 8 decision, not automatic.
- [ ] **Step 7.** Commit.

**Done when:** an entity-to-role binding is authorable in `.sea` text and
appears in `graph.entity_roles`, with a dangling-role negative test passing.

---

## Phase 6: L2 array literals, and L5 stage B (arithmetic in boolean context)

Two independent grammar/evaluator completions, grouped because both are
"finish what stage A left" work and both are lower urgency than Phases 1-5.

### 6a — array literal in `expression`

**Why:** `literal` (`sea.pest:444`) has no array form; `string_array` exists
only for annotations and policy metadata. A declared `list<T>` field on an
instantiable entity currently cannot be given a value by any instance,
**with no diagnostic** — the entity is silently uninstantiable for that field.

- [ ] **Step 1.** Add an array-literal rule to `expression`/`literal` in
  `sea.pest`, e.g. `array_literal = { "[" ~ (expression ~ ","?)* ~ "]" }`,
  reusing `expression` recursively so it can hold literals or (later)
  references, matching how `string_array` is already structured for its
  narrower use.
- [ ] **Step 2.** Lower it in the instance-field evaluator
  (`entity_validation.rs`) to populate `list<T>` and `list<ref<T>>` fields,
  applying the existing `min_items`/`max_items` constraints and, for
  `list<ref<T>>`, the existing dangling-reference check per element.
- [ ] **Step 3.** As a safety net for models that predate this fix: add a
  diagnostic (new `APP`-family or graph-level code) when a required `list<T>`
  field is declared on an instantiable entity in a file that also declares
  zero instances providing it — i.e., don't leave the old silent trap
  reachable by omission once the fix to avoid it exists.
- [ ] **Step 4.** Tests: an instance supplying `["CJ01", "CJ02"]` to a
  `list<ref<CanonicalJourney>>` field validates and each element's reference
  is checked; a list violating `min_items`/`max_items` is rejected; a
  dangling reference inside the list is rejected.
- [ ] **Step 5.** Run `cargo test`, rebuild, re-run this model's four checks.
  Do **not** revert `InterfaceBinding` back into a list-valued
  `InterfaceProjection` as part of this phase — the normalized form is the
  better model regardless of this fix, per `handoff.md`. That reversion is
  explicitly out of scope, here and in Phase 8.
- [ ] **Step 6.** Commit.

### 6b — arithmetic in boolean-context expressions

**Why:** `policy/core.rs:327` and `:510` return
`Err("Arithmetic operations not supported in boolean context")` whenever a
`+`/`-`/`*`/`/` appears where a boolean is expected — e.g., inside a `where`
predicate. Phase 1 made that error surface instead of vanish; this phase
makes the expression actually evaluate, which is what would let a per-row
arithmetic invariant (like this model's maturity-sum check, currently living
in `reconcile.py`) move into SEA directly.

- [ ] **Step 1.** At both error sites in `core.rs`, replace the `Err(...)`
  with: evaluate the arithmetic subexpression to a numeric value via the
  existing `get_runtime_value`/numeric-evaluation path used elsewhere in the
  same match arms (the `GreaterThan`/`LessThan` arms already do numeric
  comparison — the pattern exists two arms up), then require the enclosing
  comparison operator to consume that numeric result the same way it already
  consumes any other numeric expression.
- [ ] **Step 2.** Confirm this doesn't require restructuring the match — a
  `Plus`/`Minus`/`Multiply`/`Divide` node should simply be a valid operand
  wherever a numeric comparison already accepts `get_runtime_value`'s output;
  if the current match structure treats `Binary` boolean-context evaluation
  and numeric-value evaluation as fully separate code paths, this may need
  `compare_numeric` (used by the `GreaterThan` family) to recurse through
  arithmetic nodes rather than assuming its operands are always leaves — read
  `compare_numeric`'s implementation before assuming the fix is purely local
  to the two `Err(...)` lines.
- [ ] **Step 3.** Tests: a `where` predicate with `i.a + i.b != i.total` over
  data that violates it now correctly fails the policy (this is the direct
  regression test for the original L5 symptom, now proven at the "evaluates
  correctly" level rather than just the "fails loudly" level from Phase 1).
- [ ] **Step 4.** Run `cargo test`, rebuild, re-run this model's four checks.
- [ ] **Step 5.** Commit.

**Done when both 6a and 6b are done:** an array literal instantiates a
`list<T>` field with full constraint checking, and arithmetic inside a
`where` predicate evaluates to the correct boolean rather than erroring or
silently dropping rows.

---

## Explicitly not in this plan

- **L7 — typed transition graphs.** ADR-scale language design touching BPMN,
  CMMN, TLA+, and event projections. The remediation plan calls for a design
  document before any patch; this implementation plan does not attempt one.
  If the user wants this pursued, it needs its own planning pass, not a
  phase bolted onto this one.
- **L8 — per-journey step emphasis.** No action was recommended; none is
  taken here.
- **L10 — key must be `string`/`uuid`.** Closed as not-a-defect (APP005, by
  design). No action.

---

## Phase 8: SEA-Forge-side follow-through (gated, incremental)

Each step here is gated on its corresponding DomainForge phase actually
landing (merged, and ideally released as a version bump) — do not do these
preemptively against an unmerged local build.

- [ ] **After Phase 0 lands (merged):** update the L1 table in
  `.sea/interaction/validation/domainforge-change-review.md` and the L1 entry
  in `limitations.md` from "unreleased" to reflect actual release status;
  update `README.md`'s "Known Toolchain Boundaries" section accordingly.
- [ ] **After Phase 1 lands:** no model change needed (no policy in this model
  uses arithmetic in `where`) — just move L5 from "Current limitations" to
  "Resolved since the previous report" in `limitations.md`, citing the new
  test as proof, same as the existing resolved-limitation rows do.
- [ ] **After Phase 2 lands:** replace steps 4-5 of the four-check recipe in
  `README.md` and `handoff.md` with `domainforge contract` /
  `domainforge envelope` (or `validate --application`); delete
  `validation/application-contract-harness.rs`; retire limitation L4.
- [ ] **After Phase 3 lands:** no model change needed (this model already
  uses `access public` specifically to sidestep L9) — move L9 to resolved,
  noting the model's own workaround was never load-bearing beyond that one
  declaration choice.
- [ ] **After Phase 4 lands:** evaluate, don't automatically adopt — decide
  per-invariant whether `entity_instances of "X"` reads better than the
  existing `count(... where ...)` form for this model's sixteen policies. Not
  required; L6 becomes a convenience note, not a rewrite mandate.
- [ ] **After Phase 5 lands:** evaluate whether to replace this model's three
  prose/relation-based role expressions with typed bindings. Only do this if
  it strictly increases what's provable — do not adopt new syntax for its own
  sake, per the original mission's governing principle (smallest sufficient
  semantic world).
- [ ] **After Phase 6a lands:** no reversion of `InterfaceBinding` — confirmed
  out of scope above. Just retire the L2 entry.
- [ ] **After Phase 6b lands:** consider moving the maturity-sum check from
  `reconcile.py` into a SEA policy. `reconcile.py` stays regardless — it also
  does the CSV cross-check DomainForge will never be able to do — but the one
  arithmetic invariant could become native. Evaluate on its merits; not
  mandatory.
- [ ] **Independent of all of the above, do anytime:** collapse the
  four-manual-command validation recipe into one
  `.sea/interaction/validation/check-all.sh` returning a single exit status,
  so "all four must pass" is enforceable (and CI-able) rather than
  aspirational. This was flagged in the remediation plan as SEA-Forge's one
  unconditional action item.

---

## Definition of done for this plan

- Phases 0-6 each have their own commit(s) in the DomainForge repo, each with
  a passing test proving the specific defect is fixed, and `cargo test` green
  after each.
- No push or PR happened without an explicit per-branch confirmation from the
  user at that point.
- This model's four-check recipe still passes, unchanged in outcome, after
  every phase.
- `limitations.md` and `domainforge-change-review.md` are updated only as
  each corresponding phase actually lands (Phase 8), not preemptively.
- L7, L8, and L10 remain untouched, as decided.
