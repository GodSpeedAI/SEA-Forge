import { useMemo, useState } from "react";
import { Link } from "@tanstack/react-router";
import { GovernedStatusPill } from "@sea-forge/ui-components";
import type { JobContractPreview, ResolvedValue } from "@sea-forge/contracts";
import { useAssets } from "../hooks/useAssets";
import {
  useDelegationPreview,
  type DelegationPreviewRequest,
} from "../hooks/useDelegationPreview";
import { ASSET_STANDING_VARIANT, VALUE_SOURCE_LABEL, humanize } from "./standing";
import { DelegationRoster } from "./DelegationRoster";
import styles from "./RunRecordPage.module.css";

/**
 * Configure an agent task (epic 9.4): choose an eligible endpoint and inspect
 * the complete job contract a delegation would be governed by — before
 * committing to one.
 *
 * # Why this surface is inspect-only
 *
 * There is no "Run delegation" button here, and its absence is the design, not
 * an unfinished edge. `delegate` writes an intent, a plan, criteria, and an
 * authority decision; it is a protected command and belongs behind
 * `ProtectedActionButton` with its own preconditions. What was missing — and
 * what this page supplies — is the step before that: seeing the contract at all
 * without running one to find out.
 *
 * # Eligible is not authorized
 *
 * `eligible` means only that no precondition known at preview time is unmet.
 * The authority decision happens inside `delegate`, against a committed plan,
 * and is written to the ledger. This page therefore names the authority the
 * delegation will need and states plainly that no decision exists — a preview
 * that displayed a verdict would be showing a grant with no record behind it.
 *
 * # Why the contract is re-read on demand rather than as you type
 *
 * A job contract is only meaningful for one exact request, and `contract_digest`
 * identifies that request. So the form holds a draft, an explicit read commits
 * it, and editing afterwards marks the displayed contract as no longer
 * describing what is in the form. Live-previewing every keystroke would produce
 * a stream of contracts none of which the operator actually chose.
 */

/** A resolved value with the provenance that makes it readable. */
function Resolved({ value }: { value: ResolvedValue }) {
  return (
    <>
      <code>{value.value}</code>{" "}
      <span className={styles.muted}>
        ({VALUE_SOURCE_LABEL[value.source] ?? humanize(value.source)})
      </span>
    </>
  );
}

