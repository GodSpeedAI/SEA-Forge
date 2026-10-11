import { useState, useEffect } from 'react'
import type { SourceRendererProps } from '../registry'
import type { TraceModel } from '../model'
import './TraceRenderer.css'

// Try to import highlight, but it's optional
let highlight: ((text: string, query: string) => { parts: Array<{ text: string; hit: boolean }>; count: number }) | null = null
try {
  highlight = (require('./highlight').highlight || null) as typeof highlight
} catch {
  // highlight.ts not yet available
}

export function TraceRenderer(props: SourceRendererProps) {
  const model = props.model as { kind: 'trace'; trace: TraceModel }
  const trace = model.trace

  const [selectedRowIndex, setSelectedRowIndex] = useState<number | null>(null)
  const [query, setQuery] = useState(props.query || '')

  // Report matches
  useEffect(() => {
    if (!query) {
      props.onMatches?.(0)
      return
    }

    if (highlight) {
      let count = 0
      count += highlight(trace.question.predicate, query).count
      count += highlight(trace.question.target_entity, query).count
      if (trace.claim) {
        count += highlight(trace.claim.proposition, query).count
      }
      for (const d of trace.discrepancies) {
        count += highlight(d.attribution_locus, query).count
        count += highlight(d.declared_state, query).count
        count += highlight(d.observed_state, query).count
      }
      props.onMatches?.(count)
    } else {
      // Fallback: simple substring match
      let count = 0
      const queryLower = query.toLowerCase()
      if (trace.question.predicate.toLowerCase().includes(queryLower)) count++
      if (trace.question.target_entity.toLowerCase().includes(queryLower)) count++
      if (trace.claim?.proposition.toLowerCase().includes(queryLower)) count++
      for (const d of trace.discrepancies) {
        if (d.attribution_locus.toLowerCase().includes(queryLower)) count++
        if (d.declared_state.toLowerCase().includes(queryLower)) count++
        if (d.observed_state.toLowerCase().includes(queryLower)) count++
      }
      props.onMatches?.(count)
    }
  }, [query, trace, props])

  const hasDiscrepancies = trace.discrepancy_count > 0

  const handleRowClick = (idx: number) => {
    setSelectedRowIndex(selectedRowIndex === idx ? null : idx)
  }

  const handleRowKeyDown = (idx: number, e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleRowClick(idx)
    }
  }

  const handleScroll = (e: React.WheelEvent) => {
    e.stopPropagation()
  }

  return (
    <div className="gs-r-trace-container">
      <div className="gs-r-trace-header">
        <div className="gs-r-trace-question">
          <div className="gs-r-trace-question-label">Question</div>
          <div>{trace.question.predicate}</div>
        </div>

        <div>
          <div className="gs-r-trace-question-label">Target Entity</div>
          <span className="gs-r-trace-target">{trace.question.target_entity}</span>
        </div>

        {trace.claim && (
          <div className="gs-r-trace-claim">
            <div className="gs-r-trace-claim-label">Claim</div>
            <div className="gs-r-trace-claim-text">{trace.claim.proposition}</div>
          </div>
        )}

        <div
          className={`gs-r-trace-summary ${hasDiscrepancies ? 'gs-r-trace-summary--critical' : 'gs-r-trace-summary--ok'}`}
          role="status"
        >
          {hasDiscrepancies
            ? `${trace.discrepancy_count} discrepanc${trace.discrepancy_count === 1 ? 'y' : 'ies'}`
            : 'No discrepancies · matches expectation'}
        </div>
      </div>

      <div className="gs-r-trace-body" onWheel={handleScroll} data-testid="renderer-trace">
        {trace.discrepancies.length === 0 ? (
          <div className="gs-r-trace-empty">No discrepancies to display</div>
        ) : (
          <table className="gs-r-trace-table">
            <thead className="gs-r-trace-thead">
              <tr>
                <th className="gs-r-trace-th">Locus</th>
                <th className="gs-r-trace-th">Expected</th>
                <th className="gs-r-trace-th">Observed</th>
              </tr>
            </thead>
            <tbody>
              {trace.discrepancies.map((discrepancy, idx) => {
                const isSelected = selectedRowIndex === idx
                const mismatch = discrepancy.declared_state !== discrepancy.observed_state

                return (
                  <tr
                    key={idx}
                    className="gs-r-trace-tr"
                    data-selected={isSelected ? 'true' : 'false'}
                    onClick={() => handleRowClick(idx)}
                    onKeyDown={e => handleRowKeyDown(idx, e)}
                    tabIndex={0}
                    role="button"
                  >
                    <td className="gs-r-trace-td gs-r-trace-td--locus">
                      {discrepancy.attribution_locus}
                    </td>
                    <td className="gs-r-trace-td gs-r-trace-td--expected">
                      {discrepancy.declared_state}
                    </td>
                    <td
                      className="gs-r-trace-td gs-r-trace-td--observed"
                      data-mismatch={mismatch ? 'true' : 'false'}
                    >
                      {discrepancy.observed_state}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        )}
      </div>

      <div className="gs-r-trace-toolbar">
        <input
          type="text"
          className="gs-r-trace-search"
          placeholder="Search discrepancies…"
          value={query}
          onChange={e => setQuery(e.target.value)}
          onKeyDown={e => e.stopPropagation()}
        />
        <div className="gs-r-trace-match-count">
          {query ? 'Matching' : ''}
        </div>
      </div>
    </div>
  )
}

export default TraceRenderer
