# Independent source review: safe trace runtime classification repair

Date: 2026-10-05. Verdict: **SOURCE APPROVED for the next assigned focused
runtime gate only.** This record is not runtime evidence and does not grant a
compiler token.

## Review envelope and identities

Reviewed the original production assignment, approved V3 proposal, empty-slice
erratum, prior source rejections and focused runtime rejection, root runtime
adjudication, latest builder instructions and complete frozen diff. Controlling
input hashes:

| Evidence/input | SHA-256 |
|---|---|
| `safe-trace-production-root-assignment-oct05.md` | `31faeb43d160ffc2bf32f463e771fc17afc4693aff735c3ea6360853c00f0681` |
| `observation-safe-trace-port-root-proposal-v3.md` | `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6` |
| `safe-trace-production-assignment-empty-slice-erratum-oct05.md` | `3e672aaf4c06c166823b394622cb1a8946506439020863080943b26835a87b9d` |
| Prior focused runtime review | `cbcd6aeb390ffc386d2e66fe85ddc16bd8518f1d8991f2582ec3dad3773ede25` |
| Root runtime adjudication | `72477801068b3c47c2e7a5edce95001bee43423858b7a57b49594e8b726d9a6f` |
| Runtime-classification builder record | `fbdf35c33fe6e00ce15b4ff6459a759a1aa6afefe379ad6a2c19b7770cf5e702` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `3b6b0a7320c73143aca0b101b89e3c64472e85d43f76f00976aac4a21d692d39` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `dd97281929948a99a279c6133fc7568bb7fdf6a686e85b8bfdd0a89a323da78e` |

The root adjudication authorizes two focused changes: normalize only typed
`apperr.Error` response-decoder failures (`Op == "decode"`) to the existing
fixed unavailable error, while preserving any refusal in the error chain; and
change only the count-zero expected frames to nil under the empty-slice erratum.
It also explicitly requires non-decode errors to pass through unchanged.

## Decoder error classification and refusal preservation

The `Client.Do` error branch first uses `errors.As` to search the complete
chain for `*Refusal` and returns the original error unchanged when found
(`run_trace.go:72-82`). Only after that check does it search for a typed
`*apperr.Error` with `Op == "decode"` and replace it with the existing fixed
`runTraceUnavailable()` (`:78-80`). All other errors pass through unchanged
(`:82`). This follows the adjudication ordering and does not classify by error
message text or echo decoder details.

The explicit fixture regression constructs an `internal_failure` authority
response and asserts typed internal kind, exact zero snapshot, preserved
`Refusal` chain/class, and one `run.get` request (`run_trace_test.go` per the
builder diff). This verifies the refusal-preservation branch against the
strongest internal-class case. The prior raw invalid JSON expectations remain
unchanged and now align with the adjudicated decoder-error normalization.

## Empty expectation and rest of the diff

The count table now leaves `frames` nil only when `count == 0`, allocating the
nonempty expected slice for positive counts (`run_trace_test.go:442-460`). The
1024/1027 expectations, source order, count, and capacity assertion are
unchanged. This is exactly the erratum-aligned correction recorded in root
adjudication; the empty success case itself is retained.

The builder’s complete diff otherwise contains only the required `errors`
import, the narrow `Client.Do` error classification, the zero-only expected
slice initialization, and the added internal-failure refusal regression. It
preserves timestamp grammar validation, ring retention and fresh bounded copy,
full-array duplicate validation, all selected-kind/shape/privacy assertions,
and existing refusal coverage. I found no other deviation from the frozen
contract or authorized repair scope.

## Verdict and limits

The exact frozen source is **approved for the next assigned focused runtime
gate only**, once root transfers the sole compiler token. No compile, test,
scanner, Git, status, or debt operation was run during this review. Runtime
behavior remains unverified; this approval grants neither a broader SFWP or
module gate nor any publication or settlement claim.