function ContractTable({ contract }: { contract: JobContractPreview }) {
  const rows: [string, React.ReactNode][] = [
    ["Provider", <code key="p">{contract.provider_kind}</code>],
    ["Model", <Resolved key="m" value={contract.model} />],
    ["Transcript retention", <Resolved key="r" value={contract.transcript_retention} />],
    ["Turn cap", <code key="t">{contract.max_turns}</code>],
    [
      "Token budget",
      contract.token_budget != null ? (
        <code key="tb">{contract.token_budget}</code>
      ) : (
        // Absent is not zero. A zero budget would be an instruction to spend
        // nothing; an absent one means the turn cap is the only bound.
        <span key="tb" className={styles.muted}>
          no token cap — bounded by the turn cap alone
        </span>
      ),
    ],
    ["Timeout", <code key="to">{contract.timeout_secs}s</code>],
    [
      "Instruction packet",
      <span key="i">
        <code>{contract.instruction_bytes}</code> of{" "}
        <code>{contract.max_request_bytes}</code> permitted bytes ·{" "}
        <code>{contract.instruction_sha256}</code>
      </span>,
    ],
    ["Response cap", <code key="rc">{contract.max_response_bytes} bytes</code>],
    ["Endpoint identity", <code key="ed">{contract.endpoint_digest}</code>],
    ["Contract identity", <code key="cd">{contract.contract_digest}</code>],
  ];

  return (
    <table className={styles.table}>
      <tbody>
        {rows.map(([label, value]) => (
          <tr key={label}>
            <th scope="row">{label}</th>
            <td>{value}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export function DelegationWorkbench() {
  const { assets, isLoading: endpointsLoading, error: endpointsError } = useAssets();
  const endpoints = useMemo(
    () => assets.filter((asset) => asset.kind === "agent_endpoint"),
    [assets],
  );

  const [endpoint, setEndpoint] = useState("");
  const [instruction, setInstruction] = useState("");
  const [maxTurns, setMaxTurns] = useState("4");
  const [model, setModel] = useState("");
  const [tokenBudget, setTokenBudget] = useState("");
  const [committed, setCommitted] = useState<DelegationPreviewRequest | null>(null);

  const draft: DelegationPreviewRequest = {
    endpoint,
    instruction,
    max_turns: Number.parseInt(maxTurns, 10) || 0,
    ...(model ? { model } : {}),
    ...(tokenBudget ? { token_budget: Number.parseInt(tokenBudget, 10) || 0 } : {}),
  };
  const { preview, isLoading, isFetching, error } = useDelegationPreview(committed);
  const stale =
    committed !== null && JSON.stringify(draft) !== JSON.stringify(committed);

  return (
    <div className={styles.page} data-od-id="delegation-workbench">
      <header className={styles.header}>
        <h1>Configure an agent task</h1>
        <p className={styles.lede}>
          The complete job contract a delegation would run under — which endpoint, which
          model, which caps, which transcript retention, and which authority it will have
          to obtain. Reading it commits nothing: no case, no run, no ledger entry, and no
          authority decision.
        </p>
      </header>

      {endpointsError && (
        <div role="alert" className={styles.alert}>
          The endpoint catalog could not be read: {endpointsError.message}
        </div>
      )}

      <section className={styles.section} aria-label="Delegation request">
        <h2>Delegation request</h2>
        <p className={styles.sectionLede}>
          Exactly the inputs the delegation command accepts, and no others. Anything this
          form could set that <code>delegate</code> cannot would describe a delegation
          nobody can run.
        </p>

        <p>
          <label htmlFor="delegation-endpoint">Endpoint</label>
          <br />
          <select
            id="delegation-endpoint"
            value={endpoint}
            onChange={(event) => setEndpoint(event.target.value)}
          >
            <option value="">Choose an endpoint…</option>
            {endpoints.map((row) => (
              // Blocked endpoints stay selectable. Hiding them would report an
              // endpoint that exists and is refused as one that is absent, and
              // the reason it is refused is exactly what the operator came for.
              <option key={row.asset_id} value={row.name}>
                {row.name} — {humanize(row.standing)}
                {row.blocking_reason ? " (blocked)" : ""}
              </option>
            ))}
          </select>
          {endpointsLoading && <span className={styles.muted}> Reading endpoints…</span>}
          {!endpointsLoading && endpoints.length === 0 && (
            <span className={styles.muted}>
              {" "}
              No agent endpoint is configured in this cell. Delegation has nowhere to go
              until one is — see the <Link to="/assets">asset catalog</Link>.
            </span>
          )}
        </p>

        <p>
          <label htmlFor="delegation-instruction">Instruction</label>
          <br />
          <textarea
            id="delegation-instruction"
            rows={4}
            value={instruction}
            onChange={(event) => setInstruction(event.target.value)}
          />
        </p>

        <p>
          <label htmlFor="delegation-max-turns">Turn cap</label>{" "}
          <input
            id="delegation-max-turns"
            type="number"
            min={1}
            value={maxTurns}
            onChange={(event) => setMaxTurns(event.target.value)}
          />
        </p>

        <p>
          <label htmlFor="delegation-model">Model</label>{" "}
          <input
            id="delegation-model"
            value={model}
            placeholder="leave empty to use the endpoint's own"
            onChange={(event) => setModel(event.target.value)}
          />
        </p>

        <p>
          <label htmlFor="delegation-token-budget">Token budget</label>{" "}
          <input
            id="delegation-token-budget"
            type="number"
            min={1}
            value={tokenBudget}
            placeholder="leave empty for no token cap"
            onChange={(event) => setTokenBudget(event.target.value)}
          />
        </p>

        <button
          type="button"
          className="button"
          disabled={!endpoint || isFetching}
          onClick={() => setCommitted(draft)}
        >
          {isFetching ? "Reading contract…" : "Inspect job contract"}
        </button>
      </section>

      {error && (
        <div role="alert" className={styles.alert}>
          The job contract could not be read: {error.message}
        </div>
      )}

      {isLoading && <p className={styles.muted}>Reading the job contract…</p>}

      {preview && !error && (
        <section className={styles.section} aria-label="Job contract">
          <h2>Job contract</h2>

          {stale && (
            <p role="status" className={styles.muted}>
              The request above has changed since this contract was read. What is shown
              describes the earlier request, not the current form.
            </p>
          )}

          <p>
            <GovernedStatusPill
              variant={preview.eligible ? "ready" : "blocked"}
              label={preview.eligible ? "Nothing blocks it" : "Blocked"}
              className="status-pill"
            />{" "}
            {preview.standing ? (
              <GovernedStatusPill
                variant={ASSET_STANDING_VARIANT[preview.standing] ?? "unknown"}
                label={humanize(preview.standing)}
                className="status-pill"
              />
            ) : (
              <span className={styles.muted}>
                This endpoint is not configured in this cell, so it has no standing.
              </span>
            )}
          </p>

          {preview.blocking_reasons?.length ? (
            <ul>
              {preview.blocking_reasons.map((reason) => (
                <li key={reason}>{reason}</li>
              ))}
            </ul>
          ) : (
            // "Nothing blocks it" is the honest ceiling of what a preview knows.
            // Saying "ready to run" would promise the authority decision that
            // has not been made.
            <p className={styles.muted}>
              Nothing known at preview time blocks this delegation. That is not the same
              as authorized.
            </p>
          )}

          {preview.contract ? (
            <>
              <ContractTable contract={preview.contract} />
              <p className={styles.muted}>
                Committing this delegation will submit a{" "}
                <code>{preview.contract.required_authority}</code> authority action for
                evaluation. No decision has been made — naming the gate is not passing it,
                and no verdict exists until the delegation is committed and the decision
                is written to the ledger.
              </p>
            </>
          ) : (
            <p className={styles.muted}>
              No contract can be built for this endpoint, because the values a contract is
              made of are the ones that did not resolve.
            </p>
          )}

          {preview.evidence_refs?.length ? (
            <p>
              Standing evidence:{" "}
              {preview.evidence_refs.map((reference, index) => (
                <span key={reference}>
                  {index > 0 && ", "}
                  {reference.startsWith("run:") ? (
                    <Link to="/runs/$runId" params={{ runId: reference.slice(4) }}>
                      <code>{reference}</code>
                    </Link>
                  ) : (
                    <code>{reference}</code>
                  )}
                </span>
              ))}
            </p>
          ) : null}
        </section>
      )}

      <DelegationRoster />
    </div>
  );
}
