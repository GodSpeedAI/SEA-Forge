// Live stack for the T10 ladder: a FRESH temp cell per run, the production UI build served by the
// gateway (serve.static_root), the real kernel behind it. Kill/restart helpers support the
// recovery journeys. Nothing here touches the shared persistent cell (.sea-forge/casework-live).

import {
  appendFileSync,
  closeSync,
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  rmSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { assetRefs, scanSources, type BundleScan } from './bundle'

export const uiDir = resolve(import.meta.dir, '..', '..')
export const repo = resolve(uiDir, '..', '..')
export const distDir = join(uiDir, 'dist')
export const kernelBin = join(repo, 'target', 'debug', 'sea-forge-server')
export const ADDR = '127.0.0.1:4179'
export const PROXY_ADDR = '127.0.0.1:4181' // recovery journey: severable network between browser and gateway
export const UPSTREAM_ADDR = '127.0.0.1:4180' // real gateway when the stub-gateway tooth fronts it

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))

export type Tooth = 'stub-gateway'

export interface StackOpts {
  /** Where logs and the durable snapshot are preserved. */
  evidenceDir: string
  tooth?: Tooth
  /** Skip `bun run build` (dist must already be a production build). */
  skipBuild?: boolean
}

export interface Stack {
  root: string
  cell: string
  base: string
  evidenceDir: string
  /** Rebuilds the cell's self-model (once) and returns a concept Thoth can be asked about. */
  seedSelfModel(): string
  bundle: BundleScan
  servedBundle: BundleScan
  tooth?: Tooth
  killKernel(sig?: NodeJS.Signals): Promise<void>
  startKernel(): Promise<void>
  restartKernel(): Promise<void>
  killGateway(sig?: NodeJS.Signals): Promise<void>
  startGateway(): Promise<void>
  restartGateway(): Promise<void>
  /** Base URL of a severable pass-through to the gateway (recovery journeys). */
  proxyBase: string
  startProxy(): Promise<void>
  killProxy(): Promise<void>
  /** Stop processes, preserve logs + durable snapshot under evidenceDir, remove the temp cell on `clean`. */
  down(opts: { clean: boolean }): Promise<void>
}

function assertPortAvailable(hostname: string, port: number) {
  let probe: Bun.TCPSocketListener<undefined>
  try {
    probe = Bun.listen({ hostname, port, socket: { data() {} } })
  } catch (err) {
    throw new Error(`port ${hostname}:${port} is already in use; leaving its owner untouched`, { cause: err })
  }
  probe.stop(true)
}

async function waitFor(what: string, probe: () => boolean | Promise<boolean>, budgetMs = 30_000) {
  const deadline = Date.now() + budgetMs
  while (Date.now() < deadline) {
    if (await probe()) return
    await sleep(150)
  }
  throw new Error(`timed out waiting for ${what}`)
}

async function socketAccepts(path: string): Promise<boolean> {
  try {
    const s = await Bun.connect({ unix: path, socket: { data() {}, open() {}, close() {}, error() {} } })
    s.end()
    return true
  } catch {
    return false
  }
}

async function exitedWithin(proc: Bun.Subprocess, ms: number): Promise<boolean> {
  let timer: ReturnType<typeof setTimeout> | undefined
  const r = await Promise.race([
    proc.exited.then(() => true),
    new Promise<boolean>((res) => {
      timer = setTimeout(() => res(false), ms)
    }),
  ])
  if (timer) clearTimeout(timer)
  return r
}

async function stopProc(label: string, proc: Bun.Subprocess, sig: NodeJS.Signals = 'SIGTERM') {
  if (proc.exitCode === null) proc.kill(sig)
  if (await exitedWithin(proc, 5_000)) return
  if (proc.exitCode === null) proc.kill('SIGKILL')
  if (!(await exitedWithin(proc, 5_000))) throw new Error(`${label} did not exit after ${sig} and SIGKILL`)
}

