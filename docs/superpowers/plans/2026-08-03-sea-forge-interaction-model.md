# SEA Forge Interaction Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Account for all 128 governed Workbench stories in a validated SEA interaction model built from 12 canonical human-intention journeys.

**Architecture:** Keep one import-root model at `.sea/interaction/interaction-model.sea` and split ontology, canonical journeys, variant vocabulary, and interface bindings into same-namespace SEA modules. Keep exhaustive row-level evidence in CSV and explain the stable model in focused Markdown artifacts; use DomainForge 0.15.0 to validate, parse, and inspect only semantically useful projections.

**Tech Stack:** SEA DSL, DomainForge CLI 0.15.0, Markdown, CSV, repository shell tools.

## Global Constraints

- Treat the 128 `X.Y` stories in `.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md` as the complete observed catalog.
- Preserve human intention, state transition, capability, artifact, evidence, completion condition, and next decision when canonicalizing.
- Use only `canonical`, `specialization`, `variant`, `composition`, `duplicate`, `obsolete`, `ambiguous`, or `unsupported` for classification.
- Track `declared`, `specified`, `implemented`, `exercised`, and `evidenced` maturity separately from classification.
- Keep `.sea/interaction/` as the only durable model source; `.agents/reports/` may summarize and link but must not duplicate the model.
- Do not change the SEA grammar, DomainForge, SEA Forge application code, dependencies, persisted schemas, public interfaces, CI, or deployment.
- Do not build the Journey & Capability Map.
- Preserve unrelated worktree changes and secrets.
- Skip `just context-check` by owner instruction and report it as skipped; do not claim that gate.
- Use `/home/sprime01/projects/domainforge/target/debug/domainforge`, built from `/home/sprime01/projects/domainforge` with `devbox run -- cargo run --locked -p domainforge-core --features cli --bin domainforge -- --version`.

## File Structure

- `.sea/interaction/interaction-model.sea` — sole validation/import entry point.
- `.sea/interaction/interaction-domain.sea` — interaction ontology, actor roles, resources, reusable steps, relations, and enforceable graph invariants.
- `.sea/interaction/canonical-journeys.sea` — 12 canonical journey instances and their entry-to-completion flows.
- `.sea/interaction/journey-variants.sea` — classification and recurring variation-dimension instances.
- `.sea/interaction/interaction-projections.sea` — UI, API, CLI, and agent bindings represented as ordinary concepts.
- `.sea/interaction/canonicalization-matrix.csv` — exhaustive 128-row mapping and evidence ledger.
- `.sea/interaction/canonical-journey-catalog.md` — full human-readable specification of each canonical journey.
- `.sea/interaction/journey-grammar.md` — fundamental verbs, nouns, composition rules, and projection boundary.
- `.sea/interaction/coverage-report.md` — reconciled compression, classification, maturity, confidence, and variation counts.
- `.sea/interaction/README.md` — source authority, entry point, artifact index, validation instructions, and next-agent handoff.
- `.sea/interaction/validation/domainforge-output.txt` — raw concise version/help/parse/validate/project output.
- `.sea/interaction/validation/diagnostics.md` — commands, results, revisions, output inspection, and skipped checks.
- `.sea/interaction/validation/limitations.md` — recurring grammar limitations and justified future directions.
- `.agents/reports/2026-08-03-sea-forge-interaction-model.md` — consolidated completion report linking the canonical artifacts.
- `.agents/CURRENT_STATUS.md` — short resumable handoff appended without rewriting unrelated active-work entries.

---

### Task 1: Build the Exhaustive Evidence and Canonicalization Matrix

**Files:**
- Create: `.sea/interaction/canonicalization-matrix.csv`

**Interfaces:**
- Consumes: all 128 story IDs and prose from `.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md` plus repository implementation and evidence sources.
- Produces: exactly one CSV row per observed ID with columns `Observed Journey ID`, `Observed Journey`, `Canonical Journey ID`, `Canonical Journey`, `Classification`, `Variation Dimensions`, `Evidence`, `Confidence`, `Notes`, and `Implementation Maturity`.

- [ ] **Step 1: Extract and count the source catalog without creating a competing source**

Run:

```bash
awk '/^### [0-9]+\.[0-9]+ / {count++; id=$2; sub(/^### [0-9]+\.[0-9]+ /, ""); print id "\t" $0} END {print "COUNT\t" count}' \
  .agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md
```

