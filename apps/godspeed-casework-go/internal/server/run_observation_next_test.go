package server

import (
	"context"
	"errors"
	"reflect"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

type runObservationNextResult struct {
	aggregate runObservationNextAggregate
	err       error
}

type runObservationNextAuthContextMarker struct{}

func runObservationNextAuthContext(ctx context.Context) context.Context {
	return context.WithValue(ctx, runObservationNextAuthContextMarker{}, true)
}

func awaitRunObservationNext(t *testing.T, result <-chan runObservationNextResult, what string) runObservationNextResult {
	t.Helper()
	select {
	case got := <-result:
		return got
	case <-time.After(6 * time.Second):
		t.Fatalf("timed out waiting for %s", what)
		return runObservationNextResult{}
	}
}

func snapshotRunObservationWatermarks(manager *runObservationManager, lease *runObservationLease) map[runObservationPollerKey]runObservationWatermark {
	manager.mu.Lock()
	defer manager.mu.Unlock()
	copy := make(map[runObservationPollerKey]runObservationWatermark, len(lease.watermarks))
	for key, watermark := range lease.watermarks {
		copy[key] = watermark
	}
	return copy
}

func releaseRunObservationNextBarrier(t *testing.T, ch chan struct{}) func() {
	t.Helper()
	var once sync.Once
	release := func() { once.Do(func() { close(ch) }) }
	t.Cleanup(release)
	return release
}

func waitRunObservationNextGenerations(t *testing.T, manager *runObservationManager, keys []runObservationPollerKey, generation uint64) {
	t.Helper()
	timer := time.NewTimer(6 * time.Second)
	defer timer.Stop()
	for {
		manager.mu.Lock()
		ready := true
		for _, key := range keys {
			entry := manager.pollers[key]
			if entry == nil || entry.current == nil || entry.current.Generation < generation {
				ready = false
				break
			}
		}
		manager.mu.Unlock()
		if ready {
			return
		}
		select {
		case <-timer.C:
			t.Fatalf("actual worker publications did not reach generation %d for every run", generation)
		default:
			runtime.Gosched()
		}
	}
}

type runObservationNextControlledRun struct {
	key           runObservationPollerKey
	secondRead    chan struct{}
	secondGate    chan struct{}
	releaseSecond func()
	thirdRead     chan struct{}
	thirdGate     chan struct{}
	releaseThird  func()
	reads         atomic.Int32
}

func newRunObservationNextTwoRunFixture(t *testing.T, suffix string) (*runObservationManager, *runObservationLease, []*runObservationNextControlledRun) {
	t.Helper()
	const caseID = "case_1"
	caller := runObservationManagerCaller("session-next-controlled-" + suffix)
	controls := []*runObservationNextControlledRun{
		{key: runObservationPollerKey{caseID: caseID, runID: "z-run-" + suffix, planItemID: "item-z-" + suffix}, secondRead: make(chan struct{}, 1), thirdRead: make(chan struct{}, 1)},
		{key: runObservationPollerKey{caseID: caseID, runID: "a-run-" + suffix, planItemID: "item-a-" + suffix}, secondRead: make(chan struct{}, 1), thirdRead: make(chan struct{}, 1)},
	}
	rows := make([]ports.RunSummary, 0, len(controls))
	parents := make([]string, 0, len(controls))
	byRunID := make(map[string]*runObservationNextControlledRun, len(controls))
	var releaseAllOnce sync.Once
	releaseAll := func() {
		releaseAllOnce.Do(func() {
			for _, control := range controls {
				control.releaseSecond()
				control.releaseThird()
			}
		})
	}
	for _, control := range controls {
		control.secondGate = make(chan struct{})
		control.thirdGate = make(chan struct{})
		control.releaseSecond = releaseRunObservationNextBarrier(t, control.secondGate)
		control.releaseThird = releaseRunObservationNextBarrier(t, control.thirdGate)
		rows = append(rows, ports.RunSummary{RunID: control.key.runID, CaseID: caseID, PlanItemID: control.key.planItemID, Execution: "active", Settlement: "unsettled"})
		parents = append(parents, control.key.planItemID)
		byRunID[control.key.runID] = control
	}
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: rows, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(ctx context.Context, gotCase, runID, itemID string) (ports.RunTraceSnapshot, error) {
		control := byRunID[runID]
		if control == nil || gotCase != caseID || control.key.planItemID != itemID {
			return ports.RunTraceSnapshot{}, errors.New("unexpected controlled run key")
		}
		base := retainedTestFrame("base-"+runID, "2026-10-08T12:00:00Z")
		middle := retainedTestFrame("middle-"+runID, "2026-10-08T12:00:01Z")
		latest := retainedTestFrame("latest-"+runID, "2026-10-08T12:00:02Z")
		snapshot := func(execution, settlement string, frames ...ports.RunTraceFrame) ports.RunTraceSnapshot {
			return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: execution, Settlement: settlement, Frames: frames, TotalFrameCount: len(frames)}
		}
		switch control.reads.Add(1) {
		case 1:
			return snapshot("active", "unsettled", base), nil
		case 2:
			control.secondRead <- struct{}{}
			<-control.secondGate
			return snapshot("active", "unsettled", base, middle), nil
		case 3:
			control.thirdRead <- struct{}{}
			select {
			case <-control.thirdGate:
				return snapshot("completed", "accepted", base, latest), nil
			case <-ctx.Done():
				return ports.RunTraceSnapshot{}, ctx.Err()
			}
		default:
			<-ctx.Done()
			return ports.RunTraceSnapshot{}, ctx.Err()
		}
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-controlled-"+suffix, parents...))
	cleanupRunObservationManager(t, manager, releaseAll)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare controlled two-run cohort = lease %v, error %v", lease, err)
	}
	return manager, lease, controls
}

func TestRunObservationNextCompleteEmptyUnavailableAndFreshCallerContext(t *testing.T) {
	const caseID, cursor = "case_1", "opaque/cursor-next-empty"
	caller := runObservationManagerCaller("session-next-empty")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{}, errors.New("empty list must not start a trace read")
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, cursor, "item-A"))
	cleanupRunObservationManager(t, manager, nil)
	prepareCtx, cancelPrepare := context.WithCancel(context.Background())
	_, lease, err := manager.prepare(prepareCtx, caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare empty cohort = lease %v, error %v", lease, err)
	}
	cancelPrepare()
	got, err := lease.next(context.Background())
	if err != nil {
		t.Fatalf("Next with a fresh caller context after Prepare returned: %v", err)
	}
	if got.CaseID != caseID || got.AsOfCursor != cursor || got.ObservedAt.IsZero() || got.Runs == nil || len(got.Runs) != 0 || got.WindowGaps == nil || len(got.WindowGaps) != 0 {
		t.Fatalf("complete-empty Next aggregate = %#v", got)
	}

	listUnavailable := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{}, errors.New("controlled unavailable list")
	})
	refused := newRunObservationManagerFixture(listUnavailable, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, cursor, "item-A"))
	cleanupRunObservationManager(t, refused, nil)
	_, refusedLease, err := refused.prepare(context.Background(), caseID, caller)
	if err != nil || refusedLease == nil {
		t.Fatalf("Prepare unavailable-list cohort = lease %v, error %v", refusedLease, err)
	}
	got, err = refusedLease.next(context.Background())
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable || !reflect.DeepEqual(got, runObservationNextAggregate{}) {
		t.Fatalf("unavailable-list Next = (%#v, %v), want typed unavailable and zero aggregate", got, err)
	}
}

