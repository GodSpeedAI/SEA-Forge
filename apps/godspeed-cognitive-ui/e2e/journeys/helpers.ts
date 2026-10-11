import type { Ctx } from "../ladder";

// Journey helpers. Every interaction goes through the rendered UI: real pointer input at an
// element's on-screen position, real key presses, typing into the composer. `__gs` (the dev
// handle) is only READ, to inspect rendered/semantic state; no journey dispatches through it.

export const W = 1672;
export const H = 941;

export interface Snap {
  focusStack: string[];
  focus: string | null;
  surface: string;
  mode: string;
  revision: string;
  now: string;
  revisions: string[];
  timeline: boolean;
  marks: string[];
  compare: { a: string; b: string; source: string } | null;
  selection: string | null;
  artifacts: { id: string; phase: string; pinned: boolean }[];
  narrative: { id: string; index: number; status: string; error?: string } | null;
  beats: number;
  judgment: { object: string; pending?: string; outcome?: { state: string; code?: string; by: { name: string; kind: string } } } | null;
  intents: { actionName: string; actor: { kind: string; id: string }; state: string; code?: string }[];
  executions: Record<string, { state: string; progress: number }>;
  design: { revision: string; hasDraft: boolean; selected: string | null } | null;
  agent: string;
  overrides: { caption?: string; reveal: string[] };
}

export async function open(ctx: Ctx, query = ""): Promise<void> {
  await ctx.b.viewport(W, H);
  await ctx.b.open(`${ctx.base}/${query ? `?${query}` : ""}`);
  await ctx.b.waitFor("!!(window.__gs && window.__gs.ready) && document.querySelectorAll('[data-node]').length > 3", 30000, "app ready");
  await sleep(1500);
}

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function state(ctx: Ctx): Promise<Snap> {
  return ctx.b.eval<Snap>(`(() => {
    const s = window.__gs.store.getState();
    const f = s.focusStack[s.focusStack.length - 1] ?? null;
    const revs = s.history.revisions.map(r => r.id);
    const n = s.narrative ? s.narratives[s.narrative.id] : null;
    return {
      focusStack: s.focusStack, focus: f, surface: s.surface, mode: s.mode, revision: s.revision,
      now: revs[revs.length - 1], revisions: revs, timeline: s.timeline, marks: s.timeMarks,
      compare: s.compare && { a: s.compare.a, b: s.compare.b, source: s.compare.source },
      selection: s.selection, artifacts: s.artifacts,
      narrative: s.narrative, beats: n ? n.beats.length : 0,
      judgment: s.judgment && { object: s.judgment.object, pending: s.judgment.pending, outcome: s.judgment.outcome },
      intents: s.intents.map(i => ({ actionName: i.actionName, actor: { kind: i.actor.kind, id: i.actor.id }, state: i.state, code: i.code })),
      executions: Object.fromEntries(Object.entries(s.executions).map(([k, e]) => [k, { state: e.state, progress: e.progress }])),
      design: s.design && { revision: s.design.revision, hasDraft: !!s.design.draft, selected: s.design.selected },
      agent: s.agent,
      overrides: { caption: s.overrides.caption, reveal: s.overrides.reveal },
    };
  })()`);
}

/** Position of an element if it is rendered and visible on screen. */
export async function rect(ctx: Ctx, selector: string): Promise<{ x: number; y: number; w: number; h: number; op: number } | null> {
  return ctx.b.eval(`(() => {
    const els = [...document.querySelectorAll(${JSON.stringify(selector)})];
    for (const e of els) {
      const r = e.getBoundingClientRect();
      let op = 1, n = e;
      while (n && n.nodeType === 1) { const cs = getComputedStyle(n); if (cs.display === 'none' || cs.visibility === 'hidden') return null; op *= Number(cs.opacity); n = n.parentElement; }
      if (r.x < innerWidth && r.y < innerHeight && r.right >= 0 && r.bottom >= 0 && (r.width > 0 || e.children.length || e.hasAttribute('data-node'))) return { x: r.x, y: r.y, w: r.width, h: r.height, op };
    }
    return null;
  })()`);
}

export async function visible(ctx: Ctx, selector: string): Promise<boolean> {
  const r = await rect(ctx, selector);
  return !!r && r.op > 0.2;
}

/** Waits until the element is on screen and still (world objects drift slightly; tweens settle). */
export async function waitVisible(ctx: Ctx, selector: string, timeout = 15000): Promise<void> {
  const t0 = Date.now();
  let last: { x: number; y: number } | null = null;
  while (Date.now() - t0 < timeout) {
    const r = await rect(ctx, selector);
    if (r && r.op > 0.3) {
      if (last && Math.hypot(r.x - last.x, r.y - last.y) < 3) return;
      last = { x: r.x, y: r.y };
    } else last = null;
    await sleep(350);
  }
  throw new Error(`Timed out waiting for ${selector} to be visible`);
}

