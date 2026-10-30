# sea-forge-trace — src

## Overview

Implementation of the `sea-forge-trace` crate: SEA Forge trace event
recording. Provides the JSONL trace recorder for run journals, internal-error
appending on the error path, and the external event-sink contract that maps
kernel `TraceEvent`s onto the `sea.{domain}.{action}.{qualifier}` subject
taxonomy consumed by the NATS bridge.

## Interface

- Run journal recording (`lib.rs`):
  - `JsonlTraceRecorder` — buffered JSONL writer for a run's trace journal,
    carrying `run_id`, `actor_id`, and a monotonic `sequence`
    (`lib.rs:JsonlTraceRecorder`).
  - `append_internal_error(path, run_id, actor_id, error_class)` — append an
    internal-error trace to an existing journal; silently does nothing when
    the journal cannot be opened (`lib.rs:append_internal_error`).
- External bridge contract (`lib.rs`):
  - `SinkEvent` — payload shaped for the external NATS bridge, including the
    `sea.{domain}.{action}.{qualifier}` subject and an idempotency key
    (`lib.rs:SinkEvent`).
  - `EventSink` trait — sink accepting `SinkEvent`s; implementations are
    in-process JSONL only today; NATS and other buses are later plugins
    behind this trait (§2.3 E6) (`lib.rs:EventSink`).
  - `trace_to_sink(event, cell_id) -> SinkEvent` — map a kernel `TraceEvent`
    into the external bridge contract (`lib.rs:trace_to_sink`).
  - `subject_for(kind: TraceKind) -> String` — map a `TraceKind` onto the
    three-segment dotted subject taxonomy (`lib.rs:subject_for`).
  - `JsonlEventSink` — JSONL implementation of `EventSink`; appends to
    `<root>/.sea-forge/events.jsonl` (`lib.rs:JsonlEventSink`, `JsonlEventSink::create`).

## Invariants

- Error-path memory stays bounded: `append_internal_error` streams the journal
  line by line instead of slurping it, because a whole-file allocation on the
  error path is an OOM vector (`lib.rs:append_internal_error`).
- Trace records carry `RECORD_VERSION` and monotonic per-recorder sequence
  numbers (`lib.rs` imports `RECORD_VERSION`, `ids::seq_id`;
  `JsonlTraceRecorder.sequence`).
- Sink subjects are exactly three dotted segments `sea.{domain}.{action}.{qualifier}`
  per the ecosystem map (`lib.rs:subject_for`).

## Side Effects

- Disk writes: appends JSONL trace events to the run journal
  (`JsonlTraceRecorder`) and bridge-shaped events to
  `<root>/.sea-forge/events.jsonl` (`JsonlEventSink::create`, which creates
  parent directories and opens the file in append mode).
- Disk reads: `append_internal_error` reads an existing journal to append
  after the last record (`lib.rs:append_internal_error`).

## Verification

- Run: `cargo test -p sea-forge-trace` (repo-wide: `just test`)