func TestRunObservationNextAcceptsTerminalCaptureWithoutReplayingHydratedFrames(t *testing.T) {
	const caseID = "case_1"
	const runCount = 8
	const frameCount = runObservationRetainedFrameLimit
	caller := runObservationManagerCaller("session-next-terminal")
	rows := make([]ports.RunSummary, 0, runCount)
	parents := make([]string, 0, runCount)
	snapshots := make(map[string]ports.RunTraceSnapshot, runCount)
	for runIndex := 0; runIndex < runCount; runIndex++ {
		runID := "run-next-terminal-" + strconv.Itoa(runIndex)
		itemID := "item-next-terminal-" + strconv.Itoa(runIndex)
		rows = append(rows, ports.RunSummary{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"})
		parents = append(parents, itemID)
		frames := make([]ports.RunTraceFrame, 0, frameCount)
		for index := 0; index < frameCount; index++ {
			frames = append(frames, retainedTestFrame(runID+"-event-"+strings.Repeat("x", 128)+"-"+strconv.Itoa(index), "2026-10-08T12:00:00Z"))
		}
		snapshots[runID] = ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: frames, TotalFrameCount: len(frames)}
	}
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: rows, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(_ context.Context, _, runID, itemID string) (ports.RunTraceSnapshot, error) {
		snapshot, ok := snapshots[runID]
		if !ok || snapshot.PlanItemID != itemID {
			return ports.RunTraceSnapshot{}, errors.New("unexpected terminal hydration key")
		}
		return snapshot, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-terminal", parents...))
	cleanupRunObservationManager(t, manager, nil)
	event, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare terminal cohort = lease %v, error %v", lease, err)
	}
	totalDTOFrames := 0
	for _, run := range event.Payload.Runs {
		totalDTOFrames += len(run.Frames)
	}
	if len(event.Payload.Runs) != runCount || totalDTOFrames >= runCount*frameCount {
		t.Fatalf("fixture did not aggregate-hydration-prune frames: runs=%d retained=%d original=%d", len(event.Payload.Runs), totalDTOFrames, runCount*frameCount)
	}
	terminalEntries := make(map[runObservationPollerKey]*runObservationPoller, runCount)
	terminalCurrent := make(map[runObservationPollerKey]*runObservationRetainedState, runCount)
	manager.mu.Lock()
	for _, row := range rows {
		key := runObservationPollerKey{caseID: caseID, runID: row.RunID, planItemID: row.PlanItemID}
		entry := lease.pollers[key]
		if entry != nil && manager.pollers[key] == entry {
			terminalEntries[key] = entry
			terminalCurrent[key] = entry.current
		}
	}
	manager.mu.Unlock()
	if len(terminalEntries) != runCount {
		t.Fatalf("terminal Prepare retained %d exact poller entries, want %d", len(terminalEntries), runCount)
	}
	for _, entry := range terminalEntries {
		waitRunObservationManagerSignal(t, entry.workerDone, "successful terminal poller worker JOIN")
	}
	manager.mu.Lock()
	terminalOwnershipValid := len(lease.pollers) == runCount && len(manager.pollers) == runCount
	for key, entry := range terminalEntries {
		_, attached := entry.refs[lease]
		if manager.pollers[key] != entry || lease.pollers[key] != entry || !attached ||
			entry.current == nil || entry.current != terminalCurrent[key] ||
			entry.current.Availability != retainedCurrent || len(entry.refs) != 1 {
			terminalOwnershipValid = false
		}
	}
	manager.mu.Unlock()
	if !terminalOwnershipValid {
		t.Fatal("successful terminal worker JOIN lost its exact retained current attachment or counted capacity")
	}
	got, err := lease.next(context.Background())
	if err != nil {
		t.Fatalf("Next successful terminal retained capture: %v", err)
	}
	if len(got.Runs) != runCount {
		t.Fatalf("Next terminal runs = %d, want %d", len(got.Runs), runCount)
	}
	for _, run := range got.Runs {
		if run.frames == nil || len(run.frames) != 0 || run.execution != contract.RunExecutionStanding("completed") {
			t.Fatalf("Next replayed initial frames or lost terminal capture: %#v", got.Runs)
		}
	}
}

func TestRunObservationNextInitialReadUnavailableHasNoWatermarkOrPollerRef(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-initial-unavailable", "item-next-initial-unavailable"
	caller := runObservationManagerCaller("session-next-initial-unavailable")
	readStarted := make(chan struct{}, 1)
	readGate := make(chan struct{})
	releaseRead := releaseRunObservationNextBarrier(t, readGate)
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		readStarted <- struct{}{}
		<-readGate
		return ports.RunTraceSnapshot{}, errors.New("controlled initial read unavailable")
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-initial-unavailable", itemID))
	cleanupRunObservationManager(t, manager, releaseRead)
	result := make(chan runObservationPrepareResult, 1)
	go func() {
		event, lease, err := manager.prepare(context.Background(), caseID, caller)
		result <- runObservationPrepareResult{event: event, lease: lease, err: err}
	}()
	waitRunObservationManagerSignal(t, readStarted, "actual initial trace read")
	key := runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}
	manager.mu.Lock()
	entry := manager.pollers[key]
	attachedBeforeReadReturns := entry != nil && len(entry.refs) == 1
	manager.mu.Unlock()
	if !attachedBeforeReadReturns {
		t.Fatalf("initial read did not use an actual attached poller: entry=%v", entry)
	}
	releaseRead()
	prepared := awaitRunObservationManagerPrepare(t, result, "accepted-list initial-read failure")
	if prepared.event.Payload.RunListState != contract.RunTraceListState("complete") ||
		prepared.event.Payload.ObservationState != contract.RunTraceObservationState("unavailable") ||
		prepared.event.Payload.UnavailableRunCount == nil || *prepared.event.Payload.UnavailableRunCount != 1 {
		t.Fatalf("accepted-list initial read failure wrapper = %+v", prepared.event.Payload)
	}
	manager.mu.Lock()
	_, stillRegistered := manager.pollers[key]
	_, inCohort := manager.cohorts[prepared.lease]
	watermarks, refs := len(prepared.lease.watermarks), len(prepared.lease.pollers)
	entryRefs := len(entry.refs)
	manager.mu.Unlock()
	select {
	case <-entry.workerDone:
	default:
		t.Fatal("Prepare returned before the actual initial-failure worker JOIN")
	}
	if !inCohort || stillRegistered || watermarks != 0 || refs != 0 || entryRefs != 0 {
		t.Fatalf("initial unavailable cleanup cohort=%v registered=%v watermarks=%d leaseRefs=%d entryRefs=%d; want successful empty lease, no baseline/ref/entry", inCohort, stillRegistered, watermarks, refs, entryRefs)
	}
}

