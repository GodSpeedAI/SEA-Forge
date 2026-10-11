import { memo } from 'react'
import type { BeatOverrides, Lod, ObjectAction, Placement, Status, WorldObject } from '../model/types'

// One world object. Its representation is chosen by semantic LOD, never by scaling the same
// thing up: dot → labeled sphere → sphere with its parts → (parts render themselves).
// Position, size (--r) and opacity are written per frame by the scene runtime.

export interface ObjectNodeProps {
  obj: WorldObject
  placement: Placement | undefined
  lod: Lod
  parts: WorldObject[]
  isCenter: boolean
  home: boolean
  hovered: boolean
  overrides: BeatOverrides
  /** Center residue at Home ("1 thing needs you"). */
  homeResidue: string | null
  past: boolean
  /** Revision summary shown under the center while standing in the past. */
  pastSummary?: string
  register(el: HTMLElement | null): void
  /** Backend-listed actions, offered on hover/focus. Absent in the past (read-only). */
  onInvoke?: (action: ObjectAction) => void
  selected?: boolean
  /** Progressive disclosure: bring this object's artifacts forward as excerpts. */
  onArtifacts?: () => void
  /** Representation switches offered at the local center (causal view, compare, history, design). */
  centerActions?: { id: string; label: string; pressed?: boolean; onClick: () => void }[]
}

function CompareNote({ c }: { c: NonNullable<Placement['compare']> }) {
  const inside = c.inside ? `${c.inside} change${c.inside > 1 ? 's' : ''} inside` : ''
  if (c.change === 'same') {
    return inside ? (
      <div className="cmp-note cmp-changed" data-testid="compare-note" data-change="inside">
        {inside}
      </div>
    ) : null
  }
  const text =
    c.change === 'added'
      ? 'New in B'
      : c.change === 'removed'
        ? 'Not in B'
        : c.a?.label !== c.b?.label
          ? `${c.a?.label ?? '—'} → ${c.b?.label ?? '—'}`
          : c.aTitle !== c.bTitle
            ? `${c.aTitle ?? '—'} → ${c.bTitle ?? '—'}`
            : `Changed: ${(c.fields ?? []).join(', ') || 'details'}`
  return (
    <div className={`cmp-note cmp-${c.change}`} data-testid="compare-note" data-change={c.change}>
      {text}
    </div>
  )
}

