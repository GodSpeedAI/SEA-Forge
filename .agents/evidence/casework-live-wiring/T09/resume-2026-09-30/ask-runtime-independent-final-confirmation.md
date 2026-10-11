# Independent confirmation: protected Ask runtime

**Verdict: APPROVE this bounded Ask runtime unit only.** This does not settle T09 or approve the remaining observations, pollers, UI, or global T09 work.

I reviewed the unchanged `ask-runtime-implementation-assignment.md`, `ask-runtime-root-integration-supplement.md`, `ask-live-ledger-projection-clarification.md`, the prepared route/integration decisions and test-first fixtures, and the frozen phase-B source. I reran the focused Go server/SFWP tests, actual-kernel live proof, required `just casework-go-check`, and full-module race gate. The fixture rename was reviewed separately and is limited to the test function identifier.

## Source identity at the passing source graph

Production and integration files:

- `apps/godspeed-casework-go/internal/server/ask.go` — `68a588dfe741ae2e5a13b58229c78da4016a94f0355bf54e08952cccb102b699`
- `apps/godspeed-casework-go/internal/adapters/sfwp/ask.go` — `75702c8577327b84d7cafeb23171c400e914d1a60f575e9e47588f701e8749eb`
- `apps/godspeed-casework-go/internal/adapters/sfwp/client.go` — `8cdfc52a68c07df84e1f88506163da1a8dab212004d45689a4c8ab8007b3b61a`
- `apps/godspeed-casework-go/internal/adapters/sfwp/frame.go` — `ab7d05604574029f684874d9da415f9c3c1695778dc45ea12e3bbbb462167185`
- `apps/godspeed-casework-go/internal/server/server.go` — `31d5d38e8f4f080d472179ce0da7f63ac853a2ed57021d99eea2708c8cd7a023`
- `apps/godspeed-casework-go/internal/server/http.go` — `ebe23204a2390e4611c77d594e82cd6dfcb0a32343c1113e4900d11cfc5cf25c`
- `apps/godspeed-casework-go/internal/server/ratelimit.go` — `53e91b67580f79fa002073cb8eef2b43e0d63a1d21548d4f4cbb4b32686edd7b`
- `apps/godspeed-casework-go/internal/ports/ask.go` — `09c6af57138e52fe0e5dfe504b08a6a4517fa8d65955a7cb508e463f884de6b6`
- `apps/godspeed-casework-go/internal/livestack/stack.go` — `6c1021b2204cd1f4327a527b3feebd480e1e8007b0bc6bd31f2d7564b8629af5`
- `apps/godspeed-casework-go/cmd/godspeed-casework/main.go` — `2a9b5c6620aef9d8a7b8cd9f597f245a8a69eae17c84ff172bf55f9c773771a6`

Tests and fixture identity:

- `apps/godspeed-casework-go/internal/adapters/sfwp/ask_adapter_test.go` — `7564a7ef04c2b0af242b6a7f270b4a9aef701221ec8ce39ec3f6c51cb4678200`
- `apps/godspeed-casework-go/internal/server/ask_live_test.go` — `b97556553bf65a1105e0bfaaccb57e0f421b004141c3a73f782a3cebb0c5478a`
- Replacing only `TestLiveAskLedger` with its original name reproduces the pre-portability source hash exactly: `8f3613930cea8052671611549646419c6e6f6e02007a7d28631417f1d3d7eb0f`.
- Frozen `apps/godspeed-casework-go/internal/server/ask_test.go` — `90612d084a10fce6ec4871faae563790c7de5267d2ef60369bc1fc23bd538cbe`
- Frozen `apps/godspeed-casework-go/internal/adapters/sfwp/ask_transport_test.go` — `94764ad34ca808bfa92ed96bfd91acd23fb8767c5fa9645515e57f91395426cc`
- Phase-A raw UTF-8 fixture `apps/godspeed-casework-go/internal/server/ask_validation_test.go` — `a1888b765e7219513bf0ca9b2bd083631f52050f25c230c8e0ead7d1d82de861`

## Findings and material differences

The route registers protected `POST /api/ask` with session, CSRF, separate session/IP buckets, strict body decoding, effective-actor perspective verification, then Ask dispatch. The 8192-byte read is bounded, raw UTF-8 is checked before JSON normalization, and the decoder accepts only one object with the approved fields and nine kinds. It preserves valid supplied subject/case values, distinguishes omitted purpose (`planning`) from explicit empty purpose, rejects invalid or oversized values, and validates before perspective or Ask contact. Nil Ask returns unavailable.

The SFWP adapter sends `ask` with the configured gateway principal, effective actor as `on_behalf_of` and `actor_id`, and no `request_id`; omitted case remains absent. It validates and maps the full answer, including required non-null arrays, supported enums, all claim references, optional capability-reference presence, and every answer metadata field. Invalid or unsupported upstream disclosures fail unavailable instead of receiving invented defaults. Tests cover complete mapping, empty arrays, omitted optional fields, malformed/missing/null arrays, unknown values, and explicit `capability_record_ref:null` rejection.

Transport handling keeps correlated mutation recovery and read-only inspection retry behavior while excluding uncorrelated Ask from ambiguous resend and status recovery. Ask EOF, timeout, and overflow tests assert one Ask and zero `request_get_status` calls; the explicit `server_busy` refusal retains its separate bounded retry. Existing SFWP regressions passed.

The live test uses the existing kernel and CLI binaries in a disposable cell. It confirms authenticated HTTP/session/CSRF behavior, that auth/CSRF/invalid-body refusals produce no Ask ledger or delegation records, real denied and granted answers for a concept selected from a rebuilt bundled model, the four linked question/plan/decision/answer records for each answer, effective-user writer and delegation attribution, and ledger verification through the CLI. Its API/ledger comparison keeps all eleven top-level answer fields and compares claims through the exact canonical eight-required-field plus optional-reference `ClaimView` projection. It also confirms persisted `GroundedClaim`-only flags, limitations, and authored_by instead of treating unlike stored and API shapes as literal equals.

