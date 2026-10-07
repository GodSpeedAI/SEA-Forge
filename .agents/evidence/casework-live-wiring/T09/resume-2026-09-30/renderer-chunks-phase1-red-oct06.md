# Renderer chunk fixture assertion RED — independent evidence

Date: 2026-10-06. Verdict: **APPROVE the bounded fixture assertion RED only.**
This records the expected test-first failure of the explicit helper stub. It
does not release the production Vite hook or approve runtime behavior.

## Preflight and source identities

The focused command ran from `apps/godspeed-cognitive-ui` after fresh
preflight. Captured available RAM was 2,788,937,728 bytes (minimum
1,258,291,200); free swap was 4,242,993,152 bytes (minimum 536,870,912); and
`/tmp` had 1,364,447,232 bytes available. Bun reported `1.4.0`.

| Source | SHA-256 before launch |
|---|---|
| `src/build/rendererChunkContract.ts` | `c5d9e16187ede70fd3a77e5ec078b99ef0e872b99e7a644aa8be55e204e0fd77` |
| `src/build/rendererChunkContract.test.ts` | `25b74d25318ce97659c91f3d734ca9b2cd794d0b4d90b1ec54c045b7ce022d08` |

## Focused result

Executed exactly once:

```text
cwd: apps/godspeed-cognitive-ui
command: bun test src/build/rendererChunkContract.test.ts
joined process exit: 1
```

Bun ran all 13 cases and 32 expectations: `0 pass, 13 fail`, exit 1. Every
failure reached an intended test assertion. The three positive cases failed
their `not.toThrow()` assertions on the explicit stub message
`renderer chunk output assertion is not implemented`. The ten negative cases
failed their targeted `toThrow(expected)` assertions because that same stub
message did not contain the expected module/registry/`distinct` diagnostic.
There were no import, parse, type-transpile, fixture setup, or worker failures;
the output lists every case, including both new `not-src` boundary cases.
This is the expected fixture RED, not evidence of a production checker.

## Immutable capture files

The following unique `/tmp` captures were archived unchanged. `cmp -s`
returned exit 0 for each archive/original pair:

| Capture | Original in `/tmp/renderer-chunks-phase1-red-oct06.nbKSEs/` | SHA-256 |
|---|---|---|
| Preflight | `preflight.raw` | `31bc31573a5d7e75f2234263281d4774bb7db10d6ed414126858102dc82400c6` |
| Bun output | `raw.out` | `66a4c6689c696d6b27373bdc0fb509fa8950f3587088608557e711cf8251819b` |
| Joined exit | `exit.txt` | `4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865` |

Archived copies:

- `renderer-chunks-phase1-red-oct06-preflight.raw`
- `renderer-chunks-phase1-red-oct06-run.raw`
- `renderer-chunks-phase1-red-oct06-exit.raw`

## Deviations and limits

No material deviation from the bounded assertion-RED assignment was observed.
The helper remains the throw-only test-first stub, and the fixture/test source
hashes match the accepted independent source rereview. No second test, type
check, build, browser test, full UI gate, source/test edit, or runtime change
was made. The result validates only that all 13 fixtures reach the intended
assertion failure under the stub; root retains the separate runtime release
decision.
