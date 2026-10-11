# Initial lease files: formatter-only repair result

## Full release instruction

> FRESH formatting-only builder for Go lease files, independent from their implementation builder. Canonical03 actualfailedgofmt BEFOREvet/tests; /tmp/initial-lease-canonical03-62830ua6 captures currentlyrootarchiving. Source frozen manager2d01953469e3d7eedc5688ceb68c9e2d64aca86590251dcd11f3a59afbb3a6cd,test08852d598e91264a81a745ac9ba08b31e7eeb5cdb60dd01324e578967f260f50. ONLY exact gofmt output of these TWO files via nativeapplypatch, no semantic edits. Read originalfull leaseassignment/implementationresult/sourcecritreview; root focusedmanager race01passed33top+6nested0fail, ALL6rootcmp. Derive formatter dryoutput perfile, compareinverse/preimage and freeze source; nativepatch losslessmachinepreimages+result with actualbefore/afterhashes,diffhunks, exactgofmt-equality proof and fullthisgrant. No gofmt-w/shellpersistentfiles, compiler/test/gates/Git/status ornormativeedits. No cap/fixture/assertions changes. Root independent critic checks scope and will rerun canonicalrace. Ifanything beyondformat stop/escalate.

## Read inputs and frozen identities

- Original assignment: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-initial-lease-bookkeeping-assignment-oct08.md`, SHA-256 `2be0bcd6c5f7f043e482716095df32409b0147dac3f920162cd8dd76de659e03`.
- Implementation result: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-initial-lease-bookkeeping-implementation-result-oct08.md`, SHA-256 `38a1ee193299cf8be841234423ddb655ed51154f7eff4d9dff2d46a4852f18fc`.
- Independent source review: `.agents/evidence/casework-live-wiring/T09/resume-2026-09-30/run-observation-initial-lease-bookkeeping-implementation-independent-source-review-oct08.md`, SHA-256 `0e1c2222c941ea0cb2c9f365722ff5f61532d456a9037bdb1c26c6132780474f`.
- Manager preimage: `apps/godspeed-casework-go/internal/server/run_observation_manager.go`, SHA-256 `2d01953469e3d7eedc5688ceb68c9e2d64aca86590251dcd11f3a59afbb3a6cd`, 31,242 bytes.
- Test preimage: `apps/godspeed-casework-go/internal/server/run_observation_manager_test.go`, SHA-256 `08852d598e91264a81a745ac9ba08b31e7eeb5cdb60dd01324e578967f260f50`, 63,948 bytes.

## Reversible machine preimage and formatter witness

The two source preimages are losslessly reconstructible from each final file by replacing the recorded postimage line with its preimage line; every other byte is unchanged. This inverse is the exact `gofmt -d` output observed before editing, and the preimage hashes/byte counts above identify the complete inputs.

```json
{
  "formatter": "gofmt",
  "commands": [
    "gofmt -d apps/godspeed-casework-go/internal/server/run_observation_manager.go",
    "gofmt -d apps/godspeed-casework-go/internal/server/run_observation_manager_test.go"
  ],
  "preimages": [
    {
      "path": "apps/godspeed-casework-go/internal/server/run_observation_manager.go",
      "sha256": "2d01953469e3d7eedc5688ceb68c9e2d64aca86590251dcd11f3a59afbb3a6cd",
      "bytes": 31242,
      "preimage_line": "\t\tcaller: caller, asOfCursor: asOfCursor,",
      "postimage_line": "\t\tcaller:     caller, asOfCursor: asOfCursor,"
    },
    {
      "path": "apps/godspeed-casework-go/internal/server/run_observation_manager_test.go",
      "sha256": "08852d598e91264a81a745ac9ba08b31e7eeb5cdb60dd01324e578967f260f50",
      "bytes": 63948,
      "preimage_line": "\t\tFrames: []ports.RunTraceFrame{retainedTestFrame(\"later-event\", \"2026-10-06T12:00:00Z\")},",
      "postimage_line": "\t\tFrames:          []ports.RunTraceFrame{retainedTestFrame(\"later-event\", \"2026-10-06T12:00:00Z\")},"
    }
  ],
  "postimage_sha256": {
    "apps/godspeed-casework-go/internal/server/run_observation_manager.go": "76e0dcc20e87007248cec3bf5ffe9a56a997b75c4ab2d2fb960cbbbaf0978d52",
    "apps/godspeed-casework-go/internal/server/run_observation_manager_test.go": "6e3a3315e4f92e277ff302a6bc17c06cdbcb5b128d4c234922b9f2320ca249cb"
  }
}
```

Before the native patches, each `gofmt -d` invocation exited 1 and emitted only its one recorded alignment hunk. After the patches, the same two non-mutating commands each exited 0 with empty output. This establishes exact formatter equality for the current files; it does not claim compilation or tests.

## Result and limits

Only the two requested formatting lines changed: one alignment adjustment in the manager composite literal and one in the test snapshot literal. Their meanings, values, assertions, fixtures, caps, and control flow are unchanged. The postimage sizes are 31,246 and 63,957 bytes, with SHA-256 values recorded in the JSON witness above. The separate source files and other fixtures were not changed by this unit.

No `gofmt -w`, shell file write, compiler, test, gate, Git mutation, status edit, or normative edit was performed. The pre-existing Canonical03 failure remains root-owned evidence; this result records only the fresh formatter-equivalence check. Independent review and root's canonical/race verification remain pending. This is not a runtime GREEN or T09 completion claim.
