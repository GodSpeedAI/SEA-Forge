import { useState, useEffect, useRef } from 'react'
import type { SourceRendererProps } from '../registry'
import type { GraphModel } from '../model'
import { layeredLayout } from './graphLayout'
import './GraphRenderer.css'

// Try to import highlight, but it's optional - we'll use basic text matching if not available
let highlight: ((text: string, query: string) => { parts: Array<{ text: string; hit: boolean }>; count: number }) | null = null
try {
  highlight = (require('./highlight').highlight || null) as typeof highlight
} catch {
  // highlight.ts not yet available, will use fallback
}

interface NodeState {
  id: string
  x: number
  y: number
  layer: number
}

interface ViewState {
  tx: number
  ty: number
  scale: number
}

export function GraphRenderer(props: SourceRendererProps) {
  const model = props.model as { kind: 'graph'; graph: GraphModel }
  const graph = model.graph

  const [nodes, setNodes] = useState<NodeState[]>([])
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [view, setView] = useState<ViewState>({ tx: 0, ty: 0, scale: 1 })
  const [draggingNode, setDraggingNode] = useState<string | null>(null)
  const [panStartPos, setPanStartPos] = useState<{ x: number; y: number } | null>(null)
  const [query, setQuery] = useState(props.query || '')
  const svgRef = useRef<SVGSVGElement>(null)

  // Initialize layout
  useEffect(() => {
    const positions = layeredLayout(
      graph.nodes,
      graph.edges
    )

    const layoutNodes = graph.nodes.map(n => ({
      id: n.id,
      x: positions[n.id]?.x || 0,
      y: positions[n.id]?.y || 0,
      layer: positions[n.id]?.layer || 0,
    }))

    setNodes(layoutNodes)
  }, [graph])

  // Report matches
  useEffect(() => {
    if (!query) {
      props.onMatches?.(0)
      return
    }

    if (highlight) {
      let count = 0
      for (const node of graph.nodes) {
        const result = highlight(node.label, query)
        count += result.count
      }
      props.onMatches?.(count)
    } else {
      // Fallback: simple substring match
      let count = 0
      for (const node of graph.nodes) {
        if (node.label.toLowerCase().includes(query.toLowerCase())) {
          count++
        }
      }
      props.onMatches?.(count)
    }
  }, [query, graph.nodes, props])

  const getSelectedNode = () => graph.nodes.find(n => n.id === selectedNodeId)
  const selectedNode = getSelectedNode()

  const incidentEdgeIds = new Set<string>()
  if (selectedNodeId) {
    for (let i = 0; i < graph.edges.length; i++) {
      const e = graph.edges[i]
      if (e.from === selectedNodeId || e.to === selectedNodeId) {
        incidentEdgeIds.add(i.toString())
      }
    }
  }

  const handleSvgWheel = (e: React.WheelEvent<SVGSVGElement>) => {
    e.stopPropagation()
    e.preventDefault()

    const delta = e.deltaY > 0 ? 0.9 : 1.1
    setView(v => ({ ...v, scale: Math.max(0.1, Math.min(5, v.scale * delta)) }))
  }

  const handleSvgMouseDown = (e: React.MouseEvent<SVGSVGElement>) => {
    if (e.target === svgRef.current || (e.target as SVGElement).tagName === 'svg') {
      setPanStartPos({ x: e.clientX, y: e.clientY })
    }
  }

  const handleSvgMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    if (panStartPos) {
      const dx = e.clientX - panStartPos.x
      const dy = e.clientY - panStartPos.y
      setView(v => ({ ...v, tx: v.tx + dx, ty: v.ty + dy }))
      setPanStartPos({ x: e.clientX, y: e.clientY })
    }
  }

  const handleSvgMouseUp = () => {
    setPanStartPos(null)
  }

  const handleNodeMouseDown = (nodeId: string, e: React.MouseEvent) => {
    e.stopPropagation()
    setDraggingNode(nodeId)
    if (svgRef.current) {
      svgRef.current.setPointerCapture((e as any).pointerId || 0)
    }
  }

  const handleNodeMouseMove = (e: React.PointerEvent<SVGElement>) => {
    if (!draggingNode || !svgRef.current) return

    const svg = svgRef.current
    const rect = svg.getBoundingClientRect()
    const x = (e.clientX - rect.left - view.tx) / view.scale
    const y = (e.clientY - rect.top - view.ty) / view.scale

    setNodes(ns =>
      ns.map(n => (n.id === draggingNode ? { ...n, x, y } : n))
    )
  }

  const handleNodeMouseUp = () => {
    setDraggingNode(null)
  }

  const handleNodeClick = (nodeId: string) => {
    setSelectedNodeId(selectedNodeId === nodeId ? null : nodeId)
  }

  const handleNodeKeyDown = (nodeId: string, e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleNodeClick(nodeId)
    }
  }

  const handleResetView = () => {
    setView({ tx: 0, ty: 0, scale: 1 })
  }

  const handleFocusObject = () => {
    if (selectedNode?.object_id) {
      props.onFocusObject?.(selectedNode.object_id)
    }
  }

  const nodeWidth = 140
  const nodeHeight = 60
  const cornerRadius = 6

  return (
    <div className="gs-r-graph-container">
      <div className="gs-r-graph-toolbar">
        <input
          type="text"
          className="gs-r-graph-search"
          placeholder="Search nodes…"
          value={query}
          onChange={e => setQuery(e.target.value)}
          onKeyDown={e => e.stopPropagation()}
        />
        <div className="gs-r-graph-match-count">
          {query ? 'Matching' : ''}
        </div>
      </div>

      <svg
        ref={svgRef}
        className="gs-r-graph-svg"
        data-testid="renderer-graph"
        onWheel={handleSvgWheel}
        onMouseDown={handleSvgMouseDown}
        onMouseMove={handleSvgMouseMove}
        onMouseUp={handleSvgMouseUp}
        onMouseLeave={handleSvgMouseUp}
      >
        <g style={{ transform: `translate(${view.tx}px, ${view.ty}px) scale(${view.scale})` }}>
          {/* Edges */}
          {graph.edges.map((edge, idx) => {
            const fromNode = nodes.find(n => n.id === edge.from)
            const toNode = nodes.find(n => n.id === edge.to)
            if (!fromNode || !toNode) return null

            const isIncident = incidentEdgeIds.has(idx.toString())
            const isDimmed = selectedNodeId && !isIncident

            const x1 = fromNode.x + nodeWidth / 2
            const y1 = fromNode.y + nodeHeight / 2
            const x2 = toNode.x - nodeWidth / 2
            const y2 = toNode.y
            const midX = (x1 + x2) / 2
            const midY = (y1 + y2) / 2

            // Arrow direction
            const angle = Math.atan2(y2 - y1, x2 - x1)
            const arrowSize = 8

            return (
              <g key={idx}>
                <line
                  className="gs-r-graph-edge"
                  x1={x1}
                  y1={y1}
                  x2={x2}
                  y2={y2}
                  data-incident={isIncident ? 'true' : 'false'}
                  data-dimmed={isDimmed ? 'true' : 'false'}
                />
                {edge.label && (
                  <text className="gs-r-graph-edge-label" x={midX} y={midY}>
                    {edge.label}
                  </text>
                )}
                {/* Arrowhead */}
                <polygon
                  className="gs-r-graph-arrowhead"
                  points={`0,0 -${arrowSize},${arrowSize / 2} -${arrowSize},-${arrowSize / 2}`}
                  style={{
                    transform: `translate(${x2}px, ${y2}px) rotate(${(angle * 180) / Math.PI}deg)`,
                    transformOrigin: '0 0',
                  }}
                  data-incident={isIncident ? 'true' : 'false'}
                />
              </g>
            )
          })}

          {/* Nodes */}
          {nodes.map(node => {
            const gNode = graph.nodes.find(n => n.id === node.id)
            if (!gNode) return null

            const isSelected = selectedNodeId === node.id
            const isHit =
              query &&
              (highlight
                ? highlight(gNode.label, query).count > 0
                : gNode.label.toLowerCase().includes(query.toLowerCase()))

            return (
              <g key={node.id}>
                <rect
                  className="gs-r-graph-node"
                  x={node.x - nodeWidth / 2}
                  y={node.y - nodeHeight / 2}
                  width={nodeWidth}
                  height={nodeHeight}
                  rx={cornerRadius}
                  fill={isSelected ? 'rgba(59, 139, 240, 0.1)' : 'rgba(200, 200, 200, 0.1)'}
                  stroke={isSelected ? 'var(--tone-progress)' : isHit ? 'var(--tone-attention)' : 'var(--dock-line)'}
                  strokeWidth={isSelected || isHit ? 2 : 1}
                  data-node-id={node.id}
                  data-selected={isSelected ? 'true' : 'false'}
                  data-search-hit={isHit ? 'true' : 'false'}
                  onMouseDown={e => handleNodeMouseDown(node.id, e)}
                  onPointerMove={handleNodeMouseMove}
                  onPointerUp={handleNodeMouseUp}
                  onClick={() => handleNodeClick(node.id)}
                  onKeyDown={e => handleNodeKeyDown(node.id, e)}
                  tabIndex={0}
                  role="button"
                  style={{ cursor: draggingNode === node.id ? 'grabbing' : 'grab' }}
                />
                <text
                  className="gs-r-graph-node-label"
                  x={node.x}
                  y={node.y}
                  onMouseDown={e => handleNodeMouseDown(node.id, e)}
                  onPointerMove={handleNodeMouseMove}
                  onPointerUp={handleNodeMouseUp}
                >
                  {gNode.label}
                </text>
              </g>
            )
          })}
        </g>
      </svg>

      {selectedNode ? (
        <div className="gs-r-graph-detail">
          <div className="gs-r-graph-detail-content">
            <div className="gs-r-graph-detail-field">
              <div className="gs-r-graph-detail-label">Label:</div>
              <div className="gs-r-graph-detail-value">{selectedNode.label}</div>
            </div>
            {selectedNode.kind && (
              <div className="gs-r-graph-detail-field">
                <div className="gs-r-graph-detail-label">Kind:</div>
                <div className="gs-r-graph-detail-value">{selectedNode.kind}</div>
              </div>
            )}
            <div className="gs-r-graph-detail-field">
              <div className="gs-r-graph-detail-label">Degree:</div>
              <div className="gs-r-graph-detail-value">
                In: {graph.edges.filter(e => e.to === selectedNodeId).length}, Out:{' '}
                {graph.edges.filter(e => e.from === selectedNodeId).length}
              </div>
            </div>
            <div className="gs-r-graph-detail-actions">
              {selectedNode.object_id && (
                <button
                  className="gs-r-graph-button"
                  onClick={handleFocusObject}
                  data-testid="graph-focus-object"
                >
                  Show in world
                </button>
              )}
              <button
                className="gs-r-graph-button gs-r-graph-reset-button"
                onClick={handleResetView}
              >
                Reset view
              </button>
            </div>
          </div>
        </div>
      ) : (
        <div className="gs-r-graph-detail">
          <div className="gs-r-graph-detail-empty">Click a node for details</div>
        </div>
      )}
    </div>
  )
}

export default GraphRenderer
