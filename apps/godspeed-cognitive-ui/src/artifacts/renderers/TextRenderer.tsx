import { useEffect, useState } from 'react'
import React from 'react'
import type { SourceRendererProps } from '../registry'
import { highlight } from './highlight'
import './TextRenderer.css'

export default function TextRenderer(props: SourceRendererProps) {
  if (props.model.kind !== 'text') throw new Error('TextRenderer expects kind="text"')

  const lines = props.model.lines as string[]
  const [selectedLines, setSelectedLines] = useState<Set<number>>(new Set())
  const [lastSelectedLine, setLastSelectedLine] = useState<number | null>(null)
  const [wrap, setWrap] = useState(false)

  // Search highlight
  useEffect(() => {
    let totalMatches = 0
    lines.forEach((line) => {
      totalMatches += highlight(line, props.query).count
    })
    props.onMatches?.(totalMatches)
  }, [props.query, lines, props])

  const handleLineClick = (idx: number, e: React.MouseEvent) => {
    if (e.shiftKey && lastSelectedLine !== null) {
      // Range selection
      const newSelected = new Set(selectedLines)
      const minIdx = Math.min(lastSelectedLine, idx)
      const maxIdx = Math.max(lastSelectedLine, idx)
      for (let i = minIdx; i <= maxIdx; i++) {
        newSelected.add(i)
      }
      setSelectedLines(newSelected)
    } else {
      // Single toggle
      const newSelected = new Set(selectedLines)
      if (newSelected.has(idx)) {
        newSelected.delete(idx)
      } else {
        newSelected.add(idx)
      }
      setSelectedLines(newSelected)
    }
    setLastSelectedLine(idx)
  }

  const selectedArray = Array.from(selectedLines).sort((a, b) => a - b)
  const selectionLabel =
    selectedArray.length === 0
      ? ''
      : selectedArray.length === 1
        ? `Line ${selectedArray[0]! + 1} selected`
        : `Lines ${selectedArray[0]! + 1}–${selectedArray[selectedArray.length - 1]! + 1} selected`

  const maxLineNum = lines.length
  const lineNumWidth = Math.max(4, String(maxLineNum).length + 1)

  return (
    <div data-testid="renderer-text" className="gs-r-text-root">
      {/* Toolbar */}
      <div className="gs-r-text-toolbar">
        <button
          className="gs-r-text-wrap-btn"
          aria-pressed={wrap}
          onClick={() => setWrap(!wrap)}
        >
          Wrap
        </button>
      </div>

      {/* Content */}
      <div className="gs-r-text-container" onWheel={(e) => e.stopPropagation()}>
        <div className="gs-r-text-lines" role="listbox">
          {lines.map((line, idx) => {
            const isSelected = selectedLines.has(idx)
            const { parts } = highlight(line, props.query)
            const lineNum = idx + 1

            return (
              <div
                key={idx}
                className={`gs-r-text-line ${wrap ? 'gs-r-text-line--wrap' : ''}`}
                data-selected={isSelected ? 'true' : undefined}
                aria-selected={isSelected}
                role="option"
                onClick={(e) => handleLineClick(idx, e)}
              >
                <div
                  className="gs-r-text-number"
                  style={{ minWidth: `${lineNumWidth}ch` }}
                >
                  {lineNum}
                </div>
                <code className="gs-r-text-code">
                  {parts.map((part, i) => (
                    <React.Fragment key={i}>
                      {part.hit ? <mark>{part.text}</mark> : part.text}
                    </React.Fragment>
                  ))}
                </code>
              </div>
            )
          })}
        </div>
      </div>

      {/* Footer */}
      <div className="gs-r-text-footer">{selectionLabel}</div>
    </div>
  )
}
