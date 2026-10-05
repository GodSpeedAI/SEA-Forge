package sfwp

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"net"
	"os"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// The approved hard ceiling is 32 MiB per NDJSON response line, including its LF delimiter.
const approvedResponseLineLimit = 32 << 20

func responseLineOfSize(size int) []byte {
	prefix := `{"padding":"`
	suffix := `"}` + "\n"
	line := make([]byte, size)
	copy(line, prefix)
	for i := len(prefix); i < size-len(suffix); i++ {
		line[i] = 'x'
	}
	copy(line[size-len(suffix):], suffix)
	return line
}

func subscriptionEventLineOfSize(size int) []byte {
	base := `{"type":"event","event":{"cursor":"1.1","kind":"plan_mutated","case_id":"case_1","detail":{"padding":""},"committed_at":"2026-09-30T00:00:00Z"}}`
	paddingBytes := size - len(base) - 1 // include the LF delimiter in the configured limit
	if paddingBytes < 0 {
		panic("subscription event size is smaller than its JSON envelope")
	}
	line := strings.Replace(base, `"padding":""`, `"padding":"`+strings.Repeat("x", paddingBytes)+`"`, 1) + "\n"
	return []byte(line)
}

func TestResponseLineLimitConfigValidationBeforeDial(t *testing.T) {
	for _, tc := range []struct {
		name string
		cap  int
	}{
		{name: "negative", cap: -1},
		{name: "above hard ceiling", cap: approvedResponseLineLimit + 1},
	} {
		t.Run(tc.name, func(t *testing.T) {
			var dials atomic.Int32
			config := testConfig("unused")
			config.MaxResponseLineBytes = tc.cap
			config.Dial = func(context.Context, string) (net.Conn, error) {
				dials.Add(1)
				return nil, io.EOF
			}
			client, err := New(config)
			if err == nil || apperr.KindOf(err) != apperr.KindConfig {
				if client != nil {
					client.Close()
				}
				t.Fatalf("New with MaxResponseLineBytes=%d returned client=%v error=%v, want typed config error",
					tc.cap, client, err)
			}
			if got := dials.Load(); got != 0 {
				t.Fatalf("invalid config dialed %d times before refusal", got)
			}
		})
	}

	for _, tc := range []struct {
		name string
		cap  int
	}{
		{name: "zero default", cap: 0},
		{name: "exact hard ceiling", cap: approvedResponseLineLimit},
	} {
		t.Run(tc.name, func(t *testing.T) {
			config := testConfig("unused")
			config.MaxResponseLineBytes = tc.cap
			client, err := New(config)
			if err != nil {
				t.Fatalf("New with MaxResponseLineBytes=%d: %v", tc.cap, err)
			}
			client.Close()
		})
	}
}

func TestLowerResponseLineLimitAppliesToRPC(t *testing.T) {
	const lowerLimit = 96
	t.Run("exact configured boundary", func(t *testing.T) {
		line := responseLineOfSize(lowerLimit)
		fs := newFakeServer(t, func(string) (string, bool) { return strings.TrimSuffix(string(line), "\n"), false })
		config := testConfig(fs.socket)
		config.MaxResponseLineBytes = lowerLimit
		client, err := New(config)
		if err != nil {
			t.Fatal(err)
		}
		defer client.Close()
		if _, err := client.Do(context.Background(), NewCaseList()); err != nil {
			t.Fatalf("exact lower-limit RPC response failed: %v", err)
		}
		if got := fs.requestCount("case_list"); got != 1 {
			t.Fatalf("exact-boundary RPC request count=%d, want 1", got)
		}
	})

	t.Run("one byte over is unavailable after the safe inspect retry", func(t *testing.T) {
		line := responseLineOfSize(lowerLimit + 1)
		fs := newFakeServer(t, func(string) (string, bool) { return strings.TrimSuffix(string(line), "\n"), false })
		config := testConfig(fs.socket)
		config.MaxResponseLineBytes = lowerLimit
		client, err := New(config)
		if err != nil {
			t.Fatal(err)
		}
		defer client.Close()
		if _, err := client.Do(context.Background(), NewCaseList()); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("over-limit RPC response error=%v, want typed unavailable", err)
		}
		if got := fs.requestCount("case_list"); got != 2 {
			t.Fatalf("over-limit inspect sent %d requests, want exactly one retry", got)
		}
	})
}

func TestLowerResponseLineLimitAppliesToSubscription(t *testing.T) {
	const lowerLimit = 192
	for _, tc := range []struct {
		name       string
		lineSize   int
		wantEvents int
	}{
		{name: "exact configured boundary", lineSize: lowerLimit, wantEvents: 1},
		{name: "one byte over is rejected before delivery", lineSize: lowerLimit + 1, wantEvents: 0},
	} {
		t.Run(tc.name, func(t *testing.T) {
			line := subscriptionEventLineOfSize(tc.lineSize)
			fs := newFakeServer(t, func(string) (string, bool) { return strings.TrimSuffix(string(line), "\n"), false })
			config := testConfig(fs.socket)
			config.MaxResponseLineBytes = lowerLimit
			config.SubscribeIdle = 100 * time.Millisecond
			client, err := New(config)
			if err != nil {
				t.Fatal(err)
			}
			defer client.Close()

			sub := &Subscription{Events: make(chan Event, 1)}
			_, _ = client.subscriberPass(context.Background(), sub)
			if got := len(sub.Events); got != tc.wantEvents {
				t.Fatalf("subscription delivered %d events at line size %d, want %d",
					got, tc.lineSize, tc.wantEvents)
			}
		})
	}
}

