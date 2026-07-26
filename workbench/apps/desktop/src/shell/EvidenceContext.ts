import { createContext, useContext } from "react";
import type { EvidenceRecord } from "@sea-forge/ui-components";

export interface EvidenceContextValue {
  inspectEvidence: (evidence: EvidenceRecord) => void;
}

const EvidenceContext = createContext<EvidenceContextValue>({
  inspectEvidence: () => {},
});

export const EvidenceContextProvider = EvidenceContext.Provider;

export function useEvidenceContext(): EvidenceContextValue {
  return useContext(EvidenceContext);
}
