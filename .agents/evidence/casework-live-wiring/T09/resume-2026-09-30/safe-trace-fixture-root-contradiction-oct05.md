# Root source finding: contradictory blank-expected-identity fixtures

The fresh fixture `a3c13aba` cannot satisfy the binding v3 specification as
currently written. Its identity matrix loops over empty, whitespace and foreign
expected case/run/item IDs (`run_trace_test.go:276-289`) and routes every case
through `assertRunTraceCase` expecting typed unavailable and one wire request.
The separate `TestReadRunTraceV3RejectsBlankExpectedIdentityBeforeDial` instead
requires typed invalid before any dial for those same empty/whitespace expected
identities. Binding v3 and the original root assignment require the latter.

The helper's wire assertion also makes a correctly rejected pre-dial blank input
fail the former matrix. These are mutually incompatible assertions, not a
production defect or evidence of an executed test failure. No Go compiler/test
was run for this finding.

Retain response-identity missing/blank/mismatched tests as unavailable; those
use valid expected identities. Retain foreign nonblank expected identity network
tests as unavailable. Remove empty/whitespace expected inputs from the network
matrix while preserving comprehensive invalid-before-dial coverage for each of
the three expected fields in the dedicated test. A different fresh builder and
independent re-review are required before intended assertion RED.

The original independent source rejection did not identify this contradiction;
its existing records remain immutable. Root has notified the independent reviewer
to verify this additional blocker against the actual source and original envelope.
