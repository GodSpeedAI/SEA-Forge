package sfwp

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const (
	runTraceCaseID        = "case_actual"
	runTraceRunID         = "run_actual"
	runTraceItemID        = "item_actual"
	runTraceStamp         = "2026-10-05T12:13:14.123456789Z"
	runTraceRecordPresent = `[{"record":"trace.jsonl","present":true,"bytes":17}]`
	runTracePayloadAbsent = "__missing_payload__"
)

// runTraceFixture owns its listener and every worker it starts. Cleanup closes the client,
// listener, and accepted connection, then joins the sole worker with a hard bound.
type runTraceFixture struct {
	t            *testing.T
	client       *Client
	authority    *Authority
	listener     net.Listener
	connMu       sync.Mutex
	conn         net.Conn
	response     string
	requests     chan string
	workerDone   chan struct{}
	dialAttempts atomic.Int32
	requestCount atomic.Int32
	cleanupOnce  sync.Once
}

func newRunTraceFixture(t *testing.T, response string) *runTraceFixture {
	t.Helper()
	dir, err := os.MkdirTemp("", "sfwp-run-trace-*")
	if err != nil {
		t.Fatal(err)
	}
	listener, err := net.Listen("unix", filepath.Join(dir, "authority.sock"))
	if err != nil {
		os.RemoveAll(dir)
		t.Fatal(err)
	}
	fixture := &runTraceFixture{
		t: t, listener: listener, response: response,
		requests: make(chan string, 1), workerDone: make(chan struct{}),
	}
	go fixture.serveOne()
	config := testConfig(listener.Addr().String())
	config.Dial = func(ctx context.Context, address string) (net.Conn, error) {
		fixture.dialAttempts.Add(1)
		return (&net.Dialer{}).DialContext(ctx, "unix", address)
	}
	fixture.client, err = New(config)
	if err != nil {
		fixture.close()
		os.RemoveAll(dir)
		t.Fatal(err)
	}
	fixture.authority = NewAuthority(fixture.client)
	t.Cleanup(func() {
		fixture.close()
		os.RemoveAll(dir)
	})
	return fixture
}

func (f *runTraceFixture) serveOne() {
	defer close(f.workerDone)
	conn, err := f.listener.Accept()
	if err != nil {
		return
	}
	f.connMu.Lock()
	f.conn = conn
	f.connMu.Unlock()
	defer conn.Close()
	_ = conn.SetReadDeadline(time.Now().Add(4 * time.Second))
	line, err := bufio.NewReader(conn).ReadString('\n')
	if err != nil {
		return
	}
	f.requestCount.Add(1)
	f.requests <- line
	_ = conn.SetWriteDeadline(time.Now().Add(4 * time.Second))
	_, _ = io.WriteString(conn, f.response+"\n")
}

func (f *runTraceFixture) close() {
	f.cleanupOnce.Do(func() {
		if f.client != nil {
			f.client.Close()
		}
		_ = f.listener.Close()
		f.connMu.Lock()
		if f.conn != nil {
			_ = f.conn.Close()
		}
		f.connMu.Unlock()
		select {
		case <-f.workerDone:
		case <-time.After(5 * time.Second):
			f.t.Errorf("run trace fixture worker did not stop within five seconds")
		}
	})
}

func (f *runTraceFixture) assertSingleRunGet(t *testing.T, runID string) {
	t.Helper()
	if got := f.requestCount.Load(); got != 1 {
		t.Fatalf("logical read produced %d request lines, want exactly one run_get", got)
	}
	select {
	case line := <-f.requests:
		var request map[string]json.RawMessage
		if err := json.Unmarshal([]byte(line), &request); err != nil {
			t.Fatalf("decode observed request %q: %v", line, err)
		}
		var verb, requestedRunID string
		if err := json.Unmarshal(request["verb"], &verb); err != nil {
			t.Fatalf("decode request verb: %v", err)
		}
		if err := json.Unmarshal(request["run_id"], &requestedRunID); err != nil {
			t.Fatalf("decode request run_id: %v", err)
		}
		if verb != "run_get" || requestedRunID != runID || len(request) != 2 {
			t.Fatalf("wire request = %s, want exactly {verb:run_get, run_id:%q}", line, runID)
		}
	default:
		t.Fatal("the counted run_get request was not captured by the owned peer")
	}
}

