// T07 config tests: the fail-closed auth matrix. The load-bearing permanent assertion is tooth
// (b): a production posture with auth.mode=dev must be a FATAL configuration problem - the
// process refuses to start.
package config

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// baseDoc returns a minimal valid serve-posture document the individual tests mutate.
func baseDoc() Document {
	return Document{
		Version:  SupportedVersion,
		CellRoot: "cell",
		Serve:    &ServeSection{},
		Auth: &AuthSection{
			Mode: AuthModeLocal,
			Users: []AuthUser{{
				Username:     "operator",
				PasswordHash: "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$m38nzkEDz+xpkLVQWNlbA4wRb3tcwSzLMMWcTv7QN8o",
				ActorID:      "operator_local",
				Role:         "operator",
			}},
		},
	}
}

func problemMessages(problems []*apperr.Error) []string {
	out := make([]string, 0, len(problems))
	for _, p := range problems {
		out = append(out, p.Message)
	}
	return out
}

func assertFatal(t *testing.T, doc Document, wantFragment string) {
	t.Helper()
	problems := Validate(doc)
	fatal := Fatal(problems)
	if len(fatal) == 0 {
		t.Fatalf("expected a fatal problem matching %q; problems: %v", wantFragment, problemMessages(problems))
	}
	found := false
	for _, p := range fatal {
		if strings.Contains(p.Message, wantFragment) {
			found = true
		}
	}
	if !found {
		t.Fatalf("expected a fatal problem matching %q; fatal problems: %v", wantFragment, problemMessages(fatal))
	}
}

func assertNoProblems(t *testing.T, doc Document) {
	t.Helper()
	if problems := Validate(doc); len(problems) != 0 {
		t.Fatalf("expected no problems; got: %v", problemMessages(problems))
	}
}

func TestValidateServePostureRequiresExplicitAuthMode(t *testing.T) {
	doc := baseDoc()
	doc.Auth = nil
	assertFatal(t, doc, "serve posture requires an explicit auth.mode")
}

func TestValidateProductionRefusesDevAuth_T07ToothB(t *testing.T) {
	doc := baseDoc()
	doc.Serve.Production = true
	doc.Auth.Mode = AuthModeDev
	// A dev-mode user without a hash is fine for dev; the REFUSAL must come from the posture.
	doc.Auth.Users[0].PasswordHash = ""
	assertFatal(t, doc, "auth.mode dev is REFUSED in the production posture")
}

func TestValidateProductionWithAbsentAuthDefaultsToDevAndRefuses(t *testing.T) {
	// Fail closed: a production config that FORGOT the auth section entirely cannot start (the
	// absent mode falls back to dev, which production refuses).
	doc := baseDoc()
	doc.Serve.Production = true
	doc.Auth = nil
	assertFatal(t, doc, "serve posture requires an explicit auth.mode")
}

func TestValidateProductionRefusesInsecureCookie(t *testing.T) {
	doc := baseDoc()
	doc.Serve.Production = true
	doc.Auth.InsecureCookie = true
	assertFatal(t, doc, "auth.insecure_cookie is REFUSED in the production posture")
}

func TestValidateStaticTokenOnlyInDevAndOnlyAsIndirection(t *testing.T) {
	doc := baseDoc()
	doc.Auth.StaticToken = "env:GODSPEED_DEV_TOKEN"
	assertFatal(t, doc, "auth.static_token is only valid in auth.mode dev")

	doc = baseDoc()
	doc.Auth.Mode = AuthModeDev
	doc.Auth.Users[0].PasswordHash = ""
	doc.Auth.StaticToken = "env:GODSPEED_DEV_TOKEN"
	assertNoProblems(t, doc)

	doc = baseDoc()
	doc.Auth.Mode = AuthModeDev
	doc.Auth.Users[0].PasswordHash = ""
	doc.Auth.StaticToken = "super-secret-bare-value"
	assertFatal(t, doc, "auth.static_token must be an indirection")
}