func TestRunObservationNextCapturesBeforeProjectionAndKeepsPublicationForNextCall(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-capture", "item-next-capture"
	caller := runObservationManagerCaller("session-next-capture")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
	})
	secondRead := make(chan struct{}, 1)
	thirdRead := make(chan struct{}, 1)
	secondRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(secondRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		switch traceCalls.Add(1) {
		case 1:
			return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{retainedTestFrame("event-captured", "2026-10-08T12:00:00Z")}, TotalFrameCount: 1}, nil
		case 2:
			secondRead <- struct{}{}
			<-secondRelease
			return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{retainedTestFrame("event-captured", "2026-10-08T12:00:00Z"), retainedTestFrame("event-published-later", "2026-10-08T12:00:01Z")}, TotalFrameCount: 2}, nil
		case 3:
			thirdRead <- struct{}{}
			return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{retainedTestFrame("event-captured", "2026-10-08T12:00:00Z"), retainedTestFrame("event-published-later", "2026-10-08T12:00:01Z"), retainedTestFrame("event-published-again", "2026-10-08T12:00:02Z")}, TotalFrameCount: 3}, nil
		default:
			return ports.RunTraceSnapshot{}, errors.New("unexpected additional trace read")
		}
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-capture", itemID))
	cleanupRunObservationManager(t, manager, release)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare capture cohort = lease %v, error %v", lease, err)
	}
	_, survivor, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || survivor == nil {
		t.Fatalf("second shared Prepare = lease %v, error %v", survivor, err)
	}
	if cap(lease.wake) != 1 || len(lease.wake) != 1 || cap(survivor.wake) != 1 || len(survivor.wake) != 1 {
		t.Fatalf("successful Prepare did not seed one coalesced wake per lease: first=%d/%d second=%d/%d", len(lease.wake), cap(lease.wake), len(survivor.wake), cap(survivor.wake))
	}
	seed, err := survivor.next(context.Background())
	if err != nil || len(seed.Runs) != 1 || len(seed.Runs[0].frames) != 0 {
		t.Fatalf("survivor initial Next = (%#v, %v), want initial watermark without replay", seed, err)
	}
	started := make(chan struct{})
	projectRelease := make(chan struct{})
	releaseProject := releaseRunObservationNextBarrier(t, projectRelease)
	firstResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.nextWithProjector(context.Background(), func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
			close(started)
			<-projectRelease
			return buildRunObservationRunDelta(state, ledger, prior)
		})
		firstResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, started, "projector entry after complete capture")
	overlap, overlapErr := lease.next(context.Background())
	var alreadyInProgress *runObservationNextAlreadyInProgressError
	if overlapErr == nil || !errors.As(overlapErr, &alreadyInProgress) || !reflect.DeepEqual(overlap, runObservationNextAggregate{}) {
		t.Fatalf("overlapping Next = (%#v, %v), want private already-in-progress error and zero aggregate", overlap, overlapErr)
	}
	waitRunObservationManagerSignal(t, secondRead, "actual poller next read")
	release()
	manager.mu.Lock()
	entry := lease.pollers[runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}]
	manager.mu.Unlock()
	deadline := time.NewTimer(6 * time.Second)
	defer deadline.Stop()
	for {
		manager.mu.Lock()
		advanced := entry != nil && entry.current != nil && entry.current.Generation >= 2
		manager.mu.Unlock()
		if advanced {
			break
		}
		select {
		case <-deadline.C:
			t.Fatal("actual worker did not publish the later captured generation")
		default:
			runtime.Gosched()
		}
	}
	waitRunObservationManagerSignal(t, thirdRead, "subsequent publication while projector remains blocked")
	thirdDeadline := time.NewTimer(6 * time.Second)
	defer thirdDeadline.Stop()
	for {
		manager.mu.Lock()
		advanced := entry != nil && entry.current != nil && entry.current.Generation >= 3
		manager.mu.Unlock()
		if advanced {
			break
		}
		select {
		case <-thirdDeadline.C:
			t.Fatal("actual worker did not publish the coalesced later generation")
		default:
			runtime.Gosched()
		}
	}
	if len(lease.wake) != 1 || len(survivor.wake) != 1 {
		t.Fatalf("publication did not retain a coalesced wake while projection was blocked: first=%d second=%d", len(lease.wake), len(survivor.wake))
	}
	releaseProject()
	first := awaitRunObservationNext(t, firstResult, "captured Next")
	if first.err != nil || len(first.aggregate.Runs) != 1 || first.aggregate.Runs[0].window.generation != 1 {
		t.Fatalf("blocked projector Next = (%#v, %v), want captured generation 1", first.aggregate, first.err)
	}
	second, err := lease.next(context.Background())
	if err != nil || len(second.Runs) != 1 || second.Runs[0].window.generation != 3 || len(second.Runs[0].frames) != 2 {
		t.Fatalf("subsequent Next = (%#v, %v), want coalesced generation 3 with both later frames", second, err)
	}
	firstWatermarks := snapshotRunObservationWatermarks(manager, lease)
	secondWatermarks := snapshotRunObservationWatermarks(manager, survivor)
	key := runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}
	if firstWatermarks[key].highestObservedOrdinal != 3 || secondWatermarks[key].highestObservedOrdinal != 1 {
		t.Fatalf("shared lease candidate watermarks were not independent: first=%#v second=%#v", firstWatermarks, secondWatermarks)
	}
	if err := lease.detachAndDrain(context.Background()); err != nil {
		t.Fatalf("detach first shared lease: %v", err)
	}
	survivorDelta, err := survivor.next(context.Background())
	if err != nil || len(survivorDelta.Runs) != 1 || survivorDelta.Runs[0].window.generation != 3 || len(survivorDelta.Runs[0].frames) != 2 {
		t.Fatalf("surviving shared lease Next = (%#v, %v), want captured generation 3", survivorDelta, err)
	}
}

