# SEA Forge product marketing context

**Status:** Working canonical product-marketing context
**Product:** SEA Forge
**Company:** GodSpeed AI
**Primary use:** README copy, website copy, launch materials, developer documentation, enterprise explanations, sales materials, and agent-generated marketing content
**Last updated:** July 29, 2026

---

## 1. Source-of-truth hierarchy

When sources disagree, use this order:

1. The current SEA Forge repository, tests, specifications, license files, commands, and executable behavior.
2. The current SEA Forge README and architecture decisions.
3. This product-marketing context.
4. The GodSpeed AI Brand Voice Style Guide.
5. Strategic architecture and category documents.
6. Older campaign language and exploratory positioning.

Never let marketing language override current implementation, licensing, security boundaries, or technical limitations.

Separate claims into:

* **Evidence-backed:** supported by current code, tests, commands, specifications, repository structure, or license text.
* **Partially proven:** meaningful implementation exists, but production validation or an end-to-end proof remains incomplete.
* **Strategic interpretation:** follows from the architecture but should be presented as positioning, not proof.
* **Roadmap:** planned, specified, or intended but not yet proven.

---

## 2. Product summary

### Plain-language definition

SEA Forge puts enforceable authority and independent proof around consequential AI-agent work.

Before an agent changes a file, calls an API, commits code, modifies infrastructure, delegates work, or touches another protected surface, SEA Forge decides whether that action is allowed.

After the work runs, SEA Forge records what happened and independently checks whether the evidence satisfies the declared outcome.

### Technical definition

SEA Forge is a governed capability-execution kernel.

It turns operator intent into typed operations, evaluates authority before side effects, runs allowed work inside bounded execution environments, records trace and evidence, settles the requested outcome against declared criteria, and appends the resulting capability record to an integrity-protected ledger.

### Smallest useful explanation

> SEA Forge decides whether an agent may act, then checks whether the work actually succeeded.

### One-sentence position

> SEA Forge puts authority before autonomous action and proof after it.

### Sharp public version

> A prompt is not an authority boundary, and “the agent said it worked” is not proof.

### Controlling product truth

> Serious work has two boundaries: permission before action and proof after.

---

## 3. The market wedge

SEA Forge’s first market is not “all AI governance.”

The first wedge is:

> Enforceable authority and evidence for autonomous coding-agent actions.

The initial protected surfaces include:

* file access and modification;
* shell commands;
* external API calls;
* Git commits;
* pull-request merges;
* infrastructure and deployment changes;
* secrets access;
* extension installation;
* policy and evidence mutation;
* delegated agent work.

The buyer does not need to accept the entire syntelligent-infrastructure thesis before trying SEA Forge.

They need to recognize one immediate problem:

```text
Agents are starting to act.
Prompts are being treated as permission.
Completion summaries are being treated as proof.
Neither is an adequate control boundary.
```

### Recommended wedge line

> Put authority between autonomous agents and the systems they want to change.

### Recommended wedge category

> Autonomous-agent authority infrastructure for software delivery.

### Broader category

SEA Forge is part of GodSpeed AI’s larger category:

> Syntelligent infrastructure for governed human-AI work.

Do not lead a technical README with “syntelligent infrastructure.” Establish the concrete agent-action problem first. Introduce the broader category only after the reader understands the wedge.

---

## 4. Why now

AI systems are moving from generating text to changing real systems.

Agents can now:

* write and delete files;
* run commands;
* call internal and external APIs;
* open and merge pull requests;
* modify infrastructure;
* use credentials;
* delegate work to other agents;
* claim that a task is complete.

Most organizations still control these actions through a combination of:

* prompts;
* tool descriptions;
* sandbox configuration;
* repository permissions;
* logs;
* human review;
* agent-generated summaries.

These controls solve different parts of the problem, but they do not form one governed lifecycle.

The resulting gap is:

```text
The agent can act before authority is resolved.
The process can return zero without satisfying the requested outcome.
The agent can narrate success without independent acceptance.
The logs can show activity without proving completion.
```

As agent capability rises, this gap becomes more consequential.

The durable bottleneck is no longer generation alone. It is authority, judgment, evidence, verification, recovery, and trust.

---

## 5. Primary audiences

### 5.1 Primary technical user

**Who they are**

* platform engineers;
* AI infrastructure engineers;
* staff and principal engineers;
* developer-experience engineers;
* technical founders;
* engineers operating coding-agent infrastructure.

**Current situation**

They already use Claude Code, Codex, OpenCode, Cline, custom agents, agent SDKs, or similar systems to perform real software work.

They are comfortable with:

* Git;
* APIs;
* command-line tools;
* policies;
* permissions;
* tests;
* containers and sandboxes;
* CI/CD;
* structured logs.

They are skeptical of:

* vague governance claims;
* new terminology without a mechanism;
* dashboards that do not enforce anything;
* “trust the agent” operating models;
* heavy platforms that replace their agent or workflow.

**What they want**

