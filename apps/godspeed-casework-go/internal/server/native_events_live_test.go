//go:build live

package server_test

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livestack"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
)

type nativeConnContextKey struct{}

type nativeEventRequest struct {
	LastQuery    string `json:"last_query"`
	LastEventID  string `json:"last_event_id"`
	RequestCount int    `json:"request_count"`
}

type nativeEventPhase struct {
	name               string
	cell               *livetest.Cell
	stack              *livestack.Stack
	client             *sfwp.Client
	caseID             string
	initialCursor      string
	retention          int
	initialPlanMutated int
	api                http.Handler

	mu            sync.Mutex
	activeStreams map[net.Conn]struct{}
	requests      []nativeEventRequest
	reconnectGate chan struct{}
	releaseOnce   sync.Once
}

type nativeEventHarness struct {
	phases map[string]*nativeEventPhase
}

const nativeProxyDownstreamCloseMarker = "[t08-native-event-proxy]"

type nativeEventPhaseState struct {
	Phase              string               `json:"phase"`
	CaseID             string               `json:"case_id"`
	InitialCursor      string               `json:"initial_cursor"`
	Retention          int                  `json:"retention"`
	Head               string               `json:"head"`
	Oldest             string               `json:"oldest"`
	StoreLen           int                  `json:"store_len"`
	Subscribers        int                  `json:"subscribers"`
	ActiveStreams      int                  `json:"active_streams"`
	EventRequests      []nativeEventRequest `json:"event_requests"`
	ItemActivated      int                  `json:"item_activated"`
	SettlementRecorded int                  `json:"settlement_recorded"`
	ItemCompleted      int                  `json:"item_completed"`
	PlanMutated        int                  `json:"plan_mutated"`
	HeadSummary        string               `json:"head_summary"`
	HeadCaseID         string               `json:"head_case_id"`
}

