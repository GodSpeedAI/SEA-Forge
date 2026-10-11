import { describe, expect, test } from 'bun:test'
import { HttpCaseworkAdapter } from './httpCaseworkAdapter'

type ScheduledTimer = { callback: () => void; delay: number; cancelled: boolean; fired: boolean }

function installManualTimers() {
  const timers: ScheduledTimer[] = []
  const originalSetTimeout = globalThis.setTimeout
  const originalClearTimeout = globalThis.clearTimeout
  globalThis.setTimeout = ((callback: TimerHandler, delay?: number) => {
    if (typeof callback !== 'function') throw new TypeError('manual timers require a callback')
    const timer: ScheduledTimer = {
      callback: () => callback(),
      delay: Number(delay ?? 0),
      cancelled: false,
      fired: false,
    }
    timers.push(timer)
    return timers.length as unknown as ReturnType<typeof setTimeout>
  }) as unknown as typeof setTimeout
  globalThis.clearTimeout = ((handle: ReturnType<typeof setTimeout>) => {
    const timer = timers[Number(handle) - 1]
    if (timer) timer.cancelled = true
  }) as typeof clearTimeout

  return {
    timers,
    runNext() {
      const timer = timers.find((candidate) => !candidate.cancelled && !candidate.fired)
      if (!timer) throw new Error('no pending timer')
      timer.fired = true
      timer.callback()
      return timer
    },
    restore() {
      globalThis.setTimeout = originalSetTimeout
      globalThis.clearTimeout = originalClearTimeout
    },
  }
}