func TestRunObservationNextUnavailableListAndFailedProjectionDoNotAdvanceWatermarks(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-failure", "item-next-failure"
	caller := runObservationManagerCaller("session-next-failure")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
	})
	secondRead := make(chan struct{}, 1)
	secondGate := make(chan struct{})
	releaseSecond := releaseRunObservationNextBarrier(t, secondGate)
	thirdRead := make(chan struct{}, 1)
	thirdGate := make(chan struct{})
	releaseThird := releaseRunObservationNextBarrier(t, thirdGate)
	releaseAll := func() {
		releaseSecond()
		releaseThird()
	}
	var traceCalls atomic.Int32
	traces := runObservationManagerTraceFunc(func(ctx context.Context, _, _, _ string) (ports.RunTraceSnapshot, error) {
		switch traceCalls.Add(1) {
		case 1:
			return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{retainedTestFrame("event-before-failure", "2026-10-08T12:00:00Z")}, TotalFrameCount: 1}, nil
		case 2:
			secondRead <- struct{}{}
			select {
			case <-secondGate:
			case <-ctx.Done():
				return ports.RunTraceSnapshot{}, ctx.Err()
			}
			return ports.RunTraceSnapshot{}, errors.New("controlled transient read failure")
		case 3:
			thirdRead <- struct{}{}
			select {
			case <-thirdGate:
				return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{retainedTestFrame("event-before-failure", "2026-10-08T12:00:00Z"), retainedTestFrame("event-after-recovery", "2026-10-08T12:00:02Z")}, TotalFrameCount: 2}, nil
			case <-ctx.Done():
				return ports.RunTraceSnapshot{}, ctx.Err()
			}
		default:
			return ports.RunTraceSnapshot{}, errors.New("unexpected additional trace read")
		}
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-failure", itemID))
	cleanupRunObservationManager(t, manager, releaseAll)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare failure cohort = lease %v, error %v", lease, err)
	}
	initial, err := lease.next(context.Background())
	if err != nil || len(initial.Runs) != 1 || len(initial.Runs[0].frames) != 0 {
		t.Fatalf("initial Next = (%#v, %v), want no replay", initial, err)
	}
	before := snapshotRunObservationWatermarks(manager, lease)
	key := runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}
	manager.mu.Lock()
	entryBeforeFailure := manager.pollers[key]
	_, cohortBeforeFailure := manager.cohorts[lease]
	initialOwnershipValid := entryBeforeFailure != nil && lease.pollers[key] == entryBeforeFailure
	if initialOwnershipValid {
		_, entryRef := entryBeforeFailure.refs[lease]
		initialOwnershipValid = entryRef && len(entryBeforeFailure.refs) == 1 && len(lease.pollers) == 1
	}
	manager.mu.Unlock()
	if !cohortBeforeFailure || !initialOwnershipValid {
		t.Fatal("transient failure fixture did not begin with its actual attached poller and counted cohort")
	}
	failureResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.next(context.Background())
		failureResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, secondRead, "controlled failed read")
	releaseSecond()
	failure := awaitRunObservationNext(t, failureResult, "failed-candidate Next")
	if failure.err == nil || apperr.KindOf(failure.err) != apperr.KindUnavailable || !reflect.DeepEqual(failure.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("failed projection Next = (%#v, %v), want typed unavailable and zero aggregate", failure.aggregate, failure.err)
	}
	if after := snapshotRunObservationWatermarks(manager, lease); !reflect.DeepEqual(after, before) {
		t.Fatalf("failed projection advanced watermarks: before=%#v after=%#v", before, after)
	}
	manager.mu.Lock()
	entry := manager.pollers[key]
	_, cohortCounted := manager.cohorts[lease]
	failedOwnershipValid := entry == entryBeforeFailure && entry != nil && lease.pollers[key] == entry
	if failedOwnershipValid {
		_, entryRef := entry.refs[lease]
		failedOwnershipValid = entryRef && len(entry.refs) == 1 && len(lease.pollers) == 1 &&
			entry.current != nil && entry.current.Availability == retainedReadUnavailable &&
			len(manager.pollers) == 1 && len(manager.cohorts) == 1
	}
	manager.mu.Unlock()
	if !cohortCounted || !failedOwnershipValid {
		t.Fatal("transient failure fixture lost its marker, exact poller refs, or counted cohort before recovery")
	}
	recoveryResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.next(context.Background())
		recoveryResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, thirdRead, "complete recovery read")
	if after := snapshotRunObservationWatermarks(manager, lease); !reflect.DeepEqual(after, before) {
		t.Fatalf("watermarks changed while fitting recovery read was held: before=%#v after=%#v", before, after)
	}
	manager.mu.Lock()
	stillSameEntry := manager.pollers[key] == entryBeforeFailure && lease.pollers[key] == entryBeforeFailure
	_, stillReferenced := entry.refs[lease]
	_, stillCounted := manager.cohorts[lease]
	stillCounted = stillSameEntry && entry.current != nil && entry.current.Availability == retainedReadUnavailable &&
		stillReferenced && len(entry.refs) == 1 &&
		len(lease.pollers) == 1 && len(manager.pollers) == 1 && len(manager.cohorts) == 1 && stillCounted
	manager.mu.Unlock()
	if !stillCounted {
		t.Fatal("transient recovery read did not retain the exact poller, both refs, and counted cohort/entry capacity")
	}
	releaseThird()
	recovered := awaitRunObservationNext(t, recoveryResult, "recovered Next")
	if recovered.err != nil || len(recovered.aggregate.Runs) != 1 || len(recovered.aggregate.Runs[0].frames) != 1 || recovered.aggregate.Runs[0].frames[0].EventID != "event-after-recovery" {
		t.Fatalf("recovered Next = (%#v, %v), want only the new accepted frame", recovered.aggregate, recovered.err)
	}
}

