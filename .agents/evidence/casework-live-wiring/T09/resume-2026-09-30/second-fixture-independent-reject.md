# Second T09 fixture review and first actual cap RED

Independent critic ran all eight cap tests serially with actual-host preflight (2.84GB
available, no competing compiler), Go256MiB/GOGC50/GOMAXPROCS2/p1/race/count1.
Exit1: missing config validation, lower-limit enforcement, raw overflow poisoning,
inspect retry before decode and mutation correlation recovery fail for expected reasons.
Exact32MiB boundary and truncated EOF do not fail. Raw output is copied exactly in
cap-first-independent-red.log (SHA2915c10f7ec5eecac31414e39666883df2a6ab0f0698bf90eeffe2dfd3fe3822).

Fixture rejection remains: original oversized valid subscription test passes the unbounded
baseline because SubscribeIdle100ms can timeout before the huge line is read. Typed unavailable
and no event are insufficient to distinguish cap overflow from deadline failure. Fresh builder
t08_type_builder repairs only this fixture, requiring meaningful baseline failure and honest
overflow specificity. No production cap implementation is authorized yet.

Canonical inventory/optional-count/nested-schema gaps were repaired, but the command frame
execution_status fixture incorrectly uses run execution standing. Actual core ExecutionStatus
at types.rs478 has completed/spawn_failed/timed_out/sandbox_violation/suspected_sandbox_violation;
run_views.rs556 copies this from command_finished payload.execution.status. Run standing remains
pending/enabled/active/completed/failed/terminated; these are separate semantics. Fresh builder
t09_cap_config_fixture_repair repairs only canonical tests and adds non-completed status coverage.
No Bun compile ran. Critic released the sole compiler token to root.
