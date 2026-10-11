//go:build casework_fixture

// The dev/demo fixture stack (build tag casework_fixture): serves the Northstar fixture
// projection over the fixture HTTP surface. Reachable only from `just casework-go-up` /
// `casework-demo-up`; the production build (no tags) compiles main_fixture_off.go instead and
// cannot select the fixture adapters at all.
package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/artifactstore"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/config"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/coordinator"
	"github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/projection"
	fixtureserver "github.com/GodSpeedAI/SEA-Forge/apps/godspeed-casework-go/internal/server"
)

// fixtureStackAvailable marks the tagged build: fixture capabilities may register in preflight.
const fixtureStackAvailable = true

// serveFixtureStack boots the FIXTURE-LABELED stack and blocks until SIGINT or SIGTERM.
func serveFixtureStack(ctx context.Context, addr string, resolved config.Resolved) error {
	dataset, err := projection.Fixture()
	if err != nil {
		return err
	}
	proj, err := projection.NewFixtureStore(dataset)
	if err != nil {
		return err
	}
	arts := artifactstore.New(dataset.Artifacts)
	coord := coordinator.New(proj, arts, coordinator.Options{})
	api := fixtureserver.NewFixtureServer(proj, coord, arts, fixtureserver.Options{})

	httpServer := &http.Server{
		Addr:              addr,
		Handler:           api.Handler(),
		ReadHeaderTimeout: 10 * time.Second,
	}

	serveErr := make(chan error, 1)
	go func() { serveErr <- httpServer.ListenAndServe() }()
	fmt.Printf("godspeed-casework: serving fixture cognitive projection on http://%s (FIXTURE-LABELED: %s)\n", addr, projection.ProvenanceLabel)

	signals := make(chan os.Signal, 1)
	signal.Notify(signals, syscall.SIGINT, syscall.SIGTERM)
	select {
	case err := <-serveErr:
		return err
	case sig := <-signals:
		fmt.Printf("godspeed-casework: %s received - shutting down\n", sig)
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return httpServer.Shutdown(shutdownCtx)
	}
}
