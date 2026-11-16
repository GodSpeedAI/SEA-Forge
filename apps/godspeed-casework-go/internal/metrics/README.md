# metrics

## Overview
The gateway's tiny, dependency-free Prometheus text exposition. It exposes exactly the operational
signals the plan names and nothing that can identify a user or a case: request latency by route
PATTERN and status class, SFWP errors by class and verb, open SSE clients, and subscription lag.
Every label value comes from a closed set (route patterns, error kinds, wire verbs), so cardinality
is bounded. All `Registry` methods are safe on a nil receiver, so call sites need no feature checks.

## Interface
- `Registry` — the metric registry: `New()`, `ObserveRequest(route, status, d)`, `ObserveSFWPError(verb, class)`, `SetBehindSource(f)`, `Write(w)`, `Handler()`, `ServeMux()` [metrics.go:Registry]
- `SSEClient` — per-client SSE gauges: `SSEOpen(startCursor, pending)`, `Delivered(cursor, recordedAt)`, `Close()` [metrics.go:SSEClient]
- `ValidateListenAddr(addr) error` — loopback-only enforcement for the metrics listener [metrics.go:ValidateListenAddr]
- `latencyBuckets` — fixed Prometheus histogram buckets [metrics.go:latencyBuckets]

## Invariants
- Label values come from closed sets (route patterns, error classes, verbs); cardinality is bounded [metrics.go:package doc]
- No series identifies a user or a case [metrics.go:package doc]
- Nil-receiver safety: every `Registry` method tolerates a nil registry [metrics.go:package doc]
- The metrics listener address must be loopback [metrics.go:ValidateListenAddr]

## Side Effects
- Serves Prometheus text on the dedicated metrics mux/HTTP handler (`GET /metrics`) [metrics.go:Handler]
- Tracks in-process counters/histograms and SSE gauge state in memory [metrics.go:histogram]

## Verification
- Run: `go test ./...` (from apps/godspeed-casework-go/)
