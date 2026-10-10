import type { AgentNarrationStream, NarrationBeat, NarrationPort, ThothAnswerView, ThothAskRequest, ThothQuestionKind } from '../ports/contract'

// The live narration source: governed Thoth Ask (POST /api/ask). Thoth answers a finite set of
// typed questions (kind + subject), not free text, so a question is mapped onto that surface or
// answered honestly as unsupported. Every word of a grounded beat comes from the returned answer
// (claim statements, fixed disposition copy) and every citation from a returned reference: the
// client never invents claims, citations or directives, and an answer confers no authority.

export const THOTH_QUESTION_KINDS: readonly ThothQuestionKind[] = [
  'ask_capability',
  'ask_operation_requirements',
  'ask_authority_requirements',
  'ask_projection_support',
  'ask_environment_status',
  'ask_failure_explanation',
  'ask_evidence_for_claim',
  'ask_available_affordances',
  'ask_why_denied',
]

export type ParsedQuestion = { ok: true; request: ThothAskRequest } | { ok: false; reason: string }

/** "ask capability X", "capability X", "ask_why_denied X", "ask why-denied X" -> a typed request. */
export function parseThothQuestion(text: string, caseId: string): ParsedQuestion {
  const q = text.trim()
  for (const kind of THOTH_QUESTION_KINDS) {
    const words = kind.slice('ask_'.length).split('_').join('[\\s_-]+')
    const m = new RegExp(`^(?:ask[\\s_-]+)?${words}(?:\\s+([\\s\\S]+))?$`, 'i').exec(q)
    if (!m) continue
    const subject = (m[1] ?? '').trim()
    if (!subject) return { ok: false, reason: `Name a subject: “${kind.slice(4).replace(/_/g, ' ')} <subject>”.` }
    return { ok: true, request: { kind, subject, ...(caseId ? { case: caseId } : {}) } }
  }
  return { ok: false, reason: 'Thoth answers typed questions, not free text.' }
}

export const UNSUPPORTED_COPY = (reason: string) =>
  `${reason} Try “ask ${THOTH_QUESTION_KINDS[0]!.slice(4).replace(/_/g, ' ')} <subject>” — kinds: ${THOTH_QUESTION_KINDS.map((k) => k.slice(4).replace(/_/g, ' ')).join(', ')}.`

/** Beats for one answer: one per returned claim (or fixed disposition copy), then the disclosure terms. */
export function beatsOfAnswer(answer: ThothAnswerView): NarrationBeat[] {
  const beats: NarrationBeat[] = []
  for (const claim of answer.claims) {
    beats.push({
      index: beats.length,
      thoughtText: claim.statement,
      evidenceCitations: [...claim.evidence_refs, ...claim.settlement_refs, ...(claim.capability_record_ref ? [claim.capability_record_ref] : [])],
      grounded_answer: answer,
    })
  }
  if (answer.claims.length === 0) {
    beats.push({
      index: 0,
      thoughtText: `Thoth ${answer.disposition === 'denied' ? 'denied this disclosure' : 'returned no claims'} (${answer.disposition}).`,
      evidenceCitations: [],
      grounded_answer: answer,
    })
  }
  beats.push({
    index: beats.length,
    thoughtText: `${answer.disposition}, ${answer.freshness} · ${answer.authority_notice}`,
    evidenceCitations: [],
    grounded_answer: answer,
  })
  return beats
}

function streamOf(produce: (signal: AbortSignal) => Promise<NarrationBeat[]>): AgentNarrationStream {
  const abort = new AbortController()
  return {
    beats: (async function* () {
      let beats: NarrationBeat[]
      try {
        beats = await produce(abort.signal)
      } catch (e) {
        if (abort.signal.aborted) return
        throw e
      }
      for (const b of beats) {
        if (abort.signal.aborted) return
        yield b
      }
    })(),
    interrupt: () => abort.abort(),
  }
}

export interface AskSource {
  ask(request: ThothAskRequest, signal?: AbortSignal): Promise<ThothAnswerView>
}

export function createThothNarration(source: AskSource): NarrationPort {
  return {
    explain(question, ctx) {
      const parsed = parseThothQuestion(question, ctx.caseId)
      if (!parsed.ok) {
        const reason = parsed.reason
        return streamOf(async () => [{ index: 0, thoughtText: UNSUPPORTED_COPY(reason), evidenceCitations: [] }])
      }
      return streamOf(async (signal) => {
        try {
          return beatsOfAnswer(await source.ask(parsed.request, signal))
        } catch (e) {
          // Typed refusals (session ended, rate limited, denied by the kernel) are honest answers;
          // a lost transport ends the narration and marks the agent unavailable (direct UI stays usable).
          if ((e as { name?: string }).name === 'HttpRefusalError' && !/^(UNAVAILABLE|unavailable)$/.test((e as { refusalKind?: string }).refusalKind ?? '')) {
            return [{ index: 0, thoughtText: `Thoth could not answer: ${(e as Error).message}`, evidenceCitations: [] }]
          }
          throw e
        }
      })
    },
  }
}
