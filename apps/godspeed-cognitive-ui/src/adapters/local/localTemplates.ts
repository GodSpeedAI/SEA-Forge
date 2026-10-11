import type { TemplateEntryOption } from '../../ports/contract'

// Fixture templates for the local adapter (T09): the TemplateSourcePort the live gateway serves
// (GET /api/templates, POST /api/templates/preflight), authored locally so the local ladder covers
// the case-design journey. Each template also carries a non-wire `plan` describing the case world
// the local adapter builds on PROPOSE_CASE — the kernel's planner does that on the live path.

export interface LocalTemplatePlanStage {
  id: string
  name: string
  explanation: string
  /** Earlier stages this one waits on (sentry chain). */
  depends_on?: string[]
  /** The enabled action the stage offers once active. */
  action?: { id: string; label: string; intent: 'EXECUTE_ITEM' | 'APPROVE_HUMAN_TASK'; requires_justification?: boolean }
}

export interface LocalTemplate extends TemplateEntryOption {
  /** Local-only plan shape; not part of the wire contract. */
  plan: { stages: LocalTemplatePlanStage[] }
}

export const LOCAL_TEMPLATES: readonly LocalTemplate[] = [
  {
    template_ref: 'tpl-release-rollout',
    title: 'Staged release rollout',
    description: 'A sandboxed rollout followed by a dependent verification gate that needs sign-off.',
    parameters: [
      { name: 'release_version', title: 'Release version', param_type: 'string', required: true },
      { name: 'cohort_percent', title: 'Initial cohort (%)', param_type: 'number', required: false, default: '10' },
    ],
    plan: {
      stages: [
        {
          id: 'st-rollout',
          name: 'Rollout',
          explanation: 'Apply the release to the initial cohort in the sandbox',
          action: { id: 'act-execute', label: 'Execute rollout', intent: 'EXECUTE_ITEM' },
        },
        {
          id: 'st-verify',
          name: 'Verify',
          explanation: 'Check the rollout evidence before sign-off',
          depends_on: ['st-rollout'],
        },
        {
          id: 'st-signoff',
          name: 'Sign-off',
          explanation: 'Human gate: approve or reject the rollout',
          depends_on: ['st-verify'],
          action: { id: 'act-approve', label: 'Approve rollout', intent: 'APPROVE_HUMAN_TASK' },
        },
      ],
    },
  },
  {
    template_ref: 'tpl-evidence-review',
    title: 'Evidence review',
    description: 'A single human review gate over an attached evidence bundle.',
    parameters: [{ name: 'subject', title: 'Review subject', param_type: 'string', required: true }],
    plan: {
      stages: [
        {
          id: 'st-review',
          name: 'Review',
          explanation: 'Examine the evidence and record the review',
          action: { id: 'act-complete', label: 'Complete review', intent: 'EXECUTE_ITEM' },
        },
      ],
    },
  },
]

export const templateByRef = (ref: string): LocalTemplate | undefined => LOCAL_TEMPLATES.find((t) => t.template_ref === ref)

/**
 * Local preflight: the required parameters must be non-empty; the digest binds the template and
 * the exact params so a commit can prove it echoes a reviewed preflight (SHA-256 via WebCrypto,
 * same primitive the kernel content-addresses with).
 */
export async function localPreflight(
  templateRef: string,
  params: Record<string, unknown>,
): Promise<{ passed: boolean; digest?: string; reasons: readonly string[] }> {
  const template = templateByRef(templateRef)
  if (!template) return { passed: false, reasons: [`Unknown template ${templateRef}`] }
  const reasons: string[] = []
  for (const p of template.parameters) {
    const v = params[p.name]
    if (p.required && (v === undefined || v === null || String(v).trim() === '')) reasons.push(`Missing required parameter “${p.title ?? p.name}”`)
  }
  if (reasons.length) return { passed: false, reasons }
  return { passed: true, digest: await digestOf(templateRef, params), reasons: [] }
}

/** Deterministic content digest over the template ref and the exact committed params. */
export async function digestOf(templateRef: string, params: Record<string, unknown>): Promise<string> {
  const canonical = `${templateRef}\u0000${JSON.stringify(params, Object.keys(params).sort())}`
  const bytes = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(canonical)))
  return 'sha256:' + [...bytes].map((b) => b.toString(16).padStart(2, '0')).join('')
}
