// Package livetest is the shared harness for -tags live integration tests against the REAL
// sea-forge-server kernel (mirroring the T05 harness in internal/adapters/sfwp, extended for T06:
// the seeded cell carries the T02 gateway principal and two delegable end-user actors).
//
// Nothing here talks to a fake kernel: every helper boots the real binary on a fresh temp cell
// seeded the way the live stack expects - the E2E plan templates, the E2E authority policy, and a
// server.yaml binding the invoking OS user to the gateway principal with operator_local
// (operator) and rso_local (R-SO) in the delegation allowlist.
package livetest

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sync"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// ServerBinary returns the path to the kernel binary, building it once if missing.
func ServerBinary(t *testing.T) string {
	t.Helper()
	root := RepoRoot(t)
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

// RepoRoot locates the repository root from this file's location.
func RepoRoot(t *testing.T) string {
	t.Helper()
	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		t.Fatal("cannot locate repo root")
	}
	// <root>/apps/godspeed-casework-go/internal/livetest/harness.go is three directories deep.
	root := filepath.Dir(filepath.Join(thisFile, "..", "..", "..", ".."))
	if _, err := os.Stat(filepath.Join(root, "crates", "sea-forge-server", "Cargo.toml")); err != nil {
		t.Fatalf("repo root not found from %s", thisFile)
	}
	return root
}

// EnsureServerBinary builds the kernel binary once per test binary run when missing (TestMain).
func EnsureServerBinary() {
	root := repoRootForMain()
	if _, err := os.Stat(filepath.Join(root, "target", "debug", "sea-forge-server")); err == nil {
		return
	}
	cmd := exec.Command("cargo", "build", "-p", "sea-forge-server", "--bin", "sea-forge-server")
	cmd.Dir = root
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "live tests: cannot build sea-forge-server: %v\n", err)
		os.Exit(1)
	}
}

func repoRootForMain() string {
	_, thisFile, _, ok := runtime.Caller(0)
	if !ok {
		return "."
	}
	root := filepath.Dir(filepath.Join(thisFile, "..", "..", "..", ".."))
	if _, err := os.Stat(filepath.Join(root, "crates", "sea-forge-server", "Cargo.toml")); err != nil {
		return "."
	}
	return root
}

// Cell is one bootable cell with the T06 identity posture: the invoking uid is the gateway
// principal; operator_local (operator) and rso_local (R-SO) are the delegable end users.
type Cell struct {
	t      *testing.T
	root   string
	socket string
	bin    string

	mu      sync.Mutex
	proc    *os.Process
	logFile *os.File
}

// NewCell seeds and boots a fresh cell.
func NewCell(t *testing.T) *Cell {
	t.Helper()
	root := t.TempDir()
	cell := &Cell{
		t:      t,
		root:   root,
		socket: filepath.Join(root, "server.sock"),
		bin:    ServerBinary(t),
	}
	cell.seed()
	cell.start()
	t.Cleanup(cell.Stop)
	return cell
}

// Root is the cell root (durable kernel state lives under it).
func (c *Cell) Root() string { return c.root }

