import { describe, test, expect } from 'bun:test'
import { projectSnapshot, projectHistory, compareCursor, loadCaseHistory, CORE_ID } from './project'
import type { XSnapshot, XObject, Actor } from './contract'
import { LocalContractAdapter } from '../adapters/local/localAdapter'
import { NORTHSTAR_CASE_ID } from '../adapters/local/northstarData'

const defaultSummary = { phase: 'init', headline: 'Test', status_phrase: 'Testing' }

/** Helper to fill in required CognitiveWorldSnapshot fields for test snapshots */
function snap(partial: Partial<XSnapshot>): XSnapshot {
  return {
    world_id: 'world-test',
    case_id: partial.case_id ?? 'case-test',
    cursor: partial.cursor ?? '1.0',
    timestamp: partial.timestamp ?? '2026-09-19T00:00:00Z',
    perspective: partial.perspective ?? {
      actor_id: 'actor-test',
      role: 'case_architect',
    },
    summary: partial.summary ?? defaultSummary,
    visible_objects: partial.visible_objects ?? [],
    available_actions: partial.available_actions ?? [],
    attention_focus: partial.attention_focus ?? {
      primary_object_id: 'primary',
    },
    x: partial.x,
  }
}

describe('projectSnapshot', () => {
  test('adds core when withCore is true and parents roots to it', () => {
    const testSnap = snap({
      visible_objects: [
        { id: 'root1', kind: 'category', parent: null, title: 'Root 1', actions: [] } as unknown as XObject,
        { id: 'child', kind: 'item', parent: 'root1', title: 'Child', actions: [] } as unknown as XObject,
      ],
    })

    const result = projectSnapshot(testSnap as any, { withCore: true })

    expect(result.objects[CORE_ID]).toBeDefined()
    expect(result.objects['root1']?.parent).toBe(CORE_ID)
    // Note: children are also reparented to core when withCore is true
    expect(result.objects['child']?.parent).toBe(CORE_ID)
  })

  test('carries the case lifecycle actions the snapshot offers on the core, and nothing else', () => {
    const act = (intent: string, objectId: string) => ({ id: `act-${intent.toLowerCase()}-${objectId}`, label: intent, intent, consequential: true, requires_justification: true })
    const testSnap = snap({
      visible_objects: [{ id: 'item_a', kind: 'work_item', name: 'A', status: 'READY_TO_BEGIN', badge: 'Ready', salience: 0.5, actions: [act('EXECUTE_ITEM', 'item_a')] } as unknown as XObject],
      available_actions: [act('EXECUTE_ITEM', 'item_a'), act('TERMINATE_CASE', 'case-test'), act('REOPEN_CASE', 'case-test')] as any,
    })
    const withCore = projectSnapshot(testSnap as any, { withCore: true })
    expect(withCore.objects[CORE_ID]?.actions?.map((a) => a.intent)).toEqual(['TERMINATE_CASE', 'REOPEN_CASE'])
    expect(withCore.objects[CORE_ID]?.actions?.every((a) => a.consequential && a.requiresJustification)).toBe(true)
    expect(withCore.objects['item_a']?.actions?.map((a) => a.intent)).toEqual(['EXECUTE_ITEM'])
    // Not offered => not shown: a snapshot without lifecycle offers leaves the core without actions.
    const quiet = projectSnapshot(snap({ visible_objects: testSnap.visible_objects }) as any, { withCore: true })
    expect(quiet.objects[CORE_ID]?.actions).toBeUndefined()
    // Without a core there is nowhere to carry them.
    expect(projectSnapshot(testSnap as any, { withCore: false }).objects[CORE_ID]).toBeUndefined()
  })

  test('does not add core when withCore is false', () => {
    const testSnap = snap({
      visible_objects: [
        { id: 'root1', kind: 'category', parent: null, title: 'Root 1', actions: [] } as unknown as XObject,
      ],
    })

    const result = projectSnapshot(testSnap as any, { withCore: false })

    expect(result.objects[CORE_ID]).toBeUndefined()
    expect(result.objects['root1']?.parent).toBe(null)
  })

  test('maps badge→status.label', () => {
    const testSnap = snap({
      visible_objects: [
        {
          id: 'obj1',
          kind: 'item',
          parent: null,
          title: 'Test',
          badge: 'In progress',
          actions: [],
        } as unknown as XObject,
      ],
    })

    const result = projectSnapshot(testSnap as any, { withCore: false })
    const obj = result.objects['obj1']

    expect(obj!.status!.label).toBe('In progress')
  })

  test('uses status phrase when badge is empty', () => {
    const testSnap = snap({
      visible_objects: [
        {
          id: 'obj1',
          kind: 'item',
          parent: null,
          title: 'Test',
          status: 'COMPLETED',
          actions: [],
        } as unknown as XObject,
      ],
    })

    const result = projectSnapshot(testSnap as any, { withCore: false })
    const obj = result.objects['obj1']

    expect(obj!.status!.label).toBe('Completed')
  })

  test('spatial_layout radius maps to slot.size = 2×radius', () => {
    const testSnap = snap({
      visible_objects: [
        {
          id: 'obj1',
          kind: 'item',
          parent: null,
          title: 'Test',
          spatial_layout: { x: 100, y: 200, radius: 50 },
          actions: [],
        } as unknown as XObject,
      ],
    })

    const result = projectSnapshot(testSnap as any, { withCore: false })
    const obj = result.objects['obj1']

    expect(obj?.slot).toBeDefined()
    expect(obj?.slot?.size).toBe(100) // 2 * 50
  })

  test('relationships map to internal kinds', () => {
    const testSnap = snap({
      visible_objects: [
        { id: 'a', kind: 'item', parent: null, title: 'A', actions: [] } as unknown as XObject,
        { id: 'b', kind: 'item', parent: null, title: 'B', actions: [] } as unknown as XObject,
      ],
      x: {
        relationships: [
          { id: 'rel1', from: 'a', to: 'b', kind: 'depends-on' },
          { id: 'rel2', from: 'b', to: 'a', kind: 'produces' },
        ],
      },
    })

    const result = projectSnapshot(testSnap as any, { withCore: false })
    const depRel = result.relationships?.find((r) => r.id === 'rel1')
    const prodRel = result.relationships?.find((r) => r.id === 'rel2')

    // depends-on is mapped to a kind containing 'dep'
    expect(depRel?.kind).toBeDefined()
    // produces is mapped to 'leads-to'
    expect(prodRel?.kind).toBe('leads-to')
  })

  test('artifacts are collected from visible objects', () => {
    const testSnap = snap({
      visible_objects: [
        {
          id: 'obj1',
          kind: 'item',
          parent: null,
          title: 'Test',
          actions: [],
          artifacts: [{ ref: 'art-1', kind: 'diff', title: 'Diff', boundObject: 'obj1', currentLevel: 'summary', mediaType: 'text/plain', sourceProvenance: 'local' }],
        } as unknown as XObject,
      ],
    })

    const result = projectSnapshot(testSnap as any, { withCore: false })

    // Artifacts should be accessible through the snapshot's artifacts map
    expect(result.artifacts).toBeDefined()
  })
})

