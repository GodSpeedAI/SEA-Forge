//go:build live

// Live integration tests against the REAL sea-forge-server kernel (plan T05, proof level P2).
//
// TestMain builds (or reuses) the real server binary, and each test boots it on a fresh temp cell
// seeded the way `just casework-cell-init` seeds the live cell: the E2E plan templates, the E2E
// authority policy, and a server.yaml binding the invoking OS user to the operator actor
// (mirroring crates/sea-forge-server/tests/sfwp_delegated_identity.rs seeds). Nothing here talks to
// a fake: every verb round-trips over the real Unix socket, and the kill/restart teeth prove the
// recovery contract against the kernel's own durable correlation store.
package sfwp

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sync"
	"testing"
	"time"
)

// ---------------------------------------------------------------------------
// Harness: real binary, temp cell, restart control
// ---------------------------------------------------------------------------

func repoRoot(t *testing.T) string {
	t.Helper()
	return repoRootForMain()
}

func serverBinary(t *testing.T) string {
	t.Helper()
	root := repoRoot(t)
	bin := filepath.Join(root, "target", "debug", "sea-forge-server")
	if _, err := os.Stat(bin); err == nil {
		return bin
	}
	t.Log("target/debug/sea-forge-server is missing; building it once with cargo")
	cmd := exec.Command("cargo", "build", "-p", "sea-forge-server", "--bin", "sea-forge-server")
	cmd.Dir = root
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		t.Fatalf("cannot build sea-forge-server: %v", err)
	}
	return bin
}

// liveCell is one bootable cell: a seeded temp root plus the running server process.
type liveCell struct {
	t      *testing.T
	root   string
	socket string
	bin    string

	mu      sync.Mutex
	proc    *os.Process
	logFile *os.File
	logPath string
}

func newLiveCell(t *testing.T) *liveCell {
	t.Helper()
	// SFWP_KEEP_CELL=1 keeps the cell under /tmp/sfwp-live-debug-* for post-mortem inspection.
	if os.Getenv("SFWP_KEEP_CELL") != "" {
		root, err := os.MkdirTemp("/tmp", "sfwp-live-debug-")
		if err != nil {
			t.Fatal(err)
		}
		t.Logf("keeping cell at %s", root)
		return newLiveCellAt(t, root)
	}
	return newLiveCellAt(t, t.TempDir())
}

func newLiveCellAt(t *testing.T, root string) *liveCell {
	t.Helper()
	cell := &liveCell{
		t:      t,
		root:   root,
		socket: filepath.Join(root, "server.sock"),
		bin:    serverBinary(t),
	}
	cell.seed()
	cell.start()
	t.Cleanup(cell.stop)
	return cell
}

// seed mirrors `just casework-cell-init` / the Rust T02/T03 test seeds: templates, policy, and an
// identity binding for the invoking uid. No gateway section - delegation refusals are part of the
// proof.
func (c *liveCell) seed() {
	root := repoRoot(c.t)
	fixtures := filepath.Join(root, "fixtures", "cells", "e2e")

	tplDir := filepath.Join(c.root, "templates")
	if err := os.MkdirAll(tplDir, 0o755); err != nil {
		c.t.Fatal(err)
	}
	entries, err := os.ReadDir(filepath.Join(fixtures, "templates"))
	if err != nil {
		c.t.Fatalf("E2E fixture templates missing: %v", err)
	}
	installed := 0
	for _, e := range entries {
		if e.IsDir() || filepath.Ext(e.Name()) != ".yaml" {
			continue
		}
		raw, err := os.ReadFile(filepath.Join(fixtures, "templates", e.Name()))
		if err != nil {
			c.t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(tplDir, e.Name()), raw, 0o644); err != nil {
			c.t.Fatal(err)
		}
		installed++
	}
	if installed < 2 {
		c.t.Fatalf("expected both E2E templates, installed %d", installed)
	}

	authDir := filepath.Join(c.root, "authority")
	if err := os.MkdirAll(authDir, 0o755); err != nil {
		c.t.Fatal(err)
	}
	policy, err := os.ReadFile(filepath.Join(fixtures, "policy.yaml"))
	if err != nil {
		c.t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(authDir, "active-policy.json"), policy, 0o644); err != nil {
		c.t.Fatal(err)
	}

	uid := os.Getuid()
	serverYAML := fmt.Sprintf("# T05 live Go-client cell: the invoking OS user (uid %d) is the operator.\n"+
		"identity:\n  bindings:\n    - uid: %d\n      actor_id: operator_local\n      roles: [\"operator\"]\n", uid, uid)
	if err := os.WriteFile(filepath.Join(c.root, "server.yaml"), []byte(serverYAML), 0o644); err != nil {
		c.t.Fatal(err)
	}
}

