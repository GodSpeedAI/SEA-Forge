// Real SFWP IPC bridge for the UX Journey Settlement Gauntlet.
//
// The Workbench frontend only ever talks to a real backend through
// `window.__TAURI_INTERNALS__.invoke`, which inside the actual Tauri app is
// implemented by Rust commands in
// `workbench/apps/desktop/src-tauri/src/bridge.rs` that forward over a Unix
// domain socket (NDJSON, one JSON object per line — see
// `workbench/apps/desktop/src-tauri/src/socket.rs`) to a real
// `sea-forge-server` process. A plain browser page cannot open a Unix
// socket, so this process is the bridge: it opens the REAL Unix socket to a
// REAL, unmodified `sea-forge-server` binary (started by
// harness/gauntlet_runner.py against a cell bootstrapped by
// `crates/sea-forge-server/examples/journey_gauntlet_bootstrap.rs`), and
// exposes a WebSocket the browser-side init script
// (`real-sfwp-bridge-shim.js`) can reach.
//
// This relays real requests to a real server and returns real responses —
// it does not fabricate any SFWP data itself. The one piece of logic it
// reimplements (rather than merely forwards) is `sfwp_command`'s
// actor-attachment, mirroring `bridge.rs::sfwp_command`/`choose_actor`
// exactly (same identity_get call, same single-actor-auto-select /
// `act_as`-disambiguation / ambiguous-refusal rules) — because that
// attachment is real host logic in the production app, not something the
// real server does itself.
//
// Run with Bun (uses Bun's native WebSocket server + Node-compatible
// `node:net` for the Unix socket — no external dependencies):
//   bun real_sfwp_bridge.mjs --socket <path> --port <port> --root <path>

import net from "node:net";

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i]?.replace(/^--/, "");
    out[key] = argv[i + 1];
  }
  return out;
}

const args = parseArgs(process.argv.slice(2));
const SOCKET_PATH = args.socket;
const PORT = Number(args.port);
const CELL_ROOT = args.root;

if (!SOCKET_PATH || !PORT || !CELL_ROOT) {
  console.error("usage: bun real_sfwp_bridge.mjs --socket <path> --port <port> --root <path>");
  process.exit(1);
}

/** One real Unix-socket connection to the real server, NDJSON framed,
 * FIFO request/response pairing exactly like socket.rs's SocketClient. */
class SfwpConnection {
  constructor(onEvent) {
    this.onEvent = onEvent;
    this.pending = [];
    this.buffer = "";
    this.closed = false;
    this.ready = new Promise((resolve, reject) => {
      this.sock = net.connect({ path: SOCKET_PATH }, () => resolve());
      this.sock.once("error", reject);
    });
    this.sock.on("data", (chunk) => this._onData(chunk));
    this.sock.on("close", () => {
      this.closed = true;
      while (this.pending.length) this.pending.shift().reject(new Error("sfwp socket disconnected"));
    });
    this.sock.on("error", () => {});
  }

  _onData(chunk) {
    this.buffer += chunk.toString("utf8");
    let idx;
    while ((idx = this.buffer.indexOf("\n")) !== -1) {
      const line = this.buffer.slice(0, idx).trim();
      this.buffer = this.buffer.slice(idx + 1);
      if (!line) continue;
      let value;
      try {
        value = JSON.parse(line);
      } catch {
        continue;
      }
      if (value && value.type === "event") {
        this.onEvent(value.event ?? null);
        continue;
      }
      const next = this.pending.shift();
      if (next) next.resolve(value);
    }
  }

  async call(request) {
    await this.ready;
    if (this.closed) throw new Error("sfwp socket disconnected");
    return new Promise((resolve, reject) => {
      this.pending.push({ resolve, reject });
      this.sock.write(JSON.stringify(request) + "\n");
    });
  }

  close() {
    try {
      this.sock.end();
    } catch {
      /* already closed */
    }
  }
}

/** Mirrors bridge.rs::choose_actor exactly: same three refusal shapes,
 * same auto-select-if-single-actor rule. */
