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
)

type recordingRunGetAdmission struct {
	mu        sync.Mutex
	ids       []string
	events    []string
	onRelease func()
}

type recordingRunGetPermit struct{ owner *recordingRunGetAdmission }

func (p recordingRunGetPermit) record(event string) {
	p.owner.mu.Lock()
	p.owner.events = append(p.owner.events, event)
	p.owner.mu.Unlock()
}
func (p recordingRunGetPermit) WriteAttemptStarted()  { p.record("permit-start") }
func (p recordingRunGetPermit) WriteAttemptFinished() { p.record("permit-finish") }
func (p recordingRunGetPermit) Release() {
	p.record("permit-release")
	if p.owner.onRelease != nil {
		p.owner.onRelease()
	}
}

func (a *recordingRunGetAdmission) AcquireRunGet(_ context.Context, runID string) (RunGetPermit, error) {
	a.mu.Lock()
	defer a.mu.Unlock()
	a.ids = append(a.ids, runID)
	return recordingRunGetPermit{owner: a}, nil
}

func (a *recordingRunGetAdmission) snapshot() []string {
	a.mu.Lock()
	defer a.mu.Unlock()
	return append([]string(nil), a.ids...)
}

func (a *recordingRunGetAdmission) record(event string) {
	a.mu.Lock()
	a.events = append(a.events, event)
	a.mu.Unlock()
}

func (a *recordingRunGetAdmission) eventSnapshot() []string {
	a.mu.Lock()
	defer a.mu.Unlock()
	return append([]string(nil), a.events...)
}

func TestClientUsesAdmissionOnlyForPhysicalRunGetAttempts(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) { return `{"run_id":"run-a"}`, false })
	owner := &recordingRunGetAdmission{}
	cfg := testConfig(fs.socket)
	cfg.RunGetAdmission = owner
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	if _, err := client.Do(context.Background(), NewRunGet("run-a")); err != nil {
		t.Fatalf("run_get: %v", err)
	}
	if _, err := client.Do(context.Background(), NewCaseList()); err != nil {
		t.Fatalf("case_list: %v", err)
	}
	ids := owner.snapshot()
	if len(ids) != 1 || ids[0] != "run-a" {
		t.Fatalf("admission run IDs=%v, want [run-a] (client hook currently absent)", ids)
	}
}

func TestClientAdmissionCountsAllFourRunGetRetryAttempts(t *testing.T) {
	var calls atomic.Int32
	fs := newFakeServer(t, func(string) (string, bool) {
		switch calls.Add(1) {
		case 1, 3:
			return "", true // safe transport failure; roundTrip retries once
		case 2:
			return `{"error":"busy","error_class":"server_busy","no_side_effect":true}`, false
		default:
			return `{"run_id":"run-a"}`, false
		}
	})
	owner := &recordingRunGetAdmission{}
	cfg := testConfig(fs.socket)
	cfg.RunGetAdmission = owner
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	if _, err := client.Do(context.Background(), NewRunGet("run-a")); err != nil {
		t.Fatalf("run_get retry sequence: %v", err)
	}
	ids := owner.snapshot()
	if got := fs.requestCount("run_get"); got != 4 {
		t.Fatalf("physical run_get attempts=%d, want four", got)
	}
	if len(ids) != 4 {
		t.Fatalf("admission acquisitions=%v, want four physical attempts; client hook currently absent", ids)
	}
}

func TestClientDoesNotRouteAskOrMutationThroughRunGetAdmission(t *testing.T) {
	var asks int
	fs := newFakeServer(t, func(line string) (string, bool) {
		var req struct {
			Verb string `json:"verb"`
		}
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			return `{"error":"bad request"}`, false
		}
		if req.Verb == "ask" {
			asks++
			if asks == 1 {
				return `{"error":"busy","error_class":"server_busy","no_side_effect":true}`, false
			}
			return `{"answer_id":"answer-1"}`, false
		}
		return `{"error":"denied","error_class":"authority_denied"}`, false
	})
	owner := &recordingRunGetAdmission{}
	cfg := testConfig(fs.socket)
	cfg.RunGetAdmission = owner
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	if _, err := client.Do(context.Background(), rawAskRequest()); err != nil {
		t.Fatalf("ask: %v", err)
	}
	if got := fs.requestCount("ask"); got != 2 {
		t.Fatalf("Ask busy behavior sent %d requests, want one bounded refusal retry", got)
	}
	if got := fs.requestCount("request_get_status"); got != 0 {
		t.Fatalf("Ask performed %d mutation outcome lookups, want none", got)
	}
	mutation, err := NewCaseAddItem("case-a", map[string]any{"plan_item_id": "item-a", "name": "n", "item_kind": "sandboxed_task", "settlement_criteria": map[string]any{}}, "", "req-a", Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := client.Do(context.Background(), mutation); err == nil {
		t.Fatal("mutation refusal unexpectedly succeeded")
	} else {
		var refusal *Refusal
		if !errors.As(err, &refusal) || refusal.Class != "authority_denied" {
			t.Fatalf("mutation refusal changed shape: %v", err)
		}
	}
	ids := owner.snapshot()
	if len(ids) != 0 {
		t.Fatalf("non-run_get calls reached admission: IDs=%v", ids)
	}
}

