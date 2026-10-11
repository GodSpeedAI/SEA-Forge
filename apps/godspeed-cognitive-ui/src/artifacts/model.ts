import type { ArtifactPayload } from '../ports/contract'

// Contract payload (ArtifactPayload: content_type + content string) → typed model for a renderer.
// Parsing is light and lives in the main bundle. The source-level renderers are lazy chunks
// (see registry.tsx). A payload that does not parse becomes a bounded error; it never throws
// past this module.
//
// Content conventions (documented in 03-CONTRACT-PORTS-AND-SEAMS.md):
//   text/x-diff        unified diff ("diff --git", "--- a/", "+++ b/", "@@ -a,b +c,d @@")
//   text/markdown      document (#/##/### headings, paragraphs, "- " lists, "> " quotes, ``` code)
//   text/plain         text or log lines
//   application/json   object with a "schema" discriminator: table | chart | graph | realitytrace | timeline;
//                      anything else is shown as a JSON tree

export type RendererKind = 'diff' | 'table' | 'chart' | 'graph' | 'trace' | 'timeline' | 'json' | 'markdown' | 'text'

export interface DiffLine {
  sign: '+' | '-' | ' ' | '@'
  text: string
  /** Old and new line numbers (absent on the side where the line does not exist). */
  oldN?: number
  newN?: number
}
export interface DiffFile {
  path: string
  added: number
  removed: number
  lines: DiffLine[]
}

export interface TableModel {
  columns: string[]
  rows: string[][]
  caption?: string
}
export interface ChartModel {
  unit: string
  series: { label: string; value: number }[]
  caption?: string
}
export interface GraphModel {
  nodes: { id: string; label: string; object_id?: string; kind?: string }[]
  edges: { from: string; to: string; label?: string }[]
}
export interface TraceModel {
  question: { question_id: string; predicate: string; target_entity: string }
  claim?: { proposition: string; asserted_by?: string }
  discrepancy_count: number
  discrepancies: { attribution_locus: string; declared_state: string; observed_state: string }[]
}
export interface TimelineModel {
  events: { at: string; label: string; summary?: string; cursor?: string }[]
}
export interface MarkdownBlock {
  type: 'h1' | 'h2' | 'h3' | 'p' | 'li' | 'quote' | 'code'
  text: string
}

export type ArtifactModel =
  | { kind: 'diff'; files: DiffFile[] }
  | { kind: 'table'; table: TableModel }
  | { kind: 'chart'; chart: ChartModel }
  | { kind: 'graph'; graph: GraphModel }
  | { kind: 'trace'; trace: TraceModel }
  | { kind: 'timeline'; timeline: TimelineModel }
  | { kind: 'json'; value: unknown }
  | { kind: 'markdown'; blocks: MarkdownBlock[] }
  | { kind: 'text'; lines: string[] }

export type ParseResult = { ok: true; model: ArtifactModel } | { ok: false; error: string }

export function parseArtifact(p: ArtifactPayload): ParseResult {
  try {
    if (typeof p.content !== 'string') return { ok: false, error: 'Payload has no content' }
    switch (p.content_type) {
      case 'text/x-diff': {
        const files = parseDiff(p.content)
        return files.length ? { ok: true, model: { kind: 'diff', files } } : { ok: false, error: 'Diff has no files' }
      }
      case 'text/markdown':
        return { ok: true, model: { kind: 'markdown', blocks: parseMarkdown(p.content) } }
      case 'text/plain':
        return { ok: true, model: { kind: 'text', lines: p.content.split('\n') } }
      case 'application/json':
        return parseJson(p.content)
      default:
        return { ok: false, error: `Unsupported content type ${String(p.content_type)}` }
    }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) }
  }
}

