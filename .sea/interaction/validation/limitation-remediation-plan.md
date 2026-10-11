# Limitation Remediation Plan

Course of action for the ten limitations in `limitations.md`. Written
2026-08-12 against the same local DomainForge build used for the model's
validation evidence.

Two conclusions govern everything below:

1. **No SEA Forge model change is required by any limitation.** The model
   already responds correctly to each one — by normalizing, by moving a check to
   `reconcile.py`, or by declining to model around it. Nine of ten are
   DomainForge's to fix; one (L8) is correctly left alone.
2. **The most urgent item is not a code fix.** The L1 remedy already exists and
   is at risk of being lost.

## P0 — Do first

### L1 is finished work living in a git index, with no commit behind it

Verified in `/home/sprime01/projects/domainforge`:

| Property | Value |
| --- | --- |
| Branch | `feat/rdf-instance-policy-projection` |
| Commits ahead of `main` | **0** |
| Upstream | **none configured** |
| RDF work | staged in the index only |

The instance-and-policy RDF projection — `domainforge-core/src/kg.rs`,
`projection/rdf/ontology.rs`, `docs/rdf-projections.md`,
`docs/specs/SDS-005-knowledge-graph-module.md`, and a new
`tests/turtle_instance_export_tests.rs` — exists as staged changes and nothing
else. `git reset --hard`, `git checkout`, a stash mishap, or a clean clone
destroys it. This is the work that takes the model's RDF projection from 333
lines to 2101 and is the sole remedy for L1.

**Action:** commit it today, on its own branch, and push. The index also holds
35 unrelated `.claude/skills/neatcode/**` files and two `.agents/*.md` edits;
commit those separately so the RDF change stays reviewable.

**Cost:** minutes. **Risk of delay:** total loss of completed work.

### L5 is a three-line silent-falsehood bug, and it is now located

Root cause, `domainforge-core/src/policy/quantifier.rs:540-551`:

```rust
.filter(|item| {
    let substituted = filter_expr
        .substitute(variable_name, item)
        .unwrap_or_else(|_| filter_expr.as_ref().clone());   // error #1 discarded
    if let Ok(expanded) = substituted.expand(graph) {
        Self::is_true_literal(&expanded)
    } else {
        false                                                 // error #2 becomes "no match"
    }
})
```

Arithmetic in a `where` predicate makes `expand` return
`Err("Arithmetic operations not supported in boolean context")`
(`policy/core.rs:327` and `:510`). The closure maps that error to `false`, so
the row is silently dropped. Every row drops, the count is zero, and the
invariant reports green.

This is the highest-severity item in the whole list: it does not merely fail to
prove something, it *appears* to prove it. Any author who writes a per-row
arithmetic invariant gets a passing policy that checks nothing.

**Action, stage A — fail loudly (small).** Propagate both errors instead of
swallowing them: collect into a `Result<Vec<_>, _>` and return the evaluation
error. A policy that cannot be evaluated must not evaluate to true.

**Action, stage B — evaluate correctly (larger, optional).** Support arithmetic
in the filter's boolean context so the predicate means what it reads.

Stage A alone resolves the defect: after it, a meaningless invariant is
impossible to write unnoticed. Stage B is what would let this model move its
maturity-sum check from `reconcile.py` back into SEA. Do not wait on B to ship
A.

## P1 — High value, library work already done

### L4 — `domainforge contract` / `domainforge envelope` subcommands

The `application/` module is complete: `contract.rs`, `envelope.rs`,
`canonical.rs`, `resolve.rs`, `validate.rs`, `diagnostic.rs`,
`policy_context.rs`. Only CLI wiring is missing. The CLI has eleven subcommands
(`Parse`, `Validate`, `Import`, `Project`, `Format`, `Test`, `ValidateKg`,
`Normalize`, `Registry`, `Authority`, `Pack`), each a file in `src/cli/`
following one established pattern. Adding a twelfth is mechanical.

**Preferred shape:** a `Contract`/`Envelope` subcommand pair *and* an
`--application` flag on `validate` that surfaces `APP001`–`APP015` diagnostics.
The flag matters more than the subcommands: it is what makes the operation
reachable by the command an ordinary editor actually runs.

**Payoff for SEA Forge:** deletes `application-contract-harness.rs` and removes
a hand-built Rust binary from this model's validation recipe.

### L9 — exclude operation-bound policies from graph evaluation

DomainForge's own flagship fixture
`fixtures/application_generation/flagship/command-write.sea` fails
`domainforge validate`. A policy written for an operation's typed precondition
context is evaluated against the semantic graph, where its terms do not exist,
and returns `UNKNOWN (NULL)`.

Today this forces an either/or: graph-evaluable policies *or*
operation-precondition policies in one file, never both. It blocks any write
operation with a precondition — the exact case the Application Contract exists
to serve.

**Action:** partition policies by binding context; the graph validator skips
those bound to an operation, and the contract validator evaluates them in the
operation's typed context. Fixing this should make the flagship fixture pass,
which is the natural acceptance test.

