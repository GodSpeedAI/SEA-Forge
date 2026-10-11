import type { Browser } from "../browser";
import type { Ctx } from "../ladder";
import { listCases } from "../live/durable";
import type { Stack } from "../live/stack";

export const W = 1672;
export const H = 941;
export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export type Who = "operator" | "rso";

export function stackOf(ctx: Ctx): Stack {
  if (!ctx.live) throw new Error("live journey run without a live stack (use bun e2e/run.ts --live)");
  return ctx.live as Stack;
}

/** The browser session that carries one end user's identity (one agent-browser --session each). */
export const userBrowser = (ctx: Ctx, who: Who): Browser => ctx.session(who);

export async function jsEval<T>(b: Browser, js: string): Promise<T> {
  return b.eval<T>(js);
}

export async function visible(b: Browser, testid: string): Promise<boolean> {
  return b.eval<boolean>(`!!document.querySelector('[data-testid="${testid}"]')`);
}

export interface SessionInfo {
  authenticated: boolean;
  [k: string]: unknown;
}

/** What the gateway says this browser's cookie session is (read over HTTP from inside the page). */
export async function sessionOf(b: Browser): Promise<SessionInfo> {
  return b.eval<SessionInfo>(
    `fetch('/api/session', { credentials: 'include' }).then((r) => r.json())`,
  );
}

/**
 * Real login through the rendered LoginScreen: type credentials, press Sign in. Dev auth accepts
 * any non-empty password; it is generated per run and never printed.
 */
export async function loginThroughUi(b: Browser, base: string, user: Who): Promise<void> {
  await b.viewport(W, H);
  await b.open(`${base}/`);
  await b.waitFor(
    `!!document.querySelector('[data-testid="login-screen"]') || !!(window.__gs && window.__gs.ready)`,
    30000,
    "login screen or ready app",
  );
  if (await visible(b, "login-screen")) {
    await b.fill('[data-testid="login-username"]', user);
    await b.fill('[data-testid="login-password"]', `dev-${Math.random().toString(36).slice(2)}`);
    await b.click('[data-testid="login-submit"]');
  }
  await b.waitFor(
    `!!(window.__gs && window.__gs.ready) && !!document.querySelector('[data-testid="session-badge"][data-state="signed-in"]')`,
    30000,
    `signed-in app for ${user}`,
  );
  const got = (await sessionOf(b)).actor_id;
  const want = user === "operator" ? "operator_local" : "rso_local";
  if (got !== want) throw new Error(`signed in as ${String(got)} but the ${user} session must be ${want} (a session is not shared between users)`);
}

/** Ensures the named user's session is signed in as that user (logs in through the UI if not). */
export async function ensureLoggedIn(ctx: Ctx, user: Who): Promise<Browser> {
  const b = userBrowser(ctx, user);
  const want = user === "operator" ? "operator_local" : "rso_local";
  let ok = false;
  try {
    ok = (await visible(b, "session-badge")) && (await sessionOf(b)).actor_id === want;
  } catch {
    ok = false;
  }
  if (!ok) await loginThroughUi(b, stackOf(ctx).base, user);
  return b;
}

/**
 * The case L1 committed. When the ladder runs a later journey on its own (`--only L2`) the shared
 * scratch is empty; then the cell must hold exactly one case (anything else is ambiguous).
 */
export function caseAOf(ctx: Ctx): string {
  const known = ctx.shared.caseA as string | undefined;
  if (known) return known;
  const cases = listCases(stackOf(ctx).cell);
  if (cases.length !== 1) throw new Error(`no case from L1 and the cell holds ${cases.length} cases; run L1 first`);
  ctx.shared.caseA = cases[0];
  return cases[0]!;
}

/** The fixture ladder's UI helpers drive `ctx.b`; this points them at another session's page. */
export function uiCtx(ctx: Ctx, b: Browser): Ctx {
  return { ...ctx, b };
}

/** The world object as the live UI holds it (read-only view of the rendered semantic state). */
export interface UiObject {
  id: string;
  kind: string;
  title: string;
  parent: string | null;
  status: string;
  subtitle?: string;
  actions: { id: string; intent: string }[];
}

export async function uiObjects(b: Browser): Promise<Record<string, UiObject>> {
  return b.eval<Record<string, UiObject>>(`(() => {
    const snap = window.__gs.worldView().snap;
    return Object.fromEntries(Object.values(snap.objects).filter((o) => o.kind !== 'core').map((o) => [o.id, {
      id: o.id, kind: o.contractKind ?? o.kind, title: o.title, parent: o.parent, status: o.status ? o.status.label : '',
      subtitle: o.subtitle, actions: (o.actions || []).map((a) => ({ id: a.id, intent: a.intent })) }]));
  })()`);
}

const outlineIsOpen = (b: Browser) => b.eval<boolean>(`(() => { const e = document.querySelector('.outline-view'); return !!e && getComputedStyle(e).display !== 'none' })()`);

/**
 * Opens the case outline from the home view with real key presses (H, then O). In the live world the
 * case's items orbit the core and show chips only at close zoom; the outline is the product's
 * accessible representation of the same objects, with the same standing and the same actions.
 */