func TestNativeEventSourceResyncRetryAndRetentionReplay(t *testing.T) {
	if _, err := exec.LookPath("agent-browser"); err != nil {
		t.Fatalf("agent-browser is required for the native browser proof: %v", err)
	}

	phase1 := newNativeEventPhase(t, "phase1", 1)
	phase2 := newNativeEventPhase(t, "phase2", 2)
	if phase1.stack.Store.Len() != 1 || phase1.stack.Store.Oldest() != phase1.initialCursor || phase1.stack.Store.Head() != phase1.initialCursor {
		t.Fatalf("retention-one baseline must be exactly its captured cursor: len=%d oldest=%s head=%s cursor=%s",
			phase1.stack.Store.Len(), phase1.stack.Store.Oldest(), phase1.stack.Store.Head(), phase1.initialCursor)
	}
	if phase2.stack.Store.Len() != 2 || phase2.stack.Store.Head() != phase2.initialCursor || phase2.stack.Store.Oldest() >= phase2.initialCursor {
		t.Fatalf("retention-two baseline must retain the captured head plus its predecessor: len=%d oldest=%s head=%s cursor=%s",
			phase2.stack.Store.Len(), phase2.stack.Store.Oldest(), phase2.stack.Store.Head(), phase2.initialCursor)
	}

	harness := &nativeEventHarness{phases: map[string]*nativeEventPhase{
		phase1.name: phase1,
		phase2.name: phase2,
	}}
	listener, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("acquire isolated loopback listener: %v", err)
	}
	httpServer := &http.Server{
		Handler: harness,
		ConnContext: func(ctx context.Context, conn net.Conn) context.Context {
			return context.WithValue(ctx, nativeConnContextKey{}, conn)
		},
		ReadHeaderTimeout: 5 * time.Second,
	}
	serveDone := make(chan error, 1)
	go func() { serveDone <- httpServer.Serve(listener) }()
	t.Cleanup(func() {
		for _, phase := range harness.phases {
			phase.closeStreams()
		}
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = httpServer.Shutdown(ctx)
		if err := <-serveDone; err != nil && !errors.Is(err, http.ErrServerClosed) {
			t.Errorf("close native-event test server: %v", err)
		}
	})

	repoRoot := livetest.RepoRoot(t)
	uiRoot := filepath.Join(repoRoot, "apps", "godspeed-cognitive-ui")
	vitePort := reserveLoopbackPort(t)
	diagnosticDir, err := os.MkdirTemp(os.TempDir(), "sea-t08-native-diagnostic-")
	if err != nil {
		t.Fatalf("create private native diagnostic directory: %v", err)
	}
	t.Cleanup(func() {
		if t.Failed() {
			t.Logf("preserved native EventSource diagnostics at %s", diagnosticDir)
			return
		}
		_ = os.RemoveAll(diagnosticDir)
	})
	configPath := filepath.Join(diagnosticDir, "vite-native-events.config.mjs")
	config := fmt.Sprintf(`const target = %q
const nativeEventProxy = {
  target,
  configure(proxy) {
    proxy.on("proxyRes", (proxyRes, request, response) => {
      const path = (request.url ?? "").split("?", 1)[0]
      const match = /^\/(phase1|phase2)\/api\/events$/.exec(path)
      if (request.method !== "GET" || !match) return
      let destroyedDownstream = false
      const closeIncompleteEventResponse = (source) => {
        if (destroyedDownstream) return
        if (proxyRes.complete || response.destroyed || response.writableEnded) {
          removeEventResponseListeners()
          return
        }
        destroyedDownstream = true
        console.error("[t08-native-event-proxy] phase=" + match[1] + " source=" + source + " complete=false downstreamDestroyed=true")
        response.destroy()
        removeEventResponseListeners()
      }
      const onUpstreamAborted = () => closeIncompleteEventResponse("aborted")
      const onUpstreamClosed = () => closeIncompleteEventResponse("close")
      const onUpstreamError = () => closeIncompleteEventResponse("error")
      const removeEventResponseListeners = () => {
        proxyRes.removeListener("aborted", onUpstreamAborted)
        proxyRes.removeListener("close", onUpstreamClosed)
        proxyRes.removeListener("error", onUpstreamError)
        proxyRes.removeListener("end", removeEventResponseListeners)
        response.removeListener("close", removeEventResponseListeners)
      }
      proxyRes.once("aborted", onUpstreamAborted)
      proxyRes.once("close", onUpstreamClosed)
      proxyRes.once("error", onUpstreamError)
      proxyRes.once("end", removeEventResponseListeners)
      response.once("close", removeEventResponseListeners)
    })
  }
}
export default {
  root: %s,
  logLevel: "error",
  server: {
    host: "127.0.0.1",
    port: %d,
    strictPort: true,
    proxy: {
      "/phase1": nativeEventProxy,
      "/phase2": nativeEventProxy,
      "/__native-test": { target: %q }
    }
  }
};
	`, "http://"+listener.Addr().String(), strconv.Quote(uiRoot), vitePort, "http://"+listener.Addr().String())
	if err := os.WriteFile(configPath, []byte(config), 0o600); err != nil {
		t.Fatal(err)
	}
	viteLogPath := filepath.Join(diagnosticDir, "vite.stdout-stderr.log")
	viteLog, err := os.Create(viteLogPath)
	if err != nil {
		t.Fatal(err)
	}
	vite := exec.Command(filepath.Join(uiRoot, "node_modules", ".bin", "vite"), "--config", configPath)
	vite.Dir = uiRoot
	vite.Stdout, vite.Stderr = viteLog, viteLog
	if err := vite.Start(); err != nil {
		viteLog.Close()
		t.Fatalf("start private Vite server on 127.0.0.1:%d: %v", vitePort, err)
	}
	viteDone := make(chan error, 1)
	var viteMu sync.Mutex
	viteRunning := true
	go func() {
		err := vite.Wait()
		viteMu.Lock()
		viteRunning = false
		viteMu.Unlock()
		viteDone <- err
	}()
	t.Cleanup(func() {
		viteMu.Lock()
		running := viteRunning
		viteMu.Unlock()
		if running {
			_ = vite.Process.Kill()
			<-viteDone
		}
		_ = viteLog.Close()
	})
	viteURL := fmt.Sprintf("http://127.0.0.1:%d", vitePort)
	waitForVite(t, viteURL, viteDone, viteLog)
	moduleProbe := probeNativeModuleHTTP(viteURL + "/e2e/native-events-live.ts")
	writeNativeDiagnosticJSON(t, diagnosticDir, "module-http-probe.json", moduleProbe)

	session := fmt.Sprintf("sea-t08-native-%d-%d", os.Getpid(), time.Now().UnixNano())
	browserClosed := false
	t.Cleanup(func() {
		if !browserClosed {
			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			if _, err := runNativeBrowser(ctx, session, "close"); err != nil {
				t.Errorf("close isolated browser session during cleanup: %v", err)
			}
		}
	})
	if _, err := runNativeBrowser(context.Background(), session, "open", viteURL+"/"); err != nil {
		t.Fatalf("open the private Vite page in the isolated agent-browser session: %v", err)
	}

	phase1Result := runNativeBrowserPhase(t, session, "phase1", diagnosticDir, &browserClosed)
	assertNativeProxyDownstreamCloses(t, viteLogPath, "phase1", 2)
	assertPhase1KernelEvidence(t, phase1, phase1Result)
	state1 := phase1.state()
	if len(state1.EventRequests) != 2 || state1.ActiveStreams != 0 || state1.Subscribers != 0 {
		t.Fatalf("phase 1 must make one bounded retry then dispose all streams/subscribers: %+v", state1)
	}

	phase2Result := runNativeBrowserPhase(t, session, "phase2", diagnosticDir, &browserClosed)
	assertNativeProxyDownstreamCloses(t, viteLogPath, "phase2", 1)
	assertPhase2KernelEvidence(t, phase2, phase2Result)
	state2 := phase2.state()
	if len(state2.EventRequests) != 2 || state2.ActiveStreams != 0 || state2.Subscribers != 0 {
		t.Fatalf("phase 2 must make exactly one native reconnect and dispose it: %+v", state2)
	}
	if got := state2.EventRequests[1].LastQuery; got != phase2.initialCursor {
		t.Fatalf("native reconnect query cursor = %q, want retained cursor C=%q", got, phase2.initialCursor)
	}
	closeCtx, cancelClose := context.WithTimeout(t.Context(), 5*time.Second)
	_, closeErr := runNativeBrowser(closeCtx, session, "close")
	cancelClose()
	if closeErr != nil {
		t.Fatalf("close isolated browser session: %v", closeErr)
	}
	browserClosed = true
}

