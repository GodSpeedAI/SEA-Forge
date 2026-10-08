import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { useMachine } from "@xstate/react";
import { useNavigate } from "@tanstack/react-router";
import { ProtectedActionButton, GovernedStatusPill } from "@sea-forge/ui-components";
import { useCaseEntryOptions } from "../hooks/useCaseEntryOptions";
import { evaluateProtectedAction } from "../guards/protectedAction";
import { useIdentity } from "../hooks/useIdentity";
import { caseAuthoringMachine } from "../machines/caseAuthoringMachine";
import styles from "./CaseCreationWorkbench.module.css";

type ParamValues = Record<string, string>;

/**
 * Case authoring: draft -> validate -> preflight -> commit (Task 6).
 * Template selection and parameter values are a local, reversible draft
 * (`implementation-workflow.md`'s draft/canonical table) — nothing here
 * touches `.sea-forge/`; the draft becomes real only through
 * `case.preflight` -> `case.commit`. Draft values are read from the form
 * (`getValues()`) only at the moment `PREFLIGHT` is sent — never synced into
 * the machine continuously, which would re-trigger on every keystroke.
 */
export function CaseCreationWorkbench() {
  const { query: entryOptions } = useCaseEntryOptions();
  const { identity } = useIdentity();
  const navigate = useNavigate();
  const [snapshot, send] = useMachine(caseAuthoringMachine);

  const templates = entryOptions.data?.templates ?? [];
  const [templateRef, setTemplateRef] = useState<string>("");
  const selectedTemplate = templates.find((t) => t.template_ref === templateRef);

  const { register, getValues, reset } = useForm<ParamValues>({ defaultValues: {} });

  // Selecting a template resets the param form to that template's defaults.
  useEffect(() => {
    if (!selectedTemplate) return;
    const defaults: ParamValues = {};
    for (const [name, def] of Object.entries(selectedTemplate.parameters)) {
      defaults[name] = def.default ?? "";
    }
    reset(defaults);
  }, [selectedTemplate, reset]);

  const state = snapshot.value as string;
  const preflight = snapshot.context.preflight;
  const canPreflight = templateRef.length > 0 && (state === "draft" || state === "rejected_as_stale");
  const commitAction = evaluateProtectedAction(identity, undefined, {
    method: "case.commit",
    actionLabel: "case",
    requiresReadiness: false,
  });
  const canCommit = state === "preflight_ok" && commitAction.isAllowed;

  function handlePreflight() {
    if (state === "rejected_as_stale") {
      send({ type: "RETRY" });
    } else {
      send({ type: "PREFLIGHT", draft: { templateRef, params: getValues() } });
    }
  }

  function handleCommit() {
    if (!commitAction.isAllowed) return;
    send({ type: "COMMIT" });
  }

  useEffect(() => {
    if (state === "committed" && snapshot.context.commitResult) {
      void navigate({ to: "/cases" });
    }
  }, [state, snapshot.context.commitResult, navigate]);

  return (
    <div className={`${styles.page} case-creation-workbench`} data-testid="case-creation-workbench">
      <section className="governed-focus" aria-labelledby="case-creation-title">
        <div className="focus-copy">
          <h1 id="case-creation-title">Create case</h1>
          <p>Select a template, fill its parameters, preflight the draft, then commit.</p>
        </div>
      </section>

      <div className="content-grid">
        <div className="content-primary">
          <section className="panel" aria-labelledby="template-title">
            <div className="section-header">
              <h2 id="template-title">Template</h2>
            </div>
            {entryOptions.isFetching && <p>Loading templates.</p>}
            {!entryOptions.isFetching && templates.length === 0 && (
              <p>No templates are available in this build.</p>
            )}
            <div className={styles.templateList} role="radiogroup" aria-label="Case template">
              {templates.map((template) => (
                <button
                  key={template.template_ref}
                  type="button"
                  role="radio"
                  aria-checked={template.template_ref === templateRef}
                  className={styles.templateOption}
                  onClick={() => setTemplateRef(template.template_ref)}
                >
                  <strong>{template.template_ref}</strong>
                  <small>{template.description || "No description"}</small>
                </button>
              ))}
            </div>
          </section>

          {selectedTemplate && (
            <section className="panel" aria-labelledby="params-title">
              <div className="section-header">
                <h2 id="params-title">Parameters</h2>
              </div>
              <form className={styles.paramGrid} onSubmit={(e) => e.preventDefault()}>
                {Object.entries(selectedTemplate.parameters).map(([name, def]) => (
                  <label key={name} className={styles.paramField}>
                    <span>
                      {name}
                      {def.required ? " (required)" : ""}
                    </span>
                    <input
                      type="text"
                      {...register(name, { required: def.required })}
                      aria-required={def.required}
                    />
                  </label>
                ))}
                {Object.keys(selectedTemplate.parameters).length === 0 && (
                  <p>This template takes no parameters.</p>
                )}
              </form>
            </section>
          )}

          <section className="panel" aria-labelledby="preflight-title">
            <div className="section-header">
              <h2 id="preflight-title">Preflight</h2>
              <GovernedStatusPill
                variant={
                  preflight?.ok
                    ? "ready"
                    : state === "rejected_as_stale"
                      ? "blocked"
                      : "unknown"
                }
                label={
                  preflight?.ok
                    ? "Preflight passed"
                    : state === "rejected_as_stale"
                      ? "Rejected as stale"
                      : "Not yet run"
                }
                className="status-pill"
              />
            </div>
            <button
              className="button button--secondary"
              type="button"
              disabled={!canPreflight}
              onClick={handlePreflight}
            >
              {state === "rejected_as_stale" ? "Re-run preflight" : "Run preflight"}
            </button>
            {preflight && (
              <>
                {!preflight.ok && (preflight.errors?.length ?? 0) > 0 && (
                  <ul className={styles.errorList}>
                    {preflight.errors!.map((error) => (
                      <li key={error}>{error}</li>
                    ))}
                  </ul>
                )}
                {preflight.items.length > 0 && (
                  <div className={styles.itemList} role="table" aria-label="Draft plan items">
                    {preflight.items.map((item) => (
                      <div key={item.plan_item_id} role="row">
                        <strong>{item.name}</strong> <span>({item.item_kind})</span>
                      </div>
                    ))}
                  </div>
                )}
              </>
            )}
            {state === "rejected_as_stale" && (
              <p role="alert">
                The template changed since preflight. Re-run preflight to refresh before
                committing.
              </p>
            )}
            {state === "ambiguous" && (
              <div>
                <p role="alert">
                  The commit response was lost. Recover the outcome instead of resubmitting.
                </p>
                <button
                  className="button button--secondary"
                  type="button"
                  onClick={() => send({ type: "RECOVER" })}
                >
                  Recover outcome
                </button>
              </div>
            )}
          </section>
        </div>

        <aside className="attention-rail" aria-labelledby="commit-title">
          <section className="panel action-panel">
            <p className="section-kicker">Next lawful action</p>
            <h2 id="commit-title">Commit case</h2>
            <ProtectedActionButton
              label="Commit case"
              onClick={handleCommit}
              isAllowed={canCommit}
              disabledReason={
                commitAction.refusal
                  ? `${commitAction.refusal.message} ${commitAction.refusal.unchangedEffect}`
                  : canCommit
                    ? undefined
                    : "Run a passing preflight before committing"
              }
              variant="primary"
              className="lawful-action"
            />
            {commitAction.refusal && (
              <p role="alert">
                {commitAction.refusal.message} {commitAction.refusal.unchangedEffect}{" "}
                <button
                  className="button button--secondary"
                  type="button"
                  onClick={() => void navigate({ to: commitAction.refusal!.repairRoute })}
                >
                  {commitAction.refusal.repairLabel}
                </button>
              </p>
            )}
            {state === "committed" && snapshot.context.commitResult && (
              <p>Case {snapshot.context.commitResult.case_id} committed.</p>
            )}
          </section>
        </aside>
      </div>
    </div>
  );
}
