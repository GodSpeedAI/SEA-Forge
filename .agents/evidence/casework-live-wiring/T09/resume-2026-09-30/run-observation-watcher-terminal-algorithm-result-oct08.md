# Watcher and terminal algorithm source result

Date: 2026-10-08  
Builder: `manager_lifecycle_scope_recon`  
Reviewer: independent source critic pending; root retains architecture and gate ownership.

## Scope and source identities

Implemented under [the full source assignment](run-observation-watcher-terminal-algorithm-assignment-oct08.md) (SHA-256 `a886842c0d7d7ba4673916862a6bd56d1b2d2b336fd6342a983cda697e714ab2`). Exact original preimages were captured before source edits in the manager, worker, and fixture preimage JSON records alongside this result:

| File | Original SHA-256 / bytes | Final SHA-256 / bytes |
|---|---|---|
| `run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` / 26,548 | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` / 29,377 |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` / 8,873 | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` / 13,217 |
| `run_observation_manager_authority_terminal_test.go` | `ea181a2f5781a7b11fd683642cca89b89c2d902ea44d8ea276a5c9c5c1d4a829` / 41,237 | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` / 41,286 |

The fixture source change is only standard `gofmt` output. Its three formatting hunks add no test behavior, helpers, callbacks, or assertions. The manager and worker contain the authorized private algorithm implementation; no public surface, schema, port, auth policy, or dependency was changed.

The eight frozen server files were verified by full hash: `run_observation_manager_test.go` `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; `run_observation_manager_failure_test.go` `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4`; `run_observation_retained_version.go` `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; `run_observation_retained_version_test.go` `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; `run_observation_retained_policy_test.go` `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28`; `run_observation_key.go` `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`; `run_observation_poller_image.go` `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`; `run_observation_manager_retained_image_test.go` `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`.

## Implemented boundaries

- `run_observation_manager.go:321-572` keeps admission, list, atomic reserve, unconditional launch-before-wait, initializer result classification, canonical combined image bounding, and DTO creation in the existing prepare flow. It performs authorization/current-case guard checks at the list, attachment, and final disclosure boundaries. Final disclosure additionally checks the exact lease and retained-state identities under the manager mutex.
- `leaseCanPrepare` and `leaseCanDisclose` (`:573-648`) receive the already-derived prepare `Done` channel and perform only a nonblocking receive while holding the mutex. This closes the cancellation gap between an outside-lock authorization/guard check and successful DTO handoff without invoking context methods under the lock. The same pattern is used by `reservePollerBatch` (`:141-217`).
- `run_observation_poller_worker.go:19-58` keeps production read eligibility and the mandatory worker continuation on the real worker path. `authorizePollerWatchers` and `authorizeLeaseForPoller` (`:161-267`) snapshot exact eligible forward membership, validate each watcher’s caller/current session and case/cursor/parent outside the mutex, drain invalid watchers through existing ownership, and recheck membership so a newly attached eligible watcher must also be checked. The worker checks its context outside the lock immediately before the port call.
- `finishPollerRead` (`:270-430`) encodes and caps the complete candidate image before pointer publication. Fitting accepted terminal snapshots are published as current and stop recurring claims. If the outer image cap forces terminal fallback, stop/drain is derived from the final fallback phase and marker; the prior retained state is preserved. Publication reauthorizes watchers and rechecks exact entry, prior pointer, phase, and validated eligible membership. Terminal cancellation/draining occurs after unlocking.
- The final prepare path captures the prepare context’s `Done` channel outside the mutex, checks it nonblockingly at the mutex-protected handoff boundary, and returns typed failure with zero DTO/nil lease on cancellation or lost membership. Auth and guard callbacks remain outside the mutex.

## Formatter witnesses and verification limits

Each formatter was stdout-only, exited 0, and its exact emitted bytes were archived before proceeding to the next formatting command. The archived JSON witness has the pre-format source identity, command, exit status, and full stdout. Decoded stdout equals the final source byte for byte:

| File | Witness | Exact stdout SHA-256 / bytes |
|---|---|---|
| Manager | `run-observation-watcher-terminal-manager-gofmt-witness-oct08.json` | `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57` / 29,377 |
| Worker | `run-observation-watcher-terminal-worker-gofmt-witness-oct08.json` | `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403` / 13,217 |
| Fixture | `run-observation-watcher-terminal-fixture-gofmt-witness-oct08.json` | `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b` / 41,286 |

Fresh bounded process/memory preflights are archived beside each witness. They expose only PID, parent PID, state, executable path, and memory figures; at those snapshots only the Codex process and the Python preflight process were visible. This is not a claim about all host processes. One earlier formatter-capture attempt failed in the orchestration script before archiving because the JS isolate lacked `import`/`TextEncoder`; the source was not changed by that attempt. The formatter was rerun, its actual stdout archived, applied through `apply_patch`, and byte-compared successfully.

No compiler, test, vet, scanner, or build command was run by this builder. The accepted focused RED is prior evidence only; this result makes no GREEN, semantic-approval, or runtime claim. The resumed source already contained partial algorithm work from the same authorized assignment; it was preserved and completed rather than reconstructed from the rejected builder’s files. At resumption the manager and worker were `2caff199daf2fdba73fe022973c3a22466383be3d5c1c47f8f4c37763ecf88ea` and `ff451845ed594b838e814e588ac33c763830259c2bd29c71dbae5b1dd6d15328` respectively. Subsequent in-scope edits completed the authorization/terminal behavior described above.

The independent critic must review the full assignment, preimages, final source, fixture formatting-only delta, and the existing focused tests before any runtime gate. Remaining uncertainty is semantic/compiler correctness until that review and root-owned verification complete.
