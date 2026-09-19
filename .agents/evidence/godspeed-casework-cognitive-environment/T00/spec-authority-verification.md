# T00 evidence — frozen authority verification (M1, M2)

Worktree: `/home/sprime01/projects/sea-rs/.worktrees/godspeed-casework-cognitive-environment`
Revision: `6ce518fcd9b01bc5a7037f80f5d8986a33cd2924`
Date: 2026-09-19

## M1 — frozen spec hash

```
$ sha256sum .agents/specs/godspeed.casework-cognitive-environment-spec.yaml
6312453fc571ffdd14f14c47a9a7d3d8670fafe76e6dd80d004e9d50852d6641  .agents/specs/godspeed.casework-cognitive-environment-spec.yaml
```
Declared in the plan at `source.spec.sha256`:
`6312453fc571ffdd14f14c47a9a7d3d8670fafe76e6dd80d004e9d50852d6641` — **match**.
Spec metadata: `id: godspeed.casework-cognitive-environment`, `version: 0.2.1`,
`status: authoritative`; plan `source.spec.spec_id`/`spec_version` agree.

`GATE_SPEC_TRACE` (also T00's gate):

```
$ python3 .agents/plans/validate-godspeed-casework-cognitive-environment.py
PASS: spec hash, 86/86 requirements, 15-task DAG, gate activation, and UI removal
$ echo $?
0
```

Bounds checked by that validator and confirmed here: spec is `authoritative`; plan `source.spec`
path equals the real spec path; id/version agree; hash equals current bytes; no
`UNBOUND_`/`__T\d+_BIND_` sentinel remains; exactly 86 unique `REQ-[A-Z]+-\d{3}` records, each
with non-empty text; task set is exactly `T00..T14`; requirement mapping keys equal the spec
requirement set; `spec_requirement_count`/`mapped_requirement_count` are 86/86; every mapped
`(requirement → task)` pair has that requirement in the task's `settles`; every gate has a
command and a valid activation task; `parallel_work_isolation.required` is true; resource-deferred
mode is declared; `GATE_SEAFORGE`/`GATE_GAUNTLET` activate at T05 and are absent from T00/T04 but
present in T13/T14; the plan text never addresses the operator's active `../gauntlet` checkout;
no task uses a gate before its activation task; T14 settles exactly `{REQ-MIG-001, REQ-MIG-002}`;
pre-removal tasks settle exactly the other 84; `GATE_REMOVAL` is in T14; final acceptance
includes mandatory removal; the six high-risk groups are distinct and name requirements their
confirming task actually settles/confirms.

## M2 — traceability counts

```
spec_requirement_count:      86
mapped_requirement_count:    86
unmapped_requirements:       []
unknown_plan_requirement_ids: []
tasks:                       15 (T00..T14)
edges:                       24
critical_path:               T00 → T01 → T03 → T06 → T09 → T11 → T12 → T13 → T14
initial_state.ready:         ['T00']
```

## Tooth 1 (reproduced in this directory)

`.agents/evidence/godspeed-casework-cognitive-environment/T00/teeth/run-teeth.sh` copies the
validator + plan + spec into a scratch tree, confirms the unmutated copy PASSes, flips exactly
one byte of the copied spec, and re-runs:

```
-- control: unmutated copy must PASS --
   control exit=0 (expected) :: PASS: spec hash, 86/86 requirements, 15-task DAG, gate activation, and UI removal
-- attack: flip exactly one byte of the copied spec --
   mutated byte at offset 19: 0x73 -> 0x74
   attack exit non-zero (expected: blocked) :: FAIL: frozen spec SHA-256 differs from current bytes
```

So the binding detects divergence, names it, and blocks (non-zero) until authority is
explicitly re-frozen. Full output: `teeth/teeth-run.log`.