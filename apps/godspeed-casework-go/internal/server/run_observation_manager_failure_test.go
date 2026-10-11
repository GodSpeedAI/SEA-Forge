package server

import (
	"context"
	"errors"
	"reflect"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

const (
	runObservationFailureCaseID = "case_1"
	runObservationFailureRunID  = "run_1"
	runObservationFailureItemID = "item_1"
	runObservationFailureCursor = "cursor_1"
)

func newRunObservationFailureFixture(
	t *testing.T,
	runs []ports.RunSummary,
	read runObservationManagerTraceFunc,
) (*runObservationManager, *requestIdentity, *atomic.Int32, *atomic.Int32) {
	t.Helper()
	var listCalls atomic.Int32
	var readCalls atomic.Int32
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		listCalls.Add(1)
		return ports.RunListResult{Runs: runs, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(ctx context.Context, caseID, runID, itemID string) (ports.RunTraceSnapshot, error) {
		readCalls.Add(1)
		return read(ctx, caseID, runID, itemID)
	})
	parents := make([]string, 0, len(runs))
	for _, run := range runs {
		parents = append(parents, run.PlanItemID)
	}
	caller := runObservationManagerCaller("session-failure")
	manager := newRunObservationManagerFixture(
		list,
		traces,
		runObservationManagerCurrent(caller.Identity.Claim()),
		&runObservationManagerPerspectiveFake{},
		runObservationManagerContext(runObservationFailureCaseID, runObservationFailureCursor, parents...),
	)
	return manager, caller, &listCalls, &readCalls
}

func oneRunObservationFailureSummary(itemID string) []ports.RunSummary {
	return []ports.RunSummary{{
		RunID: runObservationFailureRunID, CaseID: runObservationFailureCaseID,
		PlanItemID: itemID, Execution: "active", Settlement: "unsettled",
	}}
}

func successfulRunObservationFailureSnapshot(caseID, runID, itemID string) ports.RunTraceSnapshot {
	return ports.RunTraceSnapshot{
		RunID: runID, CaseID: caseID, PlanItemID: itemID,
		Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{},
	}
}

func assertRunObservationFailureA(t *testing.T, event contract.RunTraceObservationEvent, reads int) {
	t.Helper()
	payload := event.Payload
	counts := []*int{
		payload.ListedRunCount, payload.SelectedRunCount, payload.ValidatedRunCount,
		payload.UnreadableRunCount, payload.UnavailableRunCount, payload.OmittedRunCount,
	}
	want := []int{1, 1, 0, 0, 1, 0}
	for i, count := range counts {
		if count == nil || *count != want[i] {
			t.Fatalf("A count %d = %v, want %d", i, count, want[i])
		}
	}
	if payload.RunListState != "complete" || payload.ObservationState != "unavailable" || len(payload.Runs) != 0 {
		t.Fatalf("A event = %+v, want successful-list unavailable event with no run rows", payload)
	}
	if payload.HydrationReadBudget.Limit != 8 || payload.HydrationReadBudget.ReadsAttempted != reads {
		t.Fatalf("A read budget = %+v, want limit 8 and %d actual starts", payload.HydrationReadBudget, reads)
	}
}

func waitForRunObservationFailureState(t *testing.T, what string, ready func() bool) {
	t.Helper()
	timer := time.NewTimer(2 * time.Second)
	defer timer.Stop()
	for {
		if ready() {
			return
		}
		select {
		case <-timer.C:
			t.Fatalf("timed out waiting for %s", what)
		default:
			runtime.Gosched()
		}
	}
}

func waitForRunObservationFailurePoller(t *testing.T, manager *runObservationManager, key runObservationPollerKey) *runObservationPoller {
	t.Helper()
	var entry *runObservationPoller
	waitForRunObservationFailureState(t, "reserved poller entry", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		entry = manager.pollers[key]
		return entry != nil
	})
	return entry
}

func TestRunObservationManagerFailureStopWinsBetweenReserveAndLaunch(t *testing.T) {
	manager, requestCaller, _, traceCalls := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationFailureItemID), nil
	})

	caller, err := runObservationCallerFromRequest(requestCaller)
	if err != nil {
		t.Fatalf("runObservationCallerFromRequest() error = %v", err)
	}
	present, err := manager.guard(runObservationFailureCaseID)
	if err != nil {
		t.Fatalf("manager.guard() error = %v", err)
	}
	lease, err := manager.beginPrepareOperation(caller, present.cursor)
	if err != nil || lease == nil {
		t.Fatalf("beginPrepareOperation() = (%v, %v), want reserved creator lease: %v", lease, err, err)
	}
	var finishOnce sync.Once
	finish := func() { finishOnce.Do(func() { manager.finishPrepareOperation(lease) }) }
	defer finish()
	entries, err := manager.reservePollerBatch(lease, oneRunObservationFailureSummary(runObservationFailureItemID), lease.leaseDone)
	if err != nil || len(entries) != 1 {
		t.Fatalf("reservePollerBatch() = (%d, %v), want one owned initializer: %v", len(entries), err, err)
	}
	entry := entries[0]
	if entry.ready == nil || entry.workerDone == nil {
		t.Fatalf("reserved entry lacks real readiness/completion channels: %+v", entry)
	}

	stopResult := make(chan error, 1)
	go func() { stopResult <- manager.stopAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "Stop's real admission transition", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return manager.stopping
	})
	if manager.claimPollerReadStart(entry) {
		t.Fatal("read eligibility claim passed after Stop won before worker launch")
	}
	select {
	case err := <-stopResult:
		t.Fatalf("Stop returned before its reserved creator launched: %v", err)
	default:
	}
	manager.launchPollerBatch(entries)
	finish()
	waitRunObservationManagerSignal(t, entry.ready, "stop-winning initializer readiness")
	waitRunObservationManagerSignal(t, entry.workerDone, "stop-winning worker JOIN")
	if traceCalls.Load() != 0 || entry.current != nil || entry.initResult != runObservationInitializerInvalid {
		t.Fatalf("stop-winning entry read=%d current=%v result=%d, want no read/nil current/code4", traceCalls.Load(), entry.current != nil, entry.initResult)
	}
	if entry.phase != runObservationPollerPhaseStopping {
		t.Fatalf("manager-stop phase=%d, want stopping", entry.phase)
	}
	if err := <-stopResult; err != nil {
		t.Fatalf("Stop after actual launch/JOIN: %v", err)
	}
}

