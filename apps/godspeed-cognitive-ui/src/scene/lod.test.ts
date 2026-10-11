import { describe, it, expect } from 'bun:test'
import { nextLod, LodTracker, LOD_UP, HYSTERESIS } from './lod'

describe('nextLod', () => {
  describe('plain thresholds (no hysteresis on the way up)', () => {
    it('stays at level 0 when size is below threshold', () => {
      expect(nextLod(0, 0)).toBe(0)
      expect(nextLod(0, 33)).toBe(0)
      expect(nextLod(0, 33.99)).toBe(0)
    })

    it('switches UP to level 1 when size >= 34', () => {
      expect(nextLod(0, 34)).toBe(1)
      expect(nextLod(0, 100)).toBe(1)
    })

    it('switches UP to level 2 when size >= 120', () => {
      expect(nextLod(0, 120)).toBe(2)
      expect(nextLod(0, 200)).toBe(2)
    })

    it('switches UP to level 3 when size >= 420', () => {
      expect(nextLod(0, 420)).toBe(3)
      expect(nextLod(0, 500)).toBe(3)
      expect(nextLod(0, 1000)).toBe(3)
    })

    it('switches up immediately from any level without hysteresis', () => {
      expect(nextLod(1, 120)).toBe(2)
      expect(nextLod(2, 420)).toBe(3)
      expect(nextLod(0, 500)).toBe(3)
    })
  })

  describe('hysteresis on the way down (per level)', () => {

    it('stays at level 1 when size dips below 34 but above hysteresis threshold', () => {
      // Current level is 1, size drops to 26.53 (above 26.52)
      expect(nextLod(1, (LOD_UP[0] * HYSTERESIS) + 0.01)).toBe(1)
      expect(nextLod(1, 27)).toBe(1)
      expect(nextLod(1, 33)).toBe(1)
    })

    it('drops from level 1 to 0 when size falls below hysteresis threshold', () => {
      expect(nextLod(1, (LOD_UP[0] * HYSTERESIS) - 0.01)).toBe(0)
      expect(nextLod(1, 26)).toBe(0)
      expect(nextLod(1, 0)).toBe(0)
    })

    it('stays at level 2 when size dips below 120 but above hysteresis threshold', () => {
      expect(nextLod(2, (LOD_UP[1] * HYSTERESIS) + 0.01)).toBe(2)
      expect(nextLod(2, 94)).toBe(2)
      expect(nextLod(2, 119)).toBe(2)
    })

    it('drops from level 2 to 1 when size falls below hysteresis threshold of level 2', () => {
      expect(nextLod(2, (LOD_UP[1] * HYSTERESIS) - 0.01)).toBe(1)
      expect(nextLod(2, 93)).toBe(1)
    })

    it('stays at level 3 when size dips below 420 but above hysteresis threshold', () => {
      expect(nextLod(3, (LOD_UP[2] * HYSTERESIS) + 0.01)).toBe(3)
      expect(nextLod(3, 328)).toBe(3)
      expect(nextLod(3, 419)).toBe(3)
    })

    it('drops from level 3 to 2 when size falls below hysteresis threshold of level 3', () => {
      expect(nextLod(3, (LOD_UP[2] * HYSTERESIS) - 0.01)).toBe(2)
      expect(nextLod(3, 327)).toBe(2)
    })
  })

  describe('multi-level jumps', () => {
    it('jumps from level 0 directly to level 3 when size is large', () => {
      expect(nextLod(0, 500)).toBe(3)
      expect(nextLod(0, 1000)).toBe(3)
    })

    it('jumps from level 3 directly to level 0 when size is very small', () => {
      const verySmallSize = 5
      expect(nextLod(3, verySmallSize)).toBe(0)
    })

    it('jumps down multiple levels with hysteresis applied per level', () => {
      // From level 3 to small size: should apply hysteresis checks for 3→2 and 2→1 and 1→0

      // Size 328: stays at 3 (above level 3 hysteresis)
      expect(nextLod(3, 328)).toBe(3)

      // Size 327: drops to 2 (below level 3 hysteresis)
      expect(nextLod(3, 327)).toBe(2)

      // Size 94: from level 3, drops to 2 (below level 3 hysteresis but above level 2 hysteresis)
      expect(nextLod(3, 94)).toBe(2)

      // Size 93: from level 3, should drop to 1 (below both level 3 and level 2 hysteresis)
      expect(nextLod(3, 93)).toBe(1)

      // Size 26: from level 3, should drop all the way to 0
      expect(nextLod(3, 26)).toBe(0)
    })

    it('drops from level 2 through multiple levels to level 0 with hysteresis', () => {

      // Size safely above level 2 hysteresis: stays at 2
      expect(nextLod(2, 94)).toBe(2)

      // Size safely below level 2 hysteresis: drops to 1
      expect(nextLod(2, 93)).toBe(1)

      // Size well below all thresholds: drops all the way to 0
      expect(nextLod(2, 26)).toBe(0)
    })
  })

  describe('boundary values', () => {
    it('treats exactly at threshold as switching to next level', () => {
      expect(nextLod(0, 34)).toBe(1)
      expect(nextLod(0, 120)).toBe(2)
      expect(nextLod(0, 420)).toBe(3)
    })

    it('treats just below threshold as not switching', () => {
      expect(nextLod(0, 33.9999)).toBe(0)
      expect(nextLod(0, 119.9999)).toBe(1)
      expect(nextLod(0, 419.9999)).toBe(2)
    })

    it('treats hysteresis boundary precisely', () => {
      const level1Down = LOD_UP[0] * HYSTERESIS
      expect(nextLod(1, level1Down)).toBe(1) // at boundary, stay
      expect(nextLod(1, level1Down - 0.0001)).toBe(0) // just below, drop
    })
  })

  describe('LodTracker', () => {
    it('tracks and returns LOD for new objects', () => {
      const tracker = new LodTracker()
      expect(tracker.get('obj1', 50)).toBe(1)
      expect(tracker.get('obj2', 200)).toBe(2)
      expect(tracker.get('obj3', 500)).toBe(3)
    })

    it('applies hysteresis for tracked objects', () => {
      const tracker = new LodTracker()
      // First call: obj1 at size 120 → level 2
      expect(tracker.get('obj1', 120)).toBe(2)

      // Second call: size drops to 119 → stays at level 2 (above hysteresis threshold 93.6)
      expect(tracker.get('obj1', 119)).toBe(2)

      // Third call: size drops to 90 → drops to level 1 (below hysteresis threshold)
      expect(tracker.get('obj1', 90)).toBe(1)
    })

    it('detects changes when LOD level changes', () => {
      const tracker = new LodTracker()

      // Initial: no changes yet
      expect(tracker.consumeChanged()).toBe(false)

      // First call triggers change
      tracker.get('obj1', 50)
      expect(tracker.consumeChanged()).toBe(true)

      // Same level: no change
      tracker.get('obj1', 40)
      expect(tracker.consumeChanged()).toBe(false)

      // Different level: change
      tracker.get('obj1', 150)
      expect(tracker.consumeChanged()).toBe(true)

      // Multiple objects
      tracker.get('obj2', 100)
      tracker.get('obj3', 500)
      expect(tracker.consumeChanged()).toBe(true)
    })

    it('tracks multiple objects independently', () => {
      const tracker = new LodTracker()
      expect(tracker.get('a', 50)).toBe(1)
      expect(tracker.get('b', 200)).toBe(2)
      expect(tracker.get('c', 10)).toBe(0)

      // Change only one
      expect(tracker.get('a', 150)).toBe(2)
      expect(tracker.get('b', 200)).toBe(2)
      expect(tracker.consumeChanged()).toBe(true)
    })

    it('forgets tracked objects', () => {
      const tracker = new LodTracker()
      tracker.get('obj1', 100)
      tracker.get('obj2', 200)

      tracker.forget('obj1')

      // After forgetting, the object should behave like a new one
      expect(tracker.get('obj1', 50)).toBe(1)
      expect(tracker.consumeChanged()).toBe(true) // change from new "no previous" behavior
    })

    it('consumeChanged clears the changed set', () => {
      const tracker = new LodTracker()
      tracker.get('obj1', 100)
      expect(tracker.consumeChanged()).toBe(true)

      // Now it's consumed, no more changes until something changes
      expect(tracker.consumeChanged()).toBe(false)
      expect(tracker.consumeChanged()).toBe(false)

      // Change something again
      tracker.get('obj1', 200)
      expect(tracker.consumeChanged()).toBe(true)
    })

    it('handles multi-level jumps in tracker', () => {
      const tracker = new LodTracker()

      // Start at level 0, jump to level 3
      expect(tracker.get('obj1', 500)).toBe(3)
      expect(tracker.consumeChanged()).toBe(true)

      // Drop back to level 0
      expect(tracker.get('obj1', 10)).toBe(0)
      expect(tracker.consumeChanged()).toBe(true)

      // Jump back up to level 3
      expect(tracker.get('obj1', 450)).toBe(3)
      expect(tracker.consumeChanged()).toBe(true)
    })

    it('applies hysteresis correctly across multiple updates', () => {
      const tracker = new LodTracker()
      const level2Down = LOD_UP[1] * HYSTERESIS // 93.6

      // Start at level 2
      expect(tracker.get('obj1', 150)).toBe(2)
      expect(tracker.consumeChanged()).toBe(true)

      // Dip just below threshold but above hysteresis: stay at 2
      expect(tracker.get('obj1', level2Down + 1)).toBe(2)
      expect(tracker.consumeChanged()).toBe(false)

      // Cross hysteresis threshold: drop to 1
      expect(tracker.get('obj1', level2Down - 1)).toBe(1)
      expect(tracker.consumeChanged()).toBe(true)

      // Recover above up-threshold: jump to 2
      expect(tracker.get('obj1', 121)).toBe(2)
      expect(tracker.consumeChanged()).toBe(true)
    })
  })
})
