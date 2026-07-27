import { GovernedStatusPill } from "@sea-forge/ui-components";
import { useEvidenceContext } from "../shell/EvidenceContext";
import {
  STANDING_PRESENTATION,
  standingOf,
  useServerContract,
} from "../hooks/useServerContract";
import styles from "./SurfacesPages.module.css";

/**
 * A surface whose kernel projection this cell does not provide yet.
 *
 * These routes exist because the journey is designed around them, but the
 * method that would fill them is absent from the kernel's own catalog. The
 * surface therefore reports its standing from what `system.hello` said, not
 * from a claim baked into this build — so the day the kernel implements the
 * method, the surface stops saying it is missing without anyone editing this
 * file.
 *
 * Rendering plausible rows here would be the exact failure the product exists
 * to prevent: a governed surface asserting a state it cannot evidence.
 */
export function UnbackedSurface({
  title,
  purpose,
  method,
}: {
  title: string;
  /** What this surface will show once the kernel projects it. */
  purpose: string;
  /** The SFWP method that would back this surface, in catalog notation. */
  method: string;
}) {
  const { inspectEvidence } = useEvidenceContext();
  const { contract, isLoading, error, refresh } = useServerContract();
  const standing = standingOf(method, contract);
  const presentation = STANDING_PRESENTATION[standing];

  const negotiating = isLoading && !error;

  return (
    <div className={styles.page}>
      <section
        className="governed-focus"
        data-od-id="unbacked"
        data-state={negotiating ? "pending" : presentation.variant}
      >
        <div className="focus-copy">
          <div className="title-line">
            <h1>{title}</h1>
            <GovernedStatusPill
              variant={negotiating ? "pending" : presentation.variant}
              label={negotiating ? "Negotiating catalog" : presentation.label}
              className="status-pill"
            />
          </div>
          <p>{purpose}</p>
          <div className="focus-meta">
            <span className="machine-value">requires {method}</span>
            {contract ? (
              <button
                type="button"
                className="freshness-badge"
                onClick={() =>
                  inspectEvidence({
                    id: `contract.${method}`,
                    kind: "method_catalog",
                    disclosureStatus: "permitted",
                    rawPayload: JSON.stringify(
                      {
                        method,
                        standing,
                        protocol_version: contract.protocolVersion,
                        server_protocol_version: contract.serverProtocolVersion,
                        implemented_methods: [...contract.implemented].sort(),
                      },
                      null,
                      2,
                    ),
                  })
                }
              >
                <span className="state-dot" />
                Negotiated from system.hello
              </button>
            ) : null}
          </div>
        </div>
        <div className="focus-actions">
          <button className="button" type="button" onClick={refresh}>
            Re-negotiate
          </button>
        </div>
      </section>

      <div className="content-grid">
        <div className="content-primary">
          <section className="panel" data-od-id="unbacked-detail">
            <div className="section-header">
              <div>
                <p className="section-kicker">Why this surface is empty</p>
                <h2>Nothing here can be evidenced yet</h2>
              </div>
            </div>

            {error ? (
              <p className="operational-copy" role="alert">
                The method catalog could not be negotiated: {error.message}. Until the
                kernel answers, this surface can make no claim about {method} in either
                direction.
              </p>
            ) : (
              <p className="operational-copy">{presentation.explanation}</p>
            )}

            <p className="operational-copy">
              This surface shows no rows rather than placeholder rows. A governed
              surface displaying illustrative data would be indistinguishable from one
              displaying real data, and every state it showed would be a claim the
              system cannot support.
            </p>

            <dl className="detail-grid">
              <div>
                <dt>Required method</dt>
                <dd className="machine-value">{method}</dd>
              </div>
              <div>
                <dt>Interaction class</dt>
                <dd className="machine-value">
                  {contract?.classes.get(method) ?? "not described"}
                </dd>
              </div>
              <div>
                <dt>Catalog source</dt>
                <dd className="machine-value">
                  {contract ? `SFWP v${contract.serverProtocolVersion}` : "unavailable"}
                </dd>
              </div>
            </dl>
          </section>
        </div>

        <aside className="attention-rail" aria-label={`${title} attention`}>
          <section className="panel" data-od-id="unbacked-consequence">
            <div className="section-header">
              <div>
                <p className="section-kicker">Consequence</p>
                <h2>No authority here</h2>
              </div>
            </div>
            <p>
              Because this surface holds no projection, it grants no authority and
              blocks none. Work that depends on it is neither approved nor denied — it
              is simply not yet inspectable from here.
            </p>
            <p className="operational-copy">
              Nothing is broken, and no action is pending on you.
            </p>
          </section>
        </aside>
      </div>
    </div>
  );
}
