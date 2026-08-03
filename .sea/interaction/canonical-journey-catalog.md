# SEA Forge Canonical Journey Catalog

This catalog compresses the 128 observed Workbench stories into twelve stable
user journeys. The reviewed mapping in `canonicalization-matrix.csv` is the
row-level authority for coverage, classification, confidence, and maturity.
The journey descriptions below summarize that mapping against the governed UX
epic and its cited specifications, source, tests, and execution evidence.

Source shorthand in the entries resolves to repository paths: “governed UX
epic” means
`.agents/specs/frontend/sea-forge-governed-workbench-ux-epic-v0.1.md`;
`spec-minimum.md`, `spec-full.md`, the ADLC/Thoth specification, and the
agent-orchestration specification are under `.agents/specs/`; and
`USER_JOURNEY_EVIDENCE.md`, `REPOSITORY_TRUTH.md`, and
`ARCHITECTURAL_TRUTH.md` are under `docs/execution/`. Exact source, method,
route, and test references remain on every row of the matrix.

Implementation maturity is reported as a distribution of mapped stories. It
does not claim that a whole journey has reached the strongest maturity found in
one of its rows. Routes, views, methods, and records are entry points or
evidence; they do not define journey identity.

## CJ01 — Establish Trusted Cell Context

- User intention: Enter a cell whose identity, configuration, integrity, semantic self-model, and operation-specific readiness are explicit.
- User job: Establish a trustworthy starting context for a human or automated actor without overwriting history or treating stale state as current.
- Initiating condition: A user opens or creates a cell, returns to existing or legacy history, starts the service, enters protected work, or changes the release or execution substrate.
- Preconditions: A selected cell root; readable existing records when present; configured identity, policy, ledger, model, extension, environment, endpoint, and server inputs appropriate to the intended operation.
- Entry points: Cell selection or initialization, migration, startup preflight, identity resolution, readiness inspection, policy inspection, self-model maintenance, and compatibility inspection.
- Reusable steps: Identify the cell and actor; preserve or migrate attributable history; validate required foundations; realize and hash-pin the self-model snapshot; inspect policy and version compatibility; classify readiness; expose affected capabilities and corrective actions.
- Invoked capabilities: Cell initialization and bundle-aware migration, startup validation, `identity.get`, `readiness.get`, policy inspection, self-model validation or rebuild, lifecycle inspection, and version-skew detection.
- State transitions: Unknown, legacy, or changed context becomes validated current context, explicitly stale but verifiable context, degraded context, or blocked context; invalid identity and configuration fail closed; lifecycle changes preserve in-flight snapshots.
- Artifacts: Cell identity, existing case and ledger history, migration records, policy and configuration digests, release-and-cell self-model snapshots, readiness items, assurance labels, and compatibility findings.
- Evidence: Governed UX epic §§1, 2.1–2.4, and 16.1–16.5; `spec-minimum.md` §§7.3.3, 8, 8.5, 10.2, and 14; `spec-full.md` §§7.0b–7.0c and 7.4; `sea-forge-cell`, `sea-forge-self-model`, lifecycle, identity, readiness, and version-skew conformance tests; `USER_JOURNEY_EVIDENCE.md` Journeys 1, 4, 6, 6b, and 8b.
- Completion condition: The actor is attributable, the governing snapshots and integrity state are explicit, and the cell states whether the intended operation is ready, degraded, stale, or blocked with evidence.
- Next decisions: Discover current lawful affordances, form a governed case, repair or revalidate a failed foundation, hold an incompatible operation, or stop.
- Recovery paths: Preserve prior verifiable snapshots; refuse protected work on identity conflict; rebuild or reprobe changed foundations; migrate compatible history without re-keying it; follow the readiness item to a bounded repair path.
- Known variants: New versus returning cells; legacy migration; human versus sponsored automated actors; healthy, stale, degraded, unavailable, quarantined, integrity-pending, or blocked readiness; release, extension, endpoint, environment, and toolchain changes.
- Implementation maturity: 15 mapped stories: 1 specified, 2 implemented, 5 exercised, and 7 evidenced. Sponsorship remains specified; the complete extension and endpoint lifecycle is only partly surfaced; readiness, identity, migration, snapshots, and compatibility have stronger test or execution evidence.
- Observed story coverage: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 2.2, 2.3, 2.4, 16.1, 16.2, 16.3, 16.5.

## CJ02 — Discover Lawful Affordances

