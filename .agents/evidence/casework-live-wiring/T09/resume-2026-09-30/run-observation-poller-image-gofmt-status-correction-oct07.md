# Gofmt captured status correction — 2026-10-07

This immutable correction supersedes the status attribution in `run-observation-poller-image-gofmt-result-clarification-oct07.md`; it does not alter any captured evidence.

The actual preflight's `COMMAND:` line records exactly:

```text
cd apps/godspeed-casework-go && gofmt -d internal/server/run_observation_poller_image.go internal/server/run_observation_manager_retained_image_test.go
```

There is no output-presence check or other status wrapper in that recorded command. Its separate actual exit capture is `1\n`; thus the recorded shell invocation status is the status of the `gofmt -d` command after the successful directory change. The output is nonempty and contains only the five field-alignment changes in the new fixture. This correction claims only the observed command status and diff; it does not infer why this platform's `gofmt` returned 1.
