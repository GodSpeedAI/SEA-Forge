# Independent failure-fixture formatting review

Date: 2026-10-08

## Verdict

APPROVE the formatting-only source claim. The exact preimage decodes to the archived source identity, the captured formatter stdout is byte-for-byte identical to the current file, and the full diff is one standard alignment-space change in a constant declaration. The ten other frozen source files match the canonical preflight. This is not test, build, or runtime approval; I ran none of those commands and performed no formatting write.

## Machine-derived identities

Full original assignment run-observation-failure-fixture-format-assignment-oct08.md, SHA-256 a95792d543b17e0bf842cebdb49cfbe9bf494cf905a56a595013cd01f4e6fc87. Builder result run-observation-failure-fixture-format-result-oct08.md, SHA-256 38d60c7e8503186cf040e3935260ce78013870438c674d93bc7d3c3c7d9071a9.

| Target | Bytes | SHA-256 |
|---|---:|---|
| Decoded preimage (apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go) | 47657 | ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4 |
| Current source / exact captured formatter stdout | 47658 | 8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8 |

Preimage wrapper run-observation-manager-failure-fixture-format-preimage-oct08.json, SHA-256 3e68ccee15e867db6c110e5227e1f13da1a65e5235ba40fbbf8f9ca58853d454, declares source path, original byte count, and original digest; decoded content matches. Exact formatter command was gofmt apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go; its preflight captured UTC 2026-10-08T13:08:34Z; SOURCE apps/godspeed-casework-go/internal/server/run_observation_manager_failure_test.go; SHA256 ccbbe234785365a266cd7ea10737158ca3eea2d4a8a88a9f28c24b7260b190f4; BYTES 47657.

The exact preimage diff contains 1 hunk (@@ -18,7 +18,7 @@): one additional ASCII space aligns the run ID constant with adjacent constants at run_observation_manager_failure_test.go:19–24. There is no other source-text difference. Captured formatter exit content is 0; stderr is empty. Current source bytes equal captured stdout exactly.

## Other ten source identities

These ten files were hashed from current bytes and each matches the corresponding actual canonical preflight identity:

| File | Bytes | SHA-256 | Match |
|---|---:|---|---|
| apps/godspeed-casework-go/internal/server/run_observation_key.go | 193 | a6f0114d73aefdf816d35df7fd0b18d708924c3d673f78472e79c283bef3557b | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager.go | 29377 | 22005e5c15d60c4e99768ac6c69da2d211b816ef16c20663231e8a9fc164dc57 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_authority_terminal_test.go | 41286 | 889f9648ead4399ef2d89732422ffed9a3d1ff786e4938b2aafaa45862b2752b | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_retained_image_test.go | 11463 | cf501bf7d65a413a2c575b951980be3d0e301f3515a991682680b2d3baa4f256 | True |
| apps/godspeed-casework-go/internal/server/run_observation_manager_test.go | 56118 | cf7188c23e5bad5f7b0f093561bf85e1e273b48c721a677084429e8038f0ba86 | True |
| apps/godspeed-casework-go/internal/server/run_observation_poller_image.go | 3208 | 3dba418f6f4920f89bb10ec59e9b77076e3005d3a03d7504212887369b5ed3f9 | True |
| apps/godspeed-casework-go/internal/server/run_observation_poller_worker.go | 13217 | b0fde3fed983099ec78754a71578357f82b79988a6a15e1c54962e1e1390d403 | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_policy_test.go | 13240 | e156c9cf0281baa4e532131be2606a280c846de2f2e3ce9b4fa3dc03509d3f28 | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_version.go | 10456 | 2157583fbb74f519c2e4254bf1fd26fce330ed567ac7ddaf006b510c87fd50bd | True |
| apps/godspeed-casework-go/internal/server/run_observation_retained_version_test.go | 26425 | 34df05f171f2fbb513caf86d096a87b9c3a4f353b2a5bef236d08132b040bfd7 | True |

## Capture provenance and limits

All five capture archives—preflight, command, formatter stdout, formatter stderr, and exit—were decoded and compared byte-for-byte with their actual original temporary files. The paths and hashes below are read from the archive contents and computed bytes:

| Capture | Original path | Bytes | Raw SHA-256 | Archive SHA-256 | Byte compare |
|---|---|---:|---|---|---|
| preflight | /tmp/sea-observation-failure-format-oct08.ofckVS/preflight.raw | 198 | 91b146a6899161b458ce7972b70ae5055754d262867f7059979683f6e5688d38 | 1ff92a65f783131d0a96c67310fa6a22f689141f4dc7f2fc13e1073f068bec87 | True |
| command | /tmp/sea-observation-failure-format-oct08.ofckVS/command.raw | 88 | 0932731b66ccd3b65aa26fbab92c037e0679ff82ca785779924f3f1c89f6d3de | a67ccba28a75be6cd27205b93bbbf102a22495b1e8b202f0bfdaece9a561f9a9 | True |
| stdout | /tmp/sea-observation-failure-format-oct08.ofckVS/stdout.raw | 47658 | 8c71197adc61bf0f0a3fb0a0fdf413ff41f1fdfa54e8a4af0c4a3abf63cb8df8 | bf1482548aecdc9bee0a93c25a4c32613ac3c28fc87c6e19e8e2d163d0e26e3a | True |
| stderr | /tmp/sea-observation-failure-format-oct08.ofckVS/stderr.raw | 0 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | b5be92b8231734054eecb883bae9455f7b893dd2b19e333c2dc87e612b3705cd | True |
| exit | /tmp/sea-observation-failure-format-oct08.ofckVS/exit.raw | 2 | 9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa | 8b4d9fb71de9a5a511ea103ebf283b64f43d24dcca15046b5ad6138d0cf14c49 | True |

No material deviation from the formatting-only assignment was found. The prior canonical gate stopped at this formatting check; no canonical retry or tests are included in this review.

