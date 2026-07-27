import { Link } from "@tanstack/react-router";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import type { AssetKind, AssetRow } from "@sea-forge/contracts";
import { useAssets } from "../hooks/useAssets";
import { ASSET_STANDING_VARIANT, humanize } from "./standing";
import styles from "./RunRecordPage.module.css";

/**
 * The governed asset catalog (epic journey 4): what this cell can actually
 * operate with, and on whose evidence.
 *
 * Three tables, not one. A single merged table would put `materialized`,
 * `declared`, and `active` in one column and invite reading them as rungs of one
 * ladder — but they belong to three separate kernel vocabularies with three
 * separate meanings (see `sfwp::assets`). Splitting by kind is what keeps the
 * standing column readable as "this kind's own word" rather than as a
 * cross-kind ranking the kernel never asserted.
 *
 * Standing and blocking are also separate columns, for the same reason they are
 * separate fields on the wire: an extension can be `active` and still refused
 * because its trust level is quarantined, and an endpoint can be `declared` —
 * proven nothing — while being perfectly lawful to probe. Neither fact implies
 * the other.
 */

interface KindSection {
  kind: AssetKind;
  heading: string;
  /** What the standing column means for *this* kind. */
  standingLede: string;
  emptyLede: string;
  nameHeading: string;
}

const SECTIONS: readonly KindSection[] = [
  {
    kind: "plan_template",
    heading: "Plan templates",
    standingLede:
      "A template is materialized on disk or it is not — that is the only fact a template has. Materialization says nothing about whether a case built from it will be authorized.",
    emptyLede:
      "No template has been materialized in this cell. That is a fact about this cell, not a failure to read it.",
    nameHeading: "Template",
  },
  {
    kind: "agent_endpoint",
    heading: "Agent endpoints",
    standingLede:
      "Declared, probed, then demonstrated. The kernel refuses a status asserted in configuration, so this ladder is only ever climbed by a settled probe — and the most recent probe decides, so an endpoint that has started failing does not keep an old demonstration.",
    emptyLede:
      "No agent endpoint is configured in this cell. Delegation has nowhere to go until one is.",
    nameHeading: "Endpoint",
  },
  {
    kind: "extension",
    heading: "Extensions",
    standingLede:
      "The extension registry's own status word. Trust level is a separate condition: a quarantined extension stays listed and stays refused.",
    emptyLede: "No extension is registered in this cell.",
    nameHeading: "Extension",
  },
];

/**
 * A `run:<id>` evidence ref resolves to the run record. Every other ref is a
 * ledger entry id with no view of its own yet, so it renders as the opaque
 * identifier it is rather than as a link that would go nowhere.
 */
function EvidenceRef({ reference }: { reference: string }) {
  const runId = reference.startsWith("run:") ? reference.slice("run:".length) : undefined;
  if (!runId) return <code>{reference}</code>;
  return (
    <Link to="/runs/$runId" params={{ runId }}>
      <code>{reference}</code>
    </Link>
  );
}

function AssetTable({ section, assets }: { section: KindSection; assets: AssetRow[] }) {
  return (
    <section className={styles.section} aria-label={section.heading}>
      <h2>{section.heading}</h2>
      <p className={styles.sectionLede}>{section.standingLede}</p>

      {assets.length === 0 ? (
        <p className={styles.muted}>{section.emptyLede}</p>
      ) : (
        <table className={styles.table}>
          <thead>
            <tr>
              <th scope="col">{section.nameHeading}</th>
              <th scope="col">Version</th>
              <th scope="col">Standing</th>
              <th scope="col">Blocking</th>
              <th scope="col">Identity</th>
              <th scope="col">Evidence</th>
            </tr>
          </thead>
          <tbody>
            {assets.map((asset) => (
              <tr key={asset.asset_id}>
                <th scope="row">
                  <code>{asset.name}</code>
                </th>
                <td>{asset.version ?? <span className={styles.muted}>not versioned</span>}</td>
                <td>
                  <GovernedStatusPill
                    variant={ASSET_STANDING_VARIANT[asset.standing] ?? "unknown"}
                    label={humanize(asset.standing)}
                    className="status-pill"
                  />
                </td>
                <td>
                  {asset.blocking_reason ? (
                    <>
                      <GovernedStatusPill
                        variant="blocked"
                        label="Blocked"
                        className="status-pill"
                      />{" "}
                      {asset.blocking_reason}
                    </>
                  ) : (
                    // Not blocked is not the same as proven. Saying "nothing
                    // blocks it" keeps this cell from reading as a success
                    // claim about the asset itself.
                    <span className={styles.muted}>nothing blocks it</span>
                  )}
                </td>
                <td>
                  {asset.identity_digest ? (
                    <code>{asset.identity_digest}</code>
                  ) : (
                    <span className={styles.muted}>not computed</span>
                  )}
                </td>
                <td>
                  {asset.evidence_refs?.length ? (
                    asset.evidence_refs.map((reference) => (
                      <div key={reference}>
                        <EvidenceRef reference={reference} />
                      </div>
                    ))
                  ) : (
                    <span className={styles.muted}>none recorded</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

export function AssetCatalogPage() {
  const { assets, unreadable, isLoading, isFetching, error, refresh } = useAssets();

  return (
    <div className={styles.page} data-od-id="asset-catalog">
      <header className={styles.header}>
        <h1>Asset catalog</h1>
        <p className={styles.lede}>
          Every operating asset this cell holds, with the standing its own kind's
          vocabulary gives it and the records that put it there. Declared, installed, and
          demonstrated are different claims, and this view never merges them.
        </p>
        <p className={styles.muted}>
          Read on demand. No durable event kind announces a template, endpoint, or
          extension change, so this catalog is not live — re-read it after probing an
          endpoint or installing an extension.{" "}
          <button type="button" className="button" onClick={refresh} disabled={isFetching}>
            {isFetching ? "Re-reading…" : "Re-read catalog"}
          </button>
        </p>
      </header>

      {error && (
        <div role="alert" className={styles.alert}>
          The asset catalog could not be read: {error.message}
        </div>
      )}

      {unreadable.length > 0 && (
        <div role="alert" className={styles.alert}>
          {unreadable.length} asset source(s) exist but could not be read:{" "}
          {unreadable.join(", ")}. These are not absent assets — they are unreadable
          ones, which is an integrity signal worth investigating.
        </div>
      )}

      {isLoading && <p className={styles.muted}>Reading the asset catalog…</p>}

      {!isLoading &&
        !error &&
        SECTIONS.map((section) => (
          <AssetTable
            key={section.kind}
            section={section}
            assets={assets.filter((asset) => asset.kind === section.kind)}
          />
        ))}
    </div>
  );
}
