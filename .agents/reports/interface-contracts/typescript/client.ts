/**
 * Universal Casework Client for the GodSpeed Cognitive Environment.
 * Wraps either the MockCaseworkAdapter or HttpCaseworkAdapter behind a uniform API.
 */

import { CaseworkAdapter } from './mock-adapter';
import {
  ActorRole,
  ArtifactPayload,
  CognitiveWorldSnapshot,
  IntentResponse,
  InteractionIntent,
  StreamEvent,
  TemporalTrajectoryResponse,
} from './types';

export interface CaseworkClient {
  getLiveWorld(caseId: string, actorId: string, role: ActorRole): Promise<CognitiveWorldSnapshot>;
  getHistoricalWorld(
    caseId: string,
    cursor: string,
    actorId: string,
    role: ActorRole
  ): Promise<CognitiveWorldSnapshot>;
  dispatchIntent(intent: InteractionIntent): Promise<IntentResponse>;
  resolveArtifact(evidenceId: string): Promise<ArtifactPayload>;
  queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse>;
  subscribeWorld(
    caseId: string,
    actorId: string,
    role: ActorRole,
    onSnapshot: (snapshot: CognitiveWorldSnapshot) => void,
    onError?: (err: Error) => void
  ): () => void;
}

export class DefaultCaseworkClient implements CaseworkClient {
  constructor(private adapter: CaseworkAdapter) {}

  public async getLiveWorld(
    caseId: string,
    actorId: string,
    role: ActorRole
  ): Promise<CognitiveWorldSnapshot> {
    return this.adapter.getSnapshot(caseId, actorId, role);
  }

  public async getHistoricalWorld(
    caseId: string,
    cursor: string,
    actorId: string,
    role: ActorRole
  ): Promise<CognitiveWorldSnapshot> {
    return this.adapter.getSnapshotAt(caseId, cursor, actorId, role);
  }

  public async dispatchIntent(intent: InteractionIntent): Promise<IntentResponse> {
    return this.adapter.dispatchIntent(intent);
  }

  public async resolveArtifact(evidenceId: string): Promise<ArtifactPayload> {
    return this.adapter.resolveArtifact(evidenceId);
  }

  public async queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse> {
    return this.adapter.queryTemporalTrajectory(caseId);
  }

  public subscribeWorld(
    caseId: string,
    actorId: string,
    role: ActorRole,
    onSnapshot: (snapshot: CognitiveWorldSnapshot) => void,
    onError?: (err: Error) => void
  ): () => void {
    let currentCursor: string | undefined;

    return this.adapter.subscribeEvents(
      caseId,
      currentCursor,
      (event: StreamEvent) => {
        if (event.event_type === 'snapshot' || event.event_type === 'patch') {
          currentCursor = event.cursor;
          onSnapshot(event.payload as CognitiveWorldSnapshot);
        }
      },
      (err: Error) => {
        if (onError) onError(err);
      }
    );
  }
}

/**
 * Factory to create a client backed by any adapter.
 */
export function createCaseworkClient(adapter: CaseworkAdapter): CaseworkClient {
  return new DefaultCaseworkClient(adapter);
}
