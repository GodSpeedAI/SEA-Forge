package server

import (
	"context"
	"strings"
	"sync"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/auth"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

// This private manager is not wired into Server or an SSE path.
const (
	runObservationCohortLimit = 16
	runObservationPollerLimit = 16
)

type runObservationListReader interface {
	RunsListForCase(context.Context, ports.CaseRef) (ports.RunListResult, error)
}

// runObservationCaller contains only the authenticated standing and session
// locator needed for a later Current check. It never retains a bearer token,
// CSRF token, or mutable auth.Session pointer.
type runObservationCaller struct {
	source    string
	sessionID string
	claim     ports.ActorClaim
}

type runObservationSessionCurrent interface {
	Current(string) (auth.CurrentSessionState, bool)
}

type runObservationAuthorize func(
	context.Context,
	runObservationCaller,
	string,
	runObservationSessionCurrent,
	PerspectiveVerifier,
) error
type runObservationGuard func(string) (runObservationPresentContext, error)

type runObservationManager struct {
	mu                sync.Mutex
	pollers           map[runObservationPollerKey]*runObservationPoller
	cohorts           map[*runObservationLease]struct{}
	prepareOperations sync.WaitGroup
	stopping          bool
	stopDone          chan struct{}
	list              runObservationListReader
	traces            ports.RunTracePort
	sessions          runObservationSessionCurrent
	verifier          PerspectiveVerifier
	authorize         runObservationAuthorize
	guard             runObservationGuard
	now               func() time.Time
}

// runObservationPoller is the private ownership record required by the future
// manager. All ref membership and state transitions are protected by the
// owning manager mutex. The lifecycle channels belong to the real worker
// contract: ready publishes initialization, ctx/cancel own shared worker
// lifetime, and workerDone proves actual worker join. Shared work is canceled
// only after its final lease ref leaves.
type runObservationPoller struct {
	key        runObservationPollerKey
	refs       map[*runObservationLease]struct{}
	phase      runObservationPollerPhase
	initResult runObservationInitializerResult
	current    *runObservationRetainedState
	ready      chan struct{}
	ctx        context.Context
	cancel     context.CancelFunc
	workerDone chan struct{}
}

type runObservationLease struct {
	manager     *runObservationManager
	pollers     map[runObservationPollerKey]*runObservationPoller
	listState   contract.RunTraceListState
	watermarks  map[runObservationPollerKey]runObservationWatermark
	caller      runObservationCaller
	asOfCursor  string
	draining    bool
	creatorDone chan struct{}
	leaseDone   chan struct{}
	drainDone   chan struct{}
}

func newRunObservationManager(
	list runObservationListReader,
	traces ports.RunTracePort,
	sessions runObservationSessionCurrent,
	verifier PerspectiveVerifier,
	authorize runObservationAuthorize,
	guard runObservationGuard,
	now func() time.Time,
) *runObservationManager {
	if now == nil {
		now = time.Now
	}
	return &runObservationManager{
		pollers: make(map[runObservationPollerKey]*runObservationPoller),
		cohorts: make(map[*runObservationLease]struct{}),
		list:    list, traces: traces, sessions: sessions, verifier: verifier,
		authorize: authorize, guard: guard, now: now,
	}
}

func (m *runObservationManager) beginPrepareOperation(caller runObservationCaller, asOfCursor string) (*runObservationLease, error) {
	if m == nil {
		return nil, runObservationUnavailable("run observation manager is unavailable")
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.stopping || len(m.cohorts) >= runObservationCohortLimit {
		return nil, runObservationUnavailable("run observation cohort capacity is unavailable")
	}
	lease := &runObservationLease{
		manager: m, pollers: make(map[runObservationPollerKey]*runObservationPoller),
		watermarks: make(map[runObservationPollerKey]runObservationWatermark),
		caller:     caller, asOfCursor: asOfCursor,
		creatorDone: make(chan struct{}), leaseDone: make(chan struct{}), drainDone: make(chan struct{}),
	}
	m.cohorts[lease] = struct{}{}
	m.prepareOperations.Add(1)
	return lease, nil
}

func (m *runObservationManager) finishPrepareOperation(lease *runObservationLease) {
	if lease == nil {
		return
	}
	close(lease.creatorDone)
	m.prepareOperations.Done()
}

// reservePollerBatch atomically reserves or attaches this Prepare's selected
// work and returns the immutable entries it owns to launch.
func (m *runObservationManager) reservePollerBatch(
	lease *runObservationLease,
	rows []ports.RunSummary,
	prepareDone <-chan struct{},
) ([]*runObservationPoller, error) {
	if m == nil || lease == nil || lease.manager != m || prepareDone == nil {
		return nil, runObservationUnavailable("run observation prepare is unavailable")
	}
	candidates := make(map[runObservationPollerKey]*runObservationPoller, len(rows))
	for _, row := range rows {
		key := runObservationPollerKey{caseID: row.CaseID, runID: row.RunID, planItemID: row.PlanItemID}
		if key.caseID == "" || key.runID == "" || key.planItemID == "" {
			continue
		}
		if _, exists := candidates[key]; exists {
			continue
		}
		if _, err := marshalRunObservationPollerImage(key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil); err != nil {
			continue
		}
		workerCtx, cancel := context.WithCancel(context.Background())
		candidates[key] = &runObservationPoller{
			key: key, refs: make(map[*runObservationLease]struct{}),
			phase:      runObservationPollerPhaseInitializing,
			initResult: runObservationInitializerPending,
			ready:      make(chan struct{}), ctx: workerCtx, cancel: cancel,
			workerDone: make(chan struct{}),
		}
	}

	batch := make([]*runObservationPoller, 0, len(candidates))
	used := make(map[*runObservationPoller]struct{}, len(candidates))
	m.mu.Lock()
	select {
	case <-prepareDone:
		m.mu.Unlock()
		for _, candidate := range candidates {
			candidate.cancel()
		}
		return nil, runObservationUnavailable("run observation prepare was canceled")
	default:
	}
	if m.stopping || lease.draining {
		m.mu.Unlock()
		for _, candidate := range candidates {
			candidate.cancel()
		}
		return nil, runObservationUnavailable("run observation prepare was stopped")
	}
	if _, exists := m.cohorts[lease]; !exists {
		m.mu.Unlock()
		for _, candidate := range candidates {
			candidate.cancel()
		}
		return nil, runObservationUnavailable("run observation prepare is unavailable")
	}
	for _, row := range rows {
		key := runObservationPollerKey{caseID: row.CaseID, runID: row.RunID, planItemID: row.PlanItemID}
		candidate := candidates[key]
		if candidate == nil {
			continue
		}
		entry := m.pollers[key]
		if entry != nil && entry.key != key {
			continue
		}
		if entry == nil {
			if len(m.pollers) >= runObservationPollerLimit || len(lease.pollers) >= observationRunLimit || managerAttachmentCount(m) >= runObservationCohortLimit*observationRunLimit {
				continue
			}
			entry = candidate
			m.pollers[key] = entry
			batch = append(batch, entry)
			used[entry] = struct{}{}
		} else if entry.phase == runObservationPollerPhaseInitializing && entry.initResult == runObservationInitializerPending {
			if len(lease.pollers) >= observationRunLimit || managerAttachmentCount(m) >= runObservationCohortLimit*observationRunLimit {
				continue
			}
		} else if entry.phase != runObservationPollerPhaseRunning || entry.current == nil {
			continue
		} else if len(lease.pollers) >= observationRunLimit || managerAttachmentCount(m) >= runObservationCohortLimit*observationRunLimit {
			continue
		}
		if _, exists := entry.refs[lease]; exists {
			continue
		}
		entry.refs[lease] = struct{}{}
		lease.pollers[key] = entry
	}
	m.mu.Unlock()
	for _, candidate := range candidates {
		if _, retained := used[candidate]; !retained {
			candidate.cancel()
		}
	}
	return batch, nil
}

func managerAttachmentCount(m *runObservationManager) int {
	count := 0
	for _, entry := range m.pollers {
		if entry != nil {
			count += len(entry.refs)
		}
	}
	return count
}

func (m *runObservationManager) releasePollerRef(lease *runObservationLease, entry *runObservationPoller) bool {
	if m == nil || lease == nil || entry == nil {
		return false
	}
	var cancel context.CancelFunc
	m.mu.Lock()
	if lease.pollers[entry.key] != entry {
		m.mu.Unlock()
		return false
	}
	delete(lease.pollers, entry.key)
	if _, exists := entry.refs[lease]; !exists {
		m.mu.Unlock()
		return false
	}
	if hasEligiblePollerRef(entry) {
		delete(entry.refs, lease)
	} else if entry.phase != runObservationPollerPhaseStopping && entry.phase != runObservationPollerPhaseDraining {
		entry.phase = runObservationPollerPhaseDraining
		cancel = entry.cancel
	}
	m.mu.Unlock()
	if cancel != nil {
		cancel()
	}
	return true
}

func hasEligiblePollerRef(entry *runObservationPoller) bool {
	for lease := range entry.refs {
		if lease != nil && !lease.draining && lease.pollers[entry.key] == entry {
			return true
		}
	}
	return false
}

func (m *runObservationManager) finishSelectedPollerRelease(lease *runObservationLease, entry *runObservationPoller) {
	m.mu.Lock()
	_, retained := entry.refs[lease]
	m.mu.Unlock()
	if !retained {
		return
	}
	<-entry.workerDone
	m.mu.Lock()
	if _, exists := entry.refs[lease]; exists {
		delete(entry.refs, lease)
	}
	if len(entry.refs) == 0 && m.pollers[entry.key] == entry {
		delete(m.pollers, entry.key)
	}
	m.mu.Unlock()
}

func runObservationCallerFromRequest(identity *requestIdentity) (runObservationCaller, error) {
	if identity == nil {
		return runObservationCaller{}, runObservationUnavailable("authenticated request identity is unavailable")
	}
	caller := runObservationCaller{source: identity.Source, claim: identity.Identity.Claim()}
	if identity.Session != nil {
		caller.sessionID = identity.Session.ID
	}
	if (caller.source != "session" && caller.source != "bearer") ||
		caller.claim.ActorID == "" || caller.claim.Role == "" ||
		(caller.source == "session" && (identity.Session == nil || strings.TrimSpace(caller.sessionID) == "")) ||
		(caller.source == "bearer" && identity.Session != nil) {
		return runObservationCaller{}, runObservationUnavailable("authenticated request identity is unavailable")
	}
	return caller, nil
}

func (m *runObservationManager) prepare(
	ctx context.Context,
	caseID string,
	identity *requestIdentity,
) (contract.RunTraceObservationEvent, *runObservationLease, error) {
	if ctx == nil {
		ctx = context.Background()
	}
	caller, err := runObservationCallerFromRequest(identity)
	if err != nil {
		return contract.RunTraceObservationEvent{}, nil, err
	}
	if m == nil || m.authorize == nil || m.guard == nil || m.list == nil || m.traces == nil {
		return contract.RunTraceObservationEvent{}, nil, runObservationUnavailable("run observation manager is unavailable")
	}
	if ctx.Err() != nil {
		return contract.RunTraceObservationEvent{}, nil, runObservationUnavailable("run observation prepare was canceled")
	}
	if err := m.authorize(ctx, caller, caseID, m.sessions, m.verifier); err != nil {
		return contract.RunTraceObservationEvent{}, nil, err
	}
	present, err := m.guard(caseID)
	if err != nil {
		return contract.RunTraceObservationEvent{}, nil, err
	}
	lease, err := m.beginPrepareOperation(caller, present.cursor)
	if err != nil {
		return contract.RunTraceObservationEvent{}, nil, err
	}
	prepareCtx, cancelPrepare := context.WithCancel(ctx)
	prepareDone := prepareCtx.Done()
	bridgeDone := make(chan struct{})
	go func() {
		defer close(bridgeDone)
		select {
		case <-lease.leaseDone:
			cancelPrepare()
		case <-prepareDone:
		}
	}()
	var batch []*runObservationPoller
	launched := false
	finished := false
	finishCreator := func() {
		if finished {
			return
		}
		cancelPrepare()
		<-bridgeDone
		m.finishPrepareOperation(lease)
		finished = true
	}
	failPrepare := func(failure error) (contract.RunTraceObservationEvent, *runObservationLease, error) {
		m.startLeaseDrain(lease, false)
		if !launched {
			m.launchPollerBatch(batch)
			launched = true
		}
		finishCreator()
		<-lease.drainDone
		return contract.RunTraceObservationEvent{}, nil, failure
	}
	if prepareCtx.Err() != nil {
		return failPrepare(runObservationUnavailable("run observation prepare was canceled"))
	}
	list, listErr := m.list.RunsListForCase(prepareCtx, ports.CaseRef(caseID))
	if prepareCtx.Err() != nil {
		return failPrepare(runObservationUnavailable("run observation prepare was canceled"))
	}
	currentPresent, currentValid := m.authorizeLeasePresent(prepareCtx, lease, caseID, nil)
	if !currentValid || currentPresent.cursor != present.cursor {
		return failPrepare(runObservationUnavailable("present case context or authorization changed during observation prepare"))
	}
	if listErr != nil {
		listState := contract.RunTraceListState("unavailable")
		m.mu.Lock()
		_, active := m.cohorts[lease]
		if m.stopping || lease.draining || !active {
			m.mu.Unlock()
			return failPrepare(runObservationUnavailable("run observation prepare was stopped"))
		}
		lease.listState = listState
		m.mu.Unlock()
		if !m.leaseCanPrepare(lease, prepareDone) {
			return failPrepare(runObservationUnavailable("run observation prepare was stopped"))
		}
		observedAt := m.now().UTC().Format(time.RFC3339Nano)
		unavailable, buildErr := m.initialObservationEvent(caseID, present.cursor, observedAt, nil, string(listState), 0, 0, 0, 0, 0, 0, 0, false)
		if buildErr != nil {
			return failPrepare(buildErr)
		}
		unavailable.Payload, buildErr = boundRunObservationHydration(unavailable.Payload)
		if buildErr != nil {
			return failPrepare(buildErr)
		}
		if _, valid := m.authorizeLeasePresent(prepareCtx, lease, caseID, nil); !valid || !m.leaseCanPrepare(lease, prepareDone) {
			return failPrepare(runObservationUnavailable("run observation prepare was stopped or canceled"))
		}
		finishCreator()
		return unavailable, lease, nil
	}
	if prepareCtx.Err() != nil {
		return failPrepare(runObservationUnavailable("run observation prepare was canceled"))
	}
	listState := contract.RunTraceListState("complete")
	m.mu.Lock()
	_, active := m.cohorts[lease]
	if m.stopping || lease.draining || !active {
		m.mu.Unlock()
		return failPrepare(runObservationUnavailable("run observation prepare was stopped"))
	}
	lease.listState = listState
	m.mu.Unlock()
	selected, err := selectObservationRuns(list.Runs)
	if err != nil {
		return failPrepare(runObservationUnavailable("run list contained an unavailable candidate"))
	}
	selectedCount := len(selected)
	type selectedRun struct {
		row ports.RunSummary
		key runObservationPollerKey
		bad bool
	}
	selectedRuns := make([]selectedRun, len(selected))
	validRows := make([]ports.RunSummary, 0, len(selected))
	seen := make(map[runObservationPollerKey]struct{}, len(selected))
	for i, row := range selected {
		key := runObservationPollerKey{caseID: row.CaseID, runID: row.RunID, planItemID: row.PlanItemID}
		bad := row.CaseID != caseID || row.RunID == "" || row.PlanItemID == ""
		if _, ok := currentPresent.parentIDs[row.PlanItemID]; !ok {
			bad = true
		}
		if _, duplicate := seen[key]; duplicate {
			bad = true
		}
		seen[key] = struct{}{}
		if !bad {
			if _, imageErr := marshalRunObservationPollerImage(key, runObservationPollerPhaseInitializing, runObservationInitializerPending, nil); imageErr != nil {
				bad = true
			}
		}
		selectedRuns[i] = selectedRun{row: row, key: key, bad: bad}
		if !bad {
			validRows = append(validRows, row)
		}
	}
	parentIDs := make([]string, 0, len(validRows))
	for _, row := range validRows {
		parentIDs = append(parentIDs, row.PlanItemID)
	}
	if _, valid := m.authorizeLeasePresent(prepareCtx, lease, caseID, parentIDs); !valid {
		return failPrepare(runObservationUnavailable("run observation authorization or present context changed before attachment"))
	}
	batch, err = m.reservePollerBatch(lease, validRows, prepareDone)
	if err != nil {
		return failPrepare(err)
	}
	if prepareCtx.Err() != nil || !m.leaseCanPrepare(lease, prepareDone) {
		return failPrepare(runObservationUnavailable("run observation prepare was stopped or canceled"))
	}
	m.launchPollerBatch(batch)
	launched = true
	entries := make(map[runObservationPollerKey]*runObservationPoller, len(validRows))
	for _, row := range validRows {
		key := runObservationPollerKey{caseID: row.CaseID, runID: row.RunID, planItemID: row.PlanItemID}
		m.mu.Lock()
		entry := lease.pollers[key]
		m.mu.Unlock()
		if entry == nil {
			continue
		}
		entries[key] = entry
	}
	waited := make(map[*runObservationPoller]struct{}, len(entries))
	for _, entry := range entries {
		if _, ok := waited[entry]; ok {
			continue
		}
		waited[entry] = struct{}{}
		select {
		case <-entry.ready:
		case <-prepareDone:
			return failPrepare(runObservationUnavailable("run observation prepare was canceled"))
		}
	}
	if prepareCtx.Err() != nil || !m.leaseCanPrepare(lease, prepareDone) {
		return failPrepare(runObservationUnavailable("run observation prepare was stopped or canceled"))
	}
	readsAttempted := 0
	for _, entry := range batch {
		m.mu.Lock()
		phase, result := entry.phase, entry.initResult
		m.mu.Unlock()
		if phase == runObservationPollerPhaseRunning && result != runObservationInitializerPending {
			readsAttempted++
			continue
		}
		return failPrepare(runObservationUnavailable("run observation initializer stopped before its first read"))
	}
	validatedRuns := make([]contract.RunTraceRunObservation, 0, selectedCount)
	validatedStates := make(map[runObservationPollerKey]*runObservationRetainedState, selectedCount)
	unavailableCount := 0
	capacityCount := 0
	for _, selected := range selectedRuns {
		if selected.bad {
			unavailableCount++
			continue
		}
		entry := entries[selected.key]
		if entry == nil {
			capacityCount++
			continue
		}
		m.mu.Lock()
		_, cohortActive := m.cohorts[lease]
		if m.stopping || lease.draining || !cohortActive ||
			m.pollers[selected.key] != entry || lease.pollers[selected.key] != entry {
			m.mu.Unlock()
			return failPrepare(runObservationUnavailable("run observation prepare was stopped"))
		}
		if _, referenced := entry.refs[lease]; !referenced {
			m.mu.Unlock()
			return failPrepare(runObservationUnavailable("run observation prepare was stopped"))
		}
		state := entry.current
		valid := entry.phase == runObservationPollerPhaseRunning && state != nil && state.Key == selected.key && state.Availability == retainedCurrent
		if valid {
			lease.watermarks[selected.key] = initialRunObservationWatermark(state)
		}
		m.mu.Unlock()
		if !valid {
			unavailableCount++
			if m.releasePollerRef(lease, entry) {
				m.finishSelectedPollerRelease(lease, entry)
			}
			continue
		}
		validatedRuns = append(validatedRuns, runObservationInitialRun(state))
		validatedStates[selected.key] = state
	}
	if prepareCtx.Err() != nil || !m.leaseCanPrepare(lease, prepareDone) {
		return failPrepare(runObservationUnavailable("run observation prepare was stopped or canceled"))
	}
	readableCount := len(list.Runs)
	omittedCount := readableCount - selectedCount + capacityCount
	observationState := "complete"
	if len(list.UnreadableIDs) > 0 || unavailableCount > 0 {
		observationState = "unavailable"
	} else if omittedCount > 0 {
		observationState = "capacity_limited"
	} else if readableCount == 0 {
		observationState = "no_runs"
	}
	observedAt := m.now().UTC().Format(time.RFC3339Nano)
	event, err := m.initialObservationEvent(
		caseID, present.cursor, observedAt, validatedRuns, string(listState),
		readableCount, selectedCount, len(validatedRuns), len(list.UnreadableIDs),
		unavailableCount, omittedCount, readsAttempted, readsAttempted == observationRunLimit && readableCount > observationRunLimit,
	)
	if err != nil {
		return failPrepare(err)
	}
	event.Payload.ObservationState = contract.RunTraceObservationState(observationState)
	// Re-run the canonical helper once with the final outer status value.
	event.Payload, err = boundRunObservationHydration(event.Payload)
	if err != nil {
		return failPrepare(err)
	}
	disclosedParents := make([]string, 0, len(validatedRuns))
	for _, run := range validatedRuns {
		disclosedParents = append(disclosedParents, run.PlanItemID)
	}
	if _, valid := m.authorizeLeasePresent(prepareCtx, lease, caseID, disclosedParents); !valid || !m.leaseCanDisclose(lease, validatedStates, prepareDone) {
		return failPrepare(runObservationUnavailable("run observation prepare was stopped or canceled"))
	}
	finishCreator()
	return event, lease, nil
}

func (m *runObservationManager) leaseCanPrepare(lease *runObservationLease, prepareDone <-chan struct{}) bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	select {
	case <-prepareDone:
		return false
	default:
	}
	if m.stopping || lease == nil || lease.draining {
		return false
	}
	_, exists := m.cohorts[lease]
	return exists
}

func (m *runObservationManager) authorizeLeasePresent(
	ctx context.Context,
	lease *runObservationLease,
	caseID string,
	parentIDs []string,
) (runObservationPresentContext, bool) {
	if m == nil || lease == nil || m.authorize == nil || m.guard == nil || ctx == nil || ctx.Err() != nil {
		return runObservationPresentContext{}, false
	}
	if err := m.authorize(ctx, lease.caller, caseID, m.sessions, m.verifier); err != nil || ctx.Err() != nil {
		return runObservationPresentContext{}, false
	}
	present, err := m.guard(caseID)
	if err != nil || ctx.Err() != nil || present.caseID != caseID || present.cursor != lease.asOfCursor {
		return runObservationPresentContext{}, false
	}
	for _, parentID := range parentIDs {
		if _, exists := present.parentIDs[parentID]; !exists {
			return runObservationPresentContext{}, false
		}
	}
	if ctx.Err() != nil {
		return runObservationPresentContext{}, false
	}
	return present, true
}

func (m *runObservationManager) leaseCanDisclose(
	lease *runObservationLease,
	states map[runObservationPollerKey]*runObservationRetainedState,
	prepareDone <-chan struct{},
) bool {
	if m == nil || lease == nil {
		return false
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	select {
	case <-prepareDone:
		return false
	default:
	}
	if m.stopping || lease.draining {
		return false
	}
	if _, exists := m.cohorts[lease]; !exists {
		return false
	}
	for key, state := range states {
		entry := m.pollers[key]
		if entry == nil || lease.pollers[key] != entry || entry.current != state || entry.phase != runObservationPollerPhaseRunning || state == nil || state.Key != key || state.Availability != retainedCurrent {
			return false
		}
		if _, exists := entry.refs[lease]; !exists || lease.draining {
			return false
		}
	}
	return true
}

func initialRunObservationWatermark(state *runObservationRetainedState) runObservationWatermark {
	retained := len(state.Frames)
	omitted := state.TotalFrameCount - retained
	return runObservationWatermark{
		highestObservedOrdinal: state.HighestOrdinal,
		execution:              contract.RunExecutionStanding(state.Execution),
		settlement:             contract.RunSettlementStanding(state.Settlement),
		observationState:       contract.RunTraceRunObservationState(state.ObservationState),
		window: runObservationDeltaWindow{
			generation:         state.Generation,
			totalFrameCount:    state.TotalFrameCount,
			retainedFrameCount: retained,
			omittedFrameCount:  omitted,
			truncated:          omitted > 0,
		},
	}
}

func runObservationInitialRun(state *runObservationRetainedState) contract.RunTraceRunObservation {
	frames := make([]contract.RunTraceFrame, len(state.Frames))
	for i, retained := range state.Frames {
		frame := retained.Frame
		frames[i] = contract.RunTraceFrame{
			EventID: frame.EventID, Kind: contract.RunTraceFrameKind(frame.Kind), Timestamp: frame.Timestamp,
		}
		if frame.ExecutionStatus != nil {
			status := contract.RunTraceCommandExecutionStatus(*frame.ExecutionStatus)
			frames[i].ExecutionStatus = &status
		}
		if frame.ExitCode != nil {
			code := *frame.ExitCode
			frames[i].ExitCode = &code
		}
	}
	omitted := state.TotalFrameCount - len(frames)
	if omitted < 0 {
		omitted = 0
	}
	return contract.RunTraceRunObservation{
		RunID: state.Key.runID, ObservedAt: state.AcceptedAt, PlanItemID: state.Key.planItemID,
		Execution:        contract.RunExecutionStanding(state.Execution),
		Settlement:       contract.RunSettlementStanding(state.Settlement),
		ObservationState: contract.RunTraceRunObservationState(state.ObservationState),
		Frames:           frames, TotalFrameCount: state.TotalFrameCount,
		RetainedFrameCount: len(frames), OmittedFrameCount: omitted, Truncated: omitted > 0,
	}
}

func (m *runObservationManager) initialObservationEvent(
	caseID, cursor, observedAt string,
	runs []contract.RunTraceRunObservation,
	listState string,
	listed, selected, validated, unreadable, unavailable, omitted, reads int,
	exhausted bool,
) (contract.RunTraceObservationEvent, error) {
	if runs == nil {
		runs = []contract.RunTraceRunObservation{}
	}
	payload := contract.RunTraceObservation{
		CaseID: caseID, ObservedAt: observedAt,
		RunListState:        contract.RunTraceListState(listState),
		HydrationReadBudget: contract.RunTraceHydrationReadBudget{Limit: observationRunLimit, ReadsAttempted: reads, Exhausted: exhausted},
		Runs:                runs,
	}
	if listState == "complete" {
		payload.ListedRunCount = observationCount(listed)
		payload.SelectedRunCount = observationCount(selected)
		payload.ValidatedRunCount = observationCount(validated)
		payload.UnreadableRunCount = observationCount(unreadable)
		payload.UnavailableRunCount = observationCount(unavailable)
		payload.OmittedRunCount = observationCount(omitted)
		payload.ObservationState = "complete"
	} else {
		payload.ObservationState = "unavailable"
	}
	return contract.RunTraceObservationEvent{
		EventType: contract.RunTraceObservationEventType, Cursor: cursor,
		Timestamp: observedAt, Payload: payload,
	}, nil
}

func observationCount(value int) *int { return &value }

type runObservationDrainWork struct {
	lease   *runObservationLease
	entries []*runObservationPoller
	cancels []context.CancelFunc
}

func (m *runObservationManager) takeLeaseDrainLocked(lease *runObservationLease, stopping bool) runObservationDrainWork {
	work := runObservationDrainWork{lease: lease}
	if lease == nil || lease.draining {
		return work
	}
	lease.draining = true
	entries := make(map[*runObservationPoller]struct{})
	for key, entry := range lease.pollers {
		delete(lease.pollers, key)
		if entry != nil {
			entries[entry] = struct{}{}
		}
	}
	for _, entry := range m.pollers {
		if entry != nil {
			if _, owns := entry.refs[lease]; owns {
				entries[entry] = struct{}{}
			}
		}
	}
	for entry := range entries {
		if hasEligiblePollerRef(entry) {
			delete(entry.refs, lease)
			continue
		}
		if entry.phase != runObservationPollerPhaseStopping && entry.phase != runObservationPollerPhaseDraining {
			if stopping || m.stopping {
				entry.phase = runObservationPollerPhaseStopping
			} else {
				entry.phase = runObservationPollerPhaseDraining
			}
			if entry.cancel != nil {
				work.cancels = append(work.cancels, entry.cancel)
			}
		}
		work.entries = append(work.entries, entry)
	}
	return work
}

func (m *runObservationManager) startLeaseDrain(lease *runObservationLease, stopping bool) <-chan struct{} {
	if m == nil || lease == nil || lease.drainDone == nil {
		return closedRunObservationSignal()
	}
	m.mu.Lock()
	if lease.draining {
		done := lease.drainDone
		m.mu.Unlock()
		return done
	}
	work := m.takeLeaseDrainLocked(lease, stopping)
	m.mu.Unlock()
	close(lease.leaseDone)
	for _, cancel := range work.cancels {
		cancel()
	}
	go m.finishLeaseDrain(work)
	return lease.drainDone
}

func closedRunObservationSignal() <-chan struct{} {
	done := make(chan struct{})
	close(done)
	return done
}

func (m *runObservationManager) finishLeaseDrain(work runObservationDrainWork) {
	lease := work.lease
	<-lease.creatorDone
	for _, entry := range work.entries {
		if entry.workerDone != nil {
			<-entry.workerDone
		}
	}
	m.mu.Lock()
	for key, entry := range m.pollers {
		if entry == nil {
			continue
		}
		if _, owns := entry.refs[lease]; owns {
			delete(entry.refs, lease)
		}
		if len(entry.refs) == 0 && entry.workerDone != nil {
			select {
			case <-entry.workerDone:
				if m.pollers[key] == entry {
					delete(m.pollers, key)
				}
			default:
			}
		}
	}
	delete(m.cohorts, lease)
	m.mu.Unlock()
	close(lease.drainDone)
}

func waitRunObservationDrain(ctx context.Context, done <-chan struct{}) error {
	if ctx == nil {
		ctx = context.Background()
	}
	select {
	case <-done:
		return nil
	case <-ctx.Done():
		return runObservationUnavailable("run observation drain is still in progress")
	}
}

func (m *runObservationManager) stopAndDrain(ctx context.Context) error {
	if m == nil {
		return runObservationUnavailable("run observation manager is unavailable")
	}
	if ctx == nil {
		ctx = context.Background()
	}
	var owner bool
	var works []runObservationDrainWork
	var leases []*runObservationLease
	var entries []*runObservationPoller
	var stopCancels []context.CancelFunc
	m.mu.Lock()
	if m.stopDone == nil {
		owner = true
		m.stopping = true
		m.stopDone = make(chan struct{})
		for lease := range m.cohorts {
			leases = append(leases, lease)
			if !lease.draining {
				works = append(works, m.takeLeaseDrainLocked(lease, true))
			}
		}
		for _, entry := range m.pollers {
			if entry == nil {
				continue
			}
			entries = append(entries, entry)
			if !hasEligiblePollerRef(entry) && entry.phase != runObservationPollerPhaseStopping && entry.phase != runObservationPollerPhaseDraining {
				entry.phase = runObservationPollerPhaseStopping
				if entry.cancel != nil {
					stopCancels = append(stopCancels, entry.cancel)
				}
			}
		}
	}
	done := m.stopDone
	m.mu.Unlock()
	if owner {
		for _, work := range works {
			close(work.lease.leaseDone)
		}
		for _, cancel := range stopCancels {
			cancel()
		}
		for _, work := range works {
			for _, cancel := range work.cancels {
				cancel()
			}
			go m.finishLeaseDrain(work)
		}
		go m.finishManagerStop(leases, entries, done)
	}
	return waitRunObservationDrain(ctx, done)
}

func (m *runObservationManager) finishManagerStop(leases []*runObservationLease, entries []*runObservationPoller, done chan struct{}) {
	m.prepareOperations.Wait()
	for _, lease := range leases {
		<-lease.drainDone
	}
	for _, entry := range entries {
		if entry.workerDone != nil {
			<-entry.workerDone
		}
	}
	m.mu.Lock()
	for key, entry := range m.pollers {
		if entry != nil && len(entry.refs) == 0 && entry.workerDone != nil {
			select {
			case <-entry.workerDone:
				if m.pollers[key] == entry {
					delete(m.pollers, key)
				}
			default:
			}
		}
	}
	m.mu.Unlock()
	close(done)
}

func (l *runObservationLease) detachAndDrain(ctx context.Context) error {
	if l == nil || l.manager == nil {
		return runObservationUnavailable("run observation lease is unavailable")
	}
	done := l.manager.startLeaseDrain(l, false)
	return waitRunObservationDrain(ctx, done)
}

func runObservationUnavailable(message string) error {
	return apperr.New(apperr.KindUnavailable, "", "run_observation", message)
}
