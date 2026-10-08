package server

import (
	"context"
	"encoding/json"
	"errors"
	"reflect"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const (
	runObservationAuthorityTerminalCursor = "authority-terminal-cursor"
	runObservationAuthorityTerminalItem   = "authority-terminal-item"
)

type runObservationAuthorityTerminalSessions struct {
	mu                sync.RWMutex
	states            map[string]auth.CurrentSessionState
	workerGateMu      sync.Mutex
	manager           *runObservationManager
	workerGateID      string
	workerGateEnter   chan struct{}
	workerGateRelease <-chan struct{}
}

func (s *runObservationAuthorityTerminalSessions) Current(sessionID string) (auth.CurrentSessionState, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	state, ok := s.states[sessionID]
	return state, ok
}

func (s *runObservationAuthorityTerminalSessions) gateNextWorkerAuthorization(sessionID string, entered chan struct{}, release <-chan struct{}) {
	s.workerGateMu.Lock()
	defer s.workerGateMu.Unlock()
	s.workerGateID = sessionID
	s.workerGateEnter = entered
	s.workerGateRelease = release
}

func (s *runObservationAuthorityTerminalSessions) waitAtWorkerAuthorization(ctx context.Context, sessionID string) {
	s.workerGateMu.Lock()
	manager := s.manager
	s.workerGateMu.Unlock()
	if manager == nil {
		return
	}
	manager.mu.Lock()
	workerContext := false
	for _, entry := range manager.pollers {
		if ctx == entry.ctx {
			workerContext = true
			break
		}
	}
	manager.mu.Unlock()
	if !workerContext {
		return
	}
	s.workerGateMu.Lock()
	var entered chan struct{}
	var release <-chan struct{}
	if s.workerGateID == sessionID {
		entered = s.workerGateEnter
		release = s.workerGateRelease
		s.workerGateID = ""
		s.workerGateEnter = nil
		s.workerGateRelease = nil
	}
	s.workerGateMu.Unlock()
	if entered != nil && release != nil {
		close(entered)
		<-release
	}
}

func (s *runObservationAuthorityTerminalSessions) add(caller *requestIdentity) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.states[caller.Session.ID] = runObservationManagerCurrent(caller.Identity.Claim()).state
}

func (s *runObservationAuthorityTerminalSessions) revoke(sessionID string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	state := s.states[sessionID]
	state.Claim = ports.ActorClaim{ActorID: "revoked_actor", Role: "operator"}
	s.states[sessionID] = state
}

type runObservationAuthorityTerminalContext struct {
	mu               sync.RWMutex
	cursor           string
	parentIDs        map[string]struct{}
	guardGateMu      sync.Mutex
	guardGateEntered chan struct{}
	guardGateRelease <-chan struct{}
}

func (c *runObservationAuthorityTerminalContext) guard(caseID string) (runObservationPresentContext, error) {
	c.guardGateMu.Lock()
	entered, release := c.guardGateEntered, c.guardGateRelease
	if entered != nil {
		c.guardGateEntered = nil
		c.guardGateRelease = nil
	}
	c.guardGateMu.Unlock()
	if entered != nil && release != nil {
		close(entered)
		<-release
	}
	c.mu.RLock()
	defer c.mu.RUnlock()
	if caseID != runObservationFailureCaseID || c.cursor == "" {
		return runObservationPresentContext{}, runObservationUnavailable("present case context is unavailable")
	}
	parents := make(map[string]struct{}, len(c.parentIDs))
	for parent := range c.parentIDs {
		parents[parent] = struct{}{}
	}
	return runObservationPresentContext{caseID: caseID, cursor: c.cursor, parentIDs: parents}, nil
}

func (c *runObservationAuthorityTerminalContext) gateNextGuard(entered chan struct{}, release <-chan struct{}) {
	c.guardGateMu.Lock()
	defer c.guardGateMu.Unlock()
	c.guardGateEntered = entered
	c.guardGateRelease = release
}

func (c *runObservationAuthorityTerminalContext) setCursor(cursor string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.cursor = cursor
}

func newRunObservationAuthorityTerminalManager(
	list runObservationListReader,
	traces ports.RunTracePort,
	callers ...*requestIdentity,
) (*runObservationManager, *runObservationAuthorityTerminalSessions, *runObservationAuthorityTerminalContext) {
	sessions := &runObservationAuthorityTerminalSessions{states: make(map[string]auth.CurrentSessionState, len(callers))}
	for _, caller := range callers {
		sessions.add(caller)
	}
	present := &runObservationAuthorityTerminalContext{
		cursor: runObservationAuthorityTerminalCursor,
		parentIDs: map[string]struct{}{
			runObservationAuthorityTerminalItem: {},
		},
	}
	authorize := func(ctx context.Context, caller runObservationCaller, _ string, current runObservationSessionCurrent, perspective PerspectiveVerifier) error {
		sessions.waitAtWorkerAuthorization(ctx, caller.sessionID)
		if ctx.Err() != nil {
			return runObservationUnavailable("authorization context was canceled")
		}
		state, ok := current.Current(caller.sessionID)
		if !ok || state.Claim != caller.claim {
			return runObservationUnavailable("session is no longer current")
		}
		return perspective.VerifyPerspective(ctx, caller.claim)
	}
	manager := newRunObservationManager(
		list,
		traces,
		sessions,
		&runObservationManagerPerspectiveFake{},
		authorize,
		present.guard,
		func() time.Time { return time.Date(2026, time.October, 8, 12, 0, 0, 0, time.UTC) },
	)
	sessions.workerGateMu.Lock()
	sessions.manager = manager
	sessions.workerGateMu.Unlock()
	return manager, sessions, present
}

