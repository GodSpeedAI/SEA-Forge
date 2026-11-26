# Current status

**Status revision:** 391

**Stage:** CognitiveOS #108 hardening review fixes

**Summary:** First-party Go casework client authenticates request.get_status with the original actor claim.

**Verified:**

- NewRequestGetStatus now takes Governance; recovery repeats the original request's actor and on_behalf_of; ports.RequestStatus carries governance; golden frames carry the actor.
- go vet (default and live tags) and go test ./... pass in apps/godspeed-casework-go, including a new test that an unauthenticated status is refused identity_required and an authenticated recovery succeeds.

**Limits:**

- The Workbench renderer reaches status only through the Tauri host command sfwp_request_status, whose host code is not in this tree; its actor claim is not verified here.
- Full CI, proofs, cross-platform tests and the complete server suite have not run.

**Next:** Finish the remaining hardening-review fixes, then run the full gates.

**Evidence:**

- apps/godspeed-casework-go/internal/adapters/sfwp/frame.go
- apps/godspeed-casework-go/internal/adapters/sfwp/client_test.go
- apps/godspeed-casework-go/internal/adapters/sfwp/testdata/frames.json