func TestRunObservationManagerFailureDetachWinsBeforeFirstReadUsesDrainingCode4(t *testing.T) {
	manager, requestCaller, _, traceCalls := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationFailureItemID), nil
	})
	caller, err := runObservationCallerFromRequest(requestCaller)
	if err != nil {
		t.Fatalf("runObservationCallerFromRequest() error = %v", err)
	}
	present, err := manager.guard(runObservationFailureCaseID)
	if err != nil {
		t.Fatalf("manager.guard() error = %v", err)
	}
	lease, err := manager.beginPrepareOperation(caller, present.cursor)
	if err != nil || lease == nil {
		t.Fatalf("beginPrepareOperation() = (%v, %v), want reserved creator lease", lease, err)
	}
	var finishOnce sync.Once
	finish := func() { finishOnce.Do(func() { manager.finishPrepareOperation(lease) }) }
	defer finish()
	entries, err := manager.reservePollerBatch(lease, oneRunObservationFailureSummary(runObservationFailureItemID), lease.leaseDone)
	if err != nil || len(entries) != 1 {
		t.Fatalf("reservePollerBatch() = (%d, %v), want one initializing entry", len(entries), err)
	}
	entry := entries[0]
	detachResult := make(chan error, 1)
	go func() { detachResult <- lease.detachAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "lease detach's draining transition", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return lease.draining
	})
	if manager.claimPollerReadStart(entry) {
		t.Fatal("read eligibility claim passed after detach won before worker launch")
	}
	select {
	case err := <-detachResult:
		t.Fatalf("detach returned before its owned initializer launched: %v", err)
	default:
	}
	manager.launchPollerBatch(entries)
	finish()
	waitRunObservationManagerSignal(t, entry.ready, "draining stop-before-read readiness")
	waitRunObservationManagerSignal(t, entry.workerDone, "draining stop-before-read worker JOIN")
	if traceCalls.Load() != 0 || entry.current != nil || entry.initResult != runObservationInitializerInvalid || entry.phase != runObservationPollerPhaseDraining {
		t.Fatalf("detach-winning entry phase=%d result=%d current=%v reads=%d, want draining/code4/nil/zero", entry.phase, entry.initResult, entry.current != nil, traceCalls.Load())
	}
	if err := <-detachResult; err != nil {
		t.Fatalf("detach after actual worker JOIN: %v", err)
	}
}

func TestRunObservationManagerFailureClaimThenStopUsesWorkerContinuation(t *testing.T) {
	manager, requestCaller, _, traceCalls := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationFailureItemID), nil
	})
	caller, err := runObservationCallerFromRequest(requestCaller)
	if err != nil {
		t.Fatalf("runObservationCallerFromRequest() error = %v", err)
	}
	present, err := manager.guard(runObservationFailureCaseID)
	if err != nil {
		t.Fatalf("manager.guard() error = %v", err)
	}
	lease, err := manager.beginPrepareOperation(caller, present.cursor)
	if err != nil || lease == nil {
		t.Fatalf("beginPrepareOperation() = (%v, %v), want creator lease", lease, err)
	}
	var finishOnce sync.Once
	finish := func() { finishOnce.Do(func() { manager.finishPrepareOperation(lease) }) }
	defer finish()
	entries, err := manager.reservePollerBatch(lease, oneRunObservationFailureSummary(runObservationFailureItemID), lease.leaseDone)
	if err != nil || len(entries) != 1 {
		t.Fatalf("reservePollerBatch() = (%d, %v), want one entry", len(entries), err)
	}
	entry := entries[0]
	if !manager.claimPollerReadStart(entry) {
		t.Fatal("read eligibility claim refused before Stop, want a positive production claim")
	}
	stopResult := make(chan error, 1)
	go func() { stopResult <- manager.stopAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "Stop after a winning read claim", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return manager.stopping
	})
	manager.runPollerFromClaim(entry, true)
	finish()
	waitRunObservationManagerSignal(t, entry.ready, "claimed-but-canceled initializer readiness")
	waitRunObservationManagerSignal(t, entry.workerDone, "claimed-but-canceled worker JOIN")
	if traceCalls.Load() != 0 || entry.current != nil || entry.initResult != runObservationInitializerInvalid {
		t.Fatalf("claimed then stopped entry read=%d current=%v result=%d, want no actual read/nil current/code4", traceCalls.Load(), entry.current != nil, entry.initResult)
	}
	if err := <-stopResult; err != nil {
		t.Fatalf("Stop after claimed continuation: %v", err)
	}
}

