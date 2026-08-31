# SEA-Forge Journey Settlement Gauntlet

Automated end-to-end journey settlement testing system for SEA-Forge.

## Scope
This harness drives the real Workbench frontend (Vite dev server) with
`agent-browser`. Most journeys run against a schema-valid mock of the Tauri
IPC boundary (`harness/sfwp-full-mock.js` — validated against the real
generated contracts by `harness/validate_mock_payloads.mjs`), so it does
**not** exercise the live `sea-forge-server` process or a persisted
governance ledger. CJ03 runs the real `cargo test -p sea-forge-domainforge`
suite. CJ07/CJ09 run a real local subprocess and independently recompute
its output's sha256. See each run's `summary.md` "Scope" section for the
current breakdown.

Every gate and settlement oracle verdict is computed from data actually
observed during the run — real DOM content, the mock's own command/query
transcript, real subprocess exit codes, or recomputed content hashes — not
from literals. A journey can genuinely FAIL.

## Directory Structure
- `harness/`: Journey test contract schema, trace schema, failure taxonomy,
  gate definitions, contract generator, settlement oracles, the SFWP IPC
  mock and its contract validator, and generated contracts (CJ01-CJ12).
- `runs/<run-id>/`: Run-specific evidence packages, screenshots, traces,
  snapshots, and coverage reports.
- `latest-summary.md`: Scorecard and pointer to the most recent gauntlet run.

## Running the Gauntlet
```bash
python3 .agents/reports/ux-journey-settlement/harness/contract_generator.py
node   .agents/reports/ux-journey-settlement/harness/validate_mock_payloads.mjs
python3 .agents/reports/ux-journey-settlement/harness/gauntlet_runner.py
```