Expected: 128 unique `X.Y` entries and final line `COUNT 128`.

- [ ] **Step 2: Classify every row using the normalized semantic tuple**

For each story, compare:

```text
actor | intention | initiating condition | state transition | capability
| artifact | evidence | completion condition | next decision
```

Use these canonical IDs exactly unless the evidence proves the approved design needs revision:

```text
CJ01 establish_trusted_cell_context
CJ02 discover_lawful_affordances
CJ03 ground_work_in_semantic_meaning
CJ04 form_and_commit_governed_case
CJ05 navigate_and_adapt_live_case
CJ06 resolve_human_judgment_and_approval
CJ07 execute_governed_work
CJ08 monitor_intervene_and_recover
CJ09 evaluate_settle_and_audit_outcomes
CJ10 reuse_demonstrated_knowledge_and_capability
CJ11 transform_and_mature_governed_artifacts
CJ12 transfer_and_adopt_governed_assets
```

Quote CSV fields containing commas. Use semicolon-separated values inside `Variation Dimensions` and `Evidence` so one row remains one record. Use `high`, `medium`, or `low` confidence.

- [ ] **Step 3: Ground classification and maturity in repository evidence**

Use direct file references with section, method, route, type, or test names. At minimum correlate the UX epic with:

```text
.agents/specs/spec-minimum.md
.agents/specs/spec-full.md
.agents/specs/frontend/sea-forge-workbench-api-method-catalog-v0.1.yaml
crates/sea-forge-server/src/sfwp/mod.rs
workbench/apps/desktop/src/router.tsx
docs/execution/ARCHITECTURAL_TRUTH.md
docs/execution/REPOSITORY_TRUTH.md
docs/execution/USER_JOURNEY_EVIDENCE.md
```

Do not infer `implemented`, `exercised`, or `evidenced` from the epic alone.

- [ ] **Step 4: Verify row identity, vocabulary, and required fields**

Run:

```bash
python3 - <<'PY'
import csv
from collections import Counter
from pathlib import Path

path = Path('.sea/interaction/canonicalization-matrix.csv')
rows = list(csv.DictReader(path.open(newline='', encoding='utf-8')))
required = {
    'Observed Journey ID', 'Observed Journey', 'Canonical Journey ID',
    'Canonical Journey', 'Classification', 'Variation Dimensions',
    'Evidence', 'Confidence', 'Notes', 'Implementation Maturity',
}
allowed_classifications = {
    'canonical', 'specialization', 'variant', 'composition', 'duplicate',
    'obsolete', 'ambiguous', 'unsupported',
}
allowed_maturity = {'declared', 'specified', 'implemented', 'exercised', 'evidenced'}
assert set(rows[0]) == required, set(rows[0])
assert len(rows) == 128, len(rows)
ids = [row['Observed Journey ID'] for row in rows]
assert len(set(ids)) == 128, [item for item, n in Counter(ids).items() if n != 1]
assert all(row['Classification'] in allowed_classifications for row in rows)
assert all(row['Implementation Maturity'] in allowed_maturity for row in rows)
assert all(row['Canonical Journey ID'].startswith('CJ') for row in rows)
assert all(all(row[column].strip() for column in required) for row in rows)
print('matrix: 128 unique observed journeys; vocabularies and required fields valid')
PY
```

Expected: `matrix: 128 unique observed journeys; vocabularies and required fields valid`.

- [ ] **Step 5: Commit the complete evidence matrix**

```bash
git add .sea/interaction/canonicalization-matrix.csv
git commit -m "docs(interaction): classify observed SEA Forge journeys"
```

---

### Task 2: Specify the Canonical Catalog and Journey Grammar

**Files:**
- Create: `.sea/interaction/canonical-journey-catalog.md`
- Create: `.sea/interaction/journey-grammar.md`
- Create: `.sea/interaction/coverage-report.md`

**Interfaces:**
- Consumes: the 128 validated rows from Task 1.
- Produces: one full catalog entry per `CJ01`-`CJ12`, a stable interaction grammar, and reconciled compression/count evidence.

- [ ] **Step 1: Write all 12 canonical journey entries**

For each entry, use these exact fields:

```markdown
## CJ01 — Establish Trusted Cell Context

- User intention:
- User job:
- Initiating condition:
- Preconditions:
- Entry points:
- Reusable steps:
- Invoked capabilities:
- State transitions:
- Artifacts:
- Evidence:
- Completion condition:
- Next decisions:
- Recovery paths:
- Known variants:
- Implementation maturity:
- Observed story coverage:
```

