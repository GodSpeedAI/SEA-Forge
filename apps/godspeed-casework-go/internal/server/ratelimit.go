// Token-bucket rate limiting for POST /api/intents (plan T07): one bucket per session AND one
// per remote IP; both must admit the request. Buckets are bounded (an IP flood cannot grow the
// map without limit) and time-based refills make the limiter self-healing.
package server

import (
	"net"
	"net/http"
	"sync"
	"time"
)

// RateLimitOptions configure the intent limiter. Zero values take the documented defaults
// (60/minute, burst 20) — the same defaults config.RateLimitSection applies.
type RateLimitOptions struct {
	PerMinute int
	Burst     int
}

func (o RateLimitOptions) perMinute() int {
	if o.PerMinute > 0 {
		return o.PerMinute
	}
	return 60
}

func (o RateLimitOptions) burst() float64 {
	if o.Burst > 0 {
		return float64(o.Burst)
	}
	return 20
}

// MaxBuckets bounds the tracked keys (sessions + IPs). On overflow the oldest bucket is evicted;
// a hostile client re-filling the map merely loses its own budget early.
const MaxBuckets = 4096

type bucket struct {
	tokens   float64
	lastSeen time.Time
}

// RateLimiter is the bounded token-bucket set. Safe for concurrent use.
type RateLimiter struct {
	perSecond float64
	burst     float64
	max       int
	now       func() time.Time

	mu      sync.Mutex
	buckets map[string]*bucket
}

// NewRateLimiter builds the limiter. Non-positive perMinute/burst take the defaults.
func NewRateLimiter(perMinute, burst, max int, now func() time.Time) *RateLimiter {
	opts := RateLimitOptions{PerMinute: perMinute, Burst: burst}
	if max <= 0 {
		max = MaxBuckets
	}
	if now == nil {
		now = time.Now
	}
	return &RateLimiter{
		perSecond: float64(opts.perMinute()) / 60.0,
		burst:     opts.burst(),
		max:       max,
		now:       now,
		buckets:   map[string]*bucket{},
	}
}

// Allow consumes one token for key; false means the caller is refused (429).
func (l *RateLimiter) Allow(key string) bool {
	now := l.now()
	l.mu.Lock()
	defer l.mu.Unlock()
	b, ok := l.buckets[key]
	if !ok {
		if len(l.buckets) >= l.max {
			l.evictOldestLocked(now)
		}
		b = &bucket{tokens: l.burst, lastSeen: now}
		l.buckets[key] = b
	}
	// Refill up to the burst, then consume.
	elapsed := now.Sub(b.lastSeen).Seconds()
	b.tokens += elapsed * l.perSecond
	if b.tokens > l.burst {
		b.tokens = l.burst
	}
	b.lastSeen = now
	if b.tokens < 1 {
		return false
	}
	b.tokens--
	return true
}

// Len reports the number of tracked buckets (diagnostics/tests).
func (l *RateLimiter) Len() int {
	l.mu.Lock()
	defer l.mu.Unlock()
	return len(l.buckets)
}

func (l *RateLimiter) evictOldestLocked(now time.Time) {
	oldestKey := ""
	var oldest time.Time
	for k, b := range l.buckets {
		if oldestKey == "" || b.lastSeen.Before(oldest) {
			oldestKey, oldest = k, b.lastSeen
		}
	}
	if oldestKey != "" {
		delete(l.buckets, oldestKey)
	}
}

// rateLimitIntent gates POST /api/intents with a per-session and a per-IP bucket. A refused
// request is a typed 429 and never reaches the dispatcher (the kernel stays untouched).
func (s *Server) rateLimitIntent(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ri := identityFrom(r)
		sessionKey := "anonymous"
		if ri != nil && ri.Session != nil {
			sessionKey = ri.Session.ID
		}
		if ri != nil && ri.Source == "bearer" {
			sessionKey = "bearer:" + ri.Identity.Username
		}
		ipKey := "ip:" + remoteIP(r)
		if !s.intentsLimiter.Allow(sessionKey) {
			s.writeRateLimited(w, "too many intents for this session; retry after the refill window")
			return
		}
		if !s.intentsLimiter.Allow(ipKey) {
			s.writeRateLimited(w, "too many intents from this address; retry after the refill window")
			return
		}
		next(w, r)
	}
}

// rateLimitAsk uses independent session and peer-IP budgets because Ask commits disclosure
// records and has no request correlation key. Authentication and CSRF guards wrap this handler.
func (s *Server) rateLimitAsk(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ri := identityFrom(r)
		sessionKey := "anonymous"
		if ri != nil && ri.Session != nil {
			sessionKey = ri.Session.ID
		}
		if ri != nil && ri.Source == "bearer" {
			sessionKey = "bearer:" + ri.Identity.Username
		}
		if !s.askSessionLimiter.Allow(sessionKey) {
			s.writeRateLimited(w, "too many Ask requests for this session; retry after the refill window")
			return
		}
		if !s.askIPLimiter.Allow("ip:" + remoteIP(r)) {
			s.writeRateLimited(w, "too many Ask requests from this address; retry after the refill window")
			return
		}
		next(w, r)
	}
}

func (s *Server) writeRateLimited(w http.ResponseWriter, note string) {
	w.Header().Set("Retry-After", "1")
	writeTypedError(w, http.StatusTooManyRequests, "rate_limited", note)
}

// remoteIP is the request's remote address without its port. Behind a reverse proxy this is the
// proxy's address by design: the config reference documents that the per-session bucket is the
// effective per-user control and that a proxy must not smuggle spoofable identity headers into
// the limit (X-Forwarded-For is deliberately NOT trusted).
func remoteIP(r *http.Request) string {
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		return r.RemoteAddr
	}
	return host
}
