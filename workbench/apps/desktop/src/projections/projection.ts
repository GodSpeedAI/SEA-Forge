// Projection layer: authoritative system/domain state → spatial UI.
// Direction of authority (one way, downstream only):
//
//   SEA-Forge/domain state → system front end → frontend ports/adapters
//   → semantic projections (here) → spatial UI → visual projections
//   → CoreRenderer
//
// The renderer never becomes authoritative. Interaction generates intents
// through existing ports; those travel through the authoritative system.
// No projection fabricates domain data: absent sources render as unknown,
// never as zero/empty claims.

export interface ProjectionStatus {
  state: "live" | "loading" | "unavailable";
  detail?: string;
}