func runObservationAuthorityTerminalSummary() []ports.RunSummary {
	return []ports.RunSummary{{
		RunID: runObservationFailureRunID, CaseID: runObservationFailureCaseID,
		PlanItemID: runObservationAuthorityTerminalItem, Execution: "active", Settlement: "unsettled",
	}}
}

func runObservationAuthorityTerminalSnapshot(caseID, runID, itemID, execution, settlement string) ports.RunTraceSnapshot {
	return ports.RunTraceSnapshot{
		RunID: runID, CaseID: caseID, PlanItemID: itemID,
		Execution: execution, Settlement: settlement,
		Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0,
	}
}

func runObservationAuthorityTerminalStartPrepare(
	manager *runObservationManager,
	caller *requestIdentity,
) <-chan runObservationPrepareResult {
	result := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, caller)
		result <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	return result
}

func waitRunObservationAuthorityTerminalPrepare(
	t *testing.T,
	result <-chan runObservationPrepareResult,
	what string,
) runObservationPrepareResult {
	t.Helper()
	select {
	case got := <-result:
		return got
	case <-time.After(2 * time.Second):
		t.Fatalf("timed out waiting for %s Prepare result", what)
		return runObservationPrepareResult{}
	}
}

func waitRunObservationAuthorityTerminalRefs(
	t *testing.T,
	manager *runObservationManager,
	key runObservationPollerKey,
	want int,
	what string,
) *runObservationPoller {
	t.Helper()
	var entry *runObservationPoller
	waitForRunObservationFailureState(t, what, func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		entry = manager.pollers[key]
		return entry != nil && len(entry.refs) == want
	})
	return entry
}

func requireRunObservationAuthorityTerminalEmptyOwnership(t *testing.T, manager *runObservationManager, what string) {
	t.Helper()
	manager.mu.Lock()
	cohorts, pollers := len(manager.cohorts), len(manager.pollers)
	manager.mu.Unlock()
	if cohorts != 0 || pollers != 0 {
		t.Fatalf("%s failed Prepare retained cohorts=%d pollers=%d after owned cleanup", what, cohorts, pollers)
	}
}

func requireRunObservationAuthorityTerminalWorkerJoined(t *testing.T, entry *runObservationPoller, what string) {
	t.Helper()
	select {
	case <-entry.workerDone:
	default:
		t.Fatalf("%s returned before its actual worker joined", what)
	}
}

func requireRunObservationAuthorityTerminalFailure(t *testing.T, got runObservationPrepareResult, what string) {
	t.Helper()
	if got.err == nil || apperr.KindOf(got.err) != apperr.KindUnavailable || got.lease != nil ||
		!reflect.DeepEqual(got.event, contract.RunTraceObservationEvent{}) {
		t.Fatalf("%s Prepare = (%+v, %v, %v), want typed unavailable/zero DTO/nil lease", what, got.event, got.lease, got.err)
	}
}