func (c *Cell) seed() {
	t := c.t
	root := RepoRoot(t)
	fixtures := filepath.Join(root, "fixtures", "cells", "e2e")

	tplDir := filepath.Join(c.root, "templates")
	if err := os.MkdirAll(tplDir, 0o755); err != nil {
		t.Fatal(err)
	}
	entries, err := os.ReadDir(filepath.Join(fixtures, "templates"))
	if err != nil {
		t.Fatalf("E2E fixture templates missing: %v", err)
	}
	installed := 0
	for _, e := range entries {
		if e.IsDir() || filepath.Ext(e.Name()) != ".yaml" {
			continue
		}
		raw, err := os.ReadFile(filepath.Join(fixtures, "templates", e.Name()))
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(filepath.Join(tplDir, e.Name()), raw, 0o644); err != nil {
			t.Fatal(err)
		}
		installed++
	}
	if installed < 2 {
		t.Fatalf("expected both E2E templates, installed %d", installed)
	}

	authDir := filepath.Join(c.root, "authority")
	if err := os.MkdirAll(authDir, 0o755); err != nil {
		t.Fatal(err)
	}
	policy, err := os.ReadFile(filepath.Join(fixtures, "policy.yaml"))
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(authDir, "active-policy.json"), policy, 0o644); err != nil {
		t.Fatal(err)
	}

	uid := os.Getuid()
	serverYAML := fmt.Sprintf(
		"# T06 live gateway cell: the invoking uid (%d) is the gateway principal; the two\n"+
			"# delegable end users mirror the L5 journey posture (operator executes, R-SO approves).\n"+
			"identity:\n"+
			"  bindings:\n"+
			"    - uid: %d\n"+
			"      actor_id: gateway\n"+
			"      roles: [\"service\"]\n"+
			"    - uid: 3101\n"+
			"      actor_id: operator_local\n"+
			"      roles: [\"operator\"]\n"+
			"    - uid: 3102\n"+
			"      actor_id: rso_local\n"+
			"      roles: [\"R-SO\"]\n"+
			"gateway:\n"+
			"  uid: %d\n"+
			"  actor: gateway\n"+
			"  delegable_actors: [operator_local, rso_local]\n", uid, uid, uid)
	if err := os.WriteFile(filepath.Join(c.root, "server.yaml"), []byte(serverYAML), 0o644); err != nil {
		t.Fatal(err)
	}
}

