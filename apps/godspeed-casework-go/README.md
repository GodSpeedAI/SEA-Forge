# godspeed-casework-go

The casework system front end for the GodSpeed casework environment (plan T01). It coordinates
governed casework through application-owned ports and never becomes the semantic authority: SEA Forge
owns case meaning and settlement, Gauntlet executes, and this application coordinates and projects.

## Port ownership

`internal/ports` declares the application's own interfaces and value types. Provider, transport and
vendor shape stops at an adapter:

| Port | Provider side (adapter) | What the application sees |
|---|---|---|
| `AuthorityPort` | SEA Forge SFWP over its Unix socket | `CaseSummary`, `SettlementObservation`, `HistoryWindow` |
| `ExecutionPort` | Gauntlet executor | `ExecutionObservation` (an observation, never a settlement) |
| `RepositoryPort` | the repository provider for enabled workflows | `ChangeProposal` |
| `ArtifactStore` | artifact persistence — **optional** | `ArtifactRef` |

This is enforced, not asserted: `internal/boundary` scans the core packages (`ports`, `config`,
`preflight`, `apperr`) for (a) imports that reach an adapter or a provider, (b) provider or vendor
vocabulary in declared identifiers, and (c) untyped `any`/`interface{}` fields where a provider
payload could hide without naming itself. It also proves itself non-vacuous by failing on each of
those shapes in a synthetic source. Test files are exempt from the *import* rule only, because a test
may drive a fake adapter to exercise a contract and that edge does not ship.

The `godspeedai-stack` boundary rule this implies for later tasks (T04, T05): adapters translate INTO
`ports` types. If a required behaviour cannot be expressed without a provider type in a port, that is
a design fault to raise, not a type to alias.

## Configuration authority (one, documented)

Precedence, lowest to highest:

1. **Defaults** — `config.Defaults()`.
2. **Configuration file** — the `-config` flag, else `$GODSPEED_CONFIG`. JSON, and unknown keys are
   rejected (`DisallowUnknownFields`) so a typo fails loudly instead of being ignored.
3. **Environment** — `GODSPEED_CELL_ROOT`, `GODSPEED_EVIDENCE_ROOT`, and per capability
   `GODSPEED_CAPABILITY_<NAME>_{ENDPOINT,CREDENTIAL,REQUIRED}`.
4. **Explicit overrides** — `Options.Overrides` (`cell_root`, `evidence_root`,
   `capability.<name>.{endpoint,credential,required}`), for tests and operator tooling.

No adapter may introduce a competing order. Adapters receive already-resolved values.

### Secrets

A `credential` must be an **indirection**, never a value: `env:NAME` or `file:/path`. A bare string is
rejected with a typed configuration error, because REQ-CONFIG-002 forbids embedding secrets in stable
configuration. `config.Load` returns the resolved document *and* the secrets separately, and the
document keeps only the indirection name, so a secret cannot be logged or serialised with the
configuration by accident.

## Preflight and blast radius

`internal/preflight` evaluates every configured capability before any consequential work and reports a
state per capability:

* **ready** — the capability answered.
* **degraded** — an *optional* capability is unavailable; the application proceeds without it.
* **blocking** — a *required* capability is unusable; the application refuses to proceed (exit 2).

A capability-scoped configuration fault disables **only that capability**: `config.Load` returns every
problem rather than stopping at the first, `config.Fatal` selects the document-level faults that
prevent startup, and capability-scoped faults travel into preflight so unrelated capabilities keep
their own honest states. The end-to-end behaviour is recorded in the T01 teeth.

## Gate

`just casework-go-check` (repository root) runs `gofmt -l`, `go vet ./...` and `go test ./...` in this
module. It is plan gate `GATE_GO`.

## Not implemented here (by design)

No transport, no live adapters, and no work acceptance: those are T04/T05/T11. Until an adapter is
registered, a capability reports a typed `unavailable` error rather than pretending to be connected.
