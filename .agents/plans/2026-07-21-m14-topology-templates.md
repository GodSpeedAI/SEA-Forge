# M14 Topology Templates Implementation Plan

**Goal:** Extend the E8 template mechanism (`crates/sea-forge-planner/src/templates.rs`)
only as much as required to instantiate a variable number of agent tasks and an
all-success rollup, then ship `sequential_agents@0.1.0` and `concurrent_agents@0.1.0`
as source-owned built-in templates.

**Grounding (read before editing):**
- `crates/sea-forge-planner/src/templates.rs` — `PlanTemplate`, `TemplateItem`,
  `TemplateOperation`, `check_substitution_sites`, `instantiate`, `store_builtin`.
- `crates/sea-forge-planner/src/case_engine.rs:71-101` — `entry_criteria_satisfied`
  (today: OR-of-sentries only), `evaluate_sentries`, `check_satisfiability`.
- `crates/sea-forge-planner/src/case_engine.rs:201-318` — `validate_proposal`
  (already enforces plan-wide unique IDs on the *expanded* `CasePlan`; repeated
  items get this check for free once flattened — do not duplicate it in
  `templates.rs`).
- `crates/sea-forge-core/src/types.rs:93-120` — `PlanItem`.
- `crates/sea-forge-core/src/types.rs:128-153` — `Operation` (has `AgentTask`;
  `TemplateOperation` currently does NOT — this is a real gap, not an oversight
  to skip).
- `crates/sea-forge-planner/tests/conformance_m10.rs` — test file shape/naming
  to mirror for `conformance_m14.rs`.
- `.agents/specs/spec-agent-orchestration.md:427-433` — exact T14.1–T14.3 wording.

## Global constraints

- Old templates remain byte-compatible: every new field is additive with
  `#[serde(default)]`; zero behavior change when unused.
- Keep the planner/case reducer synchronous and pure.
- Forbidden-substitution invariant (kind tags, executable identity, sentry
  source/event) extends to any new operation/field, it does not get bypassed.
- No new crate, no new dependency, no queue/pool.
- Stub latencies in built-in template demos prove scheduler behavior only —
  do not claim the spec §5 "real agent latencies" line is proven.

---

### Task 1: Typed deterministic item expansion (Slice 6.1)

**Files:**
- Modify: `crates/sea-forge-core/src/types.rs` (add `TemplateOperation`-adjacent
  `Operation::AgentTask` is already present; no core change needed here except
  none — confirm before editing)
- Modify: `crates/sea-forge-planner/src/templates.rs`
- Test: `crates/sea-forge-planner/tests/conformance_m14.rs` (new)

**Produces:** a `RepeatedItem` construct on `TemplatePlan` that expands a typed
list of entries into N plan items with deterministic IDs (`{id_prefix}_{key}`),
a `TemplateOperation::AgentTask` variant (closing the existing gap where
templates cannot express `Operation::AgentTask` at all), and per-entry extra
`entry_criteria` so a template author can chain entry `i` on entry `i-1`'s
settlement using the plain, already-existing sentry vocabulary — no new
"chain" flag or special-cased engine logic.

- [ ] **Step 1: Write failing expansion tests.**

```rust
// crates/sea-forge-planner/tests/conformance_m14.rs
#[test]
fn t14_0_repeated_item_expands_to_deterministic_ids() {
    let template = /* one RepeatedItem, id_prefix "branch", 3 entries keyed a/b/c */;
    let plan = instantiate(&template, &BTreeMap::new(), "case_1", "run_1", "intent_1").unwrap();
    assert_eq!(
        plan.items.iter().map(|i| i.plan_item_id.clone()).collect::<Vec<_>>(),
        vec!["branch_a", "branch_b", "branch_c"]
    );
}

#[test]
fn t14_0_duplicate_entry_keys_reject() { /* two entries with key "a" -> Err */ }

#[test]
fn t14_0_expansion_over_hard_max_rejects() { /* entries.len() > MAX_REPEATED_ENTRIES -> Err */ }

#[test]
fn t14_0_agent_task_template_operation_substitutes_instruction_not_endpoint() {
    /* TemplateOperation::AgentTask{endpoint_ref: "${ep}", ..} in a repeated item body -> load()/instantiate() rejects: endpoint_ref forbidden-substitution site */
}
```

- [ ] **Step 2: Run and confirm failure.**

Run: `cargo test -p sea-forge-planner --test conformance_m14 --offline`

Expected: FAIL — `conformance_m14.rs` does not exist / `RepeatedItem` unknown.

- [ ] **Step 3: Implement in `templates.rs`.**

```rust
pub const MAX_REPEATED_ENTRIES: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepeatEntry {
    pub key: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    #[serde(default)]
    pub entry_criteria: Vec<Sentry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepeatedItem {
    pub id_prefix: String,
    /// Shared body; `item.plan_item_id` must be `""` (sentinel — expansion
    /// derives the real ID from `id_prefix` + entry key).
    pub item: TemplateItem,
    pub entries: Vec<RepeatEntry>,
}
```