* to keep using their preferred agents and models;
* to govern actions without rebuilding their toolchain;
* to know why an operation was allowed, denied, or escalated;
* to inspect the evidence behind a claimed result;
* to integrate governance into existing developer workflows;
* to add controls without turning every action into a meeting.

### 5.2 Primary economic buyer

**Who they are**

* VP or Head of Engineering;
* CTO;
* platform-engineering leader;
* security leader;
* AI-governance leader;
* enterprise architecture leader;
* regulated-software leader.

**What they need to know**

* what agents are allowed to do;
* which policy authorized an action;
* what actually happened;
* whether the requested outcome was achieved;
* what evidence exists;
* whether the record can be independently inspected;
* how human approval and escalation work;
* whether the system fails closed;
* how deployment and commercial rights work.

**Emotional state**

They want the speed and leverage of autonomous agents without becoming the executive who approved uncontrolled production access.

They do not want another policy document that becomes relevant only after an incident.

### 5.3 Important secondary audiences

* security and compliance engineers;
* quality and assurance leaders;
* AI Delivery Engineers;
* Forward-Deployed Engineers;
* regulated product teams;
* government and defense contractors;
* healthcare technology teams;
* manufacturing technology teams;
* auditors evaluating agentic work;
* Domain Engineers binding policies to executable business meaning.

Do not write one paragraph that attempts to address all audiences at once.

For the README:

* lead for the technical user;
* provide assurance for the technical and economic buyer;
* preserve enough depth for security and governance review.

---

## 6. Audience knowledge and emotional state

Assume the reader understands:

* AI agents;
* tool calling;
* repositories;
* APIs;
* shell commands;
* permissions;
* sandboxes;
* tests;
* logs;
* CI/CD.

Do not assume the reader understands:

* settlement;
* semantic capability envelopes;
* CMMN;
* Thoth;
* IFL attestation;
* capability metabolization;
* the authority fabric;
* the GodSpeed product stack.

The reader is:

* interested but skeptical;
* scanning quickly;
* tired of generic AI language;
* suspicious of broad platform claims;
* concerned about operational risk;
* unwilling to learn a proprietary ontology before seeing practical value.

The first screen of copy must reduce representational burden. It should translate the product into a familiar failure, mechanism, and result.

---

## 7. Core jobs to be done

### Functional jobs

When I let agents perform consequential software work, help me:

1. define what each actor may do;
2. decide authority before side effects occur;
3. stop unknown or disallowed actions;
4. escalate risky actions for human approval;
5. constrain allowed execution;
6. record what actually happened;
7. collect artifacts, hashes, validators, traces, and transcripts;
8. determine whether evidence satisfies the requested outcome;
9. distinguish process success from outcome success;
10. preserve complete records for accepted, denied, rejected, failed, cancelled, and escalated work;
11. explain why an action was denied;
12. revalidate past evidence;
13. delegate to other agents without losing authority or accountability;
14. operate through a CLI, server, or machine-readable event stream;
15. retain enough evidence to audit, recover, learn, and improve.

### Emotional jobs

Help me feel:

* confident that agent speed is not outrunning control;
* able to explain what happened after a run;
* less dependent on agent narration;
* less likely to discover unauthorized changes after the fact;
* credible when security, management, or an auditor asks for proof;
* comfortable expanding agent access gradually.

### Social jobs

Help me:

* support agent adoption without appearing reckless;
* introduce governance without appearing anti-innovation;
* demonstrate that agent use is controlled rather than improvised;
* give security and engineering a shared operational record;
* approve autonomy based on evidence rather than enthusiasm.

---

## 8. Core customer problems

### 8.1 Prompts are being used as permission systems

Prompts can express instructions, preferences, and constraints.

They do not independently enforce authority.

Without a separate authority layer, the same system that interprets the instruction also decides how to act on it.

Customer language:

* “We told the agent not to touch that directory.”
* “The system prompt says it must ask first.”
* “It should only use approved tools.”
* “The agent knows the rules.”

Underlying problem:

> A behavioral instruction is being treated as an enforceable permission boundary.

### 8.2 Sandboxes are being confused with governance

A sandbox can restrict filesystem, network, process, and resource access.

It does not determine whether a specific actor should perform a specific operation against a specific resource under the current policy and context.

Underlying problem:

> Containment limits the blast radius. Authority decides whether the action should happen.

SEA Forge treats authority and sandboxing as separate controls that reinforce rather than replace each other.

### 8.3 Process success is being confused with outcome success

A command can exit zero while:

* the wrong artifact was produced;
* a required artifact is missing;
* the output violates the specification;
* validation was never run;
* the agent solved a different problem;
* the change cannot survive review.

Underlying problem:

> A successful process does not prove a successful job.

SEA Forge calls the independent evaluation of evidence against the declared outcome **settlement**.

### 8.4 Agents are allowed to grade their own work

Agent completion summaries are useful context, but they are not independent proof.

Underlying problem:

