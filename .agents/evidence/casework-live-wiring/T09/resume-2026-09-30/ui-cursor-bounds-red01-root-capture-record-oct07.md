# Cursor bounds focused RED capture record

Root read the full134ba616 bounds fixture and independent source review15048f3a.
Exactly one focused command ran as sole heavy owner:
`GOLDEN_UPDATE=0 bun test src/adapters/local/localAdapter.cursorBounds.test.ts`
from apps/godspeed-cognitive-ui. Synchronous shell wrapper joined0; Bun exited1.
Actual preflight UTC2026-10-07 03:58:44, MemAvailable4051MiB/freeSwap9203MiB;
resource floors passed. Four source hashes in the preflight match frozen inputs.
Filename oct06 labels reflect assignment date, not a claim about runtime date.

Actual output:14 tests,2 pass,12 fail,38 expectations,204ms. The failures are
assertions about constructor validation, floors, overflow, cancellation and
exhaustion, plus an escaped listener exception. Their semantic interpretation
and assertions not reached require independent negative-evidence review;
this record does not approve production or prove the full matrix.

All original/archive comparisons exited0:

| Capture | SHA256 |
|---|---|
| ui-cursor-bounds-red01-preflight.raw | 508d5773da6e786adfa46927b19ce059b688184fbc2698806739386100603ec8 |
| ui-cursor-bounds-red01-bun-output.raw | 83bdca48676225d5f09881789ddf669aa8b152be675af4a7e21be3f6bc327388 |
| ui-cursor-bounds-red01-exit.raw | cf205dbb8cea84897b488abcc281bf96698d5e94b1096b16657b4caba9082a22 |

Originals remain `/tmp/sea-ui-cursor-bounds-red01-resume-oct06.{preflight,bun-output,exit}.raw`.
The183451-byte Bun output exceeded a direct read's limit. That truncated read
was detected before any output archive write. A bounded base64 chunk transfer
decoded actual bytes in orchestration memory, wrote through native patch and
passed cmp. No reconstruction or formatter was used. Compiler ownership
returned IDLE only after all three captures were archived and compared.
