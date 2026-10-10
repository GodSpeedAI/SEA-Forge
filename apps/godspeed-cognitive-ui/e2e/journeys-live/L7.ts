import { readFileSync } from "node:fs";
import { join } from "node:path";
import type { Journey } from "../ladder";
import type { Browser } from "../browser";
import { kindsOf, readTrace, type TraceRecord, waitUntil } from "../live/durable";
import { click, waitVisible } from "../journeys/helpers";
import { caseAOf, closeOutline, ensureLoggedIn, jsEval, openOutlineAtHome, outlineAction, stackOf, uiCtx } from "./helpers";

const FINAL = "task_publish";
const GATE = "signoff_release";

const caseJson = (cell: string, caseId: string) => JSON.parse(readFileSync(join(cell, "cases", caseId, "case.json"), "utf8")) as { state: string; close_reason?: string | null };
const lifecycleOffers = (b: Browser) => jsEval<string[]>(b, `(window.__gs.worldView().snap.objects.core?.actions || []).map((a) => a.intent)`);

// L7 lifecycle. Every lifecycle action here is reached through the snapshot's ActionDescriptors
// (they ride on the Core, the case's own presence in the world) and every one of them asks for a
// reason in the JudgmentPanel before anything is sent. Durable truth is the kernel's case-events.jsonl
// and case.json; the UI must follow the stream with no reload.
//   case A (sentry chain): execute the last item -> case_closed -> Reopen -> case_reopened.
//   case B (the sign-off case from L5, still awaiting approval): complete the human gate -> human_task_completed;
//   Terminate with a reason -> case_terminated.
export const L7: Journey = {
  id: "L7",
  title: "Lifecycle: close, reopen, terminate",
  depends_on: ["L5", "L6"],
  settles: "a case closes when its work completes, reopens with a reason, and a second case is terminated with a reason, all as durable kernel events offered only by the snapshot",
  unlocks: [],
  async run(ctx) {
    const stack = stackOf(ctx);
    const cell = stack.cell;
    const caseA = caseAOf(ctx);
    const caseB = (ctx.shared.caseB as string | undefined) ?? "";
    ctx.expect(!!caseB, "L7 needs the sign-off case L5 created (run L5 first)");
    const b = await ensureLoggedIn(ctx, "operator");
    const ui = uiCtx(ctx, b);

    const openCase = async (caseId: string) => {
      await b.open(`${stack.base}/?case=${encodeURIComponent(caseId)}`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.worldView().snap.caseId === ${JSON.stringify(caseId)}`, 30000, `case ${caseId} loaded`);
    };

    /** Opens the Core's lifecycle action from the outline; asks for the reason; refuses to send without one. */
    const decide = async (intent: "REOPEN_CASE" | "TERMINATE_CASE", caseId: string, reason: string, before: TraceRecord[]) => {
      const actionId = `act-${intent.toLowerCase()}-${caseId}`;
      await openOutlineAtHome(b);
      await click(ui, outlineAction("core", actionId));
      await waitVisible(ui, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
      const opts = await jsEval<string[]>(b, `[...document.querySelectorAll('[data-testid="judgment-option"]')].map((e) => e.dataset.option)`);
      ctx.expect(opts.join(",") === actionId, `judgment options ${opts.join(",")} (want only ${actionId})`);
      const sent = await jsEval<number>(b, `window.__gs.store.getState().intents.length`);
      await click(ui, `[data-testid="judgment-option"][data-option="${actionId}"]`);
      await b.waitFor(`!!document.querySelector('.judgment-panel__validation-error')`, 5000, "reason-required message");
      await new Promise((r) => setTimeout(r, 800));
      ctx.expect((await jsEval<number>(b, `window.__gs.store.getState().intents.length`)) === sent, "an intent was sent without a reason");
      ctx.expect(kindsOf(readTrace(cell, caseId)).length === before.length, "the case trace changed without a reason");
      await ctx.shot(`${intent.toLowerCase()}-needs-reason`, b);
      await b.fill('[data-testid="judgment-reason"]', reason);
      await click(ui, `[data-testid="judgment-option"][data-option="${actionId}"]`);
    };

    // ---- case A: complete -> CaseClosed -> Reopen -------------------------------------------------
    let beforeA: TraceRecord[] = [];
    await ctx.step("case A is active: the snapshot offers Terminate and not Reopen, on the Core only", async () => {
      await openCase(caseA);
      beforeA = readTrace(cell, caseA);
      ctx.expect(!kindsOf(beforeA).includes("case_closed"), "case A is not closed yet");
      const offers = await lifecycleOffers(b);
      ctx.expect(offers.join(",") === "TERMINATE_CASE", `lifecycle offers while active: ${offers.join(",")}`);
      ctx.expect(caseJson(cell, caseA).state === "active", `case.json state ${caseJson(cell, caseA).state}`);
    });

    await ctx.step("durable: executing the last item completes it, achieves the milestone and appends case_closed; case.json is completed", async () => {
      await openOutlineAtHome(b);
      await click(ui, outlineAction(FINAL, `act-execute_item-${FINAL}`));
      await waitVisible(ui, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
      await click(ui, `[data-testid="judgment-option"][data-option="act-execute_item-${FINAL}"]`);
      const after = await waitUntil("case_closed in case A", () => {
        const t = readTrace(cell, caseA).slice(beforeA.length);
        return t.some((e) => e.kind === "case_closed") ? t : false;
      }, { timeoutMs: 90000 });
      const kinds = kindsOf(after);
      const idx = (k: string) => kinds.indexOf(k);
      ctx.expect(idx("item_completed") >= 0 && idx("milestone_achieved") > idx("item_completed") && idx("case_closed") > idx("milestone_achieved"), `order ${kinds.join(",")}`);
      ctx.expect(after.filter((e) => e.kind === "item_completed").every((e) => e.plan_item_id === FINAL), "only the final item completed in this step");
      ctx.expect(caseJson(cell, caseA).state === "completed", `case.json state ${caseJson(cell, caseA).state}`);
      ctx.delta({ step: "case-closed", case_id: caseA, kinds, events: after.map((e) => ({ kind: e.kind, item: e.plan_item_id, actor: e.actor_id, payload: e.payload })) });
    });

    await ctx.step("UI: with no reload the Core now offers Reopen and no longer offers Terminate", async () => {
      await closeOutline(b);
      await b.waitFor(`(window.__gs.worldView().snap.objects.core?.actions || []).some((a) => a.intent === 'REOPEN_CASE')`, 30000, "Reopen offered");
      const offers = await lifecycleOffers(b);
      ctx.expect(offers.join(",") === "REOPEN_CASE", `lifecycle offers after close: ${offers.join(",")}`);
      await ctx.shot("case-closed", b);
    });

    await ctx.step("durable: Reopen needs a reason in the panel; case_reopened is appended carrying it and by whom; case.json is active again", async () => {
      const t0 = readTrace(cell, caseA);
      const reason = "L7: late evidence arrived after the case closed";
      await decide("REOPEN_CASE", caseA, reason, t0);
      const after = await waitUntil("case_reopened in case A", () => {
        const t = readTrace(cell, caseA).slice(t0.length);
        return t.some((e) => e.kind === "case_reopened") ? t : false;
      }, { timeoutMs: 60000 });
      const ev = after.find((e) => e.kind === "case_reopened")!;
      ctx.expect(ev.actor_id === "operator_local", `reopened by ${ev.actor_id}`);
      ctx.expect((ev.payload as { reason?: string }).reason === reason, `reopen payload ${JSON.stringify(ev.payload)}`);
      ctx.expect(caseJson(cell, caseA).state === "active", `case.json state ${caseJson(cell, caseA).state}`);
      ctx.delta({ step: "case-reopened", event: ev, kinds_after_reopen: kindsOf(after) });
    });

    await ctx.step("UI: after the reopen the Core offers Terminate again and not Reopen", async () => {
      await b.waitFor(`(window.__gs.worldView().snap.objects.core?.actions || []).some((a) => a.intent === 'TERMINATE_CASE') && !(window.__gs.worldView().snap.objects.core?.actions || []).some((a) => a.intent === 'REOPEN_CASE')`, 30000, "Terminate offered, Reopen withdrawn");
      await ctx.shot("case-reopened", b);
    });

    // ---- case B: human gate, then Terminate with a reason -----------------------------------------
    let beforeB: TraceRecord[] = [];
    await ctx.step("durable: the operator completes the sign-off human task with a justification; human_task_completed is appended", async () => {
      await openCase(caseB);
      beforeB = readTrace(cell, caseB);
      ctx.expect(await jsEval<boolean>(b, `(window.__gs.worldView().snap.objects[${JSON.stringify(GATE)}]?.actions || []).some((a) => a.intent === 'COMPLETE_HUMAN_TASK')`), "Complete is offered on the parked human gate");
      await openOutlineAtHome(b);
      await click(ui, outlineAction(GATE, `act-complete_human_task-${GATE}`));
      await waitVisible(ui, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
      await b.fill('[data-testid="judgment-reason"]', "L7: release approved by the R-SO, recording the gate as done");
      await click(ui, `[data-testid="judgment-option"][data-option="act-complete_human_task-${GATE}"]`);
      const after = await waitUntil("human_task_completed in case B", () => {
        const t = readTrace(cell, caseB).slice(beforeB.length);
        return t.some((e) => e.kind === "human_task_completed") ? t : false;
      }, { timeoutMs: 60000 });
      const ev = after.find((e) => e.kind === "human_task_completed")!;
      ctx.expect(ev.plan_item_id === GATE && ev.actor_id === "operator_local", `human task completed ${JSON.stringify(ev)}`);
      ctx.expect(String((ev.payload as { note?: string }).note).includes("release approved"), `note ${JSON.stringify(ev.payload)}`);
      ctx.delta({ step: "human-task-completed", kinds: kindsOf(after), events: after.map((e) => ({ kind: e.kind, item: e.plan_item_id, actor: e.actor_id, payload: e.payload })), case_state: caseJson(cell, caseB).state });
    });

    await ctx.step("durable: Terminate on case B needs a reason; case_terminated carries it; case.json is terminated", async () => {
      await closeOutline(b);
      const offers = await lifecycleOffers(b);
      ctx.expect(offers.includes("TERMINATE_CASE"), `lifecycle offers on case B: ${offers.join(",")} (case.json ${caseJson(cell, caseB).state})`);
      const t0 = readTrace(cell, caseB);
      const reason = "L7: the release was withdrawn";
      await decide("TERMINATE_CASE", caseB, reason, t0);
      const after = await waitUntil("case_terminated in case B", () => {
        const t = readTrace(cell, caseB).slice(t0.length);
        return t.some((e) => e.kind === "case_terminated") ? t : false;
      }, { timeoutMs: 60000 });
      const ev = after.find((e) => e.kind === "case_terminated")!;
      ctx.expect((ev.payload as { reason?: string }).reason === reason && (ev.payload as { requested_by?: string }).requested_by === "operator_local", `terminate payload ${JSON.stringify(ev.payload)}`);
      const cj = caseJson(cell, caseB);
      ctx.expect(cj.state === "terminated", `case.json state ${cj.state}`);
      ctx.delta({ step: "case-terminated", event: ev, case_json: cj, kinds_after: kindsOf(after) });
    });

    await ctx.step("UI: the terminated case offers Reopen only, and no work can be executed or approved on it", async () => {
      await b.waitFor(`(window.__gs.worldView().snap.objects.core?.actions || []).map((a) => a.intent).join() === 'REOPEN_CASE'`, 30000, "only Reopen offered");
      const work = await jsEval<string[]>(b, `Object.values(window.__gs.worldView().snap.objects).flatMap((o) => (o.actions || []).map((a) => a.intent)).filter((i) => i !== 'REOPEN_CASE')`);
      ctx.expect(!work.some((i) => i === "EXECUTE_ITEM" || i === "APPROVE_HUMAN_TASK" || i === "COMPLETE_HUMAN_TASK"), `work still offered on a terminated case: ${work.join(",")}`);
      await ctx.shot("case-terminated", b);
    });
  },
};