/** Real pointer click on the visible element (centre of its hit area). */
export async function click(ctx: Ctx, selector: string, opts: { at?: "orb" } = {}): Promise<void> {
  // Panels scroll like any UI: bring a panel control into view first (world objects are never scrolled).
  await ctx.b.eval(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); if (e && !e.closest('.world-layer')) e.scrollIntoView({ block: 'center' }); return true })()`);
  // Controls on a world object (action chips) appear on hover, as for a user: hover the object first.
  const host = await ctx.b.eval<{ x: number; y: number } | null>(`(() => { const e = document.querySelector(${JSON.stringify(selector)}); const n = e && !e.hasAttribute('data-node') && e.closest('[data-node]'); if (!n) return null; const o = n.querySelector('.orb, .dot') || n; const r = o.getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 } })()`);
  if (host) {
    await ctx.b.mouseMove(host.x, host.y);
    await sleep(500);
  }
  await waitVisible(ctx, selector);
  const r = opts.at === "orb" ? await rect(ctx, `${selector} .orb, ${selector} .dot`) ?? (await rect(ctx, selector)) : await rect(ctx, selector);
  if (!r) throw new Error(`No clickable ${selector}`);
  await ctx.b.mouseClick(r.x + r.w / 2, r.y + r.h / 2);
  await sleep(300);
}

export const node = (id: string) => `[data-node="${id}"]`;

export async function clickNode(ctx: Ctx, id: string): Promise<void> {
  await click(ctx, node(id), { at: "orb" });
}

export async function wake(ctx: Ctx): Promise<void> {
  await ctx.b.mouseMove(W * 0.3, H * 0.3);
  await ctx.b.mouseMove(W * 0.32, H * 0.31);
  await sleep(800);
}

export async function ask(ctx: Ctx, text: string): Promise<void> {
  await click(ctx, ".composer-input");
  await ctx.b.fill(".composer-input", text);
  await ctx.b.press("Enter");
  await sleep(400);
}

export async function until(ctx: Ctx, label: string, pred: (s: Snap) => boolean, timeout = 20000): Promise<Snap> {
  const t0 = Date.now();
  let s = await state(ctx);
  while (!pred(s)) {
    if (Date.now() - t0 > timeout) throw new Error(`Timed out waiting for ${label} (state: ${JSON.stringify({ focus: s.focus, surface: s.surface, mode: s.mode, revision: s.revision, narrative: s.narrative, judgment: s.judgment, executions: s.executions })})`);
    await sleep(300);
    s = await state(ctx);
  }
  return s;
}

export async function count(ctx: Ctx, selector: string): Promise<number> {
  return ctx.b.eval<number>(`document.querySelectorAll(${JSON.stringify(selector)}).length`);
}

export async function text(ctx: Ctx, selector: string): Promise<string> {
  return ctx.b.eval<string>(`(document.querySelector(${JSON.stringify(selector)})?.textContent ?? '').trim()`);
}

/** Stable fingerprint of all authoritative history (to prove no source mutation). */
export async function historyHash(ctx: Ctx): Promise<string> {
  return ctx.b.eval<string>(`(() => {
    const s = JSON.stringify(window.__gs.store.getState().history.snapshots);
    let h = 2166136261; for (let i = 0; i < s.length; i++) h = Math.imul(h ^ s.charCodeAt(i), 16777619);
    return (h >>> 0).toString(16) + ':' + s.length;
  })()`);
}

export async function loadedRenderers(ctx: Ctx): Promise<string[]> {
  return ctx.b.eval<string[]>("[...window.__gs.loadedRenderers]");
}

/** Chunk requests for source renderers seen by the browser (VAR-006). */
export async function rendererRequests(ctx: Ctx): Promise<string[]> {
  return ctx.b.eval<string[]>("performance.getEntriesByType('resource').map(e => e.name).filter(n => /renderers\\/[A-Za-z]+Renderer/.test(n)).map(n => n.replace(/^.*renderers\\//, '').replace(/[?.].*$/, ''))");
}

/** Marks the persistent Core (canvas + core node) so later journeys can prove it is never recreated. */
export async function markCore(ctx: Ctx): Promise<void> {
  await ctx.b.eval("(() => { document.querySelector('canvas.core-canvas').__coreMark = 'j'; document.querySelector('[data-node=\"core\"]').__coreMark = 'j'; return true })()");
}
export async function coreIntact(ctx: Ctx): Promise<boolean> {
  return ctx.b.eval<boolean>("document.querySelectorAll('canvas.core-canvas').length === 1 && document.querySelector('canvas.core-canvas').__coreMark === 'j' && document.querySelectorAll('[data-node=\"core\"]').length === 1 && document.querySelector('[data-node=\"core\"]').__coreMark === 'j'");
}

export async function markNode(ctx: Ctx, id: string, mark: string): Promise<void> {
  await ctx.b.eval(`(() => { const e = document.querySelector('[data-node="${id}"]'); if (e) e.__mark = ${JSON.stringify(mark)}; return !!e })()`);
}
export async function nodeMark(ctx: Ctx, id: string): Promise<string | null> {
  return ctx.b.eval<string | null>(`document.querySelector('[data-node="${id}"]')?.__mark ?? null`);
}

export async function camera(ctx: Ctx): Promise<{ x: number; y: number; zoom: number }> {
  return ctx.b.eval("(() => { const c = window.__gs.runtime.currentCamera(); return { x: c.x, y: c.y, zoom: c.zoom } })()");
}
export async function flightDone(ctx: Ctx): Promise<void> {
  await ctx.b.waitFor("!window.__gs.runtime.flight", 20000, "camera flight to settle");
  await sleep(600);
}

/** Enters the Northstar case the way a user does: Projects, then Northstar. */
export async function enterCase(ctx: Ctx): Promise<void> {
  await wake(ctx);
  await clickNode(ctx, "cat-projects");
  await until(ctx, "focus Projects", (s) => s.focus === "cat-projects");
  await flightDone(ctx);
  await clickNode(ctx, "northstar");
  await until(ctx, "focus Northstar", (s) => s.focus === "northstar");
  await flightDone(ctx);
}

export async function shot(ctx: Ctx, name: string): Promise<void> {
  await sleep(700);
  await ctx.shot(name);
}
