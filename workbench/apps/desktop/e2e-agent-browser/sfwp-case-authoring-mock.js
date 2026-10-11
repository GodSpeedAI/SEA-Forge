// Mocked-IPC shim for the agent-browser case-authoring proof journeys.
//
// This is the agent-browser counterpart of `e2e/tauriMock.ts` (Playwright):
// it installs `window.__TAURI_INTERNALS__` so the real renderer code —
// `@tauri-apps/api` invoke, TanStack Query hooks, the XState authoring
// machine, generated AJV validators — runs unmodified in a plain Chromium
// page. It is NOT real-stack coverage: no Tauri host, no sea-forge-server.
//
// Unlike the Playwright shim (installed via addInitScript before app boot),
// this one is eval'd into an already-loaded page and the driver reaches the
// authoring route through SPA navigation, which preserves the page document.
//
// Scenario state (`window.__sfwpMock.mode`) is mutable from the driver:
//   stale_once — commit #1 returns `rejected_as_stale`; preflight re-runs pin
//                a fresh digest (A then B); commit #2 succeeds.
//   drop_once  — commit #1's transport drops (the promise rejects: no reply
//                at all); `request.get_status` later reports the outcome.
//
// Every `case_commit`/`case_preflight` call is logged with the precondition
// digests actually carried on the wire, so the driver can prove the stale
// repair re-pinned the digest and the ambiguous journey never duplicated the
// mutation.
(() => {
  const TEMPLATE_REF = "hello_agents@0.1.0";
  const DIGEST_A = "sha256:" + "17a4c9".repeat(11).slice(0, 64);
  const DIGEST_B = "sha256:" + "9e2f63".repeat(11).slice(0, 64);
  const CASE_ID = "case_01mockcase";

  const state = {
    mode: "stale_once",
    preflightCalls: 0,
    commitCalls: 0,
    statusCalls: 0,
    commits: [], // { request_id, preconditions } per case_commit, in order
    preflightDigests: [], // digest each preflight handed out, in order
    // Scenario branches key on calls made *in the current mode*, so the
    // driver starts each journey with reset("<mode>") and every assertion
    // reads a clean per-journey log.
    reset(mode) {
      state.mode = mode;
      state.preflightCalls = 0;
      state.commitCalls = 0;
      state.statusCalls = 0;
      state.commits = [];
      state.preflightDigests = [];
    },
  };
  state.reset("stale_once");
  window.__sfwpMock = state;

  // Same fixture family as e2e/tauriMock.ts — shapes that pass the generated
  // AJV validators (EntryOptionsResult, PreflightResult, IdentityView,
  // ReadinessView, CaseListResult).
  const ENTRY_OPTIONS = {
    templates: [
      {
        template_ref: TEMPLATE_REF,
        description: "Say hello through one governed sandboxed task",
        parameters: {
          greeting_name: { param_type: "string", required: true, default: null },
        },
      },
    ],
  };

  const IDENTITY = {
    available: [{ actor_id: "operator_a", roles: ["operator"] }],
    configured: true,
  };

  const CELL = {
    socket_path: "/tmp/sea-forge-mock.sock",
    root: "/home/operator/.sea-forge-cells/mock-cell",
  };

  const READINESS = {
    overall: "ready",
    foundations: [
      {
        id: "self_model_integrity",
        name: "Self-model integrity",
        category: "foundation",
        status: "ready",
        reason: "",
        next_lawful_action: "",
        source_ref: "sea-forge-self-model/src/store.rs::validate",
      },
    ],
    operational_capabilities: [
      {
        id: "local_governed_execution",
        name: "Local governed execution",
        category: "operational_capability",
        status: "ready",
        reason: "",
        next_lawful_action: "",
        source_ref: "sea-forge-server::case_dispatch",
      },
    ],
    recent_invalidations: [],
  };

  const CASE_LIST = { cases: [], unreadable: [] };

  const preflightResult = (digest) => ({
    ok: true,
    template_ref: TEMPLATE_REF,
    errors: [],
    items: [{ plan_item_id: "pi_hello_1", name: "Greet", item_kind: "SandboxedTask" }],
    precondition: { ref: `template:${TEMPLATE_REF}`, expected_digest: digest },
  });

  const commitSuccess = () => ({ case_id: CASE_ID, state: "active", exit_code: 0 });

  function handlePreflight() {
    state.preflightCalls += 1;
    // A stale precondition must be repairable: after the stale rejection the
    // template "changed on disk", so the next preflight pins a different
    // digest. Everything else keeps returning the same digest.
    const digest = state.mode === "stale_once" && state.preflightCalls > 1 ? DIGEST_B : DIGEST_A;
    state.preflightDigests.push(digest);
    return Promise.resolve(preflightResult(digest));
  }

  function handleCommit(command) {
    state.commitCalls += 1;
    state.commits.push({
      request_id: command?.request_id ?? null,
      preconditions: command?.preconditions ?? null,
    });
    if (state.mode === "stale_once" && state.commitCalls === 1) {
      // Refusal that certifies nothing was mutated — same shape the server's
      // stale-precondition path returns (Rust conformance
      // `case_commit_stale_precondition_rejects_with_no_case_created`).
      return Promise.resolve({
        outcome: "rejected_as_stale",
        code: "precondition_failed",
        changed_records: [{ record_ref: `template:${TEMPLATE_REF}`, change: "content_updated" }],
        next_actions: ["case.preflight"],
      });
    }
    if (state.mode === "drop_once" && state.commitCalls === 1) {
      // A dropped commit response: the invoke rejects, the client cannot know
      // whether the case was created. This is the exact condition the
      // `ambiguous` state exists for.
      return Promise.reject(new Error("mock transport: connection closed before response"));
    }
    return Promise.resolve(commitSuccess());
  }

  function handleStatus() {
    state.statusCalls += 1;
    // The server-side work landed after the dropped reply; recovery reads the
    // recorded outcome instead of resubmitting.
    return Promise.resolve({ status: "completed", outcome: commitSuccess() });
  }

  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      if (cmd === "sfwp_query") {
        const verb = args?.query?.verb;
        if (verb === "readiness_get") return Promise.resolve(READINESS);
        if (verb === "case_entry_options") return Promise.resolve(ENTRY_OPTIONS);
        if (verb === "case_preflight") return handlePreflight();
        if (verb === "case_list") return Promise.resolve(CASE_LIST);
        return Promise.resolve(null);
      }
      if (cmd === "sfwp_command") {
        if (args?.command?.verb === "case_commit") return handleCommit(args.command);
        return Promise.resolve(null);
      }
      if (cmd === "sfwp_request_status") return handleStatus();
      if (cmd === "sfwp_identity") return Promise.resolve(IDENTITY);
      if (cmd === "sfwp_cell") return Promise.resolve(CELL);
      if (cmd === "plugin:event|listen") return Promise.resolve(1);
      if (cmd === "plugin:event|unlisten") return Promise.resolve(undefined);
      return Promise.resolve(null);
    },
    transformCallback: (cb) => {
      const id = Math.floor(Math.random() * 1e9);
      window[`_${id}`] = cb;
      return id;
    },    unregisterCallback: () => {},
    unregisterListener: () => {},
    convertFileSrc: (p) => p,
  };
  // `@tauri-apps/api/event` owns listener cleanup in a separate namespace.
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };

  return `sfwp mock installed: mode=${state.mode} (mutate via window.__sfwpMock.mode)`;
})();
