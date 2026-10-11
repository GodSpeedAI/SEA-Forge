import type { Lod } from '../model/types'

/** Upward thresholds in "apparent size" px: an object switches UP to level n+1 when size >= UP[n]. */
export const LOD_UP = [34, 120, 420] as const

/** Hysteresis: it switches back DOWN from level n+1 to n only when size < UP[n] * HYSTERESIS. */
export const HYSTERESIS = 0.78

/** apparent = baseSizeWorld * screenScale (px). */
export function nextLod(prev: Lod, apparentPx: number): Lod {
  // Determine level based on plain thresholds (no hysteresis)
  let level: Lod = 0
  if (apparentPx >= LOD_UP[0]) level = 1
  if (apparentPx >= LOD_UP[1]) level = 2
  if (apparentPx >= LOD_UP[2]) level = 3

  // If going up, return immediately without hysteresis
  if (level > prev) return level

  // If going down, apply hysteresis check at each level
  let result = prev
  for (let l = prev; l > level; l--) {
    // Check if we can drop from level l to l-1
    const hysteresisThreshold = LOD_UP[l - 1] * HYSTERESIS
    if (apparentPx < hysteresisThreshold) {
      result = (l - 1) as Lod
    } else {
      // Can't drop this step, stay at level l
      break
    }
  }
  return result
}

export class LodTracker {
  private tracked = new Map<string, Lod>()
  private changed = new Set<string>()

  /** Returns the LOD for id, applying hysteresis against its last value. Unknown ids start from the plain threshold level (no hysteresis). */
  get(id: string, apparentPx: number): Lod {
    const prev = this.tracked.get(id)
    // New ids: use plain thresholds (nextLod treats prev like any other level)
    const newLod = nextLod(prev ?? 0, apparentPx)

    if (prev !== newLod) {
      this.tracked.set(id, newLod)
      this.changed.add(id)
    } else {
      this.tracked.set(id, newLod)
    }

    return newLod
  }

  /** Returns true if any tracked id changed level since the last call to consumeChanged(). */
  consumeChanged(): boolean {
    const result = this.changed.size > 0
    this.changed.clear()
    return result
  }

  forget(id: string): void {
    this.tracked.delete(id)
    this.changed.delete(id)
  }
}