func TestValidateLocalModeUserRequirements(t *testing.T) {
	doc := baseDoc()
	doc.Auth.Users[0].PasswordHash = ""
	assertFatal(t, doc, "has no password_hash")

	doc = baseDoc()
	doc.Auth.Users[0].PasswordHash = "$2b$12$saltandsaltandsaltandsaluhashhashhashhashhashhashha"
	assertFatal(t, doc, "not an argon2id PHC string")

	doc = baseDoc()
	doc.Auth.Users[0].ActorID = ""
	assertFatal(t, doc, "no kernel standing")

	doc = baseDoc()
	doc.Auth.Users = append(doc.Auth.Users, AuthUser{Username: "operator", ActorID: "x", Role: "operator",
		PasswordHash: "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$m38nzkEDz+xpkLVQWNlbA4wRb3tcwSzLMMWcTv7QN8o"})
	assertFatal(t, doc, "duplicate auth user")
}

func TestValidateOIDCModeRequirements(t *testing.T) {
	doc := baseDoc()
	doc.Auth.Mode = AuthModeOIDC
	doc.Auth.OIDC = nil
	assertFatal(t, doc, "requires the auth.oidc section")

	doc.Auth.OIDC = &OIDCSection{
		Issuer:       "https://idp.example",
		ClientID:     "casework",
		ClientSecret: "https://placeholder.invalid",
		RedirectURL:  "https://casework.example/api/auth/callback",
		Mappings:     []OIDCMappingRule{{Claim: "groups", Equals: "operators", ActorID: "operator_local", Role: "operator"}},
	}
	assertFatal(t, doc, "client_secret must be an indirection")

	doc.Auth.OIDC.ClientSecret = "env:GODSPEED_OIDC_SECRET"
	assertNoProblems(t, doc)

	doc.Auth.OIDC.Mappings = nil
	assertFatal(t, doc, "mappings is empty")
}

func TestValidateNonServePostureSkipsAuth(t *testing.T) {
	doc := baseDoc()
	doc.Serve = nil
	doc.Auth = nil
	assertNoProblems(t, doc)
}

func TestValidateUnknownAuthMode(t *testing.T) {
	doc := baseDoc()
	doc.Auth.Mode = "kerberos"
	assertFatal(t, doc, "unknown auth.mode")
}

func TestLoadResolvesAuthSecretsAndRefusesBareValues(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "cfg.json")
	doc := baseDoc()
	doc.Auth.Mode = AuthModeDev
	doc.Auth.Users[0].PasswordHash = ""
	doc.Auth.StaticToken = "env:GODSPEED_TEST_DEV_TOKEN"
	raw, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, raw, 0o600); err != nil {
		t.Fatal(err)
	}

	resolved, secrets, problems := Load(path, Options{Env: func(k string) (string, bool) {
		if k == "GODSPEED_TEST_DEV_TOKEN" {
			return "dev-token-value", true
		}
		return "", false
	}})
	if len(problems) != 0 {
		t.Fatalf("unexpected problems: %v", problemMessages(problems))
	}
	found := false
	for _, s := range secrets {
		if s.Capability == "auth.static_token" && s.Value == "dev-token-value" {
			found = true
		}
	}
	if !found {
		t.Fatalf("the static token secret was not resolved: %+v", secrets)
	}
	if resolved.Auth.Mode != AuthModeDev {
		t.Fatalf("auth section lost in load: %+v", resolved.Auth)
	}

	// A missing indirection target is a fatal document-level problem.
	_, _, problems = Load(path, Options{Env: func(string) (string, bool) { return "", false }})
	fatal := Fatal(problems)
	if len(fatal) == 0 || !strings.Contains(fatal[0].Message, "GODSPEED_TEST_DEV_TOKEN") {
		t.Fatalf("a missing static token must be fatal naming the variable, got: %v", problemMessages(problems))
	}
}
