import { useEffect, useMemo, useState } from 'react'
import type { SourceRendererProps } from '../registry'
import './TableRenderer.css'

export default function TableRenderer(props: SourceRendererProps) {
  if (props.model.kind !== 'table') {
    throw new Error(`TableRenderer expects kind 'table', got '${props.model.kind}'`)
  }

  const { columns, rows, caption } = props.model.table
  const [selectedRowIdx, setSelectedRowIdx] = useState<number | null>(null)
  const [sortCol, setSortCol] = useState<number | null>(null)
  const [sortAsc, setSortAsc] = useState(true)
  const [showOnlyMatches, setShowOnlyMatches] = useState(false)

  // Highlight function: try to import from highlight.ts, fallback to simple implementation
  const highlightText = (text: string, query: string) => {
    if (!query) return { text, hit: false }
    const lower = text.toLowerCase()
    const queryLower = query.toLowerCase()
    const idx = lower.indexOf(queryLower)
    return { hit: idx >= 0, text }
  }

  const getMatchCount = () => {
    let count = 0
    for (const row of rows) {
      for (const cell of row) {
        if (highlightText(cell, props.query).hit) count++
      }
    }
    return count
  }

  // Compute sorted rows
  const sortedRows = useMemo(() => {
    if (sortCol === null) return rows

    const sorted = [...rows].sort((a, b) => {
      const aVal = a[sortCol]
      const bVal = b[sortCol]
      const aNum = parseFloat(aVal)
      const bNum = parseFloat(bVal)
      const isNumeric = !isNaN(aNum) && !isNaN(bNum)

      let cmp = 0
      if (isNumeric) {
        cmp = aNum - bNum
      } else {
        cmp = aVal.localeCompare(bVal)
      }

      return sortAsc ? cmp : -cmp
    })
    return sorted
  }, [sortCol, sortAsc, rows])

  // Filter to matching rows
  const visibleRows = useMemo(() => {
    if (!showOnlyMatches || !props.query) return sortedRows

    return sortedRows.filter((row) =>
      row.some((cell) => highlightText(cell, props.query).hit)
    )
  }, [sortedRows, showOnlyMatches, props.query])

  // Report match count
  useEffect(() => {
    props.onMatches?.(getMatchCount())
  }, [props.query])

  const handleHeaderClick = (idx: number) => {
    if (sortCol === idx) {
      if (sortAsc) {
        setSortAsc(false)
      } else {
        setSortCol(null)
      }
    } else {
      setSortCol(idx)
      setSortAsc(true)
    }
  }

  const renderCell = (text: string) => {
    if (!props.query) return text

    const lower = text.toLowerCase()
    const queryLower = props.query.toLowerCase()
    const idx = lower.indexOf(queryLower)

    if (idx < 0) return text

    return (
      <>
        {text.slice(0, idx)}
        <mark>{text.slice(idx, idx + props.query.length)}</mark>
        {text.slice(idx + props.query.length)}
      </>
    )
  }

  return (
    <div data-testid="renderer-table" className="gs-r-table-root">
      <div className="gs-r-table-controls">
        <label className="gs-r-table-toggle">
          <input
            type="checkbox"
            checked={showOnlyMatches}
            onChange={(e) => setShowOnlyMatches(e.target.checked)}
            disabled={!props.query}
            aria-pressed={showOnlyMatches}
          />
          <span>Only matching rows</span>
        </label>
      </div>

      <div className="gs-r-table-container">
        <table className="gs-r-table">
          <thead>
            <tr>
              {columns.map((col, i) => (
                <th
                  key={i}
                  onClick={() => handleHeaderClick(i)}
                  className={sortCol === i ? 'gs-r-table-sorted' : ''}
                  aria-sort={
                    sortCol === i ? (sortAsc ? 'ascending' : 'descending') : 'none'
                  }
                >
                  {col}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {visibleRows.map((row, rowIdx) => {
              const origIdx = sortedRows.indexOf(row)
              const isSelected = selectedRowIdx === origIdx
              return (
                <tr
                  key={rowIdx}
                  onClick={() => setSelectedRowIdx(isSelected ? null : origIdx)}
                  className={isSelected ? 'gs-r-table-selected' : ''}
                  aria-selected={isSelected}
                  data-selected={isSelected ? 'true' : undefined}
                >
                  {row.map((cell, colIdx) => (
                    <td key={colIdx}>{renderCell(cell)}</td>
                  ))}
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      <div className="gs-r-table-footer">
        <span>
          {visibleRows.length} {visibleRows.length === 1 ? 'row' : 'rows'}
        </span>
        {selectedRowIdx !== null && (
          <span className="gs-r-table-selected-info">
            Row {selectedRowIdx + 1} selected
          </span>
        )}
      </div>

      {caption && (
        <div className="gs-r-table-caption">{caption}</div>
      )}
    </div>
  )
}
