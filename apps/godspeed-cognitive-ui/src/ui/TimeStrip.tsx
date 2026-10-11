import { useRef, useState } from 'react'
import './time-strip.css'

export interface TimeStripProps {
  revisions: { id: string; at: string; label: string; summary?: string }[]
  current: string
  visible: boolean
  marks: string[]
  comparing: { a: string; b: string } | null
  onSeek(id: string): void
  onReturnToNow(): void
  onMark(id: string): void
  onCompare(a: string, b: string): void
  onClose(): void
}

export function TimeStrip(p: TimeStripProps): JSX.Element {
  const trackRef = useRef<HTMLDivElement>(null)
  const [isDragging, setIsDragging] = useState(false)

  const currentIdx = p.revisions.findIndex((r) => r.id === p.current)
  const isLive = currentIdx === p.revisions.length - 1 && p.comparing === null

  const handlePrevClick = () => {
    if (currentIdx > 0) {
      p.onSeek(p.revisions[currentIdx - 1].id)
    }
  }

  const handleNextClick = () => {
    if (currentIdx < p.revisions.length - 1) {
      p.onSeek(p.revisions[currentIdx + 1].id)
    }
  }

  const handleMarkClick = () => {
    p.onMark(p.current)
  }

  const handleCompareClick = () => {
    if (p.marks.length === 2) {
      const [a, b] = p.marks
      const aIdx = p.revisions.findIndex((r) => r.id === a)
      const bIdx = p.revisions.findIndex((r) => r.id === b)
      if (aIdx < bIdx) {
        p.onCompare(a, b)
      } else {
        p.onCompare(b, a)
      }
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === 'ArrowLeft' && currentIdx > 0) {
      p.onSeek(p.revisions[currentIdx - 1].id)
      e.preventDefault()
    } else if (e.key === 'ArrowRight' && currentIdx < p.revisions.length - 1) {
      p.onSeek(p.revisions[currentIdx + 1].id)
      e.preventDefault()
    } else if (e.key === 'End') {
      p.onReturnToNow()
      e.preventDefault()
    }
  }

  const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!trackRef.current || e.button !== 0) return
    setIsDragging(true)
    trackRef.current.setPointerCapture(e.pointerId)
    scrubToPosition(e)
  }

  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!isDragging) return
    scrubToPosition(e)
  }

  const handlePointerUp = () => {
    setIsDragging(false)
  }

  const scrubToPosition = (e: React.PointerEvent<HTMLDivElement>) => {
    if (!trackRef.current) return
    const rect = trackRef.current.getBoundingClientRect()
    const x = e.clientX - rect.left
    const ratio = Math.max(0, Math.min(1, x / rect.width))

    let closest = 0
    let minDist = Math.abs(0 / p.revisions.length - ratio)

    for (let i = 1; i < p.revisions.length; i++) {
      const dist = Math.abs(i / (p.revisions.length - 1) - ratio)
      if (dist < minDist) {
        minDist = dist
        closest = i
      }
    }

    p.onSeek(p.revisions[closest].id)
  }

  const currentRev = p.revisions[currentIdx]

  const stopPositions = p.revisions.map((_, i) => (100 * i) / (p.revisions.length - 1))

  const markOrder: Record<string, string> = {}
  p.marks.forEach((id, idx) => {
    markOrder[id] = String.fromCharCode(65 + idx)
  })

  const isCurrentMarked = markOrder[p.current] !== undefined

  return (
    <div
      className={`time-strip ${p.visible ? '' : 'hidden'}`}
      data-testid="time-strip"
      role="group"
      aria-label="Time"
      tabIndex={0}
      onKeyDown={handleKeyDown}
    >
      <div className="time-strip__controls">
        <button
          className="time-strip__button time-strip__button--nav"
          onClick={handlePrevClick}
          disabled={currentIdx === 0}
          data-testid="time-prev"
          aria-label="Previous revision"
        >
          ◀
        </button>

        <button
          className="time-strip__button time-strip__button--nav"
          onClick={handleNextClick}
          disabled={currentIdx === p.revisions.length - 1}
          data-testid="time-next"
          aria-label="Next revision"
        >
          ▶
        </button>

        <div className="time-strip__current" data-testid="time-current">
          <div className="time-strip__current-label">{currentRev.label}</div>
          {currentRev.summary && <div className="time-strip__current-summary">{currentRev.summary}</div>}
        </div>

        <button
          className={`time-strip__button time-strip__button--mark ${isCurrentMarked ? 'marked' : ''}`}
          onClick={handleMarkClick}
          data-testid="time-mark"
          aria-pressed={isCurrentMarked}
          aria-label="Mark for compare"
        >
          Mark for compare
        </button>

        <button
          className="time-strip__button"
          onClick={handleCompareClick}
          disabled={p.marks.length !== 2}
          data-testid="time-compare"
          aria-label="Compare marked revisions"
        >
          Compare A ↔ B
        </button>

        {(!isLive || p.comparing !== null) && (
          <button
            className="time-strip__button"
            onClick={p.onReturnToNow}
            data-testid="time-live"
            aria-label="Return to live"
          >
            Return to live
          </button>
        )}

        <button
          className="time-strip__button time-strip__button--close"
          onClick={p.onClose}
          data-testid="time-close"
          aria-label="Close time strip"
        >
          ×
        </button>
      </div>

      <div
        ref={trackRef}
        className="time-strip__track"
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerLeave={handlePointerUp}
        role="slider"
        aria-label="Revision timeline"
      >
        {p.revisions.map((rev, idx) => {
          const isCurrent = rev.id === p.current
          const markBadge = markOrder[rev.id]

          return (
            <div key={rev.id} className="time-strip__marker-wrapper" style={{ left: `${stopPositions[idx]}%` }}>
              <button
                className={`time-strip__marker ${isCurrent ? 'current' : ''} ${markBadge ? 'marked' : ''}`}
                onClick={() => p.onSeek(rev.id)}
                data-testid="time-marker"
                data-rev={rev.id}
                aria-label={`${rev.label}${rev.summary ? ' — ' + rev.summary : ''}`}
                aria-current={isCurrent ? 'true' : undefined}
              >
                {markBadge && <span className="time-strip__marker-badge">{markBadge}</span>}
              </button>
            </div>
          )
        })}
      </div>

      <div className="time-strip__hint">{isLive ? 'Live' : 'Viewing the past · read-only'}</div>
    </div>
  )
}