func TestRunObservationManagerAuthorityTerminalFittingTerminalKeepsFinalValueAndStops(t *testing.T) {
	firstCaller := runObservationManagerCaller("authority-terminal-owner")
	secondCaller := runObservationManagerCaller("authority-terminal-waiter")
	readStarted := make(chan struct{}, 1)
	secondReadStarted := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(ctx context.Context, caseID, runID, itemID string) (ports.RunTraceSnapshot, error) {
		switch traceCalls.Add(1) {
		case 1:
			readStarted <- struct{}{}
			<-readRelease
			return runObservationAuthorityTerminalSnapshot(caseID, runID, itemID, "completed", "accepted"), nil
		default:
			secondReadStarted <- struct{}{}
			<-ctx.Done()
			return ports.RunTraceSnapshot{}, ctx.Err()
		}
	})
	manager, _, _ := newRunObservationAuthorityTerminalManager(list, traces, firstCaller, secondCaller)
	cleanupRunObservationManager(t, manager, release)
	ownerResult := runObservationAuthorityTerminalStartPrepare(manager, firstCaller)
	waitRunObservationManagerSignal(t, readStarted, "fitting terminal actual trace call")
	waiterResult := runObservationAuthorityTerminalStartPrepare(manager, secondCaller)
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationAuthorityTerminalItem}
	entry := waitRunObservationAuthorityTerminalRefs(t, manager, key, 2, "second independently authorized lease attachment")
	release()
	owner := waitRunObservationAuthorityTerminalPrepare(t, ownerResult, "terminal owner")
	waiter := waitRunObservationAuthorityTerminalPrepare(t, waiterResult, "terminal shared waiter")
	if owner.err != nil || owner.lease == nil || len(owner.event.Payload.Runs) != 1 {
		t.Fatalf("fitting terminal owner = (%+v, %v, %v), want final DTO and lease", owner.event, owner.lease, owner.err)
	}
	if waiter.err != nil || waiter.lease == nil || len(waiter.event.Payload.Runs) != 1 {
		t.Fatalf("fitting terminal waiter = (%+v, %v, %v), want shared final DTO and lease", waiter.event, waiter.lease, waiter.err)
	}
	for name, result := range map[string]runObservationPrepareResult{"owner": owner, "waiter": waiter} {
		run := result.event.Payload.Runs[0]
		if run.RunID != runObservationFailureRunID || run.PlanItemID != runObservationAuthorityTerminalItem ||
			run.Execution != contract.RunExecutionStanding("completed") || run.Settlement != contract.RunSettlementStanding("accepted") {
			t.Fatalf("fitting terminal %s DTO key=%q/%q execution=%q settlement=%q, want exact completed/accepted final value", name, run.RunID, run.PlanItemID, run.Execution, run.Settlement)
		}
	}
	if owner.event.Payload.HydrationReadBudget.ReadsAttempted != 1 || waiter.event.Payload.HydrationReadBudget.ReadsAttempted != 0 {
		t.Fatalf("terminal read counts owner=%d waiter=%d, want 1/0", owner.event.Payload.HydrationReadBudget.ReadsAttempted, waiter.event.Payload.HydrationReadBudget.ReadsAttempted)
	}
	select {
	case <-entry.workerDone:
	case <-secondReadStarted:
		t.Fatalf("fitting terminal worker started a later actual call; calls=%d", traceCalls.Load())
	case <-time.After(2 * time.Second):
		t.Fatalf("fitting terminal worker did not complete after publishing its final value; calls=%d", traceCalls.Load())
	}
	if traceCalls.Load() != 1 {
		t.Fatalf("fitting terminal actual trace calls=%d, want one after worker completion", traceCalls.Load())
	}
	if manager.claimPollerReadStart(entry) {
		t.Fatal("fitting terminal current admitted a new read claim after worker completion")
	}
	manager.mu.Lock()
	_, ownerRef := entry.refs[owner.lease]
	_, waiterRef := entry.refs[waiter.lease]
	ownerEligible := !owner.lease.draining && owner.lease.pollers[key] == entry && ownerRef
	waiterEligible := !waiter.lease.draining && waiter.lease.pollers[key] == entry && waiterRef
	current := entry.current
	manager.mu.Unlock()
	if !ownerEligible || !waiterEligible {
		t.Fatalf("fitting terminal drained valid cache leases owner=%v waiter=%v", ownerEligible, waiterEligible)
	}
	if current == nil || current.Key != key || current.Availability != retainedCurrent || current.Execution != "completed" || current.Settlement != "accepted" {
		if current == nil {
			t.Fatal("fitting terminal retained current is nil, want exact completed/accepted final value")
		}
		t.Fatalf("fitting terminal retained current key=%q/%q availability=%d execution=%q settlement=%q, want exact completed/accepted final value", current.Key.runID, current.Key.planItemID, current.Availability, current.Execution, current.Settlement)
	}
}

func runObservationAuthorityTerminalCandidate(
	previous *runObservationRetainedState,
	key runObservationPollerKey,
	now time.Time,
) ports.RunTraceSnapshot {
	low, high := 0, runObservationRetainedImageMaxBytes
	makeSnapshot := func(padding int) ports.RunTraceSnapshot {
		return ports.RunTraceSnapshot{
			RunID: key.runID, CaseID: key.caseID, PlanItemID: key.planItemID,
			Execution: "completed", Settlement: "accepted",
			Frames: []ports.RunTraceFrame{{
				EventID: "terminal-" + strings.Repeat("x", padding),
				Kind:    "command_finished", Timestamp: "2026-10-08T12:00:01Z",
			}}, TotalFrameCount: 1,
		}
	}
	for low+1 < high {
		middle := low + (high-low)/2
		candidate := buildRunObservationRetainedCandidate(previous, key, makeSnapshot(middle), now)
		if candidate.Outcome == retainedCandidateAccepted {
			low = middle
		} else {
			high = middle
		}
	}
	snapshot := makeSnapshot(low)
	candidate := buildRunObservationRetainedCandidate(previous, key, snapshot, now)
	if candidate.Outcome != retainedCandidateAccepted || candidate.State == nil || candidate.State.Key != key {
		return ports.RunTraceSnapshot{}
	}
	inner, innerErr := marshalRunObservationRetainedImage(candidate.State)
	if innerErr != nil || len(inner) == 0 || len(inner) > runObservationRetainedImageMaxBytes {
		return ports.RunTraceSnapshot{}
	}
	rawOuter, rawErr := json.Marshal(runObservationPollerImageWithCurrent{
		SchemaVersion: runObservationPollerImageSchemaVersion,
		Phase:         runObservationPollerPhaseRunning,
		InitResult:    runObservationInitializerAccepted,
		Current:       json.RawMessage(inner),
	})
	boundedOuter, boundedErr := marshalRunObservationPollerImage(key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, candidate.State)
	if rawErr != nil || len(rawOuter) <= runObservationRetainedImageMaxBytes || boundedOuter != nil || !errors.Is(boundedErr, errRunObservationPollerImageUnavailable) {
		return ports.RunTraceSnapshot{}
	}
	return snapshot
}

