# Lifecycle test-matrix supplement assignment

Date: 2026-10-07  
Builder: fresh different documentation builder after independent review.

## Full assignment received

Read the full `manager-combined-image-protocol-repair-supplement-independent-review-oct07.md`, this builder's original assignment `manager-combined-image-protocol-supplement-assignment-oct07.md`, and result `manager-combined-image-protocol-repair-supplement-oct07.md`. The review rejects only the lifecycle test matrix: one Prepare call owns one cohort lease, so several independent Prepare waiters do not share one lease drain completion.

Create ONLY a new immutable, narrow lifecycle test-matrix supplement. Do not edit any prior evidence, source, test, fixture, or current assignment/result. Before writing that new supplement, archive this original instruction in BASE.

Root agrees with the critic's correction: one cohort lease per Prepare; the sole drain owner/completion is per lease; a shared poller is not a shared lease. Replace ambiguous prior future case 3 with exactly two explicit scenarios:

(a) Multiple pending independent Prepare calls each own a distinct lease while sharing one initializing poller. Global manager Stop wakes them all. Each lease's first draining transition removes only that lease's exact refs; every Prepare finishes its own operation before joining its own lease drain. The manager drain joins the shared poller worker exactly once. No lease survives global stop.

(b) Cancel or roll back one pending Prepare while a different authorized lease survives on the shared poller. The first draining transition for the pending Prepare's lease removes only its refs exactly once; it does not cancel the shared poller worker. The surviving lease receives the one initialization result.

Cite the actual package contract/scaffold showing `prepare` returns one `*runObservationLease` per call and each lease owns its own poller map, plus the root ownership decision and the independent review. Enumerate material differences from the prior repair. Do not expand encoding or change caps. Keep the pure encoder separately released; lifecycle source remains unreleased. Preserve all earlier records.

This is documentation-only work. Do not run tests, compiler, build, scanner, runtime, or Git. The different critic will review the original instructions plus result. No implementation or fixture release is granted.

## Frozen identities to preserve

- Independent review: `manager-combined-image-protocol-repair-supplement-independent-review-oct07.md`, SHA-256 `c1d0dbe0c9d2e290b84d04c1e9a06e2b56d0f912` is not assumed; read the exact current file hash before citing.
- Prior supplement assignment: `manager-combined-image-protocol-supplement-assignment-oct07.md`.
- Prior supplement result: `manager-combined-image-protocol-repair-supplement-oct07.md`.
- Root ref ownership decision: `manager-signal-ref-ownership-root-decisions-oct07.md`.
- Frozen source identities from the prior record remain authoritative: manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; policy fixture `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.


