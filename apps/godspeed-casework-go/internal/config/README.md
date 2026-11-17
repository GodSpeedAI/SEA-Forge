# config

## Overview
The application's ONE configuration authority. REQ-CONFIG-001: one documented precedence order and
no adapter may introduce a competing one. REQ-CONFIG-002: secrets are resolved through indirection
(`env:NAME`, `file:/path`) and never appear in the loaded configuration. REQ-CONFIG-003: endpoints,
provider selections, ports and environment paths stay configurable. REQ-CONFIG-004: the schema and
its defaults are declared once, here, and validated before activation.

## Interface
- `Document` — the JSON schema (unknown keys rejected at load); `Capability`, `ServeSection`, `AuthSection`, `OIDCSection`, `RateLimitSection`, `AuthUser`, `OIDCMappingRule` [config.go:Document]
- `Load(path, Options) (Resolved, []Secret, []*apperr.Error)` — full precedence resolution returning the document and resolved secrets separately [config.go:Load]
- `Defaults() Document`, `Validate(doc) []*apperr.Error`, `Fatal(problems) []*apperr.Error` [config.go:Defaults] [config.go:Validate] [config.go:Fatal]
- `ValidateBindPosture(addr, doc)` — fail-closed loopback-only binding rule unless `serve.production` [auth.go:ValidateBindPosture]
- `SupportedVersion` — `"1"`, the only accepted schema major [config.go:SupportedVersion]
- Kind constants `KindAuthority`, `KindExecution`, `KindRepository`, `KindArtifact`; adapter constants `AdapterEmpty`, `AdapterFixture`, `AdapterLive` (`"sfwp"`) [config.go:KindAuthority] [config.go:AdapterLive]
- Auth posture types and defaults: `AuthOrDefaults`, `authDefaults`, `rateLimitDefaults`, `ServeDefaults`, `RateLimitOrDefaults` [auth.go:AuthOrDefaults]
- Build-tag fixture gate: `fixtureBuildEnabled` is `false` in the default build and `true` under `-tags casework_fixture` [fixture_enabled.go:fixtureBuildEnabled] [fixture_enabled_tagged.go:fixtureBuildEnabled]
- `DefaultExecutionTimeoutSec` (60), `MaxExecutionTimeoutSec` (7 days) [config.go:DefaultExecutionTimeoutSec]

## Invariants
- Precedence lowest to highest: `Defaults()` → config file (`-config`/`$GODSPEED_CONFIG`, unknown keys rejected) → environment → `Options.Overrides`; no adapter may introduce a competing order [config.go:Load]
- A `credential` must be an indirection (`env:`/`file:`), never a bare value; `Load` returns secrets separately from the document so secrets cannot be logged or serialised with it [config.go:resolveSecret]
- All problems are collected, not first-fail: `Load` returns every problem; `Fatal` selects document-level faults, capability-scoped faults travel into preflight [config.go:Fatal]
- Closed adapter set `knownAdapters`; unknown kinds/adapters are configuration errors [config.go:knownAdapters]
- Production build (`fixtureBuildEnabled = false`) refuses fixture adapter selections [fixture_enabled.go:fixtureBuildEnabled]
- `serve.production` forbids dev auth and insecure cookies; a serve posture with no `auth` section refuses to start; loopback-only bind unless `serve.production` [auth.go:ValidateBindPosture]

## Side Effects
- Reads the configuration file from disk and the process environment (`GODSPEED_CELL_ROOT`, `GODSPEED_EVIDENCE_ROOT`, `GODSPEED_CAPABILITY_<NAME>_*`, `SEA_FORGE_SOCKET`) [config.go:Load] [config.go:liveSocketEnv]
- Resolves secrets from env vars or files through the indirection refs [config.go:resolveSecret]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
