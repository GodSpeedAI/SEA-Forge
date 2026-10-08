# Full-module race capture erratum

Date: 2026-10-08. This immutable erratum corrects the provenance status of
`next-projection-full-race01-critic-capture-oct08.json`; it does not modify
the six captured files.

## Capture issue

The command executed was `GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1
-json ./...` from `apps/godspeed-casework-go`. The wrapper used this status
logic:

```sh
if ! GOMAXPROCS=2 GOFLAGS=-p=1 go test -race -count=1 -json ./... > "$capture_dir/stdout.raw" 2> "$capture_dir/stderr.raw"; then rc=$?; else rc=0; fi
```

Inside the `then` branch, `$?` is the successful status of the negated
condition, not the child `go test` exit code. Thus the wrapper writes `0` to
`exit.raw` for either child outcome. That file is preserved exactly as
captured, but it is not evidence of the child exit status. The child status
is unresolved.

The captured stdout is valid as a byte-preserved output stream and parses as
2,968 Go JSON event lines: 738 pass events, 5 skip events, and no fail events.
Each of the 10 executable packages has a package pass event; five packages
were skipped. Captured stderr is empty. These observations indicate a likely
successful run but do not replace a trustworthy child exit capture.

## Prior provenance lesson

This repeats the general evidence limitation recorded as CW26 in
`run-observation-focused-red-capture-provenance-correction-oct07.md`: a
preserved record is not stronger than its capture procedure. The earlier
case involved manually transcribed output and status without original raw
files; this case preserves raw stdout/stderr but the status wrapper is
incorrect. The distinction is material, so this run cannot satisfy the
full-module race gate's exit-status proof. Do not infer or rewrite the status
as zero from the JSON stream.

The six original files are under
`/tmp/sea-casework-full-race01-dAAPMr/`. The immutable lossless archive is
`next-projection-full-race01-critic-capture-oct08.json`, SHA-256
`891037c4233bf0745ec5eb820fded3c403dba94f59499db65acfdb610d7948ad`.
Each archived entry was decoded and byte-compared against its original.
Root's independent comparison is pending. No rerun is authorized by this
erratum.

No implementation, source, test, or algorithm claim is changed here. A fresh
full-module race run with a wrapper that records `subprocess.run`'s actual
return code is required before the final assembler approval.
