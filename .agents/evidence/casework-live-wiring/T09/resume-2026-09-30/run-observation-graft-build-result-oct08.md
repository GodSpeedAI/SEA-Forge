# Graft build result

Date: 2026-10-08

Full authorization: `run-observation-graft-build-oct08-assignment.md`.

## Preflight

One `graft build` ran from the repository root. The preflight exit was 0.
UTC was `2026-10-08T14:33:55+0000`; HEAD was
`7c65be70ecf14c77df1a7749e9fa5526e28eabc1`. `MemAvailable` was 2,635,660 KiB
and `SwapFree` was 3,738,800 KiB, both above the required thresholds. The
namespace-visible process query showed only its header and no matching Go,
formatter, Graft, Cargo, Rust compiler, Just, or Make process. Process
visibility is namespace-local.

All eleven source hashes matched the canonical retry03/full-module race
preflight exactly. The lossless preflight capture is
`run-observation-graft-build-preflight-oct08.raw.b64`; it decodes to the
original `/tmp/sea-observation-graft-build-oct08-EL8icd/preflight.raw`.
The exit capture is `run-observation-graft-build-preflight-exit-oct08.raw.b64`.

## Build result and captures

The exact command was `graft build` from `/home/sprime01/projects/sea-rs`.
It exited 0. Graft reported 7,882 nodes, 15,581 edges, 708 cards, and
`parsed: 0 of 708 files (708 replayed from cache)`. This records successful
index refresh; it does not claim Next implementation or T09 completion.

Original captures were retained under
`/tmp/sea-observation-graft-build-oct08-EL8icd/`:

| Kind | Original path | Bytes | SHA-256 | Archive |
|---|---|---:|---|---|
| Command | `command.raw` | 12 | `9073bc71a35aacd6e788d73a49a72e6d4a24623d7ee6312304009369fcc96a99` | `run-observation-graft-build-command-oct08.raw.b64` |
| Preflight | `preflight.raw` | 1742 | `820675c70745e11ef20c6e9c30af72e56cf756ec897b2d204830a5c87aa04a64` | `run-observation-graft-build-preflight-oct08.raw.b64` |
| Preflight exit | `preflight.exit.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `run-observation-graft-build-preflight-exit-oct08.raw.b64` |
| Stdout | `stdout.raw` | 421 | `32f076bc82d305523afce7c69ccd7ab60d67091e38663e92888e06b9f18fc6dc` | `run-observation-graft-build-stdout-oct08.raw.b64` |
| Stderr | `stderr.raw` | 48037 | `41c2444f1f310b84d7df064398e1b7260c87306141ddb4369917af7e896153cf` | `run-observation-graft-build-stderr-oct08.raw.xz.b64` |
| Exit | `exit.raw` | 2 | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` | `run-observation-graft-build-exit-oct08.raw.b64` |

The stderr archive is Base64 of an XZ-compressed lossless original; all other
capture archives are Base64 of the raw originals. No source/test/compiler/Git
work or Next tests were run. The generated index changed only through
`graft build`.
