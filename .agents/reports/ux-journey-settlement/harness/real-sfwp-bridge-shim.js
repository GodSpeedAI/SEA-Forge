// Real-backend browser shim for the UX Journey Settlement Gauntlet.
//
// Registered as an agent-browser `--init-script` (runs before first paint,
// same mechanism the old mock used — see git history / README for why that
// matters: installing after first paint lets the app's first queries fire
// against a nonexistent bridge and fail before a later install can help).
//
// Unlike the old `sfwp-full-mock.js`, this file fabricates NOTHING. It
// implements `window.__TAURI_INTERNALS__.invoke` by forwarding every call
// over a WebSocket to `harness/real_sfwp_bridge.mjs`, which relays it to a
// real, unmodified `sea-forge-server` process over the real Unix socket
// protocol. Every response the app renders is a real server response.
//
// Requires `window.__SFWP_BRIDGE_PORT__` to already be set (the harness
// prepends that assignment before this file's contents when it writes the
// combined init-script — see gauntlet_runner.py's `_write_init_script`).
(() => {
  const port = window.__SFWP_BRIDGE_PORT__;
  if (!port) {
    throw new Error("real-sfwp-bridge-shim.js: window.__SFWP_BRIDGE_PORT__ was not set before this script ran");
  }

  const state = {
    nextId: 1,
    pending: new Map(), // id -> {resolve, reject}
    eventHandlers: new Map(), // event name -> callback id
    ws: null,
    wsReady: null,
    // Real dispatch transcript — read by the harness (BrowserDriver
    // .bridge_transcript()) to verify which verbs a UI action actually
    // caused to fire, over the real bridge, to the real server.
    queryLog: [],
    commandLog: [],
  };
  window.__sfwpRealBridge = state;

  function connect() {
    const ws = new WebSocket(`ws://127.0.0.1:${port}`);
    state.ws = ws;
    state.wsReady = new Promise((resolve, reject) => {
      ws.addEventListener("open", () => resolve());
      ws.addEventListener("error", (e) => reject(e));
    });
    ws.addEventListener("message", (evt) => {
      let msg;
      try {
        msg = JSON.parse(evt.data);
      } catch {
        return;
      }
      if (msg.type === "event_push") {
        const callbackId = state.eventHandlers.get(msg.event);
        if (callbackId != null && typeof window[`_${callbackId}`] === "function") {
          window[`_${callbackId}`]({ event: msg.event, id: 0, payload: msg.payload });
        }
        return;
      }
      const pending = state.pending.get(msg.id);
      if (!pending) return;
      state.pending.delete(msg.id);
      if (msg.ok) {
        pending.resolve(msg.data);
      } else {
        const err = new Error(msg.error?.error || "sfwp bridge command failed");
        Object.assign(err, msg.error);
        pending.reject(err);
      }
    });
  }
  connect();

  async function callBridge(cmd, args) {
    await state.wsReady;
    const id = state.nextId++;
    return new Promise((resolve, reject) => {
      state.pending.set(id, { resolve, reject });
      state.ws.send(JSON.stringify({ id, cmd, args }));
    });
  }

  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd, args) => {
      if (cmd === "plugin:event|listen") {
        // Tauri v2's real `listen()` shape; field name checked defensively
        // since forwarded events are best-effort here (see
        // real_sfwp_bridge.mjs's comment on gap-recovery scope) — the core
        // query/command path above does not depend on this.
        const eventName = args?.event ?? args?.eventName;
        const handlerId = args?.handler ?? args?.handlerId;
        if (eventName && handlerId != null) {
          state.eventHandlers.set(eventName, handlerId);
        }
      }
      // Log verb + real response so the harness can pull exact values (a
      // digest, an id) out of what the UI itself actually triggered,
      // instead of re-issuing a second, possibly-inconsistent call.
      if (cmd === "sfwp_query") {
        try {
          const response = await callBridge(cmd, args);
          state.queryLog.push({ verb: args?.query?.verb, request: args?.query, response, at: new Date().toISOString() });
          return response;
        } catch (error) {
          state.queryLog.push({ verb: args?.query?.verb, request: args?.query, error: String(error), at: new Date().toISOString() });
          throw error;
        }
      }
      if (cmd === "sfwp_command") {
        try {
          const response = await callBridge(cmd, args);
          state.commandLog.push({ verb: args?.command?.verb, request: args?.command, response, at: new Date().toISOString() });
          return response;
        } catch (error) {
          state.commandLog.push({ verb: args?.command?.verb, request: args?.command, error: String(error), at: new Date().toISOString() });
          throw error;
        }
      }
      return callBridge(cmd, args);
    },
    transformCallback: (cb) => {
      const id = Math.floor(Math.random() * 1e9);
      window[`_${id}`] = cb;
      return id;
    },
    unregisterCallback: () => {},
    unregisterListener: () => {},
    convertFileSrc: (p) => p,
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };

  return `real sfwp bridge shim installed: ws://127.0.0.1:${port}`;
})();
