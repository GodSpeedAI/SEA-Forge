# SEA Forge Interaction Grammar

## Purpose and Scope

This grammar describes stable interaction meaning across Workbench, CLI, API,
agent, and future interface projections. It is grounded in the approved
interaction-model design, the governed UX epic, and the reviewed
`canonicalization-matrix.csv`. A screen, route, method, record type, or command
may bind to the grammar, but none defines a new canonical journey by itself.

## Common Skeleton

```text
intention -> context -> authority/availability -> capability -> state/artifact
-> evidence/limitations -> settlement/decision -> next lawful affordance
```

The phases mean:

1. **Intention:** State the actor's job and desired outcome independently of an interface.
2. **Context:** Select or supply the cell, actor, case, semantic model, source, provider, and relevant immutable snapshots.
3. **Authority/availability:** Resolve identity, disclosure, policy, eligibility, dependencies, compatibility, and the exact grant before a protected effect.
4. **Capability:** Invoke one typed, bounded behavior. Intent is never interpolated into an untyped command or path.
5. **State/artifact:** Append a governed transition or create an attributable artifact. Read-only interaction produces a source-linked projection rather than new truth.
6. **Evidence/limitations:** Expose records, hashes, provenance, freshness, assurance, missing conditions, and disclosure limits. Process exit and narration are evidence inputs, not acceptance.
7. **Settlement/decision:** Evaluate the declared completion rule or reach an explicit allow, deny, escalate, approve, reject, park, quarantine, adopt, or other governed decision.
8. **Next lawful affordance:** Offer only actions that are visible, reachable, permitted, and settleable in the new context, or end at an explicit terminal condition.

Denial, escalation, expiry, interruption, timeout, cancellation, stale state,
quarantine, and unsupported state use the same skeleton. They retain evidence
and close with recovery or terminal semantics instead of becoming generic
errors.

## Fundamental Verbs

The grammar uses verbs for behavior and nouns for governed identity:

- **Establish** and **resolve** cell, actor, sponsorship, policy, model, and snapshot context.
- **Discover**, **ask**, **inspect**, **list**, **preview**, and **probe** what exists, is available, is allowed, or is proven.
- **Supply**, **validate**, **pin**, and **bind** semantic sources, plans, criteria, environments, evaluators, providers, and limits.
- **Form**, **instantiate**, **propose**, **preflight**, and **commit** a case before execution effects.
- **Navigate**, **explain**, **append**, **reopen**, **replan**, **park**, and **terminate** live case state without rewriting history.
- **Approve**, **reject**, **refuse**, **expire**, and **complete** accountable human judgment or work.
- **Authorize**, **execute**, **delegate**, **capture**, and **terminate** work within an exact granted boundary.
- **Monitor**, **notify**, **cancel**, **resume**, **recover**, **reactivate**, and **escalate** operational work at a bounded scope.
- **Evaluate**, **settle**, **verify**, **audit**, and **replay** outcomes from immutable criteria and evidence.
- **Recall**, **derive**, **route**, **promote**, **contract**, and **rebuild** evidence-backed memory and capability views.
- **Project**, **transform**, **quarantine**, **synthesize**, **productize**, and **capitalize** governed artifacts through legal maturity transitions.
- **Export**, **import**, **isolate**, **adopt**, and **reject** assets across a cell boundary.

These verbs do not imply success. Each invocation produces a typed governed
outcome whose settlement and next affordance remain explicit.

## Fundamental Nouns

