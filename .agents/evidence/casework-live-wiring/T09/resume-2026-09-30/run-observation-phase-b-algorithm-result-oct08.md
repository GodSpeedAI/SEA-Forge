# Phase B manager and worker implementation result

Date: 2026-10-08

## Scope and source identities

Implemented the released Phase B algorithm only in:

| Source | Released preimage SHA-256 | Current SHA-256 | Bytes |
|---|---|---|---:|
| `apps/godspeed-casework-go/internal/server/run_observation_manager.go` | `b7f0511b5a1690253d3adb0132beafc106d3b03f5218c6bde3f1c40f4217884a` | `d7be1d7a724ed65575ccaaf35cdf1aa5af133dbdd4952d592699b7eb443b6350` | 26,384 |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go` | `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` | 8,873 |

The full release and exact source preimages are archived in
`run-observation-phase-b-algorithm-source-assignment-oct08.md`,
`run-observation-phase-b-manager-preimage-oct08.json`, and
`run-observation-phase-b-worker-preimage-oct08.json`. Their decoded preimage
bytes and hashes were verified before editing.

## Implemented paths

The manager now reserves per-Prepare leases and creator accounting atomically,
validates cancellation and exact cohort membership before attachment, reserves
bounded poller batches, publishes a single outer-hydrated DTO, and rolls back
through per-lease drain ownership. Stop closes admission, signals lease and
poller cancellation outside the mutex, and joins registered creators and
workers before completion. Detach drains only its lease. Selected-A release
uses exact forward/reverse membership and joins only a watcherless worker.

The worker now has one real eligibility claim, one continuation for claimed and
unclaimed launch paths, actual port calls outside the mutex, one-second
post-read cadence, full-image validation before candidate publication, and
single-owner readiness and worker completion. Stop-before-read produces the
existing code-4 marker with nil current state. Terminal retention markers
initiate lease drains after publication.

During final static review, two narrow repairs were made within the released
files: terminal-state staging locals were moved from `stopPollerBeforeRead` to
`finishPollerRead`, where their values are defined; an adjacent nested `if` in
`takeLeaseDrainLocked` was split onto valid Go source lines. No state field,
enum, result code, public surface, helper, fixture, or additional file was
added or changed.

## Frozen inputs checked

| Frozen file | SHA-256 |
|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `apps/godspeed-casework-go/internal/server/run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| Phase A failure fixture `apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go` | `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c` |

The listed primitive/helper files and both manager fixtures were only read and
hashed. No test, compiler, formatter, scanner, build, or Git command was run
for this implementation result. This is a source identity and static-review
receipt; it does not claim compilation, runtime correctness, GREEN, or lifecycle
approval. The existing Phase B fixture remains the expected-RED fixture until
an independent critic reviews this implementation and the authorized focused
verification is run by the root integrator.

Graft context retrieval was used before source inspection (approximately 4,300
tokens saved versus broad source reading).