// start boots the server (zero CLI arguments; env-configured) and waits for the socket.
func (c *liveCell) start() {
	c.t.Helper()
	c.mu.Lock()
	if c.proc != nil {
		c.mu.Unlock()
		c.t.Fatal("start called while a server process is already running")
	}
	logPath := filepath.Join(c.root, fmt.Sprintf("server-%d.log", time.Now().UnixNano()))
	logFile, err := os.Create(logPath)
	if err != nil {
		c.mu.Unlock()
		c.t.Fatal(err)
	}
	cmd := exec.Command(c.bin)
	cmd.Env = append(os.Environ(),
		"SEA_FORGE_ROOT="+c.root,
		"SEA_FORGE_SOCKET="+c.socket,
	)
	cmd.Stdout = logFile
	cmd.Stderr = logFile
	if err := cmd.Start(); err != nil {
		logFile.Close()
		c.mu.Unlock()
		c.t.Fatalf("cannot start sea-forge-server: %v", err)
	}
	c.proc = cmd.Process
	c.logFile = logFile
	c.logPath = logPath
	c.mu.Unlock()

	deadline := time.Now().Add(20 * time.Second)
	for time.Now().Before(deadline) {
		if _, err := os.Stat(c.socket); err == nil {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	c.t.Fatalf("server socket %s never appeared; log at %s", c.socket, c.logPath)
}

// kill SIGKILLs the server (the crash simulation - no graceful drain).
func (c *liveCell) kill() {
	c.mu.Lock()
	proc := c.proc
	c.proc = nil
	if c.logFile != nil {
		c.logFile.Close()
		c.logFile = nil
	}
	c.mu.Unlock()
	if proc == nil {
		return
	}
	if err := proc.Kill(); err != nil {
		c.t.Logf("kill: %v", err)
	}
	_, _ = proc.Wait()
	os.Remove(c.socket)
}

// stop is the cleanup path.
func (c *liveCell) stop() { c.kill() }

// client builds a fastish client against this cell.
func (c *liveCell) client() *Client {
	cl, err := New(Config{
		SocketPath:     c.socket,
		MaxConns:       4,
		RequestTimeout: 30 * time.Second,
		ConnectTimeout: 2 * time.Second,
		RecoveryBudget: 20 * time.Second,
		BackoffBase:    50 * time.Millisecond,
		BackoffMax:     500 * time.Millisecond,
		SubscribeIdle:  30 * time.Second,
	})
	if err != nil {
		c.t.Fatal(err)
	}
	c.t.Cleanup(cl.Close)
	return cl
}

// caseEvents reads one case's durable case-events.jsonl (kernel truth, read off disk).
func (c *liveCell) caseEvents(caseID string) []map[string]any {
	raw, err := os.ReadFile(filepath.Join(c.root, "cases", caseID, "case-events.jsonl"))
	if err != nil {
		return nil
	}
	var events []map[string]any
	for _, line := range splitLines(string(raw)) {
		var ev map[string]any
		if json.Unmarshal([]byte(line), &ev) == nil {
			events = append(events, ev)
		}
	}
	return events
}

// countCaseCreated counts case_created trace events across the whole cell
// (the case ledger spells TraceKinds snake_case).
func (c *liveCell) countCaseCreated() int {
	total := 0
	casesDir := filepath.Join(c.root, "cases")
	entries, err := os.ReadDir(casesDir)
	if err != nil {
		return 0
	}
	for _, e := range entries {
		if !e.IsDir() {
			continue
		}
		for _, ev := range c.caseEvents(e.Name()) {
			if ev["kind"] == "case_created" {
				total++
			}
		}
	}
	return total
}

// requestRecord reads the durable correlation record for one request id.
func (c *liveCell) requestRecord(requestID string) map[string]any {
	raw, err := os.ReadFile(filepath.Join(c.root, "requests", requestID+".json"))
	if err != nil {
		return nil
	}
	var rec map[string]any
	if json.Unmarshal(raw, &rec) != nil {
		return nil
	}
	return rec
}

// settlementBases reads one case's durable settlement records from the case
// ledger - the only place an episode's rejection basis is recorded. It is how
// a test distinguishes "the kernel judged the work and rejected it" from
// "the kernel could not run the work at all".
func (c *liveCell) settlementBases(caseID string) []string {
	raw, err := os.ReadFile(filepath.Join(c.root, "ledgers", "case-"+caseID, "entries.jsonl"))
	if err != nil {
		return nil
	}
	var bases []string
	for _, line := range splitLines(string(raw)) {
		var entry struct {
			RecordKind string          `json:"record_kind"`
			Payload    json.RawMessage `json:"payload"`
		}
		if json.Unmarshal([]byte(line), &entry) != nil || entry.RecordKind != "settlement_event" {
			continue
		}
		var payload struct {
			Basis []string `json:"basis"`
		}
		if json.Unmarshal(entry.Payload, &payload) == nil {
			bases = append(bases, payload.Basis...)
		}
	}
	return bases
}

func splitLines(s string) []string {
	var out []string
	start := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '\n' {
			line := trimSpace(s[start:i])
			if line != "" {
				out = append(out, line)
			}
			start = i + 1
		}
	}
	if start < len(s) {
		line := trimSpace(s[start:])
		if line != "" {
			out = append(out, line)
		}
	}
	return out
}

func trimSpace(s string) string {
	for len(s) > 0 && (s[0] == ' ' || s[0] == '\r' || s[0] == '\t') {
		s = s[1:]
	}
	for len(s) > 0 && (s[len(s)-1] == ' ' || s[len(s)-1] == '\r' || s[len(s)-1] == '\t') {
		s = s[:len(s)-1]
	}
	return s
}

// rawConn is a minimal line client over the socket (the recording harness for golden frames).
type rawConn struct {
	nc net.Conn
	br *bufio.Reader
}

func dialRaw(t *testing.T, socket string) *rawConn {
	t.Helper()
	nc, err := net.Dial("unix", socket)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { nc.Close() })
	return &rawConn{nc: nc, br: bufio.NewReader(nc)}
}