func TestRunObservationManagerAuthorityTerminalRecurringOuterCapRefusalDrainsAfterJoin(t *testing.T) {
	caller := runObservationManagerCaller("authority-terminal-over-cap")
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationAuthorityTerminalItem}
	secondReadStarted := make(chan struct{}, 1)
	secondReadRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(secondReadRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	initial := runObservationAuthorityTerminalSnapshot(key.caseID, key.runID, key.planItemID, "active", "unsettled")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
	})
	var candidate ports.RunTraceSnapshot
	traces := runObservationManagerTraceFunc(func(ctx context.Context, _, _, _ string) (ports.RunTraceSnapshot, error) {
		switch traceCalls.Add(1) {
		case 1:
			return initial, nil
		case 2:
			secondReadStarted <- struct{}{}
			<-secondReadRelease
			return candidate, nil
		default:
			return ports.RunTraceSnapshot{}, errors.New("unexpected read after terminal refusal")
		}
	})
	manager, _, _ := newRunObservationAuthorityTerminalManager(list, traces, caller)
	cleanupRunObservationManager(t, manager, release)
	prepared := waitRunObservationAuthorityTerminalPrepare(t, runObservationAuthorityTerminalStartPrepare(manager, caller), "initial nonterminal Prepare")
	if prepared.err != nil || prepared.lease == nil || len(prepared.event.Payload.Runs) != 1 || prepared.event.Payload.HydrationReadBudget.ReadsAttempted != 1 {
		t.Fatalf("initial nonterminal Prepare = (%+v, %v, %v), want validated run/lease/one actual read", prepared.event, prepared.lease, prepared.err)
	}
	entry := waitForRunObservationFailurePoller(t, manager, key)
	select {
	case <-secondReadStarted:
	case <-time.After(2 * time.Second):
		t.Fatalf("recurring actual trace call did not start on the manager cadence; calls=%d", traceCalls.Load())
	}
	manager.mu.Lock()
	previous := entry.current
	registeredBeforeReturn := manager.pollers[key] == entry && len(entry.refs) == 1 && !prepared.lease.draining
	manager.mu.Unlock()
	if previous == nil || previous.Execution != "active" || !registeredBeforeReturn {
		t.Fatalf("held recurring call prior=%v registered=%v, want accepted nonterminal current and retained lease", previous != nil, registeredBeforeReturn)
	}
	candidate = runObservationAuthorityTerminalCandidate(previous, key, manager.now())
	if candidate.CaseID != key.caseID || candidate.Execution != "completed" || len(candidate.Frames) != 1 {
		t.Fatal("could not construct an accepted inner terminal candidate whose complete outer image exceeds the fixed cap")
	}
	candidateState := buildRunObservationRetainedCandidate(previous, key, candidate, manager.now())
	inner, innerErr := marshalRunObservationRetainedImage(candidateState.State)
	var rawOuter []byte
	var rawErr error
	if innerErr == nil && candidateState.State != nil {
		rawOuter, rawErr = json.Marshal(runObservationPollerImageWithCurrent{
			SchemaVersion: runObservationPollerImageSchemaVersion,
			Phase:         runObservationPollerPhaseRunning,
			InitResult:    runObservationInitializerAccepted,
			Current:       json.RawMessage(inner),
		})
	}
	boundedOuter, boundedErr := marshalRunObservationPollerImage(key, runObservationPollerPhaseRunning, runObservationInitializerAccepted, candidateState.State)
	if candidateState.Outcome != retainedCandidateAccepted || candidateState.State == nil || candidateState.State.Key != key ||
		innerErr != nil || len(inner) == 0 || len(inner) > runObservationRetainedImageMaxBytes ||
		rawErr != nil || len(rawOuter) <= runObservationRetainedImageMaxBytes || boundedOuter != nil || !errors.Is(boundedErr, errRunObservationPollerImageUnavailable) {
		rawLength := len(rawOuter)
		t.Fatalf("candidate image oracle inner=%d/%v raw_outer=%d/%v bounded_nil=%v bounded_unavailable=%v, want accepted matching inner within cap, raw full outer above cap, canonical nil/sentinel", len(inner), innerErr, rawLength, rawErr, boundedOuter == nil, errors.Is(boundedErr, errRunObservationPollerImageUnavailable))
	}
	manager.mu.Lock()
	_, stillCohortBeforeReturn := manager.cohorts[prepared.lease]
	stillRegisteredBeforeReturn := manager.pollers[key] == entry && stillCohortBeforeReturn && !prepared.lease.draining
	manager.mu.Unlock()
	if !stillRegisteredBeforeReturn {
		t.Fatal("recurring candidate released capacity before the held actual call returned")
	}
	release()
	waitRunObservationManagerSignal(t, entry.workerDone, "actual recurring worker return after terminal refusal")
	if manager.claimPollerReadStart(entry) {
		t.Fatal("terminal outer-image refusal admitted a subsequent read claim")
	}
	waitRunObservationManagerSignal(t, entry.ctx.Done(), "terminal refusal cancellation signal")
	waitRunObservationManagerSignal(t, prepared.lease.leaseDone, "terminal refusal exact lease drain signal")
	waitRunObservationManagerSignal(t, prepared.lease.drainDone, "terminal refusal lease cleanup after worker JOIN")
	manager.mu.Lock()
	current := entry.current
	entryStillRegistered := manager.pollers[key] == entry
	_, cohortStillRegistered := manager.cohorts[prepared.lease]
	manager.mu.Unlock()
	if current == nil || current.Availability != retainedTerminalRetentionFailure || current.Execution != "active" || current.Settlement != "unsettled" {
		if current == nil {
			t.Fatal("terminal refusal current is nil, want safe retained prior marker")
		}
		_, candidateRetained := current.SeenByID[candidate.Frames[0].EventID]
		t.Fatalf("terminal refusal marker availability=%d execution=%q settlement=%q frames=%d ledger=%d candidateRetained=%v, want marker over prior active/unsettled state without candidate data", current.Availability, current.Execution, current.Settlement, len(current.Frames), len(current.SeenByID), candidateRetained)
	}
	if _, disclosed := current.SeenByID[candidate.Frames[0].EventID]; disclosed {
		t.Fatal("unretainable terminal candidate ID entered the published ledger")
	}
	if entryStillRegistered || cohortStillRegistered {
		t.Fatalf("capacity remained after actual worker/lease JOIN: poller=%v cohort=%v", entryStillRegistered, cohortStillRegistered)
	}
	if traceCalls.Load() != 2 {
		t.Fatalf("actual calls after terminal fallback=%d, want initial+one recurring call", traceCalls.Load())
	}
}

