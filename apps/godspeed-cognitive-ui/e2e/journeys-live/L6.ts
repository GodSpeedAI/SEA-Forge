import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { Journey } from "../ladder";
import { distDir } from "../live/stack";
import { listRuns, readJsonl, readRunTrace, readTrace } from "../live/durable";
import { click, waitVisible } from "../journeys/helpers";
import { caseAOf, ensureLoggedIn, jsEval, stackOf, uiCtx } from "./helpers";

const ITEM = "task_prepare";
const RENDERERS = ["Diff", "Text", "Markdown", "Table", "Chart", "Json", "Graph", "Trace", "Timeline"] as const;

const sha256Hex = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

interface HarEntry {
  request?: { url?: string; method?: string };
  response?: { status?: number };
}

/** Request URLs (and statuses) out of an HAR 1.2 document. */
export function harEntries(har: unknown): { url: string; method: string; status: number }[] {
  const entries = ((har as { log?: { entries?: HarEntry[] } })?.log?.entries ?? []) as HarEntry[];
  return entries.map((e) => ({ url: String(e.request?.url ?? ""), method: String(e.request?.method ?? ""), status: Number(e.response?.status ?? 0) }));
}

// L6 settlement and the evidence dock. The kernel's own files are the oracle: SettlementRecorded in
// case-events.jsonl, the run's settlement.json, and the captured artifact's bytes on disk. The UI
// side opens that artifact through the product path (execution pill -> panel -> evidence -> dock),
// where the artifact service resolves the digest through GET /api/artifacts/{digest} and refuses a
// digest mismatch. The digest the dock shows must be the sha256 of the stored bytes. A HAR of the
// page load proves the renderer chunks are lazy (VAR-006): only the one this artifact needs loads.
export const L6: Journey = {
  id: "L6",
  title: "Settlement and the evidence dock",
  depends_on: ["L4"],
  settles: "a settled item's captured artifact opens in the dock with a verified digest, and only the needed renderer chunk is fetched",
  unlocks: ["L7"],
  async run(ctx) {
    const stack = stackOf(ctx);
    const cell = stack.cell;
    const caseId = caseAOf(ctx);
    const b = await ensureLoggedIn(ctx, "operator");
    const ui = uiCtx(ctx, b);

    let runId = "";
    let artifact = { path: "", sha: "", bytes: Buffer.alloc(0), evidenceId: "" };
    await ctx.step("durable: SettlementRecorded (accepted), the run's settlement.json, and the stored artifact bytes match the evidence journal", async () => {
      const trace = readTrace(cell, caseId);
      const settled = trace.find((e) => e.kind === "settlement_recorded" && e.plan_item_id === ITEM);
      ctx.expect(!!settled, `no settlement_recorded for ${ITEM} in ${trace.map((e) => e.kind).join(",")}`);
      ctx.expect((settled!.payload as { status?: string }).status === "accepted", `settlement payload ${JSON.stringify(settled!.payload)}`);
      runId = String((settled!.payload as { run_id?: string }).run_id);
      ctx.expect(listRuns(cell, caseId).includes(runId), `run ${runId} exists on disk`);
      const runDir = join(cell, "cases", caseId, "runs", runId);
      const settlement = JSON.parse(readFileSync(join(runDir, "settlement.json"), "utf8")) as { status: string; run_id: string };
      ctx.expect(settlement.status === "accepted" && settlement.run_id === runId, `settlement.json ${JSON.stringify(settlement)}`);
      const rows = readJsonl(join(runDir, "evidence.jsonl")).records as { evidence_id: string; kind: string; uri: string; sha256: string | null }[];
      const art = rows.filter((r) => r.kind === "artifact");
      ctx.expect(art.length === 1 && !!art[0]!.sha256, `artifact evidence rows ${JSON.stringify(art)}`);
      const file = join(runDir, art[0]!.uri);
      ctx.expect(existsSync(file), `stored artifact ${file}`);
      const bytes = readFileSync(file);
      const sha = sha256Hex(bytes);
      ctx.expect(sha === art[0]!.sha256!.replace(/^sha256:/, ""), `stored bytes hash ${sha} differs from the evidence row ${art[0]!.sha256}`);
      ctx.expect(readRunTrace(cell, caseId, runId).some((e) => e.kind === "artifact_captured"), "the run trace records the capture");
      artifact = { path: file, sha, bytes, evidenceId: art[0]!.evidence_id };
      ctx.delta({ step: "settled-and-stored", run_id: runId, settlement, artifact_sha256: sha, artifact_file: file.slice(cell.length) });
    });

    const harPath = join(ctx.out, "network", "L6-evidence-dock.har");
    mkdirSync(join(ctx.out, "network"), { recursive: true });
    await ctx.step("UI: a fresh page load of the case shows the settled standing at once (no stream event needed)", async () => {
      await b.harStart();
      await b.open(`${stack.base}/?case=${encodeURIComponent(caseId)}`);
      await b.waitFor(`!!(window.__gs && window.__gs.ready) && window.__gs.worldView().snap.caseId === ${JSON.stringify(caseId)}`, 30000, "case loaded");
      await b.waitFor(`document.querySelector('[data-testid="execution-pill"]')?.dataset.state === 'settled'`, 30000, "pill settled on a fresh load");
      const loaded = await jsEval<string[]>(b, `[...window.__gs.loadedRenderers]`);
      ctx.expect(loaded.length === 0, `renderers loaded before any artifact was opened: ${loaded.join(",")}`);
      await ctx.shot("fresh-load-pill", b);
    });

    await ctx.step("UI: pill -> execution panel lists the captured artifact as evidence, bound by the snapshot (not invented)", async () => {
      const bound = await jsEval<{ id: string; ref: string }[]>(b, `Object.values(window.__gs.worldView().snap.objects).filter((o) => (o.artifacts || []).length).map((o) => ({ id: o.id, ref: o.artifacts[0] }))`);
      ctx.expect(bound.length === 1 && bound[0]!.id === artifact.evidenceId && bound[0]!.ref === `sha256:${artifact.sha}`, `snapshot artifact bindings ${JSON.stringify(bound)} (want ${artifact.evidenceId} -> sha256:${artifact.sha.slice(0, 12)}…)`);
      await click(ui, '[data-testid="execution-pill"] button');
      await waitVisible(ui, '[data-testid="execution-evidence"]');
      await ctx.shot("execution-panel", b);
    });

    await ctx.step("UI: opening the evidence resolves it through artifact.get; the dock shows the digest of the stored bytes", async () => {
      await click(ui, '[data-testid="execution-evidence"]');
      await b.waitFor(`!!document.querySelector('[data-testid="dock-digest"]') || !!document.querySelector('[data-testid="artifact-error"]')`, 30000, "dock resolved");
      ctx.expect(!(await jsEval<boolean>(b, `!!document.querySelector('[data-testid="artifact-error"]')`)), "the dock showed an artifact error card");
      const shown = await jsEval<{ text: string; full: string }>(b, `(() => { const e = document.querySelector('[data-testid="dock-digest"]'); return { text: e.innerText.trim(), full: e.dataset.digest } })()`);
      ctx.expect(shown.full === `sha256:${artifact.sha}`, `dock digest ${shown.full} != sha256 of the stored bytes sha256:${artifact.sha}`);
      ctx.expect(shown.text === `sha256:${artifact.sha}`.slice(0, 19), `displayed digest ${shown.text}`);
      // The rendered content is the stored bytes, not a stand-in: its first non-empty line is on screen.
      const firstLine = artifact.bytes.toString("utf8").split("\n").map((l) => l.replace(/^#+\s*/, "").trim()).find((l) => l.length > 3)!;
      await b.waitFor(`document.querySelector('[data-testid="artifact-dock"]').innerText.includes(${JSON.stringify(firstLine)})`, 30000, "stored content rendered in the dock");
      await ctx.shot("evidence-dock", b);
      ctx.delta({ step: "dock-digest", displayed: shown.text, digest: shown.full, matches_stored_bytes: true });
    });

    await ctx.step("network: the HAR shows artifact.get once and ONLY the needed renderer chunk (VAR-006)", async () => {
      const loaded = await jsEval<string[]>(b, `[...window.__gs.loadedRenderers]`);
      ctx.expect(loaded.join(",") === "markdown", `renderers requested in the page: ${loaded.join(",")}`);
      await b.harStop(harPath);
      const entries = harEntries(JSON.parse(readFileSync(harPath, "utf8")));
      ctx.expect(entries.length > 5, `the HAR holds ${entries.length} entries`);
      const urls = entries.map((e) => e.url);
      const assets = readdirSync(join(distDir, "assets")).filter((f) => f.endsWith(".js"));
      const chunkOf = (name: string) => assets.filter((f) => f.startsWith(`${name}Renderer-`));
      for (const name of RENDERERS) ctx.expect(chunkOf(name).length === 1, `dist holds exactly one ${name} renderer chunk (${chunkOf(name).join(",")})`);
      const requested = (name: string) => urls.some((u) => chunkOf(name).some((f) => u.includes(`/assets/${f}`)));
      ctx.expect(requested("Markdown"), "the needed Markdown renderer chunk was fetched (positive control: the HAR sees lazy chunks)");
      const others = RENDERERS.filter((n) => n !== "Markdown" && requested(n));
      ctx.expect(others.length === 0, `renderer chunks fetched that this artifact does not need: ${others.join(",")}`);
      const art = entries.filter((e) => e.url.includes("/api/artifacts/"));
      ctx.expect(art.length === 1 && art[0]!.status === 200 && decodeURIComponent(art[0]!.url).endsWith(`sha256:${artifact.sha}`), `artifact requests ${JSON.stringify(art)}`);
      ctx.delta({ step: "lazy-chunks", har: harPath.slice(ctx.out.length + 1), entries: entries.length, renderer_chunks_requested: RENDERERS.filter(requested), artifact_requests: art });
    });
  },
};