func TestRunObservationManagerFailureStopJoinsHeldActualRead(t *testing.T) {
	readStarted := make(chan struct{}, 2)
	readCanceled := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	var calls atomic.Int32
	manager, caller, _, _ := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(ctx context.Context, caseID, runID, itemID string) (ports.RunTraceSnapshot, error) {
		if calls.Add(1) == 1 {
			return successfulRunObservationFailureSnapshot(caseID, runID, itemID), nil
		}
		readStarted <- struct{}{}
		<-ctx.Done()
		readCanceled <- struct{}{}
		<-readRelease // Model an actual port call whose client retirement has not returned.
		return ports.RunTraceSnapshot{}, ctx.Err()
	})
	prepared := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, caller)
		prepared <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	got := awaitRunObservationManagerPrepare(t, prepared, "held-read cohort")
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationFailureItemID}
	entry := waitForRunObservationFailurePoller(t, manager, key)
	waitRunObservationManagerSignal(t, readStarted, "later actual trace call")

	stopResult := make(chan error, 1)
	go func() { stopResult <- manager.stopAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "held-read Stop transition", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return manager.stopping
	})
	waitRunObservationManagerSignal(t, readCanceled, "manager cancellation of held actual read")
	manager.mu.Lock()
	stillRegistered := manager.pollers[key] == entry
	manager.mu.Unlock()
	if !stillRegistered {
		t.Fatal("held actual read lost its poller reservation before return/JOIN")
	}
	select {
	case err := <-stopResult:
		t.Fatalf("Stop returned before held actual call retired: %v", err)
	default:
	}
	release()
	if err := <-stopResult; err != nil {
		t.Fatalf("Stop after actual call return/JOIN: %v", err)
	}
	waitRunObservationManagerSignal(t, entry.workerDone, "held-read worker completion")
	manager.mu.Lock()
	stillRegistered = manager.pollers[key] == entry
	manager.mu.Unlock()
	if stillRegistered {
		t.Fatal("poller reservation remained after actual worker JOIN")
	}
	if got.lease == nil || got.err != nil || calls.Load() < 2 {
		t.Fatalf("initial cohort/result/calls = (%v, %v, %d), want initial success and later held read", got.lease, got.err, calls.Load())
	}
}

func TestRunObservationManagerFailureGlobalStopReleasesDistinctPendingLeases(t *testing.T) {
	readStarted := make(chan struct{}, 1)
	readCanceled := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	manager, _, _, _ := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(ctx context.Context, caseID, runID, itemID string) (ports.RunTraceSnapshot, error) {
		readStarted <- struct{}{}
		<-ctx.Done()
		readCanceled <- struct{}{}
		<-readRelease
		return ports.RunTraceSnapshot{}, ctx.Err()
	})
	results := make(chan runObservationPrepareResult, 2)
	prepare := func(session string) context.CancelFunc {
		ctx, cancel := context.WithCancel(context.Background())
		go func() {
			event, lease, err := manager.prepare(ctx, runObservationFailureCaseID, runObservationManagerCaller(session))
			results <- runObservationPrepareResult{event: event, lease: lease, err: err}
		}()
		return cancel
	}
	firstCancel := prepare("session-first")
	defer firstCancel()
	waitRunObservationManagerSignal(t, readStarted, "one shared initial read")
	secondCancel := prepare("session-second")
	defer secondCancel()
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationFailureItemID}
	waitRunObservationManagerRefsOrSemanticFailure(t, manager, key, 2, results)
	manager.mu.Lock()
	entry := manager.pollers[key]
	refs := make([]*runObservationLease, 0, len(entry.refs))
	for lease := range entry.refs {
		refs = append(refs, lease)
	}
	manager.mu.Unlock()
	if len(refs) != 2 || refs[0] == refs[1] {
		t.Fatalf("pending shared poller refs=%d distinct leases=%v, want two distinct owners", len(refs), len(refs) == 2 && refs[0] != refs[1])
	}
	stopResult := make(chan error, 1)
	go func() { stopResult <- manager.stopAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "global Stop transition", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return manager.stopping
	})
	waitRunObservationManagerSignal(t, readCanceled, "shared worker cancellation")
	select {
	case err := <-stopResult:
		t.Fatalf("global Stop returned before actual shared read retired: %v", err)
	default:
	}
	release()
	for range 2 {
		select {
		case got := <-results:
			if got.err == nil || apperr.KindOf(got.err) != apperr.KindUnavailable || got.lease != nil || !reflect.DeepEqual(got.event, contract.RunTraceObservationEvent{}) {
				t.Fatalf("stop-interrupted Prepare = (%+v, %v, %v), want typed failure/zero wrapper/nil lease", got.event, got.lease, got.err)
			}
		case <-time.After(2 * time.Second):
			t.Fatal("global Stop stranded a pending Prepare")
		}
	}
	manager.mu.Lock()
	remainingRefs := len(entry.refs)
	manager.mu.Unlock()
	if remainingRefs != 0 {
		t.Fatalf("global Stop left %d shared poller refs after the two lease rollbacks", remainingRefs)
	}
	if err := <-stopResult; err != nil {
		t.Fatalf("global Stop after two per-lease rollbacks: %v", err)
	}
	if entry.workerDone == nil {
		t.Fatal("shared initializer has no actual worker completion channel")
	}
	waitRunObservationManagerSignal(t, entry.workerDone, "single shared worker JOIN")
}