function parseJson(text: string): ParseResult {
  let v: unknown
  try {
    v = JSON.parse(text)
  } catch {
    return { ok: false, error: 'Content is not valid JSON' }
  }
  const o = v as Record<string, unknown>
  const bad = (what: string): ParseResult => ({ ok: false, error: `Malformed ${what} artifact` })
  switch (o && typeof o === 'object' ? o.schema : undefined) {
    case 'table': {
      if (!Array.isArray(o.columns) || !Array.isArray(o.rows)) return bad('table')
      const rows = (o.rows as unknown[]).map((r) => (Array.isArray(r) ? r.map((c) => String(c)) : null))
      if (rows.some((r) => !r)) return bad('table')
      return { ok: true, model: { kind: 'table', table: { columns: (o.columns as unknown[]).map(String), rows: rows as string[][], caption: str(o.caption) } } }
    }
    case 'chart': {
      if (!Array.isArray(o.series)) return bad('chart')
      const series = (o.series as { label?: unknown; value?: unknown }[]).map((s) => ({ label: String(s.label), value: Number(s.value) }))
      if (series.some((s) => !Number.isFinite(s.value))) return bad('chart')
      return { ok: true, model: { kind: 'chart', chart: { unit: String(o.unit ?? ''), series, caption: str(o.caption) } } }
    }
    case 'graph': {
      if (!Array.isArray(o.nodes) || !Array.isArray(o.edges)) return bad('graph')
      return { ok: true, model: { kind: 'graph', graph: { nodes: o.nodes as GraphModel['nodes'], edges: o.edges as GraphModel['edges'] } } }
    }
    case 'realitytrace': {
      const ev = o.evaluation as Record<string, unknown> | undefined
      if (!o.question || !ev || !Array.isArray(ev.discrepancies)) return bad('trace')
      return {
        ok: true,
        model: {
          kind: 'trace',
          trace: {
            question: o.question as TraceModel['question'],
            claim: o.claim as TraceModel['claim'],
            discrepancy_count: Number(ev.discrepancy_count ?? (ev.discrepancies as unknown[]).length),
            discrepancies: ev.discrepancies as TraceModel['discrepancies'],
          },
        },
      }
    }
    case 'timeline': {
      if (!Array.isArray(o.events)) return bad('timeline')
      return { ok: true, model: { kind: 'timeline', timeline: { events: o.events as TimelineModel['events'] } } }
    }
    default:
      return { ok: true, model: { kind: 'json', value: v } }
  }
}

const str = (v: unknown) => (typeof v === 'string' ? v : undefined)

export function parseDiff(text: string): DiffFile[] {
  const files: DiffFile[] = []
  let cur: DiffFile | null = null
  let oldN = 0
  let newN = 0
  for (const line of text.split('\n')) {
    if (line.startsWith('diff --git ')) {
      const m = / b\/(.+)$/.exec(line)
      cur = { path: m?.[1] ?? line.slice(11), added: 0, removed: 0, lines: [] }
      files.push(cur)
      continue
    }
    if (!cur) continue
    if (line.startsWith('--- ') || line.startsWith('+++ ') || line.startsWith('index ')) continue
    if (line.startsWith('@@')) {
      const m = /@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@(.*)/.exec(line)
      oldN = m ? Number(m[1]) : 0
      newN = m ? Number(m[2]) : 0
      cur.lines.push({ sign: '@', text: line })
      continue
    }
    const sign = line[0]
    if (sign === '+') {
      cur.added++
      cur.lines.push({ sign: '+', text: line.slice(1), newN: newN++ })
    } else if (sign === '-') {
      cur.removed++
      cur.lines.push({ sign: '-', text: line.slice(1), oldN: oldN++ })
    } else if (sign === ' ' || line === '') {
      if (line === '' && cur.lines.length === 0) continue
      cur.lines.push({ sign: ' ', text: line.slice(1), oldN: oldN++, newN: newN++ })
    }
  }
  // Drop trailing blank context produced by a final newline.
  for (const f of files) while (f.lines.length && f.lines[f.lines.length - 1]!.sign === ' ' && f.lines[f.lines.length - 1]!.text === '') f.lines.pop()
  return files
}

export function parseMarkdown(text: string): MarkdownBlock[] {
  const out: MarkdownBlock[] = []
  const lines = text.split('\n')
  let para: string[] = []
  const flush = () => {
    if (para.length) out.push({ type: 'p', text: para.join(' ') })
    para = []
  }
  for (let i = 0; i < lines.length; i++) {
    const l = lines[i]!
    if (l.startsWith('```')) {
      flush()
      const code: string[] = []
      while (++i < lines.length && !lines[i]!.startsWith('```')) code.push(lines[i]!)
      out.push({ type: 'code', text: code.join('\n') })
    } else if (/^###\s/.test(l)) (flush(), out.push({ type: 'h3', text: l.replace(/^###\s+/, '') }))
    else if (/^##\s/.test(l)) (flush(), out.push({ type: 'h2', text: l.replace(/^##\s+/, '') }))
    else if (/^#\s/.test(l)) (flush(), out.push({ type: 'h1', text: l.replace(/^#\s+/, '') }))
    else if (/^\s*[-*]\s/.test(l)) (flush(), out.push({ type: 'li', text: l.replace(/^\s*[-*]\s+/, '') }))
    else if (/^>\s?/.test(l)) (flush(), out.push({ type: 'quote', text: l.replace(/^>\s?/, '') }))
    else if (!l.trim()) flush()
    else para.push(l.trim())
  }
  flush()
  return out
}

/** Human label for the renderer that will show a model. */
export const RENDERER_LABEL: Record<RendererKind, string> = {
  diff: 'Diff',
  table: 'Table',
  chart: 'Chart',
  graph: 'Graph',
  trace: 'Expected vs observed',
  timeline: 'Timeline',
  json: 'Data',
  markdown: 'Document',
  text: 'Text',
}