func callWithFragmentedPipe(t *testing.T, response []byte) ([]byte, *conn, error) {
	t.Helper()
	clientSide, authoritySide := net.Pipe()
	t.Cleanup(func() {
		_ = clientSide.Close()
		_ = authoritySide.Close()
	})
	cn := &conn{nc: clientSide, br: bufio.NewReader(clientSide)}
	served := make(chan error, 1)
	go func() {
		defer authoritySide.Close()
		if _, err := bufio.NewReader(authoritySide).ReadBytes('\n'); err != nil {
			served <- err
			return
		}
		for offset := 0; offset < len(response); offset += 64 << 10 {
			end := min(offset+(64<<10), len(response))
			if _, err := authoritySide.Write(response[offset:end]); err != nil {
				served <- err
				return
			}
		}
		served <- nil
	}()

	got, err := cn.call(context.Background(), []byte(`{"verb":"response_limit_test"}`), 30*time.Second)
	select {
	case <-served:
	case <-time.After(30 * time.Second):
		t.Fatal("fragmented response writer did not finish")
	}
	return got, cn, err
}

func TestResponseLineLimitExactBoundaryAndOverflowPoison(t *testing.T) {
	t.Run("fragmented exact limit is accepted", func(t *testing.T) {
		line := responseLineOfSize(approvedResponseLineLimit)
		got, cn, err := callWithFragmentedPipe(t, line)
		if err != nil {
			t.Fatalf("exact-limit response failed: %v", err)
		}
		if len(got) != approvedResponseLineLimit || cn.dead {
			t.Fatalf("exact-limit response length=%d, connection dead=%v", len(got), cn.dead)
		}
		if _, err := DecodeResponse(got); err != nil {
			t.Fatalf("exact-limit JSON response did not decode: %v", err)
		}
	})

	t.Run("one byte over limit is typed unavailable and poisons the connection", func(t *testing.T) {
		line := responseLineOfSize(approvedResponseLineLimit + 1)
		_, cn, err := callWithFragmentedPipe(t, line)
		if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("over-limit response error=%v, want typed unavailable", err)
		}
		if !cn.dead {
			t.Fatal("over-limit response left its partially consumed connection usable")
		}
		if _, err := cn.call(context.Background(), []byte(`{"verb":"must_not_reuse"}`), time.Second); err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
			t.Fatalf("poisoned connection was reused: %v", err)
		}
	})
}

func TestResponseLineLimitTruncatedEOFIsUnavailable(t *testing.T) {
	clientSide, authoritySide := net.Pipe()
	defer clientSide.Close()
	defer authoritySide.Close()
	cn := &conn{nc: clientSide, br: bufio.NewReader(clientSide)}
	go func() {
		defer authoritySide.Close()
		_, _ = bufio.NewReader(authoritySide).ReadBytes('\n')
		_, _ = io.WriteString(authoritySide, `{"partial":`)
	}()

	_, err := cn.call(context.Background(), []byte(`{"verb":"truncated_test"}`), 2*time.Second)
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("truncated response error=%v, want typed unavailable", err)
	}
	if !cn.dead {
		t.Fatal("truncated response left its connection usable")
	}
}

