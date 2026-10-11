# Independent review: exact-oracle poller image fixture repair

Date: 2026-10-07  
Verdict: **READY for the separately authorized focused RED.**  
Scope: source-only review; no test, compiler, formatter, scanner, runtime, or Git command was run.

## Reviewed assignment, result, and identities

- Full exact-oracle repair assignment `run-observation-poller-image-exact-oracle-repair-assignment-oct07.md`, SHA-256 `cf67136cbb4f5e89e571535a96847ffbcd75c9a593e189b784ef8a3181fedd78`.
- Full source result `run-observation-poller-image-exact-oracle-repair-result-oct07.md`, SHA-256 `d485d611c68f2e480b6f141003113749bd9abedc899703ff6cd2d1cede34822e`.
- Governing accepted assignment `run_observation_poller_image_testfirst_assignment-oct07.md`, SHA-256 `5c8b57bdf3b462fc366783bb304f1aff45af7c4f36a7e58dc3e7a55dada3a2c0`.
- Original two-file RED release `run_observation_poller_image_testfirst_release_assignment-oct07.md`, SHA-256 `4877d97deee28b572b6df553a4e7ac47e51aec26216c22e54e998ee9de49b5e8`.
- Prior source result and rejection were read for context; this review supersedes only the prior fixture rejection for the repaired file identity.

Reviewed source hashes:

- `run_observation_manager_retained_image_test.go`: `8e741f7604f49bf28674e669f820ec9e696abd84ad4c60bb2a145b36123496f8`.
- `run_observation_poller_image.go`: `32f57bd8aa6edb9c20b645e3138b4b9e9fc02076518a34587253a3af31af328e`.

The five release-frozen files were rehashed and match exactly: manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; retained helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; policy fixture `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

## Source and fixture findings

The seam file remains a throwing stub only. It defines the agreed private schema, phase codes 0–3, initializer-result codes 0–5, fixed generic error, and explicit ordered wrapper branch types, then returns nil bytes and that error for every input. It contains no partial encoder logic or manager/lifecycle behavior. This matches the exact source release.

The seven prefixed top-level tests remain intact:

1. Nil-current exact ordered JSON, exact controls/schema/key, absent current, and no synthetic helper fields (`run_observation_manager_retained_image_test.go:11–31`).
2. Current branch now assembles an independent expected full wrapper from the literal schema/phase/result/current prefix, the unchanged helper’s exact marshaled bytes, and a closing brace, then compares every byte with encoder output (`:33–77`). It also checks deterministic output, single key, raw object shape, no wrapper key, and the bound. This closes the prior defect that allowed a partial or substituted inner object.
3. Exact combined limit and `+1` refusal (`:79–144`). The candidate extends the valid timestamp by one byte, measures the helper delta, builds a separate candidate wrapper from the same canonical prefix plus helper bytes and closing brace, and asserts its length is exactly cap plus one without using encoder output. It preserves nil-byte/fixed-error assertions. The timestamp field is an approved, explicitly recorded alternative to key padding; both inner and outer length changes are asserted exactly.
4. Fixed-width marker composition and prior-state immutability (`:146–194`).
5. Oversized nil-current serializer refusal with fixed generic error and no key disclosure (`:196–211`); it makes no map-admission/no-read claim.
6. No input-state mutation and independent returned bytes for both branches (`:213–255`).
7. Invalid codes and current/key mismatch return nil bytes plus fixed generic error (`:257–296`).

The repaired exact-current oracle is strictly stronger than the earlier prefix-only check. It verifies the complete helper image—including canonical metadata and ledger—as raw nested bytes and proves no additional wrapper fields follow current. The independent cap-plus-one oracle now rests on that same exact embedding contract.

## Material differences and limits

Only the required two fixture oracles changed from the previously rejected fixture. The timestamp rather than key-padding input is a material construction detail, but the repair assignment explicitly allowed it with measured exact one-byte inner and outer assertions; it does not alter production behavior or scope. The throwing stub, all seven test names, and all five frozen code/test identities are preserved.

This approval means the exact two-file state is ready for the focused RED command named by the original release. It does not claim that RED was run or establish a RED outcome. Root retains sole compiler ownership and must issue the explicit command grant and fresh resource/hash preflight. No GREEN/algorithm, manager lifecycle, public integration, or broader package test is authorized by this review. The separate lifecycle-cardinality supplement remains approved as design evidence only; no lifecycle source or fixture release follows here.