func (c *Cell) start() {
	t := c.t
	t.Helper()
	c.mu.Lock()
	if c.proc != nil {
		c.mu.Unlock()
		t.Fatal("start called while a server process is already running")
	}
	logFile, err := os.Create(filepath.Join(c.root, "server.log"))
	if err != nil {
		c.mu.Unlock()
		t.Fatal(err)
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
		t.Fatalf("cannot start sea-forge-server: %v", err)
	}
	c.proc = cmd.Process
	c.logFile = logFile
	c.mu.Unlock()

	deadline := time.Now().Add(20 * time.Second)
	for time.Now().Before(deadline) {
		if _, err := os.Stat(c.socket); err == nil {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatal("server socket never appeared; log at " + c.root + "/server.log")
}

// Stop terminates the server (the cleanup path).
func (c *Cell) Stop() {
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
	_ = proc.Kill()
	_, _ = proc.Wait()
	_ = os.Remove(c.socket)
}

// Client builds a client against this cell (fast backoffs for tests).
func (c *Cell) Client() *sfwp.Client {
	t := c.t
	cl, err := sfwp.New(sfwp.Config{
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
		t.Fatal(err)
	}
	t.Cleanup(cl.Close)
	return cl
}

// CaseEvents reads one case's durable case-events.jsonl (kernel truth, read off disk).
func (c *Cell) CaseEvents(caseID string) []map[string]any {
	raw, err := os.ReadFile(filepath.Join(c.root, "cases", caseID, "case-events.jsonl"))
	if err != nil {
		return nil
	}
	var events []map[string]any
	for _, line := range splitLines(string(raw)) {
		var ev map[string]any
		if jsonUnmarshal([]byte(line), &ev) == nil {
			events = append(events, ev)
		}
	}
	return events
}

// CountTraceKinds counts events of a kind across one case's durable ledger.
func (c *Cell) CountTraceKinds(caseID, kind string) int {
	total := 0
	for _, ev := range c.CaseEvents(caseID) {
		if ev["kind"] == kind {
			total++
		}
	}
	return total
}

// RequestRecord reads the durable correlation record for one request id (nil when absent) - the
// proof surface for "the kernel received no call".
func (c *Cell) RequestRecord(requestID string) map[string]any {
	raw, err := os.ReadFile(filepath.Join(c.root, "requests", requestID+".json"))
	if err != nil {
		return nil
	}
	var rec map[string]any
	if jsonUnmarshal(raw, &rec) != nil {
		return nil
	}
	return rec
}

// CaseDirs lists the case directories under the cell (the durable case-existence proof).
func (c *Cell) CaseDirs() []string {
	entries, err := os.ReadDir(filepath.Join(c.root, "cases"))
	if err != nil {
		return nil
	}
	var out []string
	for _, e := range entries {
		if e.IsDir() {
			out = append(out, e.Name())
		}
	}
	return out
}

// DelegationAuditRecords reads the cell's delegation-audit ledger (the T02 durable
// pair-principal record for every admitted delegated request). The file is JSONL; records that
// fail to parse are skipped.
func (c *Cell) DelegationAuditRecords() []map[string]any {
	raw, err := os.ReadFile(filepath.Join(c.root, "ledgers", "delegation-audit", "entries.jsonl"))
	if err != nil {
		return nil
	}
	var out []map[string]any
	for _, line := range splitLines(string(raw)) {
		var rec map[string]any
		if jsonUnmarshal([]byte(line), &rec) == nil {
			out = append(out, rec)
		}
	}
	return out
}

// FindDelegationRecord returns the payload of the delegation-audit record for one request id
// (nil when none exists - the proof surface for "this delegated request was never admitted").
func (c *Cell) FindDelegationRecord(requestID string) map[string]any {
	for _, rec := range c.DelegationAuditRecords() {
		payload, _ := rec["payload"].(map[string]any)
		if payload == nil {
			continue
		}
		if rid, _ := payload["request_id"].(string); rid == requestID {
			return payload
		}
	}
	return nil
}

// GatewayGov is the gateway principal's own claim in the T06 cell.
func GatewayGov() sfwp.Governance {
	return sfwp.Governance{Actor: sfwp.Actor{ActorID: "gateway", Role: "service"}}
}

// DelegatedPortsGovernance is the ports-spelling delegation for an end user (what the governed
// verbs carry through the application's port).
func DelegatedPortsGovernance(actorID, role string) ports.Governance {
	return ports.Governance{
		Actor:      ports.ActorClaim{ActorID: "gateway", Role: "service"},
		OnBehalfOf: &ports.ActorClaim{ActorID: actorID, Role: role},
	}
}

// DelegateOnBehalfOf returns the governance for a delegated end user.
func DelegateOnBehalfOf(actorID, role string) sfwp.Governance {
	return sfwp.Governance{
		Actor:      sfwp.Actor{ActorID: "gateway", Role: "service"},
		OnBehalfOf: &sfwp.Actor{ActorID: actorID, Role: role},
	}
}

// DelegatedOpts builds governed options for a delegated end-user mutation.
func DelegatedOpts(requestID string) (sfwp.Governance, string) {
	return DelegateOnBehalfOf("operator_local", "operator"), requestID
}

// WaitUntil polls until cond is true or the timeout expires.
func WaitUntil(t *testing.T, timeout time.Duration, what string, cond func() bool) {
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

func splitLines(s string) []string {
	var out []string
	start := 0
	for i := 0; i < len(s); i++ {
		if s[i] == '\n' {
			if line := trimSpace(s[start:i]); line != "" {
				out = append(out, line)
			}
			start = i + 1
		}
	}
	if line := trimSpace(s[start:]); line != "" {
		out = append(out, line)
	}
	return out
}

func trimSpace(s string) string {
	for len(s) > 0 && (s[0] == ' ' || s[0] == '\r' || s[0] == '\t' || s[0] == '\n') {
		s = s[1:]
	}
	for len(s) > 0 && (s[len(s)-1] == ' ' || s[len(s)-1] == '\r' || s[len(s)-1] == '\t' || s[len(s)-1] == '\n') {
		s = s[:len(s)-1]
	}
	return s
}

// jsonUnmarshal is a tiny indirection so the harness does not need the encoding/json import in
// every helper signature.
func jsonUnmarshal(data []byte, v any) error { return json.Unmarshal(data, v) }
