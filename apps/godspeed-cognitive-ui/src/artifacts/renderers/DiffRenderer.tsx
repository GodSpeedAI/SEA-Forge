import { useEffect, useRef, useState } from 'react'
import type { SourceRendererProps } from '../registry'
import type { DiffFile } from '../model'
import { highlight } from './highlight'
import './DiffRenderer.css'

const SIGN_CLASS = { '+': 'add', '-': 'del', ' ': 'ctx', '@': 'hunk' } as const

export default function DiffRenderer(props: SourceRendererProps) {
  if (props.model.kind !== 'diff') throw new Error('DiffRenderer expects kind="diff"')

  const files = props.model.files as DiffFile[]
  const [activeFileIdx, setActiveFileIdx] = useState(0)
  const [selectedLines, setSelectedLines] = useState<Set<string>>(new Set())
  const [lastSelectedLine, setLastSelectedLine] = useState<string | null>(null)
  const fileRefs = useRef<(HTMLDivElement | null)[]>([])

  // Search highlight
  useEffect(() => {
    let totalMatches = 0
    files.forEach((file) => {
      file.lines.forEach((line) => {
        totalMatches += highlight(line.text, props.query).count
      })
    })
    props.onMatches?.(totalMatches)
  }, [props.query, files, props])

  const handleFileClick = (idx: number) => {
    setActiveFileIdx(idx)
    fileRefs.current[idx]?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }

  const handleLineClick = (fileIdx: number, lineIdx: number, e: React.MouseEvent) => {
    const lineKey = `${fileIdx}:${lineIdx}`

    if (e.shiftKey && lastSelectedLine) {
      // Range selection
      const [lastFileIdx, lastLineIdx] = lastSelectedLine.split(':').map(Number)
      const newSelected = new Set(selectedLines)

      const startFile = Math.min(lastFileIdx, fileIdx)
      const endFile = Math.max(lastFileIdx, fileIdx)
      const startLine = startFile === lastFileIdx ? lastLineIdx : lineIdx
      const endLine = startFile === lastFileIdx ? lineIdx : lastLineIdx

      if (startFile === endFile) {
        const minLine = Math.min(startLine, endLine)
        const maxLine = Math.max(startLine, endLine)
        for (let i = minLine; i <= maxLine; i++) {
          newSelected.add(`${startFile}:${i}`)
        }
      } else {
        for (let f = startFile; f <= endFile; f++) {
          const minL = f === startFile ? startLine : 0
          const maxL = f === endFile ? endLine : files[f]!.lines.length - 1
          for (let i = minL; i <= maxL; i++) {
            newSelected.add(`${f}:${i}`)
          }
        }
      }

      setSelectedLines(newSelected)
    } else {
      // Single toggle
      const newSelected = new Set(selectedLines)
      if (newSelected.has(lineKey)) {
        newSelected.delete(lineKey)
      } else {
        newSelected.add(lineKey)
      }
      setSelectedLines(newSelected)
    }

    setLastSelectedLine(lineKey)
  }

  const selectedArray = Array.from(selectedLines).sort((a, b) => {
    const [aFileIdx, aLineIdx] = a.split(':').map(Number)
    const [bFileIdx, bLineIdx] = b.split(':').map(Number)
    return aFileIdx === bFileIdx ? aLineIdx - bLineIdx : aFileIdx - bFileIdx
  })

  const selectionLabel =
    selectedArray.length === 0
      ? ''
      : selectedArray.length === 1
        ? `Line ${selectedArray[0]!.split(':')[1]! + 1} selected`
        : `Lines ${selectedArray[0]!.split(':')[1]! + 1}–${selectedArray[selectedArray.length - 1]!.split(':')[1]! + 1} selected`

  return (
    <div data-testid="renderer-diff" className="gs-r-diff-root">
      {/* File list */}
      <div className="gs-r-diff-file-list">
        {files.map((file, idx) => (
          <button
            key={idx}
            className="gs-r-diff-file-btn"
            aria-current={activeFileIdx === idx ? 'page' : undefined}
            onClick={() => handleFileClick(idx)}
          >
            <span className="gs-r-diff-file-path">{file.path}</span>
            <span className="gs-r-diff-file-stat gs-r-diff-file-added">+{file.added}</span>
            <span className="gs-r-diff-file-stat gs-r-diff-file-removed">−{file.removed}</span>
          </button>
        ))}
      </div>

      {/* Diff view */}
      <div className="gs-r-diff-container" onWheel={(e) => e.stopPropagation()}>
        {files.map((file, fileIdx) => (
          <div
            key={fileIdx}
            ref={(el) => {
              fileRefs.current[fileIdx] = el
            }}
            className="gs-r-diff-file"
          >
            <div className="gs-r-diff-file-header">{file.path}</div>
            <div className="gs-r-diff-lines" role="listbox">
              {file.lines.map((line, lineIdx) => {
                const lineKey = `${fileIdx}:${lineIdx}`
                const isSelected = selectedLines.has(lineKey)
                const { parts } = highlight(line.text, props.query)

                return (
                  <div
                    key={lineIdx}
                    className={`gs-r-diff-line gs-r-diff-line--${SIGN_CLASS[line.sign]}`}
                    data-selected={isSelected ? 'true' : undefined}
                    aria-selected={isSelected}
                    role="option"
                    onClick={(e) => handleLineClick(fileIdx, lineIdx, e)}
                  >
                    <div className="gs-r-diff-gutter gs-r-diff-gutter-old">
                      {line.oldN !== undefined ? line.oldN : ''}
                    </div>
                    <div className="gs-r-diff-gutter gs-r-diff-gutter-new">
                      {line.newN !== undefined ? line.newN : ''}
                    </div>
                    <div className="gs-r-diff-sign">{line.sign}</div>
                    <code className="gs-r-diff-code">
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
        ))}
      </div>

      {/* Footer */}
      <div className="gs-r-diff-footer">{selectionLabel}</div>
    </div>
  )
}

// React Fragment for JSX
import React from 'react'
