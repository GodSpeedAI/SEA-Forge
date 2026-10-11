package server

import (
	"context"
	"sort"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/contract"
)

// runObservationNextAggregate is the private transaction result. Its slices
// are nonnil even when the complete cohort has no runs.
type runObservationNextAggregate struct {
	CaseID     string
	AsOfCursor string
	ObservedAt time.Time
	Runs       []runObservationRunDelta
	WindowGaps []runObservationWindowGap
}

type runObservationRunProjector func(
	*runObservationRetainedState,
	map[string]uint64,
	runObservationWatermark,
) (runObservationRunDelta, *runObservationWindowGap, runObservationWatermark, error)

type runObservationNextAlreadyInProgressError struct{}

func (*runObservationNextAlreadyInProgressError) Error() string {
	return "run observation Next is already in progress"
}

func (l *runObservationLease) next(ctx context.Context) (runObservationNextAggregate, error) {
	return l.nextWithProjector(ctx, buildRunObservationRunDelta)
}

// nextWithProjector is the transaction seam used by deterministic tests. The
// production path always supplies buildRunObservationRunDelta.
func (l *runObservationLease) nextWithProjector(
	ctx context.Context,
	project runObservationRunProjector,
) (runObservationNextAggregate, error) {
	zero := runObservationNextAggregate{}
	if l == nil || l.manager == nil {
		return zero, runObservationUnavailable("run observation lease is unavailable")
	}
	if ctx == nil {
		ctx = context.Background()
	}
	if project == nil {
		project = buildRunObservationRunDelta
	}
	m := l.manager
	token := &struct{ marker byte }{marker: 1}
	m.mu.Lock()
	if l.nextInFlight {
		m.mu.Unlock()
		return zero, &runObservationNextAlreadyInProgressError{}
	}
	if m.stopping || l.draining || l.caseID == "" {
		m.mu.Unlock()
		return zero, runObservationUnavailable("run observation lease is unavailable")
	}
	if _, active := m.cohorts[l]; !active {
		m.mu.Unlock()
		return zero, runObservationUnavailable("run observation lease is unavailable")
	}
	l.nextInFlight = true
	l.nextToken = token
	l.operations.Add(1)
	caseID := l.caseID
	listState := l.listState
	parentIDs := make([]string, 0, len(l.pollers))
	for key := range l.pollers {
		parentIDs = append(parentIDs, key.planItemID)
	}
	m.mu.Unlock()
	defer func() {
		m.mu.Lock()
		if l.nextToken == token {
			l.nextToken = nil
			l.nextInFlight = false
		}
		m.mu.Unlock()
		l.operations.Done()
	}()

	if listState != contract.RunTraceListState("complete") {
		m.startLeaseDrain(l, false)
		return zero, runObservationUnavailable("run list is unavailable")
	}
	terminalFailure := func(message string) (runObservationNextAggregate, error) {
		m.startLeaseDrain(l, false)
		return zero, runObservationUnavailable(message)
	}
	if ctx.Err() != nil {
		return terminalFailure("run observation Next was canceled")
	}
	if _, valid := m.authorizeLeasePresent(ctx, l, caseID, parentIDs); !valid {
		return terminalFailure("run observation authorization or present context changed")
	}
	select {
	case <-ctx.Done():
		return terminalFailure("run observation Next was canceled")
	case <-l.leaseDone:
		return zero, runObservationUnavailable("run observation lease is draining")
	case <-l.wake:
	}
	if ctx.Err() != nil {
		return terminalFailure("run observation Next was canceled")
	}
	if _, valid := m.authorizeLeasePresent(ctx, l, caseID, parentIDs); !valid {
		return terminalFailure("run observation authorization or present context changed")
	}

	type capturedRun struct {
		key    runObservationPollerKey
		entry  *runObservationPoller
		state  *runObservationRetainedState
		ledger map[string]uint64
		prior  runObservationWatermark
	}
	m.mu.Lock()
	if m.stopping || l.draining || l.nextToken != token || !l.nextInFlight {
		m.mu.Unlock()
		return zero, runObservationUnavailable("run observation lease is draining")
	}
	if _, active := m.cohorts[l]; !active {
		m.mu.Unlock()
		return zero, runObservationUnavailable("run observation lease is unavailable")
	}
	keys := make([]runObservationPollerKey, 0, len(l.pollers))
	for key := range l.pollers {
		keys = append(keys, key)
	}
	sort.Slice(keys, func(i, j int) bool { return keys[i].runID < keys[j].runID })
	captured := make([]capturedRun, 0, len(keys))
	for _, key := range keys {
		entry := l.pollers[key]
		if entry == nil || entry.key != key || m.pollers[key] != entry || entry.refs == nil {
			m.mu.Unlock()
			return terminalFailure("run observation attachment changed")
		}
		if _, attached := entry.refs[l]; !attached {
			m.mu.Unlock()
			return terminalFailure("run observation attachment changed")
		}
		state := entry.current
		if state == nil || state.Key != key {
			m.mu.Unlock()
			return terminalFailure("run observation retained state is unavailable")
		}
		if state.Availability == retainedReadUnavailable || state.Availability == retainedRetentionUnavailable {
			if ctx.Err() != nil {
				m.mu.Unlock()
				return terminalFailure("run observation Next was canceled")
			}
			m.mu.Unlock()
			return zero, runObservationUnavailable("run observation retained state is temporarily unavailable")
		}
		if state.Availability == retainedTerminalRetentionFailure || state.Availability == retainedStopScheduling {
			m.mu.Unlock()
			return terminalFailure("run observation retained state is terminal")
		}
		if state.Availability != retainedCurrent {
			m.mu.Unlock()
			return terminalFailure("run observation retained state is unavailable")
		}
		ledger := make(map[string]uint64, len(state.SeenByID))
		for id, ordinal := range state.SeenByID {
			ledger[id] = ordinal
		}
		captured = append(captured, capturedRun{key: key, entry: entry, state: state, ledger: ledger, prior: l.watermarks[key]})
	}
	m.mu.Unlock()

	aggregate := runObservationNextAggregate{CaseID: caseID, AsOfCursor: l.asOfCursor, ObservedAt: m.now().UTC(), Runs: make([]runObservationRunDelta, 0, len(captured)), WindowGaps: make([]runObservationWindowGap, 0)}
	candidates := make(map[runObservationPollerKey]runObservationWatermark, len(captured))
	for _, item := range captured {
		if ctx.Err() != nil {
			return terminalFailure("run observation Next was canceled")
		}
		delta, gap, candidate, err := project(item.state, item.ledger, item.prior)
		if err != nil {
			return terminalFailure("run observation delta projection is unavailable")
		}
		aggregate.Runs = append(aggregate.Runs, delta)
		if gap != nil {
			aggregate.WindowGaps = append(aggregate.WindowGaps, *gap)
		}
		candidates[item.key] = candidate
	}
	if ctx.Err() != nil {
		return terminalFailure("run observation Next was canceled")
	}
	if _, valid := m.authorizeLeasePresent(ctx, l, caseID, parentIDs); !valid {
		return terminalFailure("run observation authorization or present context changed")
	}
	if ctx.Err() != nil {
		return terminalFailure("run observation Next was canceled")
	}
	m.mu.Lock()
	if m.stopping || l.draining || l.nextToken != token || !l.nextInFlight || ctx.Err() != nil {
		m.mu.Unlock()
		return terminalFailure("run observation Next was stopped")
	}
	if _, active := m.cohorts[l]; !active {
		m.mu.Unlock()
		return terminalFailure("run observation lease is unavailable")
	}
	for _, item := range captured {
		if m.pollers[item.key] != item.entry || l.pollers[item.key] != item.entry || item.entry.key != item.key {
			m.mu.Unlock()
			return terminalFailure("run observation attachment changed before commit")
		}
		if _, attached := item.entry.refs[l]; !attached {
			m.mu.Unlock()
			return terminalFailure("run observation attachment changed before commit")
		}
	}
	for key, candidate := range candidates {
		l.watermarks[key] = candidate
	}
	m.mu.Unlock()
	return aggregate, nil
}