> The actor doing the work is also being allowed to declare whether the work succeeded.

SEA Forge treats agent output as untrusted input. Acceptance depends on declared criteria and evidence, not narration.

### 8.5 Logs show activity but not meaning

Traditional logs may show:

* commands;
* timestamps;
* messages;
* return codes;
* API calls.

They often cannot answer:

* what outcome was declared;
* which authority applied;
* what evidence was required;
* whether the outcome was accepted;
* why the work was rejected;
* what capability was demonstrated.

Underlying problem:

> Activity records are being mistaken for outcome records.

### 8.6 Delegation creates authority ambiguity

When one agent delegates to another, teams need to know:

* which actor had authority;
* what authority was delegated;
* what limits applied;
* what the delegated agent did;
* who can cancel the work;
* which evidence supports acceptance.

Underlying problem:

> Agent chains can diffuse responsibility faster than they create accountability.

### 8.7 Governance often arrives after the action

Many governance systems inventory, report, review, or audit after execution.

Underlying problem:

> Post-action visibility does not prevent an unauthorized side effect.

SEA Forge evaluates authority before execution and records evidence afterward.

---

## 9. Status quo and alternatives

SEA Forge competes first with the status quo, not only with named products.

### 9.1 Prompt rules

**Why buyers use them**

* easy;
* already available;
* low setup cost;
* agent-specific;
* flexible.

**Where they fail**

* advisory rather than independently enforceable;
* coupled to model behavior;
* inconsistent across agents and providers;
* weak evidence of which rule applied;
* easy to bypass through ambiguity, tool behavior, or delegation.

**SEA Forge position**

> Prompts guide behavior. SEA Forge governs action.

### 9.2 Repository and operating-system permissions

**Why buyers use them**

* mature;
* enforceable;
* familiar;
* already part of production infrastructure.

**Where they fall short**

* often too coarse;
* do not represent operator intent;
* do not evaluate outcome criteria;
* do not create one case-level authority and settlement record;
* do not explain whether a completed action satisfied the requested result.

**SEA Forge position**

> Existing permissions remain valuable. SEA Forge adds actor, operation, resource, context, evidence, and outcome semantics around them.

### 9.3 Containers and sandboxes

**Why buyers use them**

* isolation;
* reproducibility;
* reduced blast radius;
* resource and network controls.

**Where they fall short**

* containment is not authorization;
* a safely contained action can still be disallowed;
* a command can run safely and still produce the wrong result;
* sandboxes do not independently settle outcomes.

**SEA Forge position**

> The sandbox controls where allowed work runs. SEA Forge decides whether it may run and whether it succeeded.

### 9.4 Logs, traces, and observability platforms

**Why buyers use them**

* visibility;
* debugging;
* monitoring;
* incident analysis.

**Where they fall short**

* record events without a declared outcome contract;
* may not bind actions to authority decisions;
* may not distinguish command success from accepted outcome;
* may be mutable or fragmented across tools.

**SEA Forge position**

> Observability tells you what the system emitted. SEA Forge records what was authorized, what happened, what evidence exists, and whether the outcome was accepted.

### 9.5 Policy engines

**Why buyers use them**

* centralized rules;
* allow and deny decisions;
* mature policy languages;
* integration with infrastructure.

**Where they fall short**

* may end at the decision;
* do not necessarily own the complete execution lifecycle;
* may not collect evidence or settle declared outcomes;
* may not create a durable capability record.

**SEA Forge position**

> SEA Forge is not only a policy decision point. It composes authority, execution, evidence, settlement, and recording around the work.

Do not claim that SEA Forge replaces general policy engines. It may consume or normalize governance verdicts while preserving one authoritative lifecycle decision.

### 9.6 Agent frameworks and orchestration systems

**Why buyers use them**

* planning;
* tool calling;
* memory;
* multi-agent workflows;
* provider integrations.

**Where they fall short**

* optimize coordination and completion;
* often trust agent-level assertions;
* authority controls vary by framework;
* evidence and settlement are not always first-class;
* switching frameworks may fragment governance.

**SEA Forge position**

> SEA Forge governs agents; it is not another agent runtime.

External agents remain replaceable executors inside the governed lifecycle.

### 9.7 Human approval

**Why buyers use it**

* judgment;
* accountability;
* existing organizational control.

**Where it fails**

* approvals arrive without enough evidence;
* humans become throughput bottlenecks;
* approval rules are inconsistent;
* reviewers cannot reconstruct what changed;
* every action is escalated because the system lacks bounded authority.

**SEA Forge position**

> Human approval should be used for real judgment, not as a substitute for missing infrastructure.

### 9.8 AI governance dashboards and inventories

**Why buyers use them**

* centralized reporting;
* policy documentation;
* compliance coordination;
* executive visibility.

**Where they fall short**

* governance may remain descriptive;
* controls may not execute before actions;
* evidence may be assembled manually after the fact;
* dashboards can show a policy without enforcing it.

**SEA Forge position**