Populate every field from the matrix and cited sources. `Observed story coverage`
must list the mapped `X.Y` IDs, not a count alone.

- [ ] **Step 2: Write the interaction grammar**

Define the fundamental verbs, nouns, canonical concepts, projection-only
concepts, composition rules, and recurring variation dimensions. Preserve this
common skeleton:

```text
intention -> context -> authority/availability -> capability -> state/artifact
-> evidence/limitations -> settlement/decision -> next lawful affordance
```

Explain why Thoth and maintenance are compositions and why identity and
authority are cross-cutting steps.

- [ ] **Step 3: Generate and reconcile coverage statistics**

Calculate observed count, canonical count, classification counts, maturity
counts, confidence counts, unresolved IDs, coverage percentage, compression
ratio, and most common variation dimensions. Define explained coverage as a row
with a valid `CJ01`-`CJ12` mapping even when its classification is `ambiguous`
or `unsupported`; list those unresolved rows separately so the percentage does
not hide uncertainty.

- [ ] **Step 4: Verify catalog-to-matrix agreement**

Run:

```bash
for id in $(seq -w 1 12); do
  rg -q "^## CJ${id} —" .sea/interaction/canonical-journey-catalog.md || exit 1
done
python3 - <<'PY'
import csv
from collections import Counter

rows = list(csv.DictReader(open('.sea/interaction/canonicalization-matrix.csv', encoding='utf-8')))
counts = Counter(row['Canonical Journey ID'] for row in rows)
assert set(counts) == {f'CJ{i:02d}' for i in range(1, 13)}, counts
assert sum(counts.values()) == 128
print('catalog coverage: CJ01-CJ12 present; mapped rows = 128')
PY
```

Expected: `catalog coverage: CJ01-CJ12 present; mapped rows = 128`.

- [ ] **Step 5: Commit the canonical analysis**

```bash
git add .sea/interaction/canonical-journey-catalog.md \
  .sea/interaction/journey-grammar.md \
  .sea/interaction/coverage-report.md
git commit -m "docs(interaction): define canonical journey grammar"
```

---

### Task 3: Implement the SEA Ontology and Module Boundary

**Files:**
- Create: `.sea/interaction/interaction-domain.sea`
- Create: `.sea/interaction/interaction-model.sea`

**Interfaces:**
- Produces: exported ontology symbols in namespace `sea_forge.interaction` and one canonical import root.
- Consumed by: Tasks 4 and 5 through relative named or wildcard imports.

- [ ] **Step 1: Prove the current relative-import pattern in a temporary directory**

Create a minimal two-module fixture under a `mktemp -d` directory using
`@namespace "sea_forge.interaction"`, one exported entity, one relative import,
and one instance of that entity. Validate the importing file with:

```bash
/home/sprime01/projects/domainforge/target/debug/domainforge \
  validate --format human --no-color "$fixture_dir/model.sea"
```

Expected: `Validation succeeded: 0 violations total`. Remove only the validated
temporary directory after recording the result.

- [ ] **Step 2: Declare the stable interaction ontology**

Define and export these concepts in `interaction-domain.sea`:

```text
Roles: Operator, CaseOwner, Approver, DomainAuthor, AgentSponsor,
       Auditor, CellAdministrator, ExternalActor, AutomatedActor
Entities: ProductPurpose, UserIntent, CanonicalJourney, JourneyVariant,
          JourneyStep, UserAction, InteractionState, SystemCapability,
          Artifact, Evidence, Decision, RecoveryPath, InterfaceProjection
Resources: InteractionProgress, GovernedEvidence, GovernedArtifact,
           DecisionContext, LawfulAffordance
```

Add role relations only where SEA's role-level relation semantics are honest:
`Sponsorship`, `ApprovalJudgment`, and `AuditReview`. Add reusable step instances
for context selection, preflight, authority resolution, capability invocation,
evidence inspection, settlement/decision, and next-affordance selection.

- [ ] **Step 3: Add only enforceable graph invariants**

Use SEA policies for invariants that the current graph evaluator can test, such
as the presence of flows and evidence resources. Keep instance-reference,
variant-cardinality, recovery, and terminal-decision constraints out of policy
syntax when DomainForge cannot evaluate them.

