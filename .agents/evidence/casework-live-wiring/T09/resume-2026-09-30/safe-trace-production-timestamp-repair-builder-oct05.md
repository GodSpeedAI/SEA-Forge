# Safe trace production timestamp repair builder record

Date: 2026-10-05. Fresh bounded source-only repair after independent review
rejected the comma-only guard because Go's fallback parser also accepts
non-fixed-width time fields and out-of-range numeric offsets. This record
requests fresh independent review. It is not compile, runtime, or approval
evidence.

## Original instructions and binding scope

The original assignment in `safe-trace-production-root-assignment-oct05.md`
authorizes only `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go`.
It requires implementing approved proposal V3 while preserving one logical
`Client.Do(NewRunGet(runID))`, existing retries and authority refusal classes,
blank expected-ID rejection before dialing, exact identity and standings,
trace presence, selected-row validation and duplicate detection across the
full response, optional command metadata, newest-1024 source-order retention,
returned-frame counts, unknown string-kind omission before inspecting other
fields, zero snapshot on decode/projection failures, and private fixed errors.
It prohibits changes to client, tests, port declarations, authorization,
stores, server, UI, Rust, schema, dependencies and hooks. It prohibits
compiler, test, scanner, Git, status and debt operations. A new immutable
record with the original instructions, exact diff, source hashes and
deviations is required.

The empty-slice erratum
`safe-trace-production-assignment-empty-slice-erratum-oct05.md` additionally
requires nil `Frames` for successful empty arrays and exactly zero snapshot on
error, and authorizes fixing the two error-case expected values in the test
fixture. The accepted V3 contract requires RFC3339/RFC3339Nano timestamp
validation while preserving the original timestamp text and otherwise
preserving selected safe frames. The previous bounded repair record
`safe-trace-production-bounded-repair-builder-oct05.md` documents source
identities before this repair and the already authorized ring-retention and
comma-guard changes. Independent review then rejected the implementation:
the comma guard plus `time.Parse` still accepted a one-digit hour and offsets
with hour >= 24 or minute >= 60 through Go's fallback parser. This fresh
assignment authorizes only `run_trace.go` and `run_trace_test.go` to add full
RFC3339Nano syntax checks before/with `time.Parse` and regression cases.

The timestamp grammar enforced here uses exact `YYYY-MM-DDTHH:MM:SS`, an
optional period followed by one or more ASCII decimal digits, and uppercase
`Z` or a numeric `+HH:MM` / `-HH:MM` zone. Numeric offset hours must be below
24 and minutes below 60. `time.Parse(time.RFC3339Nano, value)` retains the
existing parser's precision acceptance and validates calendar and clock
values. Successful projection preserves the exact input timestamp string.

## Source identities

| Source | Before SHA-256 | After SHA-256 |
|---|---|---|
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go` | `efb70d16a98e087e13149fbcf2fa4a7429603ed07614fd4436ba3d7436341d20` | `722e32701b4105b0ff4df3c4acb28e34d696521f10f05171f44afbd927077a1d` |
| `apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go` | `5ccc705112ea02baf9c26f349376e5e97b95cb86c6b83c7a44988a18fc72dae1` | `ee544329730af2dfab4cdfb34cd001f2184879d1fc5ffad285b09564221f7b53` |

## Full actual source diff

The following is the full delta from the rejected frozen source/test pair
identified above. Existing ring retention, full-array uniqueness validation,
error behavior, selected kinds, privacy, counts, and successful-empty nil
representation are unchanged.

```diff
--- a/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
+++ b/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go
@@
 const runTraceRetention = 1024
+
+func hasASCIIDigits(value string) bool {
+	for i := range value {
+		if value[i] < '0' || value[i] > '9' {
+			return false
+		}
+	}
+	return true
+}
+
+func isRFC3339NanoSyntax(value string) bool {
+	if len(value) < len("2006-01-02T15:04:05Z") ||
+		value[4] != '-' || value[7] != '-' || value[10] != 'T' ||
+		value[13] != ':' || value[16] != ':' ||
+		!hasASCIIDigits(value[:4]) || !hasASCIIDigits(value[5:7]) ||
+		!hasASCIIDigits(value[8:10]) || !hasASCIIDigits(value[11:13]) ||
+		!hasASCIIDigits(value[14:16]) || !hasASCIIDigits(value[17:19]) {
+		return false
+	}
+
+	zoneStart := 19
+	if zoneStart < len(value) && value[zoneStart] == '.' {
+		fractionStart := zoneStart + 1
+		zoneStart = fractionStart
+		for zoneStart < len(value) && value[zoneStart] >= '0' && value[zoneStart] <= '9' {
+			zoneStart++
+		}
+		if zoneStart == fractionStart {
+			return false
+		}
+	}
+
+	if zoneStart == len(value)-1 && value[zoneStart] == 'Z' {
+		return true
+	}
+	if len(value)-zoneStart != 6 || (value[zoneStart] != '+' && value[zoneStart] != '-') ||
+		value[zoneStart+3] != ':' || !hasASCIIDigits(value[zoneStart+1:zoneStart+3]) ||
+		!hasASCIIDigits(value[zoneStart+4:zoneStart+6]) {
+		return false
+	}
+	offsetHour := int(value[zoneStart+1]-'0')*10 + int(value[zoneStart+2]-'0')
+	offsetMinute := int(value[zoneStart+4]-'0')*10 + int(value[zoneStart+5]-'0')
+	return offsetHour < 24 && offsetMinute < 60
+}
 
 // ReadRunTrace returns only allowlisted metadata from one authority run.get response.