function ownsListener(proc: Bun.Subprocess, port: number): boolean {
  if (proc.exitCode !== null) return false
  const local = `0100007F:${port.toString(16).toUpperCase().padStart(4, '0')}`
  try {
    const inodes = new Set<string>()
    for (const fd of readdirSync(`/proc/${proc.pid}/fd`)) {
      try {
        const m = /^socket:\[(\d+)\]$/.exec(readlinkSync(`/proc/${proc.pid}/fd/${fd}`))
        if (m) inodes.add(m[1]!)
      } catch {}
    }
    const listening = readFileSync('/proc/net/tcp', 'utf8')
      .split(/\r?\n/)
      .map((l) => l.trim().split(/\s+/))
      .filter((f) => f[1]?.toUpperCase() === local && f[3] === '0A')
      .map((f) => f[9])
    return listening.some((i) => i && inodes.has(i))
  } catch {
    return false
  }
}

/** Build the production UI and prove the bundle on disk is free of fixture machinery. */
export function buildAndScanDist(skipBuild: boolean): BundleScan {
  if (!skipBuild) {
    const r = Bun.spawnSync(['bun', 'run', 'build'], { cwd: uiDir, stdout: 'pipe', stderr: 'pipe' })
    if (r.exitCode !== 0) throw new Error(`UI production build failed:\n${new TextDecoder().decode(r.stderr)}`)
  }
  const dir = join(distDir, 'assets')
  if (!existsSync(dir)) throw new Error(`no ${dir}; build the UI first`)
  const files: Record<string, string> = {}
  for (const f of readdirSync(dir)) if (f.endsWith('.js')) files[f] = readFileSync(join(dir, f), 'utf8')
  const scan = scanSources(files)
  if (!scan.ok) throw new Error(`bundle scan (disk) failed: ${scan.problems.join('; ')}`)
  return scan
}

/** Fetch every served JS asset over HTTP, require byte-equality with dist, and re-scan. */
export async function scanServed(base: string): Promise<BundleScan> {
  const html = await (await fetch(`${base}/`)).text()
  const seen = new Set<string>()
  const files: Record<string, string> = {}
  const queue = assetRefs(html)
  // Walk the static import graph over HTTP, plus every dist chunk so lazy chunks are covered too.
  for (const f of readdirSync(join(distDir, 'assets'))) if (f.endsWith('.js')) queue.push(f)
  while (queue.length) {
    const f = queue.pop()!
    if (seen.has(f)) continue
    seen.add(f)
    const res = await fetch(`${base}/assets/${f}`)
    if (!res.ok) throw new Error(`served /assets/${f} -> HTTP ${res.status}`)
    const body = await res.text()
    const disk = readFileSync(join(distDir, 'assets', f), 'utf8')
    if (body !== disk) throw new Error(`served /assets/${f} differs from dist on disk`)
    files[f] = body
    for (const r of assetRefs(body)) if (!seen.has(r)) queue.push(r)
  }
  const scan = scanSources(files)
  if (!scan.ok) throw new Error(`bundle scan (served over HTTP) failed: ${scan.problems.join('; ')}`)
  return scan
}

