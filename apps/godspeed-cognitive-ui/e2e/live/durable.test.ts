import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { appendFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import {
  badgeFor,
  foldStanding,
  kindsOf,
  listCases,
  parseJsonl,
  readSettlements,
  readTrace,
  waitForApproval,
  waitForKinds,
  waitForNewCase,
  waitForSettlement,
  waitUntil,
} from './durable'

let cell = ''
beforeEach(() => {
  cell = mkdtempSync(join(tmpdir(), 'durable-test-'))
})
afterEach(() => rmSync(cell, { recursive: true, force: true }))

const ev = (kind: string, extra: object = {}) => JSON.stringify({ kind, ...extra }) + '\n'
const caseFile = (id: string) => {
  mkdirSync(join(cell, 'cases', id), { recursive: true })
  return join(cell, 'cases', id, 'case-events.jsonl')
}

describe('parseJsonl', () => {
  test('parses lines, skips blanks, counts malformed and non-object lines', () => {
    const r = parseJsonl(ev('a') + '\n' + 'not json\n' + '[1]\n' + ev('b'))
    expect(kindsOf(r.records)).toEqual(['a', 'b'])
    expect(r.malformed).toBe(2)
  })
  test('flags a torn tail (no trailing newline) without hiding it', () => {
    expect(parseJsonl('{"kind":"a"}\n{"kind":').tornTail).toBe(true)
    expect(parseJsonl('{"kind":"a"}\n').tornTail).toBe(false)
    expect(parseJsonl('').tornTail).toBe(false)
  })
})

describe('readers', () => {
  test('missing files read as empty; listCases only counts dirs with case-events.jsonl', () => {
    expect(readTrace(cell, 'nope')).toEqual([])
    caseFile('case_a')
    writeFileSync(join(cell, 'cases', 'case_a', 'case-events.jsonl'), ev('case_created'))
    mkdirSync(join(cell, 'cases', 'empty'), { recursive: true })
    expect(listCases(cell)).toEqual(['case_a'])
  })
  test('readSettlements parses settlement.json per run and tolerates a corrupt one', () => {
    const runs = join(cell, 'cases', 'c', 'runs')
    mkdirSync(join(runs, 'r1'), { recursive: true })
    mkdirSync(join(runs, 'r2'), { recursive: true })
    writeFileSync(join(runs, 'r1', 'settlement.json'), '{"outcome":"ok"}')
    writeFileSync(join(runs, 'r2', 'settlement.json'), '{')
    const s = readSettlements(cell, 'c')
    expect(s).toEqual([{ run: 'r1', settlement: { outcome: 'ok' } }, { run: 'r2', settlement: null }])
  })
})

describe('waiting', () => {
  test('waitUntil times out with a message naming what was awaited', async () => {
    await expect(waitUntil('the thing', () => false, { timeoutMs: 150, pollMs: 30 })).rejects.toThrow(/timed out after 150ms waiting for the thing/)
  })
  test('waitForKinds resolves as soon as the kernel appends the kinds (watch-woken)', async () => {
    const f = caseFile('c1')
    writeFileSync(f, ev('case_created'))
    setTimeout(() => appendFileSync(f, ev('item_enabled')), 80)
    const t0 = Date.now()
    const trace = await waitForKinds(cell, 'c1', ['case_created', 'item_enabled'], { timeoutMs: 5000, pollMs: 2000 })
    expect(kindsOf(trace)).toEqual(['case_created', 'item_enabled'])
    expect(Date.now() - t0).toBeLessThan(1500) // woke on the fs event, not the 2s poll
  })
  test('waitForKinds fails when only some kinds arrive', async () => {
    writeFileSync(caseFile('c2'), ev('case_created'))
    await expect(waitForKinds(cell, 'c2', ['case_created', 'plan_mutated'], { timeoutMs: 200, pollMs: 40 })).rejects.toThrow(/plan_mutated/)
  })
  test('waitForNewCase ignores pre-existing cases and finds the new one', async () => {
    writeFileSync(caseFile('old'), ev('case_created') + ev('item_enabled'))
    setTimeout(() => writeFileSync(caseFile('new'), ev('case_created') + ev('item_enabled')), 60)
    const r = await waitForNewCase(cell, ['old'], ['case_created', 'item_enabled'], { timeoutMs: 5000, pollMs: 100 })
    expect(r.caseId).toBe('new')
  })
  test('waitForNewCase times out when the only case is the pre-existing one (stub gateway signature)', async () => {
    writeFileSync(caseFile('old'), ev('case_created'))
    await expect(waitForNewCase(cell, ['old'], ['case_created'], { timeoutMs: 200, pollMs: 40 })).rejects.toThrow(/new case/)
  })
  test('waitForApproval matches on principal fields', async () => {
    setTimeout(() => appendFileSync(join(cell, 'approvals.jsonl'), JSON.stringify({ approver: 'rso_local' }) + '\n'), 60)
    const recs = await waitForApproval(cell, (r) => r.approver === 'rso_local', { timeoutMs: 5000, pollMs: 100 })
    expect(recs.length).toBe(1)
  })
  test('waitForSettlement sees a nested settlement.json', async () => {
    setTimeout(() => {
      mkdirSync(join(cell, 'cases', 'c', 'runs', 'r'), { recursive: true })
      writeFileSync(join(cell, 'cases', 'c', 'runs', 'r', 'settlement.json'), '{"a":1}')
    }, 60)
    const s = await waitForSettlement(cell, 'c', { timeoutMs: 5000, pollMs: 100 })
    expect(s[0]!.run).toBe('r')
  })
})

describe('foldStanding / badgeFor', () => {
  const plan = { items: [{ plan_item_id: 'a' }, { plan_item_id: 'b' }] }
  const t = (kind: string, id: string | null, payload: object = {}) => ({ kind, plan_item_id: id, payload })
  test('pending until an execution event advances it; settlement stays disjoint from execution', () => {
    expect(foldStanding(plan, [])).toEqual({
      a: { execution: 'pending', settlement: 'unsettled' },
      b: { execution: 'pending', settlement: 'unsettled' },
    })
    const s = foldStanding(plan, [t('item_enabled', 'a'), t('item_activated', 'a'), t('item_completed', 'a'), t('item_enabled', 'zzz')])
    expect(s.a).toEqual({ execution: 'completed', settlement: 'unsettled' })
    expect(badgeFor('sandboxed_task', s.a!)).toBe('Executed; settlement pending')
    expect(badgeFor('sandboxed_task', s.b!)).toBe('Waiting')
  })
  test('accepted settlement comes only from the record status', () => {
    const s = foldStanding(plan, [t('item_enabled', 'a'), t('settlement_recorded', 'a', { status: 'accepted' }), t('item_completed', 'a')])
    expect(badgeFor('sandboxed_task', s.a!)).toBe('Settled accepted')
    expect(badgeFor('sandboxed_task', { execution: 'enabled', settlement: 'unsettled' })).toBe('Ready')
    expect(badgeFor('sandboxed_task', { execution: 'active', settlement: 'unsettled' })).toBe('Running')
  })
})
