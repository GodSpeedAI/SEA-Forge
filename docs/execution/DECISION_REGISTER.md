# Decision Register

## Resolved Decisions

| ID | Decision | Authority/evidence | Consequence |
|---|---|---|---|
| R-01 | DomainForge owns `.sea` semantic meaning; SEA Forge owns final governed execution. | Accepted ADR-001 | Keep exact side-effect-free library adapter; no CLI invocation or competing parser. |
| R-02 | The minimum CLI lifecycle and its record/ID compatibility remain the base contract. | `spec-minimum.md` implemented v0.1; P1-P4b | Additive completion cannot weaken or rewrite minimum history. |
| R-03 | Full-system features are additive and preserve one authority fabric, settlement semantics, and append-only truth. | `spec-full.md`, root `AGENTS.md` | Server/Workbench paths must reuse kernel services, not duplicate them. |
| R-04 | Workbench is Tauri 2 + React 19 in a separate Bun workspace and standalone Tauri Cargo workspace. | Accepted ADR-004 | Keep async/UI dependencies outside the synchronous kernel. |
| R-05 | The renderer is a governed client; canonical decisions and mutation remain server/kernel-side. | Frontend contract `:12-68`; `workbench/AGENTS.md` | Closed Tauri bridge only; no direct socket/store/SQL or generic invoke. |
| R-06 | SFWP Rust types are canonical and generate JSON Schema plus committed TS/AJV. | Accepted ADR-005 | Generated zones are projections and must be drift-gated. |
| R-07 | SFWP v1 remains local NDJSON over an owner-only Unix socket until a remote client requirement exists. | `spec-full.md:1594-1599` | No HTTP/auth framework in current completion scope. |
| R-08 | Linux is primary and macOS secondary. | `spec-full.md:5` | Restrict product claims and packaging matrix accordingly. |
| R-09 | Completion follows the representative journey, not raw route or 123-method catalog counts. | UX epic `:7-31,601-618`; catalog preamble | Map/merge/reject catalog entries; build only journey and compatibility needs. |
| R-10 | Binary distribution is sufficient for the first installable CLI; crates.io is not a completion prerequisite. | Current CLI dependency closure and local-product boundary | Do not publish 19 internal crates merely to satisfy stale release automation. |

## Provisional Decisions

These are reversible architectural defaults. They do not waive the repository's
ask-first rule: implementation still requires owner authorization when it changes
deployment/configuration, a public interface, persisted layout, dependencies, or
CI.

| ID | Decision | Why this is the smallest coherent choice | Revisit trigger |
|---|---|---|---|
| P-01 | One absolute cell root owns `server.yaml`, records, and the default socket; explicit socket override remains for tests/admin. | Eliminates CWD/home mismatch with one concept already present in the product. | Multiple simultaneous cells per server or remote cell selection is required. |
| P-02 | New case episodes target the full-spec case-owned layout; one locator reads legacy flat minimum runs until a migration decision is authorized. | Honors additive spec and persisted history without permanent dual-write. | Migration evidence shows case-owned layout cannot preserve an external compatibility commitment. |
| P-03 | Package Linux first; add macOS only after packaged Seatbelt journey proof. | Matches normative platform priority and available host evidence. | Owner sets a simultaneous cross-platform launch requirement. |
| P-04 | Preview routes remain visibly non-operational until a source-backed journey needs them. | Avoids implementing catalog breadth merely to make every route look live. | A preview route becomes part of the representative journey or a committed public promise. |

## User-Required Decisions

Backend integrity packages can be defined without another product decision.
Distribution/service-lifecycle packaging is blocked on U-06. The following
choices require the owner when their trigger occurs:

| ID | Decision requiring owner | Trigger |
|---|---|---|
| U-01 | Whether to publish the internal Rust library graph to crates.io. | Public library consumption or `cargo install sea-forge-cli` becomes a product requirement. |
| U-02 | Signing/notarization/store distribution identities and credentials. | Public macOS/Linux store or signed release is authorized. |
| U-03 | Remote/multi-user authentication and service deployment model. | A non-local client or shared daemon becomes a product requirement. |
| U-04 | Archive or delete tracked `working/face/` material. | Repository cleanup package; deletion is explicitly authorized. |
| U-05 | Licensing/edition composition of release artifacts. | A release combines components whose `LICENSE_EE.md` classification changes what may be distributed. |
| U-06 | Tauri-supervised sidecar versus separately installed local server service. | Before implementing the distribution/service lifecycle package. |
| U-07 | Exact public SFWP identity/session contract. | **RESOLVED 2026-07-30 — see below.** |

Branch merge/push and release timing are workflow authorizations, not
architectural unknowns. They remain outside this pass.

## U-07 — Resolved: the public SFWP identity contract

Decided by the owner on 2026-07-30, unblocking SF-005 and everything
downstream of it. Four answers, each binding on the wire contract.

### 1. Identity binds **per request**, not per connection

Each protected verb carries an `actor` block. The server holds no session
state for identity, a reconnect needs no re-handshake, and the actor travels
with the ledger entry the request produces. Additive to the existing
`#[serde(tag = "verb")]` request enum.

