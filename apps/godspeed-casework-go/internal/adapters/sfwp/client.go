// Connection pooling, per-request deadlines, and correlated-outcome recovery for the live adapter.
//
// Multiplexing decision (documented per the task's instruction to read the server's connection
// loop): the server processes each connection strictly sequentially - crates/sea-forge-server/
// src/lib.rs `handle_connection` reads one line, dispatches, awaits the handler, writes exactly one
// response line, and only then reads the next request; the comment on `dispatch_bounded` states
// that "NDJSON replies carry no sequence number, so a client pairs them with requests
// positionally". The reference client (workbench socket.rs) therefore multiplexes with a FIFO
// pending queue per connection. This client takes the degenerate, unambiguous case of that same
// positional contract: ONE in-flight request per pooled connection (write one line, read one
// line), with concurrency provided by a bounded pool of connections instead of by pipelining.
// That keeps pairing trivially correct under deadlines and cancellation, and matches what the
// server actually does per connection rather than relying on write-side buffering.
package sfwp

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"sync"
	"sync/atomic"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// Config configures the client. Zero values take the defaults noted per field.
type Config struct {
	// SocketPath is the authority's Unix socket (loopback transport).
	SocketPath string
	// MaxConns bounds the connection pool. Default 4. One request is in flight per connection.
	MaxConns int
	// RequestTimeout bounds one request when the caller's context carries no deadline. The
	// server's own per-request bound is 10s; the default leaves margin for that.
	RequestTimeout time.Duration
	// ConnectTimeout bounds dialing one connection. Default 5s.
	ConnectTimeout time.Duration
	// RecoveryBudget bounds the reconnect-and-resolve loop that recovers an interrupted
	// correlated outcome through request.get_status. Default 30s.
	RecoveryBudget time.Duration
	// BackoffBase/BackoffMax bound the reconnect backoff (subscription and recovery loops).
	// Defaults 200ms / 5s.
	BackoffBase time.Duration
	BackoffMax  time.Duration
	// SubscribeIdle bounds each silent stretch on the subscription connection before it is
	// treated as dead and re-established from the last cursor. Default 60s.
	SubscribeIdle time.Duration
	// IdleTTL bounds how long a pooled connection may sit idle before acquire retires it
	// (closing it and dialing fresh) instead of handing it out. The authority closes a
	// connection whose next request line exceeds its 10s server timeout, so any connection
	// idle longer than that is dead on arrival: the first request on it fails (for a mutation
	// that means an honest UNAVAILABLE with no kernel record). The default 8s stays below that
	// threshold with margin (T06 fix F6).
	IdleTTL time.Duration
	// Logger receives reconnect/recovery diagnostics. Nil discards them.
	Logger *log.Logger
	// Dial replaces the Unix dialer (tests inject fakes here).
	Dial func(ctx context.Context, socketPath string) (net.Conn, error)
}

func (c *Config) fill() error {
	if c.SocketPath == "" {
		return apperr.New(apperr.KindConfig, "", "config", "the live authority adapter requires a socket path")
	}
	if c.MaxConns <= 0 {
		c.MaxConns = 4
	}
	if c.RequestTimeout <= 0 {
		c.RequestTimeout = 15 * time.Second
	}
	if c.ConnectTimeout <= 0 {
		c.ConnectTimeout = 5 * time.Second
	}
	if c.RecoveryBudget <= 0 {
		c.RecoveryBudget = 30 * time.Second
	}
	if c.BackoffBase <= 0 {
		c.BackoffBase = 200 * time.Millisecond
	}
	if c.BackoffMax <= 0 {
		c.BackoffMax = 5 * time.Second
	}
	if c.SubscribeIdle <= 0 {
		c.SubscribeIdle = 60 * time.Second
	}
	if c.IdleTTL <= 0 {
		c.IdleTTL = 8 * time.Second
	}
	if c.Dial == nil {
		c.Dial = func(ctx context.Context, socketPath string) (net.Conn, error) {
			d := net.Dialer{}
			return d.DialContext(ctx, "unix", socketPath)
		}
	}
	return nil
}

// Client is a bounded pool of request connections to the authority, plus the machinery to recover
// an interrupted correlated outcome after a reconnect. It is safe for concurrent use.
type Client struct {
	cfg Config

	mu      sync.Mutex
	idle    []*conn
	live    int
	closed  bool
	freesig chan struct{}

	reqSeq atomic.Uint64
	pid    int
}

// New builds a client for the configured socket.
func New(cfg Config) (*Client, error) {
	if err := cfg.fill(); err != nil {
		return nil, err
	}
	return &Client{cfg: cfg, freesig: make(chan struct{}, 1), pid: os.Getpid()}, nil
}

