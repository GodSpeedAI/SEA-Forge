// Package livestack assembles the full T06 live gateway over one livetest cell: the same wiring
// main.go's serveLive performs (T05 client -> authority -> live source -> relay -> intents ->
// HTTP surface). It exists so the -tags live tests in other packages exercise the gateway exactly
// the way production wires it, without import cycles into the packages under test.
package livestack

import (
	"context"
	"log"
	"os"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/adapters/sfwp"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/intents"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/livetest"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server"
)

// NewCell boots a fresh seeded cell (re-exported for livestack callers' convenience).
var NewCell = livetest.NewCell

// Stack is the assembled live gateway over one cell.
type Stack struct {
	Cell       *livetest.Cell
	Client     *sfwp.Client
	Authority  *sfwp.Authority
	Source     *projection.LiveSource
	Store      *projection.Store
	Relay      *server.Relay
	Dispatcher *intents.Handler
	API        *server.Server
	Cancel     func()
}

// AssembleStack wires the full live stack over the cell's kernel.
func AssembleStack(t *testing.T, cell *livetest.Cell) *Stack {
	t.Helper()
	ctx, cancel := context.WithCancel(context.Background())
	t.Cleanup(cancel)

	client := cell.Client()
	authority := sfwp.NewAuthority(client)
	source := projection.NewLiveSource(authority, ports.ActorClaim{ActorID: "gateway", Role: "service"})
	store := projection.NewStore()

	sub, err := client.Subscribe(ctx)
	if err != nil {
		t.Fatalf("subscribe: %v", err)
	}
	relay := server.NewRelay(server.NewSFWPFeed(sub), source, store, server.RelayOptions{
		DefaultActor: ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
		Logger:       log.New(os.Stderr, "livetest-relay: ", 0),
	})
	go relay.Run(ctx)

	dispatcher := intents.NewHandler(authority, relay, authority, intents.Options{
		Gateway:             ports.ActorClaim{ActorID: "gateway", Role: "service"},
		PolicyRef:           "authority/active-policy.json",
		ExecutionTimeoutSec: 60,
	})
	api := server.New(source, dispatcher, source, store, relay, server.Options{
		Perspective: ports.ActorClaim{ActorID: "operator_local", Role: "operator"},
	})

	return &Stack{
		Cell:       cell,
		Client:     client,
		Authority:  authority,
		Source:     source,
		Store:      store,
		Relay:      relay,
		Dispatcher: dispatcher,
		API:        api,
		Cancel:     cancel,
	}
}

// CommitSentryChain commits the T03 sentry-chain template as the delegated operator and returns
// the case id. The preflight digest is fetched live so the commit passes the kernel's own
// precondition check.
func (s *Stack) CommitSentryChain(t *testing.T, requestID string) string {
	t.Helper()
	return s.CommitTemplate(t, "e2e-sentry-chain@0.1.0", map[string]string{
		"dataset_name":  "orders-q3",
		"dataset_label": "t06",
		"max_rows":      "25",
		"out_dir":       "work",
	}, requestID, "")
}

// CommitTemplate commits one template as the delegated operator; when pin is non-nil its digest
// rides as the commit precondition (the kernel verifies it).
func (s *Stack) CommitTemplate(t *testing.T, templateRef string, params map[string]string, requestID string, pinDigest string) string {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()
	draft := ports.CaseDraft{TemplateRef: templateRef, Params: params}
	pin := ports.PreconditionDigest{}
	if pinDigest != "" {
		pin = ports.PreconditionDigest{Present: true, TemplateRef: "template:" + templateRef, ExpectedDigest: pinDigest}
	}
	receipt, err := s.Authority.CommitCase(ctx, draft, pin, ports.GovernedOptions{
		Governance: livetest.DelegatedPortsGovernance("operator_local", "operator"),
		Policy:     "authority/active-policy.json",
		RequestID:  requestID,
	})
	if err != nil {
		t.Fatalf("commit %s: %v", templateRef, err)
	}
	if receipt.CaseID == "" {
		t.Fatalf("commit returned no case id: %+v", receipt)
	}
	return receipt.CaseID
}

// WaitRevision waits until the relay records a revision at or after the given minimum cursor
// (or any revision for the case when minCursor is empty) and returns the case's cursor.
func (s *Stack) WaitRevision(t *testing.T, caseID, minCursor string) string {
	t.Helper()
	livetest.WaitUntil(t, 10*time.Second, "relay revision for "+caseID, func() bool {
		cursor, ok := s.Relay.CursorForCase(caseID)
		return ok && (minCursor == "" || cursor > minCursor)
	})
	cursor, _ := s.Relay.CursorForCase(caseID)
	return cursor
}