> A governance record matters most when it changes what the system is allowed to do.

---

## 10. Positioning

### Positioning statement

For engineering and platform teams deploying autonomous agents into consequential software environments, SEA Forge is a governed capability-execution kernel that evaluates authority before side effects and independently settles outcomes from evidence.

Unlike prompts, sandboxes, logs, policy dashboards, and agent frameworks used alone, SEA Forge composes authorization, bounded execution, evidence, settlement, and integrity-protected records into one lifecycle.

### Category ladder

Use categories in this order:

1. **Concrete wedge:** authority and proof for autonomous coding-agent actions;
2. **Product category:** governed capability-execution kernel;
3. **Market category:** governed autonomy infrastructure;
4. **Company category:** syntelligent infrastructure.

Do not force the reader to understand level four before level one.

### Core contrast

```text
Typical agent loop:
Prompt → tool call → output → agent claims completion

SEA Forge loop:
Intent → plan → authority → bounded execution → evidence → settlement → record
```

### Product promise

> Let agents act without letting them invent their own permissions or grade their own work.

### Strategic value

SEA Forge helps organizations move from:

* prompt-based control to enforceable authority;
* action logs to evidence-bearing execution;
* return codes to outcome settlement;
* isolated agent runs to auditable capability records;
* blanket restriction to bounded, inspectable autonomy;
* governance after action to governance before action.

---

## 11. Unique mechanism

SEA Forge’s differentiation is not one feature. It is the composition of the lifecycle.

### 11.1 Authority before execution

Every declared operation is evaluated before its side effect runs.

Unknown operations deny by default. Policy-load failure does not create a fail-open path.

### 11.2 Authority and isolation remain separate

Authority decides whether work may happen.

Sandboxing constrains how and where allowed work happens.

Neither control silently substitutes for the other.

### 11.3 Evidence is a first-class product output

SEA Forge records evidence such as:

* execution results;
* artifact descriptors;
* content hashes;
* validators;
* trace events;
* agent transcripts;
* policy and decision references.

### 11.4 Settlement is independent of process success

Settlement evaluates evidence against declared criteria.

A process can exit zero and still be rejected.

A process failure, denial, cancellation, or escalation still produces a complete record.

### 11.5 Every path leaves a record

Accepted work is not the only work worth recording.

SEA Forge preserves records for:

* allowed;
* denied;
* failed;
* cancelled;
* rejected;
* escalated;
* awaiting approval;
* accepted.

### 11.6 Agent delegation is ordinary governed work

Delegated tasks do not receive a separate trust model.

They remain:

* authority checked;
* turn capped;
* cancellable;
* transcript evidenced;
* independently settled.

### 11.7 Integrity is built into the record

The ledger uses append-only records, content hashing, chained integrity structures, signed checkpoints, and witness mechanisms according to the configured assurance level.

Do not summarize all of these mechanisms as “blockchain.”

### 11.8 Self-knowledge is evidence grounded

SEA Forge includes a versioned self-model and a governed interpreter, Thoth.

Thoth can answer questions about SEA Forge’s declared, observed, and demonstrated capabilities using evidence-linked claims.

Thoth’s knowledge does not grant authority.

---

## 12. Feature-to-outcome translation

Do not market features without their consequence.

| Capability               | Reader consequence                                                                                    |
| ------------------------ | ----------------------------------------------------------------------------------------------------- |
| Default deny             | Unknown operations do not inherit permission from a vague prompt.                                     |
| Pre-action authority     | The system decides before the side effect, not during the incident review.                            |
| Separate sandbox control | Allowed work remains bounded without pretending containment equals permission.                        |
| Evidence collection      | Completion claims can be inspected against artifacts, traces, validators, and hashes.                 |
| Settlement               | A zero exit code cannot silently substitute for the requested result.                                 |
| Complete path records    | Denials, failures, cancellations, and escalations remain explainable.                                 |
| Integrity ledger         | Records can be checked for tampering and independently verified according to assurance level.         |
| Human approval workflow  | Risky operations can pause and resume rather than being automatically allowed or permanently blocked. |
| Governed delegation      | Multi-agent work does not dissolve authority or evidence requirements.                                |
| Capability memory        | Accepted prior work can be recalled as evidence without converting knowledge into permission.         |
| Self-model and Thoth     | The system can explain what it claims to support and show the evidence behind the claim.              |
| DomainForge integration  | Plans and policies can bind to canonical domain concepts rather than loose strings and folklore.      |
| Server and event stream  | Long-running and concurrent workloads can use the same governed lifecycle as the CLI.                 |
| Revalidation             | Past evidence can be checked again instead of trusting the original completion claim forever.         |

---

## 13. Product stack boundaries

### DomainForge

DomainForge owns:

* `.sea` syntax;
* semantic graphs;
* concept identities;
* validation;
* deterministic projections;
* executable domain meaning.

Plain explanation:

> DomainForge defines the world the work is happening in.

### SEA Forge

SEA Forge owns:

