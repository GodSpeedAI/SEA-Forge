import { useSyncExternalStore } from 'react'
import type { ArtifactPayload, CaseworkPort } from '../ports/contract'
import { parseArtifact, type ArtifactModel } from './model'

// Source-backed artifact resolution. Payloads come only from the port (`resolveArtifact`), are
// cached by ref and parsed once. Failures stay attached to the ref: the rest of the UI keeps
// working and the source reference stays visible.

export type ArtifactState =
  | { status: 'loading' }
  | { status: 'ready'; payload: ArtifactPayload; model: ArtifactModel }
  | { status: 'error'; error: string; payload?: ArtifactPayload }

export interface ArtifactService {
  get(ref: string): ArtifactState
  /** Starts resolution if needed. Safe to call on every render. */
  ensure(ref: string): void
  subscribe(fn: () => void): () => void
}

const LOADING: ArtifactState = { status: 'loading' }

export function createArtifactService(port: Pick<CaseworkPort, 'resolveArtifact'>): ArtifactService {
  const cache = new Map<string, ArtifactState>()
  const subs = new Set<() => void>()
  const set = (ref: string, st: ArtifactState) => {
    cache.set(ref, st)
    for (const fn of subs) fn()
  }
  return {
    get: (ref) => cache.get(ref) ?? LOADING,
    ensure(ref) {
      if (cache.has(ref)) return
      cache.set(ref, LOADING)
      port.resolveArtifact(ref).then(
        (payload) => {
          const parsed = parseArtifact(payload)
          set(ref, parsed.ok ? { status: 'ready', payload, model: parsed.model } : { status: 'error', error: parsed.error, payload })
        },
        (e: unknown) => set(ref, { status: 'error', error: e instanceof Error ? e.message : String(e) }),
      )
    },
    subscribe(fn) {
      subs.add(fn)
      return () => subs.delete(fn)
    },
  }
}

export function useArtifact(service: ArtifactService, ref: string): ArtifactState {
  service.ensure(ref)
  return useSyncExternalStore(service.subscribe, () => service.get(ref))
}
