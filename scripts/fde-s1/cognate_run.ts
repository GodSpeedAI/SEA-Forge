// S1 gate (WP-19): one Cognate `program.run` run inside a Field Case episode, with a case context.
//
//   bun scripts/fde-s1/cognate_run.ts <spec.json>
//
// spec: { cognateDir, outDir, actor, role, tenant, semanticRef, worldRef, caseId, planItemId, kind,
//         episodeRef?, assignmentId?, expectationRefs?, workspace, patchFile, argv, timeoutMs? }
// The run is real: Cognate's runtime, its `program.run` capability (hash-pinned patch, declared command),
// the execution world, and the interaction-event outbox (`interactionOutboxFor`). The only wrapper is the
// store: an in-memory `DurableStore`, dumped to `outDir` as the raw records of this run. Nothing is edited.
//
// Output (in outDir): run-events.jsonl (every committed run event), observations.jsonl (the observation
// outbox rows, each with `interactionEvent`), interaction-events.ndjson (the extension wrapped in the minimal
// `execution_trace` envelope RealityTrace's `observed ingest` takes: the ungoverned-run gap of WP-15 note 6),
// run-view.json (final run view and the `program.run` result or failure).
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

type Spec = {
  cognateDir: string; outDir: string; actor: string; role: string; tenant?: string; semanticRef: string; worldRef: string;
  caseId: string; planItemId: string; kind: "evaluated" | "exploratory"; episodeRef?: string; assignmentId?: string;
  expectationRefs?: string[]; workspace: string; patchFile: string; argv: string[]; timeoutMs?: number;
};
const spec: Spec = JSON.parse(readFileSync(process.argv[2], "utf8"));
const pkg = (p: string) => `${spec.cognateDir}/packages/${p}`;
const { createExecutionWorldRegistry, localExecutionWorld, programExecutorComponent } = await import(pkg("execution/src/index.ts"));
const { createRuntime } = await import(pkg("runtime-bun/src/index.ts"));
const { createMemoryStore } = await import(pkg("store-memory/src/index.ts"));
const { interactionOutboxFor, OBSERVATION_TOPIC } = await import(pkg("realitytrace/src/index.ts"));

const patch = readFileSync(spec.patchFile, "utf8");
const patchSha256 = createHash("sha256").update(patch, "utf8").digest("hex");
const store = createMemoryStore();
const registry = createExecutionWorldRegistry([localExecutionWorld({ worldId: "w" })]);
const agents = [
  {
    id: "agent.program", version: "1",
    run: async (_input: unknown, ctx: any) =>
      ctx.invoke("program.run", { worldId: "w", workspace: spec.workspace, patch, patchSha256, argv: spec.argv, timeoutMs: spec.timeoutMs ?? 3_600_000 }),
  },
];
const runtime = await createRuntime({
  store: { store },
  agents,
  components: [programExecutorComponent(registry)],
  policy: { authorize: () => ({ allow: true, policyId: "fde-s1-local", reason: "local program executor policy; no SEA-Forge authority decision is consulted (I-1 execution gate not wired before WP-20)" }) },
  actions: { authorize: () => ({ allow: true, reason: "local" }) },
  outboxFor: (e: unknown) => interactionOutboxFor(e as never),
});
const caller = { actor: { id: spec.actor, kind: "user" }, tenant: spec.tenant ?? "fde-s1" };
const caseContext = {
  worldRef: spec.worldRef, semanticRef: spec.semanticRef, caseId: spec.caseId, planItemId: spec.planItemId, role: spec.role, kind: spec.kind,
  ...(spec.episodeRef ? { episodeRef: spec.episodeRef } : {}),
  ...(spec.assignmentId ? { assignmentId: spec.assignmentId } : {}),
  ...(spec.expectationRefs ? { expectationRefs: spec.expectationRefs } : {}),
};
const started = await runtime.service.startRun(caller, { agent: "agent.program", input: {}, idempotencyKey: `fde-s1:${spec.caseId}:${spec.planItemId}:${spec.actor}`, caseContext });
let view: any;
for (let i = 0; i < 100000; i++) {
  view = await runtime.service.getRun(caller, { runId: started.runId });
  if (view.status === "completed" || view.status === "failed" || view.status === "cancelled") break;
  await Bun.sleep(250);
}
mkdirSync(spec.outDir, { recursive: true });
const events: unknown[] = [];
for await (const e of (await runtime.service.events(caller, {})) as AsyncIterable<unknown>) events.push(e);
const rows = (await store.listOutbox({ topic: OBSERVATION_TOPIC })).map((i: any) => ({ id: i.id, observation: i.payload }));
const jl = (xs: unknown[]) => xs.map((x) => JSON.stringify(x)).join("\n") + (xs.length ? "\n" : "");
writeFileSync(join(spec.outDir, "run-events.jsonl"), jl(events));
writeFileSync(join(spec.outDir, "observations.jsonl"), jl(rows));
writeFileSync(join(spec.outDir, "interaction-events.ndjson"), jl(rows.map((r: any) => ({ envelope_kind: "execution_trace", extensions: { "godspeed.interaction_event": r.observation.interactionEvent } }))));
writeFileSync(join(spec.outDir, "run-view.json"), JSON.stringify({ spec: { ...spec, patchSha256 }, view }, null, 1) + "\n");
await runtime.close();
const ok = view?.status === "completed";
console.log(JSON.stringify({ status: view?.status, runId: started.runId, patchSha256, interactionEvents: rows.length }));
process.exit(ok ? 0 : 3);