* authorization;
* isolation;
* side effects;
* trace;
* evidence;
* settlement;
* governed records;
* capability-execution lifecycle.

Plain explanation:

> SEA Forge governs action inside that world.

### SWE_SEED

SWE_SEED provides a lightweight proof-oriented software-work harness.

It helps agents:

* follow a route;
* load context;
* produce the required artifact;
* run proof commands;
* leave useful traces.

Plain explanation:

> SWE_SEED proves whether the software work was done.

Do not imply that SEA Forge depends on SWE_SEED for all settlement.

### GodSpeed-Agent

GodSpeed-Agent owns the developmental layer:

* desired direction;
* horizon review;
* payment estimation;
* affordance selection;
* developmental memory;
* metabolization;
* horizon change.

Plain explanation:

> GodSpeed-Agent turns repeated settlement into capability development.

### Canonical stack formula

> DomainForge defines the domain.
> SEA Forge governs the work.
> SWE_SEED proves the change.
> GodSpeed-Agent compounds the capability.

For SEA Forge’s README, explain SEA Forge fully before presenting the entire stack.

---

## 14. Differentiators

### 14.1 Governed lifecycle, not isolated controls

SEA Forge connects authority, execution, evidence, settlement, and recording.

The differentiation is the composition.

### 14.2 Outcome acceptance, not command completion

The core object is the declared outcome and its evidence, not merely the executed process.

### 14.3 Executor independence

SEA Forge is not tied conceptually to one model, provider, agent framework, or execution backend.

The executor performs the work. SEA Forge governs the work.

Do not claim universal compatibility without implemented adapters or protocol proof.

### 14.4 Fail-closed behavior

Missing policy, unknown operations, absent required evidence, and unresolved policy conflicts do not quietly become permission.

### 14.5 Complete negative-path evidence

Denied and failed work remains part of the truth record.

This matters for debugging, audit, recovery, and policy improvement.

### 14.6 Domain-grounded authority

Through DomainForge integration, authority and settlement can bind to stable domain concepts and models rather than relying only on unstructured names.

### 14.7 Evidence-grounded self-description

Declared capability, observed capability, and demonstrated capability remain distinct.

The system should not report a declared feature as demonstrated without evidence.

### 14.8 Local-first and inspectable posture

The product is designed around local execution, explicit files, inspectable records, offline foundation gates, and deployment control.

Do not expand this into an unsupported universal privacy or air-gap claim.

---

## 15. Messaging hierarchy

Use this order when explaining SEA Forge.

### Message 1: The problem

> Agents can now change real systems, but most teams still govern them with prompts and trust their completion summaries.

### Message 2: The correction

> A prompt is not an authority boundary. A successful process is not a successful outcome.

### Message 3: The mechanism

> SEA Forge decides authority before side effects, records evidence during execution, and independently checks whether the declared outcome was achieved.

### Message 4: The practical result

> Teams can grant bounded autonomy without losing inspectability, approval, or proof.

### Message 5: The broader significance

> Governed, evidenced work can become reusable organizational capability rather than another isolated agent run.

Do not begin with Message 5.

---

## 16. Approved message formulations

### Primary headline direction

> Authority before action. Proof before acceptance.

### Alternative headline

> Let agents act. Do not let them invent permission or grade their own work.

### Primary subhead

> SEA Forge evaluates what an agent is allowed to do before execution, contains allowed work, records the evidence, and accepts the outcome only when the declared criteria are met.

### Technical one-liner

> A governed capability-execution kernel for authority, evidence, settlement, and durable execution records.

### Enterprise one-liner

> SEA Forge gives autonomous-agent work an enforceable authority and evidence layer.

### Developer one-liner

> Put policy before the tool call and proof after the run.

### Sharp lines

* A prompt is not an authority boundary.
* “The agent said it worked” is not an assurance model.
* A passing process is not necessarily a passing outcome.
* Completion without proof is a confident rumor.
* A sandbox limits movement. It does not grant permission.
* If an agent claims it completed the work, it should bring receipts.
* Governance that cannot stop an action before it happens is mostly documentation.
* Agent output is evidence input, not a settlement decision.
* Fast work without evidence is noise with a commit hash.
* Knowledge is not authority.
* The agent is not the product. The governed capability loop is the product.

Use one sharp line at a time. Do not turn every paragraph into an aphorism.

---

## 17. Voice and tone

SEA Forge copy should sound:

* rigorous;
* plainspoken;
* technically credible;
* anti-theater;
* pro-evidence;
* calm under scrutiny;
* slightly sharp when attention is scarce;
* respectful of people;
* skeptical of weak systems.

The voice should sound like:

> A systems architect with enough operational experience to know that confidence, logs, and green checks are not interchangeable with authority and proof.

### Use Speed Mode for

* headlines;
* opening paragraphs;
* category contrasts;
* launch posts;
* developer attention;
* memorable warnings.

### Use Journey Mode for