export async function openOutlineAtHome(b: Browser): Promise<void> {
  if (await outlineIsOpen(b)) return;
  await b.eval<boolean>(`(document.activeElement && document.activeElement.blur && document.activeElement.blur(), true)`);
  await b.press("h");
  await b.waitFor(`window.__gs.store.getState().focusStack.length === 0`, 10000, "home");
  await b.press("o");
  await b.waitFor(`!!document.querySelector('.outline-view') && getComputedStyle(document.querySelector('.outline-view')).display !== 'none'`, 10000, "outline open");
  await b.waitFor(`document.querySelectorAll('[data-testid="outline-item"]').length >= 2`, 10000, "outline items listed");
}

export const outlineAction = (item: string, action: string) => `[data-testid="outline-action"][data-item="${item}"][data-action="${action}"]`;

export async function closeOutline(b: Browser): Promise<void> {
  if (!(await outlineIsOpen(b))) return;
  await b.eval<boolean>(`(document.activeElement && document.activeElement.blur && document.activeElement.blur(), true)`);
  await b.press("o");
  await b.waitFor(`getComputedStyle(document.querySelector('.outline-view')).display === 'none'`, 5000, "outline closed");
}

/** Presses the first outline row (a real focus on an item) so the focused-object center actions appear. */
export async function focusFirstOutlineItem(b: Browser): Promise<void> {
  await openOutlineAtHome(b);
  const item = Object.values(await uiObjects(b)).find((o) => o.kind === "work_item" || o.kind === "milestone");
  if (!item) throw new Error("the world holds no item to focus");
  await b.click(`[data-testid="outline-item"][data-id="${item.id}"] .outline-item-button`);
  await b.waitFor(`window.__gs.store.getState().focusStack.length > 0`, 10000, "an item focused");
  await closeOutline(b);
}

/**
 * Template -> parameters -> preflight -> commit through the rendered design panel, for a cell that
 * may already hold cases (home then offers "Design case" only once an item is focused; the empty
 * cell offers it directly). Returns the new case id, read from the kernel's own files.
 */
export async function designCaseViaUi(
  ctx: Ctx,
  b: Browser,
  templateRef: string,
  params: Record<string, string>,
): Promise<{ caseId: string; digest: string; trace: import("../live/durable").TraceRecord[] }> {
  const { listCases, waitForNewCase } = await import("../live/durable");
  const cell = stackOf(ctx).cell;
  const before = listCases(cell);
  if (!(await visible(b, "center-design"))) await focusFirstOutlineItem(b);
  await b.waitFor(`!!document.querySelector('[data-testid="center-design"]')`, 15000, "Design case entry");
  await b.click('[data-testid="center-design"]');
  await b.waitFor(`document.querySelectorAll('[data-testid="proposal-template"]').length >= 2`, 15000, "templates listed");
  await b.click(`[data-testid="proposal-template"][data-ref="${templateRef}"]`);
  await b.waitFor(`!!document.querySelector('[data-testid="proposal-param"]')`, 10000, "parameters rendered");
  for (const [k, v] of Object.entries(params)) await b.fill(`[data-testid="proposal-param"][data-name="${k}"]`, v);
  await b.click('[data-testid="proposal-preflight"]');
  await b.waitFor(`document.querySelector('[data-testid="proposal-preflight-result"]')?.dataset.passed === 'true'`, 20000, "preflight passed");
  const digest = await b.eval<string>(`document.querySelector('[data-testid="proposal-digest"]').innerText`);
  await b.click('[data-testid="proposal-submit"]');
  const found = await waitForNewCase(cell, before, ["case_created", "item_enabled"], { timeoutMs: 30000 });
  await b.waitFor(`!document.querySelector('[data-testid="proposal-panel"]')`, 15000, "panel closed");
  await b.waitFor(`window.__gs.worldView().snap.caseId === ${JSON.stringify(found.caseId)}`, 15000, "world shows the new case");
  return { caseId: found.caseId, digest, trace: found.trace };
}

/** The gateway's answer to an intent posted exactly the way the UI's adapter posts it (cookie + CSRF header). */
export async function postIntentFromPage(
  b: Browser,
  intent: { action: string; target: string; justification?: string; parameters?: Record<string, unknown> },
): Promise<{ status: number; body: { success?: boolean; refusal?: { refusal_kind?: string; message?: string }; error_code?: string; error_message?: string } }> {
  return b.eval(`(async () => {
    const csrf = decodeURIComponent((document.cookie.match(/(?:^|;\\s*)casework_csrf=([^;]+)/) || [])[1] || '');
    const s = window.__gs.store.getState();
    const body = {
      intent_id: crypto.randomUUID(), kind: 'CONSEQUENTIAL_CASE', action_name: ${JSON.stringify(intent.action)},
      target_object_id: ${JSON.stringify(intent.target)}, case_id: s.history.caseId,
      client_cursor: s.history.revisions[s.history.revisions.length - 1].id,
      actor: { actor_id: 'untrusted-client-claim', role: 'operator' },
      ${intent.justification ? `justification: ${JSON.stringify(intent.justification)},` : ""}
      ${intent.parameters ? `parameters: ${JSON.stringify(intent.parameters)},` : ""}
    };
    const r = await fetch('/api/intents', { method: 'POST', credentials: 'include', headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': csrf }, body: JSON.stringify(body) });
    return { status: r.status, body: await r.json().catch(() => ({})) };
  })()`);
}
