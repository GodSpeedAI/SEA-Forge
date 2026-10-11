import type { JSX } from 'react'
import type { TemplateEntryOption } from '../ports/contract'
import type { ProposalState } from '../model/types'
import './proposal.css'

// Case design from templates (T09, CJ04): pick a template, fill its parameters, run preflight
// (the digest is shown), and only then commit with PROPOSE_CASE. The submit button stays
// disabled until a preflight for the CURRENT parameters has passed — the digest binds the
// commit to exactly what was reviewed.

export interface TemplateDesignPanelProps {
  proposal: ProposalState
  onSelectTemplate(templateRef: string | null): void
  onSetParam(name: string, value: string): void
  onPreflight(): void
  onSubmit(): void
  onClose(): void
}

export function TemplateDesignPanel(p: TemplateDesignPanelProps): JSX.Element {
  const pr = p.proposal
  const selected: TemplateEntryOption | undefined = pr.templates.find((t) => t.template_ref === pr.selected)
  const preflightCurrent = !!pr.preflight && !pr.preflighting
  const canSubmit = preflightCurrent && pr.preflight!.passed && !!pr.preflight!.digest && !pr.submitting && !pr.result

  return (
    <div className="proposal-panel" data-testid="proposal-panel" role="dialog" aria-label="Design a case">
      <div className="proposal-header">
        <span className="proposal-title">Design a case</span>
        <button type="button" className="proposal-close" data-testid="proposal-close" aria-label="Close" onClick={p.onClose}>
          ×
        </button>
      </div>

      <div className="proposal-body">
        {pr.status === 'loading' && <div className="proposal-status" role="status">Loading templates…</div>}
        {pr.status === 'unavailable' && (
          <div className="proposal-status proposal-status--error" role="alert" data-testid="proposal-unavailable">
            Templates are not available in this deployment. {pr.error}
          </div>
        )}

        {pr.status === 'ready' && (
          <>
            <div className="proposal-section">
              <h3 className="proposal-heading">Template</h3>
              <div className="proposal-templates" role="listbox" aria-label="Templates">
                {pr.templates.map((t) => (
                  <button
                    key={t.template_ref}
                    type="button"
                    className={`proposal-template${pr.selected === t.template_ref ? ' selected' : ''}`}
                    data-testid="proposal-template"
                    data-ref={t.template_ref}
                    role="option"
                    aria-selected={pr.selected === t.template_ref}
                    onClick={() => p.onSelectTemplate(t.template_ref)}
                  >
                    <span className="proposal-template-title">{t.title}</span>
                    {t.description && <span className="proposal-template-desc">{t.description}</span>}
                  </button>
                ))}
                {pr.templates.length === 0 && <div className="proposal-status">No templates are published.</div>}
              </div>
            </div>

            {selected && (
              <>
                <div className="proposal-section">
                  <h3 className="proposal-heading">Parameters</h3>
                  {selected.parameters.map((param) => (
                    <label key={param.name} className="proposal-param">
                      <span className="proposal-param-label">
                        {param.title ?? param.name}
                        {param.required && <em className="proposal-param-required"> required</em>}
                      </span>
                      {param.options ? (
                        <select
                          data-testid="proposal-param"
                          data-name={param.name}
                          value={pr.params[param.name] ?? param.default ?? ''}
                          onChange={(e) => p.onSetParam(param.name, e.currentTarget.value)}
                        >
                          {param.options.map((o) => (
                            <option key={o} value={o}>
                              {o}
                            </option>
                          ))}
                        </select>
                      ) : (
                        <input
                          type="text"
                          data-testid="proposal-param"
                          data-name={param.name}
                          value={pr.params[param.name] ?? ''}
                          placeholder={param.default ? `default: ${param.default}` : undefined}
                          onChange={(e) => p.onSetParam(param.name, e.currentTarget.value)}
                        />
                      )}
                      {param.description && <span className="proposal-param-desc">{param.description}</span>}
                    </label>
                  ))}
                </div>

                <div className="proposal-section">
                  <h3 className="proposal-heading">Preflight</h3>
                  <button type="button" className="proposal-preflight" data-testid="proposal-preflight" onClick={p.onPreflight} disabled={pr.preflighting || pr.submitting}>
                    {pr.preflighting ? 'Running preflight…' : 'Run preflight'}
                  </button>
                  {pr.preflight && !pr.preflighting && (
                    <div
                      className={`proposal-preflight-result ${pr.preflight.passed ? 'passed' : 'failed'}`}
                      data-testid="proposal-preflight-result"
                      data-passed={pr.preflight.passed}
                    >
                      {pr.preflight.passed ? (
                        <>
                          <span className="proposal-preflight-ok">Preflight passed.</span>
                          <span className="proposal-preflight-digest">
                            digest <code data-testid="proposal-digest">{pr.preflight.digest}</code>
                          </span>
                        </>
                      ) : (
                        <span className="proposal-preflight-reasons">
                          {pr.preflight.reasons.map((r) => (
                            <span key={r} className="proposal-preflight-reason">
                              {r}
                            </span>
                          ))}
                        </span>
                      )}
                    </div>
                  )}
                </div>
              </>
            )}

            {pr.submitError && (
              <div className="proposal-error" role="alert" data-testid="proposal-error">
                {pr.submitCode && <code className="proposal-error-code">{pr.submitCode}</code>} {pr.submitError}
              </div>
            )}
            {pr.result && (
              <div className="proposal-ok" role="status" data-testid="proposal-result">
                Case committed. Opening {pr.result.caseId}…
              </div>
            )}
          </>
        )}
      </div>

      <div className="proposal-footer">
        <button
          type="button"
          className="proposal-submit"
          data-testid="proposal-submit"
          disabled={!canSubmit}
          title={
            !selected
              ? 'Choose a template first'
              : !pr.preflight
                ? 'Run preflight before committing'
                : !pr.preflight.passed
                  ? 'Preflight must pass before committing'
                  : pr.submitting
                    ? 'Committing…'
                    : 'Commit this case'
          }
          onClick={() => canSubmit && p.onSubmit()}
        >
          {pr.submitting ? 'Committing…' : 'Commit case'}
        </button>
        <span className="proposal-submit-note">Submit unlocks only after preflight passes.</span>
      </div>
    </div>
  )
}
