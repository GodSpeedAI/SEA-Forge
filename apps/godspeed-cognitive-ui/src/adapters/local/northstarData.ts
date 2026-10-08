import type { ActorRole, TemporalCheckpoint, XSnapshot } from '../../ports/contract'
import snapshots from './data/northstar.snapshots.json'
import trajectory from './data/northstar.trajectory.json'

// The Northstar case as contract data at rest: six CognitiveWorldSnapshots (oldest first) and
// their temporal trajectory. This is what the Go system front end would serve; the local adapter
// only stamps the requesting perspective onto it.

export const NORTHSTAR_CASE_ID = 'case-northstar'

export function buildNorthstarHistory(perspective: { actor_id: string; role: ActorRole; display_name?: string }): XSnapshot[] {
  return (snapshots as unknown as XSnapshot[]).map((s) => ({ ...structuredClone(s), perspective }))
}

export const NORTHSTAR_TRAJECTORY = trajectory as TemporalCheckpoint[]