Scope additions to `ports/ask.go`, live stack injection, application wiring, adapter tests, and the live test were expressly authorized by the root integration supplement. The phase-A decoder helper rename resolves the fixture symbol collision. Phase B adds the needed raw-byte `utf8.Valid` check before JSON decoding, and corrects stale comments that conflated protected writes with correlated mutations. The only later fixture change was the separately authorized live test name shortening; its body is byte-identical after reversing that identifier substitution.

## Verification evidence

All raw logs, exit files, and host preflights are preserved in sibling phase-B evidence files. The earlier phase-B interim manifest records the phase-B source hashes and initial gates. Repeated `/tmp` originals and byte-identical archive copies are listed below for the final source graph:

| Gate | Original `/tmp` files and SHA-256 | Archived evidence |
|---|---|---|
| Focused server + adapter race, exit 0 | `/tmp/sea-rs-ask-runtime-gates/phase-b/focused-server-adapter.log` `883490ba94f800554d97602739e41f2f3b478b2ed8056131cab9da5e54c26f40`; `.exit` `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`; `preflight-focused.txt` `26530b8f8de1e7f0fe5dd88bee37d36e26559da18b70ce7fbb12064114f17f68` | `ask-runtime-phase-b-focused-race.{log,exit}` and `ask-runtime-phase-b-focused-race-preflight.txt` |
| Live proof after short-name repair, exit 0 | `/tmp/sea-rs-ask-runtime-gates/phase-b/live-kernel-ask-renamed.log` `604c79b7a8e847c32fc756de957a177e3eb74d833f5c08290efd3f25f25a142c`; `.exit` `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`; `preflight-live-renamed.txt` `53fbd61771150f61dee6ee647df7ada7109bb6e23b9911c801e0bd17bcf4f38d` | `ask-runtime-phase-b-live-renamed.{log,exit}` and `ask-runtime-phase-b-live-renamed-preflight.txt` |
| Actual disposable socket path capture, live proof exit 0 | `/tmp/sea-rs-ask-runtime-gates/phase-b/live-kernel-ask-path-capture.log` `c883808053b8262ee92be40036a1407e52955dc6bf912bf7b4cac2c6db4fb15f`; `.exit` `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`; `.txt` `de51fd85003b49676b2194ca19606188414ed7b54e1a07df7d3173b62662a522`; `preflight-live-path-capture.txt` `5a5518175d51d015a052c76f53702828b4f12e3378032ca26ef813f8f7e2c7a4` | `ask-runtime-phase-b-live-path-capture.{log,exit,txt}` and `ask-runtime-phase-b-live-path-capture-preflight.txt` |
| Fresh `just casework-go-check`, exit 0 | `/tmp/sea-rs-ask-runtime-gates/phase-b/just-casework-go-check-renamed.log` `264f6ebcf36f2b449e549f8fff7b1504d0c4400869df091f3ccf1cc484bc7813`; `.exit` `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`; `preflight-just-go-renamed-compiler-only.txt` `523814f71e9b10a3b25d08549f4a1ab396e1737f0c3df1f5170e15d4897ee5cf` | `ask-runtime-phase-b-just-go-renamed.{log,exit}` and `ask-runtime-phase-b-just-go-renamed-preflight.txt` |
| Full Go module race/count1/parallel1, exit 0 | `/tmp/sea-rs-ask-runtime-gates/phase-b/full-module-race.log` `67cbbf416eaf67ba8d5e2034bbfd9da761ce164765e4299b770996dba4f47628`; `.exit` `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`; `preflight-full-module-race.txt` `5e75b134de7c04bb9d83d92946cf28d69deca82b760b000e23836be483e2624e` | `ask-runtime-phase-b-full-race.{log,exit}` and `ask-runtime-phase-b-full-race-preflight.txt` |

The full-module command used `GOLDEN_UPDATE=0 GOMEMLIMIT=256MiB GOMAXPROCS=2 GOGC=50 GOFLAGS=-p=1`, race detection, `-count=1`, and `-parallel=1`. The focused and live runs were serialized, with actual-host preflight before each command and no competing compiler. `gofmt -d` and `git diff --check` were clean before gates.

## Failed attempts and limitations

The original long-name live fixture failed to start because its Unix socket path exceeded the kernel's 95-byte limit. The original phase-B TMPDIR attempt's `t.TempDir` cleanup removed its server log; that missing log is not reconstructed. A captured later server log reported a 98-byte socket under `/tmp/asktmp` against the explicit 95-byte limit. A default-sandbox attempt with a 93-byte path got `Operation not permitted`; it is retained as a sandbox failure, not counted as kernel proof. These failures and their preflights/outputs remain separate in the sibling `ask-runtime-phase-b-live-*` files.

The renamed live test passed under the original task-owned TMPDIR. A concurrent watcher recorded the actual socket as `/tmp/sea-rs-ask-runtime-gates/phase-b/tmp/TestLiveAskLedger449696363/001/server.sock`, 84 bytes. The abbreviated predicted 85-byte preflight used a different illustrative random suffix; the 84-byte actual path is the direct observation.

An initial post-rename preflight used a broad argument regex and matched the Codex sandbox wrapper because its shell arguments contained the regex itself. It did not trigger a compile. The actual `just` run was preceded by a fresh compiler-process-name-only host scan, whose raw output is archived; unrelated node/sandbox process arguments were not treated as competing builds.

No UI, Rust, schema/golden, or full-T09 settlement gates ran in this bounded review. T09 remains pending its other units.
