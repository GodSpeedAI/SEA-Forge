# ADR-006: Replace the SEA Forge and Gauntlet user interfaces

## Status

Accepted direction for the casework-environment specification. Removal occurs
only after the plan's parity and conformance gates pass.

## Date

2026-09-19

## Context

SEA Forge currently has a React renderer hosted by a Rust/Tauri desktop app.
Gauntlet is a separate Rust executor with its own TUI. The requested GodSpeed
environment uses a Go operational front end and a React cognitive interface.
The operator explicitly wants both existing user interfaces removed, rather
than merely labeled deprecated. The SEA Forge kernel/server and Gauntlet
executor remain the governed and execution substrate.

The current SEA Forge checkout contains no Go module. Its typed SFWP boundary
and Workbench are real; the exact Go-facing methods needed for leases, history,
and artifact persistence must be inventoried. The Gauntlet repository is a
sibling checkout. No GitHub casework adapter, CopilotKit integration, Open MCT
checkout, or OpenMontage checkout is assumed to exist here.

## Decision

The replacement Go application is built at `apps/godspeed-casework-go`; the
replacement React application is built at `apps/godspeed-cognitive-ui`.
React uses versioned HTTP JSON intents/snapshots and cursor-based server-sent
projection events through typed Go application APIs. Go coordinates work
through typed SEA Forge SFWP and Gauntlet adapters. SEA Forge alone grants
authority and accepts settlement; Gauntlet supplies execution observations.
Neither a Go lease nor a UI projection is case truth.
Read-only Gauntlet diagnostics may use a typed Go adapter; consequential
controls still cross SEA Forge authority.

Both existing interfaces run only during the migration period. The plan first
proves source-backed workflow parity, real governed integration, recovery,
and a cutover procedure. It then removes the SEA Forge Tauri/Rust GUI and
Gauntlet TUI code and supported build entry points. Any assertions protecting
the retained server/executor contract are ported before UI-specific tests go.
The post-removal build must pass all required gates and cold confirmation.
The old binaries are not shipped as fallback user interfaces.
The Rust-generated SFWP schema and its committed-schema drift assertion move
to the replacement client contract location before Workbench contract files
are removed; neither is discarded to make the old GUI deletion pass.

ADR-004 remains a record of the former Workbench stack; it is superseded for
the supported human interface only when the T14 removal gate settles. New
dependencies, SFWP methods, and persisted contracts still require the
repository's separate review and approval before implementation.

## Consequences

- The new Go and React applications need their own scoped instructions,
  canonical `just` gates, typed contract/version policy, and real integration
  tests.
- SEA Forge and Gauntlet keep their existing authority and execution gates.
- The T14 removal check must prove the old UI code and supported launch paths
  are absent from both repositories while retained engines still work.
- If parity, authority, history, or recovery fails, removal and final
  settlement are blocked. A hidden or disabled legacy UI does not satisfy
  the decision.

## Alternatives considered

Extending the Tauri host would retain the interface the operator wants
removed. Replacing Gauntlet's executor along with its TUI would expand scope
and discard useful execution substrate. Giving Go local case authority would
duplicate SEA Forge's governed boundary. These are rejected.
