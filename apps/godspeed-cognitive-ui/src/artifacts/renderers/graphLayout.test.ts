import { describe, it, expect } from 'bun:test'
import { layeredLayout } from './graphLayout'

describe('graphLayout', () => {
  it('handles a simple chain: A -> B -> C', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }, { id: 'C' }]
    const edges = [
      { from: 'A', to: 'B' },
      { from: 'B', to: 'C' },
    ]

    const positions = layeredLayout(nodes, edges)

    expect(positions['A'].layer).toBe(0)
    expect(positions['B'].layer).toBe(1)
    expect(positions['C'].layer).toBe(2)

    // X positions should increase by layer
    expect(positions['A'].x).toBeLessThan(positions['B'].x)
    expect(positions['B'].x).toBeLessThan(positions['C'].x)
  })

  it('handles a diamond: A -> B,C; B,C -> D', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }, { id: 'C' }, { id: 'D' }]
    const edges = [
      { from: 'A', to: 'B' },
      { from: 'A', to: 'C' },
      { from: 'B', to: 'D' },
      { from: 'C', to: 'D' },
    ]

    const positions = layeredLayout(nodes, edges)

    expect(positions['A'].layer).toBe(0)
    expect(positions['B'].layer).toBe(1)
    expect(positions['C'].layer).toBe(1)
    expect(positions['D'].layer).toBe(2)

    // B and C are in same layer, should have different Y positions
    expect(positions['B'].y).not.toBe(positions['C'].y)
  })

  it('handles a 2-cycle: A <-> B (does not hang)', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }]
    const edges = [
      { from: 'A', to: 'B' },
      { from: 'B', to: 'A' },
    ]

    // Should not hang; iteration cap prevents infinite loop
    const positions = layeredLayout(nodes, edges)

    expect(Object.keys(positions).length).toBe(2)
    expect(positions['A']).toBeDefined()
    expect(positions['B']).toBeDefined()
  })

  it('is deterministic: same input produces same output', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }, { id: 'C' }]
    const edges = [
      { from: 'A', to: 'B' },
      { from: 'A', to: 'C' },
    ]

    const result1 = layeredLayout(nodes, edges)
    const result2 = layeredLayout(nodes, edges)

    expect(JSON.stringify(result1)).toBe(JSON.stringify(result2))
  })

  it('handles empty graph', () => {
    const positions = layeredLayout([], [])
    expect(positions).toEqual({})
  })

  it('handles isolated nodes', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }, { id: 'C' }]
    const edges: { from: string; to: string }[] = []

    const positions = layeredLayout(nodes, edges)

    // All isolated nodes should be in layer 0
    expect(positions['A'].layer).toBe(0)
    expect(positions['B'].layer).toBe(0)
    expect(positions['C'].layer).toBe(0)
  })

  it('spreads nodes vertically within a layer', () => {
    const nodes = [{ id: 'A' }, { id: 'B' }, { id: 'C' }]
    const edges: { from: string; to: string }[] = []

    const positions = layeredLayout(nodes, edges)

    // All in layer 0, but with different Y positions
    const ys = [positions['A'].y, positions['B'].y, positions['C'].y].sort()
    expect(ys[0]).not.toBe(ys[1])
    expect(ys[1]).not.toBe(ys[2])
  })
})