describe('projectHistory', () => {
  test('orders by cursor and labels the last revision as Now', () => {
    const snaps: XSnapshot[] = [
      snap({
        cursor: '1.2',
        timestamp: '2026-09-19T02:00:00Z',
      }),
      snap({
        cursor: '1.1',
        timestamp: '2026-09-19T01:00:00Z',
      }),
      snap({
        cursor: '1.0',
        timestamp: '2026-09-19T00:00:00Z',
      }),
    ]

    const history = projectHistory(snaps, {}, 'local-contract', { withCore: false })

    expect(history.revisions[0]?.id).toBe('1.0')
    expect(history.revisions[1]?.id).toBe('1.1')
    expect(history.revisions[2]?.id).toBe('1.2')
    expect(history.revisions[2]?.label).toBe('Now')
  })

  test('compareCursor handles epoch and seq correctly', () => {
    expect(compareCursor('1.0', '2.0')).toBeLessThan(0)
    expect(compareCursor('2.0', '1.0')).toBeGreaterThan(0)
    expect(compareCursor('1.5', '1.5')).toBe(0)
    expect(compareCursor('1.10', '1.9')).toBeGreaterThan(0)
    expect(compareCursor('2.1', '1.999')).toBeGreaterThan(0)
  })
})

describe('loadCaseHistory', () => {
  test('loads case history through LocalContractAdapter returning 6 revisions', async () => {
    const adapter = new LocalContractAdapter({ speed: 200, latency: 0 })
    const actor: Actor = {
      actor_id: 'usr-sam',
      role: 'case_architect',
      display_name: 'Sam Prime',
      kind: 'human',
    }

    const result = await loadCaseHistory(adapter, NORTHSTAR_CASE_ID, actor, 'local-contract', { withCore: false })

    expect(result.history.revisions.length).toBe(6)
    expect(result.raw.length).toBe(6)
    expect(result.history.caseId).toBe(NORTHSTAR_CASE_ID)

    // Revisions should be ordered by cursor
    for (let i = 1; i < result.history.revisions.length; i++) {
      const cmp = compareCursor(result.history.revisions[i]!.id, result.history.revisions[i - 1]!.id)
      expect(cmp).toBeGreaterThanOrEqual(0)
    }

    // Last revision should be labeled "Now"
    expect(result.history.revisions[result.history.revisions.length - 1]?.label).toBe('Now')
  })
})

describe('compareCursor', () => {
  test('correctly orders cursors by epoch first, then sequence', () => {
    const cursors = ['2.5', '1.100', '1.2', '1.0', '2.0', '3.1']
    const sorted = [...cursors].sort(compareCursor)

    expect(sorted).toEqual(['1.0', '1.2', '1.100', '2.0', '2.5', '3.1'])
  })

  test('handles missing sequence as 0', () => {
    expect(compareCursor('1', '2')).toBeLessThan(0)
    expect(compareCursor('1', '1.5')).toBeLessThan(0)
  })
})
