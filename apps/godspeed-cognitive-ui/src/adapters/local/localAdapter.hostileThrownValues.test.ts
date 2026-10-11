import { describe, expect, test } from 'bun:test'
import type { CaseworkPort, InteractionIntent, StreamEvent, XObject, XSnapshot } from '../../ports/contract'
import { LocalContractAdapter, NORTHSTAR_CASE_ID } from './localAdapter'

function installManualTimers() {
  const pending: Array<{ callback: () => void; cancelled: boolean; fired: boolean }> = []
  const setTimeoutBefore = globalThis.setTimeout
  const clearTimeoutBefore = globalThis.clearTimeout
  globalThis.setTimeout = ((callback: TimerHandler) => {
    if (typeof callback !== 'function') throw new TypeError('manual timers require a callback')
    const timer = { callback: () => callback(), cancelled: false, fired: false }
    pending.push(timer)
    return pending.length as unknown as ReturnType<typeof setTimeout>
  }) as unknown as typeof setTimeout
  globalThis.clearTimeout = ((handle: ReturnType<typeof setTimeout>) => {
    const timer = pending[Number(handle) - 1]
    if (timer) timer.cancelled = true
  }) as typeof clearTimeout
  return {
    runNext() {
      const timer = pending.find((candidate) => !candidate.cancelled && !candidate.fired)
      if (!timer) throw new Error('no pending timer')
      timer.fired = true
      timer.callback()
    },
    runAll() {
      let count = 0
      while (pending.some((timer) => !timer.cancelled && !timer.fired)) {
        if (++count > 20) throw new Error('manual timer drain exceeded 20 callbacks')
        this.runNext()
      }
    },
    restore() {
      globalThis.setTimeout = setTimeoutBefore
      globalThis.clearTimeout = clearTimeoutBefore
    },
  }
}

function appendFixture(adapter: LocalContractAdapter, summary: string): XSnapshot {
  const subject = adapter as unknown as {
    append(caseId: string, eventType: string, summary: string, intent: InteractionIntent | null, edit: (objects: XObject[]) => void): XSnapshot
  }
  return subject.append(NORTHSTAR_CASE_ID, 'fixture_revision', summary, null, () => undefined)
}

function subscribe(adapter: LocalContractAdapter, onEvent: (event: StreamEvent) => void, onError: (error: Error) => void) {
  return (adapter as CaseworkPort).subscribeEvents(NORTHSTAR_CASE_ID, undefined, onEvent, onError)
}

describe('LocalContractAdapter hostile thrown values', () => {
  test('contains coercion and instanceof traps while isolating the throwing subscriber', () => {
    const hostileValues: Array<{ name: string; value: unknown }> = [
      { name: 'null-prototype object', value: Object.create(null) as object },
      {
        name: 'throwing primitive conversion',
        value: { [Symbol.toPrimitive]() { throw new Error('primitive conversion refused') } },
      },
      {
        name: 'throwing string conversion',
        value: { toString() { throw new Error('string conversion refused') } },
      },
      {
        name: 'throwing prototype trap',
        value: new Proxy({}, { getPrototypeOf() { throw new Error('prototype lookup refused') } }),
      },
    ]

    for (const hostile of hostileValues) {
      const timers = installManualTimers()
      let stopOffending = () => {}
      let stopHealthy = () => {}
      try {
        const adapter = new LocalContractAdapter({ latency: 0 })
        const errors: Error[] = []
        const healthy: string[] = []
        let offendingCalls = 0
        stopOffending = subscribe(adapter, () => {
          offendingCalls++
          throw hostile.value
        }, (error) => errors.push(error))
        stopHealthy = subscribe(adapter, (event) => healthy.push(event.cursor), () => {})

        const first = appendFixture(adapter, `${hostile.name}: callback failure`)
        expect(() => timers.runNext()).not.toThrow()
        timers.runAll()
        expect(offendingCalls).toBe(1)
        expect(errors).toHaveLength(1)
        expect(errors[0]).toBeInstanceOf(Error)
        expect(errors[0]?.message).toBe('Local operation failed with an unprintable thrown value')
        expect(healthy).toEqual([first.cursor])

        const later = appendFixture(adapter, `${hostile.name}: healthy continuation`)
        timers.runAll()
        expect(offendingCalls).toBe(1)
        expect(errors).toHaveLength(1)
        expect(healthy).toEqual([first.cursor, later.cursor])
      } finally {
        stopOffending()
        stopHealthy()
        timers.restore()
      }
    }
  })
})