// SocketPath reports the configured socket path.
func (c *Client) SocketPath() string { return c.cfg.SocketPath }

func (c *Client) logf(format string, args ...any) {
	if c.cfg.Logger != nil {
		c.cfg.Logger.Printf(format, args...)
	}
}

// NewRequestID mints a correlation id unique across processes and restarts. Every mutation MUST
// carry one before it is sent: it is the only handle an interrupted outcome can be recovered
// through, and the authority refuses a durable mutation without it.
func (c *Client) NewRequestID(verb string) string {
	return fmt.Sprintf("gw-%d-%d-%s", c.pid, c.reqSeq.Add(1), verb)
}

var errClientClosed = apperr.New(apperr.KindUnavailable, "", "client", "the live authority client is closed")

// Close closes every pooled connection. In-flight calls finish against their own connection.
func (c *Client) Close() {
	c.mu.Lock()
	if c.closed {
		c.mu.Unlock()
		return
	}
	c.closed = true
	idle := c.idle
	c.idle = nil
	for _, cn := range idle {
		cn.close()
	}
	c.mu.Unlock()
}

// conn is one Unix-socket connection carrying at most one in-flight request at a time.
type conn struct {
	nc   net.Conn
	br   *bufio.Reader
	mu   sync.Mutex
	dead bool
	// idleAt is when the connection was released back to the pool; zero for a fresh dial.
	idleAt time.Time
}

func (cn *conn) close() {
	cn.dead = true
	cn.nc.Close()
}

// call writes one request line and reads exactly one response line, bounded by the deadline the
// context implies. A call that fails at any point leaves the connection dead: the pairing contract
// is positional, so a partially consumed connection must never be reused.
func (cn *conn) call(ctx context.Context, line []byte, timeout time.Duration) ([]byte, error) {
	cn.mu.Lock()
	defer cn.mu.Unlock()
	if cn.dead {
		return nil, apperr.New(apperr.KindUnavailable, "", "call", "connection is dead")
	}
	deadline := time.Now().Add(timeout)
	if d, ok := ctx.Deadline(); ok && d.Before(deadline) {
		deadline = d
	}
	if err := cn.nc.SetWriteDeadline(deadline); err != nil {
		cn.close()
		return nil, apperr.Wrap(apperr.KindInternal, "", "call", "cannot set write deadline", err)
	}
	if _, err := cn.nc.Write(line); err != nil {
		cn.close()
		return nil, transportErr("write", err)
	}
	if _, err := io.WriteString(cn.nc, "\n"); err != nil {
		cn.close()
		return nil, transportErr("write", err)
	}
	if err := cn.nc.SetReadDeadline(deadline); err != nil {
		cn.close()
		return nil, apperr.Wrap(apperr.KindInternal, "", "call", "cannot set read deadline", err)
	}
	resp, err := cn.br.ReadBytes('\n')
	if err != nil {
		cn.close()
		if errors.Is(err, os.ErrDeadlineExceeded) {
			return nil, apperr.Wrap(apperr.KindUnavailable, "", "call",
				"deadline exceeded while the request was in flight", context.DeadlineExceeded)
		}
		return nil, transportErr("read", err)
	}
	return resp, nil
}

// transportErr classifies a socket-level failure. The outcome of any request that crossed the wire
// before it is UNKNOWN - only the authority's correlation store can settle it.
func transportErr(stage string, err error) *apperr.Error {
	return apperr.Wrap(apperr.KindUnavailable, "", stage,
		"the connection to the governed authority failed", err)
}

func (c *Client) dial(ctx context.Context) (*conn, error) {
	dialCtx, cancel := context.WithTimeout(ctx, c.cfg.ConnectTimeout)
	defer cancel()
	nc, err := c.cfg.Dial(dialCtx, c.cfg.SocketPath)
	if err != nil {
		return nil, apperr.Wrap(apperr.KindUnavailable, "", "connect",
			"cannot reach the governed authority at "+c.cfg.SocketPath, err)
	}
	return &conn{nc: nc, br: bufio.NewReader(nc)}, nil
}

