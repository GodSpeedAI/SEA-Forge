import type { XSnapshot, ActorRole } from '../../ports/contract'

export const TEMPLATE_CASE_ID = 'case-northstar-template'

// Design nodes from designFixture with spatial layout
const DESIGN_SLOTS: Record<string, { x: number; y: number; size: number }> = {
  'tpl-goal': { x: -120, y: -223, size: 56 },
  'tpl-stages': { x: 195, y: -188, size: 56 },
  'tpl-roles': { x: 323, y: -80, size: 60 },
  'tpl-evidence': { x: 290, y: 98, size: 60 },
  'tpl-milestones': { x: 199, y: 224, size: 62 },
  'tpl-governed': { x: -115, y: 255, size: 64 },
  'tpl-triggers': { x: -302, y: 159, size: 62 },
  'tpl-versions': { x: -366, y: 11, size: 58 },
  'tpl-preview': { x: -329, y: -130, size: 56 },
}

function buildTemplateSnapshot(
  version: '1' | '2',
  perspective: {
    actor_id: string
    role: ActorRole
    display_name?: string
  }
): XSnapshot {
  const cursor = version === '1' ? '1.0000000001' : '1.0000000002'
  const versionLabel = version === '1' ? 'v0.1 published' : 'v0.2 published'

  // Design nodes - all 9 for v0.2, only 8 without 'Verify' for v0.1
  const allDesignNodes = [
    {
      id: 'tpl-goal',
      title: 'Goal',
      subtitle: 'Resolve secondary coverage validation',
      icon: 'target' as const,
      accent: 'orange' as const,
    },
    {
      id: 'tpl-stages',
      title: 'Stages',
      subtitle: version === '1' ? '4 stages' : '5 stages',
      icon: 'layers' as const,
      accent: 'blue' as const,
      // Mock 12. v0.1 had no automated verification stage; v0.2 added it.
      items: [
        { id: 'stage-intake', label: 'Intake', detail: 'Capture request and assess · Entry' },
        ...(version === '1' ? [] : [{ id: 'stage-verify', label: 'Verify', detail: 'Run validation logic · Automated' }]),
        { id: 'stage-implement', label: 'Implement', detail: 'Apply updates · Human' },
        { id: 'stage-review', label: 'Review', detail: 'Quality check and approval · Human' },
        { id: 'stage-release', label: 'Release', detail: 'Complete and notify · Exit' },
      ],
    },
    {
      id: 'tpl-roles',
      title: 'Roles',
      subtitle: '3 roles',
      icon: 'people' as const,
      accent: 'dark' as const,
      items: [
        { id: 'r1', label: 'Case Architect', detail: 'Leads investigation', required: true },
        { id: 'r2', label: 'Code Reviewer', detail: 'Evaluates fixes', required: true },
        { id: 'r3', label: 'Release Approver', detail: 'Authorizes deployment', required: true },
      ],
    },
    {
      id: 'tpl-evidence',
      title: 'Evidence',
      subtitle: 'Required artifacts',
      icon: 'doc' as const,
      accent: 'orange' as const,
      items: [
        { id: 'e1', label: 'Customer Record', detail: 'EHR data extract', required: true },
        { id: 'e2', label: 'Code Change', detail: 'Implementation diff', required: true },
        { id: 'e3', label: 'Test Report', detail: 'Coverage validation', required: false },
      ],
    },
    {
      id: 'tpl-milestones',
      title: 'Milestones',
      subtitle: 'Key checkpoints',
      icon: 'flag' as const,
      accent: 'dark' as const,
    },
    {
      id: 'tpl-governed',
      title: 'Governed Work',
      subtitle: 'Policies & rules',
      icon: 'shield' as const,
      accent: 'blue' as const,
    },
    {
      id: 'tpl-triggers',
      title: 'Triggers',
      subtitle: 'Entry conditions',
      icon: 'bolt' as const,
      accent: 'green' as const,
    },
    {
      id: 'tpl-versions',
      title: 'Versions',
      subtitle: version === '1' ? 'Draft v0.1' : 'Draft v0.2',
      icon: 'history' as const,
      accent: 'dark' as const,
    },
    {
      id: 'tpl-preview',
      title: 'Preview',
      subtitle: 'Simulate case',
      icon: 'eye' as const,
      accent: 'dark' as const,
    },
  ]

  const visibleObjects = [
    {
      id: 'tpl-northstar',
      kind: 'work_item' as const,
      name: 'Northstar Case',
      status: 'IN_PROGRESS' as const,
      badge: 'Designing',
      explanation: 'Case Template',
      salience: 1,
      actions: [],
      x: {
        presentation: 'case' as const,
      },
    },
    ...allDesignNodes.map((node) => ({
      id: node.id,
      kind: 'stage' as const,
      name: node.title,
      status: 'ACTIVE' as const,
      badge: '',
      explanation: node.subtitle,
      salience: 0.5,
      parent_id: 'tpl-northstar',
      actions: [],
      spatial_layout: {
        x: DESIGN_SLOTS[node.id]?.x || 0,
        y: DESIGN_SLOTS[node.id]?.y || 0,
        z: 0,
        radius: (DESIGN_SLOTS[node.id]?.size || 56) / 2,
      },
      x: {
        presentation: 'facet' as const,
        design: {
          icon: node.icon,
          accent: node.accent,
          items: node.items,
        },
      },
    })),
  ]

  return {
    world_id: 'world-template',
    case_id: TEMPLATE_CASE_ID,
    cursor,
    timestamp: `2026-09-19T${version === '1' ? '08' : '10'}:00:00Z`,
    perspective,
    summary: {
      headline: 'Northstar Case Template',
      phase: `Version ${version}`,
      status_phrase: versionLabel,
      progress_percent: 100,
    },
    available_actions: [],
    attention_focus: {
      primary_object_id: 'tpl-northstar',
    },
    visible_objects: visibleObjects,
    x: {
      label: versionLabel,
      relationships: [],
    },
  }
}

export function buildTemplateHistory(perspective: {
  actor_id: string
  role: ActorRole
  display_name?: string
}): XSnapshot[] {
  return [buildTemplateSnapshot('1', perspective), buildTemplateSnapshot('2', perspective)]
}
