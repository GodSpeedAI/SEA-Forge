# Gate policy fixture

Real-OPA Rego fixture whose projected decision can be allow, deny or escalate, unlike the constant-true `fixtures/cells/repair-agent/rego` bundle.
Source: `source/model.sea`, copied verbatim from the GATE model in DomainForge `domainforge-core/tests/rego_projection_tests.rs`.
Generated with: `domainforge project --format rego --created-at 2026-07-11T00:00:00Z fixtures/policies/gate/source/model.sea fixtures/policies/gate/rego` (binary `domainforge` from `domainforge-core` 0.22.0).
DomainForge release: `v0.22.0` (commit `ac8c421`); output matches the 0.21.2 projection except the `generator` field and `bundle/domainforge/sea/lib.rego` (0.22.0 makes `exists_unique` three-valued: an unknown item with at most one true item yields unknown), so its `provenance.json` digest changed `sha256:3b25cf21...` to `sha256:bff56dc0...`.
Verify: `opa eval --format json --bundle rego/bundle --input inputs/<case>.json data.domainforge.policy.decision`.
- `inputs/allow.json`: one entity with name A, capacity 10, kind y. Outcome `allow`, all results `true`.
- `inputs/deny.json`: one entity with capacity 500 violates the obligation. Outcome `deny`, `gate::must` = `false`.
- `inputs/escalate.json`: one entity without `capacity`. Outcome `escalate`, `gate::must` = `unknown`.
