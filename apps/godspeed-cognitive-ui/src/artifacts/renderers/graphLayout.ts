/**
 * Deterministic layered graph layout using longest-path algorithm.
 * Layers nodes by their longest path distance from source nodes.
 * Handles cycles by capping iterations to prevent infinite loops.
 */

export interface LayoutNode {
  id: string
}

export interface LayoutEdge {
  from: string
  to: string
}

export interface Position {
  x: number
  y: number
  layer: number
}

/**
 * Compute layered positions for nodes using longest-path algorithm.
 * Returns a map of node id -> {x, y, layer}.
 *
 * Algorithm:
 * 1. Find all source nodes (no incoming edges)
 * 2. Assign layers by computing longest path from any source
 * 3. Cap iterations at nodes.length to prevent cycle hangs
 * 4. Spread nodes vertically within each layer
 * 5. Position layers horizontally with uniform spacing
 */
export function layeredLayout(
  nodes: LayoutNode[],
  edges: LayoutEdge[]
): Record<string, Position> {
  if (nodes.length === 0) return {}

  const nodeSet = new Set(nodes.map(n => n.id))
  const inDegree = new Map<string, number>()
  const outgoing = new Map<string, string[]>()

  // Initialize degree maps
  for (const n of nodes) {
    inDegree.set(n.id, 0)
    outgoing.set(n.id, [])
  }

  // Count incoming edges and build adjacency list
  for (const e of edges) {
    if (nodeSet.has(e.from) && nodeSet.has(e.to)) {
      inDegree.set(e.to, (inDegree.get(e.to) ?? 0) + 1)
      outgoing.get(e.from)!.push(e.to)
    }
  }

  // Find source nodes (no incoming edges)
  const sources = nodes.filter(n => (inDegree.get(n.id) ?? 0) === 0)

  // Compute layers using longest-path algorithm with cycle detection
  const layer = new Map<string, number>()
  const queue: string[] = [...sources.map(n => n.id)]

  let iterations = 0
  const maxIterations = nodes.length + edges.length // Safety cap to prevent infinite loops on cycles

  while (queue.length > 0 && iterations < maxIterations) {
    iterations++
    const current = queue.shift()!
    const currentLayer = layer.get(current) ?? 0

    for (const next of outgoing.get(current) ?? []) {
      const nextLayer = currentLayer + 1
      const prevLayer = layer.get(next) ?? -1

      // Only update if we found a longer path
      if (nextLayer > prevLayer) {
        layer.set(next, nextLayer)
        if (!queue.includes(next)) {
          queue.push(next)
        }
      }
    }
  }

  // Assign layers to any nodes not visited (disconnected components, cycle members)
  for (const n of nodes) {
    if (!layer.has(n.id)) {
      layer.set(n.id, 0)
    }
  }

  // Group nodes by layer
  const layerGroups = new Map<number, string[]>()
  for (const [nodeId, nodeLayer] of layer) {
    if (!layerGroups.has(nodeLayer)) {
      layerGroups.set(nodeLayer, [])
    }
    layerGroups.get(nodeLayer)!.push(nodeId)
  }

  // Sort layers to ensure deterministic ordering
  const sortedLayers = Array.from(layerGroups.keys()).sort((a, b) => a - b)

  // Calculate dimensions
  const nodeHeight = 60
  const verticalSpacing = 20
  const horizontalSpacing = 200

  // Compute positions
  const positions: Record<string, Position> = {}
  let maxNodesInLayer = 0

  for (const layerNum of sortedLayers) {
    const layerNodes = layerGroups.get(layerNum)!
    maxNodesInLayer = Math.max(maxNodesInLayer, layerNodes.length)

    // Sort nodes in layer deterministically (by id)
    layerNodes.sort()

    const layerHeight = layerNodes.length * (nodeHeight + verticalSpacing)
    const startY = -(layerHeight / 2)

    for (let i = 0; i < layerNodes.length; i++) {
      const nodeId = layerNodes[i]
      const y = startY + i * (nodeHeight + verticalSpacing) + nodeHeight / 2
      const x = layerNum * horizontalSpacing

      positions[nodeId] = {
        x,
        y,
        layer: layerNum,
      }
    }
  }

  return positions
}
