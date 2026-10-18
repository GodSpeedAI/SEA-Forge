# Production MCP integration — 2026-10-17

Seam: `delegation::run_acp_episode` (`isolated` class) offers ACP agents
`mcpServers` entries that point only at `sea-forge-mcp-connect <socket>`.
The socket is served by `mcp::serve_workload` in the trusted server process
(launcher UID), which checks `SO_PEERCRED` against the workload's subordinate
UID, runs the existing broker (authority `mcp_tool_call`, canonical parsing,
exact request grant), and spawns the upstream outside the workload with the
operator-held environment. `ServerConfig.mcp_servers` is empty by default;
non-isolated classes with MCP servers configured fail closed
(`mcp_requires_isolated_workload`).

Observed (captured run: 600 passed, 0 failed, 2 ignored across
server/agent/sandbox):

- `conformance_m16::t16_4c`: an isolated fixture agent lists tools (only
  `metadata.read`), calls `metadata.read` (result), calls `audit.delete`
  (-32001) via the connector and via the raw socket; running the upstream
  directly against the trusted observer directory fails. The upstream's own
  `invocations.jsonl` holds only the two `metadata.read` calls; `audit.json`
  is unchanged.
- `mcp_workload_broker`: a foreign-UID peer gets no response, no upstream
  process starts, and `mcp_peer_refused` is recorded. Reconnection is a
  fresh session (tools refused until `initialize`), each session has its own
  upstream, `resources/list` is refused locally, and dropping the broker
  ends the open session and its upstream process.

Mutation controls (manual, restored): removing the broker's DENY branch fails
`t16_4c`; removing the peer-UID check fails the foreign-peer test.

Not covered: MCP transports other than stdio-over-broker (HTTP/SSE offered by
agents' own configs are outside SEA Forge; upstream credentials are not
reachable from the workload), streaming/progress notifications, server-side
requests to the client, and cancellation of an in-flight upstream call by
the client (`notifications/cancelled` is dropped by the broker, never forwarded).