export async function bootStack(opts: StackOpts): Promise<Stack> {
  if (!existsSync(kernelBin)) throw new Error('kernel binary missing; run: cargo build -p sea-forge-server --bin sea-forge-server')
  const bundle = buildAndScanDist(!!opts.skipBuild)

  const uid = process.getuid?.() ?? 1000
  const root = mkdtempSync(join(tmpdir(), 't10-live-'))
  const cell = join(root, 'cell')
  const logs = join(root, 'logs')
  mkdirSync(join(cell, 'authority'), { recursive: true })
  mkdirSync(logs, { recursive: true })
  mkdirSync(opts.evidenceDir, { recursive: true })
  const gatewayBin = join(root, 'godspeed-casework')

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
  // The e2e policy plus the self-disclosure grant the L8 Thoth Ask needs (deny-by-default otherwise:
  // without a grant the kernel answers `denied`, which is a valid governed answer but has no claims).
  copyFileSync(join(repo, 'fixtures', 'cells', 'e2e', 'policy.yaml'), join(cell, 'authority', 'active-policy.json'))
  appendFileSync(
    join(cell, 'authority', 'active-policy.json'),
    `
policy_surfaces:
  self_disclosure:
    mode: deny-by-default
    grants:
      - name: live-ladder-ask-grant
        actor_role: operator_local
        claim_classes:
          - declared_capability
          - installed_capability
          - demonstrated_capability
`,
  )
  mkdirSync(join(cell, 'templates'), { recursive: true })
  for (const tpl of ['e2e-sentry-chain@0.1.0.yaml', 'e2e-signoff-gate@0.1.0.yaml']) {
    copyFileSync(join(repo, 'fixtures', 'cells', 'e2e', 'templates', tpl), join(cell, 'templates', tpl))
  }

  // Thoth answers from the kernel's self-model snapshot; a fresh cell has none until the operator
  // rebuilds it (the documented `sea-forge self-model rebuild`). That is done by L8, not at boot: on a
  // cell with no extension registry the rebuilt snapshot is disclosed Stale and the kernel's own
  // readiness verdict then reports not_ready, which would fail L0's readiness check.
  const cliBin = join(repo, 'target', 'debug', 'sea-forge')
  if (!existsSync(cliBin)) throw new Error('sea-forge CLI missing; run: cargo build -p sea-forge-cli')
  let askSubject = ''
  const seedSelfModel = (): string => {
    if (askSubject) return askSubject
    const seeded = Bun.spawnSync([cliBin, 'self-model', '--root', cell, 'rebuild'])
    if (seeded.exitCode !== 0) throw new Error(`self-model rebuild failed:\n${new TextDecoder().decode(seeded.stderr)}`)
    const shown = Bun.spawnSync([cliBin, 'self-model', '--root', cell, 'show', '--json'])
    const subject = (JSON.parse(new TextDecoder().decode(shown.stdout)) as { system_model_ref?: { concept_refs?: string[] } }).system_model_ref?.concept_refs?.[0]
    if (!subject) throw new Error('the rebuilt self-model exposed no concept to ask about')
    askSubject = subject
    return askSubject
  }

  const gatewayAddr = opts.tooth === 'stub-gateway' ? UPSTREAM_ADDR : ADDR
  const config = {
    version: '1',
    evidence_root: join(root, 'evidence'),
    serve: {
      gateway_actor_id: 'gateway',
      gateway_role: 'service',
      policy_ref: 'authority/active-policy.json',
      perspective_actor_id: 'operator_local',
      perspective_role: 'operator',
      production: false,
      static_root: distDir,
      rate_limit: { intents_per_minute: 120, burst: 30 },
      // An approval lives as long as the execution that opened it (kernel default 60s). Browser-driven
      // sign-off by a second user needs minutes, not a minute.
      execution_timeout_sec: 900,
    },
    auth: {
      mode: 'dev',
      users: [
        { username: 'operator', display_name: 'Local Operator', actor_id: 'operator_local', role: 'operator' },
        { username: 'rso', display_name: 'R-SO', actor_id: 'rso_local', role: 'R-SO' },
      ],
    },
    capabilities: [
      // Absolute: the gateway resolves unix:// endpoints against its own cwd, not the cell root.
      { name: 'authority', kind: 'authority', required: false, adapter: 'sfwp', endpoint: `unix://${join(cell, 'server.sock')}` },
    ],
  }
  const configFile = join(cell, 'gateway.json')
  writeFileSync(configFile, JSON.stringify(config))

  const build = Bun.spawnSync(['go', 'build', '-o', gatewayBin, './cmd/godspeed-casework'], {
    cwd: join(repo, 'apps', 'godspeed-casework-go'),
  })
  if (build.exitCode !== 0) throw new Error(`gateway build failed:\n${new TextDecoder().decode(build.stderr)}`)

  assertPortAvailable('127.0.0.1', 4179)
  if (opts.tooth === 'stub-gateway') assertPortAvailable('127.0.0.1', 4180)

  let gateway: Bun.Subprocess | null = null
  let kernel: Bun.Subprocess | null = null
  let stub: Bun.Subprocess | null = null
  let proxy: Bun.Subprocess | null = null
  const openLog = (name: string) => openSync(join(logs, name), 'a')

  const startKernel = async () => {
    const fd = openLog('kernel.log')
    kernel = Bun.spawn([kernelBin], {
      env: { ...process.env, SEA_FORGE_ROOT: cell, SEA_FORGE_SOCKET: join(cell, 'server.sock') },
      stdout: fd,
      stderr: fd,
    })
    closeSync(fd)
    // After a kill -9 the old socket file is still on disk; the new kernel replaces it. Wait for a
    // connection to be ACCEPTED, not for the path to exist.
    await waitFor('the kernel socket to accept connections', async () => kernel!.exitCode === null && (await socketAccepts(join(cell, 'server.sock'))))
  }
  const startGateway = async () => {
    const [h, p] = gatewayAddr.split(':')
    assertPortAvailable(h!, Number(p))
    const fd = openLog('gateway.log')
    gateway = Bun.spawn([gatewayBin, '-serve', '-addr', gatewayAddr, '-config', configFile], {
      env: { ...process.env, GODSPEED_CELL_ROOT: cell },
      stdout: fd,
      stderr: fd,
    })
    closeSync(fd)
    await waitFor('gateway healthz', async () => {
      if (!gateway || gateway.exitCode !== null || !ownsListener(gateway, Number(p))) return false
      try {
        return (await fetch(`http://${gatewayAddr}/api/healthz`)).ok
      } catch {
        return false
      }
    })
  }

  const startStub = async () => {
    // Out-of-process: the ladder blocks this process's event loop in spawnSync browser calls.
    stub = Bun.spawn(['bun', join(import.meta.dir, 'stub-gateway.ts')], {
      env: { ...process.env, STUB_LISTEN_PORT: '4179', STUB_UPSTREAM: UPSTREAM_ADDR },
      stdout: 'inherit',
      stderr: 'inherit',
    })
    await waitFor('the stub gateway', async () => {
      try {
        return (await fetch(`http://${ADDR}/api/healthz`)).ok
      } catch {
        return false
      }
    })
  }

  const stack: Stack = {
    root,
    cell,
    base: `http://${ADDR}`,
    evidenceDir: opts.evidenceDir,
    seedSelfModel,
    bundle,
    servedBundle: bundle,
    tooth: opts.tooth,
    async killKernel(sig = 'SIGKILL') {
      if (kernel) await stopProc('kernel', kernel, sig)
    },
    startKernel,
    async restartKernel() {
      await stack.killKernel('SIGTERM')
      await startKernel()
    },
    async killGateway(sig = 'SIGKILL') {
      if (gateway) await stopProc('gateway', gateway, sig)
    },
    startGateway,
    async restartGateway() {
      await stack.killGateway('SIGTERM')
      await startGateway()
    },
    proxyBase: `http://${PROXY_ADDR}`,
    async startProxy() {
      const [h, p] = PROXY_ADDR.split(':')
      assertPortAvailable(h!, Number(p))
      proxy = Bun.spawn(['bun', join(import.meta.dir, 'netproxy.ts')], {
        env: { ...process.env, PROXY_LISTEN_PORT: p!, PROXY_UPSTREAM: gatewayAddr },
        stdout: 'inherit',
        stderr: 'inherit',
      })
      await waitFor('the network proxy', async () => {
        try {
          return (await fetch(`http://${PROXY_ADDR}/api/healthz`)).ok
        } catch {
          return false
        }
      })
    },
    async killProxy() {
      if (proxy) await stopProc('proxy', proxy, 'SIGKILL')
    },
    async down({ clean }) {
      const errs: string[] = []
      for (const [label, p] of [['proxy', proxy], ['stub', stub], ['gateway', gateway], ['kernel', kernel]] as const) {
        if (!p) continue
        try {
          await stopProc(label, p)
        } catch (e) {
          errs.push(String(e))
        }
      }
      // Preserve evidence: logs + the durable text/JSON files (no sockets, no artifact blobs).
      try {
        cpSync(logs, join(opts.evidenceDir, 'logs'), { recursive: true })
        cpSync(cell, join(opts.evidenceDir, 'cell-durable'), {
          recursive: true,
          filter: (src) => !src.endsWith('.sock') && !src.endsWith('.lock') && !/\.(png|bin)$/.test(src),
        })
      } catch (e) {
        errs.push(`evidence copy: ${e}`)
      }
      if (clean && errs.length === 0) rmSync(root, { recursive: true, force: true })
      else if (errs.length) throw new Error(errs.join('; '))
    },
  }

  try {
    if (opts.tooth === 'stub-gateway') {
      await startKernel()
      await startGateway()
      await startStub()
    } else {
      await startKernel()
      await startGateway()
    }
    stack.servedBundle = await scanServed(stack.base)
  } catch (e) {
    await stack.down({ clean: false }).catch(() => {})
    throw e
  }
  return stack
}