* mechanisms;
* security explanations;
* enterprise content;
* architecture;
* licensing;
* risk;
* implementation maturity;
* evaluation guidance.

### Humor boundary

Mock the absurd operating model, not the team trapped inside it.

Good:

> Most teams were given autonomous agents before they were given authority infrastructure. That is a design problem, not a character flaw.

Bad:

> Companies using prompt rules have no idea what they are doing.

---

## 18. Language rules

### Prefer

* authority;
* evidence;
* declared outcome;
* accepted;
* rejected;
* denied;
* escalated;
* bounded execution;
* governed run;
* policy;
* criteria;
* trace;
* record;
* inspect;
* verify;
* explain;
* settlement;
* capability;
* domain meaning;
* fail closed;
* operator;
* actor;
* operation;
* resource;
* context.

### Explain before relying on

* settlement;
* semantic capability envelope;
* authority fabric;
* CMMN;
* Thoth;
* self-model;
* IFL;
* ADLC;
* ODI;
* sentry;
* capability metabolization;
* syntelligent infrastructure.

### Avoid

* seamless;
* revolutionary;
* cutting-edge;
* unlock;
* empower;
* supercharge;
* magic;
* effortless;
* game-changing;
* fully autonomous enterprise;
* enterprise-grade without supporting evidence;
* trust the agent;
* AI-powered solution;
* single pane of glass;
* end-to-end when the actual boundary is narrower;
* open source when describing SEA Forge;
* AGPL when describing SEA Forge.

### Canonical terminology

After a technical term is defined, use it consistently.

Do not cycle among “acceptance,” “validation,” “verification,” “certification,” and “settlement” as if they are interchangeable.

* **Validation** checks an artifact or input against rules.
* **Evidence** records support for a claim.
* **Settlement** decides whether the evidence satisfies the declared outcome.
* **Acceptance** is the successful settlement result.

---

## 19. Common objections

### “We already sandbox our agents.”

Response:

Sandboxing is necessary, but it answers a different question.

A sandbox controls where and how a command may run. It does not establish whether this actor may perform this operation against this resource under the current policy. It also does not prove the requested outcome was achieved.

SEA Forge composes authority, sandboxing, evidence, and settlement.

### “We already have IAM and repository permissions.”

Response:

Keep them.

SEA Forge does not need to replace infrastructure permissions. It adds case-level intent, typed operations, policy decisions, evidence requirements, settlement criteria, and durable records around the work.

IAM can determine whether an identity has access. SEA Forge determines whether this governed operation should proceed in this context and whether its outcome was accepted.

### “Our agent asks for confirmation.”

Response:

Agent-mediated confirmation is useful interaction design, but the agent should not be the sole interpreter, enforcer, and recorder of its own authority.

SEA Forge makes approval a governed state with explicit policy, evidence, identity, expiry, and resumption behavior.

### “Our CI already proves the code works.”

Response:

CI is valuable evidence.

SEA Forge can treat tests and proof commands as settlement inputs, but a governed run also needs to know what was requested, what operations were authorized, what artifacts were produced, what evidence applies, and whether the declared outcome was satisfied.

SEA Forge does not replace CI. It gives CI evidence standing inside a broader governed lifecycle.

### “This sounds like a policy engine.”

Response:

Policy evaluation is one part of SEA Forge.

SEA Forge continues through execution, evidence, settlement, capability records, recall, approvals, cancellation, revalidation, and integrity verification.

The policy decision does not end the lifecycle.

### “This sounds like an agent framework.”

Response:

SEA Forge does not need to be the agent’s planning or conversation runtime.

Agents are replaceable executors and governed participants. SEA Forge controls the lifecycle around their consequential actions.

### “Will this slow agents down?”

Response:

Any real control adds some cost.

The alternative is to pay later through blanket restrictions, manual review, unclear incidents, weak evidence, or unauthorized changes.

The goal is not maximum friction. It is bounded autonomy: routine actions proceed under declared authority, risky actions escalate, and every result remains inspectable.

Do not promise zero latency or no productivity impact without benchmark evidence.

### “Can agents bypass it?”

Response:

The honest answer depends on deployment topology.

SEA Forge can govern actions routed through its controlled execution and policy surfaces. It cannot govern side effects that occur through unmediated paths outside its authority boundary.

Marketing must state the boundary rather than implying magical universal control.

### “Is SEA Forge production proven?”

Response:

Do not answer with a blanket yes.

Describe the specific implemented capability, test evidence, deployment mode, and unproven boundary.

Use evidence-backed maturity labels. Do not promote roadmap architecture into production proof.

### “Why not wait for model providers to solve this?”

Response:

Provider safety controls matter, but enterprise authority depends on organization-specific actors, resources, policies, environments, evidence, and outcome criteria.

Models and providers are replaceable. Organizational authority cannot be delegated entirely to each model vendor.

### “Why is this source-available rather than open source?”

Response:

SEA Forge is inspectable and available for permitted internal, personal, educational, research, evaluation, and non-commercial uses under the Sustainable Use License.

