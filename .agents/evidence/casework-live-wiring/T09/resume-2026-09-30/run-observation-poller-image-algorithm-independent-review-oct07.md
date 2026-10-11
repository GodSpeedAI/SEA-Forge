# Independent review: poller image pure encoder implementation

Date: 2026-10-07  
Verdict: **READY for a separately authorized focused GREEN.**  
Scope: source-only review; no tests, compiler, formatter, scanner, runtime, or Git command was run.

## Governing instructions and evidence

- Full algorithm assignment `run-observation-poller-image-algorithm-assignment-oct07.md` (SHA-256 `151f09cc3d187eaef91ddf08228199f86f35d08cda301d67e19bb81ac089ca65`).
- Full source result `run-observation-poller-image-algorithm-result-oct07.md` (SHA-256 `aa8dd73fea50ffe6f727f244b14099118b132a9dafdb819a378d35a69f38b3d8`).
- Original accepted fixture assignment/release and the full source-independent RED acceptance/result, including captured actual focused output.
- Current implementation `run_observation_poller_image.go` (SHA-256 `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`).
- Frozen exact-oracle fixture `run_observation_manager_retained_image_test.go` (SHA-256 `8e741f7604f49bf28674e669f820ec9e696abd84ad4c60bb2a145b36123496f8`).

The five original frozen files were rehashed and match the assignment: manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; policy fixture `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

## Code review

The implementation in `run_observation_poller_image.go:56–96` matches the bounded private encoder assignment:

- It rejects phase codes above 3 and initializer-result codes above 5 before encoding, always returning nil bytes and the fixed generic error.
- For nil current, it marshals the explicit ordered schema/phase/result/key branch; the key components are emitted once and there is no current or synthetic helper value.
- For nonnil current, it requires exact key equality, delegates to the unchanged `marshalRunObservationRetainedImage`, maps helper failures to the fixed generic error, and places those canonical bytes in `json.RawMessage` in the explicit schema/phase/result/current branch. There is no outer key.
- It measures the complete wrapper after `json.Marshal`, returns nil bytes on marshal failure or a final size above the shared 1 MiB maximum, and returns the newly allocated wrapper bytes otherwise.
- It does not normalize keys, mutate the helper state, add payload validation, truncate/prune data, change the helper, or introduce manager map/ref/worker/port/cancel/JOIN or publication behavior.

The frozen seven-case fixture’s full-current byte oracle compares the wrapper against the exact helper bytes; its independent candidate wrapper proves the exact cap-plus-one case. The previously accepted RED output showed the stub failures only on positive encoder cases and passed the two refusal-only tests; this is adequate expected RED evidence and is not confused with GREEN.

## Material differences and limits

No material deviation from the original pure encoder assignment was found. The accepted timestamp-based `+1` fixture construction remains tied to a measured one-byte inner change and an independently assembled full wrapper. The algorithm assignment correctly attributes prior fixture-oracle work to its separate builder and preserves the exact fixture hash.

This review approves only the exact implementation source for a separately authorized focused GREEN. No test or formatting result is claimed. A focused test command and any read-only `gofmt -d` diagnostic remain subject to root’s separate heavy-token grant; no format pass is asserted here. Manager lifecycle, admission/no-read, publication, public API, or broader package behavior is outside this implementation and remains unproven.
