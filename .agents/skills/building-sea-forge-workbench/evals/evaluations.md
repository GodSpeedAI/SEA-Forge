# Evaluations — building-sea-forge-workbench

Each evaluation is a prompt given to a fresh coding agent with this skill
loaded, plus the expected behaviors a reviewer checks. An evaluation passes
only if every expected behavior is observed and no prohibited behavior occurs.

## Evaluation 1 — Readiness vertical slice

**Prompt:** "Implement the Readiness screen: show operation-sensitive
readiness for the selected intended operation, with freshness and integrity,
and a governed reason when case creation is unavailable."

**Expected behavior:**
- Inspects actual readiness-relevant contracts (`crates/sea-forge-server/src/lib.rs`
  `Request` enum, `crates/sea-forge-core/src/types.rs`) before writing code;
  classifies the readiness query as MISSING-to-PARTIAL with `file:line`
  evidence and adds a view-shaped query rather than a fake.
- Uses the Tauri command/query bridge (no renderer socket access).
- Uses Astryx primitives plus SEA Forge semantic components
  (`GovernedStatusPill`, `SourceFreshnessBadge`, `ProtectedActionButton`,
  `WhyStatePanel`) — no generic card dashboard.
- Preserves operation-sensitive readiness (readiness varies with the selected
  operation; no single universal health score).
- Does not fabricate backend state; the demo runs against a real
  `sea-forge-server` on a temp root.
- Freshness and integrity metadata visible; degraded state remains usable
  with a named limitation and a disabled case-creation path with reason.
- Adds bridge, component, route, and a11y tests; gates green.

**Fails if:** invents a readiness endpoint response shape with no server
change, connects the renderer to the socket, collapses states into generic
success/warning/error, or ships without tests.

## Evaluation 2 — Case preflight and commit

**Prompt:** "Wire case authoring: draft → preflight → commit, handling stale
preflight and ambiguous submission."

**Expected behavior:**
- Finds existing submit/commit substrate (`Request::Submit`, `SubmitPayload`)
  and digest/hash conventions (e.g. `criteria_record_hash`,
  `compute_record_hash` in `crates/sea-forge-planner/`) before designing.
- Separates draft (local, reversible) / preflight (evaluate) / commit
  (protected command) — three states, three surfaces, one XState machine.
- Stale handling: precondition digests captured at preflight; commit after a
  policy/plan change yields `rejected_as_stale` rendered as a repair path
  that re-preflights — never a silent retry.
- Ambiguity handling: transport loss after commit submission enters a
  status-recovery state (`request.get_status`) before any retry state is
  reachable.
- Does not invent a generic mutation API; the commit is one typed method.
- Machine tests prove no duplicate commit is reachable from the ambiguous
  state; server test proves no side effect on stale rejection.

**Fails if:** commit is optimistic, retry skips status recovery, drafts write
into `.sea-forge/`, or a generic `mutate(payload)` appears.

## Evaluation 3 — Agent permission denial

**Prompt:** "Surface a delegated agent's permission request in the Agent Task
console and let the operator deny it without cancelling the run."

**Expected behavior:**
- Uses the existing ACP owners: `AcpPermissionRequest`, `PermissionDecision`,
  `AcpPermissionMediator` (`crates/sea-forge-agent/src/acp.rs:49-93`) and the
  server delegation path (`crates/sea-forge-server/src/delegation.rs`) —
  no parallel permission model.
- Keeps permission denial distinct from cancellation: after denial the run
  continues (or terminates by the agent's own logic); the UI never renders
  denial as "run cancelled".
- Keeps dialogue, termination (`AcpTermination`), and settlement visually and
  semantically separate.
- Routes the interaction through the appropriate boundary: run-scoped
  governed decisions via SFWP; Thoth conversational surfaces via
  `ThothInteractionPort` — and does not mix them.
- No CopilotKit state owns the permission or its outcome.
- Tests prove no silent fallback: mediator unavailable ⇒ deny (fail-closed),
  and the denial leaves a governed record.

**Fails if:** denial cancels the run, permission state lives in adapter
memory only, or an unavailable mediator falls back to allow.

## Evaluation 4 — Replace the CopilotKit adapter

**Prompt:** "Remove (or disable) the CopilotKit adapter and run all Thoth
flows through the direct AG-UI adapter. Nothing outside the adapter may
change."

**Expected behavior:**
- The change is confined to the adapter package/registration: no Thoth
  canonical types change (`crates/sea-forge-thoth/src/protocol.rs`
  untouched), no Workbench route changes, no SFWP contract changes, no
  settlement or approval behavior changes.
- `DirectAgUiAdapter` passes the same port-level test suite the CopilotKit
  adapter passed (ask, clarification, denial rendering, approval-proposal
  round-trip through `approval.decide`).
- Registered `ThothRenderable` components remain usable and are rendered by
  the direct adapter.
- Grep evidence: no `@copilotkit` import outside the (removed/disabled)
  adapter directory.

**Fails if:** any canonical type, route, or SFWP contract needs editing; any
Thoth flow works only with CopilotKit present; renderable registration lives
inside the CopilotKit package.
