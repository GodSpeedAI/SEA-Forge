# Thoth interaction contract

## Protocol ownership (never blur these)

| Protocol | Responsibility |
|---|---|
| SFWP | Workbench ↔ governed SEA Forge system (all canonical reads, commands, events) |
| AG-UI | Thoth agent ↔ in-application interaction (streams, clarification) |
| CopilotKit | Optional React adapter **for AG-UI only** |
| ACP | SEA Forge ↔ delegated execution agents (`crates/sea-forge-agent/src/acp.rs`) |
| AHP | Optional future multi-client session adapter (deferred) |

None of CopilotKit, AG-UI, ACP, or AHP replaces SFWP. Approval, settlement,
capability, and integrity truth always arrive via SFWP source-backed state.

## Repository grounding

Thoth is implemented and governed today:
`sea_forge_thoth::service::ask` (`crates/sea-forge-thoth/src/service.rs:258`)
is the one shared governance path, called by both the CLI adapter
(`crates/sea-forge-cli/src/commands/ask.rs`) and the server verb
(`Request::Ask`, `crates/sea-forge-server/src/lib.rs`). Typed protocol:
`ThothQuestion` (`protocol.rs:151`), `ThothAnswer` (`:223`), `QuestionKind`
(`:120`), `GroundedClaim` (`:91`), `ClaimClass` (`:41`) with deny-by-default
high-risk classes (`is_high_risk`, `:79`), `Disposition` (`:206`),
`Freshness` (`:214`). Answers carry an authority notice and confer no
execution authority. Denial is a governed outcome, not an error.

Exact frontend type names must be derived from these Rust types via the
generated-contract pipeline — do not invent parallel shapes.

## ThothInteractionPort (SEA Forge-owned boundary)

```ts
interface ThothInteractionPort {
  connect(session: ThothSessionRef): Promise<void>;
  ask(question: ThothQuestionDraft): Promise<RequestRef>;
  respondToClarification(requestId: string, response: unknown): Promise<void>;
  subscribe(listener: (event: ThothInteractionEvent) => void): Unsubscribe;
  disconnect(): Promise<void>;
}
```

(Names grounded against generated types at implementation time.) The port is
the only Thoth surface routes and components may import.

## Adapters

### DirectAgUiAdapter — canonical, always viable
Open AG-UI protocol contracts over the Tauri channel bridge. No proprietary
services. This adapter is the fallback that must keep working regardless of
CopilotKit's fate; CI runs the Thoth flows against it.

### CopilotKitAdapter — optional, OSS-only
May use open-source, locally runnable CopilotKit packages, translated
entirely through `ThothInteractionPort`. May be built first for development
convenience — but its removal must not require rewriting Thoth, SEA Forge
APIs, Workbench route state, canonical messages, approval state, settlement
state, or capability state (this is an eval: see `evals/evaluations.md` #4).

CopilotKit **may** assist with: AG-UI communication, stream assembly,
temporary Thoth presentation state, clarification interaction, rendering
registered SEA Forge result components.

CopilotKit **may not** own: case state, authority decisions, approval
results, execution truth, settlement truth, capability state, integrity
state, canonical evidence.

Type leakage rule: no CopilotKit type appears in canonical SEA Forge crates,
SFWP types, Thoth question/answer records, authority types, settlement
types, or capability types. The adapter file is the leak boundary.

### Explicitly prohibited
Copilot Cloud; enterprise-only CopilotKit features; proprietary hosted
runtimes; proprietary component catalogs; CopilotKit-owned persistence;
CopilotKit authentication as SEA Forge identity; CopilotKit shared state as
canonical state; CopilotKit human-in-the-loop response as approval authority;
arbitrary generated React or HTML.

## Registered rendering union

All generative UI resolves to a registered component union — nothing
free-form ever renders:

```ts
type ThothRenderable =
  | {kind: "grounded-answer";    props: GroundedAnswerProps}
  | {kind: "evidence-list";      props: EvidenceListProps}
  | {kind: "action-paths";       props: ActionPathsProps}
  | {kind: "approval-proposal";  props: ApprovalProposalProps}
  | {kind: "clarification";      props: ClarificationProps};
```

Unknown kinds render a typed "unsupported renderable" fallback — never raw
content, never silent drop.

## Approval and clarification behavior

Clarification, selection, draft edits, and lawful interpretation choices may
flow through the interaction layer. A real governed approval always takes
this path:

```text
UI renders the request (approval-proposal renderable)
→ user selects a response
→ SFWP approval.decide (typed command, preconditions, request_id)
→ SEA Forge checks identity, standing, expiry, separation of duty
→ authoritative result returns
→ UI updates from source-backed state
```

The interaction callback itself never grants approval. Permission denial for
a delegated agent (ACP `PermissionDecision`) is distinct from cancellation;
dialogue end (`AcpTermination`) is distinct from settlement — the UI keeps
all three visually and semantically separate.

## Allowed temporary state

Stream assembly buffers, in-flight clarification context, presentation-only
conversation scroll state. All of it is disposable: closing the surface and
refetching source-backed views loses nothing canonical.