func TestRunObservationManagerFailureCanceledPrepareLeavesAuthorizedLease(t *testing.T) {
	readStarted := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	var firstCalls atomic.Int32
	manager, _, _, _ := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(ctx context.Context, caseID, runID, itemID string) (ports.RunTraceSnapshot, error) {
		if firstCalls.Add(1) == 1 {
			readStarted <- struct{}{}
			select {
			case <-ctx.Done():
				return ports.RunTraceSnapshot{}, ctx.Err()
			case <-readRelease:
				return successfulRunObservationFailureSnapshot(caseID, runID, itemID), nil
			}
		}
		<-ctx.Done()
		return ports.RunTraceSnapshot{}, ctx.Err()
	})
	firstCtx, cancelFirst := context.WithCancel(context.Background())
	defer cancelFirst()
	firstResult := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(firstCtx, runObservationFailureCaseID, runObservationManagerCaller("session-cancelled"))
		firstResult <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	waitRunObservationManagerSignal(t, readStarted, "initializer owned by first Prepare")
	secondResult := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, runObservationManagerCaller("session-survivor"))
		secondResult <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationFailureItemID}
	waitRunObservationManagerRefsOrSemanticFailure(t, manager, key, 2, secondResult)
	manager.mu.Lock()
	entry := manager.pollers[key]
	manager.mu.Unlock()
	cancelFirst()
	select {
	case got := <-firstResult:
		if got.err == nil || got.lease != nil || !reflect.DeepEqual(got.event, contract.RunTraceObservationEvent{}) {
			t.Fatalf("canceled Prepare = (%+v, %v, %v), want typed failure/zero wrapper/nil lease", got.event, got.lease, got.err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("canceled Prepare did not finish its own rollback")
	}
	manager.mu.Lock()
	remainingRefs := 0
	if manager.pollers[key] == entry {
		remainingRefs = len(entry.refs)
	}
	workerContext := entry.ctx
	manager.mu.Unlock()
	if remainingRefs != 1 || workerContext == nil {
		t.Fatalf("surviving shared refs=%d worker context present=%v, want one and live", remainingRefs, workerContext != nil)
	}
	select {
	case <-workerContext.Done():
		t.Fatal("one Prepare canceled shared work still owned by the other lease")
	default:
	}
	release()
	select {
	case got := <-secondResult:
		if got.err != nil || got.lease == nil || got.event.Payload.ObservationState != "complete" {
			t.Fatalf("surviving Prepare = (%+v, %v, %v), want successful current event and lease", got.event, got.lease, got.err)
		}
		if err := got.lease.detachAndDrain(context.Background()); err != nil {
			t.Fatalf("surviving lease detach/JOIN: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("authorized surviving Prepare remained stranded")
	}
}

func TestRunObservationManagerFailureDetachAndSelectedAReleaseHaveOneOwner(t *testing.T) {
	readStarted := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	manager, requestCaller, _, _ := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		readStarted <- struct{}{}
		<-readRelease
		return ports.RunTraceSnapshot{}, errors.New("controlled initial unavailable result")
	})
	caller, err := runObservationCallerFromRequest(requestCaller)
	if err != nil {
		t.Fatalf("runObservationCallerFromRequest() error = %v", err)
	}
	present, err := manager.guard(runObservationFailureCaseID)
	if err != nil {
		t.Fatalf("manager.guard() error = %v", err)
	}
	lease, err := manager.beginPrepareOperation(caller, present.cursor)
	if err != nil || lease == nil {
		t.Fatalf("beginPrepareOperation() = (%v, %v), want preparing lease", lease, err)
	}
	var finishOnce sync.Once
	finish := func() { finishOnce.Do(func() { manager.finishPrepareOperation(lease) }) }
	defer finish()
	entries, err := manager.reservePollerBatch(lease, oneRunObservationFailureSummary(runObservationFailureItemID), lease.leaseDone)
	if err != nil || len(entries) != 1 {
		t.Fatalf("reservePollerBatch() = (%d, %v), want selected poller", len(entries), err)
	}
	entry := entries[0]
	manager.launchPollerBatch(entries)
	finish()
	waitRunObservationManagerSignal(t, readStarted, "held actual selected-A initializer")
	start := make(chan struct{})
	removed := make(chan bool, 1)
	detached := make(chan error, 1)
	go func() { <-start; removed <- manager.releasePollerRef(lease, entry) }()
	go func() { <-start; detached <- lease.detachAndDrain(context.Background()) }()
	close(start)
	var won bool
	select {
	case won = <-removed:
	case <-time.After(2 * time.Second):
		t.Fatal("selected-A ref release did not resolve")
	}
	release()
	var detachErr error
	select {
	case detachErr = <-detached:
	case <-time.After(2 * time.Second):
		t.Fatal("lease drain did not finish after its actual worker returned")
	}
	if detachErr != nil {
		t.Fatalf("lease detach: %v", detachErr)
	}
	manager.mu.Lock()
	leaseRefPresent := false
	if _, ok := lease.pollers[entry.key]; ok {
		leaseRefPresent = true
	}
	entryRefPresent := false
	if _, ok := entry.refs[lease]; ok {
		entryRefPresent = true
	}
	manager.mu.Unlock()
	if leaseRefPresent || entryRefPresent {
		t.Fatalf("selected-A/detach race left membership: lease=%v poller=%v", leaseRefPresent, entryRefPresent)
	}
	waitRunObservationManagerSignal(t, entry.workerDone, "selected-A/detach worker JOIN")
	if !won {
		t.Log("detach transition won exact membership before selected-A cleanup")
	}
}

func TestRunObservationManagerFailureSharedFirstReadErrorIsSuccessfulA(t *testing.T) {
	readStarted := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	manager, _, listCalls, traceCalls := newRunObservationFailureFixture(t, oneRunObservationFailureSummary(runObservationFailureItemID), func(ctx context.Context, _, _, _ string) (ports.RunTraceSnapshot, error) {
		readStarted <- struct{}{}
		<-readRelease
		return ports.RunTraceSnapshot{}, errors.New("controlled first read failure")
	})
	results := make(chan runObservationPrepareResult, 2)
	startPrepare := func(session string) {
		go func() {
			event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, runObservationManagerCaller(session))
			results <- runObservationPrepareResult{event: event, lease: lease, err: err}
		}()
	}
	startPrepare("session-owner")
	waitRunObservationManagerSignal(t, readStarted, "owner's real first trace read")
	startPrepare("session-waiter")
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: runObservationFailureItemID}
	waitRunObservationManagerRefsOrSemanticFailure(t, manager, key, 2, results)
	manager.mu.Lock()
	entry := manager.pollers[key]
	manager.mu.Unlock()
	if entry == nil || entry.workerDone == nil {
		t.Fatal("shared initial failure has no actual worker/JOIN owner")
	}
	select {
	case <-entry.workerDone:
		t.Fatal("failed actual first read released its entry before returning")
	default:
	}
	release()
	var owner, waiter *runObservationPrepareResult
	for range 2 {
		select {
		case got := <-results:
			if got.err != nil || got.lease == nil {
				t.Fatalf("shared first-read A = (%+v, %v, %v), want successful distinct lease", got.event, got.lease, got.err)
			}
			result := got
			if got.event.Payload.HydrationReadBudget.ReadsAttempted == 1 {
				owner = &result
			} else if got.event.Payload.HydrationReadBudget.ReadsAttempted == 0 {
				waiter = &result
			} else {
				t.Fatalf("shared Prepare ReadsAttempted=%d, want owner 1/waiter 0", got.event.Payload.HydrationReadBudget.ReadsAttempted)
			}
		case <-time.After(2 * time.Second):
			t.Fatal("shared first-read failure stranded a Prepare")
		}
	}
	if owner == nil || waiter == nil || owner.lease == waiter.lease {
		t.Fatal("shared first-read failure did not return two distinct successful leases with owner/waiter counts 1/0")
	}
	assertRunObservationFailureA(t, owner.event, 1)
	assertRunObservationFailureA(t, waiter.event, 0)
	manager.mu.Lock()
	remainingRefs := len(entry.refs)
	ownerRefs, waiterRefs := len(owner.lease.pollers), len(waiter.lease.pollers)
	manager.mu.Unlock()
	if remainingRefs != 0 || ownerRefs != 0 || waiterRefs != 0 {
		t.Fatalf("successful A exact-ref cleanup: entry=%d owner=%d waiter=%d, want all zero", remainingRefs, ownerRefs, waiterRefs)
	}
	waitRunObservationManagerSignal(t, entry.workerDone, "failed initializer worker JOIN")
	manager.mu.Lock()
	stillRegistered := manager.pollers[key] == entry
	manager.mu.Unlock()
	if stillRegistered {
		t.Fatal("A cleanup released poller capacity only before actual worker JOIN")
	}
	if listCalls.Load() != 2 || traceCalls.Load() != 1 {
		t.Fatalf("shared A calls list=%d trace=%d, want two lists and one logical/physical start", listCalls.Load(), traceCalls.Load())
	}
	if err := owner.lease.detachAndDrain(context.Background()); err != nil {
		t.Fatalf("owner's empty successful-A lease detach: %v", err)
	}
	if err := waiter.lease.detachAndDrain(context.Background()); err != nil {
		t.Fatalf("waiter's empty successful-A lease detach: %v", err)
	}
}

