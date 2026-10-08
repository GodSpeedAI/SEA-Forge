import { useEffect, useRef, useState } from 'react'
import type { SourceRendererProps } from '../registry'
import './ChartRenderer.css'

export default function ChartRenderer(props: SourceRendererProps) {
  if (props.model.kind !== 'chart') {
    throw new Error(`ChartRenderer expects kind 'chart', got '${props.model.kind}'`)
  }

  const { series, unit, caption } = props.model.chart
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null)
  const [hoveredIdx, setHoveredIdx] = useState<number | null>(null)
  const [tooltip, setTooltip] = useState<{ x: number; y: number; label: string; value: number } | null>(null)
  const svgRef = useRef<SVGSVGElement>(null)

  const maxValue = Math.max(...series.map((s) => s.value), 1)
  const chartWidth = 300
  const chartHeight = 200
  const padding = 40
  const gridLines = 4

  const getMatchCount = () => {
    return series.filter((s) =>
      s.label.toLowerCase().includes(props.query.toLowerCase())
    ).length
  }

  useEffect(() => {
    props.onMatches?.(getMatchCount())
  }, [props.query])

  const handleBarClick = (idx: number) => {
    setSelectedIdx(selectedIdx === idx ? null : idx)
  }

  const handleBarKeyDown = (idx: number, e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      handleBarClick(idx)
    }
  }

  const handleBarHover = (idx: number, x: number, y: number) => {
    setHoveredIdx(idx)
    const bar = series[idx]
    setTooltip({
      x,
      y,
      label: bar!.label,
      value: bar!.value,
    })
  }

  const renderLabel = (label: string) => {
    if (!props.query) return label

    const lower = label.toLowerCase()
    const queryLower = props.query.toLowerCase()
    const idx = lower.indexOf(queryLower)

    if (idx < 0) return label

    return (
      <>
        {label.slice(0, idx)}
        <tspan fontWeight="600" fill="var(--tone-progress)">
          {label.slice(idx, idx + props.query.length)}
        </tspan>
        {label.slice(idx + props.query.length)}
      </>
    )
  }

  const barWidth = Math.max(16, (chartWidth - padding * 2) / series.length * 0.7)
  const barSpacing = (chartWidth - padding * 2) / series.length

  return (
    <div data-testid="renderer-chart" className="gs-r-chart-root">
      <div className="gs-r-chart-container">
        <svg
          ref={svgRef}
          className="gs-r-chart-svg"
          viewBox={`0 0 ${chartWidth} ${chartHeight + padding}`}
          preserveAspectRatio="xMidYMid meet"
          role="img"
          aria-label={`Bar chart showing ${series.map((s) => `${s.label}: ${s.value}`).join(', ')}`}
        >
          {/* Gridlines */}
          {Array.from({ length: gridLines }).map((_, i) => {
            const ratio = (i + 1) / gridLines
            const y = padding + (chartHeight - chartHeight * ratio)
            const value = Math.round(maxValue * ratio)
            return (
              <g key={`grid-${i}`} className="gs-r-chart-gridline">
                <line
                  x1={padding}
                  y1={y}
                  x2={chartWidth - padding / 2}
                  y2={y}
                  stroke="var(--dock-line)"
                  strokeWidth="1"
                />
                <text
                  x={padding - 8}
                  y={y + 4}
                  textAnchor="end"
                  fontSize="11"
                  fill="var(--dock-ink-2)"
                >
                  {value}
                </text>
              </g>
            )
          })}

          {/* Y-axis label (unit) */}
          <text
            x={12}
            y={20}
            fontSize="11"
            fill="var(--dock-ink-2)"
          >
            {unit}
          </text>

          {/* Bars */}
          {series.map((bar, idx) => {
            const x = padding + idx * barSpacing + (barSpacing - barWidth) / 2
            const barHeight = (bar.value / maxValue) * chartHeight
            const y = padding + chartHeight - barHeight
            const isSelected = selectedIdx === idx
            const isHovered = hoveredIdx === idx
            const isMatched = bar.label.toLowerCase().includes(props.query.toLowerCase())

            return (
              <g
                key={idx}
                role="button"
                tabIndex={0}
                onMouseEnter={() => handleBarHover(idx, x + barWidth / 2, y)}
                onMouseLeave={() => {
                  setHoveredIdx(null)
                  setTooltip(null)
                }}
                onClick={() => handleBarClick(idx)}
                onKeyDown={(e) => handleBarKeyDown(idx, e as unknown as React.KeyboardEvent)}
                className={`gs-r-chart-bar ${isSelected ? 'gs-r-chart-bar--selected' : ''}`}
                aria-label={`${bar.label}: ${bar.value} ${unit}`}
                data-selected={isSelected ? 'true' : undefined}
              >
                {/* Bar rect */}
                <rect
                  x={x}
                  y={y}
                  width={barWidth}
                  height={barHeight}
                  fill={isSelected ? 'var(--tone-attention)' : isMatched ? 'var(--tone-progress)' : 'var(--tone-progress)'}
                  opacity={isSelected ? 1 : isHovered ? 0.9 : 0.8}
                  className="gs-r-chart-bar-rect"
                />

                {/* Value label on bar */}
                {barHeight > 20 && (
                  <text
                    x={x + barWidth / 2}
                    y={y + barHeight / 2 + 4}
                    textAnchor="middle"
                    fontSize="11"
                    fill="white"
                    fontWeight="600"
                    pointerEvents="none"
                  >
                    {bar.value}
                  </text>
                )}

                {/* Label below bar */}
                <text
                  x={x + barWidth / 2}
                  y={padding + chartHeight + 16}
                  textAnchor="middle"
                  fontSize="11"
                  fill={isMatched ? 'var(--tone-progress)' : 'var(--dock-ink)'}
                  fontWeight={isMatched ? '600' : '400'}
                  pointerEvents="none"
                >
                  {renderLabel(bar.label)}
                </text>
              </g>
            )
          })}
        </svg>

        {/* Tooltip */}
        {tooltip && (
          <div
            className="gs-r-chart-tooltip"
            style={{
              left: `${(tooltip.x / chartWidth) * 100}%`,
              top: `${(tooltip.y / (chartHeight + padding)) * 100}%`,
            }}
          >
            <div className="gs-r-chart-tooltip-label">{tooltip.label}</div>
            <div className="gs-r-chart-tooltip-value">
              {tooltip.value} {unit}
            </div>
          </div>
        )}
      </div>

      {selectedIdx !== null && (
        <div className="gs-r-chart-detail">
          {series[selectedIdx]!.label} — {series[selectedIdx]!.value} {unit}
        </div>
      )}

      {caption && (
        <div className="gs-r-chart-caption">{caption}</div>
      )}
    </div>
  )
}
