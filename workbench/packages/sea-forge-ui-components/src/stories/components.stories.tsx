import React from "react";
import type { Meta, StoryObj } from "@storybook/react";

import {
  GovernedStatusPill,
  DualStateIndicator,
  SourceFreshnessBadge,
  IntegrityIndicator,
  ProtectedActionButton,
  WhyStatePanel,
  EvidenceDrawer,
  AuthorityBoundaryPanel,
  AvailabilityLadder,
} from "../index";

// 1. GovernedStatusPill Stories
const pillMeta: Meta<typeof GovernedStatusPill> = {
  title: "Semantic/GovernedStatusPill",
  component: GovernedStatusPill,
};
export default pillMeta;

export const AllPillVariants: StoryObj<typeof GovernedStatusPill> = {
  render: () => (
    <div style={{ display: "flex", flexWrap: "wrap", gap: "8px" }}>
      <GovernedStatusPill variant="ready" />
      <GovernedStatusPill variant="degraded" />
      <GovernedStatusPill variant="blocked" />
      <GovernedStatusPill variant="integrity_halted" />
      <GovernedStatusPill variant="running" />
      <GovernedStatusPill variant="finished" />
      <GovernedStatusPill variant="cancelled" />
      <GovernedStatusPill variant="failed" />
      <GovernedStatusPill variant="pending" />
      <GovernedStatusPill variant="committed" />
      <GovernedStatusPill variant="rejected" />
      <GovernedStatusPill variant="unknown" />
      <GovernedStatusPill variant="unrecognized_variant_xyz" />
    </div>
  ),
};

// 2. DualStateIndicator Story
export const DualStateSample: StoryObj<typeof DualStateIndicator> = {
  render: () => (
    <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
      <DualStateIndicator executionState="running" settlementState="pending" />
      <DualStateIndicator executionState="finished" settlementState="committed" />
      <DualStateIndicator executionState="finished" settlementState="rejected" />
    </div>
  ),
};

// 3. SourceFreshnessBadge Story
export const SourceFreshnessVariants: StoryObj<typeof SourceFreshnessBadge> = {
  render: () => (
    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
      <SourceFreshnessBadge mode="live" sourceName="Proof Engine" lastVerifiedAt="10:45:00" />
      <SourceFreshnessBadge mode="cached" sourceName="Ledger Mirror" />
      <SourceFreshnessBadge mode="stale" sourceName="SFWP Cursor" />
      <SourceFreshnessBadge mode="offline" sourceName="Peer Endpoint" />
    </div>
  ),
};

// 4. IntegrityIndicator Story
export const IntegrityVariants: StoryObj<typeof IntegrityIndicator> = {
  render: () => (
    <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
      <IntegrityIndicator status="verified" details="MMR proof root matches" />
      <IntegrityIndicator status="checking" />
      <IntegrityIndicator status="compromised" details="Hash drift detected" />
      <IntegrityIndicator status="unverified" />
    </div>
  ),
};

// 5. ProtectedActionButton Story
export const ProtectedActionStates: StoryObj<typeof ProtectedActionButton> = {
  render: () => (
    <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
      <ProtectedActionButton label="Commit Case (Allowed)" onClick={() => alert("Committed")} />
      <ProtectedActionButton
        label="Run Execution (Blocked)"
        onClick={() => {}}
        isAllowed={false}
        disabledReason="Requires sponsor standing"
      />
      <ProtectedActionButton
        label="Purge Evidence (Requires Confirm)"
        onClick={() => alert("Purged")}
        requiresConfirmation
        confirmationMessage="Purge permanent evidence record?"
      />
    </div>
  ),
};

// 6. WhyStatePanel Story
export const WhyStateSample: StoryObj<typeof WhyStatePanel> = {
  render: () => (
    <div style={{ width: 450 }}>
      <WhyStatePanel
        statusVariant="degraded"
        summary="Local execution is active, but external agent endpoint probe timed out."
        conditions={[
          { id: "c1", name: "Kernel readiness", status: "pass", explanation: "Sync kernel ready" },
          { id: "c2", name: "ACP Mediator", status: "warn", explanation: "Timeout after 5000ms" },
        ]}
        evidenceRef="ev_ulid_998877"
        onRepairAction={() => alert("Triggering re-probe...")}
        repairActionLabel="Re-probe endpoint"
      />
    </div>
  ),
};

// 7. EvidenceDrawer Story
export const EvidenceDrawerSample: StoryObj<typeof EvidenceDrawer> = {
  render: () => (
    <EvidenceDrawer
      isOpen
      onClose={() => {}}
      evidence={{
        id: "ev_ulid_11223344",
        kind: "ledger_settlement_proof",
        ledgerUlid: "01HQX789ABCD",
        digest: "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        disclosureStatus: "permitted",
        rawPayload: JSON.stringify(
          {
            statement: "Settlement verified by authority",
            evaluator: "eval_01",
            outcome: "Accepted",
          },
          null,
          2
        ),
      }}
    />
  ),
};

// 8. AuthorityBoundaryPanel Story
export const AuthorityBoundarySample: StoryObj<typeof AuthorityBoundaryPanel> = {
  render: () => (
    <div style={{ width: 500 }}>
      <AuthorityBoundaryPanel
        actorName="Operator (Main)"
        roleName="Governed Executive"
        sponsorName="Human Principal"
        policyBundleRef="policy-bundle-2026-07-v1"
        allowedBoundaries={["local-execution", "read-evidence", "commit-case", "approve-task"]}
      />
    </div>
  ),
};

// 9. AvailabilityLadder Story
export const AvailabilityLadderSample: StoryObj<typeof AvailabilityLadder> = {
  render: () => (
    <div style={{ width: 500 }}>
      <AvailabilityLadder currentLevel={2} />
    </div>
  ),
};
