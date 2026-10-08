/**
 * Rich in-memory mock adapter for the GodSpeed Casework Operating System.
 * Simulates SEA-Forge case engine, role projections, Gauntlet execution progress,
 * RealityTrace evidence, and temporal history traversal.
 */

import {
  ActorRole,
  ArtifactPayload,
  CognitiveObject,
  CognitiveWorldSnapshot,
  ExecutionProgressPayload,
  IntentResponse,
  InteractionIntent,
  StreamEvent,
  TemporalCheckpoint,
  TemporalTrajectoryResponse,
} from './types';

export interface CaseworkAdapter {
  getSnapshot(caseId: string, actorId: string, role: ActorRole): Promise<CognitiveWorldSnapshot>;
  getSnapshotAt(caseId: string, cursor: string, actorId: string, role: ActorRole): Promise<CognitiveWorldSnapshot>;
  dispatchIntent(intent: InteractionIntent): Promise<IntentResponse>;
  resolveArtifact(evidenceId: string): Promise<ArtifactPayload>;
  queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse>;
  subscribeEvents(
    caseId: string,
    sinceCursor: string | undefined,
    onEvent: (event: StreamEvent) => void,
    onError: (err: Error) => void
  ): () => void;
}

export class MockCaseworkAdapter implements CaseworkAdapter {
  private seq = 42;
  private epoch = 1;
  private activeCaseId = 'case-auth-v2-001';
  private listeners: Set<(event: StreamEvent) => void> = new Set();
  private history: Map<string, CognitiveWorldSnapshot> = new Map();

  // Internal state of the mock case
  private stageState: 'ACTIVE' | 'COMPLETED' = 'ACTIVE';
  private ed25519TaskStatus: 'READY_TO_BEGIN' | 'IN_PROGRESS' | 'COMPLETED' = 'READY_TO_BEGIN';
  private securityGateStatus: 'WAITING' | 'ACTION_REQUIRED' | 'COMPLETED' | 'REJECTED' = 'ACTION_REQUIRED';
  private discretionaryFuzzingAdded = false;

  constructor() {
    // Seed initial historical snapshots
    const baseSnapshot = this.buildSnapshot('usr-dev-alice', 'developer');
    this.history.set(baseSnapshot.cursor, baseSnapshot);

    // Seed checkpoint snapshots matching trajectory query
    const cp1 = {
      ...baseSnapshot,
      cursor: '1.0000000001',
      summary: {
        ...baseSnapshot.summary,
        phase: 'Case Initialization',
        status_phrase: 'Case initiated from Blueprint',
      },
    };
    this.history.set('1.0000000001', cp1);

    const cp2 = {
      ...baseSnapshot,
      cursor: '1.0000000020',
      summary: {
        ...baseSnapshot.summary,
        phase: 'Specification & Architecture',
        status_phrase: 'Specification & Architecture phase completed',
      },
    };
    this.history.set('1.0000000020', cp2);
  }

  private nextCursor(): string {
    this.seq += 1;
    return `${this.epoch}.${String(this.seq).padStart(10, '0')}`;
  }

  public async getSnapshot(caseId: string, actorId: string, role: ActorRole): Promise<CognitiveWorldSnapshot> {
    const snap = this.buildSnapshot(actorId, role);
    this.history.set(snap.cursor, snap);
    return snap;
  }

  public async getSnapshotAt(
    caseId: string,
    cursor: string,
    actorId: string,
    role: ActorRole
  ): Promise<CognitiveWorldSnapshot> {
    const snap = this.history.get(cursor) ?? this.buildSnapshot(actorId, role);
    return {
      ...snap,
      cursor,
      perspective: { actor_id: actorId, role },
      summary: {
        ...snap.summary,
        status_phrase: `Historical Inspection Mode (as of ${cursor})`,
      },
      // Strip consequential actions when viewing history
      visible_objects: snap.visible_objects.map((obj) => ({
        ...obj,
        actions: obj.actions.filter((a) => !a.consequential),
      })),
      available_actions: [
        {
          id: 'act_resume_live',
          label: 'Return to Live Head',
          intent: 'RESUME_LIVE_STREAM',
          consequential: false,
          variant: 'PRIMARY',
        },
      ],
    };
  }