func runTraceResponse(trace, records, identity, execution, settlement string) string {
	if identity == "" {
		identity = fmt.Sprintf(`"run_id":%q,"case_id":%q,"plan_item_id":%q`, runTraceRunID, runTraceCaseID, runTraceItemID)
	}
	if execution == "" {
		execution = `"completed"`
	}
	if settlement == "" {
		settlement = `"accepted"`
	}
	if records == "" {
		records = runTraceRecordPresent
	}
	if trace == "__missing__" {
		return fmt.Sprintf(`{%s,"execution":%s,"settlement":%s,"records":%s}`, identity, execution, settlement, records)
	}
	if records == "__missing__" {
		return fmt.Sprintf(`{%s,"execution":%s,"settlement":%s,"trace":%s}`, identity, execution, settlement, trace)
	}
	return fmt.Sprintf(`{%s,"execution":%s,"settlement":%s,"trace":%s,"records":%s}`, identity, execution, settlement, trace, records)
}

// runTraceResponseWithoutStanding constructs valid JSON with one required standing
// member absent. Unlike runTraceResponse's empty-string defaults, this helper
// never fills in the omitted execution or settlement member.
func runTraceResponseWithoutStanding(missing string) string {
	fields := []string{
		fmt.Sprintf(`"run_id":%q`, runTraceRunID),
		fmt.Sprintf(`"case_id":%q`, runTraceCaseID),
		fmt.Sprintf(`"plan_item_id":%q`, runTraceItemID),
	}
	if missing != "execution" {
		fields = append(fields, `"execution":"completed"`)
	}
	if missing != "settlement" {
		fields = append(fields, `"settlement":"accepted"`)
	}
	fields = append(fields, `"trace":[]`, `"records":`+runTraceRecordPresent)
	return `{` + strings.Join(fields, ",") + `}`
}

func runTraceIdentityWithout(missingField string) string {
	fields := []struct{ name, value string }{
		{"run_id", runTraceRunID},
		{"case_id", runTraceCaseID},
		{"plan_item_id", runTraceItemID},
	}
	parts := make([]string, 0, len(fields)-1)
	for _, field := range fields {
		if field.name == missingField {
			continue
		}
		parts = append(parts, fmt.Sprintf(`%q:%q`, field.name, field.value))
	}
	return strings.Join(parts, ",")
}

func runTraceRow(eventID, kind, timestamp, payload string) string {
	if timestamp == "" {
		timestamp = runTraceStamp
	}
	if payload == "" {
		payload = `{}`
	}
	return fmt.Sprintf(`{"event_id":%q,"kind":%q,"timestamp":%q,"payload":%s}`, eventID, kind, timestamp, payload)
}

func runTraceRowWithoutPayload(eventID, kind, timestamp string) string {
	if timestamp == "" {
		timestamp = runTraceStamp
	}
	return fmt.Sprintf(`{"event_id":%q,"kind":%q,"timestamp":%q}`, eventID, kind, timestamp)
}

func assertRunTraceCase(t *testing.T, response string, wantKind apperr.Kind, want ports.RunTraceSnapshot, caseID, runID, itemID string) {
	t.Helper()
	fixture := newRunTraceFixture(t, response)
	got, err := fixture.authority.ReadRunTrace(context.Background(), caseID, runID, itemID)
	if err != nil && !reflect.DeepEqual(got, ports.RunTraceSnapshot{}) {
		t.Fatalf("failed run trace returned safe data: %#v", got)
	}
	if wantKind == "" {
		if err != nil {
			t.Fatalf("valid returned run trace was unavailable: %v", err)
		}
		if !reflect.DeepEqual(got, want) {
			t.Fatalf("safe returned run trace = %#v, want %#v", got, want)
		}
	} else {
		if err == nil || apperr.KindOf(err) != wantKind {
			t.Fatalf("run trace error = %v, want typed %s", err, wantKind)
		}
		if !reflect.DeepEqual(got, want) {
			t.Fatalf("failed run trace snapshot = %#v, want empty snapshot %#v", got, want)
		}
	}
	fixture.assertSingleRunGet(t, runID)
}

func validRunTraceFrame(eventID, kind string) ports.RunTraceFrame {
	return ports.RunTraceFrame{EventID: eventID, Kind: kind, Timestamp: runTraceStamp}
}

func validRunTraceSnapshot(frames []ports.RunTraceFrame, total int) ports.RunTraceSnapshot {
	return ports.RunTraceSnapshot{
		RunID: runTraceRunID, CaseID: runTraceCaseID, PlanItemID: runTraceItemID,
		Execution: "completed", Settlement: "accepted", Frames: frames, TotalFrameCount: total,
	}
}

