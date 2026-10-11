# Independent review: safe trace timestamp grammar repair

Date: 2026-10-05. Verdict: **SOURCE APPROVED for the focused V3 assertion run.**
This review alone is not runtime evidence and does not approve broader gates.

## Review envelope and identities

Reviewed the original production assignment, approved V3 proposal, empty-slice
erratum, both prior independent production rejections, the bounded-repair
record, this timestamp-repair builder record, and the full frozen source/test
diff. The controlling contract hashes match the evidence records:

| Evidence/source | SHA-256 |
|---|---|
| `safe-trace-production-root-assignment-oct05.md` | `31faeb43d160ffc2bf32f463e771fc17afc4693aff735c3ea6360853c00f0681` |
| `safe-trace-production-assignment-empty-slice-erratum-oct05.md` | `3e672aaf4c06c166823b394622cb1a8946506439020863080943b26835a87b9d` |
| `observation-safe-trace-port-root-proposal-v3.md` | `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6` |
| Prior source review of original implementation | `18136b53983f4f91f1df1f6533ea199b5d27c2f2bdb75cd606cc0e9c587deb14` |
| Prior bounded-repair review | `f4306fbc06e07e63b854481d2414e3d0407f7bd3a7f7cae65c9debaecac887e4` |
| Timestamp-repair builder record | `ffd2df5cc780862c15cc95e798f95ece578888b08b04038503c385225a33b829` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `722e32701b4105b0ff4df3c4acb28e34d696521f10f05171f44afbd927077a1d` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `ee544329730af2dfab4cdfb34cd001f2184879d1fc5ffad285b09564221f7b53` |

The assignment authorizes implementation only in `run_trace.go`; the fresh
repair specifically permits `run_trace.go` and `run_trace_test.go` changes to
complete timestamp syntax validation and add regression cases. V3 requires
RFC3339/RFC3339Nano selected timestamps and preservation of their source text.
The empty-slice erratum permits nil frames for valid empty results and requires
exact zero snapshots on errors.

## Timestamp validation: accepted by source inspection

`isRFC3339NanoSyntax` first enforces minimum input length, exact separator
positions, uppercase `T`, and fixed-width ASCII digits in year/month/day/hour/
minute/second (`run_trace.go:30-37`). The optional fraction must use a period
and contain at least one ASCII digit (`:40-50`). The zone must be uppercase
`Z` or exactly signed `HH:MM` with ASCII digits, a colon, hour less than 24,
and minute less than 60 (`:52-62`). The existing `time.Parse` call remains
after this syntax gate (`:127-132`) to validate calendar and clock values. The
projected frame retains the exact input string (`:139`); it does not normalize
or reformat timestamp text.

This closes the prior finding: Go's `time.Parse` generic fallback could accept
forms rejected by its RFC3339 fast parser, including one-digit clock/zone
fields and out-of-range zone components. The syntax gate now rejects those
forms before `time.Parse`. The added negative table covers comma fractions,
single-width hour/minute/second and offset hour, missing fractional digits,
missing zone colon, and offset hour/minute range failures (`run_trace_test.go:
390-399`). Positive cases cover UTC, fractions, zero offsets, and the maximum
`+23:59`/`-23:59` offsets while asserting exact timestamp preservation
(`:408-421`). Parsing remains responsible for calendar validity.

## Other source and fixture checks

The fresh builder diff is confined to the two specifically authorized files.
It retains the bounded ring and final fresh exact-size copy accepted in the
prior review, full-array selected ID uniqueness checks, pre-retention returned
count, source order, fixed unavailable projection errors, refusal propagation,
and nil empty-success representation. The full listed diff adds timestamp
validation and corresponding timestamp cases only; it does not weaken or
remove prior assertions. No contract, port, client, or unrelated implementation
change appears in the diff.

## Verdict and verification boundary

The frozen source is **approved for the single focused `TestReadRunTraceV3`
race run**, subject to the separate compiler-token, resource-preflight,
capture, and joined-exit controls. This is not evidence that the command has
run or passed. Full SFWP/module gates remain unapproved and held. No compile,
test, scanner, Git, status, or debt operation was run during this source
review.
