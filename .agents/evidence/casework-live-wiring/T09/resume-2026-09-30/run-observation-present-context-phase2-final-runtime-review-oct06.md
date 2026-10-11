# Run observation present-context guard: final runtime review

Date: 2026-10-06 (UTC)

## Verdict

The private present-context guard passed the four assigned runtime gates after the fixture’s two gofmt-only edits. The production source and fixture stayed frozen throughout all four gates:

- `internal/server/run_observation_present_context.go`: `19bd9a574a25beab1e76824ddf136a3091e418d77e7573068c8d3e9464a3b1c2`
- `internal/server/run_observation_present_context_test.go`: `46caa4f5869d705c1ff2678181890e26143d7903f2b697b533b7a11899e51eea`

The source remains a private guard/helper. These results do not prove a manager callsite, public integration, full T09 completion, CI, or publication.

## Gates and captures

Each gate had a fresh preflight recording UTC time, available RAM, free swap, temporary/cache space, and both frozen file hashes. Every preflight met the assigned floors of 1200 MiB available RAM and 512 MiB free swap. All commands used `GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1`.

| Gate | Command | Session / result | Preflight, raw command output, exit captures (SHA-256) |
|---|---|---|---|
| Focused guard | `go test -race -count=1 -parallel=1 -run '^TestCheckRunObservationPresentContext' -v ./internal/server` | session 7811 joined; exit 0; all seven top-level tests and their subtests passed | `/tmp/sea-casework-20261006-guard-format-green01-focused-preflight.raw` `974bb2f2e281e6e6ff665c04d215568fb6e900160e3c887ddf7b7ad66be9be05`; `/tmp/sea-casework-20261006-guard-format-green01-focused-go-output.raw` `d6d220eb72948db8d3ff77533b1588b78a6a78a5e286bddec419cec0cf417911`; `/tmp/sea-casework-20261006-guard-format-green01-focused-exit.raw` `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` |
| Server package | `go test -race -count=1 -parallel=1 ./internal/server` | session 49865 joined; exit 0; package passed in 13.435s | `/tmp/sea-casework-20261006-guard-format-green02-server-preflight.raw` `e6e37fee4851a984234dbc9f0b735ef13955c2dbb69039681b4bec217864cbba`; `/tmp/sea-casework-20261006-guard-format-green02-server-go-output.raw` `4bb0d213f63b91cf7a5620239ce7a7c7a46d95598cd41218440c2bf3d3b64eef`; `/tmp/sea-casework-20261006-guard-format-green02-server-exit.raw` `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` |
| Go module | `go test -race -count=1 -parallel=1 ./...` | session 21717 joined; exit 0; every listed package passed or had no test files; server passed in 13.742s | `/tmp/sea-casework-20261006-guard-format-green03-module-preflight.raw` `4f3988296808e438ace56a3bfb8de05695ab2536b2953d27f15b38adc0a54da3`; `/tmp/sea-casework-20261006-guard-format-green03-module-go-output.raw` `701e6ecded1afc22b3ac92215af151b6b7074b6cb74470ce643d64b39aeecd50`; `/tmp/sea-casework-20261006-guard-format-green03-module-exit.raw` `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` |
| Canonical recipe | `JUST_TEMPDIR=/tmp just casework-go-check` | session 97794 joined; exit 0; recipe reported format, vet, and tests green; server package passed in 12.922s | `/tmp/sea-casework-20261006-guard-format-green04-canonical-preflight.raw` `8a1ab8b6abbad9aa1ca34fab63a0f090d00d0e0ec43cb41eda6c65118f24278a`; `/tmp/sea-casework-20261006-guard-format-green04-canonical-go-output.raw` `4d45f55008c9ac7c91adfb5ce202537a4f70400cc9b20bb9d754e772ca70944d`; `/tmp/sea-casework-20261006-guard-format-green04-canonical-exit.raw` `19eaf43821a7660ec323a87c8457bf74823beb296c39f5e01aa8a683aa50f061` |

The raw files contain command stdout/stderr as captured by shell redirection; wrapper launch metadata is not represented as test output. Exit files contain the wrapper-recorded command exit. Sessions were joined before advancing to the next gate.

## Execution boundary and limits

The focused gate ran in the standard sandbox. The server, module, and canonical gates used the previously authorized elevated execution because these local tests bind Unix/loopback listeners and prior sandbox attempts had received permission errors. No external network access was requested. This establishes the assigned checks in this environment; it does not establish behavior outside the exercised fixtures or broader system integration.

The focused guard exercised the unavailable-input, newest-revision identity, no-fallback, relay-gap, copy, advancing-history, and wrong-case scenarios. No test was skipped or weakened. The canonical recipe reused cached results for some module packages, while its server tests ran and passed.

Earlier attempts against the pre-format fixture, including its canonical gofmt failure and sandbox listener denials, are preserved in their separate records and are superseded for this final four-gate sequence. This record does not rewrite those attempts or claim they passed on the final fixture.
