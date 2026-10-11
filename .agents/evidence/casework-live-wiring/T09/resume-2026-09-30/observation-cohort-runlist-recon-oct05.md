# Unit5C cohort run-list source recon

Date: 2026-10-05. Read-only source and contract investigation for root
architectural decisions. No coordinator is designed or implemented here.

## Existing scoped ownership path

The Go request builder is `NewRunListForCase` in
`apps/godspeed-casework-go/internal/adapters/sfwp/frame.go:298-305`. It rejects
blank/whitespace case IDs as typed `invalid`, then sends `run_list` with the
existing `case_id` field. It does not trim a nonblank ID before transmission.

Rust `run_views::list` in `crates/sea-forge-server/src/sfwp/run_views.rs:777-832`
builds an ownership index from each readable case file's `run_ids`, then
includes only run directories whose indexed owner equals the requested case
(`:484-503,785-791`). The ownership source is the case claim; the run record
does not name its own case. The result has readable `runs` and separate
`unreadable` IDs. A run is placed in `unreadable` only when its trace rows are
empty and it has no readable settlement (`:793-801`); that list does not mean
all source errors were enumerated.

`Authority.RunsListForCase` (`authority.go:256-315`) refuses the whole result
for missing/null arrays, malformed rows, blank/missing/foreign case identity,
blank run or plan-item identity, unknown standings, negative evidence count,
malformed times, duplicate run IDs, or IDs repeated/overlapping between the
readable and unreadable arrays (`:276-305,317-363`). It preserves non-`unavailable`
authority refusal errors; an `unavailable` refusal is wrapped as unavailable
(`:309-315`). It does not return partial rows or an explicit completeness flag.

The adapter validates that a row's `case_id` exactly equals the requested case
and that `plan_item_id` is nonblank. `LiveSource.Facts` adds the actual parent
check: every run plan item must occur in the current `CaseHorizon.Items`; a
blank, duplicate, or nonexistent parent makes the full Facts read unavailable
(`apps/godspeed-casework-go/internal/projection/live.go:76-123`). Facts also
checks returned case identity, repeated run IDs, malformed standings, and
readable/unreadable overlap. If Unit5C consumes `RunsListForCase` directly,
that horizon-parent validation is not automatically applied; the candidate
source must preserve an equivalent parent-existence check if the cohort
contract requires actual current parents.

## Duplicate and parent limitations

The Go adapter detects duplicate IDs only in the `run.list` response it
receives. Before that, Rust `run_dirs` collapses duplicate directory IDs into a
`BTreeMap`; a flat `<root>/runs` entry overwrites a case-local directory with
the same ID (`run_views.rs:452-481`). Separately, `case_index` inserts claims
without checking whether multiple cases claim the same run ID (`:491-503`).
Case entries are not sorted before this insertion. Conflicting claims can
therefore be overwritten before scoped filtering, with no ambiguity marker for
Go to detect. A returned row whose `case_id` mismatches the request is
rejected, but a silently selected duplicate claim that now matches cannot be
identified by the current response.

Rust folds `plan_item_id` from the first trace event that carries one
(`run_views.rs:520-535`). It does not verify that item against a case horizon.
The existing Go Facts check supplies that separate relation check. Root should
decide whether a cohort's ownership proof includes only exact run/case
`run.get` identity, as the approved T09 contract says, or also needs to retain
the existing current-horizon parent check for the initial run-list candidate.

## Statuses and ordering fields

`RunSummaryView` carries `run_id`, optional `case_id` / `plan_item_id`, string
execution and settlement standings, optional `started_at` / `finished_at`, and
evidence count (`apps/godspeed-casework-go/internal/adapters/sfwp/frame.go:669-683`).
The scoped adapter maps both timestamps to `time.Time` and records separate
`HasStarted` / `HasFinished` flags; nil is allowed, but a present malformed or
empty timestamp rejects the whole list (`authority.go:341-363`).