type nativeBrowserPhaseResult struct {
	OK                bool   `json:"ok"`
	Phase             string `json:"phase"`
	CaseID            string `json:"case_id"`
	Cursor            string `json:"cursor"`
	InitialCursor     string `json:"initial_cursor"`
	InterruptedCursor string `json:"interrupted_cursor"`
	Events            []struct {
		Type   string `json:"type"`
		Cursor string `json:"cursor"`
	} `json:"events"`
	Errors          []string `json:"errors"`
	RequestsBefore  int      `json:"requests_before"`
	RequestsAfter   int      `json:"requests_after"`
	DurableDelta    int      `json:"durable_delta"`
	ReconnectCursor string   `json:"reconnect_cursor"`
}

type nativeBrowserPhaseEnvelope struct {
	State  string                    `json:"state"`
	Result *nativeBrowserPhaseResult `json:"result,omitempty"`
	Error  string                    `json:"error,omitempty"`
}

func runNativeBrowserPhase(t *testing.T, session, phase, diagnosticDir string, browserClosed *bool) nativeBrowserPhaseResult {
	t.Helper()
	phaseCtx, cancelPhase := context.WithTimeout(t.Context(), 90*time.Second)
	defer cancelPhase()
	// Keep the actual browser promise alive across short CDP calls. A whole phase can
	// outlast agent-browser's per-command Runtime.evaluate timeout; polling this
	// same promise does not change the native EventSource or any phase assertion.
	// The phase context covers startup and every poll together.
	proofKey := fmt.Sprintf("__nativeEventProof_%s_%d_%d", phase, os.Getpid(), time.Now().UnixNano())
	start := fmt.Sprintf(`(() => {
  const key = %q;
  if (Object.prototype.hasOwnProperty.call(window, key)) throw new Error("native proof key already exists");
  window[key] = { state: "pending" };
  import("/e2e/native-events-live.ts").then((module) => module.runNativeEventPhase(%q)).then(
    (result) => { window[key] = { state: "resolved", result }; },
    (error) => {
      const message = error instanceof Error ? String(error.name) + ": " + String(error.message) + (error.stack ? "\\n" + String(error.stack) : "") : String(error);
      window[key] = { state: "rejected", error: message };
    },
  );
  return JSON.stringify({ state: "started" });
})()`, proofKey, phase)
	if _, err := runNativeBrowser(phaseCtx, session, "eval", "--stdin", start); err != nil {
		failNativeBrowserPhase(t, session, phase, diagnosticDir, proofKey, browserClosed, err)
	}

	for phaseCtx.Err() == nil {
		poll := fmt.Sprintf(`(() => {
  const state = window[%q] ?? { state: "missing" };
  if (state.state === "resolved") delete window[%q];
  return JSON.stringify(state);
})()`, proofKey, proofKey)
		output, err := runNativeBrowser(phaseCtx, session, "eval", "--stdin", poll)
		if err != nil {
			failNativeBrowserPhase(t, session, phase, diagnosticDir, proofKey, browserClosed, err)
		}
		var encoded string
		if json.Unmarshal(output, &encoded) == nil {
			output = []byte(encoded)
		}
		var envelope nativeBrowserPhaseEnvelope
		if err := json.Unmarshal(output, &envelope); err != nil {
			failNativeBrowserPhase(t, session, phase, diagnosticDir, proofKey, browserClosed,
				fmt.Errorf("parse browser phase state %q: %w", output, err))
		}
		switch envelope.State {
		case "pending":
			timer := time.NewTimer(200 * time.Millisecond)
			select {
			case <-phaseCtx.Done():
				timer.Stop()
			case <-timer.C:
			}
		case "resolved":
			if envelope.Result == nil || !envelope.Result.OK || envelope.Result.Phase != phase {
				t.Fatalf("browser proof result for %s: %+v", phase, envelope.Result)
			}
			return *envelope.Result
		case "rejected":
			failNativeBrowserPhase(t, session, phase, diagnosticDir, proofKey, browserClosed, errors.New(envelope.Error))
		default:
			failNativeBrowserPhase(t, session, phase, diagnosticDir, proofKey, browserClosed,
				fmt.Errorf("browser phase promise has unexpected state %q", envelope.State))
		}
	}
	phaseErr := fmt.Errorf("browser phase context ended: %w", phaseCtx.Err())
	if errors.Is(phaseCtx.Err(), context.DeadlineExceeded) {
		phaseErr = fmt.Errorf("browser phase promise exceeded 90-second overall deadline: %w", phaseCtx.Err())
	}
	failNativeBrowserPhase(t, session, phase, diagnosticDir, proofKey, browserClosed, phaseErr)
	return nativeBrowserPhaseResult{}
}

