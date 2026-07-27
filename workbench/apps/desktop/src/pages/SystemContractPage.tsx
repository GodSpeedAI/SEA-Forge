import { GovernedStatusPill } from "@sea-forge/ui-components";
import { useEvidenceContext } from "../shell/EvidenceContext";
import {
  CLIENT_PROTOCOL_VERSION,
  STANDING_PRESENTATION,
  SURFACED_METHODS,
  standingOf,
  useServerContract,
  type MethodStanding,
} from "../hooks/useServerContract";
import styles from "./SurfacesPages.module.css";

/**
 * What this cell can actually do, answered by the cell (epic §3.6, §16.7).
 *
 * The catalog is the kernel's own `system.hello` / `system.describe` reply, not
 * a documented aspiration. Every other surface derives its standing from the
 * same negotiation, so this page is where a disagreement between "the product
 * has a screen for it" and "the installation implements it" becomes visible
 * instead of being felt as an unexplained empty view.
 */

/** Methods the workbench knows how to reach, grouped as the operator meets them. */
const JOURNEY_GROUPS: { title: string; blurb: string; methods: string[] }[] = [
  {
    title: "Negotiation",
    blurb: "How the workbench learns what this installation supports.",
    methods: ["system.hello", "system.describe", "system.get_schema"],
  },
  {
    title: "Readiness",
    blurb: "Whether the cell is fit to accept work before any side effect.",
    methods: ["readiness.get", "readiness.check"],
  },
  {
    title: "Case authoring",
    blurb: "Composing, dry-running, and committing a governed case.",
    methods: ["case.entry_options", "case.preflight", "case.commit"],
  },
  {
    title: "Operations",
    blurb: "Durable event history and the fate of a submitted request.",
    methods: ["events.get_range", "events.subscribe", "events.unsubscribe", "request.get_status"],
  },
];

const STANDING_ORDER: MethodStanding[] = ["live", "unsurfaced", "unimplemented", "unknown"];

