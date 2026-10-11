package sfwp

import (
	"context"
	"errors"
	"net"
	"testing"
	"time"
)

// The metrics hook is told the wire verb and the apperr class of a failed Do, and stays silent
// when nothing failed to be told about (T11).
func TestOnErrorReportsVerbAndClassOfFailedCalls(t *testing.T) {
	var gotVerb, gotClass string
	calls := 0
	client, err := New(Config{
		SocketPath:     "/nonexistent/server.sock",
		RequestTimeout: 200 * time.Millisecond,
		RecoveryBudget: 50 * time.Millisecond,
		BackoffBase:    time.Millisecond,
		BackoffMax:     time.Millisecond,
		Dial: func(context.Context, string) (net.Conn, error) {
			return nil, errors.New("connection refused")
		},
		OnError: func(verb, class string) { calls++; gotVerb, gotClass = verb, class },
	})
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	if _, err := client.Do(context.Background(), NewReadinessGet()); err == nil {
		t.Fatal("expected the dial failure to surface")
	}
	if calls != 1 || gotVerb != "readiness_get" || gotClass != "unavailable" {
		t.Fatalf("OnError calls=%d verb=%q class=%q", calls, gotVerb, gotClass)
	}
}