func TestMutationTransportFailureIsNotResentThroughAdmission(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) { return "", true })
	owner := &recordingRunGetAdmission{}
	cfg := testConfig(fs.socket)
	cfg.RunGetAdmission = owner
	cfg.RecoveryBudget = 100 * time.Millisecond
	cfg.BackoffBase = time.Millisecond
	cfg.BackoffMax = time.Millisecond
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	mutation, err := NewCaseAddItem("case-a", map[string]any{"plan_item_id": "item-a", "name": "n", "item_kind": "sandboxed_task", "settlement_criteria": map[string]any{}}, "", "req-no-resend", Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := client.Do(context.Background(), mutation); err == nil {
		t.Fatal("severed mutation unexpectedly succeeded")
	}
	if got := fs.requestCount("case_add_item"); got != 1 {
		t.Fatalf("mutation resend count=%d, want exactly one", got)
	}
	if got := len(owner.snapshot()); got != 0 {
		t.Fatalf("mutation used read-only admission %d times", got)
	}
}

func TestClientAdmissionMarksPayloadAndLFWriteBoundary(t *testing.T) {
	fs := newFakeServer(t, func(string) (string, bool) { return `{"run_id":"run-a"}`, false })
	owner := &recordingRunGetAdmission{}
	poolAtRelease := make(chan admissionPoolObservation, 1)
	var expectedConn net.Conn
	cfg := testConfig(fs.socket)
	cfg.RunGetAdmission = owner
	cfg.Dial = func(ctx context.Context, socket string) (net.Conn, error) {
		cn, err := (&net.Dialer{}).DialContext(ctx, "unix", socket)
		if err != nil {
			return nil, err
		}
		tracked := writeTraceConn{Conn: cn, owner: owner}
		expectedConn = tracked
		return tracked, nil
	}
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	owner.onRelease = func() { poolAtRelease <- inspectPoolAtRelease(client, expectedConn) }
	if _, err := client.Do(context.Background(), NewRunGet("run-a")); err != nil {
		t.Fatalf("run_get: %v", err)
	}

	events := owner.eventSnapshot()
	want := []string{"permit-start", "payload-write", "lf-write", "permit-finish", "permit-release"}
	if !sameStrings(events, want) {
		t.Fatalf("admission/write sequence=%v, want %v (client hook currently absent)", events, want)
	}
	select {
	case observation := <-poolAtRelease:
		if !observation.lockAcquired || observation.live != 1 || observation.idle != 1 || !observation.expectedIdle {
			t.Fatalf("healthy permit release observed pool state %+v; want released live=1 pool containing the returned connection", observation)
		}
	case <-time.After(time.Second):
		t.Fatal("healthy permit release did not report bounded pool-state observation")
	}
}

type writeTraceConn struct {
	net.Conn
	owner *recordingRunGetAdmission
}

func (c writeTraceConn) Write(p []byte) (int, error) {
	if string(p) == "\n" {
		c.owner.record("lf-write")
	} else {
		c.owner.record("payload-write")
	}
	return c.Conn.Write(p)
}

func sameStrings(got, want []string) bool {
	if len(got) != len(want) {
		return false
	}
	for i := range got {
		if got[i] != want[i] {
			return false
		}
	}
	return true
}

