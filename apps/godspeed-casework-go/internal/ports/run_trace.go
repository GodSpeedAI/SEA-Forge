package ports

import "context"

// RunTracePort reads the safe, application-owned projection of one governed run trace.
type RunTracePort interface {
	ReadRunTrace(ctx context.Context, caseID, runID, planItemID string) (RunTraceSnapshot, error)
}

// RunTraceSnapshot contains only the identities, standings, and safe frames returned for a run.
// TotalFrameCount counts allowlisted frames in the returned run.get array before port retention;
// it does not describe source-journal completeness.
type RunTraceSnapshot struct {
	RunID           string
	CaseID          string
	PlanItemID      string
	Execution       string
	Settlement      string
	Frames          []RunTraceFrame
	TotalFrameCount int
}

// RunTraceFrame contains only fields approved for safe trace disclosure.
type RunTraceFrame struct {
	EventID         string
	Kind            string
	Timestamp       string
	ExecutionStatus *string
	ExitCode        *int64
}