func failNativeBrowserPhase(t *testing.T, session, phase, diagnosticDir, proofKey string, browserClosed *bool, cause error) {
	t.Helper()
	poll := fmt.Sprintf(`JSON.stringify(window[%q] ?? { state: "missing" })`, proofKey)
	// Failure diagnostics share one 10-second budget, independent of the 90-second
	// phase budget. Closing the owned browser session then gets a separate bounded
	// cleanup budget so a pending proof promise cannot keep running after failure.
	diagnosticCtx, cancelDiagnostics := context.WithTimeout(context.Background(), 10*time.Second)
	state, stateErr := runNativeBrowser(diagnosticCtx, session, "eval", "--stdin", poll)
	browserDiagnostic, diagnosticErr := runNativeBrowser(diagnosticCtx, session, "eval", "--stdin", nativeModuleDiagnosticScript())
	cancelDiagnostics()
	closeCtx, cancelClose := context.WithTimeout(context.Background(), 5*time.Second)
	_, closeErr := runNativeBrowser(closeCtx, session, "close")
	cancelClose()
	if closeErr != nil {
		t.Errorf("close isolated browser session after %s phase failure: %v", phase, closeErr)
	} else {
		*browserClosed = true
	}
	writeNativeDiagnosticJSON(t, diagnosticDir, phase+"-browser-module-diagnostics.json", map[string]any{
		"phase_runner_error": redactNativeDiagnostic(cause.Error()),
		"phase_runner_state": redactNativeDiagnostic(string(state)),
		"phase_state_error": func() string {
			if stateErr != nil {
				return redactNativeDiagnostic(stateErr.Error())
			}
			return ""
		}(),
		"diagnostic_eval_error": func() string {
			if diagnosticErr != nil {
				return redactNativeDiagnostic(diagnosticErr.Error())
			}
			return ""
		}(),
		"browser_close_error": func() string {
			if closeErr != nil {
				return redactNativeDiagnostic(closeErr.Error())
			}
			return ""
		}(),
		"browser_diagnostic_output": redactNativeDiagnostic(string(browserDiagnostic)),
	})
	writeNativeDiagnosticFile(t, diagnosticDir, phase+"-failure.txt", []byte(redactNativeDiagnostic(cause.Error())))
	t.Fatalf("run %s through the browser's native EventSource path: %v (diagnostics: %s)", phase, cause, diagnosticDir)
}

type nativeModuleHTTPProbe struct {
	URL           string `json:"url"`
	Status        int    `json:"status"`
	ContentType   string `json:"content_type"`
	ContentLength int64  `json:"content_length"`
	BytesRead     int    `json:"bytes_read"`
	Truncated     bool   `json:"truncated"`
	Error         string `json:"error,omitempty"`
	BodyPreview   string `json:"body_preview,omitempty"`
}

func probeNativeModuleHTTP(url string) nativeModuleHTTPProbe {
	probe := nativeModuleHTTPProbe{URL: url}
	client := &http.Client{Timeout: 5 * time.Second}
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		probe.Error = err.Error()
		return probe
	}
	req.Header.Set("Accept", "*/*")
	resp, err := client.Do(req)
	if err != nil {
		probe.Error = err.Error()
		return probe
	}
	defer resp.Body.Close()
	probe.Status = resp.StatusCode
	probe.ContentType = resp.Header.Get("Content-Type")
	probe.ContentLength = resp.ContentLength
	body, err := io.ReadAll(io.LimitReader(resp.Body, 64*1024+1))
	if err != nil {
		probe.Error = err.Error()
	}
	if len(body) > 64*1024 {
		body = body[:64*1024]
		probe.Truncated = true
	}
	probe.BytesRead = len(body)
	if resp.StatusCode >= http.StatusBadRequest || strings.Contains(strings.ToLower(probe.ContentType), "text/html") {
		probe.BodyPreview = string(body)
	}
	return probe
}

func nativeModuleDiagnosticScript() string {
	return `(async () => {
  const errors = [];
  const onWindowError = (event) => errors.push({ type: "window.error", message: String(event.message || ""), source: String(event.filename || "") });
  const onRejection = (event) => errors.push({ type: "unhandledrejection", message: String(event.reason?.stack || event.reason || "") });
  const originalConsoleError = console.error;
  console.error = (...args) => { errors.push({ type: "console.error", message: args.map((arg) => String(arg?.stack || arg)).join(" ") }); originalConsoleError.apply(console, args); };
  window.addEventListener("error", onWindowError);
  window.addEventListener("unhandledrejection", onRejection);
  const moduleURL = new URL("/e2e/native-events-live.ts", location.href).toString();
  let fetchResult;
  try {
    const response = await fetch(moduleURL, { cache: "no-store" });
    const body = await response.text();
    const contentType = response.headers.get("content-type") || "";
    fetchResult = {
      status: response.status,
      ok: response.ok,
      content_type: contentType,
      byte_length: new TextEncoder().encode(body).length,
      body_preview: (!response.ok || /text\/html/i.test(contentType)) ? body.slice(0, 4096) : "",
    };
  } catch (error) {
    fetchResult = { error: String(error?.stack || error) };
  }
  let importResult;
  try {
    const loaded = await import(moduleURL);
    importResult = { exported_phase_type: typeof loaded.runNativeEventPhase };
  } catch (error) {
    importResult = { error: String(error?.stack || error) };
  }
  const resources = performance.getEntriesByType("resource")
    .filter((entry) => entry.name.includes("native-events-live") || entry.name.includes("/src/"))
    .map((entry) => ({ name: entry.name, duration_ms: entry.duration, transfer_size: entry.transferSize, encoded_body_size: entry.encodedBodySize }));
  window.removeEventListener("error", onWindowError);
  window.removeEventListener("unhandledrejection", onRejection);
  console.error = originalConsoleError;
  return JSON.stringify({ url: location.href, module_url: moduleURL, fetch: fetchResult, import: importResult, errors, resources });
})()`
}

func writeNativeDiagnosticJSON(t *testing.T, dir, name string, value any) {
	t.Helper()
	raw, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		t.Logf("encode diagnostic %s: %v", name, err)
		return
	}
	writeNativeDiagnosticFile(t, dir, name, []byte(redactNativeDiagnostic(string(raw))))
}

func writeNativeDiagnosticFile(t *testing.T, dir, name string, value []byte) {
	t.Helper()
	if err := os.WriteFile(filepath.Join(dir, name), value, 0o600); err != nil {
		t.Logf("preserve diagnostic %s: %v", name, err)
	}
}

