# Independent initial T09 fixture rejection

Luna critic t07_live_confirmation rejected both frozen test sets; no compiler/test ran.

Canonical defects: existing Criterion12 schemaFiles inventory still omits the Ask schema;
a duplicate list asserts its own literal presence tautologically. Nested run/frame schema
property and required sets and command-finished field conditions are not pinned. An untyped
unavailable literal proves only its own shape, not optional cohort counts in the public DTO.

Cap defects: actual32MiB boundary, poison, truncation, inspect retry, mutation recovery and
oversized event tests exist, but lower configured cap and invalid over-ceiling configuration
were omitted. A missing Config field compile error is not substantive behavioral RED.

Fresh builders are t09_contract_fixture_repair and t09_cap_config_fixture_repair. The first
repairs only contract-conformance.test.ts against the original assignment and these defects.
The second preserves all five cap cases and adds configured-limit coverage. Root authorizes
one narrow compileable seam: MaxResponseLineBytes int in existing Config, without default,
validation, reader enforcement or other implementation. This is an explicit test-stage scope
exception, not a claim of implemented limits. Existing New(Config)(*Client,error) remains.

Chosen approved Config semantics: zero defaults32MiB; positive values up to that ceiling
select a smaller cap; negative or above-ceiling values return KindConfig before dialing.
Line bytes include the terminating LF. All response/event/recovery paths must obey the cap.
Behavior remains unenforced during this test stage so RED can distinguish the implementation.
No reflection, compiler suppression, new dependency or signature change is required.

Root owns the sole compiler token; both fresh builders must freeze without running tests.
Independent critic receives original instructions plus repaired fixtures and this explicit
seam amendment before source approval and serial expected RED.
