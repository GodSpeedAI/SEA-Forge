# Independent native current snapshot rejection and root decision

The strengthened native race rejected (exit1,15.270s): streamed K equals the complete
authenticated retained Store.At(K) snapshot, but differs from the fresh current snapshot.
Diagnostics `/tmp/sea-t08-native-diagnostic-2887574431` contain a generic assertion without
field-level response differences; the actual differing field is not established.

Independent Luna critic and fresh read-only investigator traced the separate read paths:

- `internal/server/server.go:172`: historical cursor uses Store.At and renderRevision;
  no-cursor current uses the case relay cursor and LiveSource.Snapshot.
- `internal/projection/live.go:99`: Snapshot calls Facts, which supplies time.Now per read.
- `internal/projection/builder.go:99`: snapshot.timestamp is Facts.Now formatted RFC3339.
- `internal/server/server.go:397`: retained Facts are cloned and rendered with captured Now.

The governing casework spec does not require fresh render timestamp equality with captured
revision time. Timestamp is a plausible source-supported cause, not a proven observed diff.
Current reads also fetch current authority facts; do not claim universal same-cursor byte
equality across arbitrary authority changes.

Root authorizes only this native test repair: preserve exact full SSE-vs-authenticated
historical comparison, including timestamp and every real object/optional extension. In this
controlled quiescent cell, require the fresh current cursor to equal the expected K/L, compare
every stable payload field, and separately validate its returned timestamp. Exclude only the
fresh render timestamp from that current comparison. Do not silently accept a changed cursor
or normalize any other field. All actual mutation, native transport, callback, durable trace,
retention and disposal assertions remain required. This changes no production contract/code.

Fresh builder shared_replay_builder implements the bounded repair; independent critic must
rerun typecheck, strengthened native race and canonical UI before any T08 approval.