- User intention: Understand what exists, what is usable, what is allowed, and why a path is unavailable in the current actor and cell context.
- User job: Choose a spendable path without confusing catalog presence, declared support, operational availability, authority, or settlement evidence.
- Initiating condition: A user asks what SEA Forge can do, previews a proposal's authority burden, browses an operating asset, probes an endpoint, or encounters denial, escalation, or unusable state.
- Preconditions: Trusted cell and actor context; disclosure scope resolved before retrieval; source records or validated catalogs for the subject being inspected.
- Entry points: Thoth questions, authority preview, denial detail, template and asset catalogs, environment and extension inspection, endpoint browsing or probing, capability browsing, and unavailable-asset explanations.
- Reusable steps: Name the intended operation or subject; resolve context and disclosure; distinguish existence from usability; inspect dependencies, compatibility, authority, evidence, and settlement requirements; probe only under authority; expose freshness and limitations; offer a lawful next action.
- Invoked capabilities: `thoth.ask`, asset and endpoint inspection, endpoint probe, capability projection, environment and adapter inspection, authority preflight, and denial explanation.
- State transitions: Unknown or merely declared support becomes available, demonstrated, unavailable, denied, escalated, partial, or still unknown; an endpoint probe adds evidence but confers no general execution authority.
- Artifacts: Bounded Thoth answers, disclosure plans, authority-burden previews, asset and endpoint rows, environment contracts, compatibility findings, capability records, probe evidence, denial reasons, and policy references.
- Evidence: Governed UX epic §§2.5–2.6, 3.1–3.6, 3.9, and 4.1–4.6, 4.8; `spec-minimum.md` §10.2; `spec-full.md` §§7.0b, 7.3, 7.6, and 10.0; Thoth and agent-orchestration specifications; Thoth, asset, endpoint, topology, authority, and case-episode tests; `USER_JOURNEY_EVIDENCE.md` Journeys 2 and 6b.
- Completion condition: The user receives a source-backed set of currently visible, reachable, permitted, and settleable actions, including explicit blockers, uncertainty, freshness, and limitations.
- Next decisions: Select an asset or provider, form or revise a case, request approval or authority, repair a dependency, probe availability, choose another path, or stop.
- Recovery paths: Explain denial or escalation from committed records; repair dependency, policy, compatibility, or probe failures; re-query after context changes; preserve `unknown` when evidence is insufficient.
- Known variants: Thoth, Workbench, and catalog interfaces; template, environment, extension, adapter, endpoint, projection, and capability subjects; HTTP and ACP providers; allow, deny, escalate, partial disclosure, unavailable, and unknown outcomes.
- Implementation maturity: 16 mapped stories: 4 specified, 1 implemented, 9 exercised, and 2 evidenced. Complete authority preview, current-affordance querying, environment browsing, and combined projection/environment support remain specified or partial.
- Observed story coverage: 2.5, 2.6, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.9, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.8.

## CJ03 — Ground Work in Semantic Meaning

- User intention: Bind work to validated, reproducible domain meaning before authority or execution can spend it.
- User job: Supply, validate, inspect, pin, and project a `.sea` model while keeping derived output subordinate to source.
- Initiating condition: A domain or plan author selects or supplies a source set, a plan needs semantic references, source drift is detected, or a projection is requested.
- Preconditions: `.sea` sources and imports are available; referenced concepts, policies, units, relations, environments, and projection adapters can be resolved and versioned.
- Entry points: Source selection, semantic validation, model inspection, plan binding, reference validation, drift response, and governed projection.
- Reusable steps: Parse sources; resolve imports; validate identities and semantics; inspect the validated namespace; pin source and adapter hashes in a `DomainModelRef`; resolve every plan reference; detect drift; project deterministically; validate or quarantine output.
- Invoked capabilities: DomainForge parsing and semantic validation, model inspection, plan reference resolution, hash pinning, version-skew checks, deterministic projection, and output validation.
- State transitions: Supplied source becomes validated or rejected; a plan moves from unresolved to semantically pinned; drift blocks silent reuse; projection output becomes validated and hash-linked or quarantined.
- Artifacts: `.sea` source sets, imports, diagnostics, validated domain models, namespace and concept views, `DomainModelRef`, source and adapter digests, projections, hashes, and quarantine records.
- Evidence: Governed UX epic §5; `spec-minimum.md` §10.1; `spec-full.md` §§7.0a and 10.4a; DomainForge, planner, projection, criteria-provenance, version-skew, and spec-pipeline conformance tests.
- Completion condition: All semantic references resolve against an exact validated model and adapter snapshot, or activation remains blocked with layer-specific diagnostics and a lawful repair path.
- Next decisions: Pin the model to a case, revise the source, revalidate or replan after drift, run a supported projection, or quarantine and rework invalid output.
- Recovery paths: Correct syntax, import, identity, policy, unit, relation, or mapping failures at their actual layer; revalidate changed sources; replan against new meaning; quarantine invalid projections without changing source truth.
- Known variants: Existing versus supplied sources; syntax, import, identity, policy, relation, and reference failures; model-view interfaces; drift; projection target, adapter, parameters, and output maturity.
- Implementation maturity: 7 mapped stories: 1 specified and 6 exercised. Semantic validation, pinning, drift checks, and projection are exercised; the complete model-browsing surface remains preview-only.
- Observed story coverage: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7.