func TestRunObservationNextNonterminalRetentionRecoversAndTerminalRetentionStops(t *testing.T) {
	for _, terminal := range []bool{false, true} {
		name := "nonterminal-recovery"
		if terminal {
			name = "terminal-stop"
		}
		t.Run(name, func(t *testing.T) {
			const caseID, runID, itemID = "case_1", "run-next-retention", "item-next-retention"
			caller := runObservationManagerCaller("session-next-retention-" + name)
			list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
				return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
			})
			secondRead := make(chan struct{}, 1)
			secondGate := make(chan struct{})
			releaseSecond := releaseRunObservationNextBarrier(t, secondGate)
			thirdRead := make(chan struct{}, 1)
			thirdGate := make(chan struct{})
			releaseThird := releaseRunObservationNextBarrier(t, thirdGate)
			releaseAll := func() {
				releaseSecond()
				releaseThird()
			}
			var traceCalls atomic.Int32
			traces := runObservationManagerTraceFunc(func(ctx context.Context, _, _, _ string) (ports.RunTraceSnapshot, error) {
				switch traceCalls.Add(1) {
				case 1:
					return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{retainedTestFrame("event-retention-before", "2026-10-08T12:00:00Z")}, TotalFrameCount: 1}, nil
				case 2:
					secondRead <- struct{}{}
					select {
					case <-secondGate:
					case <-ctx.Done():
						return ports.RunTraceSnapshot{}, ctx.Err()
					}
					large := make([]ports.RunTraceFrame, runObservationRetainedFrameLimit)
					for index := range large {
						id := "oversized-retained-event-" + strconv.Itoa(index) + strings.Repeat("x", 1100)
						large[index] = retainedTestFrame(id, "2026-10-08T12:00:01Z")
					}
					execution, settlement := "active", "unsettled"
					if terminal {
						execution, settlement = "completed", "accepted"
					}
					return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: execution, Settlement: settlement, Frames: large, TotalFrameCount: len(large)}, nil
				case 3:
					thirdRead <- struct{}{}
					select {
					case <-thirdGate:
						return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{retainedTestFrame("event-retention-before", "2026-10-08T12:00:00Z"), retainedTestFrame("event-retention-recovered", "2026-10-08T12:00:02Z")}, TotalFrameCount: 2}, nil
					case <-ctx.Done():
						return ports.RunTraceSnapshot{}, ctx.Err()
					}
				default:
					return ports.RunTraceSnapshot{}, errors.New("unexpected extra retention fixture read")
				}
			})
			manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-retention", itemID))
			cleanupRunObservationManager(t, manager, releaseAll)
			_, lease, err := manager.prepare(context.Background(), caseID, caller)
			if err != nil || lease == nil {
				t.Fatalf("Prepare retention cohort = lease %v, error %v", lease, err)
			}
			initial, err := lease.next(context.Background())
			if err != nil || len(initial.Runs) != 1 || len(initial.Runs[0].frames) != 0 {
				t.Fatalf("initial retention Next = (%#v, %v), want seeded no-replay result", initial, err)
			}
			before := snapshotRunObservationWatermarks(manager, lease)
			key := runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}
			manager.mu.Lock()
			entryBeforeFailure := manager.pollers[key]
			_, cohortBeforeFailure := manager.cohorts[lease]
			initialOwnershipValid := entryBeforeFailure != nil && lease.pollers[key] == entryBeforeFailure
			if initialOwnershipValid {
				_, entryRef := entryBeforeFailure.refs[lease]
				initialOwnershipValid = entryRef && len(entryBeforeFailure.refs) == 1 && len(lease.pollers) == 1
			}
			manager.mu.Unlock()
			if !cohortBeforeFailure || !initialOwnershipValid {
				t.Fatal("retention failure fixture did not begin with its actual attached poller and counted cohort")
			}
			failedResult := make(chan runObservationNextResult, 1)
			go func() {
				aggregate, err := lease.next(context.Background())
				failedResult <- runObservationNextResult{aggregate: aggregate, err: err}
			}()
			waitRunObservationManagerSignal(t, secondRead, "oversized retained candidate")
			releaseSecond()
			failed := awaitRunObservationNext(t, failedResult, "retention-unavailable Next")
			if failed.err == nil || apperr.KindOf(failed.err) != apperr.KindUnavailable || !reflect.DeepEqual(failed.aggregate, runObservationNextAggregate{}) {
				t.Fatalf("retention-unavailable Next = (%#v, %v), want typed failure and zero aggregate", failed.aggregate, failed.err)
			}
			if after := snapshotRunObservationWatermarks(manager, lease); !reflect.DeepEqual(after, before) {
				t.Fatalf("retention failure advanced watermark: before=%#v after=%#v", before, after)
			}
			if terminal {
				select {
				case <-lease.drainDone:
				case <-time.After(6 * time.Second):
					t.Fatal("terminal retention failure did not join its drain")
				}
				return
			}
			manager.mu.Lock()
			entry := manager.pollers[key]
			_, cohortCounted := manager.cohorts[lease]
			ownershipValid := entry == entryBeforeFailure && entry != nil && lease.pollers[key] == entry
			if ownershipValid {
				_, entryRef := entry.refs[lease]
				ownershipValid = entryRef && len(entry.refs) == 1 && len(lease.pollers) == 1 &&
					entry.current != nil && entry.current.Availability == retainedRetentionUnavailable &&
					len(manager.pollers) == 1 && len(manager.cohorts) == 1
			}
			manager.mu.Unlock()
			if !cohortCounted || !ownershipValid {
				t.Fatal("nonterminal retention failure lost its marker, exact poller refs, or counted cohort")
			}
			recoveryResult := make(chan runObservationNextResult, 1)
			go func() {
				aggregate, err := lease.next(context.Background())
				recoveryResult <- runObservationNextResult{aggregate: aggregate, err: err}
			}()
			waitRunObservationManagerSignal(t, thirdRead, "complete fitting retention recovery")
			if after := snapshotRunObservationWatermarks(manager, lease); !reflect.DeepEqual(after, before) {
				t.Fatalf("watermarks changed while fitting retention read was held: before=%#v after=%#v", before, after)
			}
			manager.mu.Lock()
			stillSameEntry := manager.pollers[key] == entryBeforeFailure && lease.pollers[key] == entryBeforeFailure
			_, stillReferenced := entry.refs[lease]
			_, stillCounted := manager.cohorts[lease]
			stillCounted = stillSameEntry && entry.current != nil && entry.current.Availability == retainedRetentionUnavailable &&
				stillReferenced && len(entry.refs) == 1 &&
				len(lease.pollers) == 1 && len(manager.pollers) == 1 && len(manager.cohorts) == 1 && stillCounted
			manager.mu.Unlock()
			if !stillCounted {
				t.Fatal("retention recovery read did not retain the exact poller, both refs, and counted cohort/entry capacity")
			}
			releaseThird()
			recovered := awaitRunObservationNext(t, recoveryResult, "fitting recovery Next")
			if recovered.err != nil || len(recovered.aggregate.Runs) != 1 || len(recovered.aggregate.Runs[0].frames) != 1 || recovered.aggregate.Runs[0].frames[0].EventID != "event-retention-recovered" {
				t.Fatalf("retention recovery Next = (%#v, %v), want only the newly accepted frame", recovered.aggregate, recovered.err)
			}
		})
	}
}

func TestRunObservationNextDiscardsWholeMultiRunCandidateAfterOneProjectionFails(t *testing.T) {
	manager, lease, controls := newRunObservationNextTwoRunFixture(t, "atomic")
	seed, err := lease.next(context.Background())
	if err != nil || len(seed.Runs) != 2 || seed.Runs[0].frames == nil || seed.Runs[1].frames == nil {
		t.Fatalf("initial two-run Next = (%#v, %v), want seeded, non-replayed baselines", seed, err)
	}
	keys := make([]runObservationPollerKey, 0, len(controls))
	for _, control := range controls {
		keys = append(keys, control.key)
		waitRunObservationManagerSignal(t, control.secondRead, "actual second read for "+control.key.runID)
	}
	for _, control := range controls {
		control.releaseSecond()
	}
	waitRunObservationNextGenerations(t, manager, keys, 2)
	manager.mu.Lock()
	workerEntries := make([]*runObservationPoller, 0, len(keys))
	for _, key := range keys {
		if entry := manager.pollers[key]; entry != nil {
			workerEntries = append(workerEntries, entry)
		}
	}
	manager.mu.Unlock()
	if len(workerEntries) != len(keys) {
		t.Fatalf("captured %d real worker entries for %d advanced runs", len(workerEntries), len(keys))
	}
	before := snapshotRunObservationWatermarks(manager, lease)
	if len(before) != 2 || before[keys[0]].highestObservedOrdinal != 1 || before[keys[1]].highestObservedOrdinal != 1 {
		t.Fatalf("real initial watermark baselines = %#v, want both at ordinal 1", before)
	}
	var projected atomic.Int32
	var advancedCandidate atomic.Bool
	got, err := lease.nextWithProjector(context.Background(), func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
		if projected.Add(1) == 2 {
			return runObservationRunDelta{}, nil, runObservationWatermark{}, runObservationUnavailable("controlled second-run projection failure")
		}
		delta, gap, candidate, projectErr := buildRunObservationRunDelta(state, ledger, prior)
		if projectErr == nil && candidate.highestObservedOrdinal > prior.highestObservedOrdinal && len(delta.frames) > 0 {
			advancedCandidate.Store(true)
		}
		return delta, gap, candidate, projectErr
	})
	if err == nil || apperr.KindOf(err) != apperr.KindUnavailable || !reflect.DeepEqual(got, runObservationNextAggregate{}) {
		t.Fatalf("partial multi-run Next = (%#v, %v), want typed failure and zero aggregate", got, err)
	}
	if projected.Load() != 2 || !advancedCandidate.Load() {
		t.Fatalf("projector calls=%d advancedCandidate=%v, want two real candidates and the first to advance from worker publication", projected.Load(), advancedCandidate.Load())
	}
	if after := snapshotRunObservationWatermarks(manager, lease); !reflect.DeepEqual(after, before) {
		t.Fatalf("one failed run advanced part of the cohort: before=%#v after=%#v", before, after)
	}
	select {
	case <-lease.drainDone:
	case <-time.After(6 * time.Second):
		t.Fatal("unexpected projector failure did not join terminal lease drain")
	}
	for _, entry := range workerEntries {
		select {
		case <-entry.workerDone:
		default:
			t.Fatalf("terminal projector failure did not join actual worker %+v", entry.key)
		}
	}
	manager.mu.Lock()
	_, cohortRetained := manager.cohorts[lease]
	entriesRetained := 0
	for _, key := range keys {
		if manager.pollers[key] != nil {
			entriesRetained++
		}
	}
	manager.mu.Unlock()
	if cohortRetained || entriesRetained != 0 {
		t.Fatalf("terminal projector failure retained ownership: cohort=%v entries=%d", cohortRetained, entriesRetained)
	}
}

