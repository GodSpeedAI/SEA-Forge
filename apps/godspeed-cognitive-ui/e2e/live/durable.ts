// Durable-state readers for the live ladder (T10). Every assertion reads the kernel's own files:
//   cases/<case_id>/case-events.jsonl   (TraceEvent per line, `kind` is snake_case)
//   approvals.jsonl                     (cell root)
//   cases/<case_id>/runs/<run>/settlement.json
// Waits are event-driven (fs.watch wakes the probe) with a slow poll only as a safety net.

import { existsSync, readFileSync, readdirSync, watch } from 'node:fs'
import { dirname, join } from 'node:path'

export interface TraceRecord {
  event_id?: string
  run_id?: string
  plan_item_id?: string | null
  kind: string
  actor_id?: string
  timestamp?: string
  payload?: unknown
  [k: string]: unknown
}

export interface JsonlParse {
  records: Record<string, unknown>[]
  malformed: number
  /** A final line with no trailing newline may be a torn write; it is reported, not dropped silently. */
  tornTail: boolean
}

/** Parse JSON-lines text. Blank lines are skipped; unparsable lines are counted, never guessed at. */
export function parseJsonl(text: string): JsonlParse {
  const records: Record<string, unknown>[] = []
  let malformed = 0
  const lines = text.split('\n')
  const tornTail = text.length > 0 && !text.endsWith('\n')
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!.trim()
    if (!line) continue
    try {
      const v = JSON.parse(line)
      if (v && typeof v === 'object' && !Array.isArray(v)) records.push(v as Record<string, unknown>)
      else malformed++
    } catch {
      malformed++
    }
  }
  return { records, malformed, tornTail }
}

export function readJsonl(file: string): JsonlParse {
  if (!existsSync(file)) return { records: [], malformed: 0, tornTail: false }
  return parseJsonl(readFileSync(file, 'utf8'))
}

export const caseEventsFile = (cell: string, caseId: string) => join(cell, 'cases', caseId, 'case-events.jsonl')
export const approvalsFile = (cell: string) => join(cell, 'approvals.jsonl')

export function readTrace(cell: string, caseId: string): TraceRecord[] {
  return readJsonl(caseEventsFile(cell, caseId)).records as TraceRecord[]
}

export function kindsOf(records: { kind?: unknown }[]): string[] {
  return records.map((r) => String(r.kind))
}

/** Case ids present in the cell (directories under cases/ that have a case-events.jsonl). */
export function listCases(cell: string): string[] {
  const dir = join(cell, 'cases')
  if (!existsSync(dir)) return []
  return readdirSync(dir, { withFileTypes: true })
    .filter((e) => e.isDirectory() && existsSync(join(dir, e.name, 'case-events.jsonl')))
    .map((e) => e.name)
    .sort()
}