- **Actors:** operator, case owner, approver, human-task assignee, domain or plan author, agent sponsor, auditor or reviewer, cell administrator, and external actor or integration.
- **Context:** cell, actor identity, sponsor, case, plan item, run episode, model, policy snapshot, environment, evaluator, endpoint, sandbox, and disclosure scope.
- **Authority:** action, resource, decision, grant, denial, escalation, approval, separation-of-duty rule, expiry, and policy reference.
- **Capabilities:** typed operations that can be declared, installed, available, validated, demonstrated, degraded, unsupported, or unknown; these states remain distinct.
- **State:** case, item, run, approval, capability, artifact, import, readiness, and assurance standing. State changes append records.
- **Artifacts:** plans, templates, model references, snapshots, projections, outputs, transcripts, bundles, manifests, evidence, declarations, settlements, and transition tokens.
- **Evidence:** source records, traces, hashes, criteria bases, provenance, ledger entries, checkpoints, witness receipts, and reproducible source references.
- **Decisions:** authority, human judgment, settlement, adoption, promotion or contraction, quarantine, recovery, and terminal decisions.
- **Affordances:** the next actions that are simultaneously visible, reachable, permitted, and settleable in the current context.

## Canonical Concepts

The twelve canonical journeys are the stable end-to-end concepts:

1. `CJ01 establish_trusted_cell_context`
2. `CJ02 discover_lawful_affordances`
3. `CJ03 ground_work_in_semantic_meaning`
4. `CJ04 form_and_commit_governed_case`
5. `CJ05 navigate_and_adapt_live_case`
6. `CJ06 resolve_human_judgment_and_approval`
7. `CJ07 execute_governed_work`
8. `CJ08 monitor_intervene_and_recover`
9. `CJ09 evaluate_settle_and_audit_outcomes`
10. `CJ10 reuse_demonstrated_knowledge_and_capability`
11. `CJ11 transform_and_mature_governed_artifacts`
12. `CJ12 transfer_and_adopt_governed_assets`

Each journey owns one materially distinct intention, transition pattern,
artifact or state result, completion condition, and next-decision class.
Specializations narrow its subject or capability. Variants change actor,
entry point, provider, interface, outcome, or recovery while preserving that
tuple. Compositions reuse steps from more than one journey and retain one
primary mapping according to the observed story's dominant intention.

## Projection-Only Concepts

The following surfaces improve access to governed concepts but never become
authoritative truth:

- Workbench routes, screens, drawers, tables, cards, badges, action menus, and previews;
- CLI commands, SFWP methods, external APIs, ACP prompts, and provider-specific controls;
- readiness, horizon, timeline, run-detail, inbox, asset, capability, artifact, maintenance, and notification views;
- Thoth questions, answers, explanations, manager judgments, and proposals;
- memory indexes, capability projections, self-model projections, search results, status summaries, and other rebuildable derived stores;
- generated contracts, reports, screenshots, and projections that cite source artifacts but do not replace them.

A projection must expose source references, freshness, rebuildability,
assurance, and limitations appropriate to its claim. If its source is absent,
stale, or unverifiable, the projection reports that state; it does not fill the
gap with inferred success.

## Composition Rules

1. Map each observed story to exactly one primary `CJ01`–`CJ12` journey. Record secondary behavior as composition, never as a second identity.
2. Choose the primary journey by actor intention, initiating condition, state transition, capability, artifact, evidence, completion condition, and next decision—not shared wording or screen placement.
3. Reuse canonical steps without copying their semantics. A composed journey carries the authority, evidence, failure, recovery, and settlement rules of every invoked step.
4. Keep authority and isolation independent. An allow decision does not create a sandbox, and a sandbox does not grant authority.
5. Decide every side effect before execution. Read-only disclosure is also authority-scoped before retrieval.
6. Append retries, replans, approvals, cancellations, adoptions, promotions, contractions, and lifecycle transitions. Never edit canonical history in place.
7. Separate execution termination from settlement. Exit code, generated bytes, or agent narration cannot satisfy a completion condition alone.
8. Treat denial, escalation, expiry, stale state, failure, quarantine, and unsupported state as explicit outcomes with evidence and a next lawful path or terminal condition.
9. Keep authoritative records and rebuildable projections distinct. Rebuilding a view may change access or speed, never truth.
10. Preserve actor separation where policy requires different authors, proposers, executors, approvers, settlers, or promoters.
11. A provider, interface, route, work source, or artifact stage creates a specialization or variant unless it materially changes the canonical tuple.
12. A larger flow is complete only when its declared settlement or decision accepts the outcome and the next affordance is explicit.