var nativeSecretPattern = regexp.MustCompile(`(?i)(["']?(cookie|set-cookie|authorization|password|token)["']?\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,;}{]+)`)
var nativeBearerPattern = regexp.MustCompile(`(?i)\bbearer\s+[A-Za-z0-9._~+/-]+=*`)

func redactNativeDiagnostic(value string) string {
	value = nativeSecretPattern.ReplaceAllString(value, `$1"[redacted]"`)
	return nativeBearerPattern.ReplaceAllString(value, "Bearer [redacted]")
}

func runNativeBrowser(ctx context.Context, session string, args ...string) ([]byte, error) {
	var stdin *strings.Reader
	if len(args) == 3 && args[0] == "eval" && args[1] == "--stdin" {
		stdin = strings.NewReader(args[2])
		args = args[:2]
	}
	args = append(args, "--session", session)
	cmdCtx, cancel := context.WithTimeout(ctx, 45*time.Second)
	defer cancel()
	cmd := exec.CommandContext(cmdCtx, "agent-browser", args...)
	if stdin != nil {
		cmd.Stdin = stdin
	}
	output, err := cmd.CombinedOutput()
	if err != nil {
		return output, fmt.Errorf("agent-browser %s: %w (%s)", strings.Join(args[:len(args)-2], " "), err, strings.TrimSpace(string(output)))
	}
	return []byte(strings.TrimSpace(string(output))), nil
}

func newNativeEventPhase(t *testing.T, name string, retention int) *nativeEventPhase {
	t.Helper()
	cell := livetest.NewCell(t)
	preserveCellOnFailure(t, cell)
	stack := livestack.AssembleStackWithRetention(t, cell, retention)
	caseID := stack.CommitSentryChain(t, "gw-t08-native-"+name+"-commit")
	stack.WaitRevision(t, caseID, "")
	waitNativeRelaySettled(t, cell, stack, caseID)
	if retention == 2 && stack.Store.Len() < 2 {
		cursor, ok := stack.Relay.CursorForCase(caseID)
		if !ok {
			t.Fatal("phase 2 setup has no case cursor")
		}
		out := stack.Dispatcher.Handle(context.Background(), nativeAddIntent("gw-t08-native-phase2-warmup", caseID, cursor, "native events retention warmup"))
		if !out.Success {
			t.Fatalf("phase 2 single-frame retention warmup refused: %+v", out.Refusal)
		}
		waitNativeRelaySettled(t, cell, stack, caseID)
	}
	if stack.Store.Len() != retention {
		t.Fatalf("%s setup did not fill its retention window: len=%d retention=%d", name, stack.Store.Len(), retention)
	}
	cursor := stack.Store.Head()
	rev, err := stack.Store.At(cursor)
	if err != nil || rev.CaseID != caseID || cursor == "" || cursor != stack.Relay.Head() {
		t.Fatalf("%s setup cursor is not a retained current kernel revision: cursor=%q relay=%q rev=%+v err=%v",
			name, cursor, stack.Relay.Head(), rev, err)
	}
	var reconnectGate chan struct{}
	if retention == 1 {
		reconnectGate = make(chan struct{})
	}
	return &nativeEventPhase{
		name:               name,
		cell:               cell,
		stack:              stack,
		client:             cell.Client(),
		caseID:             caseID,
		initialCursor:      cursor,
		retention:          retention,
		initialPlanMutated: cell.CountTraceKinds(caseID, "plan_mutated"),
		api:                stack.API.Handler(),
		activeStreams:      map[net.Conn]struct{}{},
		reconnectGate:      reconnectGate,
	}
}

func nativeAddIntent(requestID, caseID, cursor, title string) contract.InteractionIntent {
	return contract.InteractionIntent{
		IntentID:       requestID,
		Kind:           "CONSEQUENTIAL_CASE",
		ActionName:     "ADD_DISCRETIONARY_WORK",
		TargetObjectID: "task_prepare",
		CaseID:         caseID,
		ClientCursor:   cursor,
		Actor:          contract.IntentActor{ActorID: "operator_local", Role: "operator"},
		Parameters: map[string]any{
			"case_id":       caseID,
			"stage_id":      "task_prepare",
			"kind":          "sandboxed_task",
			"title":         title,
			"summary":       "Add the single governed plan mutation used to prove retained native replay.",
			"justification": "The live EventSource replay test needs one authorized plan change while disconnected.",
		},
	}
}

func waitNativeRelaySettled(t *testing.T, cell *livetest.Cell, stack *livestack.Stack, caseID string) {
	t.Helper()
	if !nativeRelaySettled(cell, stack, caseID, 20*time.Second) {
		t.Fatalf("timed out waiting for stable durable events and relay head for %s", caseID)
	}
}

func nativeRelaySettled(cell *livetest.Cell, stack *livestack.Stack, caseID string, timeout time.Duration) bool {
	deadline := time.Now().Add(timeout)
	lastEventCount := -1
	stableSince := time.Time{}
	for time.Now().Before(deadline) {
		events := cell.CaseEvents(caseID)
		head := stack.Store.Head()
		if len(events) == lastEventCount && head != "" && head == stack.Relay.Head() {
			if stableSince.IsZero() {
				stableSince = time.Now()
			} else if time.Since(stableSince) >= 250*time.Millisecond {
				return true
			}
		} else {
			stableSince = time.Time{}
		}
		lastEventCount = len(events)
		time.Sleep(10 * time.Millisecond)
	}
	return false
}

