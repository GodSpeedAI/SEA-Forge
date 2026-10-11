# Safe trace fixture fresh builder record — 2026-10-05

## Scope and binding

This fresh, source-only repair follows `safe-trace-test-first-root-assignment-oct05.md`, accepted proposal V3 (`observation-safe-trace-port-root-proposal-v3.md`, SHA-256 `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6`), and its proposal approval. It addresses the missing fixture coverage recorded by the latest independent source review and supplement. The only source file changed in this repair is `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go`.

## Exact source identity

Target SHA-256 before this repair: `3ffab96a1bdf8235db876d4ada4cc6978279952b0db676ba15b8594e991bcf64`.

Target SHA-256 after this repair: `a3c13aba36756d3f9f3e7657aeb3846b865dc430624e41f97c85640ae24806dc`.

The adjacent port and adapter implementation files remain unchanged at their recorded hashes: `ports/run_trace.go` `88191de13d6a21df988dcc3cb834df9015cdb6c2c2aa73a555e10583b108daf5`; `adapters/sfwp/run_trace.go` `944cf97fd4d6b055f6ec3940086eb3f7ccbce28b7ad0642d023c1a80870c0e51`.

## Change inventory

- Added a valid JSON identity-response builder that omits exactly one requested ownership key, and used it for each missing-field case. This avoids malformed JSON caused by assembling absent members from empty strings.
- Strengthened the common error assertion helper: every returned error must accompany the exact zero `RunTraceSnapshot`, including blank/invalid IDs, refusal, projection errors, and malformed JSON. Existing success assertions remain in place.
- Expanded selected-row rejection fixtures to cover null and numeric event IDs; missing, null, and numeric timestamps; and null and scalar trace rows. Existing malformed and blank-value cases remain.
- Added malformed unknown-kind rows with missing, null, and numeric IDs, malformed timestamps, and private payload markers. The existing unknown-kind ID-collision case remains. These are paired with a valid selected row so ignored rows cannot mask selected behavior.
- Expanded the trace-presence matrix with missing, null, and wrong-type `present` values and null `bytes`, retaining existing invalid-shape cases.
- Added a raw invalid-JSON response fixture which expects unavailable projection and retains the existing assertion that exactly one logical `run_get` request occurred. The client decodes after transport retries, so this malformed response does not itself cause another request.
- Added a dedicated row encoder that omits the `payload` member entirely. The absent-payload case uses this encoder, while existing fixtures using the shared helper retain their previous `{}` default.

All pre-existing fixture cases and assertions were preserved. The V3 semantics remain unchanged: selected event IDs are validated over the full returned array; unknown kinds are ignored before their other fields are inspected; only trace-record presence metadata is projected; and no completeness or byte/row consistency claim is introduced.

## Checks and deviations

`gofmt -d apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` produced no output. No Go compiler, tests, Rust compiler, or other source/status/debt/Git changes were made; the compiler token was reserved by the parent for the normal checkpoint hook. No material deviation from the bounded assignment was identified. This record is a builder handoff, not independent approval or a test-pass claim.