- [ ] **Step 4: Make `interaction-model.sea` the only import root**

Use relative wildcard imports with aliases for the four modules. The root file
contains metadata and imports only; it does not redeclare ontology or journey
concepts.

- [ ] **Step 5: Validate both the ontology and entry root**

Run:

```bash
df=/home/sprime01/projects/domainforge/target/debug/domainforge
"$df" validate --format human --no-color .sea/interaction/interaction-domain.sea
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea
```

Expected: both commands report zero validation errors. The entry-root command
may remain incomplete until Tasks 4 and 5 create its imported modules; if so,
commit Task 3 only after adding syntactically valid empty modules with namespace
metadata and comments, then replace those modules in Tasks 4 and 5.

- [ ] **Step 6: Commit the model boundary**

```bash
git add .sea/interaction/interaction-domain.sea \
  .sea/interaction/interaction-model.sea \
  .sea/interaction/canonical-journeys.sea \
  .sea/interaction/journey-variants.sea \
  .sea/interaction/interaction-projections.sea
git commit -m "feat(interaction): establish SEA model ontology"
```

---

### Task 4: Encode Canonical Journeys and Variation Semantics

**Files:**
- Modify: `.sea/interaction/canonical-journeys.sea`
- Modify: `.sea/interaction/journey-variants.sea`

**Interfaces:**
- Consumes: exported ontology symbols from `interaction-domain.sea` and the exact `CJ01`-`CJ12` catalog from Task 2.
- Produces: 12 canonical journey instances, end-to-end transition flows, and reusable classification/variation instances.

- [ ] **Step 1: Encode all 12 canonical journeys as instances**

Each `CanonicalJourney` instance must carry stable string fields for `journey_id`,
`name`, `intention`, `user_job`, `initiating_condition`, `preconditions`,
`entry_points`, `reusable_steps`, `capabilities`, `state_transition`,
`artifacts`, `evidence`, `completion_condition`, `next_decisions`,
`recovery_paths`, `known_variants`, and `implementation_maturity`.

Use identifiers `cj01_establish_trusted_cell_context` through
`cj12_transfer_and_adopt_governed_assets`.

- [ ] **Step 2: Encode one stable transformation flow per canonical journey**

Declare unique entry and completion state entities for each journey so flow
identity cannot collide. Use `InteractionProgress` as the resource and annotate
each flow with its canonical ID, action sequence, capabilities, evidence,
completion condition, recovery, and next decision. The flow expresses the
stable end-to-end transformation; the reusable step instances express the
shared internal grammar.

- [ ] **Step 3: Encode classification and recurring variation dimensions**

Create `JourneyVariant` instances for the eight required classifications and
for each variation dimension that occurs across materially different catalog
areas. Do not create 128 SEA instances; the CSV remains exhaustive.

- [ ] **Step 4: Validate semantic references after each module change**

Run:

```bash
df=/home/sprime01/projects/domainforge/target/debug/domainforge
"$df" validate --format human --no-color .sea/interaction/canonical-journeys.sea
"$df" validate --format human --no-color .sea/interaction/journey-variants.sea
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea
```

Expected: all three commands succeed with zero errors.

- [ ] **Step 5: Inspect the AST for instance and flow preservation**

Run:

```bash
/home/sprime01/projects/domainforge/target/debug/domainforge \
  parse --ast --format json .sea/interaction/interaction-model.sea \
  > /tmp/sea-forge-interaction.ast.json
rg -c 'cj[0-9][0-9]_' /tmp/sea-forge-interaction.ast.json
rg -c 'InteractionProgress' /tmp/sea-forge-interaction.ast.json
```

Expected: every `CJ01`-`CJ12` instance identifier appears and the transition
resource survives parsing. Inspect the JSON structure before deleting the
temporary AST.

- [ ] **Step 6: Commit canonical SEA semantics**

```bash
git add .sea/interaction/canonical-journeys.sea \
  .sea/interaction/journey-variants.sea
git commit -m "feat(interaction): model canonical journeys and variants"
```

---

### Task 5: Bind Interface Projections and Document the Canonical Source

**Files:**
- Modify: `.sea/interaction/interaction-projections.sea`
- Create: `.sea/interaction/README.md`