func (p *nativeEventPhase) state() nativeEventPhaseState {
	oldest := p.stack.Store.Oldest()
	head := p.stack.Store.Head()
	state := nativeEventPhaseState{
		Phase:              p.name,
		CaseID:             p.caseID,
		InitialCursor:      p.initialCursor,
		Retention:          p.retention,
		Head:               head,
		Oldest:             oldest,
		StoreLen:           p.stack.Store.Len(),
		Subscribers:        p.stack.Store.SubscriberCount(),
		ItemActivated:      p.cell.CountTraceKinds(p.caseID, "item_activated"),
		SettlementRecorded: p.cell.CountTraceKinds(p.caseID, "settlement_recorded"),
		ItemCompleted:      p.cell.CountTraceKinds(p.caseID, "item_completed"),
		PlanMutated:        p.cell.CountTraceKinds(p.caseID, "plan_mutated"),
	}
	if rev, err := p.stack.Store.At(head); err == nil {
		state.HeadSummary = rev.Summary
		state.HeadCaseID = rev.CaseID
	}
	p.mu.Lock()
	state.ActiveStreams = len(p.activeStreams)
	state.EventRequests = append([]nativeEventRequest{}, p.requests...)
	p.mu.Unlock()
	return state
}

func (p *nativeEventPhase) terminalEventCursor(ctx context.Context) (string, string, error) {
	caseEvents := p.cell.CaseEvents(p.caseID)
	if len(caseEvents) == 0 {
		return "", "", fmt.Errorf("case %s has no durable case trace events", p.caseID)
	}
	tail := caseEvents[len(caseEvents)-1]
	eventID, ok := tail["event_id"].(string)
	kind, kindOK := tail["kind"].(string)
	if !ok || eventID == "" || !kindOK || kind == "" {
		return "", "", fmt.Errorf("durable case trace tail lacks its actual event ID or kind: %+v", tail)
	}

	frames, err := p.client.EventsGetRange(ctx, p.initialCursor, "", 500)
	if err != nil {
		return "", "", fmt.Errorf("read durable kernel event frames after C=%s: %w", p.initialCursor, err)
	}
	for _, frame := range frames {
		if frame.CaseID != p.caseID {
			continue
		}
		var detail struct {
			EventID string `json:"event_id"`
		}
		if err := json.Unmarshal(frame.Detail, &detail); err != nil {
			return "", "", fmt.Errorf("decode durable case event detail at cursor %s: %w", frame.Cursor, err)
		}
		if detail.EventID != eventID {
			continue
		}
		wantKind := "case.trace." + kind
		if frame.Kind != wantKind {
			return "", "", fmt.Errorf("last case trace event %s (%s) was published as %s", eventID, kind, frame.Kind)
		}
		return frame.Cursor, wantKind, nil
	}
	return "", "", fmt.Errorf("last case trace event %s was absent from the first 500 durable event frames after C=%s", eventID, p.initialCursor)
}

func (p *nativeEventPhase) serveAPI(w http.ResponseWriter, r *http.Request) {
	var waitForReconnect <-chan struct{}
	if r.URL.Path == "/api/events" {
		conn, _ := r.Context().Value(nativeConnContextKey{}).(net.Conn)
		p.mu.Lock()
		p.requests = append(p.requests, nativeEventRequest{
			LastQuery:    r.URL.Query().Get("last"),
			LastEventID:  r.Header.Get("Last-Event-ID"),
			RequestCount: len(p.requests) + 1,
		})
		if len(p.requests) == 2 {
			waitForReconnect = p.reconnectGate
		}
		if conn != nil {
			p.activeStreams[conn] = struct{}{}
		}
		p.mu.Unlock()
		if conn != nil {
			defer func() {
				p.mu.Lock()
				delete(p.activeStreams, conn)
				p.mu.Unlock()
			}()
		}
	}
	if waitForReconnect != nil {
		select {
		case <-waitForReconnect:
		case <-r.Context().Done():
			return
		}
	}
	p.api.ServeHTTP(w, r)
}

func (p *nativeEventPhase) closeStreams() int {
	p.mu.Lock()
	conns := make([]net.Conn, 0, len(p.activeStreams))
	for conn := range p.activeStreams {
		conns = append(conns, conn)
	}
	p.mu.Unlock()
	for _, conn := range conns {
		_ = conn.Close()
	}
	return len(conns)
}

