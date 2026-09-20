# T01 verification round 1 — verdict CONFIRM

- **Round:** 1 (first independent peer confirmation of T01)
- **Date:** 2026-09-19
- **Verifier:** a fresh independent agent, not the builder of any T01 artifact; worked against the
  plan worktree at `ff3c285`; read every core file in full before running anything.
- **Verdict:** **CONFIRM**, with one minor representation finding (F1, corrected in the same round —
  see below). The claim held: provider/runtime technologies connect at adapters, and invalid or
  missing configuration fails with the specified blast radius before consequential operation.

## Claim-by-claim table (verifier's own commands and observations)

| # | Claim | My command / inspection | My observation |
|---|---|---|---|
| 1 | `internal/ports`, `internal/config`, `internal/preflight`, `internal/apperr` import no adapter or provider | read all four packages in full + grep for provider vocabulary (`sfwp`, `gauntlet`, `github`, `copilot`, `ndjson`, `sea-forge`) in non-test sources | only stdlib imports (`context`, `time`, `encoding/json`, `fmt`, `os`, `sort`, `strings`) and the module's own internal imports; the only grep hits are the app's own module path in import lines |
| 2 | The boundary scanner is non-vacuous | `go test ./internal/boundary/ -run TestScanDetectsInjectedViolations -v` (via the tooth) | all five injected shapes detected (provider import, adapter import, vendor identifier, untyped `any` field, empty-interface field) |
| 3 | The boundary gate scans the real core | read `internal/boundary/boundary_test.go` | AST-based scan of all `.go` files under the four core packages read from disk; refuses to run vacuously (`t.Fatal` when no sources were scanned); identifier checks apply to test files too; the import exemption for `_test.go` is documented in-file |
| 4 | ONE configuration precedence order | read `internal/config/config.go` | documented and implemented in one place: defaults < file < environment < explicit override (`merge` → `applyEnvironment` → `applyOverrides`); env/override lookup functions are injected so the order is a property of the package, not the caller |
| 5 | Bare credentials rejected | `resolveSecret` (config.go:240) + my Attack B below | a credential without an `env:`/`file:` indirection is a typed config problem ("credential must be an indirection…"); only those two schemes are accepted |
| 6 | No secret value in the resolved document | code inspection + `Resolved`/`Secret` types | values go into `[]Secret` returned separately; `Resolved.SecretNames` carries the indirection *name* only; the CLI prints counts, never values |
| 7 | Capability-scoped faults reach preflight instead of aborting startup | `config.Fatal` selects only capability-less (document-level) problems; CLI passes all problems into `preflight.Check` | reproduced end to end (Tooth 2 and my Attack B): a repository credential fault does not stop artifact/authority evaluation |
| 8 | Blast radius matches the plan's teeth | re-ran `.agents/evidence/…/T01/teeth/run-teeth.sh` | exit 0: real `sfwpEnvelope` injection into `internal/ports` FAILS the gate and passes again after removal; CLI against the fixture exits 2 with `blocking=[authority repository]`, `degraded=[artifact]`, typed `err=config: … (repository/secret)` vs `err=unavailable: …` |
| 9 | `GATE_GO` green | `just casework-go-check` | exit 0 — gofmt clean, `go vet` clean, all module tests pass (`casework-go-check: format, vet and tests green`) |
| 10 | `GATE_SPEC_TRACE` green | `python3 .agents/plans/validate-…py` | exit 0 |
| 11 | React contracts carry no renderer/agent/transport dependency | read all six files in `apps/godspeed-cognitive-ui/contracts/`; tooth grep re-run | no `@react-three`, `copilotkit`, `fetch(`, `EventSource`, `XMLHttpRequest`, `godspeed-casework-go`, `sfwp`, or `ndjson` reference; all five adapter interfaces present |

## Falsification attempts (all three failed to break the product)

1. **Provider type in a different core package than `ports`.** I wrote a `gauntletRunner` type into
   `internal/config/` and ran the boundary gate: it FAILED with `core identifier gauntletRunner names
   provider vocabulary (gauntlet)`, and passed again after removal. The gate does not only protect
   `ports`.
2. **Required capability with an endpoint but no credential.** Built the real binary against a
   crafted config (`authority`, required, endpoint set, credential absent): exit 2, authority
   blocking with the typed config fault `required consequential capability has no credential
   indirection (authority/validate)`, artifact degraded. Nothing aborted before preflight.
3. **One capability healthy while another blocks.** A temporary contract test with a healthy
   authority prober and a config-faulted repository: `ready=[authority]`, `blocking=[repository]`,
   `degraded=[artifact]`, `Proceed=false`, `Failed(rs,"authority")==nil`. Per-capability states stay
   honest in both directions. (First attempt of this test failed to compile — my own test file was
   missing an import; that failure was mine, not the product's, and the corrected test passed.)

All verifier-created files were removed and the module re-tested clean afterwards.

## Findings

- **F1 (representation, minor, corrected this round).** `T01-settlement-report.md` §Teeth claimed
  "the scanner catches all six forbidden shapes … (provider import, adapter import, vendor
  identifier, `any` field, empty-interface field)" while enumerating five. The six came from the
  tooth's `grep -c -- '--- PASS'`, which counts the parent test's PASS line together with the five
  subtests. **Correction:** the settlement report now says five and states where the six came from.
  The tooth script is unchanged (its printed count is of passing tests, not shapes). Re-ran the
  tooth after the correction: exit 0, same behaviour. No other artifact asserts the falsified
  count (grep for "six forbidden shapes" across the plan's artifacts returns only this correction).

## What I could not verify

- Real provider compatibility (SEA Forge/Gauntlet): explicitly out of T01's scope; no adapter
  exists, and the CLI reports unregistered capabilities `unavailable` rather than pretending.
- That the fixture CLI run in the tooth log used exactly the committed fixture bytes: I re-ran the
  tooth myself, so the behaviour is reproduced independently of the historical log.
- Rust-side gates (`GATE_SEAFORGE`): not T01's gates; deferred on the resource policy recorded in
  the handoff.