func TestRunObservationManagerAuthorityTerminalRevocationBeforeSelectedReadFailsClosed(t *testing.T) {
	caller := runObservationManagerCaller("authority-terminal-before-read")
	listEntered := make(chan struct{}, 1)
	listRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(listRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		listEntered <- struct{}{}
		<-listRelease
		return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		traceCalls.Add(1)
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationAuthorityTerminalItem), nil
	})
	manager, sessions, _ := newRunObservationAuthorityTerminalManager(list, traces, caller)
	cleanupRunObservationManager(t, manager, release)
	result := runObservationAuthorityTerminalStartPrepare(manager, caller)
	waitRunObservationManagerSignal(t, listEntered, "actual list boundary before selected read")
	sessions.revoke(caller.Session.ID)
	release()
	got := waitRunObservationAuthorityTerminalPrepare(t, result, "revoked-before-read")
	requireRunObservationAuthorityTerminalFailure(t, got, "revoked-before-read")
	requireRunObservationAuthorityTerminalEmptyOwnership(t, manager, "revoked-before-read")
	if traceCalls.Load() != 0 {
		t.Fatalf("revoked watcher started %d actual trace reads, want zero", traceCalls.Load())
	}
}

func TestRunObservationManagerAuthorityTerminalRevocationDuringReadPreventsPublication(t *testing.T) {
	caller := runObservationManagerCaller("authority-terminal-during-read")
	readStarted := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		traceCalls.Add(1)
		readStarted <- struct{}{}
		<-readRelease
		return runObservationAuthorityTerminalSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationAuthorityTerminalItem, "active", "unsettled"), nil
	})
	manager, sessions, _ := newRunObservationAuthorityTerminalManager(list, traces, caller)
	cleanupRunObservationManager(t, manager, release)
	result := runObservationAuthorityTerminalStartPrepare(manager, caller)
	waitRunObservationManagerSignal(t, readStarted, "actual held selected trace call")
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationAuthorityTerminalItem}
	entry := waitForRunObservationFailurePoller(t, manager, key)
	sessions.revoke(caller.Session.ID)
	release()
	got := waitRunObservationAuthorityTerminalPrepare(t, result, "revoked-during-read")
	requireRunObservationAuthorityTerminalFailure(t, got, "revoked-during-read")
	requireRunObservationAuthorityTerminalEmptyOwnership(t, manager, "revoked-during-read")
	requireRunObservationAuthorityTerminalWorkerJoined(t, entry, "revoked-during-read")
	manager.mu.Lock()
	current := entry.current
	manager.mu.Unlock()
	if current != nil {
		t.Fatalf("revoked watcher published a retained candidate: %+v", current)
	}
	if traceCalls.Load() != 1 {
		t.Fatalf("revocation-during-read actual calls=%d, want exactly the held call", traceCalls.Load())
	}
}

