# Apps Blueprint

## Overview
Standalone applications that sit on top of the SEA Forge kernel without being part of its Rust workspace: the Go casework gateway and the GodSpeed cognitive environment UI. Both are clients of the governed kernel surface; neither is the semantic authority.

## Subdirectories & Boundaries
- [godspeed-casework-go](./godspeed-casework-go/README.md) - Go casework gateway (go 1.27) coordinating governed casework through application-owned ports over the kernel SFWP socket, serving the cognitive projection over HTTP+SSE.
- [godspeed-cognitive-ui](./godspeed-cognitive-ui/README.md) - Persistent 2.5D world UI over the Go casework service: one contract port, two sources (live gateway and local fixture), nine lazy artifact renderers, agent-browser E2E ladders.

## Global Invariants
- Applications are never the semantic authority; they coordinate through kernel-owned ports (see each child contract).

## Global Side Effects
- Both children serve or render over HTTP+SSE and persist application state; specifics are declared in the child contracts.

## Verification
- Run: `go test ./...` (from `apps/godspeed-casework-go/`); `bun test` (from `apps/godspeed-cognitive-ui/`).
