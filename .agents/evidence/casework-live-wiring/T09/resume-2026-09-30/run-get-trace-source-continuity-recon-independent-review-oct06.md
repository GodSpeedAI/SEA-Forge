# `run.get` trace source continuity recon — independent source review

Date: 2026-10-06  
Reviewed recon SHA-256: `7df13fb4e9b95103ad4e17a27508893dd0a7cd19de301d2ea21ef78fa87a61e6`.  
Verdict: **APPROVE as bounded factual source recon only.** No manager policy or
runtime implementation is approved.

## Reviewed source and search boundary

I read the complete recon and checked the actual Rust `run.get` reader, trace
types and recorder, both production recorder callsites, the error-path append
helper, resume and approval parking paths, the Go safe reader/retention, and
the scoped literal callsite inventory for `trace.jsonl`, recorder creation,
and `append_internal_error`. This check confirms the recon's bounded-search
qualification; it is not a proof against external filesystem mutation or
unsearched generic writes.

## Reader and Go projection claims

Rust `read_jsonl` first rejects a journal outside `MAX_JOURNAL_BYTES`, returns
empty on size/read failure, and uses `map_while` so decoding stops at the first
malformed nonblank line while retaining the valid prefix
(`crates/sea-forge-server/src/sfwp/run_views.rs:417-436`). `get` reads
`trace.jsonl`, errors when all canonical run records are absent, folds the
current rows, and builds `RunRecord.trace` by iterating the same event vector
in order (`run_views.rs:835-857,947-959`). It does not sort by event ID or
timestamp or validate a persistent sequence watermark.

`TraceEvent.event_id` is a string field (`crates/sea-forge-core/src/types.rs:528-540`).
The Go safe trace adapter validates exact response identities and allowed
values, rejects duplicate event IDs in one response, retains up to 1,024
allowlisted rows, and reports a source-response total before that retention
(`apps/godspeed-casework-go/internal/adapters/sfwp/run_trace.go:67-177`).
Those checks do not add a Rust producer continuity contract across reads. The
recon explicitly distinguishes per-response validation from proof of unseen
loss, which is correct.

## Writers, IDs, and recovery claims

`JsonlTraceRecorder::create` uses `create_new` and initializes `sequence: 0`;
each successful `append` increments before encoding/write, produces
`seq_id("tev",4,sequence)`, writes in order, flushes, and syncs
(`crates/sea-forge-trace/src/lib.rs:78-121`). Thus one uninterrupted successful
recorder lifetime has ordered sequential IDs; it does not establish global
uniqueness or resume continuity. `run_id` is separately minted and `seq_id`
is a formatted local sequence (`crates/sea-forge-core/src/ids.rs:29-50`).

The production source search finds recorder construction in server
`case_dispatch.rs:747` and CLI `pipeline.rs:280`, plus the sole production
caller of `append_internal_error` in that pipeline (`pipeline.rs:783-791`).
The helper validates that existing rows decode and the file ends on a newline,
counts nonblank parseable rows, then appends `tev_(row_count+1)`; it does not
validate existing IDs, run IDs, ordering, or use the maximum prior sequence
(`sea-forge-trace/src/lib.rs:22-77`). The recon accurately warns that row count
is not proof of sequence recovery. Its cited unit fixtures show the intended
normal-next-ID and malformed/torn-tail behavior; they are not production crash
recovery proof.

`JsonlTraceRecorder::append` advances its in-memory sequence before the fallible
serialization/write/flush/sync operations. A failure can leave an advanced
in-memory counter or a partial tail; the code path does not restore that
counter. The error helper is a separate file scan/append path and refuses
malformed or unterminated input. The recon appropriately states possibility,
not that every error necessarily persists a gap.

The CLI approval parking excerpt writes `plan.json` and `authority.json`, but
not a trace file (`crates/sea-forge-cli/src/plan_pipeline.rs:450-464`). Resume
can reuse an approved run ID for the first instance, but the scoped production
search found no resume-specific trace recorder; its recorder construction is
the shared pipeline site (`commands/resume.rs:318-330`, `pipeline.rs:280`).
No source was found that reopens a recorder and restores sequence state.

The recon's explicit limitation on no rewrite/delete/truncate findings is
important and accurate: searches were bounded by trace filenames and recorder
sites. The `case-events.jsonl` IDs (`case-runner/src/lib.rs:31-101`,
`case_ops/mod.rs:90-129`) and global event ledger use separate ID domains and
cannot prove `tev` continuity.

## Conclusion and limitations

The bounded conclusion is supported: an uninterrupted successful recorder
emits sequential IDs in append order; actual readers tolerate valid prefixes
and can collapse unreadable/oversized input to no rows; the CLI helper derives
its suffix from row count rather than validated prior sequence; and there is no
source-provided generation/high-water watermark. A consumer must not use these
facts to infer a precise missed-frame count or unconditional exactly-once
continuity. The document makes no such inference and explicitly leaves manager
dedupe policy unapproved.

No material factual mismatch was found in the reviewed trace-source claims.
This verdict does not approve a UI bootstrap, V4 cursor architecture, manager
delta policy, public contract, or any code change. No tests/compiler/scanner,
Graft build, Git, or network action was run.
