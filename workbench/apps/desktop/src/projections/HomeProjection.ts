// HomeProjection: the canonical CORE-centered orientation state. Home is not
// a destination containing CORE — Home IS the CORE-centered state: no
// secondary object owns focus, globally relevant objects arrange around CORE,
// the Composer is available. Only real, source-backed counts appear; anything
// unanswered renders as unknown.

import type { ProjectionStatus } from "./projection";

export interface HomeOrientation {
  status: ProjectionStatus;
  caseCount: number | null;
  pendingApprovals: number | null;
  unreadableCases: number | null;
  connectionLabel: string;
}

export function projectHomeOrientation(input: {
  casesLoading: boolean;
  casesError: unknown;
  caseCount: number;
  unreadable: number;
  approvalsLoading: boolean;
  approvalsError: unknown;
  approvalsUnreadable: boolean;
  pendingApprovals: number;
  connectionLabel: string;
}): HomeOrientation {
  const loading = input.casesLoading || input.approvalsLoading;
  const unavailable = Boolean(input.casesError) || Boolean(input.approvalsError);
  return {
    status: loading
      ? { state: "loading" }
      : unavailable
        ? { state: "unavailable", detail: "One or more orientation sources did not answer." }
        : { state: "live" },
    // `null` = unanswered. Never render as 0: an unread inbox and an empty
    // queue are different facts (see AppShell inbox-count lineage).
    caseCount: input.casesError ? null : input.caseCount,
    pendingApprovals:
      input.approvalsLoading || input.approvalsError || input.approvalsUnreadable
        ? null
        : input.pendingApprovals,
    unreadableCases: input.casesError ? null : input.unreadable,
    connectionLabel: input.connectionLabel,
  };
}
