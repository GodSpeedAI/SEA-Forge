# Open Questions

Only choices that cannot be resolved from the repository or authoritative
research belong here. Include a recommendation and the trade-off. Remove the
entry when decided.

<!-- Resolved 2026-07-17: summarized-transcript verification.
Decision (owner-accepted): summarized mode retains a sealed, encrypted canonical
transcript, verifies before crypto-shredding, exposes only the deterministic
summary. Recorded canonically in spec-agent-orchestration.md "Resolved decisions"
and in .agents/CURRENT_STATUS.md. Gates M13 (TranscriptEvidence). -->

<!-- Entry format:
## Question: Decision needed
- Context: evidence and why research did not resolve it
- Recommendation: preferred choice
- Trade-off: what the recommendation gives up
- Needed by: task or milestone blocked by the decision
-->

## Question: Persisted selected-cell ownership and migration scope

- Context: The Workbench currently resolves one cell from `SEA_FORGE_ROOT` at
  process startup. `CellSupervisor` and `SocketHandle` keep that cell/socket
  immutable, while `EventCursor` stores one global cursor under app data. A
  safe in-app selection cannot swap only the root: it must atomically replace
  the supervisor, transport, and a cell-namespaced cursor. The server does not
  call `sea_forge_self_model::store::ensure_init` during startup, so it cannot
  yet perform the Task 4 compatible-history migration it would need to claim.
- Recommendation: Approve a small versioned host-owned selection record under
  the Workbench app-data directory (canonical root plus selection version), a
  cell-keyed event cursor, and server-owned compatible self-model initialization
  before exposing open/select. Keep incompatible history fail-closed and make
  the UI show the server's exact repair action.
- Trade-off: This adds a durable host record and a public readiness/selection
  contract; it deliberately does not add a database, a generic workspace
  registry, automatic incompatible migration, or a renderer filesystem path.
- Needed by: Workbench completion plan Task 4, step 1; it also gates the
  repeatable refresh/revisit requirement of the golden path.

  **recomendation is approved** 
