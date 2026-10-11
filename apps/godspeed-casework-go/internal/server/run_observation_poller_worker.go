package server

import (
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/ports"
)

type runObservationNotifierTarget struct {
	lease *runObservationLease
	wake  chan struct{}
}

// acquirePollerNotifierTargetsLocked registers notification ownership while
// the manager lock prevents drain from beginning.
func (m *runObservationManager) acquirePollerNotifierTargetsLocked(entry *runObservationPoller) []runObservationNotifierTarget {
	if m == nil || entry == nil || m.pollers[entry.key] != entry {
		return nil
	}
	targets := make([]runObservationNotifierTarget, 0, len(entry.refs))
	for lease := range entry.refs {
		if lease == nil || lease.draining || lease.manager != m || lease.pollers[entry.key] != entry || lease.wake == nil {
			continue
		}
		lease.notifyWG.Add(1)
		targets = append(targets, runObservationNotifierTarget{lease: lease, wake: lease.wake})
	}
	return targets
}

// sendPollerNotifierTargets coalesces wakes and always releases each target.
func (m *runObservationManager) sendPollerNotifierTargets(targets []runObservationNotifierTarget) {
	for _, target := range targets {
		if target.lease == nil {
			continue
		}
		select {
		case target.wake <- struct{}{}:
		default:
		}
		target.lease.notifyWG.Done()
	}
}

// launchPollerBatch starts every reserved entry once, outside mu and before
// the creator performs any readiness wait.
func (m *runObservationManager) launchPollerBatch(entries []*runObservationPoller) {
	for _, entry := range entries {
		go m.runPoller(entry)
	}
}

// claimPollerReadStart is the real mu-protected read-eligibility transition
// shared by initial and recurring reads.
func (m *runObservationManager) claimPollerReadStart(entry *runObservationPoller) bool {
	if m == nil || entry == nil {
		return false
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.stopping || m.pollers[entry.key] != entry {
		return false
	}
	switch entry.phase {
	case runObservationPollerPhaseInitializing:
		if entry.initResult != runObservationInitializerPending || entry.current != nil {
			return false
		}
	case runObservationPollerPhaseRunning:
		if entry.initResult != runObservationInitializerAccepted || entry.current == nil {
			return false
		}
		if runObservationSnapshotIsTerminal(ports.RunTraceSnapshot{Execution: entry.current.Execution}) {
			return false
		}
		if entry.current.Availability == retainedTerminalRetentionFailure || entry.current.Availability == retainedStopScheduling {
			return false
		}
	default:
		return false
	}
	return hasEligiblePollerRef(entry)
}

// runPoller is the sole worker entry point and always hands the decision to the
// same continuation used by the deterministic production-boundary fixture.
func (m *runObservationManager) runPoller(entry *runObservationPoller) {
	eligible := m.claimPollerReadStart(entry)
	m.runPollerFromClaim(entry, eligible)
}

// runPollerFromClaim is the mandatory continuation after each eligibility
// decision. It owns the first readiness result and the workerDone close.
func (m *runObservationManager) runPollerFromClaim(entry *runObservationPoller, eligible bool) {
	defer close(entry.workerDone)
	initial := m.pollerInitializationPending(entry)
	for {
		if !eligible || entry.ctx.Err() != nil {
			m.stopPollerBeforeRead(entry, initial)
			return
		}
		previous, ok := m.pollerCurrentForRead(entry, initial)
		if !ok {
			m.stopPollerBeforeRead(entry, initial)
			return
		}
		watchers, ok := m.authorizePollerWatchers(entry, previous, initial)
		if !ok {
			m.stopPollerBeforeRead(entry, initial)
			return
		}
		m.mu.Lock()
		stillCurrent := m.pollerReadStateMatchesLocked(entry, previous, initial)
		newWatcher := false
		validWatcher := false
		for lease := range entry.refs {
			if lease != nil && !lease.draining && lease.pollers[entry.key] == entry {
				if _, checked := watchers[lease]; !checked {
					newWatcher = true
				} else {
					validWatcher = true
				}
			}
		}
		m.mu.Unlock()
		if !stillCurrent {
			m.stopPollerBeforeRead(entry, initial)
			return
		}
		if newWatcher {
			continue
		}
		if !validWatcher {
			m.stopPollerBeforeRead(entry, initial)
			return
		}
		// This outside-lock context check immediately follows the final
		// mutex-protected identity, phase, and watcher-membership check.
		if entry.ctx.Err() != nil {
			m.stopPollerBeforeRead(entry, initial)
			return
		}
		snapshot, readErr := m.traces.ReadRunTrace(entry.ctx, entry.key.caseID, entry.key.runID, entry.key.planItemID)
		if !m.finishPollerRead(entry, previous, initial, snapshot, readErr) {
			return
		}
		initial = false

		timer := time.NewTimer(time.Second)
		select {
		case <-entry.ctx.Done():
			if !timer.Stop() {
				select {
				case <-timer.C:
				default:
				}
			}
			m.stopPollerBeforeRead(entry, false)
			return
		case <-timer.C:
		}
		eligible = m.claimPollerReadStart(entry)
	}
}

func (m *runObservationManager) pollerInitializationPending(entry *runObservationPoller) bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	return entry.initResult == runObservationInitializerPending
}

