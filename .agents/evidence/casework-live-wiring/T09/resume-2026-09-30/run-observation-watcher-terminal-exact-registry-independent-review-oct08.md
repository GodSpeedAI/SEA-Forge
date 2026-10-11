# Independent review: exact pre-port survivor registry assertions

Date: 2026-10-08

## Authority and limits

I read the complete exact-registry assignment
`run-observation-watcher-terminal-exact-registry-assignment-oct08.md`
(SHA-256 `e2ff3c1b1901e7199fca6d67d353729d5ccc50beab89912a4516e4bb44213c7c`),
the complete original fixture-repair assignment
`run-observation-watcher-terminal-fixture-repair-assignment-oct08.md`
(SHA-256 `61d5bf6d435b73696b8b9adba7718b70b51a7f650a5a8ac40c013a301b65da6f`),
the original TDD source assignment
`run-observation-watcher-terminal-tdd-source-assignment-oct08.md`
(SHA-256 `bfcfa5cd59f53bf5d81de7a349eac669e8899a8b842c863cb4cdb0f580a5ec82`),
the exact-registry result (SHA-256
`6a280314ef6deee988cc76e1ff865a4bde619b957b662245f65b340b5ac15368`), and
the prior review, both scope errata, and governing cleanup/worker-boundary/
membership clarifications. The first independent repair review is
`run-observation-watcher-terminal-fixture-repair-independent-review-oct08.md`
(SHA-256 `a830e3130a3c6c2f59273f788b2dc3638c533e55f4790f33353808bbfa0c1701`);
its disposition is narrowed by errata with SHA-256
`3b0aabe82e17df90fab7350806f24495a229e92d8514b8d623b5aac97cb74b3f` and
`463597733190b68a4adcac3d6287f1d1b4c927c2c6fd5aa67dfdd1e748d4c78e`.

This review is source-only. No tests, compiler, formatter, scanner, build,
Graft build, or Git command was run. Approval below is limited to readiness for
a separately authorized focused RED gate; it is not a runtime or lifecycle
implementation approval.

## Exact source change and identities

The exact-registry preimage wrapper decodes to 41,008 bytes with SHA-256
`51f73f94fed07eb97fd7b9b59d76cbf1ce65278d3acaca5b778c3a91782ad5c8`, matching
the declared metadata. The current fixture is 41,237 bytes with SHA-256
`ea181a2f5781a7b11fd683642cca89b89c2d902ea44d8ea276a5c9c5c1d4a829`. A
read-only decoded-preimage comparison shows exactly one hunk in the permitted
test file, inside the post-cleanup observations of
`invalid-initiator-before-port-survivor-authorizes`:

* At current line 788, `registeredEntry := manager.pollers[key] == entry`
  records exact manager registry membership under `manager.mu`.
* At line 795, `currentKeyMatches := current != nil && current.Key == key`
  checks retained-state identity.
* At lines 798–799, both checks participate in failure and bounded diagnostics.

These checks directly close the prior rejection: stale forward/reverse lease
refs and a retained pointer can no longer stand in for proof that the exact
entry and retained key remain registered after invalid-initiator cleanup.
They are made in the existing read-only mutex observation; no state is
manufactured. The held-read sibling is untouched. The diff adds only these two
boolean observations and their corresponding condition/diagnostic fields.

All ten other source files were independently rehashed and match the repair
result:

| Frozen file | SHA-256 |
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

## Coverage retained from the accepted fixture review

The seven top-level focused groups and six nested cases remain present: two
group-6 scenarios and four final-handoff cells. The three previous repair
findings remain resolved:

1. The recurring-cap oracle validates a matching accepted state and fitting
   canonical inner bytes, independently marshals the existing complete outer
   wrapper, proves its size exceeds the unchanged cap, and checks the bounded
   encoder's nil-byte refusal. The held recurring path follows a fitting real
   initial read, so it exercises the recurring outer-cap branch rather than
   initial retention failure.
2. The pre-port survivor case uses the existing authorization callback and
   gates only on identity equality with a registered poller context while
   holding the manager mutex for comparison; it releases that mutex before
   waiting and calls no context method under it. A survivor attaches while
   that actual worker boundary is held. The test then observes the surviving
   authorization and present-context guard before the real trace callback.
   The existing during-read survivor case remains intact.
3. Groups 3–5 inspect cohort/poller cleanup before test teardown; groups 4–5
   also require the captured worker completion channel to be closed. Both
   group-6 cases inspect invalid lease cohort and forward/reverse removal plus
   survivor ownership before teardown.

No material deviation from the exact-registry assignment was found. The
historical receipt-name mismatch from the earlier TDD source assignment remains
nonmaterial and unchanged; the current exact-registry assignment and result
names match. No formatting status or execution result is claimed.

## Verdict

**APPROVE source readiness for the separately authorized focused RED gate.**
This approval covers only the fixture identity and source-level assertions
reviewed above. The root still owns execution authorization and any later
algorithm/runtime decision.
