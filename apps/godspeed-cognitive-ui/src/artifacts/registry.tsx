import { Component, lazy, Suspense, type ComponentType, type ReactNode } from 'react'
import type { ArtifactPayload, CognitiveArtifact } from '../ports/contract'
import type { ArtifactModel, RendererKind } from './model'

// Source-level renderers are separate lazy chunks: a session that never opens a graph never
// downloads the graph renderer (VAR-006). Every renderer runs inside an error boundary, so a
// failing renderer is isolated and the source reference stays on screen (RECOV-004).

export interface SourceRendererProps {
  model: ArtifactModel
  payload: ArtifactPayload
  descriptor: CognitiveArtifact
  /** Search text from the viewer header; renderers highlight matches and report the count. */
  query: string
  onMatches?: (count: number) => void
  /** Focus a world object referenced by the artifact (graph nodes, trace targets). */
  onFocusObject?: (objectId: string) => void
  /** Move the world to a history cursor (timeline events). */
  onSetTime?: (cursor: string) => void
}

type Loader = () => Promise<{ default: ComponentType<SourceRendererProps> }>

/** Renderer kinds whose chunk has been requested this session (read by the E2E ladder through `__gs`). */
export const loadedRenderers = new Set<RendererKind>()

// Test hook: `?failRenderer=<kind>` makes that renderer throw, to exercise isolation (RECOV-004).
const failKind = typeof location !== 'undefined' ? new URLSearchParams(location.search).get('failRenderer') : null

const track = (kind: RendererKind, load: Loader): Loader => () => {
  loadedRenderers.add(kind)
  if (failKind === kind) return Promise.resolve({ default: Thrower })
  return load()
}

const LOADERS: Record<RendererKind, Loader> = {
  diff: track('diff', () => import('./renderers/DiffRenderer')),
  text: track('text', () => import('./renderers/TextRenderer')),
  markdown: track('markdown', () => import('./renderers/MarkdownRenderer')),
  table: track('table', () => import('./renderers/TableRenderer')),
  chart: track('chart', () => import('./renderers/ChartRenderer')),
  json: track('json', () => import('./renderers/JsonRenderer')),
  graph: track('graph', () => import('./renderers/GraphRenderer')),
  trace: track('trace', () => import('./renderers/TraceRenderer')),
  timeline: track('timeline', () => import('./renderers/TimelineRenderer')),
}

const LAZY = Object.fromEntries(
  Object.entries(LOADERS).map(([k, load]) => [k, lazy(load)]),
) as unknown as Record<RendererKind, ComponentType<SourceRendererProps>>

function Thrower(): ReactNode {
  throw new Error('Renderer failed (injected by ?failRenderer)')
}

export function SourceRenderer(props: SourceRendererProps & { fallbackRef: string }) {
  const R = LAZY[props.model.kind]
  return (
    <RendererBoundary key={`${props.fallbackRef}:${props.model.kind}`} artifactRef={props.fallbackRef} digest={props.payload.digest}>
      <Suspense fallback={<div className="gs-renderer-loading" role="status">Loading {props.model.kind} viewer…</div>}>
        <R {...props} />
      </Suspense>
    </RendererBoundary>
  )
}

interface BoundaryProps {
  artifactRef: string
  digest?: string
  children: ReactNode
}

export class RendererBoundary extends Component<BoundaryProps, { error: string | null }> {
  state = { error: null as string | null }
  static getDerivedStateFromError(e: unknown) {
    return { error: e instanceof Error ? e.message : String(e) }
  }
  componentDidCatch(e: unknown) {
    // Surfaced for the E2E ladder's error accounting; the UI itself stays up.
    console.warn('[artifact renderer isolated]', this.props.artifactRef, e)
  }
  render() {
    if (!this.state.error) return this.props.children
    return (
      <div className="gs-renderer-error" role="alert" data-testid="renderer-error">
        <strong>This viewer could not render the artifact.</strong>
        <span>{this.state.error}</span>
        <span className="gs-renderer-error-ref">
          Source: <code>{this.props.artifactRef}</code>
          {this.props.digest ? (
            <>
              {' '}· <code>{this.props.digest.slice(0, 19)}</code>
            </>
          ) : null}
        </span>
      </div>
    )
  }
}
