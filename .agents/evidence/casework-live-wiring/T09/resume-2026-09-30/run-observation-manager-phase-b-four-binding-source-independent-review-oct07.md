# Independent review: Phase B four-binding source repair — 2026-10-07

**Verdict: SOURCE READY for the separately authorized 12-case expected-RED gate.** This is a source-only readiness decision; it is not compiler, test, lifecycle, or runtime evidence. Root must separately grant execution after its required fresh resource preflight.

## Exact repair evidence

Reviewed the complete release (`34224d769f5b802cb7cc31b005ae330f671598590e809aede649cd98dc0a89a7`), the four-binding repair assignment (`181983dc3544026c6cad178803c357d76d3a1886d729ae2e20712fb64be9e8af`), and result (`4c272966a7dc7fcd03d42775baf0026c4227ff2ceb39b8135ecd88cf0d9bd816`), along with the prior source rejection. The original preimage JSON source strings decode exactly to the rejected current source files: manager 7,041 bytes / SHA-256 `9e8cb97a0197b1382162c0ce91501538e7d308b380765d845edda328f0a2441d`; failure fixture 46,418 bytes / SHA-256 `06c239e865aeca39fdbbe6f63b92cd23e3c181f1e0af071a6fb9b4b4c0724a1f`. The JSON records themselves were rehashed. Direct unified diffs from those exact preimages show only the four assigned changes:

1. `run_observation_manager.go:123–127` now names both initial parameters (`lease` and `rows`), while retaining the required `prepareDone` channel parameter and unchanged unavailable stub body. This removes the mixed named/unnamed parameter error.
2. `run_observation_manager_failure_test.go:747` uses the map comma-ok result for `registeredLease`.
3. `run_observation_manager_failure_test.go:1056` uses the map comma-ok result for `stillRegistered`.
4. `run_observation_manager_failure_test.go:1082–1083` separates the map comma-ok membership result from `remainingPollers := len(manager.pollers)`.

These repair the four static errors identified in the rejection. The current hashes are manager `b7f0511b5a1690253d3adb0132beafc106d3b03f5218c6bde3f1c40f4217884a` and fixture `ef587b0bd64fe42d5fb56369f620f969610a5bac3d04cd7578911472b1a4893c`.

## Scope, test inventory, and boundaries

The post-repair diff adds no other source changes. The lease still adds only the two authorized private channels; `finishPrepareOperation` still accepts its exact lease; `reservePollerBatch` still has the required channel parameter, and both bodies remain deliberately unwired stubs. The eight approved fixture call-shape updates are still limited to the four exact-lease finish wrappers and four `lease.leaseDone` reservation arguments.

The fixture still has the nine original named tests plus exactly three held-list tests. Their new helper requires actual list callback entry and fails if Prepare returns first. The cases use the real list callback, buffered signals and explicit idempotent releases, read-only cohort inspection under the mutex, and bounded cleanup. The Stop and Detach cases distinguish the held creator's completion from list return; the Detach case keeps B's context and creator alive while A drains and then checks successful B completion. The caller-cancel case returns a readable candidate with nil error only after cancellation and requires the existing `apperr.KindUnavailable` family, zero DTO/nil lease, no trace call, and no candidate membership. No custom error kind, lifecycle hook, cancel-function replacement, private-state write, or sleep was added. The successful B case asserts `creatorDone` after Prepare returns but makes no direct observation claim about the bridge goroutine; the release contract leaves bridge JOIN ordering for implementation review.

The original nine test bodies remain unchanged apart from the specifically authorized eight call-shape updates. The four-binding repair changes only its assigned four expressions. All eight protected files retain their released hashes: worker `f90cd397cd983e42e53498f13baa89fbfeac54e07eb3847b3228592ddd4d929f`; original manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; key `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`; retained version `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; retained-version test `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; retained-policy test `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28`; retained-image test `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`; poller image `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`.

## Limits

No compiler, test, formatter, scanner, Graft build, or Git operation was run. This review establishes source-scope and static source readiness only. It does not claim compilation, expected RED, or lifecycle correctness. A fresh, separately authorized 12-case gate and subsequent algorithm review remain necessary.
