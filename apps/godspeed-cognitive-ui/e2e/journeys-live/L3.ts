import type { Journey } from "../ladder";
import { kindsOf, readPlan, readTrace, waitForKinds } from "../live/durable";
import { click, waitVisible } from "../journeys/helpers";
import { caseAOf, closeOutline, ensureLoggedIn, jsEval, openOutlineAtHome, outlineAction, stackOf, uiCtx, uiObjects } from "./helpers";

const ANCHOR = "task_publish";
const ACTION = `act-add_discretionary_work-${ANCHOR}`;

// L3 discretionary add through the DiscretionaryDrawer. The UI proposes optional work anchored to
// an existing item; the kernel decides. Durable proof: exactly one new plan_mutated trace event
// whose actor is the verified operator, and a new plan.json item carrying proposed_by = that
// operator (the gateway never asserts the proposer; the kernel overwrites it with the verified
// actor). UI proof: the drawer's proposer badge names the session's operator, the new work appears
// in the world through the live stream without any reload.
export const L3: Journey = {
  id: "L3",
  title: "Discretionary add",
  depends_on: ["L2"],
  settles: "an operator-proposed item is durably in the plan with its proposer, and the UI shows it",
  unlocks: ["L4"],
  async run(ctx) {
    const cell = stackOf(ctx).cell;
    const caseId = caseAOf(ctx);
    const b = await ensureLoggedIn(ctx, "operator");
    const ui = uiCtx(ctx, b);
    const title = `L3 extra check ${Date.now().toString(36)}`;
    const summary = "verify the published report matches the dataset";
    const justification = "L3 live ladder: optional follow-up proposed through the drawer";
    const before = { trace: readTrace(cell, caseId), plan: readPlan(cell, caseId) };
    const countMutations = (t: { kind: string }[]) => t.filter((e) => e.kind === "plan_mutated").length;
    let marker = "";

    await ctx.step("the anchor item offers 'Add discretionary work'; clicking it opens the drawer with the operator as proposer", async () => {
      marker = `l3-${Math.random().toString(36).slice(2)}`;
      await b.eval<boolean>(`(window.__ladderMarker = ${JSON.stringify(marker)}, true)`);
      await openOutlineAtHome(b);
      await click(ui, outlineAction(ANCHOR, ACTION));
      await waitVisible(ui, '[data-testid="discretionary-drawer"] [data-testid="discretionary-title"]');
      const st = await jsEval<{ object: string; intent: string } | null>(b, `(() => { const d = window.__gs.store.getState().drawer; return d && { object: d.object, intent: d.action.intent } })()`);
      ctx.expect(st?.object === ANCHOR && st.intent === "ADD_DISCRETIONARY_WORK", `drawer state ${JSON.stringify(st)}`);
      // Operator is not offered a judgment panel for this: it is the add-work surface.
      ctx.expect(!(await jsEval<boolean>(b, `!!window.__gs.store.getState().judgment`)), "no judgment panel opened for discretionary work");
      const badge = await jsEval<string>(b, `document.querySelector('[data-testid="discretionary-proposer"]').innerText`);
      ctx.expect(/operator_local/.test(badge) && /operator/.test(badge), `proposer badge "${badge.replace(/\n/g, " ")}"`);
      await ctx.shot("drawer", b);
    });

    await ctx.step("an empty proposal is refused client-side with no durable effect", async () => {
      await click(ui, '[data-testid="discretionary-submit"]');
      await b.waitFor(`!!document.querySelector('[data-testid="discretionary-validation"]')`, 5000, "validation message");
      await b.fill('[data-testid="discretionary-title"]', title);
      await click(ui, '[data-testid="discretionary-submit"]');
      const msg = await jsEval<string>(b, `document.querySelector('[data-testid="discretionary-validation"]')?.innerText ?? ''`);
      ctx.expect(/justification/i.test(msg), `validation: ${msg}`);
      ctx.expect(countMutations(readTrace(cell, caseId)) === countMutations(before.trace), "no plan_mutated written for an invalid proposal");
    });

    let newId = "";
    await ctx.step("durable: submitting writes exactly one plan_mutated (actor operator_local) and a plan item with proposed_by", async () => {
      await b.fill('[data-testid="discretionary-summary"]', summary);
      await b.fill('[data-testid="discretionary-justification"]', justification);
      await click(ui, '[data-testid="discretionary-submit"]');
      // Durable delta FIRST: the drawer saying "Proposed" proves nothing about the kernel.
      const trace = await waitForKinds(cell, caseId, ["plan_mutated"], { timeoutMs: 30000 });
      ctx.expect(countMutations(trace) === countMutations(before.trace) + 1, `plan_mutated count ${countMutations(before.trace)} -> ${countMutations(trace)}`);
      const delta = trace.slice(before.trace.length);
      ctx.expect(kindsOf(delta).join(",") === "plan_mutated", `trace delta ${kindsOf(delta).join(",")}`);
      const ev = delta[0]!;
      ctx.expect(ev.actor_id === "operator_local", `plan_mutated actor ${ev.actor_id}`);
      const plan = readPlan(cell, caseId);
      const added = plan.items.filter((i) => !before.plan.items.some((p) => p.plan_item_id === i.plan_item_id));
      ctx.expect(added.length === 1, `plan gained ${added.length} items`);
      const item = added[0]!;
      newId = item.plan_item_id;
      ctx.expect(ev.plan_item_id === newId, `plan_mutated names ${ev.plan_item_id}, plan.json gained ${newId}`);
      ctx.expect(item.proposed_by === "operator_local", `proposed_by ${item.proposed_by}`);
      ctx.expect(item.name === title, `item name "${item.name}"`);
      // The kernel lowers the anchor into an entry criterion: the new item waits on the anchor's milestone event.
      const entry = (item.entry_criteria as { on?: { source?: string; event?: string } }[] | undefined) ?? [];
      ctx.expect(entry.length === 1 && entry[0]!.on?.source === ANCHOR, `item entry_criteria ${JSON.stringify(entry)}`);
      ctx.expect(item.operations?.[0]?.kind === "write_file" && item.operations[0].content_hint === summary, `item operation ${JSON.stringify(item.operations)}`);
      ctx.shared.discretionaryId = newId;
      ctx.delta({ step: "plan-mutated", case_id: caseId, item_id: newId, event: ev, proposed_by: item.proposed_by });
    });

    await ctx.step("UI: the drawer reports the kernel's acceptance and still names the operator as proposer", async () => {
      await b.waitFor(`document.querySelector('[data-testid="discretionary-outcome"]')?.dataset.state === 'accepted'`, 20000, "accepted outcome");
      const badge = await jsEval<string>(b, `document.querySelector('[data-testid="discretionary-proposer"]').innerText`);
      ctx.expect(/operator_local/.test(badge), `proposer badge after accept "${badge.replace(/\n/g, " ")}"`);
      await ctx.shot("accepted", b);
    });

    await ctx.step("UI: the proposed item arrives in the world through the live stream (no reload), waiting on its anchor", async () => {
      await b.waitFor(`!!window.__gs.worldView().snap.objects[${JSON.stringify(newId)}]`, 30000, "new item in the world");
      const objs = await uiObjects(b);
      const o = objs[newId]!;
      ctx.expect(o.title === title, `world title "${o.title}"`);
      ctx.expect(o.status === "Waiting", `new item status ${o.status}`);
      // The kernel's view verbs do not expose sentry predicates, so the honest reason is the generic sentry one.
      ctx.expect(/entry sentries have not fired/.test(o.subtitle ?? ""), `new item says why it waits: ${o.subtitle}`);
      ctx.expect(!o.actions.some((a) => a.intent === "EXECUTE_ITEM"), "a waiting item is not offered Execute");
      ctx.expect((await jsEval<string>(b, `window.__ladderMarker`)) === marker, "the page never reloaded (marker survived)");
      await click(ui, '[data-testid="discretionary-done"]');
      await closeOutline(b);
      await b.waitFor(`!window.__gs.store.getState().drawer`, 5000, "drawer closed");
    });
  },
};