Commercial rights are required for specified hosted, embedded, managed, white-labeled, client-facing, redistributed, enterprise-only, or third-party operational uses.

Use the license files for exact legal terms.

---

## 20. Proof inventory

Marketing may point to these proof surfaces when they are present in the current repository.

### Core lifecycle proof

* typed case and plan records;
* pre-action authority decisions;
* bounded execution;
* structured trace;
* evidence collection;
* settlement against declared criteria;
* semantic capability envelopes;
* records for negative and positive paths.

### Authority proof

* default-deny handling;
* policy-load failure behavior;
* deterministic verdict resolution;
* explicit deny, escalate, and allow states;
* missing-evidence behavior;
* identity and role resolution;
* separation-of-duty rules.

### Execution proof

* argv-based process execution;
* explicit environments;
* time bounds;
* sandbox classes;
* path validation;
* network controls;
* cancellation behavior.

### Integrity proof

* content hashing;
* append-only records;
* chained ledger integrity;
* checkpoint signing;
* witness receipts;
* inclusion and consistency proofs;
* assurance-level reporting.

### Agent proof

* registered provider seam;
* provider-bound grants;
* turn caps;
* transcript evidence;
* cancellation;
* independent settlement;
* sequential and concurrent orchestration templates;
* bounded Thoth manager iterations.

### Self-knowledge proof

* bundled canonical `.sea` self-model;
* declared, observed, and demonstrated views;
* evidence-linked capability answers;
* disclosure control before retrieval;
* denial explanation;
* environment-status queries.

### Development and reproducibility proof

* pinned Rust toolchain;
* Devbox environment;
* one `just` command surface;
* `just doctor`;
* `just check`;
* `just test`;
* `just proof`;
* checked-in hooks;
* locked dependency checks;
* secret scanning;
* SOPS and age handling;
* offline foundation gates.

### Server proof

* Unix-socket server;
* concurrent case dispatch;
* approval workflows;
* cancellation;
* event subscription;
* policy reload;
* last-known-good behavior where implemented;
* bounded request handling;
* typed status and error records.

Only use proof that remains accurate in the current branch.

---

## 21. Claims and maturity boundaries

### Safe evidence-backed positioning

Use when confirmed by the current repository:

* SEA Forge evaluates authority before declared side effects.
* Unknown operations deny by default.
* Authority and sandboxing are separate controls.
* Runs produce trace, evidence, settlement, and capability records.
* Settlement can reject a run even when its process exits zero.
* Denied and failed runs remain recorded.
* The ledger provides configurable integrity and verification mechanisms.
* Agent delegations use the governed lifecycle.
* SEA Forge includes a versioned self-model and evidence-linked Thoth queries.
* DomainForge owns semantic modeling; SEA Forge owns governed materialization and execution.
* SEA Forge is source-available under the Sustainable Use License.

### Claims requiring qualification

* production ready;
* enterprise ready;
* air-gapped;
* model agnostic;
* framework agnostic;
* provider agnostic;
* universal policy enforcement;
* complete multi-agent governance;
* multi-cell governance;
* federation;
* tamper proof;
* compliance ready;
* autonomous organization;
* self-improving;
* capability compounding.

Replace universal claims with the implemented boundary.

Example:

Bad:

> SEA Forge governs every agent action across the enterprise.

Better:

> SEA Forge governs actions routed through its declared execution and authority surfaces.

### Roadmap or strategic claims

Do not present these as complete unless current repository evidence proves them:

* production-proven multi-cell governance;
* active-active global authority consensus;
* universal runtime interception;
* complete organization-twin operation;
* broad regulatory certification;
* public capability marketplaces;
* automatic capability promotion without governed evidence;
* universal agent compatibility;
* full GUI or visual workflow builder;
* autonomous enterprise operation.

---

## 22. Licensing and commercial context

SEA Forge is source-available under the SEA-Forge Sustainable Use License.

Use the current license files as the legal authority.

### Approved summary

> SEA Forge is source-available for trust, inspection, permitted internal use, research, education, evaluation, and experimentation. Separate commercial rights apply to specified hosted, embedded, redistributed, white-labeled, managed, client-facing, enterprise-only, or third-party operational uses.

### Photoshop Principle

> You own what you create with the tool. You do not get to resell the tool itself.

User-generated outputs are not automatically SEA Forge-licensed merely because SEA Forge helped create them, provided those outputs do not copy, contain, host, embed, redistribute, or white-label SEA Forge or restricted components in ways requiring a commercial license.

### Never say

* SEA Forge is AGPL;
* SEA Forge is OSI open source;
* anyone may host or resell SEA Forge;
* all commercial use is automatically allowed;
* all internal enterprise use is automatically restricted;
* generated outputs are owned by GodSpeed AI.

When precision matters, quote or link the license rather than paraphrasing from memory.

---

## 23. Adoption path

### Stage 1: Govern one high-consequence action

Start with:

