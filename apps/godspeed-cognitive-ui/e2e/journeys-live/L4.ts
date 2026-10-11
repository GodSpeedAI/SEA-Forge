import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import type { Journey } from "../ladder";
import { kindsOf, listRuns, readPlan, readRunTrace, readTrace, type TraceRecord, waitUntil } from "../live/durable";
import { click, node, waitVisible } from "../journeys/helpers";
import { caseAOf, closeOutline, ensureLoggedIn, jsEval, openOutlineAtHome, outlineAction, stackOf, uiCtx, uiObjects } from "./helpers";

const ITEM = "task_prepare";
const DOWNSTREAM = "task_publish";
const ACTION = `act-execute_item-${ITEM}`;

interface PageLog {
  timeOrigin: number;
  marker: string;
  pill: { t: number; state?: string; text?: string; visible?: boolean }[];
  store: { t: number; exec: { state: string; progress: number | null; phase: string } | null; prepare?: string; publish?: string; revs: number }[];
  fetches: { t: number; url: string; method: string }[];
}

// In-page recorder installed BEFORE the click. It lives on window, so it can only survive if the page
// never reloads; it records every pill change, every store change and every fetch the page makes.
const INSTRUMENT = (marker: string) => `(() => {
  const log = { timeOrigin: performance.timeOrigin, marker: ${JSON.stringify(marker)}, pill: [], store: [], fetches: [] };
  window.__l4 = log;
  let lastPill = '';
  const pill = () => {
    const e = document.querySelector('[data-testid="execution-pill"]');
    const p = e ? { state: e.dataset.state, text: e.innerText.trim(), visible: e.classList.contains('visible') } : {};
    const k = JSON.stringify(p);
    if (k !== lastPill) { lastPill = k; log.pill.push({ t: Date.now(), ...p }); }
  };
  new MutationObserver(pill).observe(document.body, { subtree: true, childList: true, attributes: true, characterData: true });
  pill();
  const st = window.__gs.store; let lastStore = '';
  st.subscribe(() => {
    const s = st.getState(); const o = window.__gs.worldView().snap.objects; const x = s.executions[${JSON.stringify(ITEM)}];
    const rec = { exec: x ? { state: x.state, progress: x.progress, phase: x.phase } : null, prepare: o[${JSON.stringify(ITEM)}] && o[${JSON.stringify(ITEM)}].status && o[${JSON.stringify(ITEM)}].status.label,
      publish: o[${JSON.stringify(DOWNSTREAM)}] && o[${JSON.stringify(DOWNSTREAM)}].status && o[${JSON.stringify(DOWNSTREAM)}].status.label, revs: s.history.revisions.length };
    const k = JSON.stringify(rec); if (k !== lastStore) { lastStore = k; log.store.push({ t: Date.now(), ...rec }); }
  });
  const f = window.fetch;
  window.fetch = function (...a) { log.fetches.push({ t: Date.now(), url: String((a[0] && a[0].url) || a[0]), method: (a[1] && a[1].method) || 'GET' }); return f.apply(this, a); };
  return true;
})()`;

const tsMs = (e: TraceRecord) => Date.parse(String(e.timestamp));

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) out.push(...walk(p));
    else out.push(p);
  }
  return out;
}

