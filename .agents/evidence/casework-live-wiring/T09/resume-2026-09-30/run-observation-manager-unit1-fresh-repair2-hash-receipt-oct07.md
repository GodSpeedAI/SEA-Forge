# Run observation manager unit 1 — repair 2 hash receipt

Date: 2026-10-07. This receipt supplements, and does not modify, the immutable repair record `run_observation_manager_unit1_repair2_record.md` (SHA-256 `2cb8d19a1ee3597744d1e4effbc3b44f5e504d7343135620e50022a8ba1f285b`).

## Frozen identities

* Repaired private source `run_observation_manager.go`: SHA-256 `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`.
* Repaired test fixture `run_observation_manager_test.go`: SHA-256 `6bc6666ee176a56e233636bca0d72fcda439e430a03c0642e4fd2ad695af826a`.
* Pre-edit private source backup: SHA-256 `ee7afab1b22c011cf79b602f415336d459fda346aad1bf0c942a4464326ff878`.
* Pre-edit test backup: SHA-256 `2b2d1fbcd3bfb7e7c27254673ca46b2deb30d9edd4ac862fa464e2a9b18334b6`.
* Rejected review of repair 1: SHA-256 `cb3fd0cb1eb696d10228a345ffe93ff034f3afc9d43fb44efd929770431cc3e5`.
* Repair 1 provenance record: SHA-256 `0c06a7de8d55a6399c53cc2d255b1a4039eef8cf5fa63fa33717653a850c6d23`.

## Repair summary

Retry has a dedicated fake-read start signal emitted before its controlled wait. Shared-initializer cancellation waits for the actual second lease reference in the manager's mutex-protected poller entry. Poller capacity waits for all sixteen unique later case/run keys, then checks eight A contexts canceled, eight B contexts active, refs retained, and workers unjoined before attempting case C. Controlled-read fixtures register cleanup that cancels caller contexts, invokes bounded manager stop/drain, releases held reads, and waits for the join result. The only production-side additions are private manager/poller/lease ownership fields: mutex, exact-key map, actual lease refs, readiness channel, worker context/cancel, and worker-done channel. Prepare and both drain methods remain typed-unavailable stubs; there is no full algorithm, export, callback, test-only production counter/hook, caller integration, or public contract change.

Read-only `gofmt -d` returned no diff. No test, compiler, typecheck, scanner, Git, or runtime action was run. No RED or passing behavior is claimed. This artifact remains pending independent review and root disposition before any actual RED authorization.