**Interfaces:**
- Consumes: canonical journey and step IDs plus repository UI routes, SFWP methods, CLI commands, and agent surfaces.
- Produces: ordinary `InterfaceProjection` instances for web UI, API, CLI, and agent bindings; documents that these are bindings rather than canonical identities.

- [ ] **Step 1: Encode interface kinds and evidence-backed bindings**

Create projection instances with fields for `interface_kind`, `canonical_journey_ids`,
`entry_or_method`, `binding_role`, `implementation_maturity`, `source_ref`, and
`limitations`. Include the 17 Workbench routes, implemented SFWP method families,
major CLI entry points, and Thoth/delegation agent surfaces at the semantic
family level rather than one instance per button.

- [ ] **Step 2: Preserve unavailable and preview-only distinctions**

Use source-backed maturity and limitation fields. Do not label a route as an
implemented journey when it renders an unavailable or specification-preview
state. Record catalog/implementation naming drift such as target versus
implemented SFWP names in `limitations` rather than silently normalizing it.

- [ ] **Step 3: Write the canonical-source README and handoff**

Document:

```text
purpose and source authority
why `.sea/interaction/` is canonical
entry point and module responsibilities
how the CSV maps all 128 observed stories
how to validate and inspect the model
what UI/API/CLI/agent bindings mean
how the next Journey & Capability Map agent should consume each artifact
```

- [ ] **Step 4: Validate the complete import root again**

Run:

```bash
/home/sprime01/projects/domainforge/target/debug/domainforge \
  validate --format human --no-color .sea/interaction/interaction-model.sea
```

Expected: zero errors and no unresolved imports.

- [ ] **Step 5: Commit interface bindings and source guidance**

```bash
git add .sea/interaction/interaction-projections.sea .sea/interaction/README.md
git commit -m "docs(interaction): bind interface projections"
```

---

### Task 6: Exercise DomainForge and Record Limitations

**Files:**
- Create: `.sea/interaction/validation/domainforge-output.txt`
- Create: `.sea/interaction/validation/diagnostics.md`
- Create: `.sea/interaction/validation/limitations.md`

**Interfaces:**
- Consumes: complete `interaction-model.sea` and DomainForge CLI 0.15.0.
- Produces: reproducible validation evidence, inspected projection results, and recurring grammar limitations.

- [ ] **Step 1: Capture concise tool and command evidence**

Record exact output from:

```bash
df=/home/sprime01/projects/domainforge/target/debug/domainforge
"$df" --version
"$df" validate --help
"$df" parse --help
"$df" project --help
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea
"$df" validate --format json --no-color .sea/interaction/interaction-model.sea
```

Keep raw output concise; summarize long projection help while preserving the
complete supported format list.

- [ ] **Step 2: Parse and inspect the canonical AST**

Run `parse --ast --format json`, inspect counts and representative fields, and
record whether imports, instances, flow annotations, policies, and interface
bindings survived. Keep the AST in `/tmp`; do not commit a large generated AST.

- [ ] **Step 3: Generate and inspect the RDF projection**

Project with a fixed timestamp:

```bash
out_dir=$(mktemp -d)
"$df" project --format rdf \
  --created-at 2026-08-03T00:00:00Z \
  .sea/interaction/interaction-model.sea "$out_dir/rdf"
find "$out_dir/rdf" -maxdepth 2 -type f -print -exec wc -c {} \;
```

Inspect representative journey, role, resource, relation, and flow identities
in Turtle/JSON-LD/OWL. Record semantic losses. Remove only the resolved
temporary output directory afterward.

- [ ] **Step 4: Decide whether a CMMN projection is honest**

Use the current documented mapping: entities become human tasks, roles become
case roles, resources become case file items, and policies become milestones.
Generate CMMN only if that mapping adds truthful evidence about the interaction
model. If it distorts ontology entities into executable tasks, record the
reason for skipping it; do not commit misleading output.

- [ ] **Step 5: Write the limitation assessment**

For every limitation, include `interaction concept`, `desired semantics`,
`current workaround`, `cost`, `local or recurring`, `evidence for first-class
support`, and `possible future direction`. At minimum assess:

```text
first-class journey/step/composition identity
instance-to-instance typed relations and references
ordered steps and conditional/recovery transitions
entry/completion conditions and terminal decisions
enforceable cardinality/integrity policies over instance fields
UI/API/CLI/agent projection targets
role-to-entity binding
classification and maturity vocabularies
```

Recommend future grammar work only when recurrence, enforceable invariants,
multi-projection value, and poor current representation all hold.

