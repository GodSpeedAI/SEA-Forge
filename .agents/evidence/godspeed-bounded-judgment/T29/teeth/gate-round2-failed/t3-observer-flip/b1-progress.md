# T29 B1 progress notes (append-only)

## 2026-09-19 — start

- Builder B1 dispatched for T29 (native route-discovery breadth, second
  operation family; family `e2e-forged-report` x budget ladder).
- Prereg verified frozen: sha256 e844fce47c9860eeaacac0b8d27f0d28ef386da21f1c45ed878e85ba02af9672
  = the sha in decision D-2026-09-19-T29-01.
- T21 frozen modules re-hashed; ALL match T21/evidence-manifest.yml:
  route_contracts.py 0d8023877846c606bb1359d1c4ae8a683706caf7f4e0d7318165c124dac7fe8c
  admission.py 8608b495eebb6e281f50e41ee7c22317260b4219263247161789870d792bf49b
  selection_policy.py 9b25b0c8c9a65ce0771f1821b4f34b4161adfdb11dd5c5aa1da620c7b19ff5ef
  operations_catalog.yaml 85327cb23ac35df21e767c4cfe603cd287a7724906e453f4ab1d589731fec7dd
- Family-1 frozen ledger read (T23 round-3 mechanical-labels.yaml +
  route-record.round3.yaml): 12 probes (8 honest e2e-happy, 4 lying
  e2e-lying); all 12 pinned state DBs re-hashed under
  `$HOME/.local/share/godspeed-route-discovery/T23/route-001-r1/runs/` —
  every pin matches. G3 inputs are intact.
- Environment checks: gauntlet debug binary present + executable; fixture
  target present (GAUNTLET.md budget lever `max_rounds: 4` at line 19);
  forged-report fixture re-read (its workspace effects carry
  `{"tests": N, "failures": 0}` passing-claim artifacts; scripts/check.sh
  really fails on the missing marker); opencode 1.18.13 present (the frozen
  provider seam used by T23/T24 today).
- PYTHONDONTWRITEBYTECODE=1 for all python invocations; no cargo/just/build
  anywhere in this task.
- Next: author harness.py, freeze its sha256, then iteration 1.

## Frozen harness identity (written before first execution)

- harness.py sha256: fd555c9eb8536039fd0ee20d3e83abee8af66748f091b9de00c5ebda714b0f9c
  (syntax-checked only — never executed before this freeze; one typo fixed
  pre-freeze in the MAX_RETAINED constant line, then re-hashed; this is the
  executed identity. No overwrite after first execution; corrections use
  round-2 names.)
- Pre-execution hygiene: the syntax-check created a __pycache__/ directory in
  the T29 tree despite PYTHONDONTWRITEBYTECODE=1 (py_compile writes
  explicitly); it was deleted before the first execution. No other debris.