func TestRunObservationManagerAuthorityTerminalCursorChangeDuringReadPreventsPublication(t *testing.T) {
	caller := runObservationManagerCaller("authority-terminal-cursor-change")
	readStarted := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		traceCalls.Add(1)
		readStarted <- struct{}{}
		<-readRelease
		return runObservationAuthorityTerminalSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationAuthorityTerminalItem, "active", "unsettled"), nil
	})
	manager, _, present := newRunObservationAuthorityTerminalManager(list, traces, caller)
	cleanupRunObservationManager(t, manager, release)
	result := runObservationAuthorityTerminalStartPrepare(manager, caller)
	waitRunObservationManagerSignal(t, readStarted, "actual held trace call before cursor change")
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationAuthorityTerminalItem}
	entry := waitForRunObservationFailurePoller(t, manager, key)
	present.setCursor("authority-terminal-cursor-after-read-start")
	release()
	got := waitRunObservationAuthorityTerminalPrepare(t, result, "changed-cursor-during-read")
	requireRunObservationAuthorityTerminalFailure(t, got, "changed-cursor-during-read")
	requireRunObservationAuthorityTerminalEmptyOwnership(t, manager, "changed-cursor-during-read")
	requireRunObservationAuthorityTerminalWorkerJoined(t, entry, "changed-cursor-during-read")
	manager.mu.Lock()
	current := entry.current
	manager.mu.Unlock()
	if current != nil {
		t.Fatalf("stale-cursor watcher published a retained candidate: %+v", current)
	}
	if traceCalls.Load() != 1 {
		t.Fatalf("cursor-change-during-read actual calls=%d, want exactly the held call", traceCalls.Load())
	}
}

