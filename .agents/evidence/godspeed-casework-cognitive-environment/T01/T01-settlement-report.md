# T01 settlement report — application-owned ports, configuration authority, typed errors, preflight

- **Task:** T01 (P2, confirmation: peer). **Gates:** `GATE_SPEC_TRACE` PASS · `GATE_GO` PASS
  (`just casework-go-check`: gofmt, `go vet ./...`, `go test ./...`).
- **Date:** 2026-09-19. **Settles:** REQ-CONFIG-001/002/003/004/010/011/012, REQ-ARCH-001/002.
- **Preregistration:** frozen at 2026-09-19T22:18:29Z, before any module, contract, recipe or test was written.

## What was built

| Artifact | Role |
|---|---|
| `apps/godspeed-casework-go/internal/ports` | The application's own ports and value types (authority, execution, repository, optional artifact store) with no transport or provider import |
| `apps/godspeed-casework-go/internal/apperr` | Typed error model (`config`, `unavailable`, `authority_denied`, `invalid`, `internal`) carrying the capability name, so callers switch on a kind rather than a provider code |
| `apps/godspeed-casework-go/internal/config` | The ONE configuration authority: documented precedence, secret indirection, validation before activation, and one problem list instead of first-fault abort |
| `apps/godspeed-casework-go/internal/preflight` | Per-capability ready/degraded/blocking evaluation with typed errors |
| `apps/godspeed-casework-go/internal/boundary` | The mechanical REQ-ARCH-002 gate (imports, identifiers, untyped holes) with its own non-vacuity subtests |
| `apps/godspeed-casework-go/internal/adapters/fakeauthority` | A provider-native envelope plus the adapter that translates it — the provider side of the boundary |
| `apps/godspeed-casework-go/cmd/godspeed-casework` | Entrypoint: load config, run preflight, report typed per-capability states, exit 2 when a required capability blocks |
| `apps/godspeed-cognitive-ui/contracts` | React-owned world/interaction/temporal/artifact/agent adapter contracts with no renderer, agent-framework or transport dependency |
| `justfile` → `casework-go-check` | The named `GATE_GO` recipe over the real module |
| `apps/godspeed-casework-go/README.md` | Documented port ownership and configuration precedence (T01 `done_when`) |

## Gate evidence

```
$ python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py
PASS: spec hash, 86/86 requirements, 15-task DAG, gate activation, and UI removal

$ just casework-go-check
casework-go-check: format, vet and tests green
```

## Teeth (executed)

`teeth/run-teeth.sh` → exit 0. `TEETH RESULT: both teeth behaved as specified`.

**Tooth 1 — an unknown provider payload type in a core path must be rejected.** Three steps, in order:
1. the scanner catches all five forbidden shapes in synthetic sources (provider import, adapter import,
   vendor identifier, `any` field, empty-interface field) — so the check is not vacuous (the tooth's
   printed "6" counts the parent test's own `--- PASS` line and is not a sixth shape; corrected by
   verification round 1);
2. a real `sfwpEnvelope` provider type written into `internal/ports` makes the boundary gate **FAIL**
   (`boundary violation: internal/ports/zz_tooth_injected_leak.go:4 core identifier sfwpEnvelope
   names provider vocabulary (sfwp)`);
3. removing it makes the gate pass again.

**Tooth 2 — one missing required credential plus one unavailable optional adapter.** Contract level:
`TestPreflightBlastRadius` proves an unrelated healthy capability stays **ready**, the optional one is
**degraded**, and the required one is **blocking** — with `config` vs `unavailable` kinds distinguished.
End to end: the real binary against `fixtures/blast-radius.json`, with the repository credential unset,
exits **2**, reports `blocking=[authority repository]` and `degraded=[artifact]`, and attributes the
blocking fault as `err=config: ... (repository/secret)`. Stated plainly: the CLI cannot yet show a
*ready* unrelated capability because no real adapter is registered until T04/T05 — that half is
proven at contract level, and the tooth says so rather than implying otherwise.

## `done_when` adjudication

1. **Port ownership and configuration authority are implemented and documented.** MET — see the README
   table and precedence list; ports carry no provider import and configuration has one authority.
2. **Preflight and typed-error tests pass with the declared blast radius.** MET — 5 config tests, 3
   preflight tests, 3 adapter tests, 2 boundary tests; the end-to-end CLI run is in the tooth log.
3. **Provider-object leakage teeth pass.** MET — Tooth 1 above, including the real injection.
4. **All applicable global gates remain green.** MET for T01's gates. Note the plan's `GATE_SEAFORGE`
   activates at T05 and was not run here; `just security` was verified green separately during the
   approved B2 remediation, and `just check`/`just ci` must be re-run at T05 rather than assumed.

## Preserved corrections (round 0 → 1)

1. **Configuration faults aborted before preflight.** The first version of `config.Load` returned the
   first problem as a fatal error, so a missing credential killed the process before any other
   capability was evaluated — which contradicts REQ-CONFIG-012's blast radius. Load now returns every
   problem, `config.Fatal` selects document-level faults, and capability-scoped faults reach preflight.
   Found by writing the tooth scenario, not by the happy path.
2. **The tooth mis-specified its own check.** It asserted the literal string `config error
   (capability=repository)`, but the typed error prints as `err=config: ... (repository/secret)`. The
   product was right and the assertion was wrong; the assertion was corrected and the CLI's
   limitation (no ready capability until adapters exist) is now stated in the tooth output.
3. **`resolveSecret` returned `error` while its callers collected `*apperr.Error`** — a compile
   failure caught by `go vet`/`go test`, fixed by typing the return.

## What T01 does not prove

The plan is explicit: not SEA Forge or Gauntlet compatibility, not cognitive projection quality, not UI
interaction behaviour, not real execution. No adapter for those exists yet, and the CLI reports them as
`unavailable` rather than pretending otherwise.