func TestRunObservationNextSortsNonemptyWindowGapsByRunIDBytes(t *testing.T) {
	manager, lease, controls := newRunObservationNextTwoRunFixture(t, "gaps")
	if seed, err := lease.next(context.Background()); err != nil || len(seed.Runs) != 2 {
		t.Fatalf("initial gap-fixture Next = (%#v, %v), want two seeded runs", seed, err)
	}
	keys := make([]runObservationPollerKey, 0, len(controls))
	for _, control := range controls {
		keys = append(keys, control.key)
		waitRunObservationManagerSignal(t, control.secondRead, "actual gap-producing second read for "+control.key.runID)
	}
	for _, control := range controls {
		control.releaseSecond()
	}
	waitRunObservationNextGenerations(t, manager, keys, 2)
	for _, control := range controls {
		waitRunObservationManagerSignal(t, control.thirdRead, "actual gap-producing third read for "+control.key.runID)
	}
	for _, control := range controls {
		control.releaseThird()
	}
	waitRunObservationNextGenerations(t, manager, keys, 3)
	got, err := lease.next(context.Background())
	if err != nil || len(got.Runs) != 2 || len(got.WindowGaps) != 2 {
		t.Fatalf("two-run gap Next = (%#v, %v), want two runs and two nonempty gaps", got, err)
	}
	if got.Runs[0].runID != "a-run-gaps" || got.Runs[1].runID != "z-run-gaps" ||
		got.WindowGaps[0].runID != "a-run-gaps" || got.WindowGaps[1].runID != "z-run-gaps" {
		t.Fatalf("run/gap ordering = runs[%q,%q] gaps[%q,%q], want exact byte order a then z", got.Runs[0].runID, got.Runs[1].runID, got.WindowGaps[0].runID, got.WindowGaps[1].runID)
	}
	for index, gap := range got.WindowGaps {
		wantID := "middle-" + gap.runID
		if gap.previousOrdinal != 1 || gap.currentOrdinal != 3 || gap.evictedObservedEventCount != 1 || len(gap.evictedObservedEventIDs) != 1 || gap.evictedObservedEventIDs[0] != wantID || !gap.unobservedLossPossible || !gap.unknownMissedFrameCount {
			t.Fatalf("actual worker gap[%d] = %+v, want newly observed omitted ordinal 2 for %q", index, gap, wantID)
		}
	}
}

func TestRunObservationNextTerminalAuthorityContextStopAndDetachFailuresAreZeroResults(t *testing.T) {
	for _, name := range []string{"authorization", "cursor", "caller-context", "manager-stop"} {
		t.Run(name, func(t *testing.T) {
			const caseID, cursor = "case_1", "cursor-next-terminal-check"
			caller := runObservationManagerCaller("session-next-terminal-check-" + name)
			list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
				return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
			})
			traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
				return ports.RunTraceSnapshot{}, errors.New("empty cohort must not read")
			})
			present := runObservationManagerContext(caseID, cursor, "item-A")
			manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, present)
			cleanupRunObservationManager(t, manager, nil)
			_, lease, err := manager.prepare(context.Background(), caseID, caller)
			if err != nil || lease == nil {
				t.Fatalf("Prepare empty terminal-check cohort = lease %v, error %v", lease, err)
			}
			ctx := context.Background()
			var cancel context.CancelFunc
			switch name {
			case "authorization":
				manager.authorize = func(context.Context, runObservationCaller, string, runObservationSessionCurrent, PerspectiveVerifier) error {
					return runObservationUnavailable("controlled session revocation")
				}
			case "cursor":
				present.mu.Lock()
				present.cursors[caseID] = "cursor-changed-after-Prepare"
				present.mu.Unlock()
			case "caller-context":
				ctx, cancel = context.WithCancel(context.Background())
				cancel()
			case "manager-stop":
				if err := manager.stopAndDrain(context.Background()); err != nil {
					t.Fatalf("StopAndDrain before Next: %v", err)
				}
			}
			if cancel != nil {
				defer cancel()
			}
			got, err := lease.next(ctx)
			if err == nil || !reflect.DeepEqual(got, runObservationNextAggregate{}) {
				t.Fatalf("Next after %s failure = (%#v, %v), want zero result and error", name, got, err)
			}
			if name != "manager-stop" {
				select {
				case <-lease.drainDone:
				case <-time.After(6 * time.Second):
					t.Fatalf("terminal %s failure did not join its drain", name)
				}
			}
		})
	}
}

func TestRunObservationNextRechecksAuthorizationAfterWake(t *testing.T) {
	const caseID, runID, itemID, cursor = "case_1", "run-next-postwake-auth", "item-next-postwake-auth", "cursor-next-postwake-auth"
	caller := runObservationManagerCaller("session-next-postwake-auth")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{retainedTestFrame("event-postwake-auth", "2026-10-08T12:00:00Z")}, TotalFrameCount: 1}, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, cursor, itemID))
	cleanupRunObservationManager(t, manager, nil)
	baseAuthorize := manager.authorize
	var authCalls, projectionCalls atomic.Int32
	manager.authorize = func(ctx context.Context, identity runObservationCaller, id string, sessions runObservationSessionCurrent, verifier PerspectiveVerifier) error {
		if ctx.Value(runObservationNextAuthContextMarker{}) == true && authCalls.Add(1) == 2 {
			return runObservationUnavailable("controlled post-wake authorization failure")
		}
		return baseAuthorize(ctx, identity, id, sessions, verifier)
	}
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare post-wake auth cohort = lease %v, error %v", lease, err)
	}
	if len(lease.wake) != 1 {
		t.Fatalf("Prepare wake count=%d, want seeded wake before Next", len(lease.wake))
	}
	ctx := runObservationNextAuthContext(context.Background())
	got, err := lease.nextWithProjector(ctx, func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
		projectionCalls.Add(1)
		return buildRunObservationRunDelta(state, ledger, prior)
	})
	if err == nil || !reflect.DeepEqual(got, runObservationNextAggregate{}) || authCalls.Load() != 2 || projectionCalls.Load() != 0 {
		t.Fatalf("post-wake authorization Next=(%#v,%v) authCalls=%d projectionCalls=%d, want second auth failure before capture", got, err, authCalls.Load(), projectionCalls.Load())
	}
	if len(lease.wake) != 0 {
		t.Fatalf("post-wake authorization failure left the seeded wake unconsumed: %d", len(lease.wake))
	}
	select {
	case <-lease.drainDone:
	case <-time.After(6 * time.Second):
		t.Fatal("post-wake authorization failure did not join terminal drain")
	}
}

