# Ask raw UTF-8 fixture independent RED review

## Scope and decision

This records the independent expected-RED review of the phase-A raw-byte fixture only. Phase A is approved as a valid regression test: the test compiles, and all three malformed UTF-8 cases reach the Ask route and return a successful answer where HTTP 400 is required. This is evidence for the missing raw-byte validation guard. It is not phase-B approval and does not approve the Ask runtime implementation.

The test's `askStatus` assertion fails first in each subtest. Consequently, its later assertions that Ask dispatch and perspective-verification counters remain zero did not execute in this RED run. The observed HTTP 200 with the fixture answer, together with the route source order (decode, verify session perspective, invoke Ask, then write answer), shows the request was admitted. Do not report the skipped counter assertions as directly observed.

## Reviewed phase-A source identities

- `apps/godspeed-casework-go/internal/server/ask.go`: `51997f7e5ce4ed603d9092f526735c376ef3b35d8c0cdaf79d2b60436831e73c`
- `apps/godspeed-casework-go/internal/server/ask_validation_test.go`: `a1888b765e7219513bf0ca9b2bd083631f52050f25c230c8e0ead7d1d82de861`
- `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go`: `9dddfd91beb5baef2ee941f74bc213c33b5753d8704a20abbad56908b099e752`
- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go`: `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`

The two previously frozen Ask fixtures remained unchanged: `internal/server/ask_test.go` `90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe`; `internal/adapters/sfwp/ask_transport_test.go` `94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc`.

## Command and result

Working directory: `apps/godspeed-casework-go`.

```sh
env GOLDEN_UPDATE=0 GOMAXPROCS=2 GOGC=50 GOFLAGS=-p=1 \
  GOCACHE=/tmp/sea-rs-ask-runtime-gates/phase-a/gocache \
  TMPDIR=/tmp/sea-rs-ask-runtime-gates/phase-a/tmp \
  go test -race -count=1 -parallel=1 ./internal/server \
  -run '^TestAskRejectsMalformedRawUTF8BeforeVerificationOrDispatch$'
```

Actual exit status: `1` (expected RED). The Go package compiled; `subject`, `purpose`, and `case` each failed only because POST `/api/ask` returned `200` instead of `400`, with the fixture's successful disclosure body. `gofmt -d` over the four phase-A Go files was clean.

Actual-host preflight found 2.2 GiB available RAM and no competing compiler (the only matching rows were the preflight shell/awk itself). Full output, status, and preflight are preserved byte-for-byte in the sibling `.log`, `.exit`, and `-preflight.txt` files. Original `/tmp` SHA-256 values were respectively `fca430eb767f73503da249fc28548ee55d2c2dafc9c11f577c1d36f910859756`, `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`, and `5109ae09a872873f8f28ed1b1f6c3236694995760dd9a38f9b8b6db044cd358b`.

## Limits

No broad server, adapter, package, or module gates ran in phase A. No raw UTF-8 guard was present during the expected-RED run. The zero-contact counter assertions did not run because the status assertion terminated each subtest first. Phase-B source must add validation on original body bytes before JSON decoding or any identity/perspective/kernel call; rerun the focused fixture to observe 400 and both zero-contact assertions directly.