func TestRunObservationManagerFailureOversizedNilKeyIsAWithoutAdmission(t *testing.T) {
	itemID := "item_" + strings.Repeat("x", runObservationRetainedImageMaxBytes)
	summary := oneRunObservationFailureSummary(itemID)
	key := runObservationPollerKey{caseID: runObservationFailureCaseID, runID: runObservationFailureRunID, planItemID: itemID}
	if _, err := marshalRunObservationPollerImage(key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil); err == nil {
		t.Fatal("oversized key fixture does not exceed the combined nil-current image bound")
	}
	manager, caller, listCalls, traceCalls := newRunObservationFailureFixture(t, summary, func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{}, errors.New("oversized nil-current key must not be read")
	})
	event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("oversized nil-current image Prepare = (%+v, %v, %v), want successful A and nonnil cohort lease", event, lease, err)
	}
	assertRunObservationFailureA(t, event, 0)
	if len(lease.pollers) != 0 {
		t.Fatalf("oversized selected key lease owns %d poller refs, want none", len(lease.pollers))
	}
	manager.mu.Lock()
	entries := len(manager.pollers)
	cohorts := len(manager.cohorts)
	manager.mu.Unlock()
	if entries != 0 || cohorts != 1 {
		t.Fatalf("oversized nil-current admission entries=%d cohorts=%d, want no entry and one successful lease", entries, cohorts)
	}
	if listCalls.Load() != 1 || traceCalls.Load() != 0 {
		t.Fatalf("oversized-key calls list=%d trace=%d, want one allowed list and zero reads", listCalls.Load(), traceCalls.Load())
	}
	if err := lease.detachAndDrain(context.Background()); err != nil {
		t.Fatalf("oversized A lease detach: %v", err)
	}
}

func waitForRunObservationFailureListEntry(
	t *testing.T,
	entered <-chan struct{},
	result <-chan runObservationPrepareResult,
	what string,
) {
	t.Helper()
	timer := time.NewTimer(2 * time.Second)
	defer timer.Stop()
	select {
	case <-entered:
	case got := <-result:
		t.Fatalf("Prepare returned before actual list entry for %s: event=%+v lease=%v err=%v", what, got.event, got.lease, got.err)
	case <-timer.C:
		t.Fatalf("timed out waiting for actual list entry for %s", what)
	}
}

