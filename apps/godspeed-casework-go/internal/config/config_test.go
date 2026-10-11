package config

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

func writeFile(t *testing.T, body string) string {
	t.Helper()
	path := filepath.Join(t.TempDir(), "config.json")
	if err := os.WriteFile(path, []byte(body), 0o600); err != nil {
		t.Fatalf("write fixture: %v", err)
	}
	return path
}

// REQ-CONFIG-001: one documented precedence order: defaults < file < environment < overrides.
func TestPrecedenceDefaultsFileEnvironmentOverride(t *testing.T) {
	path := writeFile(t, `{
		"version": "1",
		"cell_root": "/cell/from-file",
		"capabilities": [
			{"name": "authority", "kind": "authority", "required": true,
			 "endpoint": "unix:///cell/server.sock", "credential": "env:AUTHORITY_TOKEN"}
		]
	}`)
	env := func(k string) (string, bool) {
		switch k {
		case "GODSPEED_CELL_ROOT":
			return "/cell/from-env", true
		case "AUTHORITY_TOKEN":
			return "tok", true
		}
		return "", false
	}

	resolved, _, problems := Load(path, Options{Env: env})
	if fatal := Fatal(problems); len(fatal) > 0 {
		t.Fatalf("unexpected fatal problems: %v", fatal)
	}
	if resolved.CellRoot != "/cell/from-env" {
		t.Fatalf("env must beat file: got %q", resolved.CellRoot)
	}

	resolved, _, problems = Load(path, Options{Env: env, Overrides: map[string]string{"cell_root": "/cell/from-override"}})
	if fatal := Fatal(problems); len(fatal) > 0 {
		t.Fatalf("unexpected fatal problems: %v", fatal)
	}
	if resolved.CellRoot != "/cell/from-override" {
		t.Fatalf("override must beat env: got %q", resolved.CellRoot)
	}
}

// REQ-CONFIG-002: secrets are resolved through indirection and never appear in the configuration.
func TestCredentialsMustBeIndirectAndStaySeparate(t *testing.T) {
	path := writeFile(t, `{
		"version": "1", "cell_root": "/cell",
		"capabilities": [
			{"name": "authority", "kind": "authority", "required": true,
			 "endpoint": "unix:///cell/server.sock", "credential": "env:AUTHORITY_TOKEN"}
		]
	}`)
	resolved, secrets, problems := Load(path, Options{Env: func(k string) (string, bool) {
		if k == "AUTHORITY_TOKEN" {
			return "s3cret-value", true
		}
		return "", false
	}})
	if len(problems) > 0 {
		t.Fatalf("unexpected problems: %v", problems)
	}
	if len(secrets) != 1 || secrets[0].Value != "s3cret-value" || secrets[0].Capability != "authority" {
		t.Fatalf("secret not resolved separately: %#v", secrets)
	}
	if resolved.SecretNames["authority"] != "env:AUTHORITY_TOKEN" {
		t.Fatalf("resolved config should hold the indirection *name*: %#v", resolved.SecretNames)
	}
	// The resolved document must not carry the value anywhere.
	if got := resolved.Capabilities[0].Credential; got != "env:AUTHORITY_TOKEN" {
		t.Fatalf("configuration should keep the indirection, not the value: %q", got)
	}
}

func TestBareCredentialIsRejected(t *testing.T) {
	path := writeFile(t, `{
		"version": "1", "cell_root": "/cell",
		"capabilities": [
			{"name": "authority", "kind": "authority", "required": true,
			 "endpoint": "unix:///cell/server.sock", "credential": "s3cret-value"}
		]
	}`)
	_, _, problems := Load(path, Options{Env: func(string) (string, bool) { return "", false }})
	if len(problems) == 0 {
		t.Fatal("a bare credential must be rejected: REQ-CONFIG-002 forbids embedding secrets")
	}
	if got := apperr.KindOf(problems[0]); got != apperr.KindConfig {
		t.Fatalf("want typed config error, got %q (%v)", got, problems[0])
	}
	if got := apperr.CapabilityOf(problems[0]); got != "authority" {
		t.Fatalf("the problem must be attributed to the capability, got %q", got)
	}
	if len(Fatal(problems)) != 0 {
		t.Fatalf("a capability-scoped fault must not be fatal: %v", Fatal(problems))
	}
}

func TestUnknownCredentialSchemeIsRejected(t *testing.T) {
	path := writeFile(t, `{
		"version": "1", "cell_root": "/cell",
		"capabilities": [
			{"name": "authority", "kind": "authority", "required": true,
			 "endpoint": "unix:///cell/server.sock", "credential": "vault:secret/authority"}
		]
	}`)
	_, _, problems := Load(path, Options{Env: func(string) (string, bool) { return "", false }})
	if len(problems) == 0 || apperr.KindOf(problems[0]) != apperr.KindConfig {
		t.Fatalf("unsupported scheme must be a typed config problem, got %v", problems)
	}
}