func TestRunObservationManagerAuthorityTerminalInvalidWatcherPreservesSharedSurvivor(t *testing.T) {
	t.Run("during-held-read", func(t *testing.T) {
		invalidCaller := runObservationManagerCaller("authority-terminal-invalid-owner")
		survivorCaller := runObservationManagerCaller("authority-terminal-valid-survivor")
		readStarted := make(chan struct{}, 1)
		readRelease := make(chan struct{})
		var releaseOnce sync.Once
		release := func() { releaseOnce.Do(func() { close(readRelease) }) }
		defer release()
		var traceCalls atomic.Int32
		list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
			return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
		})
		traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
			traceCalls.Add(1)
			readStarted <- struct{}{}
			<-readRelease
			return runObservationAuthorityTerminalSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationAuthorityTerminalItem, "active", "unsettled"), nil
		})
		manager, sessions, _ := newRunObservationAuthorityTerminalManager(list, traces, invalidCaller, survivorCaller)
		cleanupRunObservationManager(t, manager, release)
		invalidResult := runObservationAuthorityTerminalStartPrepare(manager, invalidCaller)
		waitRunObservationManagerSignal(t, readStarted, "one shared actual initializer read")
		survivorResult := runObservationAuthorityTerminalStartPrepare(manager, survivorCaller)
		key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationAuthorityTerminalItem}
		entry := waitRunObservationAuthorityTerminalRefs(t, manager, key, 2, "independent survivor attachment")
		manager.mu.Lock()
		var invalidLease *runObservationLease
		for lease := range entry.refs {
			if lease.caller.sessionID == invalidCaller.Session.ID {
				invalidLease = lease
			}
		}
		manager.mu.Unlock()
		if invalidLease == nil {
			t.Fatal("could not identify invalid watcher's exact retained lease before invalidation")
		}
		sessions.revoke(invalidCaller.Session.ID)
		release()
		invalid := waitRunObservationAuthorityTerminalPrepare(t, invalidResult, "invalid shared initializer owner")
		requireRunObservationAuthorityTerminalFailure(t, invalid, "invalid shared initializer owner")
		survivor := waitRunObservationAuthorityTerminalPrepare(t, survivorResult, "authorized shared survivor")
		if survivor.err != nil || survivor.lease == nil || len(survivor.event.Payload.Runs) != 1 || survivor.event.Payload.HydrationReadBudget.ReadsAttempted != 0 {
			t.Fatalf("authorized shared survivor = (%+v, %v, %v), want cached final DTO/lease/zero shared starts", survivor.event, survivor.lease, survivor.err)
		}
		manager.mu.Lock()
		validEntry := manager.pollers[key] == entry && entry.current != nil && entry.current.Availability == retainedCurrent
		_, invalidCohortPresent := manager.cohorts[invalidLease]
		_, invalidRefPresent := entry.refs[invalidLease]
		_, invalidForwardPresent := invalidLease.pollers[key]
		_, survivorCohortPresent := manager.cohorts[survivor.lease]
		_, survivorRefPresent := entry.refs[survivor.lease]
		survivorRefPresent = survivorRefPresent && survivor.lease.pollers[key] == entry && !survivor.lease.draining
		manager.mu.Unlock()
		if !validEntry || invalidCohortPresent || invalidRefPresent || invalidForwardPresent || !survivorCohortPresent || !survivorRefPresent {
			t.Fatalf("invalid watcher affected shared ownership entry=%v invalidCohort=%v invalidRef=%v invalidForward=%v survivorCohort=%v survivorRef=%v", validEntry, invalidCohortPresent, invalidRefPresent, invalidForwardPresent, survivorCohortPresent, survivorRefPresent)
		}
		if traceCalls.Load() != 1 {
			t.Fatalf("shared initializer actual calls=%d, want one owner call", traceCalls.Load())
		}
	})
	t.Run("invalid-initiator-before-port-survivor-authorizes", func(t *testing.T) {
		invalidCaller := runObservationManagerCaller("authority-terminal-pre-port-invalid-owner")
		survivorCaller := runObservationManagerCaller("authority-terminal-pre-port-valid-survivor")
		invalidWorkerAuthEntered := make(chan struct{}, 1)
		invalidWorkerAuthRelease := make(chan struct{})
		survivorWorkerAuthEntered := make(chan struct{}, 1)
		survivorWorkerAuthRelease := make(chan struct{})
		guardEntered := make(chan struct{}, 1)
		guardRelease := make(chan struct{})
		readStarted := make(chan struct{}, 1)
		readRelease := make(chan struct{})
		var invalidWorkerAuthReleaseOnce, survivorWorkerAuthReleaseOnce, guardReleaseOnce, readReleaseOnce sync.Once
		releaseInvalidWorkerAuth := func() { invalidWorkerAuthReleaseOnce.Do(func() { close(invalidWorkerAuthRelease) }) }
		releaseSurvivorWorkerAuth := func() { survivorWorkerAuthReleaseOnce.Do(func() { close(survivorWorkerAuthRelease) }) }
		releaseGuard := func() { guardReleaseOnce.Do(func() { close(guardRelease) }) }
		releaseRead := func() { readReleaseOnce.Do(func() { close(readRelease) }) }
		release := func() { releaseInvalidWorkerAuth(); releaseSurvivorWorkerAuth(); releaseGuard(); releaseRead() }
		defer release()
		var sessions *runObservationAuthorityTerminalSessions
		var present *runObservationAuthorityTerminalContext
		var listCalls atomic.Int32
		list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
			switch listCalls.Add(1) {
			case 1:
				sessions.gateNextWorkerAuthorization(invalidCaller.Session.ID, invalidWorkerAuthEntered, invalidWorkerAuthRelease)
			case 2:
				sessions.gateNextWorkerAuthorization(survivorCaller.Session.ID, survivorWorkerAuthEntered, survivorWorkerAuthRelease)
			}
			return ports.RunListResult{Runs: runObservationAuthorityTerminalSummary(), UnreadableIDs: []string{}}, nil
		})
		var traceCalls atomic.Int32
		traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
			traceCalls.Add(1)
			readStarted <- struct{}{}
			<-readRelease
			return runObservationAuthorityTerminalSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationAuthorityTerminalItem, "active", "unsettled"), nil
		})
		manager, actualSessions, actualPresent := newRunObservationAuthorityTerminalManager(list, traces, invalidCaller, survivorCaller)
		sessions = actualSessions
		present = actualPresent
		cleanupRunObservationManager(t, manager, release)
		invalidResult := runObservationAuthorityTerminalStartPrepare(manager, invalidCaller)
		gateReached := false
		readObserved := false
		select {
		case <-invalidWorkerAuthEntered:
			gateReached = true
		case <-readStarted:
			readObserved = true
		case <-time.After(2 * time.Second):
			t.Fatal("worker reached neither its existing authorization dependency nor the actual trace port")
		}
		if !gateReached {
			t.Errorf("actual trace port began before the initiating worker entered the existing authorization dependency")
		}
		survivorResult := runObservationAuthorityTerminalStartPrepare(manager, survivorCaller)
		key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationAuthorityTerminalItem}
		entry := waitRunObservationAuthorityTerminalRefs(t, manager, key, 2, "survivor attachment while initiator worker authorization is held")
		manager.mu.Lock()
		var invalidLease *runObservationLease
		for lease := range entry.refs {
			if lease.caller.sessionID == invalidCaller.Session.ID {
				invalidLease = lease
			}
		}
		manager.mu.Unlock()
		if invalidLease == nil {
			t.Fatal("could not identify invalid initiator's exact lease")
		}
		sessions.revoke(invalidCaller.Session.ID)
		// Authorization reads mutable session state only after the worker gate is released.
		releaseInvalidWorkerAuth()
		if !readObserved {
			select {
			case <-survivorWorkerAuthEntered:
				present.gateNextGuard(guardEntered, guardRelease)
				releaseSurvivorWorkerAuth()
				select {
				case <-guardEntered:
					releaseGuard()
				case <-readStarted:
					t.Errorf("actual trace port began before surviving watcher's exact present-context guard")
					readObserved = true
				case <-time.After(2 * time.Second):
					t.Fatal("worker reached neither the surviving watcher guard nor the actual trace port")
				}
			case <-readStarted:
				t.Errorf("actual trace port began before surviving worker entered the existing authorization dependency")
				readObserved = true
			case <-time.After(2 * time.Second):
				t.Fatal("worker reached neither the surviving watcher authorization dependency nor the actual trace port")
			}
		}
		if !readObserved {
			waitRunObservationManagerSignal(t, readStarted, "actual shared read after surviving auth and present-context checks")
		}
		releaseInvalidWorkerAuth()
		releaseSurvivorWorkerAuth()
		releaseGuard()
		releaseRead()
		invalid := waitRunObservationAuthorityTerminalPrepare(t, invalidResult, "pre-port invalid initiator")
		requireRunObservationAuthorityTerminalFailure(t, invalid, "pre-port invalid initiator")
		survivor := waitRunObservationAuthorityTerminalPrepare(t, survivorResult, "pre-port authorized survivor")
		if survivor.err != nil || survivor.lease == nil || len(survivor.event.Payload.Runs) != 1 || survivor.event.Payload.HydrationReadBudget.ReadsAttempted != 0 {
			t.Fatalf("authorized pre-port survivor = (%+v, %v, %v), want valid shared DTO/lease/zero read starts", survivor.event, survivor.lease, survivor.err)
		}
		manager.mu.Lock()
		registeredEntry := manager.pollers[key] == entry
		_, invalidCohortPresent := manager.cohorts[invalidLease]
		_, invalidRefPresent := entry.refs[invalidLease]
		_, invalidForwardPresent := invalidLease.pollers[key]
		_, survivorCohortPresent := manager.cohorts[survivor.lease]
		_, survivorRefPresent := entry.refs[survivor.lease]
		current := entry.current
		currentKeyMatches := current != nil && current.Key == key
		survivorRefPresent = survivorRefPresent && survivor.lease.pollers[key] == entry && !survivor.lease.draining
		manager.mu.Unlock()
		if !registeredEntry || invalidCohortPresent || invalidRefPresent || invalidForwardPresent || !survivorCohortPresent || !survivorRefPresent || current == nil || !currentKeyMatches || current.Availability != retainedCurrent {
			t.Fatalf("pre-port ownership registeredEntry=%v invalidCohort=%v invalidRef=%v invalidForward=%v survivorCohort=%v survivorRef=%v currentKeyMatches=%v currentValid=%v", registeredEntry, invalidCohortPresent, invalidRefPresent, invalidForwardPresent, survivorCohortPresent, survivorRefPresent, currentKeyMatches, current != nil && current.Availability == retainedCurrent)
		}
		if traceCalls.Load() != 1 {
			t.Fatalf("pre-port shared actual calls=%d, want one read after survivor authorization", traceCalls.Load())
		}
	})
}

