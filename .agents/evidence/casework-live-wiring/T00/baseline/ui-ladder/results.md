# E2E Test Results

| Journey | Status | Depends On | Settles | Steps | Notes |
|---------|--------|-----------|---------|-------|-------|
| J0 | PASS | — | A stable CORE-centred orientation surface from which selection is possible | 6/6 | — |
| J1 | FAIL | J0 | Focus is a reusable primitive that preserves CORE as orientation and object identity | 4/5 | Failed: focus again: identity and state survive |
| J2 | BLOCKED | J0, J1 | Artifacts disclose progressively (pill → excerpt → viewer) without destroying spatial context | — | Blocked by J1 |
| J3 | BLOCKED | J1 | Orbital, causal and comparison are projections of the same objects (identity, selection and state carry across) | — | Blocked by J1 |
| J4 | BLOCKED | J2, J3 | Time/version and comparison are trustworthy operators that never mutate source history | — | Blocked by J2 |
| J5 | BLOCKED | J2, J4 | Narration composes focus, arrangement, artifacts and time through the same grammar, and is interruptible | — | Blocked by J2 |
| J6 | BLOCKED | J2, J3 | The causal projection is a working thinking representation with inspectable evidence | — | Blocked by J2 |
| J7 | BLOCKED | J3, J6 | Case Design is a bounded thinking/proposal environment that never mutates authoritative state | — | Blocked by J3 |
| J8 | BLOCKED | J2, J7 | Human and agent consequences share one intent path; authority, execution, evidence and settlement are distinct | — | Blocked by J2 |
| J9 | BLOCKED | J4, J5, J6, J7, J8 | The full reference path works end to end on settled primitives, with no deep links | — | Blocked by J4 |
| RECOVERY | BLOCKED | J2, J4, J5 | Failures stay bounded; authoritative frontend state and the persistent Core survive | — | Blocked by J2 |