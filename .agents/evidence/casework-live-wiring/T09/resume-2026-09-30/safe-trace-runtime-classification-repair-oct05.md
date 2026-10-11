# Safe trace runtime classification repair

Date: 2026-10-05. Builder record only; runtime verification remains unrun and
the independent trace critic owns the sole compiler token.

## Original instructions and governing inputs

Assignment: fresh bounded source repair in
`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` and
`run_trace_test.go` only. Read and obey:

- `safe-trace-production-root-assignment-oct05.md` (original production
  assignment: implement the V3 projection on one `Client.Do(NewRunGet(runID))`,
  preserve refusal classes and retries, validate identities/standings/trace
  presence/rows, retain newest 1024 in source order, preserve count and exact
  integer behavior, return zero snapshots on failures, and avoid disclosure of
  raw values);
- `observation-safe-trace-port-root-proposal-v3.md` (approved semantic and
  test-matrix requirements);
- `safe-trace-production-assignment-empty-slice-erratum-oct05.md` (empty
  semantic frame slice may be nil and this fixture expects nil; preserve all
  count/order/capacity/error assertions);
- `safe-trace-timestamp-v3-race-independent-review-oct05.md` (focused runtime
  rejected count-0 expected slice and raw-response decoding classification);
- `safe-trace-runtime-root-adjudication-oct05.md` (repair only count-0 fixture
  expectation and classify a `Client.Do` error as unavailable only when it is a
  typed `apperr.Error` with `Op == "decode"`, first preserving any `Refusal`
  anywhere in the chain; preserve all other errors);
- Original builder assignment `safe-trace-test-first-root-assignment-oct05.md`,
  source-review approval/rejection records, fixture `ee544329` and timestamp
  repair source `722e3270`.

No compiler, test, scanner, Git, status, or debt operation was performed. No
decoder, retry, transport, authority, schema, dependency, or prose-classification
change is included.

## Frozen source identities

| File | Before repair SHA-256 | After repair SHA-256 |
|---|---|---|
| `run_trace.go` | `722e32701b4105b0ff4df3c4acb28e34d696521f10f05171f44afbd927077a1d` | `3b6b0a7320c73143aca0b101b89e3c64472e85d43f76f00976aac4a21d692d39` |
| `run_trace_test.go` | `ee544329730af2dfab4cdfb34cd001f2184879d1fc5ffad285b09564221f7b53` | `dd97281929948a99a279c6133fc7568bb7fdf6a686e85b8bfdd0a89a323da78e` |

## Exact source diff

```diff
--- run_trace.go (722e327)
+++ run_trace.go (3b6b0a7)
@@
 	"context"
 	"encoding/json"
+	"errors"
@@
 	resp, err := a.client.Do(ctx, NewRunGet(runID))
 	if err != nil {
+		var refusal *Refusal
+		if errors.As(err, &refusal) {
+			return ports.RunTraceSnapshot{}, err
+		}
+		var appError *apperr.Error
+		if errors.As(err, &appError) && appError.Op == "decode" {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
 		return ports.RunTraceSnapshot{}, err
 	}
--- run_trace_test.go (ee54432)
+++ run_trace_test.go (dd97281)
@@
-			frames := make([]ports.RunTraceFrame, 0, count-retainedStart)
+			var frames []ports.RunTraceFrame
+			if count > 0 {
+				frames = make([]ports.RunTraceFrame, 0, count-retainedStart)
+			}
@@
 	fixture.assertSingleRunGet(t, runTraceRunID)
 }
+
+func TestReadRunTraceV3PreservesInternalAuthorityRefusal(t *testing.T) {
+	fixture := newRunTraceFixture(t, `{"error":"authority could not process request","error_class":"internal_failure","no_side_effect":true}`)
+	got, err := fixture.authority.ReadRunTrace(context.Background(), runTraceCaseID, runTraceRunID, runTraceItemID)
+	if err == nil || apperr.KindOf(err) != apperr.KindInternal {
+		t.Fatalf("authority refusal error = %v, want typed internal", err)
+	}
+	if !reflect.DeepEqual(got, ports.RunTraceSnapshot{}) {
+		t.Fatalf("authority refusal returned safe data: %#v", got)
+	}
+	var refusal *Refusal
+	if !errors.As(err, &refusal) || refusal.RefusalClass() != "internal_failure" {
+		t.Fatalf("internal authority refusal class was not preserved: %v", err)
+	}
+	fixture.assertSingleRunGet(t, runTraceRunID)
+}
```

## Deviations and result

The assignment explicitly permitted a focused regression proving preservation
of an internal-class `Refusal` when the existing fixture covered only a
different class. Added that regression; the existing identity-required refusal
case remains unchanged. The count-0 expected frames are now nil, while the
positive counts and capacity assertion are untouched. Production recognizes
typed decode errors structurally and checks the full error chain for `Refusal`
first; all non-decode, non-refusal errors continue to pass through unchanged.

Runtime remains unverified. This record makes no test-pass or runtime-approval
claim. Independent source review and the assigned focused race gate are next.
