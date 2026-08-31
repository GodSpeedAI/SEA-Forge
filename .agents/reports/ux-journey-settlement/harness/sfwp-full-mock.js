// SFWP IPC bridge mock for the SEA-Forge Journey Settlement Gauntlet.
//
// Grounded in the real generated AJV contracts at
// workbench/packages/contracts/schema/*.schema.json — every response shape
// below is validated against those schemas by
// harness/validate_mock_payloads.mjs before this file is trusted, so the
// Workbench renders its real query-result branches instead of an AJV
// validation-error fallback.
//
// This mocks the Tauri IPC boundary only. It drives the real React app and
// its real `@tanstack/react-query` + AJV-validated data flow, but it does
// NOT exercise the live sea-forge-server process, kernel enforcement, or any
// persisted governance ledger. Every call the app makes through this bridge
// is logged into `state.commandLog` / `state.queryLog` so the test harness
// can read back what the UI *actually* invoked, rather than asserting
// against literals nobody observed.
(() => {
  function sha256hex(input) {
    // Deterministic, dependency-free digest (FNV-1a x4, hex-folded) — good
    // enough to give the harness a stable, recomputable content hash without
    // pulling WebCrypto's async API into this synchronous mock installer.
    function fnv1a(str, seed) {
      let h = seed >>> 0;
      for (let i = 0; i < str.length; i++) {
        h ^= str.charCodeAt(i);
        h = Math.imul(h, 0x01000193);
      }
      return h >>> 0;
    }
    const parts = [0x811c9dc5, 0x9e3779b9, 0x85ebca6b, 0xc2b2ae35].map((seed) =>
      fnv1a(input, seed).toString(16).padStart(8, "0"),
    );
    return (parts.join("") + parts.join("")).slice(0, 64);
  }

  const TEMPLATE_REF = "hello_agents@0.1.0";
  const CASE_ID = "case_01mockcase";
  const RUN_ID = "run-101";
  const APPROVAL_ID = "appr-101";
  const nowIso = () => new Date().toISOString();

  const DIGEST_A = "sha256:" + sha256hex("harness-fixture:template:" + TEMPLATE_REF + ":v1");
  const DIGEST_B = "sha256:" + sha256hex("harness-fixture:template:" + TEMPLATE_REF + ":v2-stale");
  const CRITERIA_SHA = sha256hex("harness-fixture:criteria:" + APPROVAL_ID);
  const ARTIFACT_CONTENT = "hello, operator\n";
  const ARTIFACT_SHA = sha256hex(ARTIFACT_CONTENT);

  function sourceRecordRef(recordKind, recordId) {
    return {
      digest: "sha256:" + sha256hex(`harness-fixture:${recordKind}:${recordId}`),
      entry_id: `entry_${recordId}`,
      freshness: "current",
      ledger_id: "ledger_mock_cell",
      rebuild_standing: "verified",
      record_id: recordId,
      record_kind: recordKind,
    };
  }

  const state = {
    mode: "clean", // "clean", "stale_once", "drop_once"
    preflightCalls: 0,
    commitCalls: 0,
    statusCalls: 0,
    commits: [],
    preflightDigests: [],
    commandLog: [], // every sfwp_command actually invoked by the real UI
    queryLog: [], // every sfwp_query actually invoked by the real UI
    pendingApprovals: [
      {
        approval_id: APPROVAL_ID,
        case_id: CASE_ID,
        decision_id: "dec_appr_101",
        expired: false,
        expires_at: new Date(Date.now() + 3600_000).toISOString(),
        plan_item_id: "pi_hello_1",
        requested_at: nowIso(),
        run_id: RUN_ID,
        criteria_ref: `criteria:${APPROVAL_ID}`,
        criteria_sha256: CRITERIA_SHA,
        governance: {
          approval_source: sourceRecordRef("approval_request", APPROVAL_ID),
          decision_source: sourceRecordRef("authority_decision", "dec_appr_101"),
          eligibility_standing: "eligible_separation_of_duty_verified",
          eligible_actors: ["approver_alice"],
          operation_kind: "model_execution_grant",
          purpose_context: { case_id: CASE_ID, plan_item_id: "pi_hello_1" },
          reason: "External model execution requires human review before dispatch.",
          requester: "operator_a",
          resource_ref: "endpoints/local.yaml",
          side_effect_standing: "not_executed",
        },
      },
    ],
    approvalDecisions: [], // {approval_id, verdict, actor, decided_at, note}
    artifactContents: { "artifacts/output.txt": ARTIFACT_CONTENT },
    reset(mode = "clean") {
      state.mode = mode;
      state.preflightCalls = 0;
      state.commitCalls = 0;
      state.statusCalls = 0;
      state.commits = [];
      state.preflightDigests = [];
      state.commandLog = [];
      state.queryLog = [];
    },
  };
  window.__sfwpMock = state;

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
        source: sourceRecordRef("self_model_snapshot", "self_model_integrity"),
      },
      {
        id: "policy_integrity",
        name: "Policy integrity",
        category: "foundation",
        status: "ready",
        reason: "",
        next_lawful_action: "",
        source: sourceRecordRef("policy_snapshot", "policy_integrity"),
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
        source: sourceRecordRef("capability_snapshot", "local_governed_execution"),
      },
    ],
    recent_invalidations: [],
  };

  const IDENTITY = {
    available: [{ actor_id: "operator_a", roles: ["operator"] }],
    configured: true,
  };

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

  function preflightResult(digest) {
    return {
      ok: true,
      template_ref: TEMPLATE_REF,
      errors: [],
      items: [{ plan_item_id: "pi_hello_1", name: "Greet", item_kind: "sandboxed_task" }],
      precondition: { ref: `template:${TEMPLATE_REF}`, expected_digest: digest },
    };
  }

  function caseList() {
    return {
      cases: state.commits.length
        ? [
            {
              case_id: CASE_ID,
              case_state: "active",
              created_at: nowIso(),
              run_count: 1,
              summary: "Hello Agents Governed Workflow",
            },
          ]
        : [],
    };
  }

  function caseOverview() {
    return {
      case_id: CASE_ID,
      case_state: "active",
      created_at: nowIso(),
      item_count: 1,
      plan_ref: `template:${TEMPLATE_REF}`,
      run_ids: [RUN_ID],
      stages: ["stg_1"],
      summary: "Hello Agents Governed Workflow",
      template_ref: TEMPLATE_REF,
    };
  }

  function caseHorizon() {
    const decided = state.approvalDecisions.length > 0;
    return {
      case_id: CASE_ID,
      case_state: "active",
      events_folded: 2 + state.commitCalls + state.approvalDecisions.length,
      items: [
        {
          plan_item_id: "pi_hello_1",
          name: "Greet",
          item_kind: "sandboxed_task",
          execution: decided ? "completed" : "active",
          settlement: decided ? "accepted" : "unsettled",
          depends_on: [],
          run_ids: [RUN_ID],
          parent_stage: "stg_1",
        },
      ],
    };
  }

  function approvalList() {
    return { approvals: state.pendingApprovals.slice() };
  }

  function runList() {
    return {
      runs: [
        {
          run_id: RUN_ID,
          case_id: CASE_ID,
          plan_item_id: "pi_hello_1",
          execution: "completed",
          settlement: state.approvalDecisions.some((d) => d.verdict === "approve")
            ? "accepted"
            : "unsettled",
          evidence_count: 1,
          started_at: nowIso(),
          finished_at: nowIso(),
        },
      ],
    };
  }

  function runRecord() {
    const approved = state.approvalDecisions.find((d) => d.verdict === "approve");
    const settlementStatus = approved ? "accepted" : "unsettled";
    return {
      run_id: RUN_ID,
      case_id: CASE_ID,
      plan_item_id: "pi_hello_1",
      plan_item_name: "Greet",
      item_kind: "sandboxed_task",
      execution: "completed",
      settlement: settlementStatus,
      started_at: nowIso(),
      finished_at: nowIso(),
      authority: {
        decision_id: "dec_1",
        verdict: "allow",
        normalized_disposition: "allow",
        reason: "Local governed execution permitted by permissive.yaml.",
        decided_at: nowIso(),
        policy_refs: ["permissive.yaml"],
      },
      termination: {
        at: nowIso(),
        execution_status: "completed",
        exit_code: 0,
        trace_kind: "run_finished",
      },
      settlement_record: approved
        ? {
            settlement_id: "stl-101",
            status: "accepted",
            review_required: false,
            settled_at: approved.decided_at,
            basis: ["crit_output_present", `approval:${approved.approval_id}:approve`],
            criteria_ref: `criteria:${RUN_ID}`,
          }
        : null,
      criteria: [
        {
          criterion: "require_exit_zero",
          expected: "0",
          standing: "satisfied",
          basis: ["crit_output_present"],
        },
      ],
      evidence: [
        {
          evidence_id: "ev_1",
          kind: "artifact",
          uri: "artifacts/output.txt",
          sha256: ARTIFACT_SHA,
          source_event_id: "evt_1",
        },
      ],
      trace: [
        {
          event_id: "evt_0",
          kind: "command_started",
          actor_id: "operator_a",
          timestamp: nowIso(),
          payload: { argv: ["echo", ARTIFACT_CONTENT.trim()] },
        },
        {
          event_id: "evt_1",
          kind: "command_exited",
          actor_id: "operator_a",
          timestamp: nowIso(),
          payload: { exit_code: 0 },
        },
      ],
    };
  }

  const ASSET_LIST = {
    assets: [
      {
        asset_id: "tmpl_hello",
        kind: "plan_template",
        name: TEMPLATE_REF,
        version: "0.1.0",
        standing: "demonstrated",
        identity_digest: DIGEST_A,
      },
      {
        asset_id: "ep_local_agent",
        kind: "agent_endpoint",
        name: "local_agent",
        version: "1.0.0",
        standing: "available",
      },
    ],
  };

  const DELEGATION_PREVIEW = {
    endpoint: "local_agent",
    eligible: true,
    standing: "available",
    contract: {
      provider_kind: "local_agent",
      endpoint_digest: "sha256:" + sha256hex("harness-fixture:endpoint:local_agent"),
      model: { value: "local-governed-agent-v1", source: "endpoint_default" },
      transcript_retention: { value: "case_lifetime", source: "endpoint_default" },
      instruction_sha256: "sha256:" + sha256hex("harness-fixture:instructions:hello_agents"),
      instruction_bytes: 128,
      max_request_bytes: 65536,
      max_response_bytes: 65536,
      timeout_secs: 30,
      max_turns: 4,
      required_authority: "operator",
      contract_digest: "sha256:" + sha256hex("harness-fixture:job-contract:local_agent"),
    },
  };

  const DELEGATION_LIST = {
    delegations: [
      {
        run_id: RUN_ID,
        cancellable: false,
        endpoint_ref: "local_agent",
        standing: "settled",
        case_id: CASE_ID,
      },
    ],
  };

  function handlePreflight() {
    state.preflightCalls += 1;
    const digest = state.mode === "stale_once" && state.preflightCalls > 1 ? DIGEST_B : DIGEST_A;
    state.preflightDigests.push(digest);
    return Promise.resolve(preflightResult(digest));
  }

  function handleCommit(command) {
    state.commitCalls += 1;
    state.commits.push({
      request_id: command?.request_id ?? null,
      preconditions: command?.preconditions ?? null,
      at: nowIso(),
    });
    if (state.mode === "stale_once" && state.commitCalls === 1) {
      return Promise.resolve({
        outcome: "rejected_as_stale",
        code: "precondition_failed",
        changed_records: [{ record_ref: `template:${TEMPLATE_REF}`, change: "content_updated" }],
        next_actions: ["case.preflight"],
      });
    }
    if (state.mode === "drop_once" && state.commitCalls === 1) {
      return Promise.reject(new Error("mock transport: connection closed before response"));
    }
    return Promise.resolve({ case_id: CASE_ID, state: "active", exit_code: 0 });
  }

  function handleApprovalDecide(command) {
    const approvalId = command?.approval_id;
    const idx = state.pendingApprovals.findIndex((a) => a.approval_id === approvalId);
    if (idx === -1) {
      return Promise.resolve({ error: `approval ${approvalId} is not pending or does not exist` });
    }
    const verdict = command?.decision === "approve" ? "approve" : "reject";
    const [approval] = state.pendingApprovals.splice(idx, 1);
    const decided = {
      approval_id: approvalId,
      case_id: command?.case_id ?? approval.case_id,
      verdict,
      actor: "approver_alice",
      note: command?.note ?? null,
      decided_at: nowIso(),
    };
    state.approvalDecisions.push(decided);
    return Promise.resolve({ approval_id: approvalId, outcome: "decided", verdict });
  }

  function handleStatus() {
    state.statusCalls += 1;
    return Promise.resolve({ status: "completed", outcome: { case_id: CASE_ID, state: "active", exit_code: 0 } });
  }

  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      if (cmd === "sfwp_query") {
        const verb = args?.query?.verb;
        state.queryLog.push({ verb, args: args?.query, at: nowIso() });
        if (verb === "readiness_get") return Promise.resolve(READINESS);
        if (verb === "identity_get") return Promise.resolve(IDENTITY);
        if (verb === "case_entry_options") return Promise.resolve(ENTRY_OPTIONS);
        if (verb === "case_preflight") return handlePreflight();
        if (verb === "case_list") return Promise.resolve(caseList());
        if (verb === "case_get_overview") return Promise.resolve(caseOverview());
        if (verb === "case_get_horizon") return Promise.resolve(caseHorizon());
        if (verb === "approval_list") return Promise.resolve(approvalList());
        if (verb === "run_list") return Promise.resolve(runList());
        if (verb === "run_get") return Promise.resolve(runRecord());
        if (verb === "asset_list") return Promise.resolve(ASSET_LIST);
        if (verb === "delegation_preview") return Promise.resolve(DELEGATION_PREVIEW);
        if (verb === "delegation_list") return Promise.resolve(DELEGATION_LIST);
        if (verb === "system_describe") return Promise.resolve({ protocol_version: "1", methods: [] });
        if (verb === "system_hello")
          return Promise.resolve({ protocol_version: "1", server_protocol_version: "1", implemented_methods: [] });
        return Promise.resolve(null);
      }
      if (cmd === "sfwp_command") {
        const verb = args?.command?.verb;
        state.commandLog.push({ verb, args: args?.command, at: nowIso() });
        if (verb === "case_commit") return handleCommit(args.command);
        if (verb === "approval_decide") return handleApprovalDecide(args.command);
        return Promise.resolve({ outcome: "accepted" });
      }
      if (cmd === "sfwp_request_status") return handleStatus();
      if (cmd === "sfwp_identity") return Promise.resolve(IDENTITY);
      if (cmd === "sfwp_cell") return Promise.resolve({ socket_path: "/tmp/sea-forge-mock.sock", root: "/home/operator/.sea-forge-cells/mock-cell" });
      if (cmd === "plugin:event|listen") return Promise.resolve(1);
      if (cmd === "plugin:event|unlisten") return Promise.resolve(undefined);
      return Promise.resolve(null);
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

  return `sfwp full mock installed: mode=${state.mode}`;
})();
