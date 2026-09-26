// Package config is the application's ONE configuration authority.
//
// REQ-CONFIG-001: there is one documented precedence order and no adapter may introduce a competing
// one. REQ-CONFIG-002: secrets are resolved through indirection (`env:NAME`, `file:/path`) and never
// appear in the loaded configuration. REQ-CONFIG-003: endpoints, provider selections, ports and
// environment paths stay configurable. REQ-CONFIG-004: the schema and its defaults are declared once,
// here, and validated before activation.
package config

import (
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strings"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/apperr"
)

// SupportedVersion is the only schema major version this application accepts.
const SupportedVersion = "1"

// Kinds of capability the application knows how to wire. A capability whose kind is unknown is a
// configuration error, not something an adapter may decide on its own.
const (
	KindAuthority  = "authority"
	KindExecution  = "execution"
	KindRepository = "repository"
	KindArtifact   = "artifact"
)

// Adapter selections. The empty string selects the historical in-process provider; AdapterFixture
// pins the test/dev fixture explicitly; AdapterLive selects the governed-kernel client. The
// production build refuses to select the fixture adapters (plan guardrail), which is why the live
// selection is named here rather than left to a free-form string.
const (
	AdapterEmpty   = ""
	AdapterFixture = "fixture"
	AdapterLive    = "sfwp"
)

// knownAdapters is the closed set of adapter selections a capability may name.
var knownAdapters = map[string]bool{
	AdapterEmpty:   true,
	AdapterFixture: true,
	AdapterLive:    true,
}

// Capability is one configured integration point.
type Capability struct {
	Name       string `json:"name"`
	Kind       string `json:"kind"`
	Required   bool   `json:"required"`
	Endpoint   string `json:"endpoint,omitempty"`
	Credential string `json:"credential,omitempty"`
	// Adapter selects which wired implementation serves this capability. Empty means the historical
	// in-process provider (which in serve mode is the fixture). Additive (T05): files without it
	// keep their meaning, and the schema major stays "1".
	Adapter string `json:"adapter,omitempty"`
}

// ServeSection holds the live-serve posture (T06, additive): the gateway principal claim the
// kernel's server.yaml must mirror, the authority policy the governed verbs authorize against,
// and the perspective relay-built revisions are rendered for. Every default is explicit below;
// nothing is invented from the environment. T07 adds the production posture, static UI serving
// and the intent rate limits.
type ServeSection struct {
	GatewayActorID string `json:"gateway_actor_id,omitempty"`
	GatewayRole    string `json:"gateway_role,omitempty"`
	PolicyRef      string `json:"policy_ref,omitempty"`
	// Perspective* is who relay-built revision snapshots speak for. Unset means the gateway
	// principal itself (the honest default: the gateway's own kernel view). Authenticated
	// sessions (T07) give /api/world its own per-user perspective; the relay's revision stream
	// keeps this configured perspective (documented: SSE snapshots render the gateway's view,
	// role-filtered action enforcement happens at intent time).
	PerspectiveActorID string `json:"perspective_actor_id,omitempty"`
	PerspectiveRole    string `json:"perspective_role,omitempty"`
	// Production is the production posture (T07): cookies must carry Secure, the dev auth surface
	// is refused, and the listener is expected to sit behind a TLS-terminating trusted proxy.
	Production bool `json:"production,omitempty"`
	// StaticRoot is the path of the built UI (apps/godspeed-cognitive-ui/dist) the gateway serves
	// with cache headers + CSP + SPA fallback. Empty disables static serving (the UI is then
	// served by a separate host, e.g. a CDN or the vite dev server).
	StaticRoot string `json:"static_root,omitempty"`
	// RateLimit bounds POST /api/intents (token bucket per session and per remote IP). Nil/zero
	// fields take the documented defaults (60/minute, burst 20).
	RateLimit *RateLimitSection `json:"rate_limit,omitempty"`
}

// Document is the on-disk configuration schema.
type Document struct {
	Version      string        `json:"version"`
	CellRoot     string        `json:"cell_root"`
	EvidenceRoot string        `json:"evidence_root"`
	Capabilities []Capability  `json:"capabilities"`
	Serve        *ServeSection `json:"serve,omitempty"`
	// Auth is the browser-authentication section (T07). It is validated (and required) only in a
	// serve posture: preflight and the fixture stack do not authenticate, and their configs do
	// not carry a serve section.
	Auth *AuthSection `json:"auth,omitempty"`
}