## CJ04 — Form and Commit a Governed Case

- User intention: Turn intent, a template, or an untrusted proposal into a validated, previewed, immutable case plan.
- User job: Choose the appropriate work source, declare dependencies and criteria, inspect the complete proposal, and commit it before execution side effects.
- Initiating condition: An operator or external actor has supported intent, a versioned template, an external plan proposal, a methodology or topology, a specification pipeline, or a bounded Thoth proposal.
- Preconditions: Trusted actor and cell context; a supported work source; resolved semantic references; available environment, evaluator, endpoint, sandbox, retention, and execution-limit declarations where required.
- Entry points: Intent submission, template instantiation, external proposal admission, ADLC or ODI case creation, topology selection, spec-to-code planning, and Thoth proposal review.
- Reusable steps: Normalize the work source; derive or instantiate the plan deterministically; validate semantic and criteria provenance; bind environments, evaluators, endpoints, isolation, and limits; preview structure and likely approvals; check staleness; commit the accepted plan and provenance immutably.
- Invoked capabilities: Intent planning, template instantiation, proposal validation, topology construction, pipeline planning, delegation preview, `case.preflight`, and `case.commit`.
- State transitions: Intent, template, or untrusted proposal becomes a draft `CasePlan`; draft becomes preflighted or rejected; preflighted plan becomes an immutable committed case; stale input returns to fresh preflight rather than direct commit.
- Artifacts: Intent records, templates and parameters, draft and preflight summaries, `CasePlan`, stages, items, sentries, milestones, criteria origins, environment and evaluator bindings, job-contract previews, provenance, digests, and immutable case records.
- Evidence: Governed UX epic §6 and §10.3; `spec-minimum.md` §§7.3.1–7.3.2; `spec-full.md` §§7.6, 8.6, 10.2, and 10.7; ADLC/Thoth and agent-orchestration specifications; lifecycle, planner, case-authoring, case-episode, topology, delegation-preview, pipeline, and manager-loop tests; `USER_JOURNEY_EVIDENCE.md` Journeys 2 and 3.
- Completion condition: The exact accepted plan, parameters, criteria, provenance, configuration digests, and model references are committed before any execution side effect.
- Next decisions: Start lawful work, resolve likely approvals, revise or abandon an invalid or unaffordable draft, or recover a lost commit response from recorded request status.
- Recovery paths: Treat proposals as untrusted; return validation errors without execution effects; repeat preflight after source drift; use request-status recovery for ambiguous delivery; create a new version instead of editing committed history.
- Known variants: Intent, versioned template, external proposal, ADLC or ODI method, sequential or concurrent topology, spec-to-code pipeline, and Thoth-proposed work; human or external-agent proposer; provider, evaluator, sandbox, and limit bindings.
- Implementation maturity: 10 mapped stories: 1 specified, 6 exercised, and 3 evidenced. Case formation and immutable commit have strong evidence; the complete all-fields preflight described by story 6.8 remains specified beyond the implemented summary.
- Observed story coverage: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 6.9, 10.3.

## CJ05 — Navigate and Adapt a Live Case

