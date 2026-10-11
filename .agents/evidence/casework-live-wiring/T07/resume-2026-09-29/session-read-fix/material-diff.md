# Material diff from original specification

- `projection.Revision` now carries an internal, JSON-omitted pointer to the captured `CaseFacts`; the retention count and cursor ordering remain unchanged.
- `projection.Store` deep-copies both snapshots and facts on append and read. Facts copying covers nested stage, settlement-basis, horizon dependency/run, approval, and run slices.
- `Relay.accept` uses `Facts` directly when the source supports it and calls `Build` on that exact capture. Other injected sources retain their prior `Snapshot` path and have no captured facts.
- `LiveSource.Facts` records its capture timestamp even when no test clock is injected, so re-rendering historical facts preserves the original projection time.
- `/api/world` refuses actor/role query keys before cursor access, verifies the session actor before current or retained reads, enforces supplied case-to-cursor binding, and rebuilds retained snapshots from captured historical facts for that actor.
- `/api/events` refuses identity override query keys and verifies the actor before subscription. Each retained/live revision is rebuilt for the session; a source without captured facts is served only when its stored perspective exactly matches, otherwise the stream closes without sending the mismatched revision.
- Corrected the adjacent `Options.Perspective` and `PerspectiveVerifier` comments to describe captured-fact re-rendering and session-verified reads; the comments no longer claim SSE retains the configured actor or that callers may request explicit query perspectives.
- No dependencies, persisted schemas, kernel policy, identity model, or wire-contract fields changed.
