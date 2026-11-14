# Limitations of the GS-FDE-SOP-001 declared model

Re-tested against DomainForge 0.21.2 on 2026-11-13. The repository's
interaction-model limitations (`../../validation/limitations.md`, L1-L10) remain
the reference for L-numbered entries; this file records only what was re-tested
for this model and the model-specific entries (S-numbered). The translation map
cites these ids; `scripts/check-fde-sop-map.sh` fails on an unknown id.

## Re-tested L-numbered limitations

### L2 - `list<T>` fields cannot be instantiated

Still binding. The Field Case content groups (SOP 4.2), assignment briefs (7.4),
handoff contents (10.1, 10.2, 10.4) and structural-revision records (9.4) are
list-valued or many-part records. They are not declared (map status `unknown`).

### L3 - role-to-entity binding: grammar now exists, responsibility kind does not

Partly resolved. `role_binding "Role" for "Entity"` works in 0.21.2 (probe:
`domainforge-output.txt` T6) and is used here. The binding carries no responsibility
kind or cardinality, so "the FDE owns ..." or "does not own ..." (SOP 3.3, 3.7)
is not expressible. A binding also cannot target an entity imported from another
namespace (probe T5), so no role is bound to the evidence-settlement entity.

### L5 - arithmetic in a `where` predicate passes silently

Still binding, still avoided: no policy here uses arithmetic over instance
fields. Counting policies over declared instances use equality on counts only.

### L6 - `forall` / `exists` cannot be scoped to one entity type

Still binding. Universal claims over declared instances are written as counts.

### L7 - branching, guards, ordering and recovery edges

Still binding. The Navigation cycle is four flows, not an enforced transition
graph. State transitions such as "candidate until it survives variation" (9.5)
are not expressible.

### L9 - an operation's `policy_governed` access broke graph validation

Resolved for 0.21.2. Operation-bound policies are scope-checked by
`validate --application` (APP007) and skipped by graph validation; the positive run (AT-01 step 1) shows
a model with 18 operations, most carrying bound policies, validates with 0 violations. See S2 for what
that does and does not prove.

## Model-specific limitations

### S1 - `postcondition` and `invariant` enforcement points are rejected

Tested: a policy bound `at postcondition` fails `validate --application` with
`policy_scope_error: enforcement point 'postcondition' ... is reserved; only
precondition is executable` (probe T2). Every bound policy here is a
precondition. Rules whose natural form is an after-effect (SOP 7.5 "update case,
residuals, next affordances, handoff") are `unknown`.

### S2 - operation-bound policies are scope-checked, not evaluated, by `validate`

A bound policy may reference only fields of the operation's input record
(APP007; probe T1 shows a misspelled field is rejected). `validate` proves the
policy parses and resolves; it does not evaluate it. Evaluation belongs to the
DomainForge evaluator (WP-12). Every row marked S2 is therefore declared and
scope-checked but its verdict is `insufficient` until WP-12 runs it. The values
the policy tests (`freeze_recorded`, `settler_ref`, ...) are supplied by the
caller, so the policy states the rule; the case keeper must supply true values.

### S3 - negative, ordering, temporal and immutability properties

"Never", "never relabelled", "no automatic advance", "starts as soon as",
"reassess when evidence changes" and similar require history, ordering or the
absence of an operation. A precondition over one input record cannot state them,
and the absence of a mutating operation is not a checkable property.

### S4 - key uniqueness is not enforced by DomainForge

The episode key (case, plan item, n) is an entity key only in the case keeper.
`episode_key_unique` tests a caller-supplied count of prior freezes; DomainForge
does not count freezes. The (case, plan item, n) key itself is design constraint
I-3 (integration architecture E 3), not text of the SOP.

### S5 - prescriptive prose without a decidable predicate

Heuristics, selection rules and instructions to people ("pick the next move that
is useful ...", "listen for differences ...", "ask the cheapest question") have
no predicate a policy could evaluate. Forcing one would validate and mean
something else, so they stay `unknown`.

### S6 - runtime measurement

Actual payment, reliability thresholds and the standing criterion need observed
values. A declaration holds no observations, and a metric over declared
instances can only count the declaration.

### S7 - not declared in this version (scope)

Content the SOP states in full but this version does not declare: the 4.2 field
groups, the 7.2 frozen-reference contents, the 7.4 assignment brief, the 10.x
handoff and continuation contents and the 9.4 revision record. Each is
list-valued or many-part (so L2 also applies) and has no structure the SOP
fixes beyond the list.
