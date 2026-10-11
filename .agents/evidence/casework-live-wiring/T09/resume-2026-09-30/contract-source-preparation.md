# T09 contract source preparation (read-only; implementation pending T08)

Luna source researcher inspected the approved proposal, actual Rust views and current
canonical contract/goldens. This preparation is not implementation or settlement evidence.

## Bounded unit-one surface

- Authored canonical `.agents/reports/interface-contracts/typescript/types.ts`.
- Event-stream schema plus a new protected Ask schema and shared goldens.
- Explicit schema inventory and contract conformance tests.
- Spec-04 contract specification and governing casework spec amendment.
- Go internal/contract mirrors and golden_test.go; UI ports/contract.ts and wireContract.test.ts.

No Workbench generated schema, Rust verb, dependency or object kind is added. Actual native
SSE no-id behavior requires a server framing test; JSONL envelope goldens cannot prove it.
Existing event schema omits interrupted/error despite both canonical mirrors and goldens
listing them; unit one must restore full exact parity when adding execution_observation.

## Grounded source facts

Rust Thoth ClaimClass (`sea-forge-thoth/src/protocol.rs:41`) has twelve snake-case values:
identity, architecture, declared_capability, installed_capability, demonstrated_capability,
authority_requirements, environment_status, failure_condition, security_implementation,
customer_private, credential_bearing, policy_thresholds. Preserve actual answer/claim views
(`sea-forge-server/src/sfwp/thoth.rs:14,40`) rather than raw GroundedClaim private fields.
Assurance and authority_notice are strings, not enums. Actual current assurance is
local_tamper_evident; authority text does not confer execution authority.

Actual run standing (`sea-forge-server/src/sfwp/case_views.rs`) separates execution
pending/enabled/active/completed/failed/terminated from settlement
unsettled/accepted/rejected/escalated. Case and plan-item references are optional in raw
views; validate exact case/run and actual item parent before emitting a child. Safe trace
frames must project only approved fields/kinds; never serialize raw TraceRow actor/payload.
Command frame execution_status is distinct from run standing: actual core ExecutionStatus
serializes completed/spawn_failed/timed_out/sandbox_violation/suspected_sandbox_violation
(core/src/types.rs478; sfwp/run_views.rs556). A completed command can still have a nonzero
exit code; never infer accepted settlement or successful run standing from that status.

## Root resolves approved-shape naming and read-state semantics

RunTraceObservation names the cohort payload used by RunTraceObservationEvent. The nested
per-run item has the distinct name RunTraceRunObservation. Existing ExecutionObservation
remains an invocation result. This resolves the proposal's overloaded name without altering
its approved cohort topology.

run_list_state is complete/unavailable: complete denotes a successful fully decoded list,
not successful hydration of every run. Cohort observation_state retains approved literals
complete/no_runs/capacity_limited/unavailable. Unavailable dominates capacity_limited when
both failures and capacity omissions occur; retain the exact separate counts so neither is
hidden. no_runs requires a complete scoped list with no runs or unreadable entries.

Nested observation_state is validated/unavailable. Initial entries require successful exact
ownership validation; initial unreadable/mismatched candidates appear only in cohort counts.
Unavailable may annotate a previously validated owned run after a later poll fails, with no
new fabricated frames/standing. Frame truncation is separate metadata, not a read failure.
The builder and independent critic must verify these semantics against the actual views and
all boundary cases before unit approval. No public contract has yet changed.
