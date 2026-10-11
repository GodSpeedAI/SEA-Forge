# Manager Unit1 repair archive provenance

The fresh builder reported its .agents evidence destination was read-only and
left three artifacts in the server package. Root read their actual bytes and
native-patched NEW .agents copies, each original/archive cmp0:

- run-observation-manager-unit1-source-preedit-oct07.raw equals
  run_observation_manager_unit1_source_preedit.txt (ee7afab1 original source).
- run-observation-manager-unit1-test-preedit-oct07.raw equals
  run_observation_manager_unit1_test_preedit.txt (2b2d1fbc original fixture).
- run-observation-manager-unit1-fresh-repair-record-oct07.md equals
  run_observation_manager_unit1_repair_record.md (0c06a7de).

The builder record explicitly reports initial backup copy operations outside
the required native-patch write workflow. Root preserves that historical
deviation; the new native archives establish exact retained bytes and do not
rewrite how the initial copies were made. Package-local artifacts remain
untouched. No permission bypass, source implementation, test run, or approval
is implied. Independent source review of aad810b9/64dc5a07 remains required,
especially shared-initializer attachment synchronization.
