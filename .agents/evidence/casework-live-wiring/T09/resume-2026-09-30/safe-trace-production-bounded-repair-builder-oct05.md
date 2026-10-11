# Safe trace bounded production repair record

This is a new source-only repair record. It is not compilation, test, scanner,
or independent-review evidence. The implementation remains runtime-unverified.

## Original instructions and review basis

The production authorization and scope are recorded in
`safe-trace-production-root-assignment-oct05.md`; the nil empty-success
clarification is in `safe-trace-production-assignment-empty-slice-erratum-oct05.md`.
The semantic requirements are approved V3 in
`observation-safe-trace-port-root-proposal-v3.md`, with test-first fixture
requirements in `safe-trace-test-first-root-assignment-oct05.md` and accepted
assertion RED in `safe-trace-fixture-independent-red-e8393465-oct05.md`.
The independent source review `safe-trace-production-independent-review-oct05.md`
rejected the earlier implementation at source hash
`6abc9572e4ae89570812a0395444aab56b1aab8a36809392a842f8dcf259523b` for (1)
retaining a full backing array behind a 1024-frame tail and (2) accepting
comma fractional seconds through Go's permissive time parser. This repair
addresses those two findings only.

Input SHA-256 identities:

| Input | SHA-256 |
|---|---|
| Production assignment | `31faeb43d160ffc2bf32f463e771fc17afc4693aff735c3ea6360853c00f0681` |
| Empty-slice erratum | `3e672aaf4c06c166823b394622cb1a8946506439020863080943b26835a87b9d` |
| Approved V3 proposal | `e280649130163c99cdc52f8d208e5eb64fd0c253a686130823d82f3a807085d6` |
| Independent production rejection | `18136b53983f4f91f1df1f6533ea199b5d27c2f2bdb75cd606cc0e9c587deb14` |
| Accepted fixture RED record | `cb8a9adc6a103fbd44ced9d43511c5bc1dfe95ef0d2334139439a5fa6c8b56fb` |
| Reviewed implementation before repair | `6abc9572e4ae89570812a0395444aab56b1aab8a36809392a842f8dcf259523b` |

## Source identities after repair

| Source | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `efb70d16a98e087e13149fbcf2fa4a7429603ed07614fd4436ba3d7436341d20` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `5ccc705112ea02baf9c26f349376e5e97b95cb86c6b83c7a44988a18fc72dae1` |

## Exact repair diff

This is the complete source/test delta from the rejected implementation above.
The retained frame ring scans and validates every returned row and keeps the
full selected-ID set for duplicate rejection, while the frame storage itself
never grows past 1024. The final ring is copied in original order into a fresh
exact-size slice. Empty `trace: []` retains its approved nil frame representation.
The test checks visible length/capacity bounds and rejects a comma-fraction
timestamp; backing-array reclamation is also evident from the fresh allocation
in the implementation and cannot be observed directly through the safe slice API.

```diff
--- a/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
+++ b/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
@@ -46,12 +46,19 @@ func (a *Authority) ReadRunTrace(ctx context.Context, caseID, runID, planItemID string) (ports.RunTraceSnapshot, error) {
 	rows, ok := view.trace.([]any)
 	if !ok {
 		return ports.RunTraceSnapshot{}, runTraceUnavailable()
 	}
 	var selected []ports.RunTraceFrame
 	if len(rows) > 0 {
-		selected = make([]ports.RunTraceFrame, 0, len(rows))
+		capacity := len(rows)
+		if capacity > runTraceRetention {
+			capacity = runTraceRetention
+		}
+		selected = make([]ports.RunTraceFrame, 0, capacity)
 	}
+	retainedStart := 0
+	total := 0
 	seen := make(map[string]struct{}, len(rows))
@@ -73,6 +80,10 @@ func (a *Authority) ReadRunTrace(ctx context.Context, caseID, runID, planItemID string) (ports.RunTraceSnapshot, error) {
 		if !ok || strings.TrimSpace(timestamp) == "" {
 			return ports.RunTraceSnapshot{}, runTraceUnavailable()
 		}
+		// Go's layout parser also accepts comma as a fractional-second separator.
+		if strings.Contains(timestamp, ",") {
+			return ports.RunTraceSnapshot{}, runTraceUnavailable()
+		}
 		if _, err := time.Parse(time.RFC3339Nano, timestamp); err != nil {
 			return ports.RunTraceSnapshot{}, runTraceUnavailable()
 		}
@@ -96,12 +107,22 @@ func (a *Authority) ReadRunTrace(ctx context.Context, caseID, runID, planItemID string) (ports.RunTraceSnapshot, error) {
 			frame.ExecutionStatus = status
 			frame.ExitCode = exitCode
 		}
-		selected = append(selected, frame)
+		if len(selected) < runTraceRetention {
+			selected = append(selected, frame)
+		} else {
+			selected[retainedStart] = frame
+			retainedStart = (retainedStart + 1) % runTraceRetention
+		}
+		total++
 	}
 
-	total := len(selected)
 	if total > runTraceRetention {
-		selected = selected[total-runTraceRetention:]
+		retained := make([]ports.RunTraceFrame, runTraceRetention)
+		copy(retained, selected[retainedStart:])
+		copy(retained[len(selected)-retainedStart:], selected[:retainedStart])
+		selected = retained
 	}
--- a/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go
+++ b/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go
@@ -387,6 +387,7 @@ func TestReadRunTraceV3AllowlistIdentityAndRetention(t *testing.T) {
 		{"wrong-type selected timestamp", `{"event_id":"evt_wrong_time","kind":"run_started","timestamp":17}`},
 		{"malformed selected timestamp", runTraceRow("evt_bad_time", "run_started", "yesterday", `{}`)},
+		{"comma fractional timestamp is not RFC3339", runTraceRow("evt_comma_time", "run_started", "2026-10-05T12:13:14,123Z", `{}`)},
 		{"null trace row", `null`},
 		{"scalar trace row", `"not an object"`},
@@ -429,6 +430,16 @@ func TestReadRunTraceV3AllowlistIdentityAndRetention(t *testing.T) {
 			}
 			want := validRunTraceSnapshot(frames, count)
 			assertRunTraceCase(t, runTraceResponse("["+strings.Join(rows, ",")+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
+			if count > 1024 {
+				fixture := newRunTraceFixture(t, runTraceResponse("["+strings.Join(rows, ",")+"]", "", "", "", ""))
+				got, err := fixture.authority.ReadRunTrace(context.Background(), runTraceCaseID, runTraceRunID, runTraceItemID)
+				if err != nil {
+					t.Fatalf("read bounded retained frames: %v", err)
+				}
+				if len(got.Frames) != 1024 || cap(got.Frames) > 1024 {
+					t.Fatalf("retained frames length/capacity = %d/%d, want 1024 and capacity at most 1024", len(got.Frames), cap(got.Frames))
+				}
+			}
 		})
 	}
 }
```

## Deviations and verification limits

There were no scope, interface, or semantic deviations. Existing full-array
validation, duplicate detection, returned count, source order, authority refusal
handling, nil empty success, and zero-on-error behavior were retained. Only the
two authorized files were edited. No compilation, tests, scanners, Git, status,
or debt operations were run; runtime behavior is unverified. The root owns the
compiler token and independent trace critic review before any runtime claim.