Add `TemplateOperation::AgentTask { endpoint_ref: String, instruction: String,
max_turns: u32, #[serde(default)] token_budget: Option<u64> }` mirroring
`Operation::AgentTask`'s substitutable fields (`instruction` substitutes like
`content_hint`; `endpoint_ref` is forbidden like `argv[0]` — extend
`check_substitution_sites` accordingly, and factor its per-item body into a
`fn check_item_substitution_sites(item: &TemplateItem) -> Result<(), ForgeError>`
reused for both `template.plan.items` and each `RepeatedItem.item`).

Add `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub repeated:
Vec<RepeatedItem>` to `TemplatePlan`. In `instantiate()`, after building
`items` from `template.plan.items`, expand each `RepeatedItem`:
validate `id_prefix` matches `[a-z0-9_-]+`, `item.plan_item_id == ""`,
`1 <= entries.len() <= MAX_REPEATED_ENTRIES`, entry keys match `[a-z0-9_-]+`
and are unique within the group (`ForgeError::Config{class:"schema_error",..}`
on violation, matching `check_substitution_sites`'s error style). For each
entry, merge `entry.params` over the group's already-`resolve_params`-resolved
map (entry-local override, same `substitute()` helper) and set
`entry_criteria = item.entry_criteria ++ entry.entry_criteria` (concatenation;
`entry_criteria_mode` — added in Task 2 — governs the combined list as a
single Any/All decision, no new combinator).

- [ ] **Step 4: Run the new tests plus the existing template/case-engine suite.**

Run: `cargo test -p sea-forge-planner --offline`

Expected: PASS.

- [ ] **Step 5: Commit.**

```sh
git add crates/sea-forge-planner
git commit -m "feat(planner): typed deterministic item expansion for templates"
```

---

### Task 2: All-of rollup semantics (Slice 6.2)

**Files:**
- Modify: `crates/sea-forge-core/src/types.rs` (new `EntryCriteriaMode` enum,
  additive field on `PlanItem`)
- Modify: `crates/sea-forge-planner/src/templates.rs` (additive field on
  `TemplateItem`, byte-projected to `PlanItem` in `instantiate`)
- Modify: `crates/sea-forge-planner/src/case_engine.rs`
- Test: `crates/sea-forge-planner/tests/conformance_m14.rs`

**Produces:** `entry_criteria_mode: EntryCriteriaMode` (`Any` default,
backward-compatible; `All` requires every listed sentry satisfied).

- [ ] **Step 1: Write failing all-of tests.**

```rust
#[test]
fn t14_0_any_of_legacy_behavior_unchanged() { /* two sentries, one fires -> satisfied (today's behavior, unchanged) */ }

#[test]
fn t14_0_all_of_waits_for_every_named_source() {
    /* entry_criteria_mode: All, sentries on "a" and "b"; only "a" settles accepted -> not satisfied; "b" also settles accepted -> satisfied */
}

#[test]
fn t14_0_all_of_unrelated_settlement_does_not_satisfy() { /* settlement on an unrelated item "c" never satisfies a+b All */ }

#[test]
fn t14_0_all_of_rejected_branch_never_satisfies() { /* "a" accepted, "b" rejected -> stays unsatisfied even though both fired */ }
```

- [ ] **Step 2: Run and confirm failure.**

Run: `cargo test -p sea-forge-planner --test conformance_m14 --offline`

Expected: FAIL — `entry_criteria_mode` field/enum does not exist.

- [ ] **Step 3: Implement.**

```rust
// sea-forge-core/src/types.rs
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntryCriteriaMode {
    #[default]
    Any,
    All,
}
```

Add `#[serde(default)] pub entry_criteria_mode: EntryCriteriaMode` to
`PlanItem` and to `TemplateItem`; project it byte-for-byte in `instantiate()`
next to `entry_criteria`/`exit_criteria`. In `case_engine.rs`:

```rust
fn entry_criteria_satisfied(item: &PlanItem, events: &[TraceEvent], workspace_files: &HashSet<String>) -> bool {
    if item.entry_criteria.is_empty() {
        return true;
    }
    match item.entry_criteria_mode {
        EntryCriteriaMode::Any => item.entry_criteria.iter().any(|s| sentry_satisfied(s, events, workspace_files)),
        EntryCriteriaMode::All => item.entry_criteria.iter().all(|s| sentry_satisfied(s, events, workspace_files)),
    }
}
```

No change to `check_satisfiability`'s cycle detection (mode does not affect
the sentry "on" graph) or to `required_item_failed`/`can_auto_complete` — an
unsatisfied `All` rollup simply never activates, which already reads as
"parked" (stalled, not terminated) under the existing reducer, matching the
plan's "a rejected branch parks the case" requirement with zero new
termination logic.

- [ ] **Step 4: Run the new tests plus the full planner suite.**

Run: `cargo test -p sea-forge-planner --offline`

Expected: PASS.

- [ ] **Step 5: Commit.**

```sh
git add crates/sea-forge-core crates/sea-forge-planner
git commit -m "feat(planner): all-of entry-criteria rollup mode"
```