// acquire checks out a connection: an idle one if available (retiring any that idled past the
// IdleTTL - the authority closes connections idle past its request-line timeout, so a stale
// pooled connection must never be handed out), a fresh one while under MaxConns, else it waits
// for a release or the context.
func (c *Client) acquire(ctx context.Context) (*conn, error) {
	for {
		c.mu.Lock()
		if c.closed {
			c.mu.Unlock()
			return nil, errClientClosed
		}
		reaped := false
		for len(c.idle) > 0 {
			cn := c.idle[len(c.idle)-1]
			c.idle = c.idle[:len(c.idle)-1]
			if !cn.idleAt.IsZero() && time.Since(cn.idleAt) < c.cfg.IdleTTL {
				c.mu.Unlock()
				return cn, nil
			}
			cn.close()
			c.live--
			reaped = true
		}
		if reaped {
			// Reaping freed capacity: wake a waiter that can now dial.
			c.signalFree()
		}
		if c.live < c.cfg.MaxConns {
			c.live++
			c.mu.Unlock()
			cn, err := c.dial(ctx)
			if err != nil {
				c.mu.Lock()
				c.live--
				c.mu.Unlock()
				return nil, err
			}
			return cn, nil
		}
		c.mu.Unlock()
		select {
		case <-ctx.Done():
			return nil, apperr.Wrap(apperr.KindUnavailable, "", "acquire",
				"no pooled connection to the governed authority became free", ctx.Err())
		case <-c.freesig:
		}
	}
}

// discard retires a connection that failed mid-call.
func (c *Client) discard(cn *conn) {
	cn.close()
	c.mu.Lock()
	c.live--
	c.mu.Unlock()
	c.signalFree()
}

// release returns a healthy connection to the pool, stamped with its idle time so acquire can
// retire it before the authority's own idle timeout closes it under us.
func (c *Client) release(cn *conn) {
	c.mu.Lock()
	if !c.closed && !cn.dead && len(c.idle) < c.cfg.MaxConns {
		cn.idleAt = time.Now()
		c.idle = append(c.idle, cn)
		c.mu.Unlock()
	} else {
		c.live--
		c.mu.Unlock()
		cn.close()
	}
	c.signalFree()
}

func (c *Client) signalFree() {
	select {
	case c.freesig <- struct{}{}:
	default:
	}
}

// Do sends one request and returns its decoded response. An error frame from the authority is
// returned as an error carrying the refusal (its class reachable via errors.As).
//
// Failure discipline (mirroring the reference bridge):
//   - An inspect (non-mutation) request that hits a transport failure is retried exactly once on a
//     fresh connection; a second failure surfaces as typed unavailable.
//   - A correlated mutation is NEVER re-sent. On a transport failure the client reconnects within
//     the recovery budget and resolves the outcome through request.get_status; what it returns is
//     the authority's own recorded terminal outcome (including its interrupted-failure record
//     after a server restart), or a typed "outcome unresolved" error it could not settle.
//   - An error response from the authority is returned as a decoded Refusal wrapped in the
//     application's error kinds; unknown-verb parse failures from an older server are typed
//     unavailable and are never retried.
func (c *Client) Do(ctx context.Context, req *Request) (*Response, error) {
	line, err := EncodeRequest(req)
	if err != nil {
		return nil, err
	}
	resp, err := c.roundTrip(ctx, req, line)
	if err != nil {
		return nil, err
	}
	if resp.Err != nil && transportRefusalKind(resp.Err.Class) {
		// The authority answered "busy" without side effect: one bounded backoff retry.
		if err := sleepCtx(ctx, c.cfg.BackoffBase); err != nil {
			return nil, err
		}
		if resp, err = c.roundTrip(ctx, req, line); err != nil {
			return nil, err
		}
	}
	if resp.Err != nil {
		return nil, resp.Err.appErr(req.verb)
	}
	return resp, nil
}

func (c *Client) roundTrip(ctx context.Context, req *Request, line []byte) (*Response, error) {
	timeout := c.cfg.RequestTimeout
	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	cn, err := c.acquire(ctx)
	if err != nil {
		return nil, err
	}
	respLine, err := cn.call(ctx, line, timeout)
	if err != nil {
		var typed *apperr.Error
		errors.As(err, &typed)
		c.discard(cn)
		if req.IsMutation() && req.RequestID() != "" && ctx.Err() == nil {
			// The request may have crossed the wire; its outcome is unknown. Resolve it through
			// the authority's correlation store rather than re-sending.
			return c.recoverOutcome(context.WithoutCancel(ctx), req)
		}
		if !req.IsMutation() && ctx.Err() == nil {
			// Inspect verbs have no side effect: one reconnect-retry is safe.
			if cn2, err2 := c.acquire(ctx); err2 == nil {
				respLine2, err3 := cn2.call(ctx, line, timeout)
				if err3 != nil {
					c.discard(cn2)
					return nil, err3
				}
				c.release(cn2)
				return DecodeResponse(respLine2)
			}
		}
		if typed != nil && typed.Kind == apperr.KindUnavailable && errors.Is(err, context.DeadlineExceeded) {
			return nil, apperr.New(apperr.KindUnavailable, "", req.verb,
				"deadline exceeded while the request was in flight; "+
					"for a correlated mutation recover the outcome by its request_id")
		}
		return nil, err
	}
	c.release(cn)
	return DecodeResponse(respLine)
}

