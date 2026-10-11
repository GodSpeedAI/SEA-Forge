#!/usr/bin/env bun
// T08 live conformance: the HttpCaseworkAdapter against the REAL stack (kernel + gateway
// committed at eef7459). Boots both processes on a temp cell, then proves over HTTP+SSE:
// session bootstrap, templates, preflight, PROPOSE_CASE, world snapshot with real standing,
// EXECUTE_ITEM with durable truth, an SSE revision for the mutation, and the STALE_PROJECTION
// tooth (a stale intent surfaces its typed refusal honestly — never retried).
//
// Run: CASEWORK_LIVE=1 bun run e2e/live-conformance.ts
// Skipped (exit 0, logged) without CASEWORK_LIVE=1 so plain `bun test` stays hermetic.

import {
  mkdtempSync,
  writeFileSync,
  mkdirSync,
  copyFileSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  existsSync,
  rmSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

if (process.env.CASEWORK_LIVE !== '1') {
  console.log('live conformance skipped (set CASEWORK_LIVE=1 to run against the real stack)')
  process.exit(0)
}

const repo = resolve(import.meta.dir, '..', '..', '..')
const kernelBin = join(repo, 'target', 'debug', 'sea-forge-server')
if (!existsSync(kernelBin)) {
  console.error('kernel binary missing; run: cargo build -p sea-forge-server --bin sea-forge-server')
  process.exit(1)
}
const uid = process.getuid?.() ?? 1000

const runRoot = mkdtempSync(join(tmpdir(), 't08-live-run-'))
const gatewayBin = join(runRoot, 'godspeed-casework')
const cell = join(runRoot, 'cell')
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
function assertPortAvailable(hostname: string, port: number) {
  let probe: Bun.TCPSocketListener<undefined>
  try {
    probe = Bun.listen({ hostname, port, socket: { data() {} } })
  } catch (err) {
    throw new Error(`live conformance port ${hostname}:${port} is already in use; leaving its owner untouched`, { cause: err })
  }
  probe.stop(true)
}

function runnerOwnsGatewayListener(proc: Bun.Subprocess, port: number): boolean {
  if (proc.exitCode !== null) return false

  const localAddress = `0100007F:${port.toString(16).toUpperCase().padStart(4, '0')}`
  const ownedSocketInodes = new Set<string>()
  try {
    // This test stack binds IPv4 loopback explicitly. Walk only the spawned gateway's fds;
    // unreadable or racing procfs entries simply fail to establish ownership.
    for (const fd of readdirSync(`/proc/${proc.pid}/fd`)) {
      try {
        const socket = /^socket:\[(\d+)\]$/.exec(readlinkSync(`/proc/${proc.pid}/fd/${fd}`))
        if (socket) ownedSocketInodes.add(socket[1]!)
      } catch {
        // The child may close an fd between readdir and readlink.
      }
    }
    const listeningInodes = new Set(
      readFileSync('/proc/net/tcp', 'utf8')
        .split(/\r?\n/)
        .map((line) => line.trim().split(/\s+/))
        .filter((fields) => fields[1]?.toUpperCase() === localAddress && fields[3] === '0A')
        .map((fields) => fields[9])
        .filter((inode): inode is string => typeof inode === 'string'),
    )
    return proc.exitCode === null && [...ownedSocketInodes].some((inode) => listeningInodes.has(inode))
  } catch {
    // Missing/restricted procfs is not evidence that the runner owns the endpoint.
    return false
  }
}

async function waitFor(what: string, probe: () => boolean | Promise<boolean>, budgetMs = 30_000) {
  const deadline = Date.now() + budgetMs
  while (Date.now() < deadline) {
    if (await probe()) return
    await sleep(200)
  }
  throw new Error(`timed out waiting for ${what}`)
}

async function exitedWithin(proc: Bun.Subprocess, budgetMs: number): Promise<boolean> {
  let timer: ReturnType<typeof setTimeout> | undefined
  const outcome = await Promise.race([
    proc.exited.then(() => true),
    new Promise<boolean>((resolve) => {
      timer = setTimeout(() => resolve(false), budgetMs)
    }),
  ])
  if (timer !== undefined) clearTimeout(timer)
  return outcome
}

async function stopOwnedProcess(label: string, proc: Bun.Subprocess) {
  if (proc.exitCode === null) proc.kill('SIGTERM')
  if (await exitedWithin(proc, 5_000)) return

  if (proc.exitCode === null) proc.kill('SIGKILL')
  if (!(await exitedWithin(proc, 5_000))) {
    throw new Error(`${label} did not exit after SIGTERM and SIGKILL`)
  }
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

  // Probe immediately before spawning to narrow the bind race. Readiness additionally proves
  // that the spawned gateway PID owns the IPv4 loopback listening socket.
  assertPortAvailable('127.0.0.1', 4179)
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
    if (!gateway || gateway.exitCode !== null || !runnerOwnsGatewayListener(gateway, 4179)) return false
    try {
      const res = await fetch(`http://${addr}/api/healthz`)
      return res.ok && gateway.exitCode === null && runnerOwnsGatewayListener(gateway, 4179)
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

  // Run the exact same CaseworkPort assertions as localAdapter.test.ts, against this
  // authenticated HTTP adapter and the fresh kernel/gateway cell.
  const eventsFile = join(cell, 'cases', caseId, 'case-events.jsonl')
  const approvalsFile = join(cell, 'approvals.jsonl')
  let artifactRef = ''
  const runsRoot = join(cell, 'cases', caseId, 'runs')
  const findArtifactDigest = async (): Promise<string> => {
    await waitFor('the executed run to record an artifact digest', () => {
      if (!existsSync(runsRoot)) return false
      for (const entry of readdirSync(runsRoot, { withFileTypes: true })) {
        if (!entry.isDirectory()) continue
        const evidenceFile = join(runsRoot, entry.name, 'evidence.jsonl')
        if (!existsSync(evidenceFile)) continue
        for (const line of readFileSync(evidenceFile, 'utf8').split(/\r?\n/)) {
          if (!line.trim()) continue
          try {
            const record = JSON.parse(line) as { sha256?: unknown }
            if (typeof record.sha256 === 'string' && /^[0-9a-f]{64}$/.test(record.sha256)) {
              artifactRef = `sha256:${record.sha256}`
              return true
            }
          } catch {
            // Ignore non-JSON journal lines; only a recorded digest is an artifact locator.
          }
        }
      }
      return false
    }, 30_000)
    return artifactRef
  }
  const { runCaseworkPortConformance } = await import('../src/adapters/conformance/caseworkPortConformance')
  await runCaseworkPortConformance({
    port: adapter,
    caseId,
    actor: { actor_id: 'operator_local', role: 'operator' as never },
    template: {
      ref: 'e2e-sentry-chain@0.1.0',
      params: { dataset_name: 't08-live', dataset_label: 'live conformance', max_rows: 3, out_dir: 'work' },
    },
    artifact: {
      refAfterDispatch: findArtifactDigest,
      expectedProvenance: { case_id: caseId, plan_item_id: 'task_prepare' },
      digestMatchesContent: true,
    },
    initialHistoryMinimum: 1,
    resume: 'replay-retained',
    resumeWaitMs: 60,
    waitForMs: 30_000,
    acceptedIntent: (cursor) => ({
      intent_id: `t08-conformance-exec-${Date.now()}`,
      kind: 'CONSEQUENTIAL_CASE',
      action_name: 'EXECUTE_ITEM',
      target_object_id: 'task_prepare',
      case_id: caseId,
      client_cursor: cursor,
      actor: { actor_id: 'ignored', role: 'ignored' as never },
      parameters: { item_id: 'task_prepare' },
    } as never),
    captureConsequentialState: async () => JSON.stringify([eventsFile, approvalsFile].map((path) => ({
      path,
      exists: existsSync(path),
      bytes: existsSync(path) ? readFileSync(path).toString('base64') : null,
    }))),
    stateBoundaryDescription: 'case-events.jsonl and approvals.jsonl byte-for-byte contents',
  })
  check('the shared CaseworkPort suite passes against the real HTTP adapter', true)

  // Durable truth: the shared suite's accepted EXECUTE_ITEM produced a kernel ledger event.
  await waitFor('durable item_activated', () => readFileSync(eventsFile, 'utf8').includes('"item_activated"'))
  check('the kernel ledger recorded item_activated', true)

  // 10. The second user's session is distinct (T07 two-user surface).
  const second = new HttpCaseworkAdapter({ base: `http://${addr}` })
  const rso = await second.login('rso', 'whatever-dev-mode-skips')
  check('the R-SO session resolves its own kernel standing', rso.actor_id === 'rso_local', JSON.stringify(rso))
} catch (err) {
  failures++
  console.error('FATAL', err)
} finally {
  for (const [label, proc] of [['kernel', kernel], ['gateway', gateway]] as const) {
    if (!proc) continue
    try {
      await stopOwnedProcess(label, proc)
    } catch (err) {
      failures++
      console.error(`CLEANUP FAIL ${label}`, err)
    }
  }
}

if (failures > 0) {
  console.error(`live conformance evidence retained at ${cell}`)
  console.error(`live conformance: ${failures} failure(s)`)
  process.exit(1)
}
rmSync(runRoot, { recursive: true, force: true })
console.log('live conformance: all checks passed')