func (h *nativeEventHarness) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if strings.HasPrefix(r.URL.Path, "/__native-test/stable-state/") {
		name := strings.TrimPrefix(r.URL.Path, "/__native-test/stable-state/")
		phase := h.phases[name]
		if phase == nil {
			http.NotFound(w, r)
			return
		}
		ctx, cancel := context.WithTimeout(r.Context(), 20*time.Second)
		defer cancel()
		cursor, kind, err := phase.terminalEventCursor(ctx)
		if err != nil {
			http.Error(w, "authoritative terminal case trace frame is unavailable: "+err.Error(), http.StatusBadGateway)
			return
		}
		ticker := time.NewTicker(10 * time.Millisecond)
		defer ticker.Stop()
		for {
			state := phase.state()
			revision, revisionErr := phase.stack.Store.At(cursor)
			observed, observedOK := phase.stack.Relay.CursorForCase(phase.caseID)
			if revisionErr == nil && revision.CaseID == phase.caseID && revision.Summary == kind &&
				observedOK && observed >= cursor && state.Head == cursor && state.Oldest == cursor && state.StoreLen == 1 &&
				state.ItemActivated > 0 && state.SettlementRecorded > 0 && state.ItemCompleted > 0 {
				writeNativeJSON(w, state)
				return
			}
			select {
			case <-ctx.Done():
				http.Error(w, "terminal case event cursor was not retained before the bounded wait ended", http.StatusGatewayTimeout)
				return
			case <-ticker.C:
			}
		}
	}
	if strings.HasPrefix(r.URL.Path, "/__native-test/state/") {
		name := strings.TrimPrefix(r.URL.Path, "/__native-test/state/")
		phase := h.phases[name]
		if phase == nil {
			http.NotFound(w, r)
			return
		}
		writeNativeJSON(w, phase.state())
		return
	}
	if strings.HasPrefix(r.URL.Path, "/__native-test/disconnect/") && r.Method == http.MethodPost {
		name := strings.TrimPrefix(r.URL.Path, "/__native-test/disconnect/")
		phase := h.phases[name]
		if phase == nil {
			http.NotFound(w, r)
			return
		}
		writeNativeJSON(w, map[string]int{"closed": phase.closeStreams()})
		return
	}
	if strings.HasPrefix(r.URL.Path, "/__native-test/release/") && r.Method == http.MethodPost {
		name := strings.TrimPrefix(r.URL.Path, "/__native-test/release/")
		phase := h.phases[name]
		if phase == nil || phase.reconnectGate == nil {
			http.NotFound(w, r)
			return
		}
		if !nativeRelaySettled(phase.cell, phase.stack, phase.caseID, 20*time.Second) {
			http.Error(w, "durable trace and relay head did not settle", http.StatusGatewayTimeout)
			return
		}
		phase.releaseOnce.Do(func() { close(phase.reconnectGate) })
		writeNativeJSON(w, phase.state())
		return
	}
	for name, phase := range h.phases {
		if strings.HasPrefix(r.URL.Path, "/"+name+"/") {
			http.StripPrefix("/"+name, http.HandlerFunc(phase.serveAPI)).ServeHTTP(w, r)
			return
		}
	}
	http.NotFound(w, r)
}

func writeNativeJSON(w http.ResponseWriter, value any) {
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(value)
}

func reserveLoopbackPort(t *testing.T) int {
	t.Helper()
	listener, err := net.Listen("tcp4", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("reserve unique Vite loopback port: %v", err)
	}
	port := listener.Addr().(*net.TCPAddr).Port
	if err := listener.Close(); err != nil {
		t.Fatalf("release reserved Vite port %d: %v", port, err)
	}
	return port
}

func waitForVite(t *testing.T, url string, done <-chan error, logFile *os.File) {
	t.Helper()
	client := &http.Client{Timeout: time.Second}
	deadline := time.Now().Add(30 * time.Second)
	for time.Now().Before(deadline) {
		select {
		case err := <-done:
			_ = logFile.Sync()
			logs, _ := os.ReadFile(logFile.Name())
			t.Fatalf("private Vite server exited before readiness: %v\n%s", err, logs)
		default:
		}
		resp, err := client.Get(url)
		if err == nil {
			_ = resp.Body.Close()
			if resp.StatusCode < 500 {
				return
			}
		}
		time.Sleep(100 * time.Millisecond)
	}
	_ = logFile.Sync()
	logs, _ := os.ReadFile(logFile.Name())
	t.Fatalf("private Vite server did not become ready at %s\n%s", url, logs)
}

func assertNativeProxyDownstreamCloses(t *testing.T, logPath, phase string, want int) {
	t.Helper()
	marker := fmt.Sprintf("%s phase=%s ", nativeProxyDownstreamCloseMarker, phase)
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		log, err := os.ReadFile(logPath)
		if err != nil {
			t.Fatalf("read private Vite proxy close diagnostics: %v", err)
		}
		var lines []string
		for _, line := range strings.Split(string(log), "\n") {
			if strings.Contains(line, marker) {
				lines = append(lines, line)
			}
		}
		if len(lines) == want {
			for _, line := range lines {
				if !strings.Contains(line, "complete=false downstreamDestroyed=true") {
					t.Fatalf("proxy close marker does not prove an incomplete upstream response and downstream destroy: %q", line)
				}
			}
			return
		}
		if len(lines) > want {
			t.Fatalf("private Vite proxy destroyed %d %s EventSource responses, want exactly %d: %q", len(lines), phase, want, lines)
		}
		time.Sleep(10 * time.Millisecond)
	}
	log, _ := os.ReadFile(logPath)
	t.Fatalf("private Vite proxy destroyed %d %s EventSource responses, want exactly %d: %s", strings.Count(string(log), marker), phase, want, log)
}

