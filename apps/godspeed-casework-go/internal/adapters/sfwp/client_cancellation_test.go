package sfwp

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"io"
	"net"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

const (
	cancellationRequestTimeout = 5 * time.Second
	cancellationWaitBound      = time.Second
)

type cancellationPeer struct {
	id   int64
	conn net.Conn
}

type cancellationResult struct {
	response *Response
	err      error
}

type cancellationHarness struct {
	client *Client
	peers  chan cancellationPeer
	dials  atomic.Int64

	mu       sync.Mutex
	peerEnds []net.Conn
	workers  sync.WaitGroup
}

func newCancellationHarness(t *testing.T, maxConns int) *cancellationHarness {
	t.Helper()
	h := &cancellationHarness{peers: make(chan cancellationPeer, 8)}
	client, err := New(Config{
		SocketPath:     "cancellation-test",
		MaxConns:       maxConns,
		RequestTimeout: cancellationRequestTimeout,
		BackoffBase:    time.Millisecond,
		Dial: func(context.Context, string) (net.Conn, error) {
			clientSide, peerSide := net.Pipe()
			id := h.dials.Add(1)
			h.mu.Lock()
			h.peerEnds = append(h.peerEnds, peerSide)
			h.mu.Unlock()
			h.peers <- cancellationPeer{id: id, conn: peerSide}
			return clientSide, nil
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	h.client = client
	t.Cleanup(func() {
		client.Close()
		h.mu.Lock()
		for _, peer := range h.peerEnds {
			_ = peer.Close()
		}
		h.mu.Unlock()
		joined := make(chan struct{})
		go func() {
			h.workers.Wait()
			close(joined)
		}()
		select {
		case <-joined:
		case <-time.After(cancellationWaitBound):
			t.Error("owned cancellation test workers did not stop after peer cleanup")
		}
	})
	return h
}

func (h *cancellationHarness) nextPeer(t *testing.T) cancellationPeer {
	t.Helper()
	select {
	case peer := <-h.peers:
		return peer
	case <-time.After(cancellationWaitBound):
		t.Fatal("client did not dial the expected peer")
		return cancellationPeer{}
	}
}

func (h *cancellationHarness) startCall(ctx context.Context, req *Request) <-chan cancellationResult {
	done := make(chan cancellationResult, 1)
	h.workers.Add(1)
	go func() {
		defer h.workers.Done()
		response, err := h.client.Do(ctx, req)
		done <- cancellationResult{response: response, err: err}
	}()
	return done
}

func (h *cancellationHarness) startPeer(peer cancellationPeer, serve func(net.Conn)) {
	h.workers.Add(1)
	go func() {
		defer h.workers.Done()
		defer peer.conn.Close()
		serve(peer.conn)
	}()
}

func readCancellationRequest(conn net.Conn) (string, error) {
	if err := conn.SetReadDeadline(time.Now().Add(4 * time.Second)); err != nil {
		return "", err
	}
	return bufio.NewReader(conn).ReadString('\n')
}

func waitCancellationValue[T any](t *testing.T, ch <-chan T, description string) T {
	t.Helper()
	select {
	case value := <-ch:
		return value
	case <-time.After(cancellationWaitBound):
		t.Fatalf("timed out waiting for %s", description)
		var zero T
		return zero
	}
}

func waitCanceledCall(t *testing.T, peer cancellationPeer, done <-chan cancellationResult) cancellationResult {
	t.Helper()
	select {
	case result := <-done:
		return result
	case <-time.After(cancellationWaitBound):
		// A failing baseline must not leave its blocked Do or peer goroutine alive for the
		// configured five-second request deadline.
		_ = peer.conn.Close()
		select {
		case <-done:
			t.Fatalf("manual cancellation did not interrupt the in-flight request within %s", cancellationWaitBound)
		case <-time.After(cancellationWaitBound):
			t.Fatalf("request remained blocked even after the owned peer was closed")
		}
		return cancellationResult{}
	}
}

func cancellationWire(t *testing.T, line, wantVerb, wantRequestID string) {
	t.Helper()
	var wire struct {
		Verb      string `json:"verb"`
		RequestID string `json:"request_id"`
	}
	if err := json.Unmarshal([]byte(line), &wire); err != nil {
		t.Fatalf("decode request line %q: %v", line, err)
	}
	if wire.Verb != wantVerb || wire.RequestID != wantRequestID {
		t.Fatalf("request verb/id = %q/%q, want %q/%q", wire.Verb, wire.RequestID, wantVerb, wantRequestID)
	}
}

func assertCanceledUnavailable(t *testing.T, err error) {
	t.Helper()
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable || !errors.Is(err, context.Canceled) {
		t.Fatalf("canceled in-flight request error = %v, want unavailable wrapping context.Canceled", err)
	}
}

func assertEmptyCaseList(t *testing.T, result cancellationResult) {
	t.Helper()
	if result.err != nil || result.response == nil {
		t.Fatalf("case_list failed: response=%v err=%v", result.response, result.err)
	}
	var view CaseListView
	if err := result.response.Into(&view); err != nil {
		t.Fatalf("decode case_list response: %v", err)
	}
	if view.Cases == nil || len(view.Cases) != 0 || view.Unreadable == nil || len(view.Unreadable) != 0 {
		t.Fatalf("case_list response = %+v, want empty cases and unreadable arrays", view)
	}
}

func TestManualCancellationInterruptsRunGetAskAndMutationWithoutRetry(t *testing.T) {
	mutation, err := NewCaseCommit("e2e-sentry-chain@0.1.0", nil, "", "", 0, "cancel-commit-unique-71", nil,
		Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	for _, tc := range []struct {
		name      string
		req       *Request
		verb      string
		requestID string
	}{
		{name: "run_get", req: NewRunGet("run-cancel-71"), verb: "run_get"},
		{name: "ask", req: rawAskRequest(), verb: "ask"},
		{name: "correlated mutation", req: mutation, verb: "case_commit", requestID: "cancel-commit-unique-71"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			h := newCancellationHarness(t, 1)
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			done := h.startCall(ctx, tc.req)
			peer := h.nextPeer(t)
			requestLine := make(chan string, 1)
			peerRead := make(chan error, 1)
			h.startPeer(peer, func(conn net.Conn) {
				line, err := readCancellationRequest(conn)
				if err != nil {
					requestLine <- ""
					peerRead <- err
					return
				}
				requestLine <- line
				var b [1]byte
				_, err = conn.Read(b[:])
				peerRead <- err
			})

			line := waitCancellationValue(t, requestLine, "actual request receipt")
			cancellationWire(t, line, tc.verb, tc.requestID)
			cancel()
			result := waitCanceledCall(t, peer, done)
			assertCanceledUnavailable(t, result.err)
			if peerErr := waitCancellationValue(t, peerRead, "owned peer EOF after cancellation"); !errors.Is(peerErr, io.EOF) {
				t.Fatalf("peer observed cancellation as %v, want EOF from closing only its connection", peerErr)
			}
			if got := h.dials.Load(); got != 1 {
				t.Fatalf("canceled %s opened %d connections, want exactly one and no retry/status recovery", tc.verb, got)
			}
		})
	}
}

func TestCancelingOnePoolRequestLeavesConcurrentRequestUsable(t *testing.T) {
	h := newCancellationHarness(t, 2)
	firstCtx, cancelFirst := context.WithCancel(context.Background())
	defer cancelFirst()
	firstDone := h.startCall(firstCtx, NewRunGet("run-cancel-pool-1"))
	firstPeer := h.nextPeer(t)
	firstLine := make(chan string, 1)
	firstEOF := make(chan error, 1)
	h.startPeer(firstPeer, func(conn net.Conn) {
		line, err := readCancellationRequest(conn)
		if err != nil {
			firstLine <- ""
			firstEOF <- err
			return
		}
		firstLine <- line
		var b [1]byte
		_, err = conn.Read(b[:])
		firstEOF <- err
	})
	cancellationWire(t, waitCancellationValue(t, firstLine, "first run_get receipt"), "run_get", "")

	secondDone := h.startCall(context.Background(), NewCaseList())
	secondPeer := h.nextPeer(t)
	secondLine := make(chan string, 1)
	writeSecond := make(chan struct{})
	var releaseSecondOnce sync.Once
	releaseSecond := func() {
		releaseSecondOnce.Do(func() { close(writeSecond) })
	}
	t.Cleanup(releaseSecond)
	secondWriteErr := make(chan error, 1)
	h.startPeer(secondPeer, func(conn net.Conn) {
		line, err := readCancellationRequest(conn)
		if err != nil {
			secondLine <- ""
			secondWriteErr <- err
			return
		}
		secondLine <- line
		<-writeSecond
		_, err = io.WriteString(conn, "{\"cases\":[],\"unreadable\":[]}\n")
		secondWriteErr <- err
		var b [1]byte
		_, _ = conn.Read(b[:])
	})
	cancellationWire(t, waitCancellationValue(t, secondLine, "concurrent case_list receipt"), "case_list", "")
	if firstPeer.id == secondPeer.id {
		t.Fatal("concurrent checked-out requests unexpectedly share one positional connection")
	}

	cancelFirst()
	firstResult := waitCanceledCall(t, firstPeer, firstDone)
	assertCanceledUnavailable(t, firstResult.err)
	if peerErr := waitCancellationValue(t, firstEOF, "first peer EOF"); !errors.Is(peerErr, io.EOF) {
		t.Fatalf("first peer observed %v, want EOF after local request cancellation", peerErr)
	}
	releaseSecond()
	if err := waitCancellationValue(t, secondWriteErr, "second response write"); err != nil {
		t.Fatalf("canceling first request affected second checked-out connection: %v", err)
	}
	secondResult := waitCancellationValue(t, secondDone, "second request completion")
	assertEmptyCaseList(t, secondResult)
	if got := h.dials.Load(); got != 2 {
		t.Fatalf("two concurrent checked-out requests used %d connections, want exactly two", got)
	}
}

func TestCanceledPartialRunGetConnectionIsNotReused(t *testing.T) {
	h := newCancellationHarness(t, 1)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	done := h.startCall(ctx, NewRunGet("run-partial-71"))
	firstPeer := h.nextPeer(t)
	requestLine := make(chan string, 1)
	partialWritten := make(chan error, 1)
	firstEOF := make(chan error, 1)
	h.startPeer(firstPeer, func(conn net.Conn) {
		line, err := readCancellationRequest(conn)
		if err != nil {
			requestLine <- ""
			partialWritten <- err
			return
		}
		requestLine <- line
		_, err = io.WriteString(conn, `{"run_id":`)
		partialWritten <- err
		if err == nil {
			var b [1]byte
			_, err = conn.Read(b[:])
		}
		firstEOF <- err
	})
	cancellationWire(t, waitCancellationValue(t, requestLine, "partial run_get receipt"), "run_get", "")
	if err := waitCancellationValue(t, partialWritten, "partial response bytes"); err != nil {
		t.Fatalf("write partial response: %v", err)
	}
	cancel()
	result := waitCanceledCall(t, firstPeer, done)
	assertCanceledUnavailable(t, result.err)
	if peerErr := waitCancellationValue(t, firstEOF, "partial connection EOF"); !errors.Is(peerErr, io.EOF) {
		t.Fatalf("partial response connection ended with %v, want EOF", peerErr)
	}
	if got := h.dials.Load(); got != 1 {
		t.Fatalf("canceled run_get retried before the later inspect: dials=%d", got)
	}

	laterDone := h.startCall(context.Background(), NewCaseList())
	secondPeer := h.nextPeer(t)
	secondLine := make(chan string, 1)
	h.startPeer(secondPeer, func(conn net.Conn) {
		line, err := readCancellationRequest(conn)
		if err == nil {
			secondLine <- line
			_, err = io.WriteString(conn, "{\"cases\":[],\"unreadable\":[]}\n")
		}
		if err != nil {
			secondLine <- ""
		}
		var b [1]byte
		_, _ = conn.Read(b[:])
	})
	cancellationWire(t, waitCancellationValue(t, secondLine, "fresh case_list receipt"), "case_list", "")
	if secondPeer.id == firstPeer.id || h.dials.Load() != 2 {
		t.Fatalf("later inspect did not use a fresh connection: first=%d second=%d dials=%d", firstPeer.id, secondPeer.id, h.dials.Load())
	}
	laterResult := waitCancellationValue(t, laterDone, "fresh inspect completion")
	assertEmptyCaseList(t, laterResult)
}

func TestCancelAfterCompletedMutationPreservesAndReusesItsConnection(t *testing.T) {
	h := newCancellationHarness(t, 1)
	ctx, cancel := context.WithCancel(context.Background())
	mutation, err := NewCaseCommit("e2e-sentry-chain@0.1.0", nil, "", "", 0, "cancel-completed-unique-72", nil,
		Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	firstLine := make(chan string, 1)
	secondLine := make(chan string, 1)
	peerErr := make(chan error, 1)
	// Constructing the pipe is lazy, so start Do before retrieving its exact peer identity.
	firstDone := h.startCall(ctx, mutation)
	firstPeer := h.nextPeer(t)
	h.startPeer(firstPeer, func(conn net.Conn) {
		line, err := readCancellationRequest(conn)
		if err != nil {
			firstLine <- ""
			peerErr <- err
			return
		}
		firstLine <- line
		if _, err = io.WriteString(conn, "{\"case_id\":\"case-cancel-completed\",\"state\":\"awaiting_activation\",\"exit_code\":0}\n"); err != nil {
			peerErr <- err
			return
		}
		line, err = readCancellationRequest(conn)
		if err != nil {
			secondLine <- ""
			peerErr <- err
			return
		}
		secondLine <- line
		_, err = io.WriteString(conn, "{\"cases\":[],\"unreadable\":[]}\n")
		peerErr <- err
		var b [1]byte
		_, _ = conn.Read(b[:])
	})

	firstLineText := waitCancellationValue(t, firstLine, "completed mutation receipt")
	cancellationWire(t, firstLineText, "case_commit", "cancel-completed-unique-72")
	firstResult := waitCancellationValue(t, firstDone, "successful mutation response")
	if firstResult.err != nil || firstResult.response == nil {
		t.Fatalf("valid completed mutation response was lost: response=%v err=%v", firstResult.response, firstResult.err)
	}
	var commit CommitView
	if err := firstResult.response.Into(&commit); err != nil {
		t.Fatalf("decode completed mutation response: %v", err)
	}
	if commit.CaseID != "case-cancel-completed" {
		t.Fatalf("completed mutation returned case %q, want case-cancel-completed", commit.CaseID)
	}

	// Cancellation is deliberately after Client.Do returned the valid record-writing result.
	cancel()
	secondResult := h.startCall(context.Background(), NewCaseList())
	cancellationWire(t, waitCancellationValue(t, secondLine, "request on completed connection"), "case_list", "")
	if got := h.dials.Load(); got != 1 {
		t.Fatalf("post-completion context cancellation retired/replaced the idle peer: dials=%d", got)
	}
	if err := waitCancellationValue(t, peerErr, "reused connection response"); err != nil {
		t.Fatalf("write reused-connection response: %v", err)
	}
	second := waitCancellationValue(t, secondResult, "case_list on the same connection")
	assertEmptyCaseList(t, second)
}

func TestRunGetCancellationRacesResponseAndPoolReturn(t *testing.T) {
	h := newCancellationHarness(t, 1)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	firstDone := h.startCall(ctx, NewRunGet("run-cancel-race-73"))
	firstPeer := h.nextPeer(t)
	requestLine := make(chan string, 1)
	raceStart := make(chan struct{})
	var releaseRaceStartOnce sync.Once
	releaseRaceStart := func() { releaseRaceStartOnce.Do(func() { close(raceStart) }) }
	t.Cleanup(releaseRaceStart)
	writeResult := make(chan error, 1)
	secondLine := make(chan string, 1)
	peerEOF := make(chan error, 1)
	h.startPeer(firstPeer, func(conn net.Conn) {
		line, err := readCancellationRequest(conn)
		if err != nil {
			requestLine <- ""
			writeResult <- err
			peerEOF <- err
			return
		}
		requestLine <- line
		<-raceStart
		_, err = io.WriteString(conn, "{\"run_id\":\"run-cancel-race-73\"}\n")
		writeResult <- err
		// Write completion and the caller's Do result are separate race outcomes.
		// Read independently to learn whether this peer is reused or retired.
		// The initial request read already armed the peer deadline before raceStart.
		// Read directly here so a retired pipe reports EOF from Read itself instead
		// of SetReadDeadline returning ErrClosedPipe before any read is attempted.
		line, err = bufio.NewReader(conn).ReadString('\n')
		if err != nil {
			peerEOF <- err
			return
		}
		secondLine <- line
		_, err = io.WriteString(conn, "{\"cases\":[],\"unreadable\":[]}\n")
		if err != nil {
			peerEOF <- err
			return
		}
		var b [1]byte
		_, err = conn.Read(b[:])
		peerEOF <- err
	})
	cancellationWire(t, waitCancellationValue(t, requestLine, "raced run_get receipt"), "run_get", "")
	cancelStarted := make(chan struct{})
	h.workers.Add(1)
	go func() {
		defer h.workers.Done()
		<-raceStart
		cancel()
		close(cancelStarted)
	}()
	releaseRaceStart()
	select {
	case <-cancelStarted:
	case <-time.After(cancellationWaitBound):
		t.Fatal("cancellation racer did not complete")
	}
	first := waitCanceledCall(t, firstPeer, firstDone)
	_ = waitCancellationValue(t, writeResult, "racing response write")
	if first.err == nil {
		if first.response == nil {
			t.Fatal("successful response race returned a nil response")
		}
		var run struct {
			RunID string `json:"run_id"`
		}
		if err := first.response.Into(&run); err != nil {
			t.Fatalf("decode valid raced run_get response: %v", err)
		}
		if run.RunID != "run-cancel-race-73" {
			t.Fatalf("raced run_get returned run %q, want run-cancel-race-73", run.RunID)
		}
	} else {
		assertCanceledUnavailable(t, first.err)
	}

	secondDone := h.startCall(context.Background(), NewCaseList())
	type peerDisposition struct {
		peer   cancellationPeer
		reused bool
		line   string
	}
	var next peerDisposition
	select {
	case line := <-secondLine:
		next = peerDisposition{peer: firstPeer, reused: true, line: line}
	case peer := <-h.peers:
		next = peerDisposition{peer: peer}
	case <-time.After(cancellationWaitBound):
		t.Fatal("next request used neither the raced peer nor a fresh peer")
	}
	if next.peer.conn == nil {
		t.Fatal("next request used neither the raced peer nor a fresh peer")
	}
	if next.reused {
		if first.err != nil {
			t.Fatal("canceled Do returned its retired connection to the pool")
		}
		cancellationWire(t, next.line, "case_list", "")
		if got := h.dials.Load(); got != 1 {
			t.Fatalf("same-peer reuse opened %d connections, want exactly one", got)
		}
	} else {
		if next.peer.id == firstPeer.id {
			t.Fatal("fresh peer has the retired race peer's identity")
		}
		if peerErr := waitCancellationValue(t, peerEOF, "retired raced peer EOF"); !errors.Is(peerErr, io.EOF) {
			t.Fatalf("retired race peer ended with %v, want independently observed EOF", peerErr)
		}
		if first.err != nil {
			assertCanceledUnavailable(t, first.err)
		}
		freshLine := make(chan string, 1)
		freshWriteErr := make(chan error, 1)
		h.startPeer(next.peer, func(conn net.Conn) {
			line, err := readCancellationRequest(conn)
			freshLine <- line
			if err == nil {
				_, err = io.WriteString(conn, "{\"cases\":[],\"unreadable\":[]}\n")
			}
			freshWriteErr <- err
			var b [1]byte
			_, _ = conn.Read(b[:])
		})
		cancellationWire(t, waitCancellationValue(t, freshLine, "fresh case_list receipt"), "case_list", "")
		if got := h.dials.Load(); got != 2 {
			t.Fatalf("fresh peer disposition opened %d connections, want exactly two", got)
		}
		if err := waitCancellationValue(t, freshWriteErr, "fresh response write"); err != nil {
			t.Fatalf("write fresh case_list response: %v", err)
		}
	}
	second := waitCanceledCall(t, next.peer, secondDone)
	assertEmptyCaseList(t, second)
}
