# Current status

**Status revision:** 243

**Stage:** FDE integration branch: case.record.append Go builder, golden frames and spec note (WP-14, part 2, steps 3-4)

**Summary:** fde/integration: ran the repo's contract generators for the new verb (gen_sfwp_schema and bun run generate:contracts write 64 schema and TypeScript files, byte-identical to the committed ones: the verb's bodies are contracts/fde schemas, not schemars types, so no generated output exists for it). The Go client is hand-written, not generated: added NewCaseRecordAppend and the case_record_append durable-mutation entry in internal/adapters/sfwp/frame.go. Golden frames case.record.append.{freeze, freeze_replayed, refused_free_key, settle} were captured from the real server by a new live-tagged capture test that merges by name into testdata/frames.json (the other 32 frames are untouched) and pinned by a non-live test; the capture also asserts the real server delivered 3 interaction events. Documented the verb and its semantic exhaust in the Workbench API spec method table and method catalog.

**Verified:**
- go test ./... in apps/godspeed-casework-go passes (TestGoldenCaseRecordAppend, TestNewCaseRecordAppendRequiresBothKeysAndMarksTheVerbDurable)
- GOLDEN_CAPTURE_APPEND=testdata go test -tags live -run TestGoldenCaptureCaseRecordAppend: ok
- cargo run -p sea-forge-server --bin gen_sfwp_schema and bun run generate:contracts: no diff
- scripts/check-agent-context.sh passes

**Limits:**
- A repeated request_id is answered from the transport correlation record (replayed as first returned); contract-level replay needs a new request_id with the same idempotency_key, which is what the replay frame records
- Golden frames embed a case id and digests of the capture run
- Spec catalog authority class case_control is the nearest existing class; no policy evaluation runs for this verb
- AT-05 wiring is the next step
- Integration branch only: not merged to main, no PR

**Next:** AT-05 SEA-Forge half (PARTIAL until WP-15) and final gates.

**Evidence:**
- apps/godspeed-casework-go/internal/adapters/sfwp/record_append_frame_test.go
- apps/godspeed-casework-go/internal/adapters/sfwp/golden_capture_record_append_test.go
- .agents/specs/frontend/sea-forge-workbench-api-spec-v0.1.md