func TestRunObservationNextRechecksCursorBeforeFinalDisclosure(t *testing.T) {
	const caseID, runID, itemID, cursor = "case_1", "run-next-final-cursor", "item-next-final-cursor", "cursor-next-final-cursor"
	caller := runObservationManagerCaller("session-next-final-cursor")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{retainedTestFrame("event-final-cursor", "2026-10-08T12:00:00Z")}, TotalFrameCount: 1}, nil
	})
	present := runObservationManagerContext(caseID, cursor, itemID)
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, present)
	cleanupRunObservationManager(t, manager, nil)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare final-cursor cohort = lease %v, error %v", lease, err)
	}
	projectEntered := make(chan struct{})
	projectGate := make(chan struct{})
	releaseProject := releaseRunObservationNextBarrier(t, projectGate)
	result := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.nextWithProjector(context.Background(), func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
			close(projectEntered)
			<-projectGate
			return buildRunObservationRunDelta(state, ledger, prior)
		})
		result <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, projectEntered, "captured projection before final cursor mutation")
	present.mu.Lock()
	present.cursors[caseID] = "cursor-changed-during-projection"
	present.mu.Unlock()
	releaseProject()
	got := awaitRunObservationNext(t, result, "final cursor recheck")
	if got.err == nil || !reflect.DeepEqual(got.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("Next after final cursor changed = (%#v, %v), want zero aggregate and terminal failure", got.aggregate, got.err)
	}
	select {
	case <-lease.drainDone:
	case <-time.After(6 * time.Second):
		t.Fatal("final cursor failure did not join terminal drain")
	}
}

func TestRunObservationNextBlockedAuthCallbackIsOwnedByDetach(t *testing.T) {
	const caseID, cursor = "case_1", "cursor-next-auth-drain"
	caller := runObservationManagerCaller("session-next-auth-drain")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{}, errors.New("empty cohort must not read")
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, cursor, "item-A"))
	cleanupRunObservationManager(t, manager, nil)
	baseAuthorize := manager.authorize
	authEntered := make(chan struct{}, 1)
	authGate := make(chan struct{})
	releaseAuth := releaseRunObservationNextBarrier(t, authGate)
	manager.authorize = func(ctx context.Context, identity runObservationCaller, id string, sessions runObservationSessionCurrent, verifier PerspectiveVerifier) error {
		if ctx.Value(runObservationNextAuthContextMarker{}) == true {
			authEntered <- struct{}{}
			<-authGate
			return runObservationUnavailable("controlled authorization release during detach")
		}
		return baseAuthorize(ctx, identity, id, sessions, verifier)
	}
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare auth-drain cohort = lease %v, error %v", lease, err)
	}
	nextResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.next(runObservationNextAuthContext(context.Background()))
		nextResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, authEntered, "Next authorization callback")
	if cap(lease.wake) != 1 || len(lease.wake) != 1 {
		t.Fatalf("blocked first Next authorization consumed or lost the successful Prepare-seeded wake: len=%d cap=%d", len(lease.wake), cap(lease.wake))
	}
	manager.mu.Lock()
	operationOwned := lease.nextInFlight
	manager.mu.Unlock()
	if !operationOwned {
		t.Fatal("blocked authorization callback was not registered as the Next operation")
	}
	drainResult := make(chan error, 1)
	go func() { drainResult <- lease.detachAndDrain(context.Background()) }()
	deadline := time.NewTimer(6 * time.Second)
	defer deadline.Stop()
	for {
		manager.mu.Lock()
		draining := lease.draining
		_, counted := manager.cohorts[lease]
		manager.mu.Unlock()
		if draining {
			if !counted {
				t.Fatal("detach released cohort capacity before blocked authorization returned")
			}
			select {
			case err := <-drainResult:
				t.Fatalf("detach completed before blocked authorization returned: %v", err)
			default:
			}
			break
		}
		select {
		case <-deadline.C:
			t.Fatal("detach did not claim lease with auth callback blocked")
		default:
			runtime.Gosched()
		}
	}
	releaseAuth()
	got := awaitRunObservationNext(t, nextResult, "Next after detach claimed blocked auth callback")
	if got.err == nil || !reflect.DeepEqual(got.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("Next after detach claimed auth callback = (%#v, %v), want zero result and failure", got.aggregate, got.err)
	}
	select {
	case err := <-drainResult:
		if err != nil {
			t.Fatalf("detach after blocked authorization release: %v", err)
		}
	case <-time.After(6 * time.Second):
		t.Fatal("detach did not join the registered authorization callback operation")
	}
}

func TestRunObservationNextDetachDuringProjectionJoinsOperationAndWorker(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-detach", "item-next-detach"
	caller := runObservationManagerCaller("session-next-detach")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0}, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-detach", itemID))
	cleanupRunObservationManager(t, manager, nil)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare detach cohort = lease %v, error %v", lease, err)
	}
	projectEntered := make(chan struct{})
	projectRelease := make(chan struct{})
	releaseProject := releaseRunObservationNextBarrier(t, projectRelease)
	nextResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.nextWithProjector(context.Background(), func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
			close(projectEntered)
			<-projectRelease
			return buildRunObservationRunDelta(state, ledger, prior)
		})
		nextResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, projectEntered, "captured projection before detach")
	drainResult := make(chan error, 1)
	go func() { drainResult <- lease.detachAndDrain(context.Background()) }()
	deadline := time.NewTimer(6 * time.Second)
	defer deadline.Stop()
	for {
		manager.mu.Lock()
		draining := lease.draining
		_, stillCounted := manager.cohorts[lease]
		manager.mu.Unlock()
		if draining {
			if !stillCounted {
				t.Fatal("detached lease capacity was released before its blocked Next operation joined")
			}
			select {
			case err := <-drainResult:
				t.Fatalf("detach completed before blocked Next returned: %v", err)
			default:
			}
			break
		}
		select {
		case <-deadline.C:
			t.Fatal("detach did not claim the lease")
		default:
			runtime.Gosched()
		}
	}
	releaseProject()
	got := awaitRunObservationNext(t, nextResult, "Next refusal after detach")
	if got.err == nil || !reflect.DeepEqual(got.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("Next after detach-before-commit = (%#v, %v), want zero result and error", got.aggregate, got.err)
	}
	select {
	case err := <-drainResult:
		if err != nil {
			t.Fatalf("drain after Next operation release: %v", err)
		}
	case <-time.After(6 * time.Second):
		t.Fatal("detach did not join the blocked Next operation")
	}
}

func TestRunObservationNextCallerCancellationDuringProjectionReturnsZeroAndDrains(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-cancel", "item-next-cancel"
	caller := runObservationManagerCaller("session-next-cancel")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0}, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-cancel", itemID))
	cleanupRunObservationManager(t, manager, nil)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare cancellation cohort = lease %v, error %v", lease, err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	projectEntered := make(chan struct{})
	projectRelease := make(chan struct{})
	releaseProject := releaseRunObservationNextBarrier(t, projectRelease)
	nextResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.nextWithProjector(ctx, func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
			close(projectEntered)
			<-projectRelease
			return buildRunObservationRunDelta(state, ledger, prior)
		})
		nextResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, projectEntered, "blocked projection before caller cancellation")
	cancel()
	releaseProject()
	got := awaitRunObservationNext(t, nextResult, "canceled Next")
	if got.err == nil || !reflect.DeepEqual(got.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("Next after caller canceled during projection = (%#v, %v), want zero result and error", got.aggregate, got.err)
	}
	select {
	case <-lease.drainDone:
	case <-time.After(6 * time.Second):
		t.Fatal("canceled Next did not join its terminal drain")
	}
}