- User intention: Understand a committed case's purpose, current horizon, history, and lawful adaptation paths without rewriting prior truth.
- User job: Orient to live work, explain its state, follow attempts, and append authorized changes when new work or lifecycle decisions arise.
- Initiating condition: A case is committed, its state changes, work becomes blocked or newly discovered, a retry creates another episode, or a bounded manager loop is invoked.
- Preconditions: A committed `CasePlan`, attributable actor context, source-backed case and run records, and authority for any mutation.
- Entry points: Case overview, structure and horizon views, timeline or run detail, discretionary work proposal, lifecycle control, and Thoth manager invocation.
- Reusable steps: Restate intent and desired outcome; project stages, items, milestones, and sentries; separate active, enabled, waiting, blocked, completed, rejected, and future work; explain transitions from evidence; link immutable attempts; validate and authorize appended work or lifecycle changes.
- Invoked capabilities: `case.get_overview`, `case.get_horizon`, run listing and detail, discretionary-item admission, reopen, replan, terminate, manager-loop judgment, and ordinary plan mutation.
- State transitions: Committed cases move among active, waiting, parked, failed, complete, reopened, replanned, or terminated states through appended events; retries create successor runs; approved proposals append new items.
- Artifacts: Case overview and horizon projections, plan graph, sentry and milestone state, timelines, run-episode links, deterministic manager judgments, discretionary proposals, new plan versions, lifecycle events, actor identity, and reasons.
- Evidence: Governed UX epic §7 and §§10.1–10.2, 10.4; `spec-minimum.md` §9; `spec-full.md` §§7.1 and 9; agent-orchestration specification §§7.6, 10.3, and 17.4; case-view, run-view, run-locator, case-runner, topology, and manager-loop tests; `USER_JOURNEY_EVIDENCE.md` Journeys 5 and 8b.
- Completion condition: The user can identify the case's governing outcome, current spendable and blocked work, the evidence for every state, and each presently lawful action.
- Next decisions: Start enabled work, resolve a blocker, add discretionary work, approve a proposal, retry as a successor episode, reopen, replan, terminate, park, or stop a bounded manager loop.
- Recovery paths: Distinguish parked from failed; preserve attempt history; append a replan or reopen event; reject unauthorized mutation; escalate after bounded manager exhaustion.
- Known variants: Human, command, projection, transition, and agent items; active, waiting, future, settled, parked, failed, and complete standing; retry, discretionary addition, reopen, replan, terminate, and Thoth-managed adaptation.
- Implementation maturity: 11 mapped stories: 10 exercised and 1 evidenced. The complete journey is broadly exercised; distinct run persistence has execution evidence, while several adaptation methods remain primarily test-grounded.
- Observed story coverage: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8, 10.1, 10.2, 10.4.

## CJ06 — Resolve Human Judgment and Approval

- User intention: Admit accountable human decisions and work into the governed record through the same authority and evidence fabric as other work.
- User job: Find pending action, inspect its full context, act only when eligible, and record the decision or human-task evidence durably.
- Initiating condition: An approval, human plan item, or ACP permission request becomes pending, or an existing request is denied or expires.
- Preconditions: Resolved actor identity; an existing request or assigned human item; visible policy, resource, purpose, evidence, boundaries, expiry, and downstream effect; separation-of-duty eligibility.
- Entry points: Shared inbox, approval detail, approval decision, human-task completion, ACP permission prompt, and denial or expiry detail.
- Reusable steps: List pending action; load source-backed context; check identity, authority, assignment, expiry, and independence; approve, reject, or supply required human evidence; append the outcome; reevaluate dependent sentries; expose downstream effects and next actions.
- Invoked capabilities: `approval.list`, approval context inspection, `approval.decide`, human-task listing and completion, ACP permission handling, identity and separation-of-duty checks, and case reevaluation.
- State transitions: Pending request becomes approved, rejected, expired, or refused for ineligible actor; a human item becomes criterion-backed complete; dependent session, run, item, or case becomes enabled, parked, blocked, or terminal.
- Artifacts: Approval requests, governance context, actor and eligibility records, notes, decision records, human-task evidence or artifacts, ACP permission records, expiry markers, and dependent case events.
- Evidence: Governed UX epic §8; `spec-full.md` §§7.2, 7.2.1, 10.2, and 10.3; agent-orchestration specification §10.4; approval, identity, ACP, human-waiting, and CLI approval tests; `USER_JOURNEY_EVIDENCE.md` Journeys 6 and 8c.
- Completion condition: An eligible human decision or contribution is committed with its actor, context, evidence, rationale, and downstream effect, or refusal, denial, or expiry is recorded without unauthorized side effects.
- Next decisions: Resume dependent work, reject or revise the path, reassign or request an eligible approver, escalate, or stop when the governed outcome is terminal.
- Recovery paths: Preserve pending state for unauthorized actors; show expiry and denial effects; obtain a fresh request or eligible decision; keep ACP inside the common approval fabric; never infer human completion from off-system conversation.
- Known variants: Approval versus human work; approve, reject, refuse, deny, and expire; Workbench versus ACP interface; assigned human versus approver; downstream session, run, item, or case scope.
- Implementation maturity: 7 mapped stories: 2 specified, 3 exercised, and 2 evidenced. Approval listing and decisions have evidence, but the complete combined inbox and user-facing `human_task.complete` path remain specified.
- Observed story coverage: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7.

## CJ07 — Execute Governed Work

