# Private Next projection TDD source preparation result

Date: 2026-10-08. This is test-first source preparation only. It grants no
algorithm, lifecycle integration, public wiring, runtime, or T09 approval.

## Authority and identities

The full assignment is preserved in
`run-observation-next-projection-assignment-oct08.md` (SHA-256
`c6bd03f372140f0542f92c3c15a7e6b3f2471d311ef9bd705f0a2acc3c832c9d`). Its
additive malformed-input and gap-flag clarification is
`run-observation-next-projection-root-clarification-oct08.md` (SHA-256
`1a3685eb42c287c54ecbe09830827fbe6edce85aa96349b0f9f724269c526fee`). The
reviewed source basis read before editing was revision 6
(`60498c53f9cf953ed59015dfa338d592b89a5a9f8652483a511ce36ff7a9b99b`), its
addendum (`9439cbfe8cb30fdf7b1beb41c617ff031ab383296b220317a8b7f529ce3b859a`),
and retention initializer correction revision 3
(`b8077e9b086f4fe3f15b9733ad3f1cb6fc6f87ca22c5a989494216c50e26d15e`). The
existing immutable retained state and clone helper were inspected. Both
authorized destination paths were absent before editing.

Only these two new Go paths were added:

| Path | Bytes | SHA-256 | Contents |
|---|---:|---|---|
| `apps/godspeed-casework-go/internal/server/run_observation_delta_test.go` | 13,947 | `6e6e3c0e8388104dca64ea66a3487e0b7174fa75235894bfd5ab669ac04629ce` | Seven focused test functions and twelve malformed/unavailable subcases. |
| `apps/godspeed-casework-go/internal/server/run_observation_delta.go` | 1,651 | `aff93e3a1b01aedd8075be1f59ff52cebb4dc36de59eaf4140cb939cdf148973` | Private window, run-delta, gap, and watermark types plus a typed-unavailable stub. |

The stub returns a generic typed unavailable error for every call and contains
no projection algorithm. The draft tests cover: exact `P < ordinal <= H` frame
and gap boundaries; H and P inclusivity; byte-sorted exact gap IDs; later
ledger identities above captured H; captured window/time/identity and source
ordering; window growth and shrinkage with standing-only updates; opaque
identity reappearance and rewritten payload; optional-pointer ownership and
input immutability; nil/impossible/unavailable captures, `P > H`, and ledger
or frame ordinal disagreement; and a valid empty terminal capture.

## Frozen source identities

All eleven current-observation files still match the accepted canonical Go
retry03 preflight (`run-observation-canonical-go-retry03-preflight-oct08.raw.json`):

* `run_observation_manager.go` — `22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57`
* `run_observation_poller_worker.go` — `b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403`
* `run_observation_manager_authority_terminal_test.go` — `889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b`
* `run_observation_manager_test.go` — `cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86`
* `run_observation_manager_failure_test.go` — `8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8`
* `run_observation_key.go` — `a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b`
* `run_observation_retained_version.go` — `2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd`
* `run_observation_retained_version_test.go` — `34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7`
* `run_observation_retained_policy_test.go` — `e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28`
* `run_observation_poller_image.go` — `3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9`
* `run_observation_manager_retained_image_test.go` — `cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256`

## Verification boundary

No tests, compile, formatter, scanner, build, Git operation, or Graft build was
run. The stub is intended to be compilable but that has not been verified; no
expected-RED result or test compilation is claimed. A different reviewer must
review the two-file source and this full result against both original contract
documents before a separate release for an actual focused RED. No projection
algorithm was implemented.
