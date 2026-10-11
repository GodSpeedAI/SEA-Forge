import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import type { Journey } from "../ladder";
import type { Browser } from "../browser";
import { approvalsFile, delegatedRequests, kindsOf, readJsonl, readLedger, readTrace, waitForApproval, waitUntil } from "../live/durable";
import { digestOf, sessionCookieValue, sharedSessionFindings, type SessionFingerprint } from "../live/sessions";
import { click, waitVisible } from "../journeys/helpers";
import { closeOutline, designCaseViaUi, ensureLoggedIn, jsEval, openOutlineAtHome, outlineAction, postIntentFromPage, sessionOf, stackOf, uiCtx, uiObjects } from "./helpers";

const TEMPLATE = "e2e-signoff-gate@0.1.0";
const DRAFT = "task_draft";
const GATE = "signoff_release";
const EXECUTE = `act-execute_item-${DRAFT}`;
const APPROVE = `act-approve_human_task-${DRAFT}`;
const REJECT = `act-reject_human_task-${DRAFT}`;

type Rec = Record<string, unknown>;

async function fingerprint(label: string, b: Browser): Promise<SessionFingerprint> {
  let cookieDigest: string | null = null;
  let actorId: string | null = null;
  try {
    cookieDigest = digestOf(sessionCookieValue(await b.cookiesGet()));
  } catch {
    cookieDigest = null;
  }
  try {
    const s = await sessionOf(b);
    actorId = s.authenticated === false ? null : ((s.actor_id as string | undefined) ?? null);
  } catch {
    actorId = null;
  }
  return { label, agentSession: b.sessionName, cookieDigest, actorId };
}

const approvalsOf = (cell: string, caseId: string): Rec[] => readJsonl(approvalsFile(cell)).records.filter((r) => r.case_id === caseId);
const digestOfRecords = (rs: Rec[]) => createHash("sha256").update(JSON.stringify(rs)).digest("hex").slice(0, 16);