- User intention: Perform human, command, projection, transition, or agent work only within the exact granted boundary and retain evidence for later settlement.
- User job: Authorize first, execute under declared isolation and environment constraints, observe bounded progress, and hand outcomes to evaluation without equating termination with success.
- Initiating condition: A committed plan item is enabled, its exact authority request has been decided, and its executor, provider, environment, sandbox, limits, and criteria are bound.
- Preconditions: Trusted actor and cell context; immutable plan and criteria; an allow decision for the exact action and resources; available provider and environment; enforceable sandbox, network, tool, secret, timeout, retention, and cost limits.
- Entry points: Command or projection execution, agent-task configuration, HTTP or ACP delegation, and SWE_SEED-harnessed coding work.
- Reusable steps: Normalize the operation; request and record authority; construct the granted sandbox and environment; invoke the selected executor with explicit bounds; capture progress, outputs, artifacts, transcript material, and termination; submit evidence for independent settlement.
- Invoked capabilities: Authority decision, sandbox realization, environment provision, command and projection execution, `delegation.preview`, HTTP and ACP agent execution, SWE_SEED route and proof harvesting, artifact capture, and settlement handoff.
- State transitions: Enabled work becomes denied, escalated, running, terminated, timed out, cancelled, violated, or disconnected; execution results and evidence become settlement inputs but never settlement themselves.
- Artifacts: Authority decisions, sandbox and environment snapshots, job contracts, instruction packets, context references, progress and output records, artifacts, transcript summaries or retained transcripts, termination reasons, and execution results.
- Evidence: Governed UX epic §§9.1–9.6; `spec-minimum.md` §§7.3.6, 10.2, and 10.3; agent-orchestration specification §§7.3, 10.2, 10.4, and 17.5; authority, sandbox, delegation-preview, HTTP, ACP, and SWE_SEED portable tests; `USER_JOURNEY_EVIDENCE.md` Journeys 2, 3, and 8d.
- Completion condition: Execution terminates within the granted boundary and leaves complete outcome evidence for settlement, including governed denial or failure; only settlement may accept the work.
- Next decisions: Evaluate and settle the outcome, monitor or cancel active work, choose a recovery path, change provider or environment through a new authorized attempt, or stop.
- Recovery paths: Deny before effects when authority fails; retain evidence for timeout, sandbox violation, cancellation, endpoint failure, and disconnect; create a successor attempt rather than overwrite a run; repair provider or boundary configuration before retry.
- Known variants: Command, projection, transition, and agent executors; OpenAI-compatible, Anthropic-compatible, ACP, and SWE_SEED providers; sandbox class, network, tools, secrets, model, turn, token, timeout, retention, and continuation bounds.
- Implementation maturity: 6 mapped stories: 3 exercised and 3 evidenced. Core authority-before-effect, sandboxing, and non-agent execution have execution evidence; agent-provider paths are exercised portably, while real external ACP and SWE_SEED host evidence remains release-gated.
- Observed story coverage: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6.

## CJ08 — Monitor, Intervene, and Recover

- User intention: Keep concurrent and interrupted work visible, controllable, and recoverable without changing committed meaning or affecting unrelated work.
- User job: Observe source-backed operational state, intervene at the right scope, resume committed work, and construct a lawful path after failure or change.
- Initiating condition: Work is running concurrently, emits a committed event, needs cancellation, parks, exhausts a bound, survives a restart or upgrade, fails, or accumulates unresolved operational debt.
- Preconditions: Committed case and run identities; ordered source records and cursors; scoped control authority; preserved launch snapshots; typed failure or blocking evidence.
- Entry points: Delegation roster, event stream, capacity view, notification, scoped cancel, parked-case resume, restart recovery, developmental reactivation, failure detail, and maintenance debt queue.
- Reusable steps: Read committed events; distinguish execution, capacity, authority, and dependency states; correlate siblings and rollups; notify with exact record links; authorize scoped intervention; append control outcomes; recover from committed state; diagnose typed failure; offer a bounded lawful next path.
- Invoked capabilities: Delegation listing, `events.subscribe`, `events.get_range`, concurrency and capacity projection, scoped cancellation, case resume, restart reconciliation, ADLC reactivation, failure diagnosis, lifecycle-snapshot preservation, and maintenance-debt inspection.
- State transitions: Running work becomes cancelled, completed, failed, interrupted, orphaned, or unsettled; parked work returns to active after resolution; failed developmental work reactivates an earlier stage; bounded manager work stops, parks, or escalates; upgrades preserve launch meaning.
- Artifacts: Ordered events and cursors, delegation and capacity views, linked notifications, cancellation and settlement records, preserved launch snapshots, recovery decisions, typed failure details, reactivation events, and maintenance debt items.
- Evidence: Governed UX epic §9.7, §10.5, §11, and §§16.6–16.7; `spec-minimum.md` §§14 and 14.3; `spec-full.md` §8.4; ADLC/Thoth and agent-orchestration recovery rules; event, topology, concurrency, cancellation, lifecycle, run-locator, run-view, delegation, and manager-loop tests; `USER_JOURNEY_EVIDENCE.md` Journeys 5, 8b, and 8c.
- Completion condition: The operator sees the authoritative operational standing and either completes a scoped intervention or reaches an explicit resume, retry, replan, repair, endpoint, environment, escalation, or terminal decision.
- Next decisions: Continue, cancel one run, resolve approval, resume, settle interruption, retry as a successor, replan, reactivate an earlier stage, change endpoint or environment, repair debt, escalate, or stop.
- Recovery paths: Catch up from cursors after event gaps; preserve sibling work; resume committed state; identify orphaned and unsettled runs after restart; retain launch snapshots across reloads; localize failure before choosing a new path.
- Known variants: Live agent dialogue, general event stream, concurrency and capacity, notification, scoped cancel, parked resume, restart, developmental loop, typed failure, upgrade continuity, and maintenance debt.
- Implementation maturity: 12 mapped stories: 1 specified, 1 implemented, 9 exercised, and 1 evidenced. Restart recovery has execution evidence; actionable notifications and the consolidated maintenance queue remain incomplete interaction surfaces.
- Observed story coverage: 9.7, 10.5, 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7, 11.8, 16.6, 16.7.

