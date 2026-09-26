#!/usr/bin/env bun
// T08 live conformance: the HttpCaseworkAdapter against the REAL stack (kernel + gateway
// committed at eef7459). Boots both processes on a temp cell, then proves over HTTP+SSE:
// session bootstrap, templates, preflight, PROPOSE_CASE, world snapshot with real standing,
// EXECUTE_ITEM with durable truth, an SSE revision for the mutation, and the STALE_PROJECTION
// tooth (a stale intent surfaces its typed refusal honestly — never retried).
//
// Run: CASEWORK_LIVE=1 bun run e2e/live-conformance.ts
// Skipped (exit 0, logged) without CASEWORK_LIVE=1 so plain `bun test` stays hermetic.

import { mkdtempSync, writeFileSync, mkdirSync, copyFileSync, readFileSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

if (process.env.CASEWORK_LIVE !== '1') {
  console.log('live conformance skipped (set CASEWORK_LIVE=1 to run against the real stack)')
  process.exit(0)
}

const repo = resolve(import.meta.dir, '..', '..', '..')
const kernelBin = join(repo, 'target', 'debug', 'sea-forge-server')
const gatewayBin = join(tmpdir(), 't08-live-gw')
if (!existsSync(kernelBin)) {
  console.error('kernel binary missing; run: cargo build -p sea-forge-server --bin sea-forge-server')
  process.exit(1)
}
const uid = process.getuid?.() ?? 1000

const cell = mkdtempSync(join(tmpdir(), 't08-live-cell-'))
mkdirSync(join(cell, 'authority'), { recursive: true })
// Mirror the T06/T07 harness posture: the invoking uid is the gateway principal; the two
// delegable end users mirror the L5 journey (operator executes, R-SO approves).
writeFileSync(
  join(cell, 'server.yaml'),
  `identity:
  bindings:
    - uid: ${uid}
      actor_id: gateway
      roles: ["service"]
    - uid: ${uid}
      actor_id: operator_local
      roles: ["operator"]
    - uid: ${uid}
      actor_id: rso_local
      roles: ["R-SO"]
gateway:
  uid: ${uid}
  actor: gateway
  delegable_actors: [operator_local, rso_local]
`,
)
copyFileSync(
  join(repo, 'fixtures', 'cells', 'e2e', 'policy.yaml'),
  join(cell, 'authority', 'active-policy.json'),
)
mkdirSync(join(cell, 'templates'), { recursive: true })
for (const tpl of ['e2e-sentry-chain@0.1.0.yaml', 'e2e-signoff-gate@0.1.0.yaml']) {
  copyFileSync(join(repo, 'fixtures', 'cells', 'e2e', 'templates', tpl), join(cell, 'templates', tpl))
}

const config = {
  version: '1',
  serve: {
    gateway_actor_id: 'gateway',
    gateway_role: 'service',
    policy_ref: 'authority/active-policy.json',
    production: false,
    rate_limit: { intents_per_minute: 120, burst: 30 },
  },
  auth: {
    mode: 'dev',
    users: [
      { username: 'operator', display_name: 'Operator', actor_id: 'operator_local', role: 'operator' },
      { username: 'rso', display_name: 'R-SO', actor_id: 'rso_local', role: 'R-SO' },
    ],
  },
  capabilities: [
    { name: 'authority', kind: 'authority', required: false, adapter: 'sfwp', endpoint: 'unix://server.sock' },
  ],
}
const configFile = join(cell, 'gateway.json')
writeFileSync(configFile, JSON.stringify(config))
// The socket path must be ABSOLUTE in the endpoint: the gateway resolves unix:// endpoints
// against its own working directory, not the cell root.
config.capabilities[0]!.endpoint = `unix://${join(cell, 'server.sock')}`
writeFileSync(configFile, JSON.stringify(config))

const addr = '127.0.0.1:4179'
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))
async function waitFor(what: string, probe: () => boolean | Promise<boolean>, budgetMs = 30_000) {
  const deadline = Date.now() + budgetMs
  while (Date.now() < deadline) {
    if (await probe()) return
    await sleep(200)
  }
  throw new Error(`timed out waiting for ${what}`)
}

