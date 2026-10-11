# Physical admission — bounded test-first builder assignment

Original instructions for builder and independent critic. Proposal approval is
limited to declarations/fixtures, not production or T09 settlement.

## Binding context and permitted writes

Read the initial proposal, independent rejection, complete repair proposal, root
adjudication, wait clarification, round-two review and approving supplement.
Apply repository/crates boundary instructions where relevant and Graft first.
The package is apps/godspeed-casework-go/internal/adapters/sfwp.

Permitted: NEW run_get_admission.go containing the proposed interfaces, fixed
production constructor and minimal compiling STUB declarations; NEW
run_get_admission_test.go and, if genuinely clearer, NEW
client_run_get_admission_test.go. In existing client.go add ONLY the documented
Config.RunGetAdmission field. Read every file before editing. Preserve all existing
code/tests, response-cap/Unit5A fixtures and unrelated operator work. No production
algorithm, client send hook, constructor wiring, manager, source schema, new
dependency, Git, status/debt or compile/gate is released to the builder.

The stub AcquireRunGet must return a fixed typed unavailable error, never a pretend
successful implementation. Minimal declarations may include bounded two-record
storage and package-private controlled clock/timer seams required for observable
tests; identify every such seam and why necessary. Constructor fixes two slots and
one second; test-only shortened timing must not become a production option.

## Observable fixture requirements

Cover same/different-run admission, maximum two owned attempts, exact matching-slot
reuse without fallthrough, no unexpired idle/busy eviction, one-second default,
cooldown at final attempted payload/LF write completion (including blocked and
partial/error writes), all four transport/busy retry attempts, no cooldown when
canceled before writing, cancellation/deadline while queued, callback join and
connection retirement/release before permit release, and bounded two-record state.
Use nearby controlled-connection patterns and bounded deadlines/completion channels;
no unbounded WaitGroup or timing-sensitive fixed sleeps.

Mandatory no-spin case: same-run record remains busy after its cooldown expires,
another idle record is already expired, then a same-run waiter must wait on change
or context without repeatedly creating immediate timers. Controlled clock/timer
observation must prove no spin, cancellation and normal acquisition after release.

Preserve Ask/mutation no-resend/refusal behavior and provide focused regression
coverage for run_get-only discrimination. Test transport integration against the
unchanged client: assertions must distinguish the absent hook, not fail to compile.
Use helper permits/admission doubles when that isolates call ordering; tests must
not reproduce implementation algorithms and assert their own behavior as proof.
No manager logical-budget, production hydration, SSE or UI claim belongs here.

## Delivery and independent verification

Return exact file hashes, full changed-path list, fixture inventory and every
material difference/omission. Do not run any compiler. A different independent
critic will receive this ORIGINAL assignment plus the COMPLETE actual result,
review source first, then receive sole compiler ownership for assertion RED with
fresh RAM/swap preflight, controlled Go memory, writable cache, count1/race/parallel1,
real session exit joins and exact immutable raw/exit captures. Compile/setup failure
is not expected RED. Only approved fixtures and actual assertion RED permit a
separately bounded fresh production assignment.
