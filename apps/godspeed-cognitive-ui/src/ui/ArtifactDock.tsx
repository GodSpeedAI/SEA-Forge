import { useState, useRef, useEffect } from 'react'
import type { CognitiveArtifact } from '../ports/contract'
import type { ArtifactState } from '../artifacts/service'
import { RENDERER_LABEL } from '../artifacts/model'
import { SourceRenderer } from '../artifacts/registry'
import './artifact-dock.css'

export interface ArtifactDockProps {
  descriptor: CognitiveArtifact
  state: ArtifactState
  /** Other artifacts bound to the same object (shown as tabs; includes this one). */
  siblings: CognitiveArtifact[]
  /** e.g. ['Northstar', 'Missing Secondary Coverage'] */
  breadcrumb: string[]
  pinned: boolean
  onSelectSibling(ref: string): void
  onClose(): void
  onPin(pinned: boolean): void
  onFocusObject(objectId: string): void
  onSetTime(cursor: string): void
}

export function ArtifactDock(p: ArtifactDockProps): JSX.Element {
  const [query, setQuery] = useState('')
  const [matches, setMatches] = useState(0)
  const bodyRef = useRef<HTMLDivElement>(null)

  // Reset query when ref changes
  useEffect(() => {
    setQuery('')
  }, [p.descriptor.ref])

  const title =
    p.state.status === 'ready' ? p.state.payload.name : p.descriptor.title
  const kindLabel =
    p.state.status === 'ready'
      ? RENDERER_LABEL[p.state.model.kind]
      : 'Artifact'

  const handleBodyWheel = (e: React.WheelEvent) => {
    e.stopPropagation()
  }

  return (
    <aside
      className="artifact-dock"
      data-testid="artifact-dock"
      data-ref={p.descriptor.ref}
      role="complementary"
      aria-label="Artifact viewer"
    >
      {/* Header row: breadcrumb, title, kind badge, pin, close */}
      <div className="artifact-dock__header">
        <div className="artifact-dock__breadcrumb">
          {p.breadcrumb.map((item, i) => (
            <span key={i}>
              {i > 0 && <span className="artifact-dock__breadcrumb-sep"> / </span>}
              <span>{item}</span>
            </span>
          ))}
        </div>

        <div className="artifact-dock__header-actions">
          <h2 className="artifact-dock__title">{title}</h2>
          <span className="artifact-dock__kind-badge">{kindLabel}</span>

          <button
            className="artifact-dock__button"
            data-testid="dock-pin"
            aria-pressed={p.pinned}
            onClick={() => p.onPin(!p.pinned)}
            title={p.pinned ? 'Pinned · local' : 'Pin'}
          >
            {p.pinned ? '📌' : '📍'}
          </button>

          <button
            className="artifact-dock__button artifact-dock__button--close"
            data-testid="dock-close"
            aria-label="Close artifact"
            onClick={p.onClose}
          >
            ×
          </button>
        </div>
      </div>

      {/* Tabs: one per sibling */}
      {p.siblings.length > 1 && (
        <div className="artifact-dock__tabs" role="tablist">
          {p.siblings.map((sibling) => (
            <button
              key={sibling.ref}
              className={`artifact-dock__tab ${
                sibling.ref === p.descriptor.ref
                  ? 'artifact-dock__tab--active'
                  : ''
              }`}
              role="tab"
              aria-selected={sibling.ref === p.descriptor.ref}
              data-testid="dock-tab"
              data-ref={sibling.ref}
              onClick={() => p.onSelectSibling(sibling.ref)}
            >
              {sibling.title}
            </button>
          ))}
        </div>
      )}

      {/* Search bar */}
      <div className="artifact-dock__search-container">
        <input
          type="search"
          className="artifact-dock__search"
          data-testid="dock-search"
          placeholder="Search in artifact"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <span className="artifact-dock__search-matches">
          {matches > 0
            ? `${matches} ${matches === 1 ? 'match' : 'matches'}`
            : 'No matches'}
        </span>
      </div>

      {/* Body: loading, error, or renderer */}
      <div
        className="artifact-dock__body"
        ref={bodyRef}
        onWheel={handleBodyWheel}
      >
        {p.state.status === 'loading' && (
          <div className="artifact-dock__status">Resolving source…</div>
        )}

        {p.state.status === 'error' && (
          <div data-testid="artifact-error" className="artifact-dock__error">
            <div className="artifact-dock__error-message">
              Could not load this artifact
            </div>
            <div className="artifact-dock__error-text">{p.state.error}</div>
            <div className="artifact-dock__error-ref">
              <code>{p.descriptor.ref}</code>
            </div>
            {p.state.payload && (
              <div className="artifact-dock__error-digest">
                Digest: <code>{p.state.payload.digest.slice(0, 19)}</code>
              </div>
            )}
          </div>
        )}

        {p.state.status === 'ready' && (
          <SourceRenderer
            model={p.state.model}
            payload={p.state.payload}
            descriptor={p.descriptor}
            query={query}
            onMatches={setMatches}
            onFocusObject={p.onFocusObject}
            onSetTime={p.onSetTime}
            fallbackRef={p.descriptor.ref}
          />
        )}
      </div>

      {/* Footer: provenance */}
      {p.state.status === 'ready' && (
        <div className="artifact-dock__footer" data-testid="dock-provenance">
          <div className="artifact-dock__provenance">
            <div className="artifact-dock__provenance-field">
              <span className="artifact-dock__provenance-key">Evidence:</span>
              <span className="artifact-dock__provenance-value">
                {p.state.payload.evidence_id}
              </span>
            </div>
            <div className="artifact-dock__provenance-field">
              <span className="artifact-dock__provenance-key">Digest:</span>
              <span className="artifact-dock__provenance-value">
                <code data-testid="dock-digest" data-digest={p.state.payload.digest}>{p.state.payload.digest.slice(0, 19)}</code>
              </span>
            </div>
            <div className="artifact-dock__provenance-field">
              <span className="artifact-dock__provenance-key">Type:</span>
              <span className="artifact-dock__provenance-value">
                {p.state.payload.content_type}
              </span>
            </div>

            {p.state.payload.provenance && (
              <>
                {p.state.payload.provenance.case_id && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">
                      case_id
                    </span>
                    <span className="artifact-dock__provenance-value">
                      {p.state.payload.provenance.case_id}
                    </span>
                  </div>
                )}
                {p.state.payload.provenance.plan_item_id && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">
                      plan_item_id
                    </span>
                    <span className="artifact-dock__provenance-value">
                      {p.state.payload.provenance.plan_item_id}
                    </span>
                  </div>
                )}
                {p.state.payload.provenance.invocation_id && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">
                      invocation_id
                    </span>
                    <span className="artifact-dock__provenance-value">
                      {p.state.payload.provenance.invocation_id}
                    </span>
                  </div>
                )}
                {p.state.payload.provenance.run_id && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">run_id</span>
                    <span className="artifact-dock__provenance-value">
                      {p.state.payload.provenance.run_id}
                    </span>
                  </div>
                )}
                {p.state.payload.provenance.question_id && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">
                      question_id
                    </span>
                    <span className="artifact-dock__provenance-value">
                      {p.state.payload.provenance.question_id}
                    </span>
                  </div>
                )}
                {p.state.payload.provenance.commit_sha && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">
                      commit_sha
                    </span>
                    <span className="artifact-dock__provenance-value">
                      <code>
                        {p.state.payload.provenance.commit_sha.slice(0, 7)}
                      </code>
                    </span>
                  </div>
                )}
                {p.state.payload.provenance.pr_number && (
                  <div className="artifact-dock__provenance-field">
                    <span className="artifact-dock__provenance-key">
                      pr_number
                    </span>
                    <span className="artifact-dock__provenance-value">
                      #{p.state.payload.provenance.pr_number}
                    </span>
                  </div>
                )}
              </>
            )}
          </div>
          <div className="artifact-dock__provenance-source">
            Source-backed · resolved through the contract port
          </div>
        </div>
      )}
    </aside>
  )
}