// recoverOutcome reconnects (bounded backoff) and resolves a correlated request's outcome through
// request.get_status. Terminal states return the recorded outcome verbatim; "pending" is polled;
// "unknown" is retried briefly and then reported honestly as unresolved (the request never
// crossed the authority's admission boundary, so only the operator can lawfully retry it - with a
// NEW request id).
func (c *Client) recoverOutcome(ctx context.Context, req *Request) (*Response, error) {
	ctx, cancel := context.WithTimeout(ctx, c.cfg.RecoveryBudget)
	defer cancel()

	backoff := c.cfg.BackoffBase
	unknowns := 0
	for {
		if err := ctx.Err(); err != nil {
			return nil, apperr.Wrap(apperr.KindUnavailable, "", "recover", "recovery budget exhausted before the outcome of request "+
				req.RequestID()+" could be resolved (do not re-send it; query request.get_status with its id)", err)
		}
		resp, err := c.Do(ctx, NewRequestGetStatus(req.RequestID()))
		if err != nil {
			// Still unreachable (or busy): back off and keep the budget.
			if apperr.KindOf(err) == apperr.KindUnavailable {
				c.logf("sfwp: recovery of %s still unavailable: %v", req.RequestID(), err)
				if err := sleepCtx(ctx, backoff); err != nil {
					return nil, apperr.Wrap(apperr.KindUnavailable, "", "recover",
						"the governed authority stayed unreachable while resolving request "+req.RequestID(), err)
				}
				backoff = min(backoff*2, c.cfg.BackoffMax)
				continue
			}
			return nil, err
		}
		var status RequestStatusView
		if err := resp.Into(&status); err != nil {
			return nil, err
		}
		switch status.Status {
		case "completed", "failed":
			// Return the authority's recorded outcome as the response to the original call. A
			// recorded failure surfaces as its typed refusal (e.g. request_interrupted after a
			// server restart).
			if len(status.Outcome) == 0 {
				return nil, apperr.New(apperr.KindInternal, "", "recover",
					"request "+req.RequestID()+" is terminal on the authority but recorded no outcome")
			}
			outResp, derr := DecodeResponse(status.Outcome)
			if derr != nil {
				return nil, derr
			}
			if outResp.Err != nil {
				return nil, outResp.Err.appErr(req.verb)
			}
			return outResp, nil
		case "pending":
			c.logf("sfwp: request %s still pending on the authority", req.RequestID())
			if err := sleepCtx(ctx, backoff); err != nil {
				return nil, apperr.Wrap(apperr.KindUnavailable, "", "recover",
					"request "+req.RequestID()+" was still pending when the recovery budget expired", err)
			}
			backoff = min(backoff*2, c.cfg.BackoffMax)
		case "unknown":
			unknowns++
			if unknowns >= 3 {
				return nil, apperr.New(apperr.KindUnavailable, "", "recover",
					"the authority has no record of request "+req.RequestID()+
						" (it never crossed admission); it is safe to retry only with a NEW request id")
			}
			if err := sleepCtx(ctx, backoff); err != nil {
				return nil, apperr.Wrap(apperr.KindUnavailable, "", "recover",
					"request "+req.RequestID()+" stayed unrecorded while the recovery budget expired", err)
			}
		default:
			return nil, apperr.New(apperr.KindInternal, "", "recover",
				"unknown correlation status "+status.Status+" for request "+req.RequestID())
		}
	}
}

// RequestStatus resolves a correlated request's outcome on demand (the explicit recovery entry
// point, mirroring sfwp_request_status in the reference bridge).
func (c *Client) RequestStatus(ctx context.Context, requestID string) (RequestStatusView, error) {
	resp, err := c.Do(ctx, NewRequestGetStatus(requestID))
	if err != nil {
		return RequestStatusView{}, err
	}
	var view RequestStatusView
	if err := resp.Into(&view); err != nil {
		return RequestStatusView{}, err
	}
	return view, nil
}

// Hello negotiates the protocol version.
func (c *Client) Hello(ctx context.Context, clientName string) (HelloView, error) {
	resp, err := c.Do(ctx, NewSystemHello(clientName))
	if err != nil {
		return HelloView{}, err
	}
	var view HelloView
	if err := resp.Into(&view); err != nil {
		return HelloView{}, err
	}
	return view, nil
}

func sleepCtx(ctx context.Context, d time.Duration) error {
	t := time.NewTimer(d)
	defer t.Stop()
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-t.C:
		return nil
	}
}
