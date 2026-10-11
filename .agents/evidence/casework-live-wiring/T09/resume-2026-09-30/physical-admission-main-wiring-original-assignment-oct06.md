# Physical admission — original CLI wiring assignment

Luna builder original scope: edit ONLY
`apps/godspeed-casework-go/cmd/godspeed-casework/main.go` after reading the complete
target and applicable instructions. Instantiate EXACTLY ONE `sfwp.NewRunGetLimiter()`
immediately before liveClients/ALL-capabilities loop. Inject that same owner through
`RunGetAdmission` in the existing sole production `sfwp.Config` literal, including
live clients not selected for serve. No extra constructor, startup behavior,
dependencies, interfaces, tests or refactoring. Independent runtime builder owns
client.go/run_get_admission.go; preserve those and frozen fixtures. Native apply_patch
only. No compiling/tests/scanner/Graft build/Git/docs/status/ledger writes.
Return exact diff, hash, anchors and all material deviations.

Follow-up before freezing: apply gofmt-equivalent spacing ONLY to the new Config
literal using native patch. Read-only `gofmt -d` may determine and verify spacing;
no `gofmt -w` or shell writes. Return final hash/diff. Independent critic receives
these ORIGINAL instructions plus actual complete implementation and must cite
every production construction site and both run-trace/provenance run_get paths.