func TestReadRunTraceV3IdentityAndStandingMatrix(t *testing.T) {
	t.Run("exact ownership and request identity", func(t *testing.T) {
		row := runTraceRow("evt_actual", "run_started", "", "{}")
		want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_actual", "run_started")}, 1)
		assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
	})
	t.Run("source order and original timestamp text", func(t *testing.T) {
		late := "2026-10-05T12:13:15.987654321+00:00"
		first := runTraceRow("evt_second_time", "run_finished", late, `{}`)
		second := runTraceRow("evt_first_time", "run_started", runTraceStamp, `{}`)
		want := validRunTraceSnapshot([]ports.RunTraceFrame{
			{EventID: "evt_second_time", Kind: "run_finished", Timestamp: late},
			{EventID: "evt_first_time", Kind: "run_started", Timestamp: runTraceStamp},
		}, 2)
		assertRunTraceCase(t, runTraceResponse("["+first+","+second+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
	})

	for _, field := range []string{"run_id", "case_id", "plan_item_id"} {
		for _, expected := range []string{"", " \t", "foreign_identity"} {
			t.Run(field+"/"+fmt.Sprintf("%q", expected), func(t *testing.T) {
				identity := fmt.Sprintf(`"run_id":%q,"case_id":%q,"plan_item_id":%q`, runTraceRunID, runTraceCaseID, runTraceItemID)
				parts := map[string]string{"run_id": runTraceRunID, "case_id": runTraceCaseID, "plan_item_id": runTraceItemID}
				parts[field] = expected
				identity = fmt.Sprintf(`"run_id":%q,"case_id":%q,"plan_item_id":%q`, parts["run_id"], parts["case_id"], parts["plan_item_id"])
				response := runTraceResponse("[]", "", identity, "", "")
				assertRunTraceCase(t, response, apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
			})
		}
		t.Run("missing response "+field, func(t *testing.T) {
			identity := runTraceIdentityWithout(field)
			assertRunTraceCase(t, runTraceResponse("[]", "", identity, "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	for _, expected := range []string{"foreign_case"} {
		t.Run("expected case identity/"+fmt.Sprintf("%q", expected), func(t *testing.T) {
			assertRunTraceCase(t, runTraceResponse("[]", "", "", "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, expected, runTraceRunID, runTraceItemID)
		})
	}
	for _, expected := range []string{"foreign_run"} {
		t.Run("expected run identity/"+fmt.Sprintf("%q", expected), func(t *testing.T) {
			assertRunTraceCase(t, runTraceResponse("[]", "", "", "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, expected, runTraceItemID)
		})
	}
	for _, expected := range []string{"foreign_item"} {
		t.Run("expected item identity/"+fmt.Sprintf("%q", expected), func(t *testing.T) {
			assertRunTraceCase(t, runTraceResponse("[]", "", "", "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, expected)
		})
	}

	for _, execution := range []string{"pending", "enabled", "active", "completed", "failed", "terminated"} {
		for _, settlement := range []string{"unsettled", "accepted", "rejected", "escalated"} {
			t.Run(execution+"/"+settlement, func(t *testing.T) {
				row := runTraceRow("evt_standing", "run_started", "", "{}")
				want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_standing", "run_started")}, 1)
				want.Execution, want.Settlement = execution, settlement
				assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", fmt.Sprintf("%q", execution), fmt.Sprintf("%q", settlement)), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
			})
		}
	}

	for _, tc := range []struct {
		name, execution, settlement string
	}{
		{"unknown execution", `"mystery"`, `"accepted"`},
		{"null execution", `null`, `"accepted"`},
		{"wrong execution shape", `[]`, `"accepted"`},
		{"unknown settlement", `"completed"`, `"mystery"`},
		{"null settlement", `"completed"`, `null`},
		{"wrong settlement shape", `"completed"`, `[]`},
	} {
		t.Run("malformed standing/"+tc.name, func(t *testing.T) {
			assertRunTraceCase(t, runTraceResponse("[]", "", "", tc.execution, tc.settlement), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}
	for _, missing := range []string{"execution", "settlement"} {
		t.Run("absent "+missing, func(t *testing.T) {
			assertRunTraceCase(t, runTraceResponseWithoutStanding(missing), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}
}

func TestReadRunTraceV3AllowlistIdentityAndRetention(t *testing.T) {
	kinds := []string{"run_started", "item_activated", "command_started", "command_finished", "item_completed", "item_failed", "item_terminated", "human_task_completed", "run_halted", "run_finished"}
	for _, kind := range kinds {
		t.Run("safe kind/"+kind, func(t *testing.T) {
			row := runTraceRow("evt_"+kind, kind, "", `{}`)
			want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_"+kind, kind)}, 1)
			assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	t.Run("unknown kind malformed fields and selected id collision are omitted", func(t *testing.T) {
		unknown := `{"event_id":"evt_selected","kind":"future_private_kind","timestamp":false,"payload":{"actor":"SYNTHETIC_PRIVATE_MARKER"}}`
		selected := runTraceRow("evt_selected", "run_started", "", `{}`)
		want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_selected", "run_started")}, 1)
		assertRunTraceCase(t, runTraceResponse("["+unknown+","+selected+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
	})
	for _, tc := range []struct{ name, eventIDField string }{
		{"missing", ""},
		{"null", `"event_id":null`},
		{"wrong-type", `"event_id":17`},
	} {
		t.Run("unknown kind "+tc.name+" event id is ignored", func(t *testing.T) {
			rowPrefix := ""
			if tc.eventIDField != "" {
				rowPrefix = tc.eventIDField + ","
			}
			unknown := fmt.Sprintf(`{%s"kind":"future_private_kind","timestamp":false,"payload":{"actor":"SYNTHETIC_PRIVATE_MARKER"}}`, rowPrefix)
			selected := runTraceRow("evt_selected", "run_started", "", `{}`)
			want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_selected", "run_started")}, 1)
			assertRunTraceCase(t, runTraceResponse("["+unknown+","+selected+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	for _, tc := range []struct{ name, row string }{
		{"missing selected event kind", `{"event_id":"evt_missing_kind","timestamp":"` + runTraceStamp + `"}`},
		{"null selected event kind", `{"event_id":"evt_null_kind","kind":null,"timestamp":"` + runTraceStamp + `"}`},
		{"numeric selected event kind", `{"event_id":"evt_numeric_kind","kind":17,"timestamp":"` + runTraceStamp + `"}`},
		{"boolean selected event kind", `{"event_id":"evt_boolean_kind","kind":true,"timestamp":"` + runTraceStamp + `"}`},
		{"blank selected event id", runTraceRow(" \t", "run_started", "", `{}`)},
		{"missing selected event id", `{"kind":"run_started","timestamp":"` + runTraceStamp + `"}`},
		{"null selected event id", `{"event_id":null,"kind":"run_started","timestamp":"` + runTraceStamp + `"}`},
		{"wrong-type selected event id", `{"event_id":17,"kind":"run_started","timestamp":"` + runTraceStamp + `"}`},
		{"blank selected timestamp", runTraceRow("evt_blank_time", "run_started", " ", `{}`)},
		{"missing selected timestamp", `{"event_id":"evt_missing_time","kind":"run_started"}`},
		{"null selected timestamp", `{"event_id":"evt_null_time","kind":"run_started","timestamp":null}`},
		{"wrong-type selected timestamp", `{"event_id":"evt_wrong_time","kind":"run_started","timestamp":17}`},
		{"malformed selected timestamp", runTraceRow("evt_bad_time", "run_started", "yesterday", `{}`)},
		{"comma fractional timestamp is not RFC3339", runTraceRow("evt_comma_time", "run_started", "2026-10-05T12:13:14,123Z", `{}`)},
		{"single-digit hour is not RFC3339", runTraceRow("evt_single_hour", "run_started", "2026-10-05T1:13:14Z", `{}`)},
		{"single-digit minute is not RFC3339", runTraceRow("evt_single_minute", "run_started", "2026-10-05T12:3:14Z", `{}`)},
		{"single-digit second is not RFC3339", runTraceRow("evt_single_second", "run_started", "2026-10-05T12:13:4Z", `{}`)},
		{"fraction requires digits", runTraceRow("evt_empty_fraction", "run_started", "2026-10-05T12:13:14.Z", `{}`)},
		{"single-digit offset hour is not RFC3339", runTraceRow("evt_single_offset_hour", "run_started", "2026-10-05T12:13:14+1:00", `{}`)},
		{"offset requires colon", runTraceRow("evt_offset_without_colon", "run_started", "2026-10-05T12:13:14+0100", `{}`)},
		{"offset hour must be below 24", runTraceRow("evt_offset_hour_overflow", "run_started", "2026-10-05T12:13:14+24:00", `{}`)},
		{"offset minute must be below 60", runTraceRow("evt_offset_minute_overflow", "run_started", "2026-10-05T12:13:14-23:60", `{}`)},
		{"offset hour and minute ranges are independently checked", runTraceRow("evt_offset_both_overflow", "run_started", "2026-10-05T12:13:14+24:60", `{}`)},
		{"null trace row", `null`},
		{"scalar trace row", `"not an object"`},
	} {
		t.Run(tc.name, func(t *testing.T) {
			assertRunTraceCase(t, runTraceResponse("["+tc.row+"]", "", "", "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	for _, timestamp := range []string{
		"2026-10-05T12:13:14Z",
		"2026-10-05T12:13:14.1+00:00",
		"2026-10-05T12:13:14.123456789-00:00",
		"2026-10-05T12:13:14.123456789+23:59",
		"2026-10-05T12:13:14-23:59",
	} {
		t.Run("valid RFC3339 timestamp preserved/"+timestamp, func(t *testing.T) {
			row := runTraceRow("evt_timestamp_boundary", "run_started", timestamp, `{}`)
			frame := validRunTraceFrame("evt_timestamp_boundary", "run_started")
			frame.Timestamp = timestamp
			want := validRunTraceSnapshot([]ports.RunTraceFrame{frame}, 1)
			assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	t.Run("duplicate selected id invalidates response", func(t *testing.T) {
		first := runTraceRow("evt_duplicate", "run_started", "", `{}`)
		second := runTraceRow("evt_duplicate", "run_finished", "", `{}`)
		assertRunTraceCase(t, runTraceResponse("["+first+","+second+"]", "", "", "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
	})

	t.Run("selected duplicate beyond retained newest 1024 is invalid", func(t *testing.T) {
		rows := make([]string, 1025)
		for i := range rows {
			id := fmt.Sprintf("evt_%04d", i)
			if i == 1024 {
				id = "evt_0000"
			}
			rows[i] = runTraceRow(id, "run_started", "", `{}`)
		}
		assertRunTraceCase(t, runTraceResponse("["+strings.Join(rows, ",")+"]", "", "", "", ""), apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
	})

	for _, count := range []int{0, 1024, 1027} {
		t.Run(fmt.Sprintf("returned row count %d and newest retention", count), func(t *testing.T) {
			rows := make([]string, count)
			for i := range rows {
				rows[i] = runTraceRow(fmt.Sprintf("evt_%04d", i), "run_started", "", `{}`)
			}
			retainedStart := count - 1024
			if retainedStart < 0 {
				retainedStart = 0
			}
			var frames []ports.RunTraceFrame
			if count > 0 {
				frames = make([]ports.RunTraceFrame, 0, count-retainedStart)
			}
			for i := retainedStart; i < count; i++ {
				frames = append(frames, validRunTraceFrame(fmt.Sprintf("evt_%04d", i), "run_started"))
			}
			want := validRunTraceSnapshot(frames, count)
			assertRunTraceCase(t, runTraceResponse("["+strings.Join(rows, ",")+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
			if count > 1024 {
				fixture := newRunTraceFixture(t, runTraceResponse("["+strings.Join(rows, ",")+"]", "", "", "", ""))
				got, err := fixture.authority.ReadRunTrace(context.Background(), runTraceCaseID, runTraceRunID, runTraceItemID)
				if err != nil {
					t.Fatalf("read bounded retained frames: %v", err)
				}
				if len(got.Frames) != 1024 || cap(got.Frames) > 1024 {
					t.Fatalf("retained frames length/capacity = %d/%d, want 1024 and capacity at most 1024", len(got.Frames), cap(got.Frames))
				}
			}
		})
	}
}

func TestReadRunTraceV3TracePresenceAndReturnedRowBoundary(t *testing.T) {
	for _, tc := range []struct {
		name, trace, records string
		wantKind             apperr.Kind
		count                int
	}{
		{"empty returned rows zero bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":0}]`, "", 0},
		{"empty returned rows positive bytes", `[]`, runTraceRecordPresent, "", 0},
		{"nonempty returned rows zero bytes", `[{"event_id":"evt_prefix","kind":"run_started","timestamp":"` + runTraceStamp + `"}]`, `[{"record":"trace.jsonl","present":true,"bytes":0}]`, "", 1},
		{"nonempty returned prefix positive bytes", `[{"event_id":"evt_prefix","kind":"run_started","timestamp":"` + runTraceStamp + `"}]`, runTraceRecordPresent, "", 1},
		{"missing trace", `__missing__`, runTraceRecordPresent, apperr.KindUnavailable, 0},
		{"null trace", `null`, runTraceRecordPresent, apperr.KindUnavailable, 0},
		{"wrong trace shape", `{}`, runTraceRecordPresent, apperr.KindUnavailable, 0},
		{"missing records", `[]`, `__missing__`, apperr.KindUnavailable, 0},
		{"null records", `[]`, `null`, apperr.KindUnavailable, 0},
		{"wrong records shape", `[]`, `{}`, apperr.KindUnavailable, 0},
		{"null trace record array entry", `[]`, `[null]`, apperr.KindUnavailable, 0},
		{"scalar trace record array entry", `[]`, `[17]`, apperr.KindUnavailable, 0},
		{"missing trace record name", `[]`, `[{"present":true,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"null trace record name", `[]`, `[{"record":null,"present":true,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"missing trace record", `[]`, `[]`, apperr.KindUnavailable, 0},
		{"duplicate trace records", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":0},{"record":"trace.jsonl","present":true,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"absent trace record", `[]`, `[{"record":"other.jsonl","present":true,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"false trace presence", `[]`, `[{"record":"trace.jsonl","present":false,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"missing trace presence", `[]`, `[{"record":"trace.jsonl","bytes":0}]`, apperr.KindUnavailable, 0},
		{"null trace presence", `[]`, `[{"record":"trace.jsonl","present":null,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"wrong-type trace presence", `[]`, `[{"record":"trace.jsonl","present":"true","bytes":0}]`, apperr.KindUnavailable, 0},
		{"missing bytes", `[]`, `[{"record":"trace.jsonl","present":true}]`, apperr.KindUnavailable, 0},
		{"null bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":null}]`, apperr.KindUnavailable, 0},
		{"negative bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":-1}]`, apperr.KindUnavailable, 0},
		{"fractional bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":0.5}]`, apperr.KindUnavailable, 0},
		{"boolean bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":true}]`, apperr.KindUnavailable, 0},
		{"string bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":"0"}]`, apperr.KindUnavailable, 0},
		{"malformed record entry", `[]`, `[{"record":17,"present":true,"bytes":0}]`, apperr.KindUnavailable, 0},
		{"exact u64 maximum bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":18446744073709551615}]`, "", 0},
		{"u64 overflow bytes", `[]`, `[{"record":"trace.jsonl","present":true,"bytes":18446744073709551616}]`, apperr.KindUnavailable, 0},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var frames []ports.RunTraceFrame
			if tc.count == 1 {
				frames = []ports.RunTraceFrame{validRunTraceFrame("evt_prefix", "run_started")}
			}
			want := validRunTraceSnapshot(frames, tc.count)
			if tc.wantKind != "" {
				want = ports.RunTraceSnapshot{}
			}
			assertRunTraceCase(t, runTraceResponse(tc.trace, tc.records, "", "", ""), tc.wantKind, want, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	t.Run("returned prefix count is not a source journal total", func(t *testing.T) {
		row := runTraceRow("evt_valid_prefix", "run_started", "", `{}`)
		want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_valid_prefix", "run_started")}, 1)
		assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
		// The v3 source returns no parse-completeness signal: this only proves one safe returned row.
		// The actual assertion deliberately makes no claim about malformed suffixes, read failures,
		// over-cap source journals, or source-row totals.
	})
}

func TestReadRunTraceV3RejectsRawInvalidJSON(t *testing.T) {
	for _, response := range []struct{ name, body string }{
		{"invalid JSON text", "not-json"},
		{"null JSON root", `null`},
		{"array JSON root", `[]`},
		{"string JSON root", `"not an object"`},
		{"number JSON root", `17`},
	} {
		t.Run(response.name, func(t *testing.T) {
			assertRunTraceCase(t, response.body, apperr.KindUnavailable, ports.RunTraceSnapshot{}, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}
}

func TestReadRunTraceV3CommandMetadataProjection(t *testing.T) {
	for _, tc := range []struct {
		name, payload string
		status        *string
		exit          *int64
		wantKind      apperr.Kind
	}{
		{"missing payload", runTracePayloadAbsent, nil, nil, ""},
		{"null payload", `null`, nil, nil, ""},
		{"scalar payload", `"ignored scalar"`, nil, nil, ""},
		{"array payload", `[]`, nil, nil, ""},
		{"object missing execution", `{}`, nil, nil, ""},
		{"null execution", `{"execution":null}`, nil, nil, ""},
		{"execution missing optionals", `{"execution":{}}`, nil, nil, ""},
		{"null status and exit", `{"execution":{"status":null,"exit_code":null}}`, nil, nil, ""},
		{"status completed", `{"execution":{"status":"completed"}}`, stringPointer("completed"), nil, ""},
		{"status spawn failed", `{"execution":{"status":"spawn_failed"}}`, stringPointer("spawn_failed"), nil, ""},
		{"status timed out", `{"execution":{"status":"timed_out"}}`, stringPointer("timed_out"), nil, ""},
		{"status sandbox violation", `{"execution":{"status":"sandbox_violation"}}`, stringPointer("sandbox_violation"), nil, ""},
		{"status suspected sandbox violation", `{"execution":{"status":"suspected_sandbox_violation"}}`, stringPointer("suspected_sandbox_violation"), nil, ""},
		{"exit only", `{"execution":{"exit_code":0}}`, nil, int64Pointer(0), ""},
		{"safe lower exit boundary", `{"execution":{"exit_code":-9007199254740991}}`, nil, int64Pointer(-9007199254740991), ""},
		{"safe upper exit boundary", `{"execution":{"exit_code":9007199254740991}}`, nil, int64Pointer(9007199254740991), ""},
		{"execution wrong shape", `{"execution":[]}`, nil, nil, apperr.KindUnavailable},
		{"unknown status", `{"execution":{"status":"unknown"}}`, nil, nil, apperr.KindUnavailable},
		{"nonstring status", `{"execution":{"status":7}}`, nil, nil, apperr.KindUnavailable},
		{"unsafe adjacent upper", `{"execution":{"exit_code":9007199254740992}}`, nil, nil, apperr.KindUnavailable},
		{"unsafe adjacent lower", `{"execution":{"exit_code":-9007199254740992}}`, nil, nil, apperr.KindUnavailable},
		{"int64 positive overflow", `{"execution":{"exit_code":9223372036854775808}}`, nil, nil, apperr.KindUnavailable},
		{"int64 negative overflow", `{"execution":{"exit_code":-9223372036854775809}}`, nil, nil, apperr.KindUnavailable},
		{"fraction exit", `{"execution":{"exit_code":1.25}}`, nil, nil, apperr.KindUnavailable},
		{"string exit", `{"execution":{"exit_code":"1"}}`, nil, nil, apperr.KindUnavailable},
		{"boolean exit", `{"execution":{"exit_code":true}}`, nil, nil, apperr.KindUnavailable},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var row string
			if tc.payload == runTracePayloadAbsent {
				row = runTraceRowWithoutPayload("evt_command", "command_finished", "")
			} else {
				row = runTraceRow("evt_command", "command_finished", "", tc.payload)
			}
			frame := validRunTraceFrame("evt_command", "command_finished")
			frame.ExecutionStatus, frame.ExitCode = tc.status, tc.exit
			want := validRunTraceSnapshot([]ports.RunTraceFrame{frame}, 1)
			if tc.wantKind != "" {
				want = ports.RunTraceSnapshot{}
			}
			assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), tc.wantKind, want, runTraceCaseID, runTraceRunID, runTraceItemID)
		})
	}

	t.Run("other kinds ignore all payload metadata", func(t *testing.T) {
		row := runTraceRow("evt_started", "run_started", "", `{"execution":[],"actor":"SYNTHETIC_PRIVATE_MARKER"}`)
		want := validRunTraceSnapshot([]ports.RunTraceFrame{validRunTraceFrame("evt_started", "run_started")}, 1)
		assertRunTraceCase(t, runTraceResponse("["+row+"]", "", "", "", ""), "", want, runTraceCaseID, runTraceRunID, runTraceItemID)
	})

	t.Run("semantic serialization contains only safe fields", func(t *testing.T) {
		row := runTraceRow("evt_safe", "command_finished", "", `{"execution":{"status":"completed","exit_code":7},"actor":"SYNTHETIC_PRIVATE_MARKER","arguments":"SYNTHETIC_ARGS","stdout":"SYNTHETIC_OUTPUT","environment":"SYNTHETIC_ENV","evidence":"SYNTHETIC_EVIDENCE"}`)
		status, exit := "completed", int64(7)
		frame := validRunTraceFrame("evt_safe", "command_finished")
		frame.ExecutionStatus, frame.ExitCode = &status, &exit
		want := validRunTraceSnapshot([]ports.RunTraceFrame{frame}, 1)
		fixture := newRunTraceFixture(t, runTraceResponse("["+row+"]", "", "", "", ""))
		got, err := fixture.authority.ReadRunTrace(context.Background(), runTraceCaseID, runTraceRunID, runTraceItemID)
		if err != nil {
			if !reflect.DeepEqual(got, ports.RunTraceSnapshot{}) {
				t.Fatalf("failed run trace returned safe data: %#v", got)
			}
			t.Fatalf("valid safe projection was unavailable: %v", err)
		}
		if !reflect.DeepEqual(got, want) {
			t.Fatalf("semantic snapshot = %#v, want %#v", got, want)
		}
		encoded, err := json.Marshal(got)
		if err != nil {
			t.Fatalf("marshal semantic snapshot: %v", err)
		}
		for _, marker := range []string{"SYNTHETIC_PRIVATE_MARKER", "SYNTHETIC_ARGS", "SYNTHETIC_OUTPUT", "SYNTHETIC_ENV", "SYNTHETIC_EVIDENCE", "payload", "actor"} {
			if strings.Contains(string(encoded), marker) {
				t.Errorf("semantic serialization leaked ignored marker %q: %s", marker, encoded)
			}
		}
		// This semantic serialization check is not the outgoing canonical wire representation.
		fixture.assertSingleRunGet(t, runTraceRunID)
	})
}

func stringPointer(value string) *string { return &value }
func int64Pointer(value int64) *int64    { return &value }

func TestReadRunTraceV3RejectsBlankExpectedIdentityBeforeDial(t *testing.T) {
	for _, tc := range []struct {
		name, caseID, runID, itemID string
	}{
		{"empty case", "", runTraceRunID, runTraceItemID},
		{"blank case", " \n", runTraceRunID, runTraceItemID},
		{"empty run", runTraceCaseID, "", runTraceItemID},
		{"blank run", runTraceCaseID, "\t", runTraceItemID},
		{"empty item", runTraceCaseID, runTraceRunID, ""},
		{"blank item", runTraceCaseID, runTraceRunID, " "},
	} {
		t.Run(tc.name, func(t *testing.T) {
			fixture := newRunTraceFixture(t, runTraceResponse(`[]`, "", "", "", ""))
			got, err := fixture.authority.ReadRunTrace(context.Background(), tc.caseID, tc.runID, tc.itemID)
			if err == nil || apperr.KindOf(err) != apperr.KindInvalid {
				t.Fatalf("blank expected identity error = %v, want typed invalid", err)
			}
			if !reflect.DeepEqual(got, ports.RunTraceSnapshot{}) {
				t.Fatalf("invalid expected identity returned safe data: %#v", got)
			}
			if got := fixture.dialAttempts.Load(); got != 0 {
				t.Fatalf("blank expected identity caused %d dial attempts, want zero", got)
			}
			if got := fixture.requestCount.Load(); got != 0 {
				t.Fatalf("blank expected identity caused %d wire requests, want zero", got)
			}
		})
	}
}

func TestReadRunTraceV3PreservesAuthorityRefusalClass(t *testing.T) {
	fixture := newRunTraceFixture(t, `{"error":"run trace refused","error_class":"identity_required","no_side_effect":true}`)
	got, err := fixture.authority.ReadRunTrace(context.Background(), runTraceCaseID, runTraceRunID, runTraceItemID)
	if err == nil || apperr.KindOf(err) != apperr.KindAuthorityDenied {
		t.Fatalf("authority refusal error = %v, want typed authority_denied", err)
	}
	if !reflect.DeepEqual(got, ports.RunTraceSnapshot{}) {
		t.Fatalf("authority refusal returned safe data: %#v", got)
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) || refusal.RefusalClass() != "identity_required" {
		t.Fatalf("authority refusal class was not preserved: %v", err)
	}
	fixture.assertSingleRunGet(t, runTraceRunID)
}

func TestReadRunTraceV3PreservesInternalAuthorityRefusal(t *testing.T) {
	fixture := newRunTraceFixture(t, `{"error":"authority could not process request","error_class":"internal_failure","no_side_effect":true}`)
	got, err := fixture.authority.ReadRunTrace(context.Background(), runTraceCaseID, runTraceRunID, runTraceItemID)
	if err == nil || apperr.KindOf(err) != apperr.KindInternal {
		t.Fatalf("authority refusal error = %v, want typed internal", err)
	}
	if !reflect.DeepEqual(got, ports.RunTraceSnapshot{}) {
		t.Fatalf("authority refusal returned safe data: %#v", got)
	}
	var refusal *Refusal
	if !errors.As(err, &refusal) || refusal.RefusalClass() != "internal_failure" {
		t.Fatalf("internal authority refusal class was not preserved: %v", err)
	}
	fixture.assertSingleRunGet(t, runTraceRunID)
}
