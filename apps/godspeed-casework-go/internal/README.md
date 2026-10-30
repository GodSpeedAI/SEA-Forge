# internal

## Overview
All packages of the casework gateway below the command entrypoint: the application-owned ports and
error model, the single configuration authority, the SFWP adapter over the governed kernel, the
cognitive projection (live builder + revision stores + fixture dataset), the intent translator, the
live HTTP+SSE server, preflight, metrics, and the `-tags live` test harnesses. The core packages
(`ports`, `config`, `preflight`, `apperr`) must never import an adapter or use provider vocabulary;
`internal/boundary` enforces this.

## Subdirectories & Boundaries
- [./adapters/README.md](./adapters/README.md) - provider-side adapters (live SFWP kernel client; boundary-test fake)
- [./apperr/README.md](./apperr/README.md) - the stable, provider-independent error model (closed `Kind` set)
- [./artifactstore/README.md](./artifactstore/README.md) - fixture-scoped in-memory artifact persistence
- [./auth/README.md](./auth/README.md) - browser authentication (local/oidc/dev), argon2id hashing, session store
- [./config/README.md](./config/README.md) - the ONE configuration authority (precedence, secret indirections, validation)
- [./contract/README.md](./contract/README.md) - Go mirror of the canonical UI<->gateway wire contract
- [./coordinator/README.md](./coordinator/README.md) - fixture-stack intent handling: allowlist authority, leases, staged revisions
- [./intents/README.md](./intents/README.md) - live intent translator: one governed SFWP verb per intent, typed refusal outcomes
- [./livestack/README.md](./livestack/README.md) - assembles the full live gateway for `-tags live` tests
- [./livetest/README.md](./livetest/README.md) - real-kernel test harness (boots `sea-forge-server` on fresh temp cells)
- [./metrics/README.md](./metrics/README.md) - dependency-free Prometheus text exposition
- [./ports/README.md](./ports/README.md) - application-owned port interfaces and value types
- [./preflight/README.md](./preflight/README.md) - capability validation before consequential work (ready/degraded/blocking)
- [./projection/README.md](./projection/README.md) - cognitive projection: live builder, revision stores, fixture dataset
- [./server/README.md](./server/README.md) - the HTTP+SSE surface (live `Server` and fixture `FixtureServer`)
- `boundary` - test-only package (`boundary_test.go`); enforces the kernel-boundary rules, no production source
- `loadtest` - test-only package (`*_test.go`); load/budget rig, no production source
- `sfwp` - no production source: contains only a stray empty `client.go/` directory artifact (untracked graft cache output), not a package
