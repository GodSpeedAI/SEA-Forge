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

/**
 * Event kinds the server emits today, verified against its `publish_event`
 * call sites.
 *
 * The two `agent_run.*` kinds were emitted by the server well before they were
 * listed here. Because the narrowing is asymmetric they over-invalidated rather
 * than being ignored, so nothing rendered stale — which is exactly the failure
 * mode the asymmetry exists to produce, and exactly why the drift went
 * unnoticed. Adding a kind here is what lets a surface *stop* refetching on it.
 */
export const KNOWN_EVENT_KINDS = [
  "case.submitted",
  "approval.approved",
  "approval.rejected",
  "agent_run.delegated",
  "agent_run.cancellation_requested",
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
  // Delegating or cancelling changes neither the self-model nor the endpoint
  // set. It *does* change an endpoint's standing once the probe settles, but
  // that is `asset.list`'s projection, not readiness'.
  "agent_run.delegated",
  "agent_run.cancellation_requested",
];

/** Kinds that can change a case list, overview, or horizon. */
const CASE_RELEVANT: readonly string[] = [
  "case.submitted",
  "approval.approved",
  "approval.rejected",
  // A delegation runs inside a case and moves its horizon.
  "agent_run.delegated",
  "agent_run.cancellation_requested",
];

/** Kinds that can change the approval inbox. */
const APPROVAL_RELEVANT: readonly string[] = ["approval.approved", "approval.rejected"];

/**
 * Kinds that can change the delegation roster.
 *
 * Both `agent_run.*` kinds move a row's standing: one adds a delegation, the
 * other flips `cancellable` to false. `case.submitted` can too — a committed
 * case may dispatch an agent task — so it is included rather than excluded on
 * the grounds that it is "a case event".
 */
const DELEGATION_RELEVANT: readonly string[] = [
  "agent_run.delegated",
  "agent_run.cancellation_requested",
  "case.submitted",
];

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

/** True when `frame` should invalidate the delegation roster. */
export function affectsDelegations(frame: unknown): boolean {
  const kind = eventKindOf(frame);
  if (kind === undefined) return true;
  return !isKnown(kind) || DELEGATION_RELEVANT.includes(kind);
}