// L5 sign-off. The e2e policy escalates the signoff-gate template's governed draft write, so
// executing task_draft opens a real approval (approvals.jsonl) that only an approver may decide.
//  - The operator drives the work; the gateway offers it no Approve/Reject, and an approve intent it
//    posts anyway is refused with a typed UNAUTHORIZED_ROLE before the kernel is touched. The
//    durable approvals journal and case trace are byte-for-byte unchanged by the attempt.
//  - A second authenticated user (rso, actor rso_local, role R-SO) in a SEPARATE agent-browser
//    session logs in through the real LoginScreen, is offered Approve/Reject, is stopped by the
//    mandatory-justification gate, and approves with a reason. approvals.jsonl then shows the
//    request and the decision by a different principal than the one who executed.
export const L5: Journey = {
  id: "L5",
  title: "Sign-off: operator denied, R-SO approves in a separate session",
  depends_on: ["L4"],
  settles: "an approval opened by the operator's execution is resolved only by a distinct R-SO principal, with both principals visible in the durable record",
  unlocks: ["L6", "L7"],
  async run(ctx) {
    const cell = stackOf(ctx).cell;
    const opB = ctx.session("operator");
    const rsoB = ctx.session("rso");

    // Tooth (2) guard: two principals MUST NOT share an agent-browser session. This runs first and
    // by name only, so a shared-session run stops here without touching either user.
    await ctx.step("guard: the operator and the R-SO drive distinct agent-browser sessions", async () => {
      const findings = sharedSessionFindings(
        { label: "operator", agentSession: opB.sessionName, cookieDigest: null, actorId: null },
        { label: "rso", agentSession: rsoB.sessionName, cookieDigest: null, actorId: null },
      );
      ctx.expect(findings.length === 0, findings.join("; "));
      ctx.delta({ step: "session-guard", sessions: [opB.sessionName, rsoB.sessionName] });
    });

    const op = await ensureLoggedIn(ctx, "operator");
    const opCtx = uiCtx(ctx, op);

    let caseId = "";
    await ctx.step("operator designs a sign-off case through the UI; the kernel parks the human gate and enables the draft", async () => {
      const made = await designCaseViaUi(ctx, op, TEMPLATE, { change_summary: `l5-${Date.now().toString(36)}` });
      caseId = made.caseId;
      ctx.shared.caseB = caseId;
      const kinds = kindsOf(made.trace);
      ctx.expect(kinds[0] === "case_created", `first kind ${kinds[0]}`);
      const trace = await waitUntil("signoff gate parked", () => {
        const t = readTrace(cell, caseId);
        return t.some((e) => e.kind === "item_activated" && e.plan_item_id === GATE) ? t : false;
      }, { timeoutMs: 30000 });
      const parked = trace.find((e) => e.kind === "item_activated" && e.plan_item_id === GATE)!;
      ctx.expect((parked.payload as { human_task?: boolean }).human_task === true, `gate activation payload ${JSON.stringify(parked.payload)}`);
      ctx.expect(approvalsOf(cell, caseId).length === 0, "no approval exists before the draft is executed");
      ctx.delta({ step: "signoff-case", case_id: caseId, kinds: kindsOf(trace) });
    });

    let requested: Rec[] = [];
    await ctx.step("operator executes the draft; the kernel escalates it and opens a pending approval requested for this case", async () => {
      const objs = await uiObjects(op);
      ctx.expect(objs[DRAFT]!.actions.some((a) => a.intent === "EXECUTE_ITEM"), "Execute offered on the draft");
      await openOutlineAtHome(op);
      await click(opCtx, outlineAction(DRAFT, EXECUTE));
      await waitVisible(opCtx, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
      await click(opCtx, `[data-testid="judgment-option"][data-option="${EXECUTE}"]`);
      requested = (await waitForApproval(cell, (r) => r.case_id === caseId && r.status === "pending", { timeoutMs: 90000 })).filter((r) => r.case_id === caseId);
      ctx.expect(requested.length === 1 && requested[0]!.plan_item_id === DRAFT, `approvals for the case: ${JSON.stringify(requested)}`);
      ctx.expect(!("resolved_by" in requested[0]!) || requested[0]!.resolved_by == null, "the request carries no resolver");
      const trace = readTrace(cell, caseId);
      // The kernel's case trace names the engine; the requester is the actor of the escalated authority decision.
      const decision = readLedger(cell, `case-${caseId}`).find((e) => e.record_kind === "authority_decision" && e.payload.decision_id === requested[0]!.decision_id);
      const requester = ((decision?.payload.action_request as { actor?: { actor_id?: string } } | undefined)?.actor?.actor_id);
      ctx.expect(requester === "operator_local", `the escalated decision was requested by ${requester}`);
      ctx.expect(trace.some((e) => e.kind === "item_activated" && e.plan_item_id === DRAFT), "draft activated");
      ctx.shared.signoffRequester = requester;
      ctx.delta({ step: "approval-requested", approval: requested[0], requester });
    });

    const approvalId = () => String(requested[0]!.approval_id);
    let beforeDenied: { approvals: Rec[]; kinds: string[] } = { approvals: [], kinds: [] };
    await ctx.step("operator is denied: no Approve/Reject is offered and an approve intent posted anyway is refused, durable state unchanged", async () => {
      await closeOutline(op);
      await op.waitFor(`window.__gs.worldView().snap.objects[${JSON.stringify(DRAFT)}]?.status?.label !== 'Ready'`, 30000, "draft no longer Ready");
      const objs = await uiObjects(op);
      const intents = objs[DRAFT]!.actions.map((a) => a.intent);
      ctx.expect(!intents.includes("APPROVE_HUMAN_TASK") && !intents.includes("REJECT_HUMAN_TASK"), `operator was offered ${intents.join(",")}`);
      ctx.expect(/approval decision is open/i.test(objs[DRAFT]!.subtitle ?? "") || (await jsEval<boolean>(op, `JSON.stringify(window.__gs.worldView().snap.objects[${JSON.stringify(DRAFT)}]).includes('approval')`)), "the open approval is explained on the item");
      beforeDenied = { approvals: approvalsOf(cell, caseId), kinds: kindsOf(readTrace(cell, caseId)) };
      const delegatedBefore = delegatedRequests(cell).length;
      const res = await postIntentFromPage(op, { action: "APPROVE_HUMAN_TASK", target: DRAFT, justification: "operator approving their own execution" });
      const refusal = res.body.refusal;
      ctx.expect(res.body.success === false && !!refusal, `expected a typed refusal, got ${JSON.stringify(res)}`);
      ctx.expect(refusal!.refusal_kind === "UNAUTHORIZED_ROLE", `refusal kind ${refusal!.refusal_kind}: ${refusal!.message}`);
      await new Promise((r) => setTimeout(r, 1500));
      const after = { approvals: approvalsOf(cell, caseId), kinds: kindsOf(readTrace(cell, caseId)) };
      ctx.expect(digestOfRecords(after.approvals) === digestOfRecords(beforeDenied.approvals), "approvals.jsonl changed after the denied attempt");
      ctx.expect(after.kinds.join(",") === beforeDenied.kinds.join(","), `case trace changed after the denied attempt: ${after.kinds.join(",")}`);
      ctx.expect(after.approvals.every((a) => a.status === "pending"), "the approval is still pending");
      ctx.expect(delegatedRequests(cell).length === delegatedBefore, "the refused attempt never reached the kernel: no delegated request was recorded for it");
      // This step asserts the GATEWAY's refusal (UNAUTHORIZED_ROLE, before the kernel). Kernel-level SoD,
      // that the requester of an approval cannot resolve it themselves, is proven independently by:
      //  - crates/sea-forge-case-runner/tests/case_ops.rs
      //      the_requester_cannot_resolve_the_approval_their_own_action_opened (requester rule, case_ops::resolve_approval)
      //      the_proposer_cannot_resolve_its_own_items_approval (proposer rule)
      //  - crates/sea-forge-server/tests/conformance_identity.rs
      //      a_submitter_cannot_approve_their_own_work_but_another_actor_can (separation_of_duty over the socket)
      //  - crates/sea-forge-server/tests/sfwp_delegated_identity.rs
      //      separation_of_duty_holds_between_end_users_behind_one_gateway (SoD behind one gateway principal)
      ctx.delta({ step: "operator-denied", refusal, approvals_unchanged: true });
    });

    await ctx.step("guard: after both sign in, the two sessions hold different cookies and different principals", async () => {
      const rso = await ensureLoggedIn(ctx, "rso");
      const [a, b] = [await fingerprint("operator", op), await fingerprint("rso", rso)];
      // Identity collapse first (it is the root cause), then the expected principals.
      const findings = sharedSessionFindings(a, b);
      ctx.expect(findings.length === 0, findings.join("; "));
      ctx.expect(a.cookieDigest !== null && b.cookieDigest !== null, `session cookies were not observable (${a.cookieDigest}, ${b.cookieDigest})`);
      ctx.expect(a.actorId === "operator_local" && b.actorId === "rso_local", `principals ${a.actorId} / ${b.actorId}`);
      const sess = await sessionOf(rso);
      ctx.expect(sess.role === "R-SO", `rso role ${String(sess.role)}`);
      ctx.delta({ step: "sessions-distinct", operator: a, rso: b });
    });

    const rso = ctx.session("rso");
    const rsoCtx = uiCtx(ctx, rso);
    await ctx.step("R-SO (separate session) sees the same case with Approve/Reject on the draft; Approve without a reason is stopped in the panel", async () => {
      await rso.waitFor(`window.__gs.worldView().snap.caseId === ${JSON.stringify(caseId)}`, 30000, "rso world shows the sign-off case");
      const objs = await uiObjects(rso);
      const intents = objs[DRAFT]!.actions.map((a) => a.intent);
      ctx.expect(intents.includes("APPROVE_HUMAN_TASK") && intents.includes("REJECT_HUMAN_TASK"), `R-SO offered ${intents.join(",")}`);
      ctx.expect(!intents.includes("EXECUTE_ITEM"), "R-SO is not offered Execute");
      await openOutlineAtHome(rso);
      await click(rsoCtx, outlineAction(DRAFT, APPROVE));
      await waitVisible(rsoCtx, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
      const opts = await jsEval<string[]>(rso, `[...document.querySelectorAll('[data-testid="judgment-option"]')].map((e) => e.dataset.option)`);
      ctx.expect(opts.includes(APPROVE) && opts.includes(REJECT), `judgment options ${opts.join(",")}`);
      await ctx.shot("approve-needs-reason", rso);
      const sentBefore = await jsEval<number>(rso, `window.__gs.store.getState().intents.length`);
      await click(rsoCtx, `[data-testid="judgment-option"][data-option="${APPROVE}"]`);
      await rso.waitFor(`!!document.querySelector('.judgment-panel__validation-error')`, 5000, "reason-required message");
      await new Promise((r) => setTimeout(r, 800));
      ctx.expect((await jsEval<number>(rso, `window.__gs.store.getState().intents.length`)) === sentBefore, "an intent was sent without a justification");
      ctx.expect(digestOfRecords(approvalsOf(cell, caseId)) === digestOfRecords(beforeDenied.approvals), "approvals changed without a justification");
    });

    let resolved: Rec | undefined;
    const reason = "L5: checked the draft change note against the release policy";
    await ctx.step("durable: R-SO approves with a reason; approvals.jsonl shows the request and a decision by rso_local", async () => {
      await rso.fill('[data-testid="judgment-reason"]', reason);
      await click(rsoCtx, `[data-testid="judgment-option"][data-option="${APPROVE}"]`);
      const recs = await waitForApproval(cell, (r) => r.case_id === caseId && r.status === "approved", { timeoutMs: 90000 });
      const forCase = recs.filter((r) => r.case_id === caseId && r.approval_id === approvalId());
      ctx.expect(forCase.length >= 2, `request + decision expected for ${approvalId()}: ${forCase.map((r) => r.status).join(",")}`);
      ctx.expect(forCase[0]!.status === "pending" && forCase[0]!.resolved_by == null, "the first record is the unresolved request");
      resolved = forCase.at(-1);
      ctx.expect(resolved!.status === "approved" && resolved!.resolved_by === "rso_local", `decision ${JSON.stringify(resolved)}`);
      const principals = new Set<string>([String(ctx.shared.signoffRequester), String(resolved!.resolved_by)]);
      ctx.expect(principals.size === 2 && principals.has("operator_local") && principals.has("rso_local"), `principals ${[...principals].join(",")}`);
      ctx.expect(String(resolved!.note ?? "").includes("checked the draft"), `the decision records the justification: ${JSON.stringify(resolved!.note)}`);
      // The gateway acted for R-SO on the kernel's decision verb, and never for the operator on it.
      const audit = delegatedRequests(cell);
      const decisions = audit.filter((r) => /approv|decide/i.test(r.verb));
      ctx.expect(decisions.length === 1 && decisions[0]!.actor === "rso_local" && decisions[0]!.role === "R-SO", `delegated approval requests: ${JSON.stringify(decisions)}`);
      ctx.delta({ step: "approval-decided", records: forCase, principals: [...principals], delegated: audit });
    });

    await ctx.step("durable: the kernel records the resolution in the case ledger under rso_local; and the case trace gains nothing it did not emit", async () => {
      const ledger = readLedger(cell, `case-${caseId}`);
      const kinds = ledger.map((e) => e.record_kind);
      const resolution = ledger.find((e) => e.record_kind === "approval_resolution" && e.payload.approval_id === approvalId());
      ctx.expect(!!resolution, `case ledger lacks approval_resolution: ${kinds.join(",")}`);
      ctx.expect(kinds.indexOf("approval_request") < kinds.indexOf("approval_resolution"), `ledger order ${kinds.join(",")}`);
      const decisions = ledger.filter((e) => e.record_kind === "authority_decision");
      const last = decisions.at(-1)!;
      const decidedBy = (last.payload.action_request as { actor?: { actor_id?: string; role?: string } }).actor;
      ctx.expect(decisions.length === 2 && decidedBy?.actor_id === "rso_local", `resolution authority decision was requested by ${JSON.stringify(decidedBy)}`);
      // What the kernel really emits on approval: the decision lives in approvals.jsonl and the ledger. The
      // case trace and the run's settlement are NOT rewritten (settlement stays escalated).
      const trace = readTrace(cell, caseId);
      ctx.expect(kindsOf(trace).join(",") === beforeDenied.kinds.join(","), `the case trace changed on approval: ${kindsOf(trace).join(",")}`);
      const runId = readdirSync(join(cell, "cases", caseId, "runs"))[0]!;
      const settlement = JSON.parse(readFileSync(join(cell, "cases", caseId, "runs", runId, "settlement.json"), "utf8")) as { status: string };
      ctx.delta({ step: "post-approval-durable", ledger_kinds: kinds, case_trace_kinds: kindsOf(trace), settlement_status_after_approval: settlement.status, decided_by: decidedBy });
    });

    await ctx.step("UI: both sessions follow the approval over the stream with no reload; the decision is no longer offered", async () => {
      await rso.waitFor(`!window.__gs.worldView().snap.objects[${JSON.stringify(DRAFT)}]?.actions.some((a) => a.intent === 'APPROVE_HUMAN_TASK')`, 30000, "R-SO no longer offered Approve");
      await op.waitFor(`!/approval decision is open/i.test(window.__gs.worldView().snap.objects[${JSON.stringify(DRAFT)}]?.subtitle ?? '')`, 30000, "operator no longer sees an open approval");
      await ctx.shot("operator-after-approval", op);
      await ctx.shot("rso-after-approval", rso);
    });
  },
};