func assertPhase1KernelEvidence(t *testing.T, phase *nativeEventPhase, result nativeBrowserPhaseResult) {
	t.Helper()
	state := phase.state()
	if result.CaseID != phase.caseID || result.InitialCursor != phase.initialCursor {
		t.Fatalf("phase 1 browser used a different real case/cursor: result=%+v state=%+v", result, state)
	}
	if result.Cursor == "" || result.Cursor <= phase.initialCursor || state.StoreLen != 1 || state.Oldest != result.Cursor || state.Head != result.Cursor {
		t.Fatalf("phase 1 must leave exactly the final retained L: result=%+v state=%+v", result, state)
	}
	if state.ItemActivated != 1 || state.SettlementRecorded < 1 || state.ItemCompleted < 1 {
		t.Fatalf("phase 1 must leave one durable execute/settlement/completion trace: %+v", state)
	}
	if result.InterruptedCursor == "" || result.InterruptedCursor <= phase.initialCursor || result.Cursor <= result.InterruptedCursor {
		t.Fatalf("phase 1 must advance from C to interrupted K and then retained L: %+v", result)
	}
	if result.DurableDelta != 1 || state.PlanMutated != phase.initialPlanMutated+1 || state.HeadSummary != "case.trace.plan_mutated" || state.HeadCaseID != phase.caseID {
		t.Fatalf("phase 1 offline recovery must add exactly one durable plan mutation at L: result=%+v state=%+v", result, state)
	}
	if len(result.Events) != 4 || result.Events[0].Type != "resync_required" || result.Events[0].Cursor != result.InterruptedCursor || result.Events[1].Type != "snapshot" || result.Events[1].Cursor != result.InterruptedCursor || result.Events[2].Type != "resync_required" || result.Events[2].Cursor != result.Cursor || result.Events[3].Type != "snapshot" || result.Events[3].Cursor != result.Cursor {
		t.Fatalf("phase 1 must deliver resync(K), snapshot(K), then resync(L), snapshot(L): %+v", result.Events)
	}
	if len(result.Errors) != 2 || result.RequestsBefore != 1 || result.RequestsAfter != 2 || result.ReconnectCursor != result.InterruptedCursor {
		t.Fatalf("phase 1 must observe two drops, one reconnect from K, then dispose pending retry: %+v", result)
	}
	if len(state.EventRequests) != 2 || state.EventRequests[0].LastQuery != phase.initialCursor || state.EventRequests[1].LastQuery != result.InterruptedCursor {
		t.Fatalf("phase 1 event requests must start at C and reconnect from K: %+v", state.EventRequests)
	}
	rev, err := phase.stack.Store.At(result.Cursor)
	if err != nil || rev.CaseID != phase.caseID || rev.Summary != "case.trace.plan_mutated" {
		t.Fatalf("phase 1 L is not the retained real plan-mutation revision: rev=%+v err=%v", rev, err)
	}
	if _, err := phase.stack.Store.At(result.InterruptedCursor); err == nil {
		t.Fatalf("phase 1 ret1 store unexpectedly retained interrupted K after L=%s", result.Cursor)
	}
}

func assertPhase2KernelEvidence(t *testing.T, phase *nativeEventPhase, result nativeBrowserPhaseResult) {
	t.Helper()
	state := phase.state()
	if result.CaseID != phase.caseID || result.InitialCursor != phase.initialCursor {
		t.Fatalf("phase 2 browser used a different real case/cursor: result=%+v state=%+v", result, state)
	}
	if result.Cursor == "" || result.Cursor <= phase.initialCursor || state.StoreLen != 2 || state.Oldest != phase.initialCursor || state.Head != result.Cursor {
		t.Fatalf("phase 2 must retain C and append exactly one new L: result=%+v state=%+v", result, state)
	}
	if result.DurableDelta != 1 || state.PlanMutated != phase.initialPlanMutated+1 || state.HeadSummary != "case.trace.plan_mutated" || state.HeadCaseID != phase.caseID {
		t.Fatalf("phase 2 must leave one durable plan mutation at L: result=%+v state=%+v", result, state)
	}
	if len(result.Events) != 1 || result.Events[0].Type != "snapshot" || result.Events[0].Cursor != result.Cursor || result.ReconnectCursor != phase.initialCursor {
		t.Fatalf("phase 2 must reconnect from C and deliver only snapshot L without resync: %+v", result)
	}
	if len(result.Errors) != 1 || result.RequestsBefore != 1 || result.RequestsAfter != 2 {
		t.Fatalf("phase 2 must observe one disconnect and exactly one native reconnect: %+v", result)
	}
	if _, err := phase.stack.Store.At(phase.initialCursor); err != nil {
		t.Fatalf("phase 2 must still retain C for replay: %v", err)
	}
	if rev, err := phase.stack.Store.At(result.Cursor); err != nil || rev.Summary != "case.trace.plan_mutated" {
		t.Fatalf("phase 2 L must be the retained plan mutation: rev=%+v err=%v", rev, err)
	}
}

func preserveCellOnFailure(t *testing.T, cell *livetest.Cell) {
	t.Helper()
	t.Cleanup(func() {
		if !t.Failed() {
			return
		}
		cell.Stop()
		dir, err := os.MkdirTemp(os.TempDir(), "sea-t08-f14-failed-cell-")
		if err != nil {
			t.Errorf("preserve failed live cell: %v", err)
			return
		}
		destination := filepath.Join(dir, "cell")
		if err := copyNativeTree(cell.Root(), destination); err != nil {
			t.Errorf("preserve failed live cell at %s: %v", destination, err)
			return
		}
		t.Logf("preserved failed native EventSource proof cell at %s", destination)
	})
}

func copyNativeTree(source, destination string) error {
	return filepath.Walk(source, func(path string, info os.FileInfo, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		relative, err := filepath.Rel(source, path)
		if err != nil {
			return err
		}
		target := filepath.Join(destination, relative)
		if info.IsDir() {
			return os.MkdirAll(target, info.Mode().Perm())
		}
		if info.Mode()&os.ModeSymlink != 0 {
			link, err := os.Readlink(path)
			if err != nil {
				return err
			}
			return os.Symlink(link, target)
		}
		data, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		return os.WriteFile(target, data, info.Mode().Perm())
	})
}
