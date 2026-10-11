import type { CognitiveArtifact } from '../ports/contract'
import type { ArtifactState } from '../artifacts/service'
import type { ArtifactModel } from '../artifacts/model'
import { RENDERER_LABEL } from '../artifacts/model'
import './artifact-excerpt.css'

export interface ArtifactExcerptProps {
  descriptor: CognitiveArtifact
  state: ArtifactState
  pinned: boolean
  onExpand(): void
  onDismiss(): void
  onPin(pinned: boolean): void
}

function renderPreview(model: ArtifactModel): JSX.Element {
  switch (model.kind) {
    case 'diff': {
      const files = model.files
      const totalAdded = files.reduce((sum, f) => sum + f.added, 0)
      const totalRemoved = files.reduce((sum, f) => sum + f.removed, 0)
      const firstLines = files.length > 0 ? files[0]!.lines.slice(0, 3) : []
      return (
        <div className="artifact-excerpt--diff">
          {firstLines.map((line, i) => (
            <div
              key={i}
              className={`diff-line diff-sign-${line.sign}`}
              data-sign={line.sign}
            >
              <span className="diff-sign">{line.sign === '@' ? '@@' : line.sign}</span>
              <span className="diff-text">{line.text}</span>
            </div>
          ))}
          <div className="artifact-excerpt--summary">
            {files.length} files · <span className="added">+{totalAdded}</span> <span className="removed">−{totalRemoved}</span>
          </div>
        </div>
      )
    }

    case 'table': {
      const { table } = model
      return (
        <table className="artifact-excerpt--table">
          <thead>
            <tr>
              {table.columns.map((col, i) => (
                <th key={i}>{col}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {table.rows.slice(0, 3).map((row, i) => (
              <tr key={i}>
                {row.map((cell, j) => (
                  <td key={j}>{cell}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      )
    }

    case 'chart': {
      const { chart } = model
      const max = Math.max(...chart.series.map((s) => s.value), 1)
      const width = 240
      const height = 60
      const barWidth = width / chart.series.length
      const svgPoints = chart.series.map((s, i) => {
        const barHeight = (s.value / max) * (height - 8)
        const x = i * barWidth + barWidth / 2
        const y = height - 4 - barHeight
        return { x, y, height: barHeight, value: s.value }
      })

      return (
        <div className="artifact-excerpt--chart-container">
          <svg
            className="artifact-excerpt--chart"
            width={width}
            height={height}
            viewBox={`0 0 ${width} ${height}`}
            aria-label={`Chart: ${chart.unit}`}
          >
            {svgPoints.map((p, i) => (
              <rect
                key={i}
                x={p.x - barWidth / 2 + 2}
                y={p.y}
                width={barWidth - 4}
                height={p.height}
                fill="var(--tone-progress)"
                opacity={0.7}
              />
            ))}
          </svg>
          <div className="artifact-excerpt--chart-info">
            Max: {Math.max(...chart.series.map((s) => s.value))} {chart.unit}
          </div>
        </div>
      )
    }

    case 'markdown': {
      const block = model.blocks.find((b) => b.type === 'quote' || b.type === 'p')
      return (
        <div className="artifact-excerpt--markdown">
          {block ? (
            <p className="artifact-excerpt--text">{block.text}</p>
          ) : model.blocks.length > 0 ? (
            <p className="artifact-excerpt--text">{model.blocks[0]!.text}</p>
          ) : null}
        </div>
      )
    }

    case 'text': {
      return (
        <div className="artifact-excerpt--text-lines">
          {model.lines.slice(0, 3).map((line, i) => (
            <div key={i} className="artifact-excerpt--text-line">
              {line}
            </div>
          ))}
        </div>
      )
    }

    case 'graph': {
      const nodeCount = model.graph.nodes.length
      const edgeCount = model.graph.edges.length
      return (
        <div className="artifact-excerpt--summary">
          {nodeCount} nodes · {edgeCount} links
        </div>
      )
    }

    case 'trace': {
      const { trace } = model
      const isMatch = trace.discrepancy_count === 0
      return (
        <div className="artifact-excerpt--trace">
          <div
            className={`artifact-excerpt--trace-status ${isMatch ? 'ok' : 'critical'}`}
          >
            {isMatch
              ? 'Matches expectation'
              : `${trace.discrepancy_count} ${trace.discrepancy_count === 1 ? 'discrepancy' : 'discrepancies'}`}
          </div>
          {trace.discrepancies.length > 0 && (
            <div className="artifact-excerpt--trace-detail">
              Expected: {trace.discrepancies[0]!.declared_state} / Observed:{' '}
              {trace.discrepancies[0]!.observed_state}
            </div>
          )}
        </div>
      )
    }

    case 'timeline': {
      const { timeline } = model
      return (
        <div className="artifact-excerpt--timeline">
          {timeline.events.slice(-2).map((event, i) => (
            <div key={i} className="artifact-excerpt--timeline-event">
              <span className="artifact-excerpt--timeline-label">{event.label}</span>
              {event.summary && (
                <span className="artifact-excerpt--timeline-summary">{event.summary}</span>
              )}
            </div>
          ))}
        </div>
      )
    }

    case 'json': {
      const keys = Object.keys(model.value as Record<string, unknown>).slice(0, 5)
      return (
        <div className="artifact-excerpt--json">
          {keys.map((key) => (
            <div key={key} className="artifact-excerpt--json-key">
              {key}
            </div>
          ))}
        </div>
      )
    }
  }
}

export function ArtifactExcerpt(p: ArtifactExcerptProps): JSX.Element {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      p.onExpand()
    }
  }

  const handleClick = (e: React.MouseEvent) => {
    if (
      (e.target as HTMLElement).closest(
        '.artifact-excerpt--footer, .artifact-excerpt--button'
      )
    )
      return
    p.onExpand()
  }

  const title = p.state.status === 'ready' ? p.state.payload.name : p.descriptor.title
  const kindLabel =
    p.state.status === 'ready'
      ? RENDERER_LABEL[p.state.model.kind]
      : 'Artifact'

  return (
    <div
      className="artifact-excerpt"
      data-testid="artifact-excerpt"
      data-ref={p.descriptor.ref}
      role="button"
      tabIndex={0}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
    >
      {/* Header */}
      <div className="artifact-excerpt--header">
        <div className="artifact-excerpt--title-group">
          <div className="artifact-excerpt--title">{title}</div>
          <span className="artifact-excerpt--kind-badge">{kindLabel}</span>
        </div>

        <div className="artifact-excerpt--header-buttons">
          <button
            className="artifact-excerpt--button"
            onClick={(e) => {
              e.stopPropagation()
              p.onPin(!p.pinned)
            }}
            aria-pressed={p.pinned}
            title={p.pinned ? 'Keep available (local)' : 'Keep available (local)'}
            data-testid="excerpt-pin"
          >
            {p.pinned ? '📌' : '📍'}
          </button>
          <button
            className="artifact-excerpt--button"
            onClick={(e) => {
              e.stopPropagation()
              p.onDismiss()
            }}
            title="Dismiss"
            data-testid="excerpt-dismiss"
          >
            ×
          </button>
        </div>
      </div>

      {/* Body */}
      <div className="artifact-excerpt--body">
        {p.state.status === 'loading' && (
          <div className="artifact-excerpt--status">Resolving source…</div>
        )}
        {p.state.status === 'error' && (
          <div data-testid="artifact-error" className="artifact-excerpt--error">
            <div className="artifact-excerpt--error-message">
              Could not load this artifact
            </div>
            <div className="artifact-excerpt--error-text">{p.state.error}</div>
            <div className="artifact-excerpt--error-ref">
              <code>{p.descriptor.ref}</code>
            </div>
          </div>
        )}
        {p.state.status === 'ready' && renderPreview(p.state.model)}
      </div>

      {/* Footer */}
      <div className="artifact-excerpt--footer">
        <button
          className="artifact-excerpt--expand-link"
          onClick={p.onExpand}
          data-testid="excerpt-expand"
        >
          Expand
        </button>
        <span className="artifact-excerpt--ref">{p.descriptor.ref}</span>
      </div>
    </div>
  )
}
