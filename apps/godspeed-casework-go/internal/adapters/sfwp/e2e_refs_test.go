// Shared E2E ladder constants (used by both the live tests and the golden codec tests).
package sfwp

const (
	sentryChainRef = "e2e-sentry-chain@0.1.0"
	signoffGateRef = "e2e-signoff-gate@0.1.0"
	policyRef      = "authority/active-policy.json"
)

// executionDispatchFinding is the durable signature of the T05 kernel finding:
// an episode that settles rejected with basis "episode_dispatch_error" was
// never executed at all. crates/sea-forge-server/src/case_dispatch.rs
// (execute_sandbox) sends every operation through the command sandbox, which
// refuses write_file operations (input_error); the CLI routes write_file
// through sea_forge_sandbox::materialize instead (cli/src/pipeline.rs:519).
// Until the dispatch routes WriteFile the same way, the E2E templates'
// governed writes cannot execute on a live cell, so no accepted episode - and
// therefore no captured artifact - can exist. The live tests assert this
// signature explicitly and skip (never silently pass) the assertions that
// depend on accepted executions.
const executionDispatchFinding = "episode_dispatch_error"

// executionFindingNote is the one-paragraph finding every skip site prints so
// the skip is a documented finding, never a convenience.
const executionFindingNote = `KERNEL FINDING (plan casework-live-wiring T05): sea-forge-server cannot execute ` +
	`write_file plan items. crates/sea-forge-server/src/case_dispatch.rs execute_sandbox routes every ` +
	`operation through sea_forge_runtime::execute -> the sandbox's execute, which accepts only ` +
	`execute_command (local.rs / jail.rs return input_error for write_file); the CLI routes write_file ` +
	`through sea_forge_sandbox::materialize (crates/sea-forge-cli/src/pipeline.rs:519) but the server ` +
	`does not. The durable signature is a settlement_event with basis ` +
	`["episode_dispatch_error","input_error"] and no CommandFinished/ArtifactCaptured trace. Fix: route ` +
	`Operation::WriteFile through sea_forge_sandbox::materialize in execute_sandbox, exactly as the CLI ` +
	`does. The affected assertions are skips that disappear once the fix lands.`

// sentryChainParams mirrors the valid parameter set the Rust live tests use for the chain
// template.
func sentryChainParams() map[string]string {
	return map[string]string{
		"dataset_name":  "orders-q3",
		"dataset_label": "t05",
		"max_rows":      "25",
		"out_dir":       "work",
	}
}