/** settlement.json files under cases/<id>/runs/<run>/, parsed. */
export function readSettlements(cell: string, caseId: string): { run: string; settlement: unknown }[] {
  const runs = join(cell, 'cases', caseId, 'runs')
  if (!existsSync(runs)) return []
  const out: { run: string; settlement: unknown }[] = []
  for (const e of readdirSync(runs, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const f = join(runs, e.name, 'settlement.json')
    if (e.isDirectory() && existsSync(f)) {
      try {
        out.push({ run: e.name, settlement: JSON.parse(readFileSync(f, 'utf8')) })
      } catch {
        out.push({ run: e.name, settlement: null })
      }
    }
  }
  return out
}

export interface WaitOpts {
  timeoutMs?: number
  /** Files/dirs whose changes should wake the probe immediately. */
  watch?: string[]
  /** Safety-net poll (ms) for filesystems/paths fs.watch cannot observe (e.g. dir not yet created). */
  pollMs?: number
}

/**
 * Resolve with the first non-undefined/false probe result; reject on timeout. fs.watch on the
 * nearest existing ancestor wakes the probe the moment the kernel writes; the poll only covers
 * paths that do not exist yet or platforms that drop events.
 */
export async function waitUntil<T>(
  what: string,
  probe: () => T | false | undefined | null,
  opts: WaitOpts = {},
): Promise<T> {
  const timeoutMs = opts.timeoutMs ?? 30_000
  const pollMs = opts.pollMs ?? 500
  const first = probe()
  if (first) return first
  return new Promise<T>((resolve, reject) => {
    const watchers: ReturnType<typeof watch>[] = []
    let wake: (() => void) | null = null
    let done = false
    const finish = (fn: () => void) => {
      if (done) return
      done = true
      for (const w of watchers) w.close()
      clearTimeout(deadline)
      clearInterval(poll)
      fn()
    }
    const check = () => {
      if (done) return
      try {
        const v = probe()
        if (v) finish(() => resolve(v as T))
      } catch (e) {
        finish(() => reject(e))
      }
    }
    wake = check
    for (const target of opts.watch ?? []) {
      let p = target
      while (!existsSync(p) && dirname(p) !== p) p = dirname(p)
      try {
        watchers.push(watch(p, () => wake?.()))
      } catch {
        // fall back to the poll
      }
    }
    const poll = setInterval(check, pollMs)
    const deadline = setTimeout(() => finish(() => reject(new Error(`timed out after ${timeoutMs}ms waiting for ${what}`))), timeoutMs)
  })
}

/** Wait until the given trace kinds (all) appear in the case's case-events.jsonl; returns the trace. */
export async function waitForKinds(
  cell: string,
  caseId: string,
  kinds: string[],
  opts: WaitOpts = {},
): Promise<TraceRecord[]> {
  const file = caseEventsFile(cell, caseId)
  return waitUntil(
    `kinds [${kinds.join(', ')}] in ${file}`,
    () => {
      const trace = readTrace(cell, caseId)
      const have = new Set(kindsOf(trace))
      return kinds.every((k) => have.has(k)) ? trace : false
    },
    { ...opts, watch: [file, dirname(file), ...(opts.watch ?? [])] },
  )
}

/** Wait for a NEW case (not in `before`) whose trace contains all `kinds`. */
export async function waitForNewCase(
  cell: string,
  before: string[],
  kinds: string[],
  opts: WaitOpts = {},
): Promise<{ caseId: string; trace: TraceRecord[] }> {
  const known = new Set(before)
  return waitUntil(
    `a new case with kinds [${kinds.join(', ')}] under ${cell}/cases`,
    () => {
      for (const id of listCases(cell)) {
        if (known.has(id)) continue
        const trace = readTrace(cell, id)
        const have = new Set(kindsOf(trace))
        if (kinds.every((k) => have.has(k))) return { caseId: id, trace }
      }
      return false
    },
    { ...opts, watch: [join(cell, 'cases'), ...(opts.watch ?? [])] },
  )
}

/** Wait for approvals.jsonl to hold a record matching `pred`. */
export async function waitForApproval(
  cell: string,
  pred: (r: Record<string, unknown>) => boolean,
  opts: WaitOpts = {},
): Promise<Record<string, unknown>[]> {
  const file = approvalsFile(cell)
  return waitUntil(
    `matching record in ${file}`,
    () => {
      const recs = readJsonl(file).records
      return recs.some(pred) ? recs : false
    },
    { ...opts, watch: [file, cell, ...(opts.watch ?? [])] },
  )
}

export async function waitForSettlement(cell: string, caseId: string, opts: WaitOpts = {}) {
  return waitUntil(
    `settlement.json for ${caseId}`,
    () => {
      const s = readSettlements(cell, caseId)
      return s.length > 0 ? s : false
    },
    { ...opts, watch: [join(cell, 'cases', caseId, 'runs'), join(cell, 'cases', caseId), ...(opts.watch ?? [])] },
  )
}

export interface PlanItem {
  plan_item_id: string
  name: string
  item_kind: string
  operations?: { kind: string; path?: string; content_hint?: string }[]
  proposed_by?: string
  depends_on?: string[]
  [k: string]: unknown
}

/** The case's durable plan (cases/<id>/plan.json). */
export function readPlan(cell: string, caseId: string): { items: PlanItem[] } {
  return JSON.parse(readFileSync(join(cell, 'cases', caseId, 'plan.json'), 'utf8')) as { items: PlanItem[] }
}

export type ExecutionStanding = 'pending' | 'enabled' | 'active' | 'completed' | 'failed' | 'terminated'
export type SettlementStanding = 'unsettled' | 'accepted' | 'rejected' | 'escalated'
export interface Standing {
  execution: ExecutionStanding
  settlement: SettlementStanding
}

/**
 * An independent fold of the durable trace into per-item standing (the same disjoint rules the
 * kernel's case.get_horizon applies: execution advances only on execution events, settlement only
 * from a settlement_recorded record's own status). The ladder compares this with what the UI shows.
 */
export function foldStanding(plan: { items: { plan_item_id: string }[] }, trace: TraceRecord[]): Record<string, Standing> {
  const rows: Record<string, Standing> = {}
  for (const it of plan.items) rows[it.plan_item_id] = { execution: 'pending', settlement: 'unsettled' }
  for (const ev of trace) {
    const id = ev.plan_item_id
    const row = id ? rows[id] : undefined
    if (!row) continue
    switch (ev.kind) {
      case 'item_enabled': row.execution = 'enabled'; break
      case 'item_activated': row.execution = 'active'; break
      case 'item_completed':
      case 'human_task_completed': row.execution = 'completed'; break
      case 'item_failed': row.execution = 'failed'; break
      case 'item_terminated': row.execution = 'terminated'; break
      case 'settlement_recorded': {
        const status = (ev.payload as { status?: string } | undefined)?.status
        if (status === 'accepted' || status === 'rejected' || status === 'escalated') row.settlement = status
        break
      }
    }
  }
  return rows
}

/** The badge the gateway projects for a standing (projection/builder.go cognitiveStatus). */
export function badgeFor(kind: string, s: Standing): string {
  switch (s.execution) {
    case 'failed': return 'Failed'
    case 'terminated': return 'Terminated'
    case 'active': return kind === 'human_task' ? 'Waiting on your decision' : 'Running'
    case 'enabled': return 'Ready'
    case 'completed':
      return s.settlement === 'accepted' ? 'Settled accepted'
        : s.settlement === 'rejected' ? 'Settlement rejected'
        : s.settlement === 'escalated' ? 'Awaiting approval'
        : 'Executed; settlement pending'
    default: return 'Waiting'
  }
}

/** trace.jsonl records of one run directory (authority_evaluated, workspace_created, ...). */
export function readRunTrace(cell: string, caseId: string, runId: string): TraceRecord[] {
  return readJsonl(join(cell, 'cases', caseId, 'runs', runId, 'trace.jsonl')).records as TraceRecord[]
}

export function listRuns(cell: string, caseId: string): string[] {
  const dir = join(cell, 'cases', caseId, 'runs')
  if (!existsSync(dir)) return []
  return readdirSync(dir, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name).sort()
}

export interface LedgerEntry {
  record_kind: string
  subject_refs?: string[]
  payload: Record<string, unknown>
  [k: string]: unknown
}

/** Entries of ledgers/<ledgerId>/entries.jsonl ("case-<case_id>", "delegation-audit", "events"). */
export function readLedger(cell: string, ledgerId: string): LedgerEntry[] {
  return readJsonl(join(cell, 'ledgers', ledgerId, 'entries.jsonl')).records as unknown as LedgerEntry[]
}

/** Who the gateway acted for on each delegated kernel request: {verb, effective_actor_id, effective_role}. */
export function delegatedRequests(cell: string): { verb: string; actor: string; role: string }[] {
  return readLedger(cell, 'delegation-audit')
    .filter((e) => e.record_kind === 'delegated_request')
    .map((e) => ({ verb: String(e.payload.verb), actor: String(e.payload.effective_actor_id), role: String(e.payload.effective_role) }))
}
