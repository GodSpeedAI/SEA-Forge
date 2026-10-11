# Independent retained pure-helper algorithm review

Date: 2026-10-07  
Disposition: **APPROVE bounded pure-helper algorithm only**

## Scope and source identities

Reviewed the full original helper assignment, root's private algorithm decisions,
archived implementation preregistration, the complete algorithm and both frozen
test files, and the pinned manager source/test baseline. No source or test file
was edited. The prior transient agent report `508da` was unavailable and was not
reconstructed.

| Artifact | SHA-256 |
|---|---|
| `internal/server/run_observation_retained_version.go` | `38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a` |
| `internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `internal/server/run_observation_retained_policy_test.go` | `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7` |
| `internal/server/run_observation_manager.go` | `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d` |
| `internal/server/run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |

The three runtime capture archives are the original preflight, stdout/stderr,
and exit file. Each was immediately archived by native patch and `cmp`ed against
its `/tmp` original before any further gate; all three comparisons returned
zero. Their SHA-256 values are recorded in the raw capture receipts referenced
below.

## Contract and implementation assessment

The contract is a private, pure retained-version transform: immutable
replacement candidates, exact lifetime event-ID ordinals, ordered current
frames, a deterministic canonical image bounded at 1 MiB, a 1,024-frame
window, and explicit recovery/terminal marker behavior. It must not claim
manager lifecycle, worker join, `Next`, or public integration behavior.

The implementation conforms to the ratified algorithm:

* It checks snapshot identity, prior-key identity, frame-count consistency,
  window bound, and within-window duplicate IDs before candidate construction
  (`run_observation_retained_version.go:76-92`). It relies on the explicitly
  ratified `RunTracePort` safe-snapshot precondition for the rest of source
  validation, as stated at lines 67-69 and root decision 3.
* It refuses an existing terminal-retention/stop marker, prevents generation
  wrap, starts at generation 1, clones the lifetime ledger, and adds exact new
  IDs in source order while preserving prior ordinals (`:94-143`). Overflow
  preserves the prior safe state and sets the exact root-required marker
  (`:101-104`, `:124-134`).
* It deep-copies optional frame pointers, records the accepted timestamp only
  on a candidate, builds a new state, then checks its complete canonical image
  before returning it (`:140-167`, `:179-219`). Budget refusal discards the
  candidate; a first refusal returns nil, while a prior refusal returns only a
  deep prior copy with the appropriate marker.
* Terminality is based only on execution standing `completed`, `failed`, or
  `terminated` (`:170-177`). The canonical image uses explicit ordered structs,
  source-order frames, sorted exact ledger IDs, 20-digit uint64 values, and an
  always-present one-digit availability code (`:222-310`). The bound is exactly
  `1<<20` (`:14-17`).

The additional policy fixtures close the material gaps called out in the prior
design review: exact ordinal/generation overflow markers and prior atomicity
(`run_observation_retained_policy_test.go:67-129`), execution-only terminality
(`:131-170`), refusal of later candidates after final markers (`:172-197`),
distinct source totals vs current window/ledger counts (`:199-235`), and recovery
plus identity/count rejection (`:237-282`). The original frozen fixtures retain
the ordinal/reorder/window, duplicate, atomic refusal/recovery, terminal safe
copy, exact image-boundary, and max-counter coverage
(`run_observation_retained_version_test.go:91-540`).

No material algorithm mismatch or formatting defect was found in the bounded
helper. The one-digit availability enum is the private lifecycle control; the
image's `truncated` field is derived accounting data and remains a JSON boolean,
consistent with correction revision 3 distinguishing bounded internal
lifecycle controls from variable-length candidate/source representation
(`run-observation-manager-revision6-retention-initializer-correction-revision3-oct06.md:53-75`).

## Executed evidence

Focused command, from `apps/godspeed-casework-go`:

```text
GOMEMLIMIT=256MiB GOGC=50 GOMAXPROCS=2 GOFLAGS=-p=1 GOLDEN_UPDATE=0 GOCACHE=/tmp/sea-casework-go-build-cache-e8859aa2-setupfix1 CARGO_BUILD_JOBS=1 go test -race -count=1 -parallel=1 -timeout=60s ./internal/server -run '^TestRunObservationRetained' -v
```

Exit status 0. All 13 top-level retained tests and 13 nested cases passed under
the race detector; package reported `ok .../internal/server 1.657s`. Raw
preflight/output/exit archives:

* `run-observation-retained-helper-review-preflight-oct07.raw`
  SHA-256 `30c18aea254489616d468e7e6e2185be27375179416da37195678ff9b8f116b7`
* `run-observation-retained-helper-review-output-oct07.raw`
  SHA-256 `225e213a7e224285a5bbf435b29232ffd673526e90555e1ad91dff9ac8762a47`
* `run-observation-retained-helper-review-exit-oct07.raw`
  SHA-256 `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa`

The preflight records 4,569,024 kB available memory, 12,582,904 kB free swap,
UTC time, all five pinned source hashes, and the exact command. It therefore
meets the required 1,200 MiB RAM / 512 MiB swap gate.

## Approval boundary

This evidence supports the bounded pure helper algorithm and its focused
fixtures only. It does not verify manager integration, actual worker lifecycle
or join, read scheduling, capacity release, `Next`, SSE/HTTP, public contract,
or T09 settlement. The manager and manager test remain at their pinned stub
identities listed above. No full Go module or broader repository gate was run.

No compiler ownership remains with this critic; the sole Go compiler token is
returned to root.