export function SystemContractPage() {
  const { inspectEvidence } = useEvidenceContext();
  const { contract, isLoading, error, refresh } = useServerContract();

  // Methods the kernel reports that no group above accounts for. Listing them
  // matters: a kernel that grew a verb this build does not know about is the
  // signal that the workbench is behind, and silence would hide it.
  const grouped = new Set(JOURNEY_GROUPS.flatMap((g) => g.methods));
  const ungrouped = contract ? [...contract.implemented].filter((m) => !grouped.has(m)).sort() : [];

  const counts = contract
    ? STANDING_ORDER.map((standing) => ({
        standing,
        n: [...new Set([...contract.implemented, ...SURFACED_METHODS])].filter(
          (m) => standingOf(m, contract) === standing,
        ).length,
      })).filter((c) => c.n > 0)
    : [];

  const headline = error
    ? { variant: "blocked", label: "Catalog unavailable" }
    : contract?.incompatible
      ? { variant: "blocked", label: "Protocol mismatch" }
      : isLoading
        ? { variant: "pending", label: "Negotiating" }
        : { variant: "ready", label: "Catalog negotiated" };

  return (
    <div className={styles.page}>
      <section className="governed-focus" data-od-id="system-contract" data-state={headline.variant}>
        <div className="focus-copy">
          <div className="title-line">
            <h1>Installation contract</h1>
            <GovernedStatusPill
              variant={headline.variant}
              label={headline.label}
              className="status-pill"
            />
          </div>
          <p>
            Every method this cell implements, as the cell reports it. Surfaces elsewhere in
            the workbench take their availability from this same answer, so what you see here
            is what they will honour.
          </p>
          <div className="focus-meta">
            <span className="machine-value">
              client SFWP v{CLIENT_PROTOCOL_VERSION}
              {contract ? ` · server v${contract.serverProtocolVersion}` : ""}
            </span>
            {contract ? (
              <button
                type="button"
                className="freshness-badge"
                onClick={() =>
                  inspectEvidence({
                    id: "contract.catalog",
                    kind: "method_catalog",
                    disclosureStatus: "permitted",
                    rawPayload: JSON.stringify(
                      {
                        protocol_version: contract.protocolVersion,
                        server_protocol_version: contract.serverProtocolVersion,
                        implemented_methods: [...contract.implemented].sort(),
                        classes: Object.fromEntries(contract.classes),
                      },
                      null,
                      2,
                    ),
                  })
                }
              >
                <span className="state-dot" />
                {contract.implemented.size} methods implemented
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
          {contract?.incompatible ? (
            <section className="panel" data-od-id="contract-mismatch">
              <div className="section-header">
                <div>
                  <p className="section-kicker">Compatibility</p>
                  <h2>This client cannot read this server</h2>
                </div>
              </div>
              <p className="operational-copy" role="alert">
                The server speaks SFWP v{contract.serverProtocolVersion}; this workbench is
                built against v{CLIENT_PROTOCOL_VERSION}. Field meanings are not guaranteed
                to survive a major-version difference, so no governed state is presented.
                Rendering it anyway would risk showing misread values as fact.
              </p>
            </section>
          ) : error ? (
            <section className="panel" data-od-id="contract-error">
              <div className="section-header">
                <div>
                  <p className="section-kicker">Negotiation</p>
                  <h2>The catalog could not be read</h2>
                </div>
              </div>
              <p className="operational-copy" role="alert">
                {error.message}. Until the cell answers, the workbench makes no claim about
                which methods exist — surfaces will show <em>unknown</em> rather than guess.
              </p>
            </section>
          ) : isLoading ? (
            <section className="panel" data-od-id="contract-loading">
              <p className="operational-copy">Asking the cell what it implements…</p>
            </section>
          ) : (
            <>
              {JOURNEY_GROUPS.map((group) => (
                <section className="panel" data-od-id={`contract-${group.title}`} key={group.title}>
                  <div className="section-header">
                    <div>
                      <p className="section-kicker">{group.title}</p>
                      <h2>{group.blurb}</h2>
                    </div>
                  </div>
                  <ul className="method-list">
                    {group.methods.map((method) => {
                      const standing = standingOf(method, contract);
                      const presentation = STANDING_PRESENTATION[standing];
                      return (
                        <li className="method-row" key={method} data-standing={standing}>
                          <span className="machine-value">{method}</span>
                          <span className="method-class">
                            {contract?.classes.get(method) ?? "—"}
                          </span>
                          <GovernedStatusPill
                            variant={presentation.variant}
                            label={presentation.label}
                            className="status-pill"
                          />
                        </li>
                      );
                    })}
                  </ul>
                </section>
              ))}

              {ungrouped.length > 0 ? (
                <section className="panel" data-od-id="contract-ungrouped">
                  <div className="section-header">
                    <div>
                      <p className="section-kicker">Beyond this build</p>
                      <h2>Implemented here, unknown to this workbench</h2>
                    </div>
                  </div>
                  <p className="operational-copy">
                    The cell implements these, but this build of the workbench has no view
                    that reads them. The installation is ahead of the interface.
                  </p>
                  <ul className="method-list">
                    {ungrouped.map((method) => (
                      <li className="method-row" key={method} data-standing="unsurfaced">
                        <span className="machine-value">{method}</span>
                        <span className="method-class">
                          {contract?.classes.get(method) ?? "—"}
                        </span>
                        <GovernedStatusPill
                          variant="degraded"
                          label="No view here"
                          className="status-pill"
                        />
                      </li>
                    ))}
                  </ul>
                </section>
              ) : null}
            </>
          )}
        </div>

        <aside className="attention-rail" aria-label="Contract attention">
          <section className="panel" data-od-id="contract-summary">
            <div className="section-header">
              <div>
                <p className="section-kicker">Standing</p>
                <h2>How the surface divides</h2>
              </div>
            </div>
            {counts.length > 0 ? (
              <dl className="detail-grid">
                {counts.map(({ standing, n }) => (
                  <div key={standing}>
                    <dt>{STANDING_PRESENTATION[standing].label}</dt>
                    <dd className="machine-value">{n}</dd>
                  </div>
                ))}
              </dl>
            ) : (
              <p className="operational-copy">Nothing to count until the catalog is read.</p>
            )}
            <p className="operational-copy">
              A method the cell implements is not automatically reachable from here, and a
              screen that exists is not evidence the cell supports it. The two are counted
              separately on purpose.
            </p>
          </section>
        </aside>
      </div>
    </div>
  );
}
