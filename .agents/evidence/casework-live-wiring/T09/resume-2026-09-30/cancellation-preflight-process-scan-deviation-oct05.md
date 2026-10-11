# Cancellation gate preflight process-scan deviation

Date: 2026-10-05. Source/evidence-only supplement. The four gate preflight files, gate raw/exit files, approval documents, source, tests, status, debt, and Git history are unchanged.

## What the archived preflights contain

The four files under `final-cancellation-fixed-fixtures-oct05/` record UTC time, `/proc/meminfo` RAM/swap fields, and only the output of a filtered process pipeline. Each shows one matching line for `bun` and no other matching line:

| Capture | UTC time | MemAvailable | SwapFree | Captured matching comm/RSS |
| --- | --- | ---: | ---: | --- |
| `gate1-preflight.raw` | 17:49:38 | 2,415,252 kB | 1,053,728 kB | `bun 24480` |
| `gate2-preflight.raw` | 17:50:18 | 2,566,024 kB | 1,008,544 kB | `bun 24468` |
| `gate3-preflight.raw` | 17:51:05 | 2,601,276 kB | 948,780 kB | `bun 24464` |
| `gate4-preflight.raw` | 17:51:44 | 2,595,728 kB | 950,272 kB | `bun 24468` |

SHA-256 values, in table order: `25915cb3caabc3374c97e4cdf5450a1fa96783b0eae625a19961b1a5f7a44e17`, `40a39eecc8067501d49fdb133285bdf8238cfbbfadb24347c9b7b673f78c06cc`, `3dd0b88db974634d845d37db42d26a44883f896f19dde74a159a79d3b78d351e`, `ff41a2febce5b677115dd3ba3b53b5951a5d76888c937489388417a027acc7d5`.

## Deviation and evidence limit

The captured command used `ps -e -o comm=,rss= | awk '$1 ~ /^(go|cargo|rustc|compile|bun)$/ {print; found=1} END {if (!found) print "none"}'`. The archived output is therefore filtered to exact comm values `go`, `cargo`, `rustc`, `compile`, or `bun`; it omits the PID column and every nonmatching command. The unfiltered `ps` output was not saved. These files do **not** provide the assigned full PID/comm/RSS process listing, nor do they support claims about compiler-like processes with other comm values. They show only that the filter emitted Bun and no other exact-name match at those four capture instants.

The earlier approval wording that calls this a “full process” scan is overbroad and is superseded by this clarification. Do not reconstruct or infer omitted rows. The RAM/swap measurements and the gate raw/exit evidence remain as captured. Separately, the command sessions were explicitly joined before the next gate was started; the process-output limitation does not change their captured exit codes or timestamp ordering.

For any future gate preflight, capture the complete unfiltered `ps -eo pid,comm,rss` output before the compile command, then separately inspect/report matching compiler process names. No preflight or compiler was rerun for this supplement because a later scan cannot recover the historical process table.
