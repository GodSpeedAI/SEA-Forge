# Canonical Go gate on the current private lifecycle source

Date: 2026-10-08
Executor: independent Luna renderer_resume_critic, sole compiler owner only
after root release. Root accepted focused retry02 7d321d11 after comparing all
six originals/archives, reading command/preflight/exits and actual outcomes.
53 top-level and22 nested outcomes pass; this is focused evidence only.

Read full algorithm grant/source review+errata, fixture grant75cd9bfd/result/
review, focused original5cc99e87/retry02grant1ef5657b/result7d321d11 and the
actual retry02 preflight. Run exactly ONE broader gate from repository root:

```
env GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS='-p=1 -count=1' GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 JUST_TEMPDIR=/tmp just casework-go-check
```

Fresh actual eleven-source/HEAD/memory/swap/bounded process preflight first,
requiring all current source identities exactly equal retry02 preflight,
MemAvailable>=1200MiB, SwapFree>=512MiB, one compiler owner/jobs1.
Root/other agents do no heavy work. Process visibility is namespace-local.
No source/test/formatter edits, scanner, Git, Graft build or second gate.
Canonical recipe's own checks are authorized; no bypass or golden update.

Immediately after actual JOIN archive NEW exact command/preflight/preflightexit/
stdout/stderr/testexit captures via native apply_patch with original_tmp_path,
kind and raw; decode/compare all six originals BEFORE any next command/gate.
Keep failed runs and originals immutable; derive all identity/outcome tables
from actual data. Native patch sole persistent authoring writer; program-
produced /tmp captures permitted. One concise NEW canonical result receipt.

If sandbox loopback/listener permission denies an otherwise legitimate gate,
preserve and archive that actual failure, return token/report root; a separate
normal permission retry may be authorized after review, never a hook/gate bypass.
Compiler/setup/fmt/test errors are actual failures, not accepted GREEN.
Return token after JOIN/archives/receipt; root independently verifies before
full-module race or any next gate. Do not claim lifecycle/T09 settlement.
