// LocalUserStore: browser users from configuration, argon2id-verified. It serves BOTH the local
// mode (passwords verified) and the dev mode (verification skipped, optional static bearer
// token) - the dev posture is a construction flag, and the config layer refuses dev mode in the
// production posture, so the flag can only be reached on a dev-configured gateway.
package auth

import (
	"context"
	"fmt"
	"sort"
	"sync"
)

// User is one configured local user.
type User struct {
	Username     string
	DisplayName  string
	PasswordHash string // argon2id PHC string; may be empty only in dev mode
	ActorID      string // kernel actor
	Role         string // kernel role
}

// LocalUserStore verifies username/password pairs against configured users.
type LocalUserStore struct {
	mode string
	// devMode skips password verification (any non-empty password logs in). It exists ONLY for
	// auth.mode=dev, which the production posture refuses at configuration validation.
	devMode bool

	mu    sync.RWMutex
	users map[string]User // normalized username -> user
	order []string        // configured order, for the static-token default identity
}

// NewLocalUserStore builds a store over the configured users. In local mode every user must
// carry a parsable argon2id hash; in dev mode hashes are optional and unverified.
func NewLocalUserStore(mode string, users []User) (*LocalUserStore, error) {
	if mode != ModeLocal && mode != ModeDev {
		return nil, fmt.Errorf("local user store supports modes %q and %q, got %q", ModeLocal, ModeDev, mode)
	}
	dev := mode == ModeDev
	s := &LocalUserStore{mode: mode, devMode: dev, users: map[string]User{}}
	for _, u := range users {
		name := normalizeUsername(u.Username)
		if name == "" {
			return nil, fmt.Errorf("a configured user has an empty username")
		}
		if _, dup := s.users[name]; dup {
			return nil, fmt.Errorf("duplicate configured user %q", u.Username)
		}
		if u.ActorID == "" || u.Role == "" {
			return nil, fmt.Errorf("configured user %q has no kernel standing (actor_id/role)", u.Username)
		}
		if !dev {
			if err := VerifyPassword(u.PasswordHash, ""); err != nil && err != ErrInvalidCredentials {
				return nil, fmt.Errorf("configured user %q: %w", u.Username, err)
			}
		}
		s.users[name] = u
		s.order = append(s.order, name)
	}
	if len(s.users) == 0 {
		return nil, fmt.Errorf("the local user store requires at least one configured user")
	}
	return s, nil
}

// Mode implements Authenticator.
func (s *LocalUserStore) Mode() string { return s.mode }

// Login implements PasswordLogin. Unknown users and wrong passwords are indistinguishable
// (ErrInvalidCredentials); an unknown user still pays one argon2id verification against a dummy
// hash so login timing does not enumerate valid usernames.
func (s *LocalUserStore) Login(_ context.Context, username, password string) (Identity, error) {
	name := normalizeUsername(username)
	s.mu.RLock()
	user, ok := s.users[name]
	s.mu.RUnlock()

	if !ok {
		// Timing-equalizing verification against a fixed dummy hash.
		_ = VerifyPassword(dummyHash, "timing-equalizer")
		return Identity{}, ErrInvalidCredentials
	}
	if !s.devMode {
		if password == "" || VerifyPassword(user.PasswordHash, password) != nil {
			return Identity{}, ErrInvalidCredentials
		}
	} else if password == "" {
		// Even the dev posture does not accept an empty password: a client that forgot the
		// password field entirely must not look authenticated.
		return Identity{}, ErrInvalidCredentials
	}
	return Identity{
		Username:    user.Username,
		DisplayName: user.DisplayName,
		ActorID:     user.ActorID,
		Role:        user.Role,
	}, nil
}

// StaticTokenIdentity resolves the identity a valid dev-mode bearer token acts as: the named
// user, or the first configured user when none was named. It is only reachable in dev mode.
func (s *LocalUserStore) StaticTokenIdentity(staticTokenUser string) (Identity, error) {
	if !s.devMode {
		return Identity{}, fmt.Errorf("static token identities only exist in dev mode")
	}
	name := normalizeUsername(staticTokenUser)
	if name == "" {
		s.mu.RLock()
		if len(s.order) == 0 {
			s.mu.RUnlock()
			return Identity{}, ErrNoActorMapping
		}
		name = s.order[0]
		user := s.users[name]
		s.mu.RUnlock()
		return Identity{Username: user.Username, DisplayName: user.DisplayName, ActorID: user.ActorID, Role: user.Role}, nil
	}
	s.mu.RLock()
	user, ok := s.users[name]
	s.mu.RUnlock()
	if !ok {
		return Identity{}, fmt.Errorf("static_token_user %q is not a configured user", staticTokenUser)
	}
	return Identity{Username: user.Username, DisplayName: user.DisplayName, ActorID: user.ActorID, Role: user.Role}, nil
}

// Usernames lists the configured usernames in configuration order (test/diagnostics helper).
func (s *LocalUserStore) Usernames() []string {
	s.mu.RLock()
	defer s.mu.RUnlock()
	out := append([]string(nil), s.order...)
	sort.Strings(out)
	return out
}

// dummyHash is a valid argon2id hash of an unguessable constant, used only to equalize login
// timing for unknown usernames (its preimage is irrelevant; it is never compared to a real
// credential). Derived at init with the package's own parameters.
var dummyHash = mustDummyHash()

func mustDummyHash() string {
	h, err := HashPassword("auth-timing-equalizer-no-real-password")
	if err != nil {
		panic("auth: cannot build dummy hash: " + err.Error())
	}
	return h
}