func (m *runObservationManager) pollerCurrentForRead(entry *runObservationPoller, initial bool) (*runObservationRetainedState, bool) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if m.pollers[entry.key] != entry {
		return nil, false
	}
	if initial {
		return nil, entry.phase == runObservationPollerPhaseInitializing && entry.initResult == runObservationInitializerPending
	}
	if entry.phase != runObservationPollerPhaseRunning || entry.current == nil {
		return nil, false
	}
	return entry.current, true
}

func (m *runObservationManager) pollerReadStateMatchesLocked(entry *runObservationPoller, previous *runObservationRetainedState, initial bool) bool {
	if m.stopping || entry == nil || m.pollers[entry.key] != entry || entry.current != previous {
		return false
	}
	if initial {
		return entry.phase == runObservationPollerPhaseInitializing && entry.initResult == runObservationInitializerPending
	}
	return entry.phase == runObservationPollerPhaseRunning && previous != nil
}

func (m *runObservationManager) authorizePollerWatchers(
	entry *runObservationPoller,
	previous *runObservationRetainedState,
	initial bool,
) (map[*runObservationLease]struct{}, bool) {
	for {
		if entry == nil || entry.ctx == nil || entry.ctx.Err() != nil {
			return nil, false
		}
		m.mu.Lock()
		if !m.pollerReadStateMatchesLocked(entry, previous, initial) {
			m.mu.Unlock()
			return nil, false
		}
		watchers := make([]*runObservationLease, 0, len(entry.refs))
		for lease := range entry.refs {
			if lease != nil && !lease.draining && lease.pollers[entry.key] == entry {
				watchers = append(watchers, lease)
			}
		}
		m.mu.Unlock()
		if len(watchers) == 0 {
			return nil, false
		}
		validated := make(map[*runObservationLease]struct{}, len(watchers))
		for _, lease := range watchers {
			if entry.ctx.Err() != nil {
				return nil, false
			}
			if m.authorizeLeaseForPoller(entry, lease) {
				validated[lease] = struct{}{}
			} else {
				m.startLeaseDrain(lease, false)
			}
		}
		if entry.ctx.Err() != nil {
			return nil, false
		}
		m.mu.Lock()
		if !m.pollerReadStateMatchesLocked(entry, previous, initial) {
			m.mu.Unlock()
			return nil, false
		}
		unvalidated := false
		authorized := make(map[*runObservationLease]struct{}, len(validated))
		for lease := range entry.refs {
			if lease == nil || lease.draining || lease.pollers[entry.key] != entry {
				continue
			}
			if _, checked := validated[lease]; !checked {
				unvalidated = true
				continue
			}
			authorized[lease] = struct{}{}
		}
		m.mu.Unlock()
		if unvalidated {
			continue
		}
		if len(authorized) == 0 {
			return nil, false
		}
		return authorized, true
	}
}

func (m *runObservationManager) authorizeLeaseForPoller(entry *runObservationPoller, lease *runObservationLease) bool {
	if m == nil || entry == nil || lease == nil || entry.ctx == nil || entry.ctx.Err() != nil || m.authorize == nil || m.guard == nil {
		return false
	}
	if err := m.authorize(entry.ctx, lease.caller, entry.key.caseID, m.sessions, m.verifier); err != nil || entry.ctx.Err() != nil {
		return false
	}
	present, err := m.guard(entry.key.caseID)
	if err != nil || entry.ctx.Err() != nil || present.caseID != entry.key.caseID || present.cursor != lease.asOfCursor {
		return false
	}
	if _, exists := present.parentIDs[entry.key.planItemID]; !exists {
		return false
	}
	return true
}

func (m *runObservationManager) stopPollerBeforeRead(entry *runObservationPoller, initial bool) {
	closeReady := false
	m.mu.Lock()
	if m.pollers[entry.key] == entry {
		if initial && entry.initResult == runObservationInitializerPending && entry.current == nil {
			entry.initResult = runObservationInitializerInvalid
			if m.stopping {
				entry.phase = runObservationPollerPhaseStopping
			} else {
				entry.phase = runObservationPollerPhaseDraining
			}
			closeReady = true
		} else if entry.phase != runObservationPollerPhaseStopping && entry.phase != runObservationPollerPhaseDraining {
			if m.stopping {
				entry.phase = runObservationPollerPhaseStopping
			} else {
				entry.phase = runObservationPollerPhaseDraining
			}
		}
	}
	m.mu.Unlock()
	if closeReady {
		close(entry.ready)
	}
}