// ServeDefaults returns the ServeSection with every unset field filled from the schema's
// documented defaults: the kernel's own default gateway spelling ("gateway" at the service role),
// the E2E cell's authority-policy location, the gateway-principal perspective, and the intent
// rate-limit defaults.
func ServeDefaults(section *ServeSection) ServeSection {
	out := ServeSection{}
	if section != nil {
		out = *section
	}
	if out.GatewayActorID == "" {
		out.GatewayActorID = "gateway"
	}
	if out.GatewayRole == "" {
		out.GatewayRole = "service"
	}
	if out.PolicyRef == "" {
		out.PolicyRef = "authority/active-policy.json"
	}
	if out.PerspectiveActorID == "" {
		out.PerspectiveActorID = out.GatewayActorID
		out.PerspectiveRole = out.GatewayRole
	}
	return out
}

// RateLimitOrDefaults returns the serve section's rate limits with the documented defaults
// applied. Callers never read ServeSection.RateLimit directly.
func (s ServeSection) RateLimitOrDefaults() RateLimitSection {
	return rateLimitDefaults(s.RateLimit)
}

// Options controls loading. Env and Overrides are injected so tests never depend on the process
// environment, and so the precedence order is a property of this package rather than of the caller.
type Options struct {
	// Env looks up an environment variable. Nil means os.LookupEnv.
	Env func(string) (string, bool)
	// Overrides are explicit operator overrides, the highest-precedence source.
	Overrides map[string]string
}

// Resolved is the validated runtime configuration. Secrets are deliberately NOT part of it: they are
// returned separately by Load so a caller cannot accidentally serialise them with the configuration.
type Resolved struct {
	Document
	// SecretNames lists, per capability, the *name* of the indirection that resolved, never a value.
	SecretNames map[string]string
}

// Secret is a resolved credential value, held apart from Resolved.
type Secret struct {
	Capability string
	Value      string
}

// Defaults returns the schema's defaults. Documented precedence is:
//
//	defaults < config file < environment < explicit overrides
func Defaults() Document {
	return Document{
		Version:      SupportedVersion,
		CellRoot:     "",
		EvidenceRoot: "",
		Capabilities: nil,
	}
}

func lookup(o Options) func(string) (string, bool) {
	if o.Env != nil {
		return o.Env
	}
	return os.LookupEnv
}

// envKey maps a capability field to its environment override, so the mapping is declared once.
func envKey(field, capability string) string {
	if capability == "" {
		switch field {
		case "cell_root":
			return "GODSPEED_CELL_ROOT"
		case "evidence_root":
			return "GODSPEED_EVIDENCE_ROOT"
		}
		return "GODSPEED_" + strings.ToUpper(field)
	}
	return "GODSPEED_CAPABILITY_" + strings.ToUpper(capability) + "_" + strings.ToUpper(field)
}

// Load reads, merges, resolves and validates configuration. It returns the merged document, the
// resolved secrets, and EVERY problem it found.
//
// It deliberately does not stop at the first fault. REQ-CONFIG-012 requires that a preflight failure
// disable only the capability whose dependency is unavailable, so a capability-scoped problem must
// reach preflight and become that capability's state - not abort the whole application before the
// other capabilities have been evaluated. Use Fatal to select the document-level problems that do
// prevent the application from starting at all.
func Load(path string, opts Options) (Resolved, []Secret, []*apperr.Error) {
	doc := Defaults()
	var problems []*apperr.Error

	if path != "" {
		raw, err := os.ReadFile(path)
		if err != nil {
			return Resolved{}, nil, []*apperr.Error{
				apperr.Wrap(apperr.KindConfig, "", "load", "cannot read configuration file", err)}
		}
		var fileDoc Document
		dec := json.NewDecoder(strings.NewReader(string(raw)))
		dec.DisallowUnknownFields() // an unknown key is a typo, not something to silently ignore
		if err := dec.Decode(&fileDoc); err != nil {
			return Resolved{}, nil, []*apperr.Error{
				apperr.Wrap(apperr.KindConfig, "", "load", "configuration is not valid JSON for the declared schema", err)}
		}
		doc = merge(doc, fileDoc)
	}

	doc = applyEnvironment(doc, lookup(opts))
	doc = applyOverrides(doc, opts.Overrides)
	problems = append(problems, Validate(doc)...)

	resolved := Resolved{Document: doc, SecretNames: map[string]string{}}
	var secrets []Secret
	for _, c := range doc.Capabilities {
		resolved.SecretNames[c.Name] = c.Credential
		if c.Credential == "" {
			continue
		}
		value, err := resolveSecret(c.Name, c.Credential, lookup(opts))
		if err != nil {
			problems = append(problems, err)
			continue
		}
		secrets = append(secrets, Secret{Capability: c.Name, Value: value})
	}
	// The auth section's own indirections (dev static token, OIDC client secret) resolve through
	// the same machinery; their problems are document-level and therefore fatal (T07: a serve
	// posture whose authentication secrets cannot be resolved must not start).
	authSecrets, authProblems := resolveAuthSecrets(doc, lookup(opts))
	problems = append(problems, authProblems...)
	secrets = append(secrets, authSecrets...)
	return resolved, secrets, problems
}