## P2 — Real gaps, no current blockage

### L6 — typed instance collections

**Recommended:** add `entity_instances of "X"` as a typed collection.

**Rejected alternative:** making `or` Kleene-correct. That changes the semantics
of every existing policy in every model, to fix a problem the typed collection
solves locally. Wrong blast radius.

Note this is a convenience, not a correctness gap: the aggregation-with-`where`
form works today, and for the universal claims this model needs, the
`min_length` field constraint is a stronger guarantee than a `forall` policy
would be.

### L3 — role-to-entity binding syntax

`graph/mod.rs` already carries `entity_roles: IndexMap<ConceptId,
Vec<ConceptId>>` with accessors (lines 55, 225, 233, 237) and serialization
(833, 873). The graph supports the feature; the text surface cannot author it,
so `entity_roles` is always `{}`. This is a parser-and-lowering task over an
existing data model, not a new capability.

Design it with explicit cardinality and responsibility kind — otherwise it will
not carry CMMN `performerRef` or any real authority projection, which is the
point of having it.

## P3 — Language completeness

### L2 — array literals in `expression`

Beyond authoring convenience, there is a soundness angle worth stating: an
entity may declare a **required** `list<T>` field that no instance can ever
satisfy, making the entity uninstantiable with no diagnostic. DomainForge should
either add the array literal to `literal` (`sea.pest:444`) or emit a diagnostic
when a required `list<T>` field is declared on an instantiable entity.

Low urgency here — normalization into `InterfaceBinding` is the better model
regardless — but the trap is real for other authors.

## Deferred, and deliberately

### L7 — typed transition graphs

ADR-scale language design touching BPMN, CMMN, TLA+, and event projections. It
is the most valuable long-term item in the list and the one most likely to be
done badly if approached opportunistically. It needs a design document, not a
patch.

**Until then, do not encode a transition table in typed entities.** DomainForge
still could not check reachability, guard satisfaction, or terminal coverage, so
it would be a large artifact with no teeth.

### L8 — per-journey step emphasis

**No action.** All twelve journeys traverse all eight phases; ninety-six
journey-to-step rows would carry no information. The revisit trigger is already
recorded: a journey that legitimately skips a phase.

### L10 — key must be `string` or `uuid`

**Close as not-a-defect.** APP005 by design. The small duplication in
`CanonicalizationClassification` is documented so it is not mistaken for an
oversight, and the enum retains its teeth (T03).

## Sequence

| Order | Item | Owner | Why here |
| --- | --- | --- | --- |
| 1 | Commit and push the RDF branch (L1) | DomainForge | Prevents loss of finished work |
| 2 | L5 stage A — fail loudly | DomainForge | Silent falsehood outranks every missing feature |
| 3 | L4 — CLI reach into the contract | DomainForge | Library done; removes this model's manual harness step |
| 4 | L9 — policy partitioning | DomainForge | Unblocks write operations; own fixture is the test |
| 5 | L6, L3 | DomainForge | Real gaps, current workarounds are sound |
| 6 | L2, L5 stage B | DomainForge | Completeness |
| — | L7 | DomainForge, via ADR | Needs design, not a patch |

## SEA Forge's own actions

Small and independent of the above.

- **Wrap the four-check recipe in one script.** Today `README.md` lists four
  commands, one of which requires hand-building a Rust binary. A single
  `validation/check-all.sh` returning one exit status makes "all four must pass"
  enforceable rather than aspirational, and makes it CI-able.
- **After L4 lands:** delete `application-contract-harness.rs`, replace steps 4
  and 5 of the recipe with the subcommand, and retire limitation L4.
- **After L5 stage B lands:** the per-journey maturity-sum check *may* move from
  `reconcile.py` into a SEA policy. `reconcile.py` stays regardless — DomainForge
  will never read the CSV, and the row-level checks are its real job.
- **Change nothing else in the model.** No limitation on this list is evidence
  that a journey, binding, or invariant is wrong.

## What not to do

- Do not write any policy with arithmetic in a `where` predicate until L5 stage
  A lands. It will look green and prove nothing.
- Do not reintroduce `list<ref<T>>` fields when L2 lands. The normalized
  `InterfaceBinding` is the better model on its own merits.
- Do not re-split the model into modules. Closure works (T24–T26); one file is
  an architectural choice, already settled.
- Do not add `JourneyStepUse` (L8) or a transition table (L7) speculatively.
- Do not treat a green `domainforge validate` as sufficient while L4 and L5
  stand.

## Evidence

Verified by direct inspection of `/home/sprime01/projects/domainforge` on
2026-08-12: branch and index state; the eleven CLI subcommands and the absence
of any contract or envelope path; the complete `application/` module; the filter
closure at `quantifier.rs:540-551` and the arithmetic rejection at
`core.rs:327`/`:510`; and `entity_roles` support in `graph/mod.rs`.

Effort characterizations ("three-line", "mechanical", "ADR-scale") are estimates
from reading the call sites, not from implementing the fixes.