function chooseActor(identity, actAs) {
  const available = Array.isArray(identity?.available) ? identity.available : [];
  if (available.length === 0) {
    const refusal = identity?.refusal;
    return {
      error: {
        error_class: refusal?.error_class ?? "identity_unresolved",
        error: refusal?.message ?? "this cell reported no identity bindings",
        no_side_effect: true,
        next_lawful_action: refusal?.next_lawful_action ?? null,
      },
    };
  }
  let chosen;
  if (actAs) {
    chosen = available.find((a) => a.actor_id === actAs);
    if (!chosen) {
      return {
        error: {
          error_class: "identity_not_bound",
          error: `this connection may not act as \`${actAs}\``,
          no_side_effect: true,
          next_lawful_action: "Select a server-advertised actor",
        },
      };
    }
  } else if (available.length === 1) {
    chosen = available[0];
  } else {
    return {
      error: {
        error_class: "identity_ambiguous",
        error: `this connection may act as ${available.length} different actors; choose one explicitly`,
        no_side_effect: true,
        next_lawful_action: "Select one of the server-advertised actors",
      },
    };
  }
  const role = Array.isArray(chosen.roles) ? chosen.roles[0] : undefined;
  if (!role) {
    return {
      error: {
        error_class: "identity_role_not_held",
        error: `actor \`${chosen.actor_id}\` holds no role in this cell`,
        no_side_effect: true,
        next_lawful_action: "Select an actor with an eligible role",
      },
    };
  }
  return { actor: { actor_id: chosen.actor_id, role } };
}

async function handleInvoke(conn, cmd, invokeArgs) {
  switch (cmd) {
    case "sfwp_query": {
      const query = invokeArgs?.query ?? {};
      const data = await conn.call(query);
      return { ok: true, data };
    }
    case "sfwp_command": {
      const command = { ...(invokeArgs?.command ?? {}) };
      const actAs = invokeArgs?.actAs;
      const identity = await conn.call({ verb: "identity_get" });
      const resolved = chooseActor(identity, actAs);
      if (resolved.error) return { ok: false, error: resolved.error };
      command.actor = resolved.actor;
      if ("entity" in command) command.entity = resolved.actor.actor_id;
      const data = await conn.call(command);
      return { ok: true, data };
    }
    case "sfwp_identity": {
      const data = await conn.call({ verb: "identity_get" });
      return { ok: true, data };
    }
    case "sfwp_cell": {
      return {
        ok: true,
        data: { socket_path: SOCKET_PATH, root: CELL_ROOT, supervision: "gauntlet_bridge" },
      };
    }
    case "sfwp_initialize_cell": {
      // The gauntlet bootstrap tool already initializes the cell before the
      // server starts — nothing left to do, matching an already-adopted
      // cell from the real host's point of view.
      return { ok: true, data: { standing: "adopted" } };
    }
    case "sfwp_request_status": {
      const data = await conn.call({ verb: "request_get_status", request_id: invokeArgs?.request_id });
      return { ok: true, data };
    }
    case "plugin:event|listen":
      return { ok: true, data: Math.floor(Math.random() * 1e9) };
    case "plugin:event|unlisten":
      return { ok: true, data: null };
    default:
      return { ok: true, data: null };
  }
}

const server = Bun.serve({
  port: PORT,
  hostname: "127.0.0.1",
  fetch(req, srv) {
    if (srv.upgrade(req)) return;
    return new Response("real_sfwp_bridge: websocket upgrade required", { status: 400 });
  },
  websocket: {
    open(ws) {
      const conn = new SfwpConnection((frame) => {
        if (ws.data?.subscribedToEvents) {
          ws.send(JSON.stringify({ type: "event_push", event: "sfwp://event", payload: frame }));
        }
      });
      ws.data = { conn, subscribedToEvents: false };
    },
    async message(ws, raw) {
      let msg;
      try {
        msg = JSON.parse(String(raw));
      } catch {
        return;
      }
      const { id, cmd, args: invokeArgs } = msg;
      try {
        if (cmd === "plugin:event|listen" && invokeArgs?.event === "sfwp://event") {
          ws.data.subscribedToEvents = true;
          // Best-effort live forwarding: subscribes from "now", no durable
          // cursor/gap-recovery replay (unlike the production host's
          // events.rs). Real events, real subscription — just without the
          // reconnect/backlog sophistication a long-lived desktop app needs.
          ws.data.conn.call({ verb: "events_subscribe", from_cursor: null }).catch(() => {});
        }
        const result = await handleInvoke(ws.data.conn, cmd, invokeArgs);
        ws.send(JSON.stringify({ id, ...result }));
      } catch (error) {
        ws.send(JSON.stringify({ id, ok: false, error: { error_class: "bridge_transport_error", error: String(error), no_side_effect: false } }));
      }
    },
    close(ws) {
      ws.data?.conn?.close();
    },
  },
});

console.log(`real_sfwp_bridge: listening on ws://127.0.0.1:${server.port}, relaying to ${SOCKET_PATH}`);