func (m *runObservationManager) finishPollerRead(
	entry *runObservationPoller,
	previous *runObservationRetainedState,
	initial bool,
	snapshot ports.RunTraceSnapshot,
	readErr error,
) bool {
	acceptedAt := m.now()
	phase := runObservationPollerPhaseRunning
	initResult := runObservationInitializerAccepted
	var state *runObservationRetainedState
	continuePolling := false
	if readErr != nil {
		if initial {
			initResult = runObservationInitializerReadUnavailable
			continuePolling = false
		} else {
			state = retainedMarkerCopy(previous, retainedReadUnavailable)
			initResult = runObservationInitializerAccepted
			continuePolling = true
		}
	} else {
		candidate := buildRunObservationRetainedCandidate(previous, entry.key, snapshot, acceptedAt)
		switch candidate.Outcome {
		case retainedCandidateAccepted:
			state = candidate.State
			continuePolling = !runObservationSnapshotIsTerminal(snapshot)
		case retainedCandidateRejected, retainedCandidateInvalid:
			if initial {
				if candidate.Outcome == retainedCandidateInvalid {
					initResult = runObservationInitializerInvalid
				} else {
					initResult = runObservationInitializerRetentionUnavailable
				}
			} else if candidate.State != nil {
				state = candidate.State
				if state.Availability == retainedTerminalRetentionFailure || state.Availability == retainedStopScheduling {
					phase = runObservationPollerPhaseStopping
					initResult = runObservationInitializerTerminalRetentionFailure
				} else {
					initResult = runObservationInitializerAccepted
					continuePolling = true
				}
			} else {
				state = retainedMarkerCopy(previous, retainedReadUnavailable)
				initResult = runObservationInitializerAccepted
				continuePolling = true
			}
		case retainedCandidateOrdinalOverflow, retainedCandidateGenerationOverflow:
			if initial {
				initResult = runObservationInitializerRetentionUnavailable
			} else if candidate.State != nil {
				state = candidate.State
				if state.Availability == retainedTerminalRetentionFailure || state.Availability == retainedStopScheduling {
					phase = runObservationPollerPhaseStopping
					initResult = runObservationInitializerTerminalRetentionFailure
				} else {
					initResult = runObservationInitializerAccepted
					continuePolling = true
				}
			}
		}
	}
	if !initial && state == nil {
		state = previous
	}
	var terminalLeases []*runObservationLease
	var cancelTerminal bool
	if image, err := marshalRunObservationPollerImage(entry.key, phase, initResult, state); err != nil || len(image) > runObservationRetainedImageMaxBytes {
		if initial {
			state = nil
			phase = runObservationPollerPhaseRunning
			initResult = runObservationInitializerRetentionUnavailable
			continuePolling = false
		} else {
			terminal := readErr == nil && runObservationSnapshotIsTerminal(snapshot)
			availability := retainedRetentionUnavailable
			if terminal {
				availability = retainedTerminalRetentionFailure
				phase = runObservationPollerPhaseStopping
				initResult = runObservationInitializerTerminalRetentionFailure
				continuePolling = false
			} else {
				phase = runObservationPollerPhaseRunning
				initResult = runObservationInitializerAccepted
				continuePolling = true
			}
			state = retainedMarkerCopy(previous, availability)
		}
		if _, markerErr := marshalRunObservationPollerImage(entry.key, phase, initResult, state); markerErr != nil {
			m.stopPollerBeforeRead(entry, initial)
			return false
		}
	}
	terminalStop := phase == runObservationPollerPhaseStopping && state != nil &&
		(state.Availability == retainedTerminalRetentionFailure || state.Availability == retainedStopScheduling)

	closeReady := false
	var notifierTargets []runObservationNotifierTarget
	for {
		watchers, authorized := m.authorizePollerWatchers(entry, previous, initial)
		if !authorized || entry.ctx.Err() != nil {
			m.stopPollerBeforeRead(entry, initial)
			return false
		}
		m.mu.Lock()
		if !m.pollerReadStateMatchesLocked(entry, previous, initial) {
			m.mu.Unlock()
			m.stopPollerBeforeRead(entry, initial)
			return false
		}
		newWatcher := false
		validWatcher := false
		for lease := range entry.refs {
			if lease == nil || lease.draining || lease.pollers[entry.key] != entry {
				continue
			}
			if _, checked := watchers[lease]; !checked {
				newWatcher = true
			} else {
				validWatcher = true
			}
		}
		if newWatcher {
			m.mu.Unlock()
			continue
		}
		if !validWatcher {
			m.mu.Unlock()
			m.stopPollerBeforeRead(entry, initial)
			return false
		}
		entry.current = state
		entry.phase = phase
		if state != nil {
			notifierTargets = m.acquirePollerNotifierTargetsLocked(entry)
		}
		if terminalStop {
			cancelTerminal = true
			for lease := range entry.refs {
				if !lease.draining && lease.pollers[entry.key] == entry {
					terminalLeases = append(terminalLeases, lease)
				}
			}
		}
		if initial {
			entry.initResult = initResult
			closeReady = true
		} else if phase == runObservationPollerPhaseStopping {
			entry.initResult = runObservationInitializerTerminalRetentionFailure
		}
		m.mu.Unlock()
		break
	}
	if closeReady {
		close(entry.ready)
	}
	m.sendPollerNotifierTargets(notifierTargets)
	if cancelTerminal && entry.cancel != nil {
		entry.cancel()
	}
	for _, lease := range terminalLeases {
		m.startLeaseDrain(lease, true)
	}
	return continuePolling && phase == runObservationPollerPhaseRunning && state != nil
}