// Fatal selects the problems that prevent the application from starting at all: document-level faults
// (version, cell_root, unreadable or unparsable file) carry no capability attribution, whereas a
// capability-scoped fault is expressed as that capability's state by preflight.
func Fatal(problems []*apperr.Error) []*apperr.Error {
	var fatal []*apperr.Error
	for _, p := range problems {
		if p != nil && p.Capability == "" {
			fatal = append(fatal, p)
		}
	}
	return fatal
}

// merge applies file values over defaults. Zero-valued file fields do not erase defaults.
func merge(base, over Document) Document {
	out := base
	if over.Version != "" {
		out.Version = over.Version
	}
	if over.CellRoot != "" {
		out.CellRoot = over.CellRoot
	}
	if over.EvidenceRoot != "" {
		out.EvidenceRoot = over.EvidenceRoot
	}
	if over.Capabilities != nil {
		out.Capabilities = over.Capabilities
	}
	if over.Serve != nil {
		out.Serve = over.Serve
	}
	if over.Auth != nil {
		out.Auth = over.Auth
	}
	return out
}

func applyEnvironment(doc Document, env func(string) (string, bool)) Document {
	if v, ok := env(envKey("cell_root", "")); ok {
		doc.CellRoot = v
	}
	if v, ok := env(envKey("evidence_root", "")); ok {
		doc.EvidenceRoot = v
	}
	for i := range doc.Capabilities {
		c := &doc.Capabilities[i]
		if v, ok := env(envKey("adapter", c.Name)); ok {
			c.Adapter = v
		}
		if v, ok := env(envKey("endpoint", c.Name)); ok {
			c.Endpoint = v
		}
		if v, ok := env(envKey("credential", c.Name)); ok {
			c.Credential = v
		}
		if v, ok := env(envKey("required", c.Name)); ok {
			c.Required = v == "true" || v == "1"
		}
		// The live adapter's socket follows the kernel's own env posture
		// (crates/sea-forge-server resolves its socket from SEA_FORGE_SOCKET):
		// a capability that selects it may omit the endpoint and inherit the
		// same environment variable. An EXPLICIT endpoint in the configuration
		// file wins over the ambient environment (pinned by the T05 tests: the
		// deployment's declared choice must not be overridden by whatever the
		// ambient kernel socket happens to be). Anything still empty fails
		// validation, so the fallback stays fail-closed.
		if c.Endpoint == "" && c.Adapter == AdapterLive {
			if v, ok := env(liveSocketEnv); ok {
				c.Endpoint = v
			}
		}
	}
	return doc
}

// liveSocketEnv is the environment variable the governed kernel itself reads
// its Unix socket path from; the sfwp adapter selection inherits it.
const liveSocketEnv = "SEA_FORGE_SOCKET"

func applyOverrides(doc Document, overrides map[string]string) Document {
	for k, v := range overrides {
		switch {
		case k == "cell_root":
			doc.CellRoot = v
		case k == "evidence_root":
			doc.EvidenceRoot = v
		case strings.HasPrefix(k, "capability."):
			rest := strings.TrimPrefix(k, "capability.")
			name, field, ok := strings.Cut(rest, ".")
			if !ok {
				continue
			}
			for i := range doc.Capabilities {
				if doc.Capabilities[i].Name != name {
					continue
				}
				switch field {
				case "endpoint":
					doc.Capabilities[i].Endpoint = v
				case "credential":
					doc.Capabilities[i].Credential = v
				case "required":
					doc.Capabilities[i].Required = v == "true" || v == "1"
				case "adapter":
					doc.Capabilities[i].Adapter = v
				}
			}
		}
	}
	return doc
}