export const ObjectNode = memo(function ObjectNode(p: ObjectNodeProps) {
  const { obj, placement: pl, lod } = p
  const title = pl?.title ?? obj.title
  const subtitle = pl?.subtitle ?? obj.metric ?? obj.subtitle
  const status = pl?.status ?? obj.status
  const receded = pl?.role === 'receded'
  const satellite = !!pl?.satellite
  const tone = status?.tone ?? 'muted'
  const cls = [
    'node',
    `kind-${obj.kind}`,
    `lod-${lod}`,
    `tone-${tone}`,
    receded && 'receded',
    pl?.role === 'faded' && 'faded',
    satellite && 'satellite',
    p.isCenter && 'center',
    obj.ghost && 'ghost',
    obj.dormant && 'dormant',
    obj.accent && `accent-${obj.accent}`,
    obj.icon && 'has-icon',
    p.hovered && 'hovered',
    p.selected && 'selected',
    pl?.highlight && 'highlighted',
    pl?.compare && `cmp-${pl.compare.change}`,
    `side-${pl?.labelSide ?? 'right'}`,
  ]
    .filter(Boolean)
    .join(' ')

  if (p.isCenter) {
    const caption = p.overrides.caption
    const residue: Status | undefined =
      p.overrides.residue ?? (p.past && p.pastSummary ? { label: p.pastSummary, tone: 'muted' } : obj.residue)
    const sub = p.overrides.subcaption ?? (p.past ? 'Earlier state' : obj.subtitle)
    return (
      <div ref={p.register} className={cls} data-node={obj.id} aria-label={`${obj.title}, current center`}>
        {pl?.labelInside && !caption ? (
          <div className="void-title">
            <div className="vt-title">{obj.title}</div>
            {obj.subtitle && <div className="vt-sub">{obj.subtitle}</div>}
            {obj.status && (
              <div className={`vt-status tone-${obj.status.tone}`}>
                <Dot />
                {obj.status.label}
              </div>
            )}
          </div>
        ) : caption ? (
          <div className="void-caption" key={caption}>
            {caption}
          </div>
        ) : p.home && p.homeResidue ? (
          <div className="void-caption residue-home">
            <span className="rh-1">Good progress</span>
            <span className="rh-2">{p.homeResidue}</span>
          </div>
        ) : null}
        {!!p.centerActions?.length && (
          <div className="center-actions" role="toolbar" aria-label="Representations">
            {p.centerActions.map((a) => (
              <button
                key={a.id}
                type="button"
                className="center-chip"
                data-testid={`center-${a.id}`}
                aria-pressed={a.pressed}
                onPointerDown={(e) => e.stopPropagation()}
                onClick={(e) => {
                  e.stopPropagation()
                  a.onClick()
                }}
              >
                {a.label}
              </button>
            ))}
          </div>
        )}
        {obj.kind !== 'core' && !pl?.labelInside && (
          <div className="center-label">
            <div className="cl-title">{obj.title}</div>
            {sub && <div className="cl-sub">{sub}</div>}
            {!!obj.artifacts?.length && p.onArtifacts && (
              <button
                type="button"
                className="artifact-pill"
                data-testid="artifact-pill"
                aria-label={`Show ${obj.artifacts.length} artifact${obj.artifacts.length > 1 ? 's' : ''} for ${obj.title}`}
                onPointerDown={(e) => e.stopPropagation()}
                onClick={(e) => {
                  e.stopPropagation()
                  p.onArtifacts!()
                }}
              >
                ▤ {obj.artifacts.length} artifact{obj.artifacts.length > 1 ? 's' : ''}
              </button>
            )}
            
            {residue && (
              <div className={`cl-residue tone-${residue.tone}`}>
                {p.past ? <ClockIcon /> : residue.tone === 'critical' || residue.tone === 'attention' ? <WarnIcon /> : <Dot />}
                <span>{residue.label}</span>
              </div>
            )}
          </div>
        )}
      </div>
    )
  }

  const showLabel = !receded && (!satellite || lod >= 1 || p.hovered || tone === 'attention' || tone === 'critical')
  return (
    <div
      ref={p.register}
      className={cls}
      data-node={obj.id}
      data-kind={obj.kind}
      data-change={pl?.compare?.change}
      tabIndex={receded ? -1 : 0}
      role="button"
      aria-pressed={p.selected || undefined}
      aria-label={`${title}${status ? `, ${status.label}` : ''}${pl?.compare && pl.compare.change !== 'same' ? `, ${pl.compare.change} between A and B` : ''}`}
    >
      {pl?.annotation && !receded && <div className="annotation-pill" data-testid="annotation">{pl.annotation}</div>}
      <div className={lod === 0 && !obj.icon ? 'dot' : 'orb'}>{obj.icon && <Glyph name={obj.icon} />}</div>
      {showLabel && lod === 0 && (
        <div className="lbl lbl-0">
          <span className="l0-title">{title}</span>
          {status && !satellite && <span className={`l0-status tone-${status.tone}`}>{status.label}</span>}
          {pl?.compare && <CompareNote c={pl.compare} />}
        </div>
      )}
      {showLabel && lod >= 1 && (
        <div className="lbl lbl-1">
          <div className="l1-title">{title}</div>
          {subtitle && <div className="l1-sub">{subtitle}</div>}
          {status && (
            <div className={`l1-status tone-${status.tone}`}>
              <Dot />
              {status.label}
            </div>
          )}
          {pl?.compare && <CompareNote c={pl.compare} />}
          {!!obj.artifacts?.length && p.onArtifacts && (
            <button
              type="button"
              className="artifact-pill"
              data-testid="artifact-pill"
              aria-label={`Show ${obj.artifacts.length} artifact${obj.artifacts.length > 1 ? 's' : ''} for ${obj.title}`}
              onPointerDown={(e) => e.stopPropagation()}
              onClick={(e) => {
                e.stopPropagation()
                p.onArtifacts!()
              }}
            >
              ▤ {obj.artifacts.length} artifact{obj.artifacts.length > 1 ? 's' : ''}
            </button>
          )}
          {!!obj.actions?.length && p.onInvoke && !p.past && (
            <div className="l1-actions">
              {obj.actions.map((a) => (
                <button
                  key={a.id}
                  type="button"
                  className={`action-chip${a.consequential ? ' needs-judgment' : ''}`}
                  data-testid="action-chip"
                  data-action={a.id}
                  onPointerDown={(e) => e.stopPropagation()}
                  onClick={(e) => {
                    e.stopPropagation()
                    p.onInvoke!(a)
                  }}
                >
                  {a.label} →
                </button>
              ))}
            </div>
          )}
          {lod === 2 && p.parts.length > 0 && (
            <ul className="l2-parts">
              {p.parts.slice(0, 4).map((c) => (
                <li key={c.id} className={`tone-${c.status?.tone ?? 'muted'}`}>
                  <Dot />
                  {c.title}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {pl?.role && !receded && pl.role !== 'faded' && <div className="role-pill">{pl.role}</div>}
    </div>
  )
})

function Glyph({ name }: { name: NonNullable<WorldObject['icon']> }) {
  const paths: Record<string, string> = {
    target: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Zm0 4a5 5 0 1 0 0 10 5 5 0 0 0 0-10Zm0 3.5a1.5 1.5 0 1 0 0 3 1.5 1.5 0 0 0 0-3Z',
    layers: 'M12 3 3 8l9 5 9-5-9-5Zm-9 9 9 5 9-5M3 16l9 5 9-5',
    people: 'M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm-6 9c0-3.3 2.7-5.5 6-5.5s6 2.2 6 5.5M16 4.5a3.3 3.3 0 0 1 0 6.3M18 14.8c1.8.8 3 2.6 3 5.2',
    doc: 'M6 3h8l4 4v14H6V3Zm8 0v4h4M9 12h6M9 16h6',
    flag: 'M6 21V4m0 0h10l-2 4 2 4H6',
    shield: 'M12 3 4.5 6v6c0 4.5 3.2 7.8 7.5 9 4.3-1.2 7.5-4.5 7.5-9V6L12 3Zm-3.5 9 2.5 2.5 4.5-5',
    bolt: 'M13 2 5 14h6l-1 8 8-12h-6l1-8Z',
    history: 'M4 12a8 8 0 1 0 2.3-5.7M4 4v4h4M12 8v4l3 2',
    eye: 'M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12Zm10-3a3 3 0 1 0 0 6 3 3 0 0 0 0-6Z',
  }
  return (
    <svg className="orb-glyph" viewBox="0 0 24 24" aria-hidden>
      <path d={paths[name]} />
    </svg>
  )
}

function Dot() {
  return <i className="sdot" aria-hidden />
}

function WarnIcon() {
  return (
    <svg className="ico" width="16" height="15" viewBox="0 0 16 15" aria-hidden>
      <path d="M8 1 15 14H1Z" fill="currentColor" />
      <path d="M8 5.5v4" stroke="#fff" strokeWidth="1.5" strokeLinecap="round" />
      <circle cx="8" cy="11.6" r=".9" fill="#fff" />
    </svg>
  )
}

function ClockIcon() {
  return (
    <svg className="ico" width="15" height="15" viewBox="0 0 16 16" aria-hidden>
      <circle cx="8" cy="8" r="6.4" fill="none" stroke="currentColor" strokeWidth="1.3" />
      <path d="M8 4.5V8l2.4 1.6" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
    </svg>
  )
}
