// Pure verdicts for the stub-gateway tooth (judged on results.json plus the cell's durable truth).

interface StepLike {
  name: string
  ok: boolean
  error?: string
}
interface ResultLike {
  journey_id: string
  status: string
  steps: StepLike[]
}

/**
 * Tooth (1): a gateway that accepts intents without a kernel must make L1 fail AT the durable step
 * (timeout waiting for the kernel's files), AND the cell must hold no case directory that was not
 * there before the ladder ran: the durable truth agrees that nothing was created. Returns null when
 * the tooth bit, else why not.
 */
export function stubGatewayToothProblem(results: ResultLike[], casesBefore: string[], casesAfter: string[]): string | null {
  const l1 = results.find((r) => r.journey_id === 'L1')
  const failed = l1?.steps.find((s) => !s.ok)
  if (l1?.status !== 'FAIL' || !failed) return `L1 status=${l1?.status} with no failed step`
  if (!failed.name.startsWith('durable:')) return `L1 failed at "${failed.name}" (${failed.error}), not at a durable step`
  if (!/timed out/.test(failed.error ?? '')) return `the durable step failed for another reason than waiting for kernel files: ${failed.error}`
  const created = casesAfter.filter((c) => !casesBefore.includes(c))
  if (created.length > 0) return `the cell gained case directories despite the stub: ${created.join(', ')}`
  return null
}