## CJ09 — Evaluate, Settle, and Audit Outcomes

- User intention: Prove what was intended, authorized, attempted, observed, accepted, and promoted from immutable evidence.
- User job: Inspect a linked episode, compare criteria with evidence, keep execution and settlement distinct, validate provenance and integrity, and reproduce governance history.
- Initiating condition: Execution terminates, a claim or failure needs explanation, a reviewer must settle criteria, an assurance label needs proof, or an auditor inspects authority, disclosure, ledger, or replay history.
- Preconditions: Immutable plan criteria and origins; authority, trace, evidence, artifact, transcript, declaration, settlement, and ledger records; policy-bounded disclosure; independent actor standing where required.
- Entry points: Run record, Thoth evidence or failure question, criterion comparison, declaration detail, ledger verification, replay, audit search, machine-readable source records, and assurance-label inspection.
- Reusable steps: Resolve the exact episode and configuration; verify record, artifact, and transcript identity; pair each criterion with pass, fail, or unavailable evidence; validate declaration standing and independence; settle separately from termination; reject invalid provenance; verify inclusion and ordering; expose source references and limitations.
- Invoked capabilities: `run.get`, evidence and claim explanation, artifact and transcript verification, settlement evaluation, declaration weighting, provenance checks, ledger inclusion and consistency proof, deterministic replay, audit search, disclosure history, and source-record access.
- State transitions: Execution evidence becomes accepted, rejected, or escalated settlement; declarations qualify, lose weight, or are excluded; assurance becomes supported, degraded, pending, stale, or unsupported; invalid proof leads to fresh-evidence recovery rather than silent promotion.
- Artifacts: Linked run records, immutable criteria, authority and configuration snapshots, traces, evidence artifacts, transcript descriptors, settlement records, declarations, provenance chains, ledger entries, checkpoints, witness receipts, replay order, disclosure plans, and typed source references.
- Evidence: Governed UX epic §§2.7, 3.7–3.8, 9.8–9.9, 10.6, 12, and 16.8; `spec-minimum.md` §§10.4, 10.6, 12.1, and 14; `spec-full.md` §§7.0c, 7.2.1, and 10.4; agent-orchestration evidence and separation rules; run-view, settlement, capability, ledger, replay, and manager separation tests; `USER_JOURNEY_EVIDENCE.md` Journeys 2, 3, 6b, 8c, and 8d.
- Completion condition: Every outcome or assurance claim resolves to attributable, integrity-checked evidence and an independent settlement or explicit unavailability; execution termination remains separately visible.
- Next decisions: Accept, reject, or escalate; obtain fresh qualifying proof; retry or replan; adjust capability standing; verify another record; or end at an auditable terminal condition.
- Recovery paths: Exclude mismatched criteria, broken origins, missing evidence, unauthorized declarations, and self-certification; obtain fresh proof; rebuild human-readable views from typed source records; verify ledger consistency without mutation.
- Known variants: Operator, reviewer, and auditor actors; command and agent executors; pass, fail, unavailable, accepted, rejected, and escalated outcomes; human-readable and machine-readable interfaces; local-digest, tamper-evident, witnessed, degraded, pending, stale, and unsupported assurance.
- Implementation maturity: 17 mapped stories: 1 specified, 11 exercised, and 5 evidenced. Core run linkage and execution-versus-settlement separation have execution evidence; cross-family audit search remains specified.
- Observed story coverage: 2.7, 3.7, 3.8, 9.8, 9.9, 10.6, 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 12.8, 12.9, 12.10, 16.8.

