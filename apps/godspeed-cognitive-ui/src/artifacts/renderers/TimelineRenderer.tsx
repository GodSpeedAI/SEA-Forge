import { useState, useEffect } from 'react'
import type { SourceRendererProps } from '../registry'
import type { TimelineModel } from '../model'
import './TimelineRenderer.css'

// Try to import highlight, but it's optional
let highlight: ((text: string, query: string) => { parts: Array<{ text: string; hit: boolean }>; count: number }) | null = null
try {
  highlight = (require('./highlight').highlight || null) as typeof highlight
} catch {
  // highlight.ts not yet available
}

export function TimelineRenderer(props: SourceRendererProps) {
  const model = props.model as { kind: 'timeline'; timeline: TimelineModel }
  const timeline = model.timeline

  const [selectedEventIndex, setSelectedEventIndex] = useState<number | null>(null)
  const [query, setQuery] = useState(props.query || '')

  // Report matches
  useEffect(() => {
    if (!query) {
      props.onMatches?.(0)
      return
    }

    if (highlight) {
      let count = 0
      for (const event of timeline.events) {
        count += highlight(event.label, query).count
        if (event.summary) {
          count += highlight(event.summary, query).count
        }
      }
      props.onMatches?.(count)
    } else {
      // Fallback: simple substring match
      let count = 0
      const queryLower = query.toLowerCase()
      for (const event of timeline.events) {
        if (event.label.toLowerCase().includes(queryLower)) count++
        if (event.summary?.toLowerCase().includes(queryLower)) count++
      }
      props.onMatches?.(count)
    }
  }, [query, timeline.events, props])

  const selectedEvent = selectedEventIndex !== null ? timeline.events[selectedEventIndex] : null

  const handleEventClick = (idx: number) => {
    setSelectedEventIndex(selectedEventIndex === idx ? null : idx)
  }

  const handleEventKeyDown = (idx: number, e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleEventClick(idx)
    }
  }

  const handleSetTime = () => {
    if (selectedEvent?.cursor) {
      props.onSetTime?.(selectedEvent.cursor)
    }
  }

  const handleScroll = (e: React.WheelEvent) => {
    e.stopPropagation()
  }

  const formatDate = (isoString: string): string => {
    try {
      const date = new Date(isoString)
      return date.toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      })
    } catch {
      return isoString
    }
  }

  return (
    <div className="gs-r-timeline-container">
      <div className="gs-r-timeline-toolbar">
        <input
          type="text"
          className="gs-r-timeline-search"
          placeholder="Search events…"
          value={query}
          onChange={e => setQuery(e.target.value)}
          onKeyDown={e => e.stopPropagation()}
        />
        <div className="gs-r-timeline-match-count">
          {query ? 'Matching' : ''}
        </div>
      </div>

      <div className="gs-r-timeline-body" onWheel={handleScroll} data-testid="renderer-timeline">
        {timeline.events.length === 0 ? (
          <div className="gs-r-timeline-empty">No events in timeline</div>
        ) : (
          <div className="gs-r-timeline-events">
            {timeline.events.map((event, idx) => {
              const isSelected = selectedEventIndex === idx
              const isHit =
                query &&
                (highlight
                  ? highlight(event.label, query).count > 0 ||
                    (event.summary ? highlight(event.summary, query).count > 0 : false)
                  : event.label.toLowerCase().includes(query.toLowerCase()) ||
                    (event.summary?.toLowerCase().includes(query.toLowerCase()) ?? false))

              return (
                <div
                  key={idx}
                  className="gs-r-timeline-event"
                  data-selected={isSelected ? 'true' : 'false'}
                  onClick={() => handleEventClick(idx)}
                  onKeyDown={e => handleEventKeyDown(idx, e)}
                  tabIndex={0}
                  role="button"
                >
                  <div
                    className="gs-r-timeline-dot"
                    style={{
                      borderColor: isHit ? 'var(--tone-attention)' : undefined,
                    }}
                  />
                  <div className="gs-r-timeline-event-content">
                    <div
                      className="gs-r-timeline-label"
                      style={{
                        color: isHit ? 'var(--tone-attention)' : undefined,
                      }}
                    >
                      {event.label}
                    </div>
                    <div className="gs-r-timeline-date">{formatDate(event.at)}</div>
                    {event.summary && (
                      <div className="gs-r-timeline-summary">{event.summary}</div>
                    )}
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {selectedEvent ? (
        <div className="gs-r-timeline-detail">
          <div className="gs-r-timeline-detail-content">
            <div>
              <div className="gs-r-timeline-detail-label">Event</div>
              <div className="gs-r-timeline-detail-value">{selectedEvent.label}</div>
            </div>
            <div>
              <div className="gs-r-timeline-detail-label">Time</div>
              <div className="gs-r-timeline-detail-value">{formatDate(selectedEvent.at)}</div>
            </div>
            {selectedEvent.summary && (
              <div>
                <div className="gs-r-timeline-detail-label">Summary</div>
                <div className="gs-r-timeline-detail-value">{selectedEvent.summary}</div>
              </div>
            )}
            {selectedEvent.cursor && (
              <button
                className="gs-r-timeline-button"
                onClick={handleSetTime}
                data-testid="timeline-set-time"
              >
                Show the world at this point
              </button>
            )}
          </div>
        </div>
      ) : (
        <div className="gs-r-timeline-detail">
          <div className="gs-r-timeline-detail-empty">Select an event for details</div>
        </div>
      )}
    </div>
  )
}

export default TimelineRenderer