func (rc *rawConn) call(t *testing.T, req map[string]any) string {
	t.Helper()
	raw, err := json.Marshal(req)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := rc.nc.Write(append(raw, '\n')); err != nil {
		t.Fatalf("raw write: %v", err)
	}
	_ = rc.nc.SetReadDeadline(time.Now().Add(30 * time.Second))
	line, err := rc.br.ReadString('\n')
	if err != nil {
		t.Fatalf("raw read: %v", err)
	}
	return trimSpace(line)
}

// ---------------------------------------------------------------------------
// Shared ladder pieces
// ---------------------------------------------------------------------------

func operatorGov() Governance {
	return Governance{Actor: Actor{ActorID: "operator_local", Role: "operator"}}
}

// commitCase commits one sentry-chain case through the client.
func commitCase(ctx context.Context, t *testing.T, cl *Client, requestID string) CommitView {
	t.Helper()
	req, err := NewCaseCommit(sentryChainRef, sentryChainParams(), policyRef, "sfwp_live_tests", 60,
		requestID, nil, operatorGov())
	if err != nil {
		t.Fatal(err)
	}
	resp, err := cl.Do(ctx, req)
	if err != nil {
		t.Fatalf("case.commit failed: %v", err)
	}
	var view CommitView
	if err := resp.Into(&view); err != nil {
		t.Fatal(err)
	}
	if view.CaseID == "" {
		t.Fatalf("commit returned no case id: %+v", view)
	}
	return view
}

// artifactDigestFromRun reads the run's evidence journal on disk to find a committed digest.
// Runs live under <root>/runs/<run> or <root>/cases/<case>/runs/<run>.
func artifactDigestFromRun(c *liveCell, runID string) string {
	var raw []byte
	for _, pattern := range []string{
		filepath.Join(c.root, "runs", runID, "evidence.jsonl"),
		filepath.Join(c.root, "cases", "*", "runs", runID, "evidence.jsonl"),
	} {
		matches, _ := filepath.Glob(pattern)
		for _, m := range matches {
			if data, err := os.ReadFile(m); err == nil {
				raw = data
				break
			}
		}
		if raw != nil {
			break
		}
	}
	if raw == nil {
		c.t.Logf("no evidence.jsonl found for run %s", runID)
		return ""
	}
	for _, line := range splitLines(string(raw)) {
		var rec struct {
			URI    string `json:"uri"`
			SHA256 string `json:"sha256"`
		}
		if json.Unmarshal([]byte(line), &rec) == nil && rec.SHA256 != "" {
			return rec.SHA256
		}
	}
	return ""
}

// waitSeen polls until cond is true or the timeout expires.
func waitSeen(t *testing.T, timeout time.Duration, what string, cond func() bool) {
	t.Helper()
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		if cond() {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatalf("timed out waiting for %s", what)
}

func slicesContains(haystack []string, needle string) bool {
	for _, s := range haystack {
		if s == needle {
			return true
		}
	}
	return false
}

// ---------------------------------------------------------------------------
// TestMain: ensure the binary exists before any live test runs.
// ---------------------------------------------------------------------------

func TestMain(m *testing.M) {
	// Build once up front (reused per test); failures are fatal, not skip: a live gate that
	// cannot run must not report green.
	if _, err := os.Stat(filepath.Join(repoRootForMain(), "target", "debug", "sea-forge-server")); err != nil {
		cmd := exec.Command("cargo", "build", "-p", "sea-forge-server", "--bin", "sea-forge-server")
		cmd.Dir = repoRootForMain()
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			fmt.Fprintf(os.Stderr, "live tests: cannot build sea-forge-server: %v\n", err)
			os.Exit(1)
		}
	}
	os.Exit(m.Run())
}

// repoRootForMain locates the repository root from THIS FILE's location (robust against the
// test binary's working directory).
func repoRootForMain() string {
	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		return "."
	}
	// <root>/apps/godspeed-casework-go/internal/adapters/sfwp/live_test.go
	root := filepath.Dir(filepath.Join(thisFile, "..", "..", "..", "..", ".."))
	if _, err := os.Stat(filepath.Join(root, "crates", "sea-forge-server", "Cargo.toml")); err != nil {
		return "."
	}
	return root
}