  public async dispatchIntent(intent: InteractionIntent): Promise<IntentResponse> {
    const actorRole = intent.actor.role;

    if (intent.action_name === 'BEGIN_WORK') {
      if (this.ed25519TaskStatus !== 'READY_TO_BEGIN') {
        return {
          intent_id: intent.intent_id,
          success: false,
          error_code: 'INVALID_STATE',
          error_message: `Task is not ready to begin (current status: ${this.ed25519TaskStatus})`,
        };
      }

      this.ed25519TaskStatus = 'IN_PROGRESS';
      const newCursor = this.nextCursor();
      this.broadcastPatch(newCursor);

      // Simulate Gauntlet execution lifecycle in background
      this.simulateGauntletRun();

      return {
        intent_id: intent.intent_id,
        success: true,
        new_cursor: newCursor,
      };
    }

    if (intent.action_name === 'APPROVE_HUMAN_TASK') {
      if (actorRole !== 'security_officer' && actorRole !== 'case_architect') {
        return {
          intent_id: intent.intent_id,
          success: false,
          error_code: 'UNAUTHORIZED_ROLE',
          error_message: 'Only a Security Officer or Case Architect may approve this gate.',
        };
      }

      this.securityGateStatus = 'COMPLETED';
      this.stageState = 'COMPLETED';
      const newCursor = this.nextCursor();
      this.broadcastPatch(newCursor);

      return {
        intent_id: intent.intent_id,
        success: true,
        new_cursor: newCursor,
      };
    }

    if (intent.action_name === 'REJECT_HUMAN_TASK') {
      if (actorRole !== 'security_officer' && actorRole !== 'case_architect') {
        return {
          intent_id: intent.intent_id,
          success: false,
          error_code: 'UNAUTHORIZED_ROLE',
          error_message: 'Only a Security Officer or Case Architect may reject this gate.',
        };
      }

      this.securityGateStatus = 'REJECTED';
      const newCursor = this.nextCursor();
      this.broadcastPatch(newCursor);

      return {
        intent_id: intent.intent_id,
        success: true,
        new_cursor: newCursor,
      };
    }

    if (intent.action_name === 'ADD_DISCRETIONARY_WORK') {
      this.discretionaryFuzzingAdded = true;
      const newCursor = this.nextCursor();
      this.broadcastPatch(newCursor);

      return {
        intent_id: intent.intent_id,
        success: true,
        new_cursor: newCursor,
      };
    }

    if (intent.action_name === 'ESCALATE_OR_OVERRIDE') {
      if (!intent.justification || intent.justification.length < 10) {
        return {
          intent_id: intent.intent_id,
          success: false,
          error_code: 'JUSTIFICATION_REQUIRED',
          error_message: 'Sentry override requires a documented justification.',
        };
      }

      this.securityGateStatus = 'COMPLETED';
      const newCursor = this.nextCursor();
      this.broadcastPatch(newCursor);

      return {
        intent_id: intent.intent_id,
        success: true,
        new_cursor: newCursor,
      };
    }

    // Default response for informational or local intents
    return {
      intent_id: intent.intent_id,
      success: true,
      new_cursor: `${this.epoch}.${String(this.seq).padStart(10, '0')}`,
    };
  }

  public async resolveArtifact(evidenceId: string): Promise<ArtifactPayload> {
    return {
      evidence_id: evidenceId,
      name: 'ed25519_verification_report.md',
      digest: 'sha256:5a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b',
      content_type: 'text/markdown',
      content: `# Cryptographic Verification Report\n\n- **Target**: Ed25519 Bridge Module\n- **Ruler**: .wayfinder/answer-key.md (Digest: sha256:4b5c...)\n- **Tests Run**: 42 passed; 0 failed.\n- **Discrepancy Score**: 0.0 (Zero defect residual)\n- **Settlement Decision**: ACCEPTED by SEA-Forge Policy Engine\n`,
      provenance: {
        case_id: this.activeCaseId,
        plan_item_id: 'pi-stage_impl-004',
        invocation_id: 'inv-lease-opp-pi-7b8c9d0e1f2a3b4c',
        run_id: 'run-ed25519-bridge-20260920101500',
        question_id: 'q-ed25519-sig-valid',
        claim_id: 'clm-sig-verifies-with-fixture',
        commit_sha: '7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b',
        pr_number: 42,
      },
    };
  }

  public async queryTemporalTrajectory(caseId: string): Promise<TemporalTrajectoryResponse> {
    return {
      case_id: caseId,
      base_cursor: '1.0000000001',
      head_cursor: `${this.epoch}.${String(this.seq).padStart(10, '0')}`,
      points: [
        {
          cursor: '1.0000000001',
          timestamp: '2026-09-20T10:00:00Z',
          event_type: 'CaseStarted',
          summary: 'Case initiated from Blueprint: Authentication Gateway Refactor',
          actor_id: 'usr-admin-01',
          actor_role: 'case_architect',
          consequential: true,
          completed_plan_items_count: 0,
          total_plan_items_count: 5,
        },
        {
          cursor: '1.0000000020',
          timestamp: '2026-09-20T10:10:00Z',
          event_type: 'StageAdvanced',
          summary: 'Specification & Architecture phase completed',
          actor_id: 'usr-admin-01',
          actor_role: 'case_architect',
          consequential: true,
          completed_plan_items_count: 2,
          total_plan_items_count: 5,
        },
        {
          cursor: `${this.epoch}.${String(this.seq).padStart(10, '0')}`,
          timestamp: new Date().toISOString(),
          event_type: 'CurrentHead',
          summary: 'Live execution head',
          actor_id: 'system',
          actor_role: 'system',
          consequential: false,
          completed_plan_items_count: this.ed25519TaskStatus === 'COMPLETED' ? 3 : 2,
          total_plan_items_count: 5,
        },
      ],
    };
  }