Rust execution standing comes from trace folding: pending by default, enabled
on `item_enabled`, active on run/item/command start, completed on item or human
task completion, failed on item failure/internal error, terminated on item
termination (`run_views.rs:520-550`). Settlement is independently derived from
the settlement record (`:812-815`); execution completion is not settlement.
The valid execution values are `pending`, `enabled`, `active`, `completed`,
`failed`, `terminated`; settlement is `unsettled`, `accepted`, `rejected`, or
`escalated` (Go validators at `authority.go:370-386`).

Kernel `run.list` order is newest `started_at` then descending `run_id`, with
unreadable IDs sorted lexically (`run_views.rs:822-830`). That order is not the
T09 cohort priority. The approved proposal requires active first, then
pending/enabled, then terminal (`completed`/`failed`/`terminated`) candidates,
with recency and run ID ordering (`.agents/reports/casework-live-wiring/t09-contract-extension-proposal.md:35`).
The proposed recency uses finished/started times, so the coordinator must
reorder rather than trust the kernel list order. The text does not fully define
the tie/fallback order when `finished_at` is absent, both times are absent, or
the candidate is active/pending; deterministic handling of those cases is a
root decision. `time.Time` from the Go adapter should be used for chronological
comparison across offsets, not raw timestamp strings.

## List completeness and errors

The Rust wire result contains arrays, not a source-completeness signal. The
`run_dirs` scan flattens read-directory and entry errors and returns whatever
it gathered (`run_views.rs:452-481`); `case_index` skips case files it cannot
read or decode (`:491-503`). `read_jsonl` returns empty on over-cap/read
failure and stops at the valid prefix after malformed JSONL
(`run_views.rs:425-436`). Thus a successful response means a response was
produced, not that every filesystem directory/case file/journal was fully
enumerated and parsed.

The `unreadable` list is narrower: it records known, case-claimed run
directories with no returned trace rows and no readable settlement. If a
settlement exists but trace/plan identity is incomplete, Rust can emit a row
with no plan item; Go's strict scoped summary then rejects the entire response
as unavailable. A successful scoped result has non-nil `runs` and `unreadable`
arrays; an empty pair may be interpreted as no returned candidates, but it
cannot prove source-level absence given the suppressed scan/read failures.

The current proposal says counts are present only after a successful complete
`run.list`, and list refusal/over-cap becomes unavailable with counts absent
(proposal lines 31, 41). Root should define “complete” at the response boundary:
the current Rust API cannot certify full filesystem enumeration. Count
semantics also need one explicit mapping: `RunListResult.Runs` contains
readable summaries, while `UnreadableIDs` contains IDs without summaries.
The proposal separates unreadable from unavailable `run.get` candidates and
defines omitted count over listed summaries (lines 31, 41); it should state
whether `listed_run_count` counts only readable summaries or includes those
separate unreadable IDs. Unreadable IDs lack status/time fields and should not
be silently treated as selectable summaries.

The contract and spec explicitly exclude upstream enumeration and aggregate
transport/journal bytes from the cohort bound (`t09-contract-extension-proposal.md:39-43`,
`godspeed.casework-cognitive-environment-spec.yaml:82-106`). There is no
pagination; `run.list` walks all run directories before returning the filtered
response (`t09-contract-extension-proposal.md:19,41`). The proposed 32 MiB
line cap is still not implemented according to the proposal, and the current
Go client reads a newline-delimited response before decode. Limiting selection
to eight therefore bounds downstream candidate `run.get` reads, not list
enumeration or the current list response allocation.

The coarse plan entry at `.agents/plans/2026-09-23-casework-live-wiring-production.plan.yaml:503-545`
describes T09 UI journey wiring and does not define the Unit5C cohort details;
the design specifics are in the approved proposal and normative run-trace
boundary in the spec.

## Retrieval and scope

Graft retrieval preceded direct source inspection. Graft reported approximately
83,020 tokens saved (~$0.07) across the two successful retrieval calls. No
compiler, tests, source changes, status changes, or Git mutations were made.