* one agent;
* one repository;
* one protected path or operation;
* one allowed action;
* one denied action;
* one escalated action;
* one evidence record;
* one settlement result.

This is the smallest credible market proof.

### Stage 2: Expand the action surface

Add policy for:

* files;
* APIs;
* commits;
* pull requests;
* secrets;
* infrastructure;
* extensions;
* delegated agent tasks.

### Stage 3: Add proof discipline

Bind the work to:

* required artifacts;
* validators;
* tests;
* hashes;
* proof commands;
* settlement criteria.

SWE_SEED may serve as a companion workflow for software proof.

### Stage 4: Bind domain meaning

Use DomainForge when policies and settlement criteria depend on stable business concepts, semantic models, or projections.

### Stage 5: Add operational memory

Use accepted records, capability recall, the self-model, and Thoth to improve inspection and reuse without treating past knowledge as present authority.

### Stage 6: Expand deployment and governance

Only after the local lifecycle is proven should buyers expand toward:

* long-running server operation;
* more agents;
* more teams;
* broader policy surfaces;
* federation;
* cell-based deployment;
* organization-level capability infrastructure.

---

## 24. Primary use cases

### Governed coding-agent work

Control file writes, shell commands, commits, pull requests, tests, and generated artifacts.

### Governed infrastructure changes

Require authority, approvals, bounded execution, evidence, and settlement before infrastructure work is accepted.

### Governed API access

Bind an agent’s external calls to approved destinations, models, credentials, limits, and evidence requirements.

### Multi-agent delegation

Preserve authority, limits, cancellation, transcripts, evidence, and independent settlement across agent chains.

### Governed spec-to-code pipelines

Track the progression from requirements and design artifacts through generated code and proof, with evidence at each stage.

### Auditable automation

Produce inspectable records that show:

* what was requested;
* what was authorized;
* what ran;
* what evidence was produced;
* how the outcome was decided.

### Evidence-grounded system self-description

Ask what the installed system supports, what it has observed, what it has demonstrated, and why an operation was denied.

---

## 25. README-specific guidance

The README must not begin with the full product ontology.

Its upper section should answer, in order:

1. What familiar failure does this solve?
2. Why are prompts, sandboxes, logs, and exit codes insufficient?
3. What does SEA Forge do differently?
4. What practical result does that produce?
5. What can the reader run to see it?

Recommended upper-page sequence:

1. sharp headline;
2. plain-language explanation;
3. concrete failure pattern;
4. before-and-after lifecycle;
5. smallest working demonstration;
6. core guarantees translated into consequences;
7. architecture and technical depth;
8. installation, contribution, and license reference.

### Ten-second outcome

The reader should understand:

> SEA Forge decides whether an agent may act and checks whether the work actually met the requested outcome.

### Sixty-second outcome

The reader should understand:

* prompt instructions are not enforceable authority;
* sandboxes and authority solve different problems;
* zero exit does not prove outcome success;
* agent narration is not independent acceptance;
* SEA Forge produces authority, trace, evidence, settlement, and integrity records;
* they can inspect and revalidate a governed run.

### README conversion action

For a public developer README, the primary action is:

> Run the smallest governed lifecycle and inspect its result.

Secondary actions:

* review the architecture;
* run the proof suite;
* inspect the license;
* evaluate a protected agent-action pilot.

Do not use a generic “Contact us” CTA as the main developer action.

---

## 26. Content acceptance tests

Before publishing SEA Forge copy, verify that the reader can answer:

1. What does SEA Forge prevent?
2. What does it govern?
3. Why is a prompt not an authority boundary?
4. Why is a sandbox not authorization?
5. Why is a zero exit code not acceptance?
6. What evidence does the system preserve?
7. How is an outcome accepted or rejected?
8. What happens when work fails, is denied, or is cancelled?
9. What is the smallest useful deployment?
10. What does SEA Forge replace?
11. What does it complement rather than replace?
12. What remains outside its boundary?
13. What is implemented?
14. What is partially proven?
15. What is roadmap?
16. How is it licensed?
17. What should the reader do next?

Apply four final tests:

### So what?

Does each technical capability connect to a reader consequence?

### Prove it

Can each claim be supported by repository evidence?

### Boundary

Does the copy state where SEA Forge’s authority begins and ends?

### Compression

Could a technically competent reader explain SEA Forge in one plain sentence after reading the opening?

---

## 27. Final strategic summary

SEA Forge should not be marketed as another AI agent, agent framework, sandbox, governance dashboard, or logging product.

Its immediate value is narrower and more concrete:

> It places enforceable authority before consequential agent actions and independent proof after execution.

Its product-level differentiation is the governed lifecycle:

```text
intent
→ plan
→ authority
→ bounded execution
→ evidence
→ settlement
→ durable record
```

Its broader strategic role is:

> SEA Forge is the governed-action layer of GodSpeed AI’s syntelligent infrastructure.

Lead with the immediate failure.

Prove the mechanism.

Earn the larger category.
