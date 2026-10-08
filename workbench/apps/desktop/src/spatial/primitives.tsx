// Spatial UI grammar: Surface / Object / Relationship / Focus / Zoom /
// Time-Version / Composer / Artifact. These are the semantic interface of the
// application; generic dashboard/page/card/tab concepts live strictly below
// it as implementation detail. See `docs/CORE_ARCHITECTURE.md` §7.

import type { ReactNode } from "react";

/** A Surface is an arrangement of meaningful things around an orientation. */
export interface SurfaceProps {
  id: string;
  title: string;
  children?: ReactNode;
}

/** An Object is something meaningful that can become a focus. */
export interface SpatialObjectProps {
  id: string;
  label: string;
  focused?: boolean;
  onFocus?: (id: string) => void;
  children?: ReactNode;
}

/** A Relationship expresses meaningful connection between objects. */
export interface RelationshipProps {
  fromId: string;
  toId: string;
  label: string;
}

/** An Artifact is a meaningful inspectable product/evidence/output. */
export interface ArtifactProps {
  id: string;
  title: string;
  children?: ReactNode;
}

export function Surface({ id, title, children }: SurfaceProps) {
  return (
    <section aria-label={title} data-surface={id} data-testid={`surface-${id}`}>
      {children}
    </section>
  );
}

export function SpatialObject({ id, label, focused, onFocus, children }: SpatialObjectProps) {
  return (
    <div
      role="button"
      tabIndex={0}
      aria-label={label}
      aria-current={focused ? "true" : undefined}
      data-object={id}
      data-testid={`object-${id}`}
      onClick={() => onFocus?.(id)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onFocus?.(id);
        }
      }}
    >
      {children ?? label}
    </div>
  );
}

export function Relationship({ fromId, toId, label }: RelationshipProps) {
  return (
    <div data-relationship={`${fromId}->${toId}`} data-testid={`relationship-${fromId}-${toId}`}>
      {label}
    </div>
  );
}

export function Artifact({ id, title, children }: ArtifactProps) {
  return (
    <article aria-label={title} data-artifact={id} data-testid={`artifact-${id}`}>
      {children}
    </article>
  );
}
