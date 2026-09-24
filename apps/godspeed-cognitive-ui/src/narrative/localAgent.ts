import type { AgentNarrationStream, NarrationBeat, NarrationPort } from '../ports/contract'

// A scripted, local stand-in for the agent side (spec 04 §8). It streams narration beats whose
// directives use only the UI's declared vocabulary: focus, camera, highlight, annotate,
// openArtifact and temporalStep, plus the proposed x-directives for arrange, compare and reveal.
// It knows the Northstar world the way an agent would: by the ids in the snapshots it was given.
// There is no LLM. The UI never depends on this port being present.

interface Script {
  id: string
  triggers: string[]
  /** Only offered when the case in context matches. */
  caseId: string
  beats: Omit<NarrationBeat, 'index'>[]
}

const SCRIPTS: Script[] = [
  {
    id: 'why-pilot-failed',
    caseId: 'case-northstar',
    triggers: ['why did the pilot fail', 'why did northstar fail', 'why did this fail', 'why did it fail', 'why this failed', 'why'],
    beats: [
      {
        thoughtText: 'The implementation itself worked.',
        evidenceCitations: ['evi-impl-graph'],
        directives: [{ focus: 'northstar', x: { arrange: 'orbital', reveal: ['ns-impl'], dim: ['ns-goal', 'ns-release', 'ns-workflow', 'ns-evidence'] } }, { annotate: [{ target: 'ns-impl', label: 'Verified' }] }],
      },
      {
        thoughtText: 'The coverage assumption was wrong.',
        evidenceCitations: ['evi-record'],
        directives: [
          { x: { reveal: ['ns-assumption', 'ns-record'], dim: ['ns-impl', 'ns-goal', 'ns-release', 'ns-workflow', 'ns-secondary'] } },
          { openArtifact: { ref: 'evi-record', level: 'summary' } },
        ],
      },
      {
        thoughtText: '1 path diverged',
        evidenceCitations: ['evi-coverage-trace'],
        directives: [{ x: { arrange: 'causal', dim: [] } }, { openArtifact: { ref: 'evi-coverage-trace', level: 'summary' } }],
      },
      {
        thoughtText: 'This is the resulting failure.',
        evidenceCitations: ['evi-coverage-trace'],
        directives: [{ x: { reveal: ['ns-failure'] }, highlight: ['ns-failure'] }],
      },
    ],
  },
  {
    id: 'evidence',
    caseId: 'case-northstar',
    triggers: ['what is the evidence', "what's the evidence", 'show me the evidence', 'why do we think that', 'prove it', 'evidence'],
    beats: [
      {
        thoughtText: 'Rejections began on Sep 16.',
        evidenceCitations: ['evi-rejections-chart'],
        directives: [{ focus: 'northstar', x: { arrange: 'orbital' } }, { openArtifact: { ref: 'evi-rejections-chart', level: 'summary' } }],
      },
      {
        thoughtText: 'Only secondary-coverage paths fail.',
        evidenceCitations: ['evi-checks-table'],
        directives: [{ openArtifact: { ref: 'evi-checks-table', level: 'summary' } }],
      },
      {
        thoughtText: 'The pilot scope assumed primary only.',
        evidenceCitations: ['evi-assumption'],
        directives: [{ x: { reveal: ['ns-assumption'] } }, { openArtifact: { ref: 'evi-assumption', level: 'summary' } }],
      },
    ],
  },
  {
    id: 'how-we-got-here',
    caseId: 'case-northstar',
    triggers: ['walk me through how we got here', 'how did we get here', 'go back before this changed', 'show history', 'what changed'],
    beats: [
      {
        thoughtText: 'It started as a planning assumption.',
        evidenceCitations: ['evi-assumption'],
        directives: [{ focus: 'northstar', x: { arrange: 'orbital', temporalTo: '1.0000000002', reveal: ['ns-assumption'] } }, { openArtifact: { ref: 'evi-assumption', level: 'summary' } }],
      },
      { thoughtText: 'The customer record was examined.', evidenceCitations: ['evi-record'], directives: [{ temporalStep: 1 }] },
      { thoughtText: 'The coverage gap was confirmed.', evidenceCitations: ['evi-coverage-trace'], directives: [{ temporalStep: 1 }] },
      {
        thoughtText: 'Here is what changed since the plan.',
        evidenceCitations: ['evi-case-timeline'],
        directives: [{ x: { compare: { a: '1.0000000002', b: 'live' } } }],
      },
    ],
  },
]

export function normalize(text: string) {
  return text.toLowerCase().replace(/[?!.,;:]+/g, ' ').replace(/\s+/g, ' ').trim()
}

function match(question: string, caseId: string): Script | null {
  const q = normalize(question)
  for (const s of SCRIPTS) {
    if (s.caseId !== caseId) continue
    if (s.triggers.some((t) => q === t || q.includes(t) || (t.includes(q) && q.length > 8))) return s
  }
  return null
}

export interface LocalAgentOptions {
  /** ms between streamed beats. */
  pace?: number
  /** Simulate losing the agent adapter after this many beats (RECOV-003). */
  failAfter?: number
}

export function createLocalAgent(opts: LocalAgentOptions = {}): NarrationPort {
  const pace = opts.pace ?? 250
  return {
    explain(question, ctx) {
      const script = match(question, ctx.caseId)
      if (!script) return null
      let stopped = false
      const beats: AsyncIterable<NarrationBeat> = {
        async *[Symbol.asyncIterator]() {
          for (let i = 0; i < script.beats.length; i++) {
            await new Promise((r) => setTimeout(r, pace))
            if (stopped) return
            if (opts.failAfter !== undefined && i >= opts.failAfter) throw new Error('Agent adapter disconnected')
            yield { index: i, ...script.beats[i]! }
          }
        },
      }
      const stream: AgentNarrationStream = { beats, interrupt: () => void (stopped = true) }
      return Object.assign(stream, { id: script.id })
    },
  }
}