func TestInspectOverLimitResponseRetriesOnceOnFreshConnection(t *testing.T) {
	var caseLists atomic.Int32
	oversized := strings.Repeat("x", approvedResponseLineLimit+1)
	fs := newFakeServer(t, func(line string) (string, bool) {
		var request struct {
			Verb string `json:"verb"`
		}
		if err := json.Unmarshal([]byte(line), &request); err != nil {
			return `{"error":"bad request"}`, false
		}
		if request.Verb != "case_list" {
			return `{"error":"unexpected verb"}`, false
		}
		if caseLists.Add(1) == 1 {
			return oversized, false // Invalid JSON proves the line cap runs before DecodeResponse.
		}
		return `{"cases":[],"unreadable":[]}`, false
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	if _, err := client.Do(context.Background(), NewCaseList()); err != nil {
		t.Fatalf("inspect should succeed after exactly one safe retry: %v", err)
	}
	if got := fs.requestCount("case_list"); got != 2 {
		t.Fatalf("inspect sent %d requests, want one initial call and one retry", got)
	}
	if got := fs.conns.Load(); got != 2 {
		t.Fatalf("inspect used %d connections, want a fresh connection after overflow", got)
	}
}

func TestOverLimitMutationAndRecoveryResponsesNeverResendMutation(t *testing.T) {
	var commits, statuses atomic.Int32
	oversized := strings.Repeat("x", approvedResponseLineLimit+1)
	fs := newFakeServer(t, func(line string) (string, bool) {
		var request struct {
			Verb string `json:"verb"`
		}
		if err := json.Unmarshal([]byte(line), &request); err != nil {
			return `{"error":"bad request"}`, false
		}
		switch request.Verb {
		case "case_commit":
			commits.Add(1)
			return oversized, false
		case "request_get_status":
			if statuses.Add(1) == 1 {
				return oversized, false
			}
			return `{"request_id":"req-line-cap-1","status":"completed","method":"case.commit","outcome":{"case_id":"case_0177","state":"awaiting_activation","exit_code":0}}`, false
		default:
			return `{"error":"unexpected verb"}`, false
		}
	})
	client, err := New(testConfig(fs.socket))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	request, err := NewCaseCommit("e2e-sentry-chain@0.1.0", nil, "", "", 0, "req-line-cap-1", nil,
		Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	response, err := client.Do(context.Background(), request)
	if err != nil {
		t.Fatalf("correlation recovery did not return the recorded mutation outcome: %v", err)
	}
	var outcome CommitView
	if err := response.Into(&outcome); err != nil {
		t.Fatal(err)
	}
	if outcome.CaseID != "case_0177" {
		t.Fatalf("recovered outcome case_id=%q, want case_0177", outcome.CaseID)
	}
	if commits.Load() != 1 || statuses.Load() != 2 {
		t.Fatalf("mutation sends=%d, status reads=%d; want exactly one mutation and one capped status retry", commits.Load(), statuses.Load())
	}
}

func TestSubscribeOverLimitEventIsRejectedBeforeDecode(t *testing.T) {
	eventLine := subscriptionEventLineOfSize(approvedResponseLineLimit + 1)
	decodedEvent, isEvent, err := DecodeEvent(eventLine)
	if err != nil || !isEvent || decodedEvent.Cursor != "1.1" {
		t.Fatalf("oversized fixture is not a valid subscription event: event=%+v isEvent=%v err=%v",
			decodedEvent, isEvent, err)
	}
	clientSide, authoritySide := net.Pipe()
	requestRead := make(chan error, 1)
	writerDone := make(chan error, 1)
	go func() {
		defer authoritySide.Close()
		_, err := bufio.NewReader(authoritySide).ReadBytes('\n')
		requestRead <- err
		if err != nil {
			writerDone <- err
			return
		}
		// Keep the first line as the already-validated oversized event, then offer
		// trailing bytes in the same write so a cap+detection-byte reader must close
		// before the owned writer can finish the entire offered buffer.
		writeBuffer := append(eventLine, bytes.Repeat([]byte{'x'}, 64<<10)...)
		_, err = authoritySide.Write(writeBuffer)
		writerDone <- err
	}()
	t.Cleanup(func() {
		_ = clientSide.Close()
		_ = authoritySide.Close()
	})
	config := testConfig("subscription-over-limit")
	config.Dial = func(context.Context, string) (net.Conn, error) { return clientSide, nil }
	config.SubscribeIdle = 30 * time.Second
	client, err := New(config)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	sub := &Subscription{Events: make(chan Event, 1)}
	type passResult struct {
		dropped bool
		err     error
	}
	passDone := make(chan passResult, 1)
	passCtx, cancelPass := context.WithCancel(context.Background())
	t.Cleanup(cancelPass)
	go func() {
		dropped, err := client.subscriberPass(passCtx, sub)
		passDone <- passResult{dropped: dropped, err: err}
	}()
	select {
	case requestErr := <-requestRead:
		if requestErr != nil {
			t.Fatalf("subscribe handshake read failed: %v", requestErr)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("subscribe handshake did not reach the owned writer")
	}

	var result passResult
	select {
	case event := <-sub.Events:
		cancelPass()
		_ = clientSide.Close()
		t.Fatalf("unbounded subscription reader delivered oversized valid event before rejection: cursor=%q", event.Cursor)
	case result = <-passDone:
	case <-time.After(45 * time.Second):
		t.Fatal("subscription pass neither rejected the oversized line nor delivered its valid event")
	}
	// Delivery happens before subscriberPass returns, so inspect the buffer after selecting its
	// result too; otherwise an unbounded reader can race EOF against the event case above.
	select {
	case event := <-sub.Events:
		t.Fatalf("unbounded subscription reader delivered oversized valid event before rejection: cursor=%q", event.Cursor)
	default:
	}
	if result.dropped {
		t.Fatal("line overflow was misclassified as consumer backpressure")
	}
	err = result.err
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable {
		t.Fatalf("over-limit event error=%v, want typed unavailable", err)
	}
	if errors.Is(err, io.EOF) || errors.Is(err, os.ErrDeadlineExceeded) || errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("subscription ended with EOF/deadline, not line overflow: %v", err)
	}
	if len(sub.Events) != 0 {
		t.Fatalf("over-limit event was decoded/delivered before rejection: %d events", len(sub.Events))
	}
	select {
	case writeErr := <-writerDone:
		if writeErr == nil {
			t.Fatal("owned writer completed the full oversized line; reader did not stop at the configured cap")
		}
	case <-time.After(5 * time.Second):
		t.Fatal("owned writer did not observe the subscription reader closing at the cap")
	}
}