func TestRunObservationManagerFailureStopJoinsHeldListCreator(t *testing.T) {
	listEntered := make(chan struct{}, 1)
	listCanceled := make(chan struct{}, 1)
	listRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(listRelease) }) }
	var listCalls atomic.Int32
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(ctx context.Context, _ ports.CaseRef) (ports.RunListResult, error) {
		if listCalls.Add(1) == 1 {
			listEntered <- struct{}{}
			select {
			case <-ctx.Done():
				listCanceled <- struct{}{}
			case <-listRelease:
			}
			<-listRelease
		}
		return ports.RunListResult{Runs: oneRunObservationFailureSummary(runObservationFailureItemID), UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		traceCalls.Add(1)
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationFailureItemID), nil
	})
	caller := runObservationManagerCaller("session-held-stop")
	manager := newRunObservationManagerFixture(
		list, traces, runObservationManagerCurrent(caller.Identity.Claim()),
		&runObservationManagerPerspectiveFake{},
		runObservationManagerContext(runObservationFailureCaseID, runObservationFailureCursor, runObservationFailureItemID),
	)
	cleanupRunObservationManager(t, manager, release)
	prepared := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, caller)
		prepared <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	waitForRunObservationFailureListEntry(t, listEntered, prepared, "Stop-held Prepare")

	manager.mu.Lock()
	var lease *runObservationLease
	for current := range manager.cohorts {
		lease = current
	}
	cohortCount := len(manager.cohorts)
	manager.mu.Unlock()
	if cohortCount != 1 || lease == nil || lease.creatorDone == nil {
		t.Fatalf("held Prepare cohorts=%d lease=%v creatorDone=%v, want one registered creator", cohortCount, lease != nil, lease != nil && lease.creatorDone != nil)
	}
	select {
	case <-lease.creatorDone:
		t.Fatal("creatorDone closed while its list callback was held")
	default:
	}

	stopResult := make(chan error, 1)
	go func() { stopResult <- manager.stopAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "held-list Stop admission transition", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return manager.stopping
	})
	waitRunObservationManagerSignal(t, listCanceled, "Stop cancellation of held list callback")
	select {
	case <-prepared:
		t.Fatal("Prepare returned while the canceled list callback remained held")
	default:
	}
	select {
	case err := <-stopResult:
		t.Fatalf("Stop returned before the held list callback was released: %v", err)
	default:
	}
	manager.mu.Lock()
	stillRegistered := len(manager.cohorts) == 1
	_, registeredLease := manager.cohorts[lease]
	manager.mu.Unlock()
	if !stillRegistered || !registeredLease {
		t.Fatal("Stop released the held creator's cohort before list return/JOIN")
	}
	select {
	case <-lease.creatorDone:
		t.Fatal("creatorDone closed before held list callback returned")
	default:
	}

	refusedResult := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, runObservationManagerCaller("session-refused-after-stop"))
		refusedResult <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	var refused runObservationPrepareResult
	select {
	case refused = <-refusedResult:
	case <-time.After(2 * time.Second):
		t.Fatal("post-Stop Prepare did not return while the original list callback remained held")
	}
	refusedEvent, refusedLease, refusedErr := refused.event, refused.lease, refused.err
	if refusedErr == nil || apperr.KindOf(refusedErr) != apperr.KindUnavailable || refusedLease != nil || !reflect.DeepEqual(refusedEvent, contract.RunTraceObservationEvent{}) {
		t.Fatalf("post-Stop Prepare = (%+v, %v, %v), want typed failure/zero wrapper/nil lease", refusedEvent, refusedLease, refusedErr)
	}
	if listCalls.Load() != 1 || traceCalls.Load() != 0 {
		t.Fatalf("post-Stop calls list=%d trace=%d, want no second list and no reads", listCalls.Load(), traceCalls.Load())
	}

	release()
	var got runObservationPrepareResult
	select {
	case got = <-prepared:
	case <-time.After(2 * time.Second):
		t.Fatal("Stop-held Prepare did not return after list release")
	}
	if got.err == nil || apperr.KindOf(got.err) != apperr.KindUnavailable || got.lease != nil || !reflect.DeepEqual(got.event, contract.RunTraceObservationEvent{}) {
		t.Fatalf("Stop-held Prepare = (%+v, %v, %v), want typed failure/zero wrapper/nil lease", got.event, got.lease, got.err)
	}
	select {
	case <-lease.creatorDone:
	default:
		t.Fatal("creatorDone remained open after canceled Prepare returned")
	}
	select {
	case err := <-stopResult:
		if err != nil {
			t.Fatalf("Stop after actual list return/JOIN: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("Stop did not finish after actual creator completion")
	}
	manager.mu.Lock()
	cohorts, pollers := len(manager.cohorts), len(manager.pollers)
	manager.mu.Unlock()
	if cohorts != 0 || pollers != 0 {
		t.Fatalf("Stop left cohorts=%d pollers=%d after creator JOIN", cohorts, pollers)
	}
}

func TestRunObservationManagerFailureDetachJoinsOnlyHeldListCreator(t *testing.T) {
	aEntered := make(chan struct{}, 1)
	aCanceled := make(chan struct{}, 1)
	aRelease := make(chan struct{})
	bEntered := make(chan struct{}, 1)
	bContextDone := make(chan (<-chan struct{}), 1)
	bRelease := make(chan struct{})
	var releaseAOnce, releaseBOnce sync.Once
	releaseA := func() { releaseAOnce.Do(func() { close(aRelease) }) }
	releaseB := func() { releaseBOnce.Do(func() { close(bRelease) }) }
	var listCalls atomic.Int32
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(ctx context.Context, _ ports.CaseRef) (ports.RunListResult, error) {
		switch listCalls.Add(1) {
		case 1:
			aEntered <- struct{}{}
			select {
			case <-ctx.Done():
				aCanceled <- struct{}{}
			case <-aRelease:
			}
			<-aRelease
		case 2:
			bContextDone <- ctx.Done()
			bEntered <- struct{}{}
			<-bRelease
		}
		return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		traceCalls.Add(1)
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationFailureItemID), nil
	})
	caller := runObservationManagerCaller("session-held-detach-a")
	manager := newRunObservationManagerFixture(
		list, traces, runObservationManagerCurrent(caller.Identity.Claim()),
		&runObservationManagerPerspectiveFake{},
		runObservationManagerContext(runObservationFailureCaseID, runObservationFailureCursor),
	)
	cleanupRunObservationManager(t, manager, func() { releaseA(); releaseB() })
	aResult := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, runObservationManagerCaller("session-held-detach-a"))
		aResult <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	waitForRunObservationFailureListEntry(t, aEntered, aResult, "Detach A Prepare")
	manager.mu.Lock()
	var aLease *runObservationLease
	for current := range manager.cohorts {
		aLease = current
	}
	aCohorts := len(manager.cohorts)
	manager.mu.Unlock()
	if aCohorts != 1 || aLease == nil || aLease.creatorDone == nil {
		t.Fatalf("held A cohorts=%d lease=%v creatorDone=%v, want one registered creator", aCohorts, aLease != nil, aLease != nil && aLease.creatorDone != nil)
	}

	bResult := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, runObservationManagerCaller("session-held-detach-b"))
		bResult <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	waitForRunObservationFailureListEntry(t, bEntered, bResult, "Detach B Prepare")
	bDone := <-bContextDone
	if bDone == nil {
		t.Fatal("B list callback received no cancellable request context")
	}
	manager.mu.Lock()
	var bLease *runObservationLease
	for current := range manager.cohorts {
		if current != aLease {
			bLease = current
		}
	}
	cohortCount := len(manager.cohorts)
	manager.mu.Unlock()
	if cohortCount != 2 || bLease == nil || bLease == aLease || bLease.creatorDone == nil {
		t.Fatalf("held A/B cohorts=%d distinct B=%v creatorDone=%v, want two exact leases", cohortCount, bLease != nil && bLease != aLease, bLease != nil && bLease.creatorDone != nil)
	}

	detachA := make(chan error, 1)
	go func() { detachA <- aLease.detachAndDrain(context.Background()) }()
	waitForRunObservationFailureState(t, "A lease drain transition", func() bool {
		manager.mu.Lock()
		defer manager.mu.Unlock()
		return aLease.draining
	})
	waitRunObservationManagerSignal(t, aCanceled, "Detach A cancellation of A list")
	select {
	case <-bDone:
		t.Fatal("Detach A canceled B's independent list context")
	default:
	}
	select {
	case <-aResult:
		t.Fatal("A Prepare returned while its canceled list callback remained held")
	default:
	}
	select {
	case err := <-detachA:
		t.Fatalf("Detach A returned before its held list callback was released: %v", err)
	default:
	}
	select {
	case <-aLease.creatorDone:
		t.Fatal("A creatorDone closed while A list callback remained held")
	default:
	}
	select {
	case <-bLease.creatorDone:
		t.Fatal("B creatorDone closed while B list callback remained held")
	default:
	}

	releaseA()
	var gotA runObservationPrepareResult
	select {
	case gotA = <-aResult:
	case <-time.After(2 * time.Second):
		t.Fatal("A Prepare did not return after its list callback release")
	}
	if gotA.err == nil || apperr.KindOf(gotA.err) != apperr.KindUnavailable || gotA.lease != nil || !reflect.DeepEqual(gotA.event, contract.RunTraceObservationEvent{}) {
		t.Fatalf("detached A Prepare = (%+v, %v, %v), want typed failure/zero wrapper/nil lease", gotA.event, gotA.lease, gotA.err)
	}
	select {
	case err := <-detachA:
		if err != nil {
			t.Fatalf("Detach A after creator completion: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("Detach A did not finish after its actual creator completed")
	}
	select {
	case <-aLease.creatorDone:
	default:
		t.Fatal("A creatorDone remained open after Prepare returned")
	}
	select {
	case <-aLease.drainDone:
	default:
		t.Fatal("A drainDone remained open after Detach returned")
	}
	select {
	case <-bDone:
		t.Fatal("Detach A canceled B while A drained")
	default:
	}
	select {
	case <-bLease.creatorDone:
		t.Fatal("B creatorDone closed while B's list callback remained held")
	default:
	}

	releaseB()
	var gotB runObservationPrepareResult
	select {
	case gotB = <-bResult:
	case <-time.After(2 * time.Second):
		t.Fatal("B Prepare did not return after its list callback release")
	}
	if gotB.err != nil || gotB.lease == nil || gotB.lease != bLease || gotB.event.Payload.RunListState != "complete" || len(gotB.event.Payload.Runs) != 0 {
		t.Fatalf("surviving B Prepare = (%+v, %v, %v), want successful empty-list event and exact lease", gotB.event, gotB.lease, gotB.err)
	}
	select {
	case <-bLease.creatorDone:
	default:
		t.Fatal("B creatorDone remained open after successful Prepare returned")
	}
	detachBContext, cancelDetachB := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancelDetachB()
	detachBResult := make(chan error, 1)
	go func() { detachBResult <- bLease.detachAndDrain(detachBContext) }()
	select {
	case err := <-detachBResult:
		if err != nil {
			t.Fatalf("normal B lease detach/JOIN: %v", err)
		}
	case <-detachBContext.Done():
		t.Fatalf("normal B lease detach/JOIN did not complete: %v", detachBContext.Err())
	}
	if listCalls.Load() != 2 || traceCalls.Load() != 0 {
		t.Fatalf("A/B calls list=%d trace=%d, want two held lists and no reads", listCalls.Load(), traceCalls.Load())
	}
}

func TestRunObservationManagerFailureCallerCancelHeldReadableListDoesNotReserve(t *testing.T) {
	listEntered := make(chan struct{}, 1)
	listCanceled := make(chan struct{}, 1)
	listRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(listRelease) }) }
	var listCalls atomic.Int32
	var traceCalls atomic.Int32
	list := runObservationManagerListFunc(func(ctx context.Context, _ ports.CaseRef) (ports.RunListResult, error) {
		if listCalls.Add(1) == 1 {
			listEntered <- struct{}{}
			select {
			case <-ctx.Done():
				listCanceled <- struct{}{}
			case <-listRelease:
			}
			<-listRelease
			return ports.RunListResult{Runs: oneRunObservationFailureSummary(runObservationFailureItemID), UnreadableIDs: []string{}}, nil
		}
		return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		traceCalls.Add(1)
		return successfulRunObservationFailureSnapshot(runObservationFailureCaseID, runObservationFailureRunID, runObservationFailureItemID), nil
	})
	caller := runObservationManagerCaller("session-canceled-readable-list")
	manager := newRunObservationManagerFixture(
		list, traces, runObservationManagerCurrent(caller.Identity.Claim()),
		&runObservationManagerPerspectiveFake{},
		runObservationManagerContext(runObservationFailureCaseID, runObservationFailureCursor, runObservationFailureItemID),
	)
	callerContext, cancelCaller := context.WithCancel(context.Background())
	cleanupRunObservationManager(t, manager, release, cancelCaller)
	prepared := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(callerContext, runObservationFailureCaseID, caller)
		prepared <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	waitForRunObservationFailureListEntry(t, listEntered, prepared, "caller-canceled readable-list Prepare")
	manager.mu.Lock()
	var lease *runObservationLease
	for current := range manager.cohorts {
		lease = current
	}
	cohortCount := len(manager.cohorts)
	manager.mu.Unlock()
	if cohortCount != 1 || lease == nil || lease.creatorDone == nil {
		t.Fatalf("held canceled Prepare cohorts=%d lease=%v creatorDone=%v, want one registered creator", cohortCount, lease != nil, lease != nil && lease.creatorDone != nil)
	}

	cancelCaller()
	waitRunObservationManagerSignal(t, listCanceled, "caller cancellation observed by actual list callback")
	select {
	case <-prepared:
		t.Fatal("Prepare returned before canceled list callback released its readable candidate")
	default:
	}
	select {
	case <-lease.creatorDone:
		t.Fatal("creatorDone closed while canceled list callback remained held")
	default:
	}
	manager.mu.Lock()
	_, stillRegistered := manager.cohorts[lease]
	cohortCount = len(manager.cohorts)
	manager.mu.Unlock()
	if !stillRegistered || cohortCount != 1 {
		t.Fatalf("held canceled list cohort registered=%v count=%d, want exact cohort retained", stillRegistered, cohortCount)
	}
	if listCalls.Load() != 1 || traceCalls.Load() != 0 {
		t.Fatalf("before list release calls list=%d trace=%d, want one list and zero reads", listCalls.Load(), traceCalls.Load())
	}

	release()
	var got runObservationPrepareResult
	select {
	case got = <-prepared:
	case <-time.After(2 * time.Second):
		t.Fatal("caller-canceled Prepare did not finish after list callback release")
	}
	if got.err == nil || apperr.KindOf(got.err) != apperr.KindUnavailable || got.lease != nil || !reflect.DeepEqual(got.event, contract.RunTraceObservationEvent{}) {
		t.Fatalf("caller-canceled readable-list Prepare = (%+v, %v, %v), want typed cancellation/zero wrapper/nil lease", got.event, got.lease, got.err)
	}
	select {
	case <-lease.creatorDone:
	default:
		t.Fatal("creatorDone remained open after canceled Prepare returned")
	}
	manager.mu.Lock()
	_, remainingCohort := manager.cohorts[lease]
	remainingPollers := len(manager.pollers)
	leasePollers := len(lease.pollers)
	manager.mu.Unlock()
	if remainingCohort || remainingPollers != 0 || leasePollers != 0 || traceCalls.Load() != 0 {
		t.Fatalf("canceled readable candidate retained cohort=%v managerPollers=%d leasePollers=%d reads=%d", remainingCohort, remainingPollers, leasePollers, traceCalls.Load())
	}

	// A new Prepare can reuse the manager only after the failed creator has
	// returned with creatorDone closed and its cohort removed.
	retryResult := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), runObservationFailureCaseID, runObservationManagerCaller("session-after-cancel"))
		retryResult <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	var retry runObservationPrepareResult
	select {
	case retry = <-retryResult:
	case <-time.After(2 * time.Second):
		t.Fatal("Prepare after canceled creator did not complete")
	}
	retryEvent, retryLease, retryErr := retry.event, retry.lease, retry.err
	if retryErr != nil || retryLease == nil || retryEvent.Payload.RunListState != "complete" || len(retryEvent.Payload.Runs) != 0 {
		t.Fatalf("Prepare after canceled creator = (%+v, %v, %v), want successful empty-list lease", retryEvent, retryLease, retryErr)
	}
	select {
	case <-retryLease.creatorDone:
	default:
		t.Fatal("retry creatorDone remained open after successful Prepare returned")
	}
	detachRetryContext, cancelDetachRetry := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancelDetachRetry()
	detachRetryResult := make(chan error, 1)
	go func() { detachRetryResult <- retryLease.detachAndDrain(detachRetryContext) }()
	select {
	case err := <-detachRetryResult:
		if err != nil {
			t.Fatalf("post-cancellation retry detach/JOIN: %v", err)
		}
	case <-detachRetryContext.Done():
		t.Fatalf("post-cancellation retry detach/JOIN did not complete: %v", detachRetryContext.Err())
	}
}