  public subscribeEvents(
    caseId: string,
    sinceCursor: string | undefined,
    onEvent: (event: StreamEvent) => void,
    _onError: (err: Error) => void
  ): () => void {
    this.listeners.add(onEvent);

    // Initial snapshot on connect
    const snap = this.buildSnapshot('usr-dev-alice', 'developer');
    onEvent({
      event_type: 'snapshot',
      cursor: snap.cursor,
      timestamp: snap.timestamp,
      payload: snap,
    });

    return () => {
      this.listeners.delete(onEvent);
    };
  }

  private simulateGauntletRun(): void {
    const runId = `run-ed25519-bridge-${Date.now()}`;
    const phases: Array<{ phase: ExecutionProgressPayload['phase']; percent: number; log: string }> = [
      { phase: 'orchestrator', percent: 0.2, log: 'Bounding task units against frozen ruler' },
      { phase: 'builder', percent: 0.5, log: 'Builder generated Ed25519 bridge transform in sandbox' },
      { phase: 'critic', percent: 0.8, log: 'Critic evaluating changes against answer key' },
      { phase: 'verifier', percent: 0.95, log: 'Verifier running deterministic cargo test suite' },
      { phase: 'settling', percent: 1.0, log: 'SEA-Forge evaluating evidence against case policy' },
    ];

    phases.forEach((step, index) => {
      setTimeout(() => {
        const cursor = this.nextCursor();
        const payload: ExecutionProgressPayload = {
          run_id: runId,
          phase: step.phase,
          progress_percent: step.percent,
          log_line: step.log,
        };

        this.emitToListeners({
          event_type: 'execution_progress',
          cursor,
          timestamp: new Date().toISOString(),
          payload,
        });

        // Final step: settle task
        if (index === phases.length - 1) {
          this.ed25519TaskStatus = 'COMPLETED';
          const finalCursor = this.nextCursor();
          this.broadcastPatch(finalCursor);
        }
      }, (index + 1) * 300);
    });
  }

  private broadcastPatch(cursor: string): void {
    const snap = this.buildSnapshot('usr-dev-alice', 'developer');
    this.history.set(cursor, snap);
    this.emitToListeners({
      event_type: 'patch',
      cursor,
      timestamp: snap.timestamp,
      payload: snap,
    });
  }

  private emitToListeners(event: StreamEvent): void {
    for (const listener of this.listeners) {
      listener(event);
    }
  }

