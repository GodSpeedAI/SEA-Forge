import { describe, expect, test } from 'bun:test'
import { renderToStaticMarkup } from 'react-dom/server'
import { createElement } from 'react'
import { GroundedAnswer } from '../ui/GroundedAnswer'
import { createStore, initialState } from '../model/store'
import { projectHistory } from '../ports/project'
import { buildNorthstarHistory } from '../adapters/local/northstarData'
import { explain } from './conduct'
import type { ThothAnswerView, ThothAskRequest } from '../ports/contract'
import { beatsOfAnswer, createThothNarration, parseThothQuestion } from './thothNarration'

const answer = (over: Partial<ThothAnswerView> = {}): ThothAnswerView => ({
  answer_id: 'tha_1',
  question_id: 'thq_1',
  disposition: 'answered',
  claims: [
    { claim_id: 'c1', claim_class: 'declared_capability', subject: 'ApprovalRequest', status: 'declared', statement: 'ApprovalRequest is declared.', snapshot_ref: 'smsnap_1', evidence_refs: ['ev:1'], settlement_refs: ['st:1'], capability_record_ref: 'cap:1' },
  ],
  omitted_claim_classes: ['environment_status'],
  snapshot_ref: 'smsnap_1',
  freshness: 'current',
  assurance: 'local_tamper_evident',
  limitations: ['model is stale'],
  authority_notice: 'This answer confers no execution authority.',
  answered_at: '2026-10-10T00:00:00Z',
  ...over,
})

describe('parseThothQuestion', () => {
  test('maps typed phrasings onto the nine kernel kinds and keeps subject case', () => {
    expect(parseThothQuestion('ask capability ApprovalRequest', 'case-1')).toEqual({ ok: true, request: { kind: 'ask_capability', subject: 'ApprovalRequest', case: 'case-1' } })
    expect(parseThothQuestion('Ask why denied  X y', '')).toEqual({ ok: true, request: { kind: 'ask_why_denied', subject: 'X y' } })
    expect(parseThothQuestion('ask_projection_support S', '')).toMatchObject({ ok: true, request: { kind: 'ask_projection_support' } })
  })
  test('free text and missing subjects are honest refusals, never guesses', () => {
    expect(parseThothQuestion('Why did the pilot fail?', '').ok).toBe(false)
    expect(parseThothQuestion('ask capability', '').ok).toBe(false)
  })
})

describe('beatsOfAnswer', () => {
  test('claims become beats from returned statements; citations are only returned refs; a terms beat closes', () => {
    const beats = beatsOfAnswer(answer())
    expect(beats.length).toBe(2)
    expect(beats[0]!.thoughtText).toBe('ApprovalRequest is declared.')
    expect(beats[0]!.evidenceCitations).toEqual(['ev:1', 'st:1', 'cap:1'])
    expect(beats[0]!.directives).toBeUndefined()
    expect(beats[1]!.thoughtText).toContain('confers no execution authority')
    expect(beats.every((b) => b.grounded_answer?.answer_id === 'tha_1')).toBe(true)
  })
  test('a denied answer says so with fixed copy and no citations', () => {
    const beats = beatsOfAnswer(answer({ disposition: 'denied', claims: [] }))
    expect(beats[0]!.thoughtText).toContain('denied')
    expect(beats.flatMap((b) => b.evidenceCitations)).toEqual([])
  })
})

describe('live narration through explain()', () => {
  const mk = () => {
    const history = projectHistory(buildNorthstarHistory({ actor_id: 'usr-sam', role: 'case_architect' }), {}, 'local-contract', { withCore: false })
    return createStore(initialState(history))
  }
  test('asks once, streams beats, and interrupt() before the answer ends the stream without error', async () => {
    const calls: ThothAskRequest[] = []
    const port = createThothNarration({ ask: async (r) => (calls.push(r), answer()) })
    const store = mk()
    expect(explain(store, port, 'ask capability ApprovalRequest')).toBe(true)
    await new Promise((r) => setTimeout(r, 20))
    const n = Object.values(store.getState().narratives)[0]!
    expect(calls.length).toBe(1)
    expect(n.beats.length).toBe(2)
    expect(n.beats[0]!.grounded?.answer_id).toBe('tha_1')
    expect(store.getState().agent).toBe('available')
  })
  test('an interrupted stream yields no beats and no failure', async () => {
    const port = createThothNarration({ ask: (_r, signal) => new Promise((_res, rej) => signal?.addEventListener('abort', () => rej(new DOMException('aborted', 'AbortError')))) })
    const stream = port.explain('ask capability X', { caseId: '', focus: null, cursor: 'r0' })!
    const it = stream.beats[Symbol.asyncIterator]()
    const next = it.next()
    stream.interrupt()
    expect((await next).done).toBe(true)
  })
  test('a transport failure marks the agent unavailable, a typed refusal is an honest beat', async () => {
    const store = mk()
    explain(store, createThothNarration({ ask: async () => { throw new Error('network down') } }), 'ask capability X')
    await new Promise((r) => setTimeout(r, 20))
    expect(store.getState().agent).toBe('unavailable')
    const refusal = Object.assign(new Error('Too many Ask requests'), { name: 'HttpRefusalError', refusalKind: 'rate_limited' })
    const store2 = mk()
    explain(store2, createThothNarration({ ask: async () => { throw refusal } }), 'ask capability X')
    await new Promise((r) => setTimeout(r, 20))
    expect(store2.getState().agent).toBe('available')
    expect(Object.values(store2.getState().narratives)[0]!.beats[0]!.caption).toContain('Too many Ask requests')
  })
})

describe('GroundedAnswer', () => {
  test('renders every disclosure field verbatim', () => {
    const html = renderToStaticMarkup(createElement(GroundedAnswer, { answer: answer(), activeClaim: 0, paused: false }))
    for (const s of ['answered', 'current', 'local_tamper_evident', 'smsnap_1', 'ApprovalRequest is declared.', 'ev:1', 'st:1', 'cap:1', 'environment_status', 'model is stale', 'confers no execution authority']) {
      expect(html).toContain(s)
    }
  })
})
