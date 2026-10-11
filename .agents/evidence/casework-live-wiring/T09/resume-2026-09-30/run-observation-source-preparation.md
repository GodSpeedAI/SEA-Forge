# Run observation source preparation

Read-only Luna research scoped later units4/5; root verified unreadable-list ownership
against actual Rust run_views.rs777-832. This is preparation, not implementation evidence.

Current LiveSource.Facts calls actorless/unscoped RunsList and filters Go summaries by case.
Existing RunList wire accepts optional case_id. Kernel list scopes by case_index membership
before reading each run: scoped unreadable run IDs are case-claimed, not global unknown rows.
Return both runs and unreadable IDs through the adapter; do not silently drop unreadable
signals or call a failed/partial list a complete empty cohort. Kernel still enumerates global
run directories; case filtering is not pagination or an upstream enumeration bound.

RunSummary already has real run/case/item IDs, separate execution/settlement and optional
start/finish times. Build currently emits no execution_trace children. Validate exact
requested case/run and actual parent horizon item before attachment; no invented parent.
Current RunArtifactProvenanceView decodes only ownership/evidence/reduced trace. A full safe
run observation view must preserve actual event_id/kind/timestamp and only real optional
command_finished execution.status/exit_code, never raw trace actor/payload/command/env/output.

SSE handleEvents verifies session perspective at connection open and uses Store case revision
IDs. writeSSE with empty ID already omits id:. New side-channel observations must use that
path, with latest real cursor informational only; never Store.Append trace metadata or mutate
retained Facts. Capture historical run standing, but never overlay current frames on history.

Prepared bounds remain approved: one scoped list plus at most8 initial logical get reads,
active/pending-enabled/recent-terminal order, exact separate omission/failure counts, at most
16 shared pollers/two concurrent get reads/one-second per-run floor/1024 safe frames per run,
1MiB serialized initial metadata. Failed list has no counts/runs. Omitted candidates never
fan out within that cohort; terminal/no-subscriber pollers cancel AND drain before removal.
Neither client per-line cap nor call counts bound cumulative transport or kernel journal work.

Authorize every subscription and read for effective actor. Current run.list/get kernel reads
are unprotected factual reads; that alone does not prove future authority invariance. Root
must either verify shared factual scope or key by effective authority. Current SSE handler
has no logout cancellation signal: deleting a cookie does not close an open connection.
Later observation lease/delivery tests must establish chosen revocation lifecycle, unsubscribe
cleanup and no stale authority buffer fanout. Do not silently claim immediate revocation.

Nearby evidence patterns: artifact_provenance_test.go exact run/item owner refusals;
projection/live_golden_test.go frozen projection facts; server_test.go SSE replay/header/resync;
session_read_test.go and server_live_test.go actual session perspective/retained history.
