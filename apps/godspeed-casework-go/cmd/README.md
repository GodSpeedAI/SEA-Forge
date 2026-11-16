# cmd

## Overview
Command entrypoints of the casework Go module. Currently a single command that owns process-level
concerns: configuration loading, preflight, preflight reporting, and the serve-mode assembly of the
live (or, build-tag gated, fixture) gateway.

## Subdirectories & Boundaries
- [./godspeed-casework/README.md](./godspeed-casework/README.md) - the casework front end's entrypoint (`main`, exit-code contract, live/fixture build gate)