*Rejected:* a connection-scoped handshake. A dropped connection would lose the
binding mid-case, and the server would have to hold state whose only purpose is
to be re-derived after every reconnect.

### 2. The server **derives** the caller's identity from the socket

`SO_PEERCRED` on the accepted Unix stream gives the peer's uid. The `actor`
block a client asserts must map to that uid through policy; a mismatch is a
denial, not a warning.

This is what makes the actor block trustworthy. Believing the host's assertion
because the socket is `0600` would let any process running as the owner claim
any actor id — which is the same fabrication SF-005 exists to remove, moved one
layer down. A signed credential would be stronger still, but it introduces key
issuance and rotation that nothing in the repository has today, and the threat
model here is local-only.

*Consequence:* the identity contract is Unix-socket-shaped. A future remote or
multi-user model is **U-03**, and will need a different answer.

### 3. The approver is identified **independently** and compared to the ledger

An approval carries its own actor block. Separation of duty is enforced by
comparing it against the submitter recorded in the ledger for that case — not
against the connection the approval arrived on.

This is the only answer that survives a restart, and the only one under which a
second person can approve from a second window hours later. Requiring the
approver to be on the submitting session would make two-person approval
impossible in exactly the situations it exists for.

### 4. The actor block is **optional on inspect, required on protected**

Read-only verbs (`run_get`, `run_list`, `case_get_*`, `system_*`, `events_*`)
continue to work without it. Verbs that cause a side effect (`submit`,
`approve`, `reject`, `delegate`, `cancel_delegation`, `case_commit`, …) refuse
without it, with a typed denial and no side effect.

This confirms as the contract what SF-005's compatibility criterion — "Old SFWP
clients without actor context still work for inspect verbs" — had only implied.
Whether a call is governed is a property of the *verb*, decided by the
protocol, not a policy question resolved per deployment.

## Stale Decisions Or Claims

| ID | Stale item | Current disposition |
|---|---|---|
| S-01 | Root guide/README/architecture describe a not-yet-implemented two-crate system. | Historical foundation narrative; update to current 22-crate + Workbench truth. |
| S-02 | Pass 1 says Copilot instructions are absent. | False; tracked `.github/copilot-instructions.md` is active. |
| S-03 | Pass 1 says `working/face/` is untracked. | False; tracked but non-authoritative to current product. |
| S-04 | Pass 1 says DomainForge remains stubbed. | False for adapter/library binding; composition into every case path remains incomplete. |
| S-05 | Pass 1 says every route has G1-G9 guards. | False; current production router uses mock context and only base G1. |
| S-06 | Pass 1 says 0 P0 and operator desktop paths work end-to-end. | Replace with: minimum CLI works from source; integrated distributable product is blocked. |
| S-07 | ADR-005's count of eleven schema types. | Historical initial set; additive types are allowed. Update wording, not architecture. |
| S-08 | ADR-005/Pass 1 imply Workbench checks enforce generated TS/token drift. | Scripts exist, but routine gate/CI wiring is absent. |
| S-09 | Tauri `targets: all` implies broad platform support. | Product support is Linux primary/macOS secondary only. |
| S-10 | Release workflow's two-crate publication model still installs the CLI. | False after CLI dependency expansion; use binary release or publish full closure. |

## Compatibility Commitments

| ID | Commitment | Completion rule |
|---|---|---|
| K-01 | Existing minimum record schemas, IDs, exit codes, and P1-P4b behavior. | Keep proofs green; migrations preserve IDs and readable history. |
| K-02 | Existing CLI commands and scriptable output used by proofs/operators. | Additive changes only unless an explicit public-contract decision authorizes a break. |
| K-03 | SFWP protocol major `1`, clean unknown-version/variant failure, and additive ADR-003 behavior. | Maintain old-client fixtures and protocol negotiation. |
| K-04 | Existing flat run history. | Locator/migration reads it; never rewrite ledger identity. |
| K-05 | DomainForge exact version/model refs in evidence. | Version changes create new refs and require reviewed compatibility evidence. |
| K-06 | Separate Tauri and root Cargo workspaces. | Automated boundary check; no Tauri async dependencies in kernel. |
| K-07 | Generated Rust-schema/TS contract ownership. | Never hand-edit generated outputs; regenerate and commit deterministically. |
| K-08 | Single-operator local socket security model for v1. | Keep `0600`; do not imply remote or multi-user safety. |

## Agent Authority Classification

- **Agent-autonomous:** internal code organization within accepted boundaries,
  focused verifier design, and vertical slice ordering that changes no public
  contract, persisted layout, dependency, CI, or deployment configuration.
- **Provisional:** P-01 through P-04 guide planning. Implement them only after any
  applicable ask-first authorization.
- **User-required:** U-01 through U-07 when triggered, plus repository ask-first
  changes to persisted schemas/layouts, public SFWP behavior, dependencies, CI,
  or deployment configuration. Ordinary internal implementation choices must
  not be escalated.