// L4 execute through the UI. The operator clicks Execute on the enabled item and confirms in the
// judgment panel. The kernel's durable state must show the activation, the run's own trace
// (authority, workspace, captured artifact; NO command events, because the template's item is a
// governed write_file and a write-only item runs no command), settlement accepted, completion, and
// the downstream item enabled by its settlement_status sentry. The UI side must follow through the
// live event stream alone: the page is never reloaded (a window marker survives), no snapshot is
// refetched, the pill and the downstream item change only AFTER the durable event they report, and
// no progress percentage is ever shown (the live gateway emits no execution_progress frames).
export const L4: Journey = {
  id: "L4",
  title: "Execute through the UI",
  depends_on: ["L2", "L3"],
  settles: "an executed item is durably active, settled and completed, and its downstream unlocks over SSE without a reload",
  unlocks: ["L5"],
  async run(ctx) {
    const cell = stackOf(ctx).cell;
    const caseId = caseAOf(ctx);
    const b = await ensureLoggedIn(ctx, "operator");
    const ui = uiCtx(ctx, b);
    const marker = `l4-${Math.random().toString(36).slice(2)}`;
    const before = readTrace(cell, caseId);
    const runsBefore = listRuns(cell, caseId);
    const plan = readPlan(cell, caseId);
    const planItem = plan.items.find((i) => i.plan_item_id === ITEM)!;

    await ctx.step("precondition: sentry-gated downstream is blocked; Execute is offered only on the enabled item", async () => {
      ctx.expect(!kindsOf(before).includes("item_activated"), "nothing has been activated yet");
      const objs = await uiObjects(b);
      ctx.expect(objs[DOWNSTREAM]!.status === "Waiting" && /entry sentries have not fired/.test(objs[DOWNSTREAM]!.subtitle ?? ""), `${DOWNSTREAM}: ${objs[DOWNSTREAM]!.status} / ${objs[DOWNSTREAM]!.subtitle}`);
      ctx.expect(objs[ITEM]!.actions.some((a) => a.intent === "EXECUTE_ITEM"), "Execute offered on the enabled item");
      ctx.expect(!objs[DOWNSTREAM]!.actions.some((a) => a.intent === "EXECUTE_ITEM"), "Execute not offered on the blocked item");
      ctx.expect(await b.eval<boolean>(INSTRUMENT(marker)), "page recorder installed");
    });

    await ctx.step("UI: Execute chip opens the judgment panel offering exactly Execute (not discretionary work); confirming sends the intent", async () => {
      await openOutlineAtHome(b);
      await click(ui, outlineAction(ITEM, ACTION));
      await waitVisible(ui, '[data-testid="judgment-panel"] [data-testid="judgment-option"]');
      const opts = await jsEval<string[]>(b, `[...document.querySelectorAll('[data-testid="judgment-option"]')].map((e) => e.dataset.option)`);
      ctx.expect(opts.join(",") === ACTION, `judgment options: ${opts.join(",")}`);
      await ctx.shot("judgment", b);
      await click(ui, `[data-testid="judgment-option"][data-option="${ACTION}"]`);
    });

    let after: TraceRecord[] = [];
    await ctx.step("durable: item_activated, settlement_recorded (accepted), item_completed, then item_enabled for the downstream item", async () => {
      after = await waitUntil(
        "execution trace in case-events.jsonl",
        () => {
          const t = readTrace(cell, caseId).slice(before.length);
          const ready = t.some((e) => e.kind === "item_enabled" && e.plan_item_id === DOWNSTREAM) && t.some((e) => e.kind === "item_completed" && e.plan_item_id === ITEM);
          return ready ? t : false;
        },
        { timeoutMs: 90000, watch: [join(cell, "cases", caseId, "case-events.jsonl")] },
      );
      const idx = (kind: string, item: string) => after.findIndex((e) => e.kind === kind && e.plan_item_id === item);
      const [act, set, done, en] = [idx("item_activated", ITEM), idx("settlement_recorded", ITEM), idx("item_completed", ITEM), idx("item_enabled", DOWNSTREAM)];
      ctx.expect(act >= 0 && set > act && done > set && en > set, `order activated=${act} settled=${set} completed=${done} downstream-enabled=${en}; delta kinds ${kindsOf(after).join(",")}`);
      ctx.expect(after[act]!.actor_id !== undefined, "activation names an actor");
      const status = (after[set]!.payload as { status?: string }).status;
      ctx.expect(status === "accepted", `settlement status ${status}`);
      ctx.expect(!after.some((e) => e.kind === "item_activated" && e.plan_item_id !== ITEM), "no other item was activated by the click");
      ctx.delta({ step: "execution-trace", case_id: caseId, delta_kinds: kindsOf(after), events: after.map((e) => ({ kind: e.kind, item: e.plan_item_id, at: e.timestamp, actor: e.actor_id })) });
    });

    let runId = "";
    await ctx.step("durable: the run's own trace shows authority, workspace, captured artifact and no command events; settlement.json accepted", async () => {
      const newRuns = listRuns(cell, caseId).filter((r) => !runsBefore.includes(r));
      ctx.expect(newRuns.length === 1, `new run directories: ${newRuns.join(",")}`);
      runId = newRuns[0]!;
      const run = await waitUntil("run trace with artifact_captured", () => {
        const t = readRunTrace(cell, caseId, runId);
        return t.some((e) => e.kind === "artifact_captured") ? t : false;
      }, { timeoutMs: 30000 });
      const kinds = kindsOf(run);
      for (const k of ["authority_evaluated", "workspace_created", "artifact_captured"]) ctx.expect(kinds.includes(k), `run trace lacks ${k}: ${kinds.join(",")}`);
      const writeOnly = (planItem.operations ?? []).every((o) => o.kind === "write_file");
      ctx.expect(writeOnly, "the template item's operations are all write_file");
      ctx.expect(!kinds.some((k) => k === "command_started" || k === "command_finished"), `a write-only item emitted command events: ${kinds.join(",")}`);
      ctx.expect(kinds.indexOf("authority_evaluated") < kinds.indexOf("workspace_created") && kinds.indexOf("workspace_created") < kinds.indexOf("artifact_captured"), `trace order ${kinds.join(",")}`);
      const settlement = JSON.parse(readFileSync(join(cell, "cases", caseId, "runs", runId, "settlement.json"), "utf8")) as { status?: string };
      ctx.expect(settlement.status === "accepted", `settlement.json status ${settlement.status}`);
      // The governed write really happened: the captured artifact exists under the run directory.
      const files = walk(join(cell, "cases", caseId, "runs", runId)).filter((f) => f.endsWith("dataset.md"));
      ctx.expect(files.length > 0 && statSync(files[0]!).size > 0, `captured dataset.md under the run dir (${files.length} found)`);
      const content = readFileSync(files[0]!, "utf8");
      const params = ctx.shared.caseAParams as { dataset_name: string } | undefined;
      ctx.expect(!params || content.includes(params.dataset_name), `artifact content carries the committed parameters: ${content.slice(0, 120)}`);
      ctx.delta({ step: "run-trace", run_id: runId, kinds, settlement, artifact: files[0] });
    });

    await ctx.step("UI: pill settles and the downstream item unlocks with no reload, no snapshot refetch, no fake progress", async () => {
      await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 60000, "pill settled");
      await b.waitFor(`window.__gs.worldView().snap.objects[${JSON.stringify(DOWNSTREAM)}]?.status?.label === 'Ready'`, 30000, "downstream Ready");
      // The outline the operator is looking at gains the downstream Execute button by itself.
      await b.waitFor(`!!document.querySelector('${outlineAction(DOWNSTREAM, "act-execute_item-task_publish")}')`, 30000, "downstream Execute appears in the open outline");
      await closeOutline(b);
      await waitVisible(ui, `${node(DOWNSTREAM)}`);
      const log = await jsEval<PageLog>(b, `window.__l4`);
      ctx.expect(log && log.marker === marker && log.timeOrigin === (await jsEval<number>(b, `performance.timeOrigin`)), "the page recorder survived: the page was never reloaded");
      const world = log.fetches.filter((f) => /\/api\/world/.test(f.url));
      ctx.expect(world.length === 0, `the page refetched snapshots instead of following the stream: ${JSON.stringify(world)}`);
      ctx.expect(log.fetches.some((f) => /\/api\/intents/.test(f.url) && f.method === "POST"), "the click sent an intent");
      ctx.expect(log.store.length >= 2 && log.store.at(-1)!.revs > log.store[0]!.revs, `revisions grew via the stream: ${log.store.map((s) => s.revs).join(",")}`);

      // No fake progress: no percentage anywhere, and progress is null until settled.
      ctx.expect(!log.pill.some((p) => /%/.test(p.text ?? "")), `the pill showed a percentage: ${JSON.stringify(log.pill.map((p) => p.text))}`);
      ctx.expect(log.store.every((s) => !s.exec || s.exec.state === "settled" || s.exec.progress === null), `progress reported while not settled: ${JSON.stringify(log.store.map((s) => s.exec))}`);

      // The UI never runs ahead of the kernel: each UI standing appears only after the durable event.
      const ev = (kind: string, item: string) => tsMs(after.find((e) => e.kind === kind && e.plan_item_id === item)!);
      const firstAt = <T extends { t: number }>(xs: T[], pred: (x: T) => boolean) => xs.find(pred)?.t;
      const checks: [string, number | undefined, number][] = [
        ["pill 'executed'", firstAt(log.pill, (p) => p.state === "executed"), ev("item_completed", ITEM)],
        ["pill 'settled'", firstAt(log.pill, (p) => p.state === "settled"), ev("settlement_recorded", ITEM)],
        [`${ITEM} 'Running'`, firstAt(log.store, (s) => s.prepare === "Running"), ev("item_activated", ITEM)],
        [`${ITEM} 'Settled accepted'`, firstAt(log.store, (s) => s.prepare === "Settled accepted"), ev("settlement_recorded", ITEM)],
        [`${DOWNSTREAM} 'Ready'`, firstAt(log.store, (s) => s.publish === "Ready"), ev("item_enabled", DOWNSTREAM)],
      ];
      const lines: string[] = [];
      for (const [what, uiAt, durableAt] of checks) {
        if (uiAt === undefined) continue; // fast runs may skip an intermediate standing; the final ones are required below
        lines.push(`${what}: ui ${uiAt} vs durable ${Math.floor(durableAt)}`);
        ctx.expect(uiAt >= Math.floor(durableAt), `${what} shown at ${uiAt}, before its durable event at ${durableAt}`);
      }
      ctx.expect(checks[1]![1] !== undefined && checks[3]![1] !== undefined && checks[4]![1] !== undefined, `final standings never observed: ${JSON.stringify(checks.map((c) => [c[0], c[1]]))}`);
      const objs = await uiObjects(b);
      ctx.expect(objs[ITEM]!.status === "Settled accepted", `${ITEM} status ${objs[ITEM]!.status}`);
      ctx.expect(objs[DOWNSTREAM]!.actions.some((a) => a.intent === "EXECUTE_ITEM"), `${DOWNSTREAM} now offers Execute`);
      ctx.expect(/ready to execute/.test(objs[DOWNSTREAM]!.subtitle ?? ""), `${DOWNSTREAM}: ${objs[DOWNSTREAM]!.subtitle}`);
      ctx.delta({ step: "ui-follows-stream", pill: log.pill, store: log.store, fetches: log.fetches.map((f) => `${f.method} ${f.url}`), ordering: lines });
      await ctx.shot("settled-unlocked", b);
    });
  },
};