func TestMissingEnvCredentialIsRejectedAndAttributed(t *testing.T) {
	path := writeFile(t, `{
		"version": "1", "cell_root": "/cell",
		"capabilities": [
			{"name": "repository", "kind": "repository", "required": true,
			 "endpoint": "https://api.example.invalid", "credential": "env:REPO_TOKEN"}
		]
	}`)
	_, _, problems := Load(path, Options{Env: func(string) (string, bool) { return "", false }})
	if len(problems) == 0 {
		t.Fatal("a missing credential indirection must be reported before activation")
	}
	if apperr.KindOf(problems[0]) != apperr.KindConfig || apperr.CapabilityOf(problems[0]) != "repository" {
		t.Fatalf("want a typed config problem attributed to repository, got %v", problems)
	}
}

// REQ-CONFIG-004: validation happens before activation and reports problems per capability.
func TestValidateCollectsProblemsPerCapability(t *testing.T) {
	doc := Document{
		Version:  SupportedVersion,
		CellRoot: "/cell",
		Capabilities: []Capability{
			{Name: "authority", Kind: KindAuthority, Required: true},
			{Name: "authority", Kind: "nonsense", Required: false},
			{Name: "", Kind: KindArtifact},
		},
	}
	problems := Validate(doc)
	if len(problems) < 3 {
		t.Fatalf("want several attributed problems, got %d: %v", len(problems), problems)
	}
	seen := map[string]bool{}
	for _, p := range problems {
		seen[p.Capability+"|"+string(p.Kind)] = true
	}
	if !seen["authority|config"] {
		t.Fatalf("the required capability without an endpoint must be reported: %v", problems)
	}
}

func TestUnsupportedVersionIsRejected(t *testing.T) {
	path := writeFile(t, `{"version": "2", "cell_root": "/cell"}`)
	_, _, problems := Load(path, Options{Env: func(string) (string, bool) { return "", false }})
	fatal := Fatal(problems)
	if len(fatal) == 0 || apperr.KindOf(fatal[0]) != apperr.KindConfig {
		t.Fatalf("a document-level fault must be fatal and typed, got %v", problems)
	}
}

func TestUnknownKeyIsRejected(t *testing.T) {
	path := writeFile(t, `{"version": "1", "cell_root": "/cell", "surprise": true}`)
	_, _, problems := Load(path, Options{Env: func(string) (string, bool) { return "", false }})
	if len(Fatal(problems)) == 0 {
		t.Fatal("an unknown configuration key is a typo, not something to ignore silently")
	}
}

// The sfwp adapter selection is selectable via config, env, or overrides, and
// its socket may be inherited from SEA_FORGE_SOCKET - the same env the
// governed kernel itself reads (plan T05).
func TestLiveAdapterSelectionAndSocketFallback(t *testing.T) {
	capabilityJSON := func(endpoint string) string {
		if endpoint == "" {
			return `{"name": "authority", "kind": "authority", "required": false, "adapter": "sfwp"}`
		}
		return `{"name": "authority", "kind": "authority", "required": false, "adapter": "sfwp", "endpoint": "` + endpoint + `"}`
	}
	load := func(endpoint string, env func(string) (string, bool)) (Capability, []*apperr.Error) {
		path := writeFile(t, `{
			"version": "1", "cell_root": "/cell",
			"capabilities": [`+capabilityJSON(endpoint)+`]
		}`)
		resolved, _, problems := Load(path, Options{Env: env})
		if len(resolved.Capabilities) != 1 {
			t.Fatalf("want one capability, got %#v", resolved.Capabilities)
		}
		return resolved.Capabilities[0], problems
	}
	noEnv := func(string) (string, bool) { return "", false }

	// A live selection with an explicit endpoint validates.
	cap, problems := load("/cell/kernel.sock", noEnv)
	if len(problems) != 0 {
		t.Fatalf("a live selection with an endpoint must validate: %v", problems)
	}
	if cap.Adapter != AdapterLive || cap.Endpoint != "/cell/kernel.sock" {
		t.Fatalf("selection not carried: %+v", cap)
	}

	// Adapter selection via environment, socket inherited from SEA_FORGE_SOCKET.
	socketEnv := func(k string) (string, bool) {
		if k == "GODSPEED_CAPABILITY_AUTHORITY_ADAPTER" {
			return AdapterLive, true
		}
		if k == "SEA_FORGE_SOCKET" {
			return "/run/sea-forge/server.sock", true
		}
		return "", false
	}
	cap, problems = load("", socketEnv)
	if len(problems) != 0 {
		t.Fatalf("live selection via env with SEA_FORGE_SOCKET must validate: %v", problems)
	}
	if cap.Adapter != AdapterLive {
		t.Fatalf("env adapter selection not applied: %+v", cap)
	}
	if cap.Endpoint != "/run/sea-forge/server.sock" {
		t.Fatalf("the socket must be inherited from SEA_FORGE_SOCKET, got %q", cap.Endpoint)
	}

	// An explicit endpoint wins over the SEA_FORGE_SOCKET inheritance.
	cap, problems = load("/cell/kernel.sock", socketEnv)
	if len(problems) != 0 {
		t.Fatalf("unexpected problems: %v", problems)
	}
	if cap.Endpoint != "/cell/kernel.sock" {
		t.Fatalf("the explicit endpoint must win over SEA_FORGE_SOCKET, got %q", cap.Endpoint)
	}

	// No endpoint anywhere: validation refuses the live selection (fail-closed).
	_, problems = load("", noEnv)
	if len(problems) == 0 || apperr.CapabilityOf(problems[0]) != "authority" {
		t.Fatalf("a live selection with no socket must be a capability problem: %v", problems)
	}
}
