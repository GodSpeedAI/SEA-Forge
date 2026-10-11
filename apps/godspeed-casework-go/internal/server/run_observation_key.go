package server

// runObservationPollerKey is the exact source identity used to share a poller.
type runObservationPollerKey struct {
	caseID     string
	runID      string
	planItemID string
}