## CJ10 — Reuse Demonstrated Knowledge and Capability

- User intention: Reuse settled history and evidence-backed capability without allowing indexes, summaries, or stale projections to become authoritative.
- User job: Recall relevant memory under disclosure authority, inspect what influenced work, assess capability quality, and route future work from the same qualifying evidence.
- Initiating condition: Planning or execution needs prior settled context, an operator inspects capability standing, a capability changes tier, or a derived memory or capability view is missing, stale, or suspect.
- Preconditions: Authoritative source records and qualifying settlements; disclosure scope resolved before retrieval; rebuildable memory and capability projections; provenance links to downstream use.
- Entry points: Governed recall, influence inspection, capability browse or detail, promotion or contraction explanation, projection rebuild, routing, Thoth answer, and maintenance rebuild.
- Reusable steps: Authorize disclosure before search; query settled memory; preserve influence links; derive capability standing from qualifying settlements and variation; expose recovery, reliability, regression, and burden; promote or contract only through policy; fall back to source records; rebuild views deterministically; use the same record for routing and explanation.
- Invoked capabilities: Governed memory recall, source-record fallback, influence tracking, capability derivation, promotion and contraction, capability browse and detail, deterministic rebuild, routing gates, and Thoth claim derivation.
- State transitions: Settled evidence becomes recall influence; capability moves among attempted, demonstrated, proven, degraded, quarantined, and contracted through qualifying evidence; stale projections become rebuilt equivalent views without changing source truth.
- Artifacts: Recall requests and results, memory items, influence links, source references, capability records, qualifying settlements, variation and regression summaries, promotion or contraction bases, rebuilt indexes and views, and routing decisions.
- Evidence: Governed UX epic §4.7, §13, and §16.4; `spec-full.md` §§7.3, 7.5, 10.4, and 10.5; ADLC/Thoth claim-derivation rules; capability, memory, CLI recall, index-deletion, and projection tests.
- Completion condition: Reused knowledge or capability is disclosure-safe, traceable to source settlements, explicit about scope and weakness, and linked to the resulting plan, decision, answer, or settlement.
- Next decisions: Route work through a demonstrated capability, gather more qualifying evidence, narrow or quarantine a regressed capability, rebuild a stale view, or refuse reuse.
- Recovery paths: Fall back to authoritative scans when indexes fail; rebuild derived stores independently; contract capability on regression or lost proof; require fresh qualifying settlements to leave contracted standing.
- Known variants: Plan-time versus run-time recall; content, kind, entity, process, session, result, and case queries; attempted through contracted capability states; variation coverage, recovery, reliability, regression, and orchestration burden; routing versus Thoth consumers.
- Implementation maturity: 10 mapped stories: 2 specified and 8 exercised. Memory and capability derivation are exercised; the Workbench browse and detail methods for capability records remain specified and preview-only.
- Observed story coverage: 4.7, 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8, 16.4.

## CJ11 — Transform and Mature Governed Artifacts