  private buildSnapshot(actorId: string, role: ActorRole): CognitiveWorldSnapshot {
    const cursor = `${this.epoch}.${String(this.seq).padStart(10, '0')}`;
    const visibleObjects: CognitiveObject[] = [];

    // Stage 1 Object
    visibleObjects.push({
      id: 'stage-review-hardening',
      kind: 'stage',
      name: 'Review & Hardening',
      status: this.stageState === 'COMPLETED' ? 'COMPLETED' : 'ACTIVE',
      badge: this.stageState === 'COMPLETED' ? 'Stage Finished' : 'Active Stage',
      salience: 0.9,
      actions: [],
    });

    // Ed25519 Implementation Task
    visibleObjects.push({
      id: 'obj-task-ed25519-impl',
      kind: 'work_item',
      name: 'Verify Ed25519 Bridge Implementation',
      status:
        this.ed25519TaskStatus === 'READY_TO_BEGIN'
          ? 'READY_TO_BEGIN'
          : this.ed25519TaskStatus === 'IN_PROGRESS'
          ? 'IN_PROGRESS'
          : 'COMPLETED',
      badge:
        this.ed25519TaskStatus === 'READY_TO_BEGIN'
          ? 'Ready to Begin'
          : this.ed25519TaskStatus === 'IN_PROGRESS'
          ? 'Automated Run in Progress'
          : 'Done & Proven',
      explanation: 'Verifies the cryptographic Ed25519 signing bridge against test vectors.',
      salience: this.ed25519TaskStatus === 'READY_TO_BEGIN' ? 1.0 : 0.6,
      parent_id: 'stage-review-hardening',
      actions:
        this.ed25519TaskStatus === 'READY_TO_BEGIN' && role === 'developer'
          ? [
              {
                id: 'act_begin_work',
                label: 'Begin Work (Gauntlet Run)',
                intent: 'BEGIN_WORK',
                consequential: true,
                variant: 'PRIMARY',
              },
            ]
          : this.ed25519TaskStatus === 'COMPLETED'
          ? [
              {
                id: 'view_evidence',
                label: 'Inspect Provenance Evidence',
                intent: 'OPEN_ARTIFACT',
                consequential: false,
                variant: 'SECONDARY',
              },
            ]
          : [],
    });

    // Security Gate Task
    visibleObjects.push({
      id: 'obj-gate-security',
      kind: 'decision_gate',
      name: 'Security Officer Architecture Signoff',
      status:
        this.securityGateStatus === 'ACTION_REQUIRED'
          ? 'ACTION_REQUIRED'
          : this.securityGateStatus === 'COMPLETED'
          ? 'COMPLETED'
          : 'REJECTED',
      badge:
        this.securityGateStatus === 'ACTION_REQUIRED'
          ? 'Needs Decision'
          : this.securityGateStatus === 'COMPLETED'
          ? 'Approved'
          : 'Rejected',
      explanation: 'Review cryptographic diffs and evidence before closing the milestone.',
      salience: 0.95,
      parent_id: 'stage-review-hardening',
      actions:
        role === 'security_officer' || role === 'case_architect'
          ? this.securityGateStatus === 'ACTION_REQUIRED'
            ? [
                {
                  id: 'act_sec_approve',
                  label: 'Approve Architecture',
                  intent: 'APPROVE_HUMAN_TASK',
                  consequential: true,
                  variant: 'PRIMARY',
                },
                {
                  id: 'act_sec_reject',
                  label: 'Reject & Request Remediation',
                  intent: 'REJECT_HUMAN_TASK',
                  consequential: true,
                  variant: 'DANGER',
                },
              ]
            : []
          : [],
    });

    // Optional Discretionary Work
    if (!this.discretionaryFuzzingAdded) {
      visibleObjects.push({
        id: 'obj-disc-pen-test',
        kind: 'discretionary_opportunity',
        name: 'Run Boundary Fuzzing Test Suite',
        status: 'AVAILABLE_TO_ADD',
        badge: 'Optional Work',
        explanation: 'Add boundary fuzzing test suite to the active stage if deeper assurance is desired.',
        salience: 0.4,
        actions:
          role === 'developer' || role === 'case_architect'
            ? [
                {
                  id: 'act_add_work',
                  label: 'Add this work',
                  intent: 'ADD_DISCRETIONARY_WORK',
                  consequential: true,
                  variant: 'SECONDARY',
                },
              ]
            : [],
      });
    } else {
      visibleObjects.push({
        id: 'obj-disc-pen-test-active',
        kind: 'work_item',
        name: 'Boundary Fuzzing Test Suite (Added)',
        status: 'READY_TO_BEGIN',
        badge: 'Discretionary Work Added',
        salience: 0.7,
        actions: [
          {
            id: 'act_begin_fuzz',
            label: 'Begin Fuzzing Run',
            intent: 'BEGIN_WORK',
            consequential: true,
            variant: 'PRIMARY',
          },
        ],
      });
    }

    return {
      world_id: `ws-${this.activeCaseId}-${cursor}`,
      case_id: this.activeCaseId,
      cursor,
      timestamp: new Date().toISOString(),
      perspective: { actor_id: actorId, role },
      summary: {
        headline: 'Authentication Gateway Refactor',
        phase: 'Review & Hardening',
        status_phrase:
          this.securityGateStatus === 'ACTION_REQUIRED'
            ? role === 'security_officer'
              ? 'Action Required: Your security signoff is pending'
              : 'Waiting on security officer review'
            : this.stageState === 'COMPLETED'
            ? 'Milestone Achieved: All gates green'
            : 'Work in progress',
        progress_percent: this.securityGateStatus === 'COMPLETED' ? 1.0 : 0.65,
      },
      visible_objects: visibleObjects,
      available_actions:
        role === 'security_officer' && this.securityGateStatus === 'ACTION_REQUIRED'
          ? [
              {
                id: 'act_sec_override',
                label: 'Emergency Sentry Override',
                intent: 'ESCALATE_OR_OVERRIDE',
                consequential: true,
                requires_justification: true,
                variant: 'WARNING',
              },
            ]
          : [],
      attention_focus: {
        primary_object_id:
          this.securityGateStatus === 'ACTION_REQUIRED' ? 'obj-gate-security' : 'obj-task-ed25519-impl',
        narration:
          this.securityGateStatus === 'ACTION_REQUIRED'
            ? 'Security signoff is the governing gate to proceed.'
            : 'All stage requirements satisfied.',
      },
    };
  }
}