func TestRunObservationNextStopDuringProjectionPreventsDisclosure(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-stop", "item-next-stop"
	caller := runObservationManagerCaller("session-next-stop")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0}, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-stop", itemID))
	cleanupRunObservationManager(t, manager, nil)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare Stop cohort = lease %v, error %v", lease, err)
	}
	projectEntered := make(chan struct{})
	projectRelease := make(chan struct{})
	releaseProject := releaseRunObservationNextBarrier(t, projectRelease)
	nextResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.nextWithProjector(context.Background(), func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
			close(projectEntered)
			<-projectRelease
			return buildRunObservationRunDelta(state, ledger, prior)
		})
		nextResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, projectEntered, "captured projection before Stop")
	stopCtx, cancelStop := context.WithCancel(context.Background())
	cancelStop()
	if err := manager.stopAndDrain(stopCtx); err == nil {
		t.Fatal("Stop with canceled wait context succeeded before captured Next released")
	}
	manager.mu.Lock()
	stopping := manager.stopping
	_, stillCounted := manager.cohorts[lease]
	manager.mu.Unlock()
	if !stopping || !stillCounted {
		t.Fatalf("Stop released capacity before Next joined: stopping=%v counted=%v", stopping, stillCounted)
	}
	releaseProject()
	got := awaitRunObservationNext(t, nextResult, "Next refusal during Stop")
	if got.err == nil || !reflect.DeepEqual(got.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("Next after Stop began = (%#v, %v), want zero result and error", got.aggregate, got.err)
	}
	if err := manager.stopAndDrain(context.Background()); err != nil {
		t.Fatalf("Stop join after Next release: %v", err)
	}
	manager.mu.Lock()
	_, stillCounted = manager.cohorts[lease]
	manager.mu.Unlock()
	if stillCounted {
		t.Fatal("Stop retained the cohort after its Next operation joined")
	}
}

func TestRunObservationNextRetainsCapacityUntilBlockedWorkerReadJoins(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-worker-join", "item-next-worker-join"
	caller := runObservationManagerCaller("session-next-worker-join")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
	})
	secondRead := make(chan struct{}, 1)
	readRelease := make(chan struct{})
	var releaseOnce sync.Once
	release := func() { releaseOnce.Do(func() { close(readRelease) }) }
	defer release()
	var traceCalls atomic.Int32
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		if traceCalls.Add(1) == 1 {
			return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0}, nil
		}
		secondRead <- struct{}{}
		<-readRelease
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled", Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0}, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-worker-join", itemID))
	cleanupRunObservationManager(t, manager, release)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare worker-join cohort = lease %v, error %v", lease, err)
	}
	waitRunObservationManagerSignal(t, secondRead, "actual recurring trace read")
	projectEntered := make(chan struct{})
	projectRelease := make(chan struct{})
	releaseProject := releaseRunObservationNextBarrier(t, projectRelease)
	nextResult := make(chan runObservationNextResult, 1)
	go func() {
		aggregate, err := lease.nextWithProjector(context.Background(), func(state *runObservationRetainedState, ledger map[string]uint64, prior runObservationWatermark) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error) {
			close(projectEntered)
			<-projectRelease
			return buildRunObservationRunDelta(state, ledger, prior)
		})
		nextResult <- runObservationNextResult{aggregate: aggregate, err: err}
	}()
	waitRunObservationManagerSignal(t, projectEntered, "Next operation while worker read is held")
	drainCtx, cancelDrain := context.WithCancel(context.Background())
	cancelDrain()
	drainErr := lease.detachAndDrain(drainCtx)
	if drainErr == nil || apperr.KindOf(drainErr) != apperr.KindUnavailable {
		t.Fatalf("canceled detach while worker read is held = %v, want typed still-draining result", drainErr)
	}
	manager.mu.Lock()
	_, counted := manager.cohorts[lease]
	entry := manager.pollers[runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}]
	manager.mu.Unlock()
	if !counted || entry == nil {
		t.Fatal("cohort or actual poller capacity was released before the held worker read joined")
	}
	release()
	releaseProject()
	got := awaitRunObservationNext(t, nextResult, "Next release during worker drain")
	if got.err == nil || !reflect.DeepEqual(got.aggregate, runObservationNextAggregate{}) {
		t.Fatalf("Next after worker drain began = (%#v, %v), want zero result and error", got.aggregate, got.err)
	}
	select {
	case <-lease.drainDone:
	case <-time.After(6 * time.Second):
		t.Fatal("drain did not join the actual worker after the read returned")
	}
	manager.mu.Lock()
	_, counted = manager.cohorts[lease]
	entry = manager.pollers[runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}]
	manager.mu.Unlock()
	if counted || entry != nil {
		t.Fatalf("joined drain retained capacity: cohort=%v poller=%v", counted, entry != nil)
	}
}

func TestRunObservationNextNotifierOwnershipKeepsDrainCountedUntilRelease(t *testing.T) {
	const caseID, runID, itemID = "case_1", "run-next-notifier", "item-next-notifier"
	caller := runObservationManagerCaller("session-next-notifier")
	list := runObservationManagerListFunc(func(context.Context, ports.CaseRef) (ports.RunListResult, error) {
		return ports.RunListResult{Runs: []ports.RunSummary{{RunID: runID, CaseID: caseID, PlanItemID: itemID, Execution: "active", Settlement: "unsettled"}}, UnreadableIDs: []string{}}, nil
	})
	traces := runObservationManagerTraceFunc(func(context.Context, string, string, string) (ports.RunTraceSnapshot, error) {
		return ports.RunTraceSnapshot{CaseID: caseID, RunID: runID, PlanItemID: itemID, Execution: "completed", Settlement: "accepted", Frames: []ports.RunTraceFrame{}, TotalFrameCount: 0}, nil
	})
	manager := newRunObservationManagerFixture(list, traces, runObservationManagerCurrent(caller.Identity.Claim()), &runObservationManagerPerspectiveFake{}, runObservationManagerContext(caseID, "cursor-next-notifier", itemID))
	cleanupRunObservationManager(t, manager, nil)
	_, lease, err := manager.prepare(context.Background(), caseID, caller)
	if err != nil || lease == nil {
		t.Fatalf("Prepare notifier cohort = lease %v, error %v", lease, err)
	}
	manager.mu.Lock()
	entry := lease.pollers[runObservationPollerKey{caseID: caseID, runID: runID, planItemID: itemID}]
	targets := manager.acquirePollerNotifierTargetsLocked(entry)
	manager.mu.Unlock()
	var releaseTargetsOnce sync.Once
	releaseTargets := func() { releaseTargetsOnce.Do(func() { manager.sendPollerNotifierTargets(targets) }) }
	t.Cleanup(releaseTargets)
	if len(targets) != 1 || targets[0].lease != lease || targets[0].wake == nil {
		t.Fatalf("registered notifier targets = %#v, want one exact attached lease", targets)
	}
	drainResult := make(chan error, 1)
	go func() { drainResult <- lease.detachAndDrain(context.Background()) }()
	deadline := time.NewTimer(6 * time.Second)
	defer deadline.Stop()
	for {
		manager.mu.Lock()
		draining := lease.draining
		_, counted := manager.cohorts[lease]
		manager.mu.Unlock()
		if draining {
			if !counted {
				t.Fatal("lease capacity was released before the registered notifier released its reference")
			}
			select {
			case err := <-drainResult:
				t.Fatalf("drain completed before the registered notifier released its reference: %v", err)
			default:
			}
			break
		}
		select {
		case <-deadline.C:
			t.Fatal("drain did not claim the lease")
		default:
			runtime.Gosched()
		}
	}
	releaseTargets()
	select {
	case err := <-drainResult:
		if err != nil {
			t.Fatalf("drain after notifier release: %v", err)
		}
	case <-time.After(6 * time.Second):
		t.Fatal("drain did not join the released notifier")
	}
}
