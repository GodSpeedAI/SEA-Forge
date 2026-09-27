import { describe, expect, test } from 'bun:test'
import type { XObject } from '../ports/contract'
import { presentationOf, sentryExplanation, statusPhrase, statusTone, toneOf } from './derive'

// The presentation mapping table (T09, binding decision D-PRESENTATION): kernel fields → the
// UI's presentation. Live snapshots carry no `x` extensions, so this is what the journey UI reads.

const obj = (over: Partial<XObject>): XObject => ({
  id: 'x',
  kind: 'work_item',
  name: 'X',
  status: 'IN_PROGRESS',
  badge: '',
  salience: 0.5,
  actions: [],
  ...over,
})

describe('derive: kernel kind → presentation', () => {
  test('the kernel kinds map to the UI presentation classes', () => {
    expect(presentationOf('work_item')).toBe('item')
    expect(presentationOf('stage')).toBe('facet')
    expect(presentationOf('milestone')).toBe('facet')
    expect(presentationOf('decision_gate')).toBe('facet')
    expect(presentationOf('discretionary_opportunity')).toBe('item')
    expect(presentationOf('execution_trace')).toBe('run')
    expect(presentationOf('evidence_record')).toBe('item')
    expect(presentationOf('something_new')).toBe('item') // unknown kinds read as items
  })
})

describe('derive: kernel status → tone and phrase', () => {
  test('ACTIVE/ready work is progress; COMPLETED is ok; BLOCKED/WAITING is attention', () => {
    expect(statusTone('ACTIVE')).toBe('progress')
    expect(statusTone('IN_PROGRESS')).toBe('progress')
    expect(statusTone('READY_TO_BEGIN')).toBe('progress')
    expect(statusTone('COMPLETED')).toBe('ok')
    expect(statusTone('BLOCKED')).toBe('attention')
    expect(statusTone('WAITING')).toBe('attention')
    expect(statusTone('ACTION_REQUIRED')).toBe('attention')
    expect(statusTone('WAITING_ON_OTHERS')).toBe('muted')
    expect(statusTone('FAILED')).toBe('critical')
    expect(statusTone('REJECTED')).toBe('critical')
    expect(statusTone('AVAILABLE_TO_ADD')).toBe('hypothesis')
    expect(statusTone('SOMETHING_ELSE')).toBe('muted')
  })

  test('phrases replace engine jargon only when the backend sent no badge', () => {
    expect(statusPhrase('ACTION_REQUIRED')).toBe('Needs your attention')
    expect(statusPhrase('AVAILABLE_TO_ADD')).toBe('Available to add if needed')
    expect(statusPhrase('COMPLETED')).toBe('Completed')
    expect(statusPhrase('BLOCKED')).toBe('Blocked')
  })

  test('an authored tone extension wins over derivation', () => {
    expect(toneOf(obj({ status: 'COMPLETED' }))).toBe('ok')
    expect(toneOf(obj({ status: 'COMPLETED', x: { tone: 'critical' } }))).toBe('critical')
  })
})

describe('derive: depends_on → sentry explanation', () => {
  const peers: Record<string, XObject> = {
    upstream: obj({ id: 'upstream', name: 'Rollout', status: 'IN_PROGRESS' }),
    done: obj({ id: 'done', name: 'Evidence', status: 'COMPLETED' }),
  }

  test('unmet dependencies are named with their standing', () => {
    expect(sentryExplanation(obj({ depends_on: ['upstream'] }), peers)).toBe('Waiting on Rollout (in progress)')
  })

  test('completed dependencies never block', () => {
    expect(sentryExplanation(obj({ depends_on: ['done'] }), peers)).toBeUndefined()
  })

  test('multiple blockers are listed together', () => {
    const more = { ...peers, other: obj({ id: 'other', name: 'Sign-off', status: 'WAITING' }) }
    expect(sentryExplanation(obj({ depends_on: ['upstream', 'other'] }), more)).toBe(
      'Waiting on Rollout (in progress), Sign-off (waiting)',
    )
  })

  test('settled work is not sentry-blocked, and unknown dependencies are ignored', () => {
    expect(sentryExplanation(obj({ status: 'COMPLETED', depends_on: ['upstream'] }), peers)).toBeUndefined()
    expect(sentryExplanation(obj({ depends_on: ['ghost-item'] }), peers)).toBeUndefined()
    expect(sentryExplanation(obj({}), peers)).toBeUndefined()
  })
})