// secretSchemes are the only accepted credential spellings. A bare value is rejected on purpose:
// REQ-CONFIG-002 forbids embedding secrets in stable configuration.
func resolveSecret(capability, ref string, env func(string) (string, bool)) (string, *apperr.Error) {
	scheme, target, ok := strings.Cut(ref, ":")
	if !ok {
		return "", apperr.New(apperr.KindConfig, capability, "secret",
			"credential must be an indirection (env:NAME or file:/path); a bare value is not accepted")
	}
	switch scheme {
	case "env":
		value, found := env(target)
		if !found {
			return "", apperr.New(apperr.KindConfig, capability, "secret",
				"credential indirection env:"+target+" is not set")
		}
		return value, nil
	case "file":
		raw, err := os.ReadFile(target)
		if err != nil {
			return "", apperr.Wrap(apperr.KindConfig, capability, "secret",
				"credential indirection file:"+target+" is unreadable", err)
		}
		return strings.TrimSpace(string(raw)), nil
	default:
		return "", apperr.New(apperr.KindConfig, capability, "secret",
			"unsupported credential indirection scheme "+scheme+":")
	}
}

// Validate reports every configuration problem, attributed to a capability where applicable, so the
// caller can decide blast radius rather than aborting on the first fault.
func Validate(doc Document) []*apperr.Error {
	var problems []*apperr.Error
	if doc.Version != SupportedVersion {
		problems = append(problems, apperr.New(apperr.KindConfig, "", "validate",
			fmt.Sprintf("unsupported configuration version %q (want %q)", doc.Version, SupportedVersion)))
	}
	if strings.TrimSpace(doc.CellRoot) == "" {
		problems = append(problems, apperr.New(apperr.KindConfig, "", "validate", "cell_root is required"))
	}
	// The serve posture's auth + production matrix (T07). A configuration with no serve section
	// does not authenticate and skips these checks entirely.
	if doc.Serve != nil {
		problems = append(problems, validateServe(doc)...)
		problems = append(problems, validateAuth(doc, ServeDefaults(doc.Serve))...)
	}
	seen := map[string]bool{}
	for _, c := range doc.Capabilities {
		if strings.TrimSpace(c.Name) == "" {
			problems = append(problems, apperr.New(apperr.KindConfig, "", "validate", "capability without a name"))
			continue
		}
		if seen[c.Name] {
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate", "duplicate capability name"))
		}
		seen[c.Name] = true
		switch c.Kind {
		case KindAuthority, KindExecution, KindRepository, KindArtifact:
		default:
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate",
				"unknown capability kind "+c.Kind))
		}
		if !knownAdapters[c.Adapter] {
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate",
				"unknown adapter selection "+c.Adapter+" (known: fixture, sfwp)"))
		}
		// Plan guardrail: the production build must not be able to select the fixture adapters.
		// The dev/demo fixture stack builds with -tags casework_fixture (just casework-go-up),
		// which is the only place this selection is legal.
		if c.Adapter == AdapterFixture && !fixtureBuildEnabled {
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate",
				"the fixture adapter selection is refused by this build (production binaries cannot select fixtures; the dev fixture stack builds with -tags casework_fixture)"))
		}
		if c.Adapter == AdapterLive && strings.TrimSpace(c.Endpoint) == "" {
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate",
				"live adapter selection has no endpoint (the governed kernel's socket path)"))
		}
		if c.Required && strings.TrimSpace(c.Endpoint) == "" {
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate",
				"required capability has no endpoint"))
		}
		if c.Required && c.Kind != KindArtifact && strings.TrimSpace(c.Credential) == "" {
			problems = append(problems, apperr.New(apperr.KindConfig, c.Name, "validate",
				"required consequential capability has no credential indirection"))
		}
	}
	sort.SliceStable(problems, func(i, j int) bool {
		return problems[i].Capability < problems[j].Capability
	})
	return problems
}
