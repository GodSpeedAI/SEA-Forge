# Failure fixture formatting result

Date: 2026-10-08

## Source and formatter result

Only `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` changed. Exact preimage: 47,657 bytes, SHA-256 `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4`. Final: 47,658 bytes, SHA-256 `8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8`.

Preimage wrapper: `run-observation-manager-failure-fixture-format-preimage-oct08.json`, SHA-256 `3e68ccee15e867db6c110e5227e1f13da1a65e5235ba40fbbf8f9ca58853d454`; it declares the path, SHA, byte count, and full source content. Decoded content compared exactly to the source before the formatter call.

The read-only command was `gofmt apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go`, run at `2026-10-08T13:08:34Z`. Its stdout was 47,658 bytes with SHA-256 `8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8`; stderr was empty; exit status was 0. The post-edit source was compared with the actual captured stdout using `cmp`: exit 0, exact bytes equal.

## Captures

The untouched original captures are under `/tmp/sea-observation-failure-format-oct08.ofckVS/`. Each was losslessly archived immediately after `gofmt` joined and decoded bytes were compared equal to its `/tmp` original before the next command.

| Capture | Original `/tmp` path | Bytes / SHA-256 | Immutable archive / SHA-256 |
|---|---|---|---|
| Preflight | `/tmp/sea-observation-failure-format-oct08.ofckVS/preflight.raw` | 198 / `91b146a6899161b458ce7972b70ae5055754d262867f7059979683f6e5688d38` | `run-observation-failure-format-preflight-oct08.raw.json` / `1ff92a65f783131d0a96c67310fa6a22f689141f4dc7f2fc13e1073f068bec87` |
| Exact command | `/tmp/sea-observation-failure-format-oct08.ofckVS/command.raw` | 88 / `0932731b66ccd3b65aa26fbab92c037e0679ff82ca785779924f3f1c89f6d3de` | `run-observation-failure-format-command-oct08.raw.json` / `a67ccba28a75be6cd27205b93bbbf102a22495b1e8b202f0bfdaece9a561f9a9` |
| Formatter stdout | `/tmp/sea-observation-failure-format-oct08.ofckVS/stdout.raw` | 47,658 / `8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8` | `run-observation-failure-format-stdout-oct08.raw.json` / `bf1482548aecdc9bee0a93c25a4c32613ac3c28fc87c6e19e8e2d163d0e26e3a` |
| Formatter stderr | `/tmp/sea-observation-failure-format-oct08.ofckVS/stderr.raw` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | `run-observation-failure-format-stderr-oct08.raw.json` / `b5be92b8231734054eecb883bae9455f7b893dd2b19e333c2dc87e612b3705cd` |
| Exit | `/tmp/sea-observation-failure-format-oct08.ofckVS/exit.raw` | 2 / `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `run-observation-failure-format-exit-oct08.raw.json` / `8b4d9fb71de9a5a511ea103ebf283b64f43d24dcca15046b5ad6138d0cf14c49` |

All five archive decodes compared equal to their original captures.

## Exact diff and frozen sources

The captured formatter diff was one alignment change in the constant block: `runObservationFailureRunID` receives one additional space before `=` to align with adjacent constants. No assertions, helpers, callbacks, test ordering, or runtime behavior changed. Formatting was applied through native `apply_patch` using that exact hunk; no `gofmt -w` was used.

All ten other observation source files listed by `run-observation-canonical-go-preflight-oct08.raw.json` were rehashed after the edit and matched their preflight expected SHA-256 values. In particular, manager, worker, the other two fixtures, and the six frozen primitive files remained unchanged.

No compiler, test, build, or Git command was run. This is a formatting-only source result for independent review, not a test or lifecycle approval.