- User intention: Derive, project, promote, or quarantine artifacts through explicit, evidence-backed maturity gates without skipping stages.
- User job: Select an exact source and transformation, validate the result, keep generated output distinct from runtime readiness, and prove each maturity transition and lineage link.
- Initiating condition: A validated source needs projection, a spec-to-code stage becomes eligible, generated output fails validation, or an artifact owner requests synthesis, productization, capitalization, or gate review.
- Preconditions: Identified and hash-pinned source artifacts; versioned adapters or transition rules; resolved semantic anchors; declared parameters, authority, criteria, ownership, license, review, attestation, reuse, and separation requirements appropriate to the target stage.
- Entry points: Governed projection, pipeline stage view, quarantine decision, artifact lineage view, maturity-transition request, gate preview, and `TransitionToken` verification.
- Reusable steps: Select source, target, adapter or transition mode; preview gates and limitations; authorize the exact change; derive or promote content; validate semantics and identity; settle runtime bindings separately; record hashes, lineage, evidence, and authority; quarantine invalid output; verify legal predecessor chains.
- Invoked capabilities: Deterministic DomainForge projection, spec-pipeline transformation, artifact registration and lineage inspection, gate evaluation, derive or promote transition, quarantine, separation-of-duty checks, and `TransitionToken` validation.
- State transitions: Source becomes validated derived output or quarantined output; cognitive becomes intellectual, intellectual becomes product, and product becomes capital only through legal transitions; candidate becomes eligible or blocked; generated contract remains distinct from runtime-accepted binding.
- Artifacts: Source and output descriptors, semantic projections, pipeline stage outputs, hashes, adapter versions, validation and quarantine records, ownership and license data, review and reuse evidence, settlement records, and `TransitionToken` predecessor chains.
- Evidence: Governed UX epic §14; `spec-full.md` §§7.8a, 7.9, 10.4a, 10.7, and 10.8; DomainForge projection, spec-pipeline, artifact-IP, and CLI artifact conformance tests; `ARCHITECTURAL_TRUTH.md` “What Usable Means.”
- Completion condition: The output is either validated, hash-linked, and settled at its actual maturity or quarantined with typed reasons; every transition preserves provenance and no file creation alone claims runtime readiness.
- Next decisions: Use the validated projection, advance to the next legal stage, satisfy missing gates, implement and settle a runtime binding, rework quarantined output, or stop.
- Recovery paths: Quarantine malformed, invalid, drifted, or unverifiable output; repair missing semantic, evidence, review, ownership, license, attestation, reuse, or approval gates; re-run from the last valid predecessor.
- Known variants: Projection target and adapter; ADR through acceptance pipeline stages; derive versus promote; cognitive-to-intellectual, intellectual-to-product, and product-to-capital transitions; eligible, blocked, quarantined, and runtime-unready outcomes.
- Implementation maturity: 10 mapped stories: 1 implemented and 9 exercised. Projection, pipeline, and maturity transitions are exercised; artifact records exist, but the complete Workbench artifact browser remains preview-only.
- Observed story coverage: 14.1, 14.2, 14.3, 14.4, 14.5, 14.6, 14.7, 14.8, 14.9, 14.10.

## CJ12 — Transfer and Adopt Governed Assets

- User intention: Move verifiable records and assets across cell boundaries while preserving provenance and withholding local usability until governed adoption.
- User job: Preview and export a bounded bundle, verify and import it atomically, keep imported content isolated and inert, and deliberately adopt or reject each asset.
- Initiating condition: A cell administrator needs to export selected records or assets, inspect an external bundle, import it, or decide whether imported content may become locally usable.
- Preconditions: Explicit source and destination cells; selected records, runs, templates, environments, or descriptors; a verifiable manifest, hashes, versions, size bounds, dependencies, and local authority context.
- Entry points: Export preview, bundle import, imported-history view, inert-asset view, provenance and compatibility review, adoption request, and rejection detail.
- Reusable steps: Select the transfer boundary; create and hash the manifest and bundle; verify the entire bundle before admission; import atomically; mark history as external and assets as inactive; inspect source, versions, hashes, dependencies, authority surfaces, and compatibility; authorize adoption; retain provenance or reject without partial state.
- Invoked capabilities: Bundle export and verification, atomic import, imported-history isolation, inactive-asset enforcement, manifest inspection, compatibility checks, governed template or environment adoption, and bounded rejection.
- State transitions: Local selected content becomes an export bundle; verified external bundle becomes isolated imported history and inert assets; an inert verified asset becomes locally usable only after adoption; invalid input becomes rejected with the existing cell unchanged.
- Artifacts: Export selections and previews, bundle manifests, hashes, source-cell identity, imported records and runs, inactive templates and environments, compatibility findings, adoption records, provenance links, and rejection evidence.
- Evidence: Governed UX epic §15; `spec-full.md` §§7.4, 10.9, and 14; `sea-forge-cell` bundle and size-bound tests; CLI closeout tests; bundle manifest implementation.
- Completion condition: The transfer is either rejected atomically or admitted with provenance intact and assets inert; a separately authorized adoption is required before local usability.
- Next decisions: Adopt a compatible asset, keep it inert, repair or replace an incompatible bundle, reject it, or export another bounded selection.
- Recovery paths: Reject tampered, incomplete, oversized, semantically invalid, or incompatible bundles before partial admission; leave the destination unchanged; repeat verification after repair; never count imported history as local capability proof.
- Known variants: Export versus import; records and runs versus templates, environments, extensions, and descriptors; compatible versus incompatible; inert versus adopted; tampered, oversized, incomplete, or semantically invalid rejection.
- Implementation maturity: 7 mapped stories: 1 implemented and 6 exercised. Export, atomic import, isolation, inertness, adoption, and rejection are exercised; provenance and compatibility inspection is implemented at the kernel or CLI boundary while the Workbench federation surface remains preview-only.
- Observed story coverage: 15.1, 15.2, 15.3, 15.4, 15.5, 15.6, 15.7.
