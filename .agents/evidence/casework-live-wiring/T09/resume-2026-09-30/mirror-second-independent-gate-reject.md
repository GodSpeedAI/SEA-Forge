# T09 Go/UI mirror: second independent gate rejection

Date: 2026-10-01
Result: **REJECT — focused Go contract race/count-one gate failed.**
Reviewed repair builder: `t09_subscription_writer_repair` (two test files only).
Compiler token: released to ROOT after this failure. No further compile/test work was run.

## Source review and identities

The repair addressed both findings in
[`mirror-first-independent-source-reject.md`](mirror-first-independent-source-reject.md):

- `golden_test.go:305–383` now pins `reflect.Type` for all fields in all eight added Go
  DTOs. Combined with `TestCanonicalMirrorFieldSets`, this covers the complete JSON field
  and optionality inventory.
- `wireContract.test.ts:432–438` now rejects extra `hydration_read_budget` keys; the new
  `wireContract.test.ts:963–977` test clones the observation fixture, injects `extra: true`,
  and expects the runtime validator to throw. UI structural type pins cover all eight
  complete canonical DTO shapes (`wireContract.test.ts:248–320`).

The source also retains the previously reviewed Ask/observation field and vocabulary
mirrors, separate execution/settlement standings, pointer-backed count absence, absent
versus zero command metadata, typed observation event decoding, legacy event arms, and the
`NarrationBeat.grounded_answer` port. The four Ask goldens were present and read by the
Go test. This source review does not approve GREEN; the focused Go gate below fails.

Four frozen source SHA-256 identities at the gate:

| File | SHA-256 |
| --- | --- |
| `apps/godspeed-casework-go/internal/contract/contract.go` | `f0b51bfb61698e56faf16cf4a0216010e52e9c2f447f0ff673b78980964363c6` |
| `apps/godspeed-casework-go/internal/contract/golden_test.go` | `f3d3f79d3667156042474b3152782a4bfbc9e9b11eaf959b60e4991a3d069f39` |
| `apps/godspeed-cognitive-ui/src/ports/contract.ts` | `042970baaa5dc131eb2ee2ab00a29a6fc5fcf62abffa1f11b6f5d9b253336bf4` |
| `apps/godspeed-cognitive-ui/src/ports/wireContract.test.ts` | `ce48a051ebbd6037da828f94986a757adf25ec083fc5eddb25230f3f635036c0` |

Ask fixture SHA-256 identities at the gate:

| Fixture | SHA-256 |
| --- | --- |
| `ask-request.json` | `30efdb892da29fa21760db57511bac75f85d4116df0f06d4a1461e5ecf295864` |
| `ask-answer-answered.json` | `9d90a163f10d3ad066cc0ed0e5911eaa19f633bf6a1f7a9ae3858258d4fbd945` |
| `ask-answer-partial.json` | `2d8f2f659b540c5c2842d0080a517c18b39b327ca9973a8b42eab94a19d6d7bc` |
| `ask-answer-denied.json` | `2df780254d4332afd6d4885bcf6467009bb7860ad96e7be223a334af32ef52aa` |

## Focused Go gate

Command, from repository root:

```text
mkdir -p /tmp/t09-mirror-repair/go-cache /tmp/t09-mirror-repair/go-tmp
{
  date -u '+%Y-%m-%dT%H:%M:%SZ'
  free -b
  ps -eo pid,rss,comm --sort=-rss | awk 'NR <= 15 {print}'
} | tee /tmp/t09-mirror-repair/preflight-go-focused.txt
cd apps/godspeed-casework-go
GOCACHE=/tmp/t09-mirror-repair/go-cache GOTMPDIR=/tmp/t09-mirror-repair/go-tmp GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 ./internal/contract 2>&1 | tee /tmp/t09-mirror-repair/go-focused-race-count1.log
```

Preflight at `2026-10-01T04:11:11Z`: host available memory `3,149,533,184` bytes;
swap available `7,326,474,928` bytes. The compact process table had only the Codex process
and the preflight shell/`tee`/`awk`/`ps`; no competing compiler/test process was present.
The exact 11-line preflight and 155-line full test output are copied byte-for-byte into
`mirror-repair-preflight-go-focused.txt` and `mirror-repair-go-focused-race-count1.log`.
The copies were verified with `cmp` against `/tmp`; SHA-256 values are respectively
`23fa9a81973a2ffd7694ccda50af6202191d458c4925675c116878b449fc9543` and
`1a107f090b8e0e84963fc8832509732528a8e5eec20ee46ad5e49bffeb99e1ad`.

The log reports `TestGoldenRoundTrip` failing for `ask-answer-answered.json`,
`ask-answer-partial.json`, and `ask-answer-denied.json`: nonempty JSON arrays in those
fixtures are on one line, while Go's indented re-marshal places their elements on multiple
lines, so the required byte-stable round trip fails. `ask-request.json` emitted no error.
The Go test output ends with `FAIL`; the pipeline did not enable `pipefail`, so the shell
pipeline status itself was not captured as the Go process status. The test result is
nevertheless a failure based on the full Go output, and no passing result is claimed.

No UI typecheck, UI test, full Go gate, or full-module race gate was run after this failure.
`git diff --check` and `gofmt -d` over the four frozen files completed with no output.

## Next bounded move

Root assigned a fresh canonical-fixture-only repair: format whitespace in only the three
Ask answer goldens, with no value, type, field-order, or assertion changes. After that freeze,
rerun canonical Bun and explicit TypeScript pins, then resume the prescribed mirror gates with
fresh per-command RAM/process preflights and immutable logs. The mirror implementation itself
is not approved; T09 settlement remains pending.
