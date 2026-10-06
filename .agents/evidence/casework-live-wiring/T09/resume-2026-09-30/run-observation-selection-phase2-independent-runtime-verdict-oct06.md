# Run observation selection — independent Phase 2 runtime verdict

Date: 2026-10-06  
Verdict: **ACCEPT the four assigned selector gates.** This accepts the bounded private selector prerequisite only. It does not establish manager invocation, live run-list behavior, public correction, T09 completion, CI, or publication.

## Frozen sources

| Path | SHA-256 before and after the final gate |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_selection.go` | `b9de3dae19ea9f473eaa0bbf680d06bcab3975195d90eea3bc0e40866c1ca63d` |
| `apps/godspeed-casework-go/internal/server/run_observation_selection_test.go` | `1e2295a37aec4383f2473226680129f26394253e0ed43d6f05120ef6b30bada4` |

Both hashes matched the Phase 2 source freeze in every gate preflight and were rechecked after the final gate. The independent source approval is `run-observation-selection-phase2-independent-production-source-review-oct06.md`.

All gates used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1`. Each accepted gate had a fresh captured preflight with RAM, swap, `/tmp`, cache and source hashes; every preflight capture below byte-compared with its `/tmp` original (`cmp` exit 0). The reported available RAM / free swap were, respectively:

- Focused: 2,793,467,904 / 1,661,194,240 bytes.
- Server: 2,809,847,808 / 1,653,809,152 bytes before the default-sandbox attempt; 2,808,397,824 / 1,655,894,016 bytes before the authorized retry.
- Module: 2,896,785,408 / 1,632,104,448 bytes.
- Canonical: 2,275,430,400 / 3,757,711,360 bytes.

These are the process/container-visible resource values recorded by the preflight commands; they are not a claim about resources outside that namespace. Every value exceeded the assigned 1,200 MiB RAM and 512 MiB swap floors.

## Gate results and exact captures

| Gate | Command / cwd | Actual joined process | Result and archived evidence |
|---|---|---|---|
| Focused selector race | `go test -race -v -count=1 -parallel=1 -timeout=90s ./internal/server -run '^TestSelectObservationRuns'` / `apps/godspeed-casework-go` | session `6095` | Exit 0. All four selector test functions and all three empty/empty-nonnil/short subtests passed; ranking, full-field/permutation/input-immutability, short-output independence and unknown-standing assertions ran. Preflight `run-observation-selection-phase2-focused-preflight-01-oct06.txt`; output `.raw` and `.exit` are `run-observation-selection-phase2-focused-01-oct06.raw/.exit`. Their `/tmp` originals are `/tmp/run-observation-selection-phase2-focused-preflight-01.txt`, `/tmp/run-observation-selection-phase2-focused-01.raw`, and `/tmp/run-observation-selection-phase2-focused-01.exit`. All three archive comparisons exited 0. |
| Server package race | `go test -race -count=1 -parallel=1 -timeout=180s ./internal/server` / `apps/godspeed-casework-go` | default attempt completed synchronously; authorized retry session `38121` | Default-sandbox attempt exited 1 at `TestArtifactGetReturnsVerifiedCanonicalPayloadAndSessionPerspective`: `httptest.NewServer` was denied binding `[::1]:0` (`operation not permitted`), before regression evidence. It is preserved as `run-observation-selection-phase2-server-01-sandbox-oct06.raw/.exit`, from `/tmp/run-observation-selection-phase2-server-01.raw/.exit`; both cmp exits were 0. After fresh preflight and root-authorized local-only escalation, the same command exited 0 in 14.698s. Accepted preflight is `run-observation-selection-phase2-server-preflight-02-oct06.txt`; successful raw/exit are `run-observation-selection-phase2-server-02-oct06.raw/.exit`, from `/tmp/run-observation-selection-phase2-server-02.raw/.exit`. Preflight and both result captures byte-compared (exit 0). |
| Full module race | `go test -race -count=1 -parallel=1 -timeout=180s ./...` / `apps/godspeed-casework-go` | session `44990` | Exit 0; all listed module packages passed, including SFWP adapters, auth, projection and server. Root-authorized local-only escalation was used due to the confirmed loopback denial above; no external network access was used. Preflight `run-observation-selection-phase2-module-preflight-01-oct06.txt`; raw/exit `run-observation-selection-phase2-module-01-oct06.raw/.exit`, from `/tmp/run-observation-selection-phase2-module-preflight-01.txt` and `/tmp/run-observation-selection-phase2-module-01.raw/.exit`. All three cmp exits were 0. |
| Canonical Go check | `just casework-go-check` / repository root | session `1684` | Exit 0; output says `casework-go-check: format, vet and tests green`; all module packages passed/cached. Root-authorized local-only escalation was used for local test fixtures. Preflight `run-observation-selection-phase2-canonical-preflight-01-oct06.txt`; raw/exit `run-observation-selection-phase2-canonical-01-oct06.raw/.exit`, from `/tmp/run-observation-selection-phase2-canonical-preflight-01.txt` and `/tmp/run-observation-selection-phase2-canonical-01.raw/.exit`. All three cmp exits were 0. |

The accepted Phase 1 actual-RED boundary is separately recorded in `run-observation-selection-focused-red-independent-runtime-verdict-oct06.md` (session `66279`, exit 1 against the typed-unavailable stub). It is not conflated with the Phase 2 green results.

## Immutable preflight-transcription erratum

During the earlier Phase 1 RED work, three files were created while transcribing preliminary `/tmp/run-observation-selection-red-oct06-final-preflight-02.txt` into evidence:

- `run-observation-selection-red-preflight-02-oct06.txt`: comparison against that `/tmp` original exited 1; alignment/lines were incomplete. **Not byte-identical; superseded.**
- `run-observation-selection-red-preflight-02-exact-oct06.txt`: comparison exited 1; several field alignments still differed. **Not byte-identical; superseded.**
- `run-observation-selection-red-preflight-02-native-oct06.txt`: dynamically archived from the original capture and `cmp` exited 0. **Byte-identical to that preliminary original**, but not used as any accepted Phase 2 preflight.

These historical files remain unchanged. The focused Phase 1 runtime verdict uses the separate fresh `...preflight-03...` capture. This Phase 2 verdict relies only on its listed gate-specific preflights, whose comparisons all exited 0.

## Scope, deviations, and limits

The default-sandbox server failure was an observed local listener permission denial, not a code/test regression; root authorized the retry strictly for local loopback/Unix fixture needs. The successful retry and following broad gates ran with that bounded permission and did not use external network access. No other unexpected failure, race, setup issue or timeout occurred.

No source/test, Git/status/debt, manager, public interface, or evidence outside the listed immutable gate captures and verdict was edited. No scanner, Graft build, live integration, other-platform CI or publication gate was run. Runtime approval is limited to the exact selector implementation and assigned Go gates above; production wiring and broader T09 acceptance remain unproven.