describe('HttpCaseworkAdapter native EventSource recovery', () => {
  test('uses capped exponential retries, resumes from the last cursor, and ignores replayed frames', () => {
    class FakeEventSource {
      static instances: FakeEventSource[] = []
      readonly url: string
      readonly withCredentials: boolean
      closed = false
      onerror: ((event: Event) => void) | null = null
      onopen: ((event: Event) => void) | null = null
      private listeners = new Map<string, (event: MessageEvent) => void>()

      constructor(url: string, options?: { withCredentials?: boolean }) {
        this.url = url
        this.withCredentials = Boolean(options?.withCredentials)
        FakeEventSource.instances.push(this)
      }

      addEventListener(type: string, listener: EventListener) {
        this.listeners.set(type, listener as (event: MessageEvent) => void)
      }

      close() { this.closed = true }

      open() { this.onopen?.(new Event('open')) }
      fail() { this.onerror?.(new Event('error')) }
      emit(cursor: string, caseId = 'case_a') {
        this.listeners.get('snapshot')?.(new MessageEvent('snapshot', {
          data: JSON.stringify({ event_type: 'snapshot', cursor, payload: { case_id: caseId } }),
          lastEventId: cursor,
        }))
      }
    }

    const timers = installManualTimers()
    const originalSource = globalThis.EventSource
    const originalFetch = globalThis.fetch
    let fetchCount = 0
    globalThis.EventSource = FakeEventSource as unknown as typeof EventSource
    globalThis.fetch = Object.assign(
      (..._args: Parameters<typeof fetch>): ReturnType<typeof fetch> => {
        fetchCount++
        return Promise.reject(new Error('native EventSource must not use fetch fallback'))
      },
      { preconnect: originalFetch.preconnect },
    )
    try {
      const errors: Error[] = []
      const received: string[] = []
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test', reconnectBaseMs: 10, reconnectMaxMs: 25 })
      const unsubscribe = adapter.subscribeEvents('case_a', '01M0', (event) => received.push(event.cursor), (error) => errors.push(error))

      expect(FakeEventSource.instances[0]!.url).toContain('/api/events?last=01M0')
      expect(FakeEventSource.instances[0]!.withCredentials).toBe(true)
      FakeEventSource.instances[0]!.fail()
      expect(timers.timers[0]!.delay).toBe(10)
      timers.runNext()
      FakeEventSource.instances[1]!.fail()
      expect(timers.timers[1]!.delay).toBe(20)
      timers.runNext()
      FakeEventSource.instances[2]!.fail()
      expect(timers.timers[2]!.delay).toBe(25)
      timers.runNext()

      const connected = FakeEventSource.instances[3]!
      connected.open()
      connected.emit('01M1')
      connected.emit('01M1')
      expect(received).toEqual(['01M1'])
      connected.fail()
      expect(timers.timers[3]!.delay).toBe(10)
      timers.runNext()

      const resumed = FakeEventSource.instances[4]!
      expect(resumed.url).toContain('/api/events?last=01M1')
      resumed.emit('01M1')
      resumed.emit('01M2')
      expect(received).toEqual(['01M1', '01M2'])
      expect(errors).toHaveLength(4)
      expect(fetchCount).toBe(0)
      unsubscribe()
    } finally {
      globalThis.EventSource = originalSource
      globalThis.fetch = originalFetch
      timers.restore()
    }
  })

  test('reports each (re)open of the stream through onOpen and never fetches on its own', () => {
    class FakeEventSource {
      static instances: FakeEventSource[] = []
      onerror: ((event: Event) => void) | null = null
      onopen: ((event: Event) => void) | null = null
      constructor(_url: string, _options?: { withCredentials?: boolean }) { FakeEventSource.instances.push(this) }
      addEventListener(_type: string, _listener: EventListener) {}
      close() {}
    }
    const timers = installManualTimers()
    const originalSource = globalThis.EventSource
    globalThis.EventSource = FakeEventSource as unknown as typeof EventSource
    try {
      let opens = 0
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test', reconnectBaseMs: 10, reconnectMaxMs: 25 })
      const unsubscribe = adapter.subscribeEvents('case_a', undefined, () => {}, () => {}, () => { opens++ })
      FakeEventSource.instances[0]!.onopen!(new Event('open'))
      expect(opens).toBe(1)
      FakeEventSource.instances[0]!.onerror!(new Event('error'))
      timers.runNext()
      FakeEventSource.instances[1]!.onopen!(new Event('open'))
      expect(opens).toBe(2)
      unsubscribe()
    } finally {
      globalThis.EventSource = originalSource
      timers.restore()
    }
  })

  test('disposal closes the source and cancels a pending retry', () => {
    class FakeEventSource {
      static instances: FakeEventSource[] = []
      onerror: ((event: Event) => void) | null = null
      onopen: ((event: Event) => void) | null = null
      closed = false
      constructor(_url: string, _options?: { withCredentials?: boolean }) { FakeEventSource.instances.push(this) }
      addEventListener(_type: string, _listener: EventListener) {}
      close() { this.closed = true }
      fail() { this.onerror?.(new Event('error')) }
    }

    const timers = installManualTimers()
    const originalSource = globalThis.EventSource
    globalThis.EventSource = FakeEventSource as unknown as typeof EventSource
    try {
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test', reconnectBaseMs: 10, reconnectMaxMs: 20 })
      const unsubscribe = adapter.subscribeEvents('case_a', undefined, () => {}, () => {})
      const source = FakeEventSource.instances[0]!
      source.fail()
      const pending = timers.timers[0]!
      expect(source.closed).toBe(true)
      expect(pending.cancelled).toBe(false)

      unsubscribe()
      expect(pending.cancelled).toBe(true)
      expect(FakeEventSource.instances).toHaveLength(1)
      expect(() => timers.runNext()).toThrow('no pending timer')
    } finally {
      globalThis.EventSource = originalSource
      timers.restore()
    }
  })

  test('open and drop cycles grow to the cap; malformed and foreign frames do not advance the cursor', () => {
    class FakeEventSource {
      static instances: FakeEventSource[] = []
      readonly url: string
      onerror: ((event: Event) => void) | null = null
      onopen: ((event: Event) => void) | null = null
      closed = false
      private listeners = new Map<string, (event: MessageEvent) => void>()
      constructor(url: string) { this.url = url; FakeEventSource.instances.push(this) }
      addEventListener(type: string, listener: EventListener) { this.listeners.set(type, listener as (event: MessageEvent) => void) }
      close() { this.closed = true }
      open() { this.onopen?.(new Event('open')) }
      fail() { this.onerror?.(new Event('error')) }
      emit(cursor: string, data: string) {
        this.listeners.get('snapshot')?.(new MessageEvent('snapshot', { data, lastEventId: cursor }))
      }
    }

    const timers = installManualTimers()
    const originalSource = globalThis.EventSource
    globalThis.EventSource = FakeEventSource as unknown as typeof EventSource
    try {
      const received: string[] = []
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test', reconnectBaseMs: 10, reconnectMaxMs: 25 })
      const unsubscribe = adapter.subscribeEvents('case_a', '01M0', (event) => received.push(event.cursor), () => {})

      for (const expectedDelay of [10, 20, 25]) {
        const source = FakeEventSource.instances.at(-1)!
        source.open()
        source.fail()
        expect(timers.timers.at(-1)?.delay).toBe(expectedDelay)
        timers.runNext()
      }

      const resumed = FakeEventSource.instances.at(-1)!
      expect(resumed.url).toContain('/api/events?last=01M0')
      expect(FakeEventSource.instances).toHaveLength(4)
      const preRetryCount = FakeEventSource.instances.length
      resumed.emit('01M1', '{malformed json')
      resumed.emit('01M2', JSON.stringify({ event_type: 'snapshot', cursor: '01M2', timestamp: 't', payload: { case_id: 'case_b' } }))
      resumed.emit('01M2', JSON.stringify({ event_type: 'snapshot', cursor: '01M2', timestamp: 't', payload: { case_id: 'case_a' } }))
      expect(received).toEqual(['01M2'])
      resumed.fail()
      expect(timers.timers.at(-1)?.delay).toBe(10)
      expect(FakeEventSource.instances).toHaveLength(preRetryCount)
      timers.runNext()
      expect(FakeEventSource.instances).toHaveLength(preRetryCount + 1)
      expect(FakeEventSource.instances.at(-1)?.url).toBe('http://gw.test/api/events?last=01M2')
      unsubscribe()
    } finally {
      globalThis.EventSource = originalSource
      timers.restore()
    }
  })

  test('delivers a resync control frame without suppressing the snapshot at the same retained cursor', () => {
    class FakeEventSource {
      static instances: FakeEventSource[] = []
      onerror: ((event: Event) => void) | null = null
      private listeners = new Map<string, (event: MessageEvent) => void>()
      constructor(_url: string) { FakeEventSource.instances.push(this) }
      addEventListener(type: string, listener: EventListener) { this.listeners.set(type, listener as (event: MessageEvent) => void) }
      close() {}
      emit(type: string, cursor: string) {
        this.listeners.get(type)?.(new MessageEvent(type, {
          data: JSON.stringify({ event_type: type, cursor, timestamp: 't', payload: { case_id: 'case_a' } }),
          lastEventId: cursor,
        }))
      }
    }

    const originalSource = globalThis.EventSource
    globalThis.EventSource = FakeEventSource as unknown as typeof EventSource
    try {
      const received: string[] = []
      const adapter = new HttpCaseworkAdapter({ base: 'http://gw.test' })
      const unsubscribe = adapter.subscribeEvents('case_a', '01M0', (event) => received.push(`${event.event_type}:${event.cursor}`), () => {})
      const source = FakeEventSource.instances[0]!

      source.emit('resync_required', '01M1')
      source.emit('snapshot', '01M1')
      source.emit('snapshot', '01M1')

      expect(received).toEqual(['resync_required:01M1', 'snapshot:01M1'])
      unsubscribe()
    } finally {
      globalThis.EventSource = originalSource
    }
  })
})