## Why Thoth Is a Composition

Thoth is a bounded epistemic and management interface over existing journeys:

- identity, capability, requirement, authority, and affordance questions project `CJ02` discovery;
- failure and evidence questions project `CJ09` audit;
- bounded proposals enter `CJ04` case formation as untrusted candidate work;
- deterministic manager judgments and admitted proposals adapt a case through `CJ05`;
- answers and routing may consult `CJ10` evidence-backed capability and memory.

Thoth creates no private authority, execution, approval, settlement, or plan
mutation channel. Its answer is disclosure-controlled and confers no execution
authority. Its proposal uses ordinary validation and governance. Its manager
cannot certify its own work. Those constraints make Thoth an interface
composition, not a thirteenth journey or separate grammar.

## Why Maintenance Is a Composition

Maintenance preserves existing journey semantics across change:

- self-model rebuild, version compatibility, and lifecycle inspection re-establish `CJ01` trusted context;
- derived-store repair reuses `CJ10` rebuild semantics;
- in-flight protection, interruption recovery, and debt resolution use `CJ08`;
- honest assurance labels require `CJ09` evidence and audit rules.

Maintenance has no independent user outcome that survives those components.
It composes context, recovery, reuse, and verification while preserving launch
snapshots and authoritative history.

## Identity and Authority Are Cross-Cutting Steps

Identity resolves who acts, under which role, sponsor, cell, and case context.
It begins in `CJ01`, then accompanies protected requests, evidence,
declarations, approvals, settlements, promotions, and adoptions. A missing or
conflicting identity fails closed; a dedicated identity screen would remain an
entry-point projection rather than a separate end-to-end journey.

Authority determines whether the exact actor may disclose, inspect, mutate,
execute, decide, settle, promote, transfer, or adopt the exact resource under
the governing snapshot. It therefore appears between context and capability
in every journey. Denial and escalation remain governed outcomes, and no
interface, agent provider, projection, or maintenance path may bypass the
common authority fabric.

## Recurring Variation Dimensions

The reviewed matrix uses recurring dimensions without turning each value into
a new journey:

| Dimension family | Typical values |
| --- | --- |
| Actor and separation | operator, author, owner, sponsor, approver, assignee, auditor, administrator, external actor; independent proposer, executor, settler, or promoter |
| Subject and context | cell, policy, model, plan, case, item, run, approval, endpoint, capability, artifact, bundle; current cell, case, or snapshot |
| Entry point and interface | new or returning cell, migration, Workbench, CLI, SFWP, Thoth, ACP, machine-readable record, notification |
| Provider and executor | human, command, projection, transition, HTTP agent, ACP agent, SWE_SEED host, adapter, evaluator |
| Work source and topology | intent, versioned template, external proposal, ADLC, ODI, specification pipeline, discretionary proposal, sequential or concurrent topology |
| Lifecycle and state | startup, active, waiting, parked, interrupted, failed, settled, stale, degraded, quarantined, imported, adopted, promoted, contracted |
| Authority outcome | allow, deny, escalate, approve, reject, refuse, expire, disclose, partial, quarantine, adopt |
| Assurance and evidence | source hash, immutable binding, criterion basis, trace, settlement, ledger proof, provenance, freshness, limitations |
| Artifact and maturity | source, plan, snapshot, run, transcript, projection, transition token, bundle; cognitive, intellectual, product, capital |
| Boundary and scope | sandbox, environment, network, tools, secrets, model, tokens, turns, timeout, retention, concurrency, sibling or run scope |
| Recovery and next decision | repair, revalidate, reprobe, retry, resume, replan, reactivate, change provider or environment, rebuild, escalate, stop |

New dimensions belong in the grammar only when multiple observed stories need
them and they change how a canonical tuple is described. A one-off interface
detail remains a binding, not a grammar extension.
