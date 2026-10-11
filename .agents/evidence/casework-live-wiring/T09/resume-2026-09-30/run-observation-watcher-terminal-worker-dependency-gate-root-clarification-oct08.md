# Identify the actual worker authorization dependency

Date: 2026-10-08
Applies to fresh fixture-only repair61d5bf6d, group6 pre-port subcase.

The barrier must identify the actual worker's authorization dependency.
Counting the initiating session's second Current call alone can accidentally
gate a legitimate additional Prepare authorization check before attachment or
launch. The test must permit such extra fail-closed checks.

Use the existing injected authorize callback's context and read-only exact
entry/context identity to distinguish worker validation from creator Prepare,
then gate before synchronized Current validation. Context identity comparison
is allowed; invoke no context method under the manager mutex. Do not replace
or mutate a live manager field to manufacture the transition. Construct the
dependency through the existing manager constructor. No production hook,
field, mode, auth policy change or exact auth-call-count requirement is added.

If a different existing dependency barrier supplies equivalent direct proof
that the worker is held, explain that evidence to the independent critic.
All gate/release/teardown paths remain bounded. A premature real trace call on
the unrepaired algorithm is an expected missing-check failure, not permission
to weaken the actual-worker boundary.
