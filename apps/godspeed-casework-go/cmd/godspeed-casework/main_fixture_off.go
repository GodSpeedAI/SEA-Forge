//go:build !casework_fixture

// The production build's fixture stub: the fixture stack does not exist here, so a serve
// configuration that would fall back to it is refused before any listener opens.
package main

import (
	"context"
	"errors"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/config"
)

// fixtureStackAvailable marks this binary as production: no fixture stack is compiled in.
const fixtureStackAvailable = false

// serveFixtureStack is never called in this build (main.go refuses before it would be reached);
// it exists so the untagged build links.
func serveFixtureStack(ctx context.Context, addr string, resolved config.Resolved) error {
	return errors.New("the fixture stack is not compiled into this build")
}
