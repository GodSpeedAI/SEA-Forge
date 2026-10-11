# Lease-cardinality test-matrix supplement assignment

Date: 2026-10-07  
Builder: fresh different documentation builder after lifecycle-matrix review.

## Full assignment received

Read the full `manager-combined-image-protocol-repair-supplement-independent-review-oct07.md` (SHA-256 `026aa35a6326ed1c3f8223ddda80e8b2d2fc70a3ad00e09dc6d9665edba5e9cf`) and the previous supplement assignment/result:

- `manager-combined-image-protocol-supplement-assignment-oct07.md` (SHA-256 `408df95df56b5ebae6dfee1c82afb00ada18d9b7c69a0449c3dbbc7d3f5b3522`)
- `manager-combined-image-protocol-repair-supplement-oct07.md` (SHA-256 `474fcc42c544c0a3dc21092cf3dbf532f6c7541d2ca6ca07fb76deffe4b85307`)

Create ONLY one new immutable narrow lifecycle test-matrix supplement. Do not edit any prior evidence, source, test, fixture, or prior assignment/result. Archive this original instruction before writing the new supplement.

The reviewer rejected lifecycle test case 3 because one Prepare call owns one cohort lease; independent Prepare waiters do not share one lease drain completion. Root agrees: one cohort lease per Prepare, sole drain owner/completion per lease, and a shared poller is not a shared lease. The supplement must replace case 3 with exactly two clearly distinct scenarios:

1. Multiple pending independent Prepare calls each have a distinct lease but share one initializing poller. Global manager Stop wakes all. Each lease's first drain transition removes only its own exact refs; each Prepare finishes its own operation before joining its own lease drain; manager Stop joins the shared poller worker exactly once; no lease survives global stop.
2. Cancel or roll back one pending Prepare while another distinct authorized lease survives on the shared poller. The canceled Prepare's sole lease-drain owner removes only its own refs exactly once. It must not cancel the shared worker because another eligible lease remains. The surviving lease receives the single initialization result.

Cite the actual package contract/scaffold: `prepare` returns one `*runObservationLease` per call, each lease contains its own poller membership map, and each poller tracks refs by lease. Cite the root reference ownership decision and the independent review. Enumerate material differences from the prior supplement.

Do not expand encoder tests or alter image encoding, cap values, or field/code choices. Keep pure encoder work separately released; lifecycle source remains unreleased. Preserve all prior records. This is documentation-only: no source/test/compiler/build/scanner/runtime/Git. A different critic will review the original instructions plus result. No implementation or fixture release is granted.

## Frozen identities

The previous supplement records these exact source identities, which remain frozen: manager `fa1601f3746bca6c6697e5e6c6861bb9442bac6aafc2aa762a4f32580f8a905d`; manager fixture `af2dfcb8c83d8a53186d8e446a11de35ce51d45cac74461ea3c3f971d0da30c4`; helper `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`; base helper fixture `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`; policy fixture `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

