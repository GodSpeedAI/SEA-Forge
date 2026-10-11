# Source-only formatting assignment: retained helper

Date: 2026-10-07

## Authorized scope

Apply only the exact output of read-only `gofmt` to
`apps/godspeed-casework-go/internal/server/run_observation_retained_version.go`.
The authorized pre-edit source identity is SHA-256
`38ca7fdaf45b016fb8a55fdb72a32b15cad100fb5b31585410af943dddf7447a`.
Use native `apply_patch` for the source change. Do not run `gofmt -w`, redirect
formatter output into the source, or make semantic repairs.

The exact prior UTF-8 source is preserved in
`run_observation_retained_version_format_preimage_oct07.json`; its
`exact_text` member must decode byte-for-byte to the authorized source. The
full original algorithm assignment remains
`run_observation_retained_version_helper_assignment_oct07.md` (SHA-256
`cc1fed59eb1f440e53756d3f2adee79369d0c57f17f800481b9c92a1093c446a`), along
with the root decision records and frozen fixtures. This format-only assignment
does not supersede or amend them.

## Frozen inputs and boundaries

- Root algorithm decisions:
  `retained-publisher-algorithm-root-decisions-oct07.md`, SHA-256
  `9dcf3886f98d4416fd1bbcded12ecc939a9f5281b9bebbe3e929dd865185aa3a`.
- Root private-unit decomposition:
  `retained-publisher-private-unit-root-decomposition-oct07.md`, SHA-256
  `f3bcff7bb923a1ac39d744a984044ed838a006485a26b6390a78f07306995c4e`.
- Frozen base fixture:
  `apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go`,
  SHA-256 `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`.
- Frozen policy fixture:
  `apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go`,
  SHA-256 `4c70bc853ae73f4025b177b5fb505d8a456c89c41b1b13ac86faed5cba9620e7`.

Do not edit tests, manager code, the original assignment, decision records, or
fixtures. Do not change algorithm behavior or repair any semantic issue. Keep
the helper as the only authorized source path.

## Required evidence after the formatting patch

Record the gofmt stdout identity/hash and show that the edited source is
byte-identical to that output. Inspect the complete diff and establish that it
contains only the four already identified formatting hunks. Record a lexical or
token-level comparison supporting that no semantic tokens changed. Do not run
tests, compiler, build, Git commands, or a lifecycle check; the separately
assigned renderer/critic owns compiler work. Report the resulting source hash
and identify the exact format-only scope.