@@
-		// Go's layout parser also accepts comma as a fractional-second separator.
-		if strings.Contains(timestamp, ",") {
+		// Go's RFC3339 parser falls back to a more permissive layout parser.
+		if !isRFC3339NanoSyntax(timestamp) {
 			return ports.RunTraceSnapshot{}, runTraceUnavailable()
 		}
--- a/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go
+++ b/apps/godspeed-casework-go/internal/adapters/sfwp/run_trace_test.go
@@
 		{"wrong-type selected timestamp", `{"event_id":"evt_wrong_time","kind":"run_started","timestamp":17}`},
 		{"malformed selected timestamp", runTraceRow("evt_bad_time", "run_started", "yesterday", `{}`)},
 		{"comma fractional timestamp is not RFC3339", runTraceRow("evt_comma_time", "run_started", "2026-10-05T12:13:14,123Z", `{}`)},
+		{"single-digit hour is not RFC3339", runTraceRow("evt_single_hour", "run_started", "2026-10-05T1:13:14Z", `{}`)},
+		{"single-digit minute is not RFC3339", runTraceRow("evt_single_minute", "run_started", "2026-10-05T12:3:14Z", `{}`)},
+		{"single-digit second is not RFC3339", runTraceRow("evt_single_second", "run_started", "2026-10-05T12:13:4Z", `{}`)},
+		{"fraction requires digits", runTraceRow("evt_empty_fraction", "run_started", "2026-10-05T12:13:14.Z", `{}`)},
+		{"single-digit offset hour is not RFC3339", runTraceRow("evt_single_offset_hour", "run_started", "2026-10-05T12:13:14+1:00", `{}`)},
+		{"offset requires colon", runTraceRow("evt_offset_without_colon", "run_started", "2026-10-05T12:13:14+0100", `{}`)},
+		{"offset hour must be below 24", runTraceRow("evt_offset_hour_overflow", "run_started", "2026-10-05T12:13:14+24:00", `{}`)},
+		{"offset minute must be below 60", runTraceRow("evt_offset_minute_overflow", "run_started", "2026-10-05T12:13:14-23:60", `{}`)},
+		{"offset hour and minute ranges are independently checked", runTraceRow("evt_offset_both_overflow", "run_started", "2026-10-05T12:13:14+24:60", `{}`)},
 		{"null trace row", `null`},
@@
 	}
+
+	for _, timestamp := range []string{
+		"2026-10-05T12:13:14Z",
+		"2026-10-05T12:13:14.1+00:00",
+		"2026-10-05T12:13:14.123456789-00:00",
+		"2026-10-05T12:13:14.123456789+23:59",
+		"2026-10-05T12:13:14-23:59",
+	} {
+		t.Run("valid RFC3339 timestamp preserved/"+timestamp, func(t *testing.T) {
+			row := runTraceRow("evt_timestamp_boundary", "run_started", timestamp, `{}`)
+			frame := validRunTraceFrame("evt_timestamp_boundary", "run_started")
+			frame.Timestamp = timestamp
+			want := validRunTraceSnapshot([]ports.RunTraceFrame{frame}, 1)
+			assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
+		})
+	}
 
 	t.Run("duplicate selected id invalidates response", func(t *testing.T) {
```

## Deviations and verification boundary

No material deviation. The timestamp gate rejects comma fractions, fallback
single-width clock/offset fields, malformed zone shapes, and offsets outside
the assigned numeric ranges before calling `time.Parse`. Valid no-fraction,
fractional, zero-offset and ±23:59 inputs are checked for exact source-text
preservation. All-invalid and all-valid expected snapshots retain the
existing safe projection assertions. The scope contains only the two assigned
source files and this new evidence record; no dependencies or runtime
interfaces changed. No compiler, test, scanner, Git, status or debt operation
was run. Fresh independent review is required before any assertion run.
