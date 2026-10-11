# T00 evidence — build-resource preflight readings

Rule applied (plan `operating_contract.build_resource_policy.preflight`): record
`MemAvailable`, the cgroup memory limit/usage when finite, `/proc/pressure/memory`, swap
usage, and other active builds **before every compile/test/broad gate**; effective headroom
is `min(MemAvailable, cgroup limit − usage)`. No finite cgroup limit exists on this host
(`/sys/fs/cgroup/memory.max` is absent), so effective headroom **is** `MemAvailable` — a
missing limit is not zero headroom.

Machine: `nproc` = 6, `MemTotal` = 8132716 kB (7.76 GiB), `SwapTotal` = 12582912 kB (12 GiB).

| Moment | MemAvailable | SwapFree | pressure/memory `some avg300` | Active heavy build | Gate attempted |
|---|---|---|---|---|---|
| 17:57:30Z (prereg freeze) | 3735720 kB | 7832572 kB | 0.84 | none (only the operator's `bun scripts/serve.ts` dev server — a process command line, not a repository path) | — |
| 17:59:48Z pre-`lint` | 3589484 kB | 7833572 kB | — | none | `just lint` |
| pre-`typecheck` | 3540912 kB | 7820248 kB | — | none | `just typecheck` |
| pre-`test` | 3395144 kB | 7821000 kB | — | none | `just test` |
| pre-`proof` | 3601372 kB | 7688776 kB | — | none | `just proof` (exit 127 — harness defect) |
| pre-`security` | 3752492 kB | 7666700 kB | — | none | `just security` |
| pre-`build` | 3781800 kB | 7486428 kB | — | none | `just build` |
| 18:08:33Z chain end | 3831780 kB | 7356680 kB | — | none | `just proof` re-run (exit 0) |

Narrow observation that preceded the broad gates (plan: "Until a peak is known, begin with a
narrow single-job gate and observe it"): `CARGO_BUILD_JOBS=1 cargo check -p sea-forge-server
--locked`, shared warm cache, completed in ~2 minutes with `MemAvailable` 3.49 GiB (3663112 kB) — no
pressure excursion. Observed RSS maxima afterwards over the broad gates: **0.99 GiB** (`just test`,
1036928 kB) and 0.53 GiB (`just proof`, 556952 kB) as reported by `/usr/bin/time -v`. Caveat that must travel with these numbers:
that wrapper attributes the waited-for tree for the direct child, and its attribution of deep
descendants (rustc) was demonstrably incomplete in the unrelated Gauntlet observation, so treat
these as lower bounds. The reliable headroom signal is the `/proc/meminfo` series above, which
never fell below ≈3.24 GiB available (3395144 kB, the minimum across the gate series). All memory
figures here convert with 1 GiB = 1048576 kB; an earlier draft of this file used the GB value
(1036928 kB → "1.04") under a GiB label, which independent verification round 3 caught.

Observed peak for the SEA Forge gate set ≈ 1.0 GiB against ≈ 3.24 GiB headroom, i.e. ≈ 3×
margin. Consequence recorded for later tasks: the SEA Forge gate set is affordable at one
gate at a time with `CARGO_BUILD_JOBS=1`; it is **not** authorised to run two heavy gates
concurrently, and the Gauntlet gate set (29 crates, colder cache) was not attempted on the
strength of a narrow single-crate observation only.

Raw: `raw-logs/gate-status.txt`, `raw-logs/gate-*.log`.
## Round-6 correction: which readings are backed, and a headroom warning

Independent verification round 5 found three rows in the table above whose `MemAvailable`/`SwapFree`
values appear in **no** raw log. They were read inline during T00 and never written to a file — the
same class of defect round 1 raised about the `just proof` wall time. Corrected accounting:

| Row | Backed by |
|---|---|
| the six per-gate rows (lint → build) | `raw-logs/gate-status.txt` (the same values, verbatim) |
| `17:57:30Z (prereg freeze)` | **inline observation only** — no raw log retained at the time |
| `18:08:33Z chain end` | **inline observation only** |
| the narrow-observation `3.49 GiB (3663112 kB)` | **inline observation only** |
| the `/proc/pressure/memory` figure `0.84` | **inline observation only** |
| the pressure figure `2.45` (upper bound of the quoted range `0.84–2.45`) | **inline observation only** |

`raw-logs/preflight-readings.log` now exists as a real preflight raw log (three sampled
`/proc/meminfo` + `/proc/pressure/memory` readings, cgroup, `nproc`, and the active-build check at
capture time). It cannot retroactively back the T00 rows, and nothing here pretends it does: the
rows above are labelled inline observations, and the ranges quoted in the brief are bounded by what
was actually sampled.

**Headroom warning for later tasks (recorded because it changed materially during T00's own work):**
at the round-6 capture (2026-09-19T20:05:32Z) `MemAvailable` was **1.89 GiB (1980248 kB)** and
`SwapFree` **3.22 GiB (3380012 kB)** — against 3.24–3.65 GiB and 7.14–7.47 GiB respectively when the
T00 gate baselines ran, with `/proc/pressure/memory` `some avg300` 0.96. The effective headroom for
this plan has therefore roughly halved. Per the plan's `build_resource_policy`, later tasks must
re-run the preflight before any heavy gate and **defer** broad gates while this condition holds; a
deferred gate stays pending, never passed.