func TestRunObservationManagerAuthorityTerminalRechecksFinalPrepareHandoff(t *testing.T) {
	for _, tc := range []struct {
		name       string
		invalidate string
		listError  bool
	}{
		{name: "auth-invalid-empty-list", invalidate: "auth"},
		{name: "auth-invalid-unavailable-list", invalidate: "auth", listError: true},
		{name: "cursor-invalid-empty-list", invalidate: "cursor"},
		{name: "cursor-invalid-unavailable-list", invalidate: "cursor", listError: true},
	} {
		t.Run(tc.name, func(t *testing.T) {
			caller := runObservationManagerCaller("authority-terminal-handoff-" + tc.name)
			listEntered := make(chan struct{}, 1)
			listRelease := make(chan struct{})
			var releaseOnce sync.Once
			release := func() { releaseOnce.Do(func() { close(listRelease) }) }
			defer release()
			list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
				listEntered <- struct{}{}
				<-listRelease
				if tc.listError {
					return ports.RunListResult{}, errors.New("controlled unavailable list")
				}
				return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
			})
			var traceCalls atomic.Int32
			traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
				traceCalls.Add(1)
				return ports.RunTraceSnapshot{}, errors.New("empty/unavailable list must not read")
			})
			manager, sessions, present := newRunObservationAuthorityTerminalManager(list, traces, caller)
			cleanupRunObservationManager(t, manager, release)
			result := runObservationAuthorityTerminalStartPrepare(manager, caller)
			waitRunObservationManagerSignal(t, listEntered, "held actual list entry")
			switch tc.invalidate {
			case "auth":
				sessions.revoke(caller.Session.ID)
			case "cursor":
				present.setCursor("authority-terminal-handoff-cursor-changed")
			default:
				t.Fatalf("unknown handoff invalidation %q", tc.invalidate)
			}
			release()
			got := waitRunObservationAuthorityTerminalPrepare(t, result, tc.name)
			requireRunObservationAuthorityTerminalFailure(t, got, tc.name)
			if traceCalls.Load() != 0 {
				t.Fatalf("%s started %d trace reads, want zero", tc.name, traceCalls.Load())
			}
			manager.mu.Lock()
			cohorts, pollers := len(manager.cohorts), len(manager.pollers)
			manager.mu.Unlock()
			if cohorts != 0 || pollers != 0 {
				t.Fatalf("%s failed handoff retained cohorts=%d pollers=%d after owned cleanup", tc.name, cohorts, pollers)
			}
		})
	}
}