func TestClientAdmissionFinishesAtFinalAttemptedWriteOnPayloadAndLFFailure(t *testing.T) {
	for _, failAt := range []int{1, 2} {
		t.Run(map[int]string{1: "partial payload", 2: "LF write error"}[failAt], func(t *testing.T) {
			owner := &recordingRunGetAdmission{}
			var mu sync.Mutex
			var peers []net.Conn
			cfg := Config{
				SocketPath: "controlled-write-failure", MaxConns: 1, RequestTimeout: time.Second,
				RunGetAdmission: owner,
				Dial: func(context.Context, string) (net.Conn, error) {
					clientSide, peerSide := net.Pipe()
					mu.Lock()
					peers = append(peers, peerSide)
					mu.Unlock()
					go func() { _, _ = io.Copy(io.Discard, peerSide); _ = peerSide.Close() }()
					return &failWriteConn{Conn: clientSide, failAt: failAt}, nil
				},
			}
			client, err := New(cfg)
			if err != nil {
				t.Fatal(err)
			}
			defer func() {
				client.Close()
				mu.Lock()
				defer mu.Unlock()
				for _, peer := range peers {
					_ = peer.Close()
				}
			}()
			_, err = client.Do(context.Background(), NewRunGet("run-a"))
			if err == nil {
				t.Fatal("controlled partial/error writes unexpectedly succeeded")
			}
			events := owner.eventSnapshot()
			if countEvent(events, "permit-start") == 0 || countEvent(events, "permit-finish") != countEvent(events, "permit-start") {
				t.Fatalf("attempt completion accounting=%v; every started partial/LF attempt must finish before cleanup (hook currently absent)", events)
			}
		})
	}
}

type failWriteConn struct {
	net.Conn
	failAt int
	writes int
}

func (c *failWriteConn) Write(p []byte) (int, error) {
	c.writes++
	if c.writes == c.failAt {
		if c.failAt == 1 && len(p) > 1 {
			return len(p) / 2, errors.New("controlled partial payload")
		}
		return 0, errors.New("controlled LF write failure")
	}
	return c.Conn.Write(p)
}

func countEvent(events []string, event string) int {
	n := 0
	for _, got := range events {
		if got == event {
			n++
		}
	}
	return n
}

func TestClientWaitsForBlockedPayloadAndLFAttemptBeforeFinish(t *testing.T) {
	for _, failAt := range []int{1, 2} {
		name := map[int]string{1: "blocked payload", 2: "blocked LF"}[failAt]
		t.Run(name, func(t *testing.T) {
			owner := &recordingRunGetAdmission{}
			entered := make(chan struct{})
			unblock := make(chan struct{})
			var once sync.Once
			var dialCount int
			cfg := Config{
				SocketPath: "controlled-blocked-write", MaxConns: 1, RequestTimeout: time.Second,
				RunGetAdmission: owner,
				Dial: func(context.Context, string) (net.Conn, error) {
					dialCount++
					if dialCount > 1 {
						return nil, errors.New("bounded retry dial failure")
					}
					clientSide, peerSide := net.Pipe()
					go func() { _, _ = io.Copy(io.Discard, peerSide); _ = peerSide.Close() }()
					return &blockedWriteConn{Conn: clientSide, failAt: failAt, entered: entered, unblock: unblock, owner: owner}, nil
				},
			}
			client, err := New(cfg)
			if err != nil {
				t.Fatal(err)
			}
			callDone := make(chan error, 1)
			joined := false
			defer func() {
				once.Do(func() { close(unblock) })
				client.Close()
				if !joined {
					select {
					case <-callDone:
					case <-time.After(time.Second):
						t.Error("blocked-write caller did not join during cleanup")
					}
				}
			}()
			go func() { _, err := client.Do(context.Background(), NewRunGet("run-a")); callDone <- err }()
			select {
			case <-entered:
			case <-time.After(time.Second):
				t.Fatal("physical write did not reach controlled block")
			}
			finishWhileBlocked := countEvent(owner.eventSnapshot(), "permit-finish")
			once.Do(func() { close(unblock) })
			select {
			case err := <-callDone:
				joined = true
				if err == nil {
					t.Fatal("controlled blocked write unexpectedly succeeded")
				}
			case <-time.After(time.Second):
				t.Fatal("blocked write/retry did not finish after release")
			}
			if finishWhileBlocked != 0 {
				t.Fatalf("permit finished before blocked payload/LF write returned: %v", owner.eventSnapshot())
			}
			events := owner.eventSnapshot()
			if countEvent(events, "permit-start") == 0 || countEvent(events, "permit-finish") != countEvent(events, "permit-start") {
				t.Fatalf("blocked attempt did not finish exactly once after write return: %v (client hook currently absent)", events)
			}
		})
	}
}

type blockedWriteConn struct {
	net.Conn
	failAt  int
	writes  int
	entered chan struct{}
	unblock <-chan struct{}
	owner   *recordingRunGetAdmission
}

func (c *blockedWriteConn) Write(p []byte) (int, error) {
	c.writes++
	writeName := "payload-write"
	if string(p) == "\n" {
		writeName = "lf-write"
	}
	if c.writes == c.failAt {
		c.owner.record(writeName + "-blocked")
		close(c.entered)
		<-c.unblock
		c.owner.record(writeName + "-returned")
		if c.writes == 1 && len(p) > 1 {
			return len(p) / 2, errors.New("controlled blocked partial payload")
		}
		return 0, errors.New("controlled blocked LF failure")
	}
	c.owner.record(writeName)
	return c.Conn.Write(p)
}

