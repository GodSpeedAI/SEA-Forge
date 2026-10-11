import { describe, expect, test } from 'bun:test'
import { buildNorthstarHistory, NORTHSTAR_TRAJECTORY } from './northstarData'
import { PAYLOADS } from './payloads'
import { buildTemplateHistory } from './templateData'

const p = { actor_id: 'usr-test', role: 'case_architect' as const }

describe('northstar contract data', () => {
  const h = buildNorthstarHistory(p)
  test('six snapshots with strictly increasing contract cursors', () => {
    expect(h.length).toBe(6)
    for (const s of h) expect(s.cursor).toMatch(/^[0-9]+\.[0-9]{10}$/)
    for (let i = 1; i < h.length; i++) expect(Number(h[i]!.cursor.split('.')[1])).toBeGreaterThan(Number(h[i - 1]!.cursor.split('.')[1]))
  })
  test('trajectory has one checkpoint per snapshot', () => {
    expect(NORTHSTAR_TRAJECTORY.map((t) => t.cursor)).toEqual(h.map((s) => s.cursor))
  })
  test('every artifact ref resolves and every relationship endpoint exists', () => {
    for (const s of h) {
      const ids = new Set(s.visible_objects.map((o) => o.id))
      for (const o of s.visible_objects) for (const a of o.x?.artifacts ?? []) expect(PAYLOADS[a.ref]).toBeDefined()
      for (const r of s.x?.relationships ?? []) {
        expect(ids.has(r.from)).toBe(true)
        expect(ids.has(r.to)).toBe(true)
      }
    }
  })
  test('the live release offers a consequential approval', () => {
    const rel = h.at(-1)!.visible_objects.find((o) => o.actions.some((a) => a.intent === 'APPROVE_HUMAN_TASK'))
    expect(rel?.actions.find((a) => a.intent === 'APPROVE_HUMAN_TASK')?.consequential).toBe(true)
  })
  test('causal representations use the role vocabulary', () => {
    const roles = h.at(-1)!.visible_objects.flatMap((o) => (o.x?.representations?.causal ? [o.x.representations.causal.role] : []))
    expect(roles.sort()).toEqual(['assumption', 'condition', 'expected', 'missing', 'observed'])
  })
  test('perspective is stamped and data at rest is not shared between calls', () => {
    const a = buildNorthstarHistory(p)
    ;(a[0]!.visible_objects as unknown as { name: string }[])[0]!.name = "mutated"
    expect(buildNorthstarHistory(p)[0]!.visible_objects[0]!.name).not.toBe('mutated')
    expect(a[0]!.perspective.actor_id).toBe('usr-test')
  })
})

describe('template contract data', () => {
  test('two published versions, 4 then 5 stages', () => {
    const t = buildTemplateHistory(p)
    expect(t.length).toBe(2)
    const stages = (i: number) => t[i]!.visible_objects.find((o) => o.x?.design?.icon === 'layers')?.x?.design?.items?.length
    expect(stages(0)).toBe(4)
    expect(stages(1)).toBe(5)
  })
})
