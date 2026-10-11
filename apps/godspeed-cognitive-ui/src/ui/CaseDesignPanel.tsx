import type { JSX } from "react";
import "./case-design.css";
import React from "react";

export interface DesignItem {
  id: string;
  label: string;
  detail?: string;
  required?: boolean;
}

export interface CaseDesignModel {
  name: string;
  versionLabel: string;
  goal: string;
  description: string;
  stages: DesignItem[];
  roles: DesignItem[];
  evidence: DesignItem[];
}

export interface CaseDesignPanelProps {
  model: CaseDesignModel;
  isDraft: boolean;
  dirty: boolean;
  versions: { id: string; label: string }[];
  showing: string;
  selected: string | null;
  onShowVersion(id: string): void;
  onCompare(a: string, b: string): void;
  onMoveStage(stageId: string, dir: -1 | 1): void;
  onToggleEvidence(evidenceId: string): void;
  onSaveDraft(): void;
  onDiscardDraft(): void;
  onClose(): void;
  submitDisabledReason: string;
}

export function CaseDesignPanel(p: CaseDesignPanelProps): JSX.Element {
  const [savingDraft, setSavingDraft] = React.useState(false);

  const hasLatestPublished = p.versions.length > 0 && p.versions[0].id !== 'draft';
  // With a draft: latest published ↔ draft. Without: the two latest published versions.
  const published = p.versions.filter((v) => v.id !== 'draft');
  const hasDraft = p.versions.some((v) => v.id === 'draft');
  const compareA = (hasDraft ? published[published.length - 1]?.id : published[published.length - 2]?.id) ?? '';
  const compareB = (hasDraft ? 'draft' : published[published.length - 1]?.id) ?? '';
  void hasLatestPublished;
  const canCompare = compareA && compareB;

  const handleSaveDraft = async () => {
    setSavingDraft(true);
    try {
      await Promise.resolve(p.onSaveDraft());
    } finally {
      setSavingDraft(false);
    }
  };

  return (
    <div className="case-design-panel" data-testid="design-panel">
      <div className="case-design-header">
        <div className="case-design-title-pill">
          <div className="case-design-name">{p.model.name}</div>
          <div className={`case-design-version-pill ${p.isDraft ? 'attention' : ''}`}>
            {p.isDraft ? 'Local draft · not submitted' : p.model.versionLabel}
          </div>
        </div>
      </div>

      <div className="case-design-version-selector">
        {p.versions.map((ver) => (
          <button
            key={ver.id}
            data-testid="design-version"
            data-version={ver.id}
            aria-pressed={ver.id === p.showing}
            className="case-design-version-button"
            onClick={() => p.onShowVersion(ver.id)}
          >
            {ver.label}
          </button>
        ))}
      </div>

      <button
        data-testid="design-compare"
        className="case-design-compare-button"
        disabled={!canCompare}
        onClick={() => canCompare && p.onCompare(compareA, compareB)}
      >
        Compare
      </button>

      <div className="case-design-scroll">
        <div className="case-details-section">
          <h2 className="case-design-heading">Goal</h2>
          <div className="case-design-readonly-text">{p.model.goal}</div>
        </div>

        <div className="case-details-section">
          <h2 className="case-design-heading">Description</h2>
          <div className="case-design-readonly-text">{p.model.description}</div>
        </div>

        <div className="case-details-section" data-selected={p.selected?.includes('stages')}>
          <h2 className="case-design-heading">Stages</h2>
          <div className="stage-list">
            {p.model.stages.map((stage, idx) => (
              <div key={stage.id} className="stage-item" data-testid="design-stage" data-id={stage.id}>
                <div className="stage-content">
                  <div className="stage-name">{stage.label}</div>
                  {stage.detail && <div className="stage-detail">{stage.detail}</div>}
                </div>
                <div className="stage-controls">
                  <button
                    data-testid="stage-up"
                    className="stage-move-button"
                    disabled={idx === 0}
                    onClick={() => p.onMoveStage(stage.id, -1)}
                    aria-label="Move stage up"
                  >
                    ▲
                  </button>
                  <button
                    data-testid="stage-down"
                    className="stage-move-button"
                    disabled={idx === p.model.stages.length - 1}
                    onClick={() => p.onMoveStage(stage.id, 1)}
                    aria-label="Move stage down"
                  >
                    ▼
                  </button>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="case-details-section" data-selected={p.selected?.includes('roles')}>
          <h2 className="case-design-heading">Roles</h2>
          <div className="roles-list">
            {p.model.roles.map((role) => (
              <div key={role.id} className="role-item">
                <div className="role-name">{role.label}</div>
                {role.detail && <div className="role-detail">{role.detail}</div>}
              </div>
            ))}
          </div>
        </div>

        <div className="case-details-section" data-selected={p.selected?.includes('evidence')}>
          <h2 className="case-design-heading">Evidence</h2>
          <div className="evidence-list">
            {p.model.evidence.map((evidence) => (
              <div key={evidence.id} className="evidence-item">
                <label className="evidence-label">
                  <input
                    type="checkbox"
                    data-testid="evidence-required"
                    className="evidence-checkbox"
                    checked={evidence.required ?? false}
                    onChange={() => p.onToggleEvidence(evidence.id)}
                  />
                  <span className="evidence-text">
                    <div className="evidence-name">{evidence.label}</div>
                    {evidence.detail && <div className="evidence-detail">{evidence.detail}</div>}
                  </span>
                </label>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="case-design-footer">
        <button
          data-testid="design-save"
          className="case-design-save-button"
          disabled={!p.dirty}
          onClick={handleSaveDraft}
        >
          {savingDraft ? 'Saving…' : p.dirty ? 'Save draft' : p.versions.some((v) => v.id === 'draft') ? 'Saved locally' : 'Save draft'}
        </button>
        <button
          data-testid="design-discard"
          className="case-design-discard-button"
          disabled={!p.isDraft}
          onClick={p.onDiscardDraft}
        >
          Discard draft
        </button>
        <div className="case-design-submit-container">
          <button
            data-testid="design-submit"
            className="case-design-submit-button"
            disabled={true}
            title={p.submitDisabledReason}
            aria-label={`Submit disabled: ${p.submitDisabledReason}`}
          >
            Submit proposal
          </button>
          {p.submitDisabledReason && (
            <div className="case-design-submit-reason">{p.submitDisabledReason}</div>
          )}
        </div>
        <button
          data-testid="design-close"
          className="case-design-close-button"
          onClick={p.onClose}
          aria-label="Close"
        >
          ×
        </button>
      </div>
    </div>
  );
}
