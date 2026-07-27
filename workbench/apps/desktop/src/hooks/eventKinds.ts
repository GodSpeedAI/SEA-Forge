/**
 * Which durable event kinds matter to which surface.
 *
 * Before this module every surface refetched on every `sfwp://event` frame,
 * because no taxonomy existed to narrow against. That is correct but wasteful:
 * on a busy cell, a case event refetches readiness even though readiness cannot
 * have changed.
 *
 * The narrowing rule is deliberately asymmetric. A kind is skipped only when it
 * is *known* to be irrelevant to a surface; an **unrecognized** kind still
 * invalidates. A new server event kind therefore starts out over-invalidating
 * (harmless, just an extra read) rather than being silently ignored (a stale
 * render the operator has no way to detect). The failure modes are not
 * symmetric, so the default must not be either.
 */

/** Event kinds the server emits today, verified against its publish sites. */
export const KNOWN_EVENT_KINDS = [
  "case.submitted",
  "approval.approved",
  "approval.rejected",
] as const;

export type KnownEventKind = (typeof KNOWN_EVENT_KINDS)[number];

function isKnown(kind: string): kind is KnownEventKind {
  return (KNOWN_EVENT_KINDS as readonly string[]).includes(kind);
}

/**
 * Kinds that cannot change the readiness projection.
 *
 * `readiness.get` projects self-model validation plus agent-endpoint config.
 * None of the kinds below touch either: committing a case and deciding an
 * approval leave the self-model and endpoint set untouched. There is still no
 * `self_model.changed` / `endpoint.updated` kind to subscribe to positively,
 * which is why this is expressed as an exclusion list rather than an inclusion
 * one — see `.agents/OBSERVED_DEBT.md`.
 */
const READINESS_IRRELEVANT: readonly string[] = [
  "case.submitted",
  "approval.approved",
  "approval.rejected",
];

/** Kinds that can change a case list, overview, or horizon. */
const CASE_RELEVANT: readonly string[] = [
  "case.submitted",
  "approval.approved",
  "approval.rejected",
];

/** Kinds that can change the approval inbox. */
const APPROVAL_RELEVANT: readonly string[] = ["approval.approved", "approval.rejected"];

/** Read the `kind` off a raw event frame without asserting its full shape. */
export function eventKindOf(frame: unknown): string | undefined {
  const kind = (frame as { kind?: unknown } | undefined)?.kind;
  return typeof kind === "string" ? kind : undefined;
}

/**
 * True when `frame` should invalidate the readiness view. An absent or
 * unrecognized kind invalidates, per the asymmetry above.
 */
export function affectsReadiness(frame: unknown): boolean {
  const kind = eventKindOf(frame);
  if (kind === undefined) return true;
  return !READINESS_IRRELEVANT.includes(kind);
}

/** True when `frame` should invalidate case views. */
export function affectsCases(frame: unknown): boolean {
  const kind = eventKindOf(frame);
  if (kind === undefined) return true;
  return !isKnown(kind) || CASE_RELEVANT.includes(kind);
}

/** True when `frame` should invalidate the approval inbox. */
export function affectsApprovals(frame: unknown): boolean {
  const kind = eventKindOf(frame);
  if (kind === undefined) return true;
  return !isKnown(kind) || APPROVAL_RELEVANT.includes(kind);
}