---

### Task 3: Source-owned topology assets (Slice 6.3)

**Files:**
- Modify: `crates/sea-forge-planner/src/templates.rs`
- Test: `crates/sea-forge-planner/tests/conformance_m14.rs`

**Produces:** `sequential_agents@0.1.0` (chained `AgentTask` entries via
per-entry `entry_criteria` referencing the deterministic ID of the prior
entry) and `concurrent_agents@0.1.0` (independent `AgentTask` entries plus a
flat rollup `Milestone` item with `entry_criteria_mode: All` naming every
branch), both built through `RepeatedItem` from Task 1 and `EntryCriteriaMode`
from Task 2. Registered via `store_builtin`, matching the existing
`adlc_case_template`/`odi_adlc_case_template` precedent (no tracked files
under `.sea-forge/`, no new CLI wiring — `store_builtin` is the existing,
already-precedented installation point).

- [ ] **Step 1: Write failing T14.1–T14.3 tests (exact spec wording).**

```rust
#[test]
fn t14_1_sequential_agents_instantiation_is_deterministic_and_settles_in_order() {
    let a = instantiate(&sequential_agents_template(), &params, "case_1", "run_1", "intent_1").unwrap();
    let b = instantiate(&sequential_agents_template(), &params, "case_1", "run_1", "intent_1").unwrap();
    assert_eq!(a, b); // byte-identical, ×2 instantiation
    // chain: entry i's entry_criteria references entry i-1's computed id via settlement_status accepted
}

#[test]
fn t14_2_concurrent_agents_rollup_fires_only_when_all_n_branches_settle() {
    // N branches all settle accepted -> rollup milestone satisfied
    // fewer than N accepted -> rollup not satisfied
}

#[test]
fn t14_3_one_branch_rejected_plus_unrelated_rejection_rollup_never_fires() {
    // branch_b rejected, plus an unrelated item's rejected settlement recorded
    // -> rollup still unsatisfied; the unrelated rejection proves source-binding, not leakage
}
```

- [ ] **Step 2: Run and confirm failure.**

Run: `cargo test -p sea-forge-planner --test conformance_m14 t14_1 --offline && cargo test -p sea-forge-planner --test conformance_m14 t14_2 --offline && cargo test -p sea-forge-planner --test conformance_m14 t14_3 --offline`

Expected: FAIL — `sequential_agents_template`/`concurrent_agents_template` do
not exist.

- [ ] **Step 3: Implement the two templates and wire `store_builtin`.**

```rust
pub fn sequential_agents_template() -> PlanTemplate { /* RepeatedItem, id_prefix "step", entries chained via entries[i].entry_criteria referencing "step_{entries[i-1].key}" */ }
pub fn concurrent_agents_template() -> PlanTemplate { /* RepeatedItem, id_prefix "branch", independent entries + flat rollup Milestone with entry_criteria_mode: All */ }
```

```rust
pub fn store_builtin(root: &Path) -> Result<std::path::PathBuf, ForgeError> {
    let template = sea_model_demo_template();
    let path = store_template(root, &template)?;
    store_template(root, &adlc_case_template())?;
    store_template(root, &odi_adlc_case_template())?;
    store_template(root, &sequential_agents_template())?;
    store_template(root, &concurrent_agents_template())?;
    Ok(path)
}
```

- [ ] **Step 4: Run T14.1–T14.3 and the full planner suite.**

Run: `cargo test -p sea-forge-planner --offline`

Expected: PASS.

- [ ] **Step 5: Commit.**

```sh
git add crates/sea-forge-planner
git commit -m "feat(planner): sequential_agents and concurrent_agents built-in templates"
```

---

### Task 4: Full proof and handoff

**Files:**
- Modify: `.agents/specs/spec-agent-orchestration.md` (flip T14.1–T14.3 status
  rows to green, citing the actual test names)
- Modify: `.agents/CURRENT_STATUS.md`

- [ ] **Step 1: Format and inspect the complete diff.**

Run: `cargo fmt --all -- --check && git diff <task-1-commit>..HEAD --check`

Expected: PASS.

- [ ] **Step 2: Run the required proof suite.**

Run: `cargo test --workspace --offline && devbox run -- just context-check && devbox run -- just no-async-kernel && devbox run -- just proof`

Expected: PASS. `sea-forge-planner` and `sea-forge-core` stay kernel crates
(no async/HTTP deps introduced) — confirm `no-async-kernel`'s `kernel_crates`
array needs no addition since no new crate was created.

- [ ] **Step 3: Refresh the knowledge graph after the source commits.**

Run the repository incremental Understand Anything update and validate
`.ua/meta.json` against the final source commit.

- [ ] **Step 4: Record evidence and residual scope.**

Document the passing count and commands in `.agents/CURRENT_STATUS.md`; mark
the M14 spec row green only if T14.1–T14.3 pass. Note explicitly that stub
latencies in the built-in templates do not upgrade the spec §5 "real agent
latencies" claim — that stays open for Task 8 (M16) real integration.