- [ ] **Step 6: Commit validation evidence**

```bash
git add .sea/interaction/validation/domainforge-output.txt \
  .sea/interaction/validation/diagnostics.md \
  .sea/interaction/validation/limitations.md
git commit -m "docs(interaction): record DomainForge validation"
```

---

### Task 7: Review, Reconcile, and Hand Off

**Files:**
- Create: `.agents/reports/2026-08-03-sea-forge-interaction-model.md`
- Modify: `.agents/CURRENT_STATUS.md`
- Modify as corrections require: `.sea/interaction/*`

**Interfaces:**
- Consumes: all model, matrix, catalog, coverage, and validation artifacts.
- Produces: one evidence-backed completion report and resumable project handoff.

- [ ] **Step 1: Reconcile all counts and cross-references**

Re-run the Task 1 and Task 2 checks. Confirm every CSV canonical ID names a
catalog entry and SEA instance, every interface binding names an existing
canonical journey, and coverage totals sum to 128. Search for forbidden empty
markers:

```bash
rg -n 'T[B]D|T[O]DO|F[I]XME|place[Hh]older' \
  .sea/interaction .agents/reports/2026-08-03-sea-forge-interaction-model.md
```

Expected: no matches.

- [ ] **Step 2: Review semantic quality and scope**

Review diffs for false collapsing, duplicated canonical semantics, route-shaped
journeys, invented implementation claims, decorative policies, invalid SEA,
unexplained ambiguity, unsupported grammar, secrets, and accidental application
changes. Correct the source artifacts before reporting completion.

- [ ] **Step 3: Write the consolidated report**

Report the 128-row source, final canonical count, classification and maturity
counts, compression and coverage, most common dimensions, unresolved issues,
DomainForge version and results, meaningful projection inspection, limitations,
changed files, skipped `just context-check`, and next-agent handoff. Link to the
canonical `.sea/interaction/` artifacts rather than copying them.

End with exactly one required stability statement from the mission.

- [ ] **Step 4: Update the durable project handoff carefully**

Append a compact dated section to `.agents/CURRENT_STATUS.md` with objective,
branch/worktree, files, verification, remaining issues, and decisions. Preserve
all unrelated status edits. Record `just context-check` as owner-skipped.

- [ ] **Step 5: Run final applicable checks**

Run:

```bash
df=/home/sprime01/projects/domainforge/target/debug/domainforge
"$df" validate --format human --no-color .sea/interaction/interaction-model.sea
"$df" parse --ast --format json .sea/interaction/interaction-model.sea > /tmp/sea-forge-interaction.ast.json
python3 -m json.tool /tmp/sea-forge-interaction.ast.json >/dev/null
git diff --check
git status --short
```

Expected: DomainForge validation succeeds, AST output is valid JSON, and
`git diff --check` reports no whitespace errors. Do not run `just context-check`.
Report unrelated dirty files without modifying or claiming them.

- [ ] **Step 6: Commit the report and handoff**

```bash
git add .agents/reports/2026-08-03-sea-forge-interaction-model.md \
  .agents/CURRENT_STATUS.md .sea/interaction
git commit -m "docs(interaction): hand off canonical journey model"
```

## Checkpoints

- **After Tasks 1-2:** all 128 observed stories have one classification and one canonical mapping; the catalog and counts reconcile.
- **After Tasks 3-5:** the modular SEA model validates and the interface layer remains a projection binding rather than canonical identity.
- **After Task 6:** DomainForge has parsed, validated, and meaningfully projected the model; limitations reflect inspected behavior.
- **After Task 7:** reports, counts, validation, and handoff agree; no application or grammar code changed.

## Self-Review

- Spec coverage: Tasks 1-2 cover normalization, canonical catalog, grammar, and compression; Tasks 3-5 cover ontology, journeys, variants, transitions, and interface projections; Task 6 covers DomainForge validation and grammar limitations; Task 7 covers reporting, review, and handoff.
- Empty-marker scan: every task names exact files, inputs, outputs, commands, and expected evidence; no incomplete markers remain.
- Type consistency: `CJ01`-`CJ12`, namespace `sea_forge.interaction`, the CSV vocabulary, and maturity vocabulary are consistent across all tasks.
- Scope: the plan creates only model, analysis, validation, report, and handoff artifacts. It changes no product or DomainForge code.