func TestClientAdmissionReleaseFollowsCancellationAndConnectionRetirement(t *testing.T) {
	owner := &recordingRunGetAdmission{}
	poolAtRelease := make(chan admissionPoolObservation, 1)
	clientSide, peerSide := net.Pipe()
	closeEntered := make(chan struct{})
	allowClose := make(chan struct{})
	var allowOnce sync.Once
	unblockClose := func() { allowOnce.Do(func() { close(allowClose) }) }
	cn := &closeGateConn{Conn: clientSide, entered: closeEntered, allow: allowClose, owner: owner}
	cfg := Config{
		SocketPath: "controlled-close-order", RequestTimeout: time.Second,
		RunGetAdmission: owner,
		Dial:            func(context.Context, string) (net.Conn, error) { return cn, nil },
	}
	client, err := New(cfg)
	if err != nil {
		t.Fatal(err)
	}
	owner.onRelease = func() { poolAtRelease <- inspectPoolAtRelease(client, cn) }
	peerDone := make(chan struct{})
	var peerDoneOnce sync.Once
	stopPeer := func() { peerDoneOnce.Do(func() { close(peerDone) }) }
	requestReceived := make(chan struct{})
	peerReadError := make(chan error, 1)
	go func() {
		if _, err := bufio.NewReader(peerSide).ReadString('\n'); err != nil {
			peerReadError <- err
			return
		}
		close(requestReceived)
		<-peerDone
		_ = peerSide.Close()
	}()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	callDone := make(chan error, 1)
	joined := false
	defer func() {
		unblockClose()
		cancel()
		stopPeer()
		client.Close()
		_ = peerSide.Close()
		if !joined {
			select {
			case <-callDone:
			case <-time.After(time.Second):
				t.Error("canceled client call did not join during cleanup")
			}
		}
	}()
	go func() { _, err := client.Do(ctx, NewRunGet("run-a")); callDone <- err }()
	select {
	case <-requestReceived:
	case err := <-peerReadError:
		t.Fatalf("peer failed before receiving the physical request line: %v", err)
	case <-time.After(time.Second):
		t.Fatal("peer did not receive the physical request line before cancellation")
	}
	cancel()
	select {
	case <-closeEntered:
	case <-time.After(time.Second):
		t.Fatal("cancellation did not enter owned connection close")
	}
	if countEvent(owner.eventSnapshot(), "permit-release") != 0 {
		t.Fatal("permit released while the cancellation close callback was still blocked")
	}
	unblockClose()
	stopPeer()
	select {
	case <-callDone:
		joined = true
	case <-time.After(time.Second):
		t.Fatal("canceled run_get did not join callback/retirement")
	}
	events := owner.eventSnapshot()
	closeAt, releaseAt := indexEvent(events, "connection-close"), indexEvent(events, "permit-release")
	if closeAt < 0 || releaseAt < 0 || closeAt > releaseAt {
		t.Fatalf("connection retirement/permit release order=%v, want close before release (hook currently absent)", events)
	}
	select {
	case observation := <-poolAtRelease:
		if !observation.lockAcquired || observation.live != 0 || observation.idle != 0 {
			t.Fatalf("canceled permit release observed pool state %+v; want retired live=0 with no idle connection", observation)
		}
	case <-time.After(time.Second):
		t.Fatal("canceled permit release did not report bounded pool-state observation")
	}
}

type admissionPoolObservation struct {
	lockAcquired bool
	live         int
	idle         int
	expectedIdle bool
}

func inspectPoolAtRelease(client *Client, expected net.Conn) admissionPoolObservation {
	if !client.mu.TryLock() {
		return admissionPoolObservation{}
	}
	defer client.mu.Unlock()
	observation := admissionPoolObservation{lockAcquired: true, live: client.live, idle: len(client.idle)}
	for _, cn := range client.idle {
		if cn.nc == expected {
			observation.expectedIdle = true
			break
		}
	}
	return observation
}

type closeGateConn struct {
	net.Conn
	entered chan struct{}
	allow   chan struct{}
	owner   *recordingRunGetAdmission
	first   sync.Once
}

func (c *closeGateConn) Close() error {
	waited := false
	c.first.Do(func() { waited = true; close(c.entered) })
	if waited {
		<-c.allow
	}
	c.owner.record("connection-close")
	return c.Conn.Close()
}

func indexEvent(events []string, event string) int {
	for i, got := range events {
		if got == event {
			return i
		}
	}
	return -1
}