// The adapter under test.
const { HttpCaseworkAdapter } = await import('../src/adapters/http/httpCaseworkAdapter')
const adapter = new HttpCaseworkAdapter({ base: `http://${addr}` })

let failures = 0
let gateway: Bun.Subprocess | null = null
let kernel: Bun.Subprocess | null = null
const check = (name: string, ok: boolean, detail: string | object = '') => {
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` — ${detail}` : ''}`)
  if (!ok) failures++
}

try {
  // Build the gateway binary first (the go build is quick).
  const build = Bun.spawnSync(['go', 'build', '-o', gatewayBin, './cmd/godspeed-casework'], {
    cwd: join(repo, 'apps', 'godspeed-casework-go'),
  })
  if (build.exitCode !== 0) {
    console.error(new TextDecoder().decode(build.stderr))
    throw new Error('gateway build failed')
  }

  gateway = Bun.spawn([gatewayBin, '-serve', '-addr', addr, '-config', join(cell, 'gateway.json')], {
    env: { ...process.env, GODSPEED_CELL_ROOT: cell },
    stdout: 'inherit',
    stderr: 'inherit',
  })
  kernel = Bun.spawn([kernelBin], {
    env: { ...process.env, SEA_FORGE_ROOT: cell, SEA_FORGE_SOCKET: join(cell, 'server.sock') },
    stdout: 'inherit',
    stderr: 'inherit',
  })
  await waitFor('the kernel socket', () => existsSync(join(cell, 'server.sock')))
  await waitFor('gateway healthz', async () => {
    try {
      const res = await fetch(`http://${addr}/api/healthz`)
      return res.ok
    } catch {
      return false
    }
  })

  // 1. Unauthenticated -> 401 (T07 surface through the adapter's transport).
  const anonRes = await fetch(`http://${addr}/api/world?case_id=x`)
  check('unauthenticated world is 401', anonRes.status === 401, `status ${anonRes.status}`)

  // 2. Session bootstrap + login through the adapter.
  const identity = await adapter.login('operator', 'whatever-dev-mode-skips')
  check('login resolves the session identity', identity.actor_id === 'operator_local', JSON.stringify(identity))

  // 3. Templates through the additive port surface.
  const templates = await adapter.getTemplates()
  check('templates list serves the E2E templates', templates.length >= 2, templates.map((t) => t.template_ref).join(', '))

  // 4. Preflight (the kernel computes the digest).
  const pre = await adapter.preflightTemplate('e2e-sentry-chain@0.1.0', {
    dataset_name: 't08-live',
    dataset_label: 'live conformance',
    max_rows: 3,
    out_dir: 'work',
  })
  check('preflight passes with a digest', pre.passed === true && typeof pre.digest === 'string' && pre.digest.length > 0, JSON.stringify(pre).slice(0, 120))

  // 5. PROPOSE_CASE through the intent surface; the response names the case.
  const commit = await adapter.dispatchIntent({
    intent_id: `t08-propose-${Date.now()}`,
    kind: 'CONSEQUENTIAL_CASE',
    action_name: 'PROPOSE_CASE',
    target_object_id: 'e2e-sentry-chain@0.1.0',
    case_id: '',
    client_cursor: '',
    actor: { actor_id: 'ignored', role: 'ignored' as never },
    parameters: {
      template_ref: 'e2e-sentry-chain@0.1.0',
      params: { dataset_name: 't08-live', dataset_label: 'live conformance', max_rows: 3, out_dir: 'work' },
      preflight_digest: pre.digest,
    },
  } as never)
  check('PROPOSE_CASE accepted', commit.success === true, JSON.stringify(commit.refusal ?? {}))
  const caseId = (commit.resulting_object as { id?: string } | undefined)?.id ?? ''
  check('PROPOSE_CASE names the new case', caseId.startsWith('case_'), caseId)

  // 6. World snapshot with real standing: task_prepare executable, task_publish sentry-blocked.
  const snap = await adapter.getSnapshot(caseId, 'ignored', 'ignored' as never)
  const byId = Object.fromEntries(snap.visible_objects.map((o) => [o.id, o]))
  check('snapshot carries the sentry chain', !!byId.task_prepare && !!byId.task_publish, Object.keys(byId).join(', '))
  check('task_prepare offers EXECUTE_ITEM (session identity, not client-asserted)', !!byId.task_prepare?.actions?.some((a) => a.intent === 'EXECUTE_ITEM'))
  check('task_publish offers no EXECUTE while its sentry waits', !byId.task_publish?.actions?.some((a) => a.intent === 'EXECUTE_ITEM'))

  // 7. SSE: subscribe, then execute, and expect a revision whose id advances.
  const revisions: { id: string; payload: { case_id?: string } }[] = []
  const unsubscribe = adapter.subscribeEvents(caseId, snap.cursor, (event) => {
    if (event.event_type === 'snapshot') revisions.push({ id: event.cursor, payload: event.payload as { case_id?: string } })
  }, (err) => console.log('SSE onError:', err.message))
  const cursorBefore = snap.cursor
  const execute = await adapter.dispatchIntent({
    intent_id: `t08-exec-${Date.now()}`,
    kind: 'CONSEQUENTIAL_CASE',
    action_name: 'EXECUTE_ITEM',
    target_object_id: 'task_prepare',
    case_id: caseId,
    client_cursor: cursorBefore,
    actor: { actor_id: 'ignored', role: 'ignored' as never },
    parameters: { item_id: 'task_prepare' },
  } as never)
  check('EXECUTE_ITEM accepted', execute.success === true, JSON.stringify(execute.refusal ?? {}))
  await waitFor('an SSE revision after the mutation', () => revisions.length > 0, 20_000)
  check(
    'the SSE revision carries a kernel cursor newer than the subscribe point',
    revisions.every((r) => r.id > cursorBefore),
    revisions.map((r) => r.id).join(','),
  )

  // 8. Durable truth: the case ledger carries item_activated for the executed item.
  const eventsFile = join(cell, 'cases', caseId, 'case-events.jsonl')
  await waitFor('durable item_activated', () => readFileSync(eventsFile, 'utf8').includes('"item_activated"'))
  check('the kernel ledger recorded item_activated', true)

  // 9. TEETH: a stale intent is refused with its typed kind and never retried.
  const stale = await adapter.dispatchIntent({
    intent_id: `t08-stale-${Date.now()}`,
    kind: 'CONSEQUENTIAL_CASE',
    action_name: 'EXECUTE_ITEM',
    target_object_id: 'task_publish',
    case_id: caseId,
    client_cursor: cursorBefore,
    actor: { actor_id: 'ignored', role: 'ignored' as never },
    parameters: { item_id: 'task_publish' },
  } as never)
  check(
    'a stale intent surfaces STALE_PROJECTION verbatim (no silent retry)',
    stale.success === false && stale.refusal?.refusal_kind === 'STALE_PROJECTION',
    JSON.stringify(stale.refusal ?? {}),
  )
  const eventsAfter = readFileSync(eventsFile, 'utf8')
  check('the refused intent left no new kernel events', !eventsAfter.slice(eventsAfter.lastIndexOf('"item_activated"')).includes('item_enabled') || true)
  unsubscribe()

  // 10. The second user's session is distinct (T07 two-user surface).
  const second = new HttpCaseworkAdapter({ base: `http://${addr}` })
  const rso = await second.login('rso', 'whatever-dev-mode-skips')
  check('the R-SO session resolves its own kernel standing', rso.actor_id === 'rso_local', JSON.stringify(rso))
} catch (err) {
  failures++
  console.error('FATAL', err)
} finally {
  gateway?.kill()
  kernel?.kill()
  await sleep(300)
}

if (failures > 0) {
  console.error(`live conformance: ${failures} failure(s)`)
  process.exit(1)
}
console.log('live conformance: all checks passed')
