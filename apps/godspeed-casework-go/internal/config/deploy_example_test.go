package config

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

const productionExample = "../../../../deploy/systemd/gateway.production.json.example"

// The shipped production example must stay loadable under the production posture; and flipping it
// to the dev auth surface must be refused (the guard the config reference documents).
func TestDeployProductionExampleLoadsAndDevAuthIsRefused(t *testing.T) {
	env := func(k string) (string, bool) {
		switch k {
		case "OIDC_CLIENT_SECRET":
			return "not-a-real-secret", true
		case "GODSPEED_CELL_ROOT":
			return "/var/lib/sea-forge/cell", true
		}
		return "", false
	}
	_, _, problems := Load(productionExample, Options{Env: env})
	if fatal := Fatal(problems); len(fatal) > 0 {
		t.Fatalf("shipped production example has fatal problems: %v", fatal)
	}

	raw, err := os.ReadFile(productionExample)
	if err != nil {
		t.Fatal(err)
	}
	dev := strings.Replace(string(raw), `"mode": "oidc"`, `"mode": "dev"`, 1)
	if dev == string(raw) {
		t.Fatal("example no longer carries auth.mode oidc; update this test")
	}
	path := filepath.Join(t.TempDir(), "dev-in-prod.json")
	if err := os.WriteFile(path, []byte(dev), 0o600); err != nil {
		t.Fatal(err)
	}
	_, _, problems = Load(path, Options{Env: env})
	found := false
	for _, p := range Fatal(problems) {
		if strings.Contains(p.Error(), "REFUSED in the production posture") {
			found = true
		}
	}
	if !found {
		t.Fatalf("auth.mode dev with serve.production=true must be a fatal refusal, got %v", problems)
	}
}
