import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { validateThothAnswerView, type ThothAnswerView } from "@sea-forge/contracts";
import { describeAjv, rejectGovernedError, GovernedViewError } from "./governedQuery";
import { toError } from "./bridgeError";

/**
 * Ask the cell a grounded question (`thoth.ask`, epic journey 3).
 *
 * The kernel has answered this since M11, but it was absent from the method
 * catalog, so no client could discover it and the Thoth surface rendered a
 * written-in answer instead. That answer looked exactly like a real one and
 * cited "UX epic §4.6" as its evidence — a claim about the system sourced from
 * a design document rather than from the system.
 *
 * `thoth.ask` is a *command*, not an inspect: answering is disclosure-controlled
 * and recorded, so it carries the actor block the host attaches and it goes
 * through `sfwp_command`.
 */

/** The typed questions this cell can answer, from `parse_question_kind`. */
export const QUESTION_KINDS = [
  { kind: "ask_capability", label: "Is this capability available?" },
  { kind: "ask_operation_requirements", label: "What does this operation require?" },
  { kind: "ask_authority_requirements", label: "What authority does it need?" },
  { kind: "ask_projection_support", label: "Is this projection supported?" },
  { kind: "ask_environment_status", label: "What is this environment's status?" },
  { kind: "ask_failure_explanation", label: "Why did this fail?" },
  { kind: "ask_evidence_for_claim", label: "What evidence backs this claim?" },
  { kind: "ask_available_affordances", label: "What can I do right now?" },
  { kind: "ask_why_denied", label: "Why was this denied?" },
] as const;

export type QuestionKind = (typeof QUESTION_KINDS)[number]["kind"];

export interface AskInput {
  kind: QuestionKind;
  /** The typed subject — a concept ref or capability name, never prose. */
  subject: string;
  purpose: string;
}

/**
 * The answer with its `#[serde(default)]` collections materialized.
 *
 * Absent and empty mean the same thing for all four of these — the kernel omits
 * them when empty — so normalizing here keeps `?? []` out of the render path.
 * That matters beyond tidiness: `omitted_claim_classes` and `limitations` are
 * disclosures, and a render that silently skipped an absent one would be
 * withholding the fact that something was withheld.
 */
export type Answer = Omit<
  ThothAnswerView,
  "claims" | "omitted_claim_classes" | "limitations"
> & {
  claims: NonNullable<ThothAnswerView["claims"]>;
  omitted_claim_classes: string[];
  limitations: string[];
};

function normalize(view: ThothAnswerView): Answer {
  return {
    ...view,
    claims: view.claims ?? [],
    omitted_claim_classes: view.omitted_claim_classes ?? [],
    limitations: view.limitations ?? [],
  };
}

export function useThoth() {
  const [answer, setAnswer] = useState<Answer | undefined>();
  const [error, setError] = useState<Error | undefined>();
  const [asking, setAsking] = useState(false);

  const ask = useCallback(async (input: AskInput) => {
    setAsking(true);
    setError(undefined);
    try {
      const raw = await invoke<unknown>("sfwp_command", {
        command: {
          verb: "ask",
          kind: input.kind,
          subject: input.subject,
          purpose: input.purpose,
        },
      });
      // A denial is a governed answer with its own shape, not an error — but a
      // *refusal* (unknown kind, unverifiable identity) is the typed error
      // envelope, and that has to be told apart from an answer before
      // validation, or every refusal reads as a malformed response.
      rejectGovernedError(raw);
      if (!validateThothAnswerView(raw)) {
        throw new Error(
          `thoth.ask response failed contract validation: ${describeAjv(validateThothAnswerView.errors)}`,
        );
      }
      const normalized = normalize(raw as ThothAnswerView);
      setAnswer(normalized);
      return normalized;
    } catch (reason) {
      const failure =
        reason instanceof GovernedViewError ? reason : toError(reason);
      setError(failure);
      // The previous answer is deliberately left on screen. It was true when it
      // was given, and blanking it would lose the one thing the operator could
      // still act on.
      return undefined;
    } finally {
      setAsking(false);
    }
  }, []);

  return { ask, answer, error, asking };
}
