# Exact survivor registry assertion repair result — 2026-10-08

## Authority and scope

This fixture-only source result follows the full assignment `run-observation-watcher-terminal-exact-registry-assignment-oct08.md`, SHA-256 `e2ff3c1b1901e7199fca6d67d353729d5ccc50beab89912a4516e4bb44213c7c`, and prior source grant `run-observation-watcher-terminal-tdd-source-assignment-oct08.md`, SHA-256 `bfcfa5cd59f53bf5d81de7a349eac669e8899a8b842c863cb4cdb0f580a5ec82`. The immediate assignment permits only two read-only assertions in the new pre-port survivor subcase. It prohibits tests, compilation, formatting, scanning, builds, and Git operations. This records source presence only; it does not claim RED, runtime behavior, or approval.

## Exact preimage and current identity

Before editing, the complete UTF-8 fixture preimage was stored in `run-observation-watcher-terminal-exact-registry-preimage-oct08.json` with path, source, SHA-256, and byte count. The wrapper’s declaration and decoded source were independently recomputed and match: preimage SHA-256 `51f73f94fed07eb97fd7b9b59d76cbf1ce65278d3acaca5b778c3a91782ad5c8`, 41,008 bytes. The edited fixture `apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go` is now SHA-256 `ea181a2f5781a7b11fd683642cca89b89c2d902ea44d8ea276a5c9c5c1d4a829`, 41,237 bytes.

## Exact change

Only the post-cleanup assertion in `TestRunObservationManagerAuthorityTerminalInvalidWatcherPreservesSharedSurvivor`, subtest `invalid-initiator-before-port-survivor-authorizes`, changed. While holding the existing read-only manager mutex observation, it now records `manager.pollers[key] == entry` and `entry.current != nil && entry.current.Key == key`. The existing condition requires both, alongside invalid lease removal, survivor cohort and forward/reverse membership, non-draining survivor, and retained-current availability. Failure diagnostics report only bounded booleans. No helper, scenario, callback, assertion elsewhere, or production file changed.

The decoded-preimage unified diff contains exactly this hunk:

```diff
@@ pre-port survivor post-cleanup ownership assertion @@
       manager.mu.Lock()
+      registeredEntry := manager.pollers[key] == entry
       ... existing exact lease/cohort membership observations ...
       current := entry.current
+      currentKeyMatches := current != nil && current.Key == key
       ... existing survivor eligibility observation ...
       manager.mu.Unlock()
-      if ... || current == nil || current.Availability != retainedCurrent {
+      if !registeredEntry || ... || current == nil || !currentKeyMatches || current.Availability != retainedCurrent {
-          ... existing bounded ownership diagnostic ...
+          ... bounded diagnostic including registeredEntry/currentKeyMatches ...
```

The actual textual hunk was verified by read-only unified comparison against the exact preimage; the abbreviated diff above omits unchanged membership expressions only for readability.

## Frozen files and deviations

All ten assigned source files outside the permitted fixture remain byte-identical by SHA-256:

| File | SHA-256 |
|---|---|
| `run_observation_manager.go` | `47c95f3ba90abb4355f02d626e664a49027f602e9ac47771c40eda0ba8f22780` |
| `run_observation_poller_worker.go` | `84382ffd0cd77e1c85bd11b607c1427075c4899e1c534fbe7368b0af5ad7c9cc` |
| `run_observation_manager_failure_test.go` | `ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4` |
| `run_observation_manager_test.go` | `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4` |
| `run_observation_key.go` | `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b` |
| `run_observation_retained_version.go` | `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd` |
| `run_observation_retained_version_test.go` | `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7` |
| `run_observation_retained_policy_test.go` | `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28` |
| `run_observation_poller_image.go` | `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9` |
| `run_observation_manager_retained_image_test.go` | `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256` |

No material deviation from the assignment. The earlier historical filename mismatch recorded by the original source critic remains unchanged: the prior source result is named `run-observation-watcher-terminal-tdd-source-result-oct08.md`, not the earlier request’s `...-tdd-result-...`; neither earlier immutable record was overwritten. No compiler, test, formatter, scanner, build, or Git command was run. Independent source review and any later execution remain pending; no RED or runtime result is claimed.
