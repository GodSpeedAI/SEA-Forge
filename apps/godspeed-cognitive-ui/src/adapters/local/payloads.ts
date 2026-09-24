import type { ArtifactPayload } from '../../ports/contract'

// Artifact payload data for local adapter.
// Each payload has evidence_id keyed by 'evi-' prefix.

const sha256 = (str: string): string => {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    const charCode = str.charCodeAt(i)
    hash = ((hash << 5) - hash) + charCode
    hash |= 0 // Convert to 32bit integer
  }
  const hex = Math.abs(hash).toString(16)
  return 'sha256:' + hex.padStart(64, '0').slice(-64)
}

const diffContent = (_description: string, files: { path: string; lines: string[] }[]): string => {
  let result = `diff --git a/northstar b/northstar
index 0000000..1111111 100644
--- a/northstar
+++ b/northstar
${files
  .map((f) => {
    const added = f.lines.filter((l) => l.startsWith('+')).length
    const removed = f.lines.filter((l) => l.startsWith('-')).length
    return `@@ -1,${removed} +1,${added} @@ ${f.path}
${f.lines.join('\n')}`
  })
  .join('\n')}
`
  return result
}

export const PAYLOADS: Record<string, ArtifactPayload> = {
  'evi-diff-491': {
    evidence_id: 'evi-diff-491',
    name: 'PR #491 · Add secondary coverage guard',
    digest: sha256('PR-491-coverage-guard'),
    content_type: 'text/x-diff',
    content: diffContent('Secondary coverage validation', [
      {
        path: 'src/coverage/eligibility.ts',
        lines: [
          ' const primaryEligible = await checkPrimaryCoverage(',
          ' memberId,',
          ' );',
          '-if (!primaryEligible) {',
          '-return { eligible: false, reason: "Primary not eligible" };',
          '-}',
          '+// Also validate secondary coverage if present',
          '+if (member.secondaryCoverage) {',
          '+const secondaryEligible = await checkSecondaryCoverage(',
          '+memberId,',
          '+member.secondaryCoverage',
          '+);',
          '+if (!secondaryEligible) {',
          '+return {',
          '+eligible: false,',
          '+reason: "Secondary coverage not eligible",',
          '+code: "SECONDARY_INELIGIBLE",',
          '+};',
          '+}',
          '+}',
          ' return { eligible: true, reason: "Eligible" };',
          ' }',
        ],
      },
    ]),
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-secondary',
      invocation_id: 'inv-491-review',
      run_id: 'run-491-review-1',
      pr_number: 491,
    },
  },

  'evi-record': {
    evidence_id: 'evi-record',
    name: 'Customer record EHR-77392',
    digest: sha256('EHR-77392-record'),
    content_type: 'text/markdown',
    content: `# Customer Record EHR-77392

## Member
ID: M-20930144, Acme Health Plan, enrolled 2024-03-15

## Primary Coverage
Acme Health PPO active, coverage effective through 2026-12-31

## Secondary Coverage
None on file at time of claim (09:12, Sep 16)

## Claim
#CLM-20931 rejected — secondary payer required by policy

## Source
EHR export, retrieved Sep 18, 2026
`,
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-record',
      invocation_id: 'inv-record-ehr77392',
      run_id: 'run-ehr-export-sep18',
    },
  },

  'evi-assumption': {
    evidence_id: 'evi-assumption',
    name: 'Initial assumption: Primary-only scope',
    digest: sha256('assumption-v0.1'),
    content_type: 'text/markdown',
    content: `# Initial Assumption

## Scope
The pilot scope assumed members carry primary coverage only.

## Eligibility Logic
Current eligibility check validates primary insurance only.

## Next Steps
Revisit scope if secondary payers appear in real-world workflow.
`,
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-assumption',
      invocation_id: 'inv-assumption-draft',
      run_id: 'run-assumption-v0.1',
    },
  },

  'evi-checks-table': {
    evidence_id: 'evi-checks-table',
    name: 'Nightly checks: Coverage validation suite',
    digest: sha256('checks-table-sep19'),
    content_type: 'application/json',
    content: JSON.stringify({
      schema: 'table',
      columns: ['Test Path', 'Result', 'Duration (s)'],
      rows: [
        ['primary-only', 'pass', '0.8'],
        ['secondary-present', 'pass', '1.2'],
        ['secondary-missing', 'fail', '0.9'],
        ['dual-payer', 'pass', '1.5'],
        ['medicare-secondary', 'pass', '2.1'],
        ['no-coverage', 'fail', '0.7'],
        ['expired-primary', 'fail', '0.6'],
        ['under-investigation', 'pass', '1.3'],
        ['pending-coordination', 'pass', '2.0'],
        ['edge-case-carve-out', 'pass', '1.8'],
      ],
      caption: 'Nightly checks run against coverage validation logic',
    }),
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-impl',
      invocation_id: 'inv-checks-nightly',
      run_id: 'run-checks-sep19-0300',
    },
  },

  'evi-rejections-chart': {
    evidence_id: 'evi-rejections-chart',
    name: 'Claim rejections by day',
    digest: sha256('rejections-chart-sep19'),
    content_type: 'application/json',
    content: JSON.stringify({
      schema: 'chart',
      unit: 'claims',
      series: [
        { label: 'Sep 13', value: 2 },
        { label: 'Sep 14', value: 3 },
        { label: 'Sep 15', value: 4 },
        { label: 'Sep 16', value: 12 },
        { label: 'Sep 17', value: 8 },
        { label: 'Sep 18', value: 6 },
        { label: 'Sep 19', value: 3 },
      ],
      caption: 'Rejected claims by day, showing spike around secondary coverage gap discovery',
    }),
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-evidence',
      invocation_id: 'inv-rejections-daily',
      run_id: 'run-analytics-sep19',
    },
  },

  'evi-impl-graph': {
    evidence_id: 'evi-impl-graph',
    name: 'Northstar dependency graph',
    digest: sha256('northstar-dependency-graph'),
    content_type: 'application/json',
    content: JSON.stringify({
      schema: 'graph',
      nodes: [
        { object_id: 'ns-goal', label: 'Goal', kind: 'work_item' },
        { object_id: 'ns-workflow', label: 'Customer Workflow', kind: 'work_item' },
        { object_id: 'ns-assumption', label: 'Implementation Assumption', kind: 'work_item' },
        { object_id: 'ns-record', label: 'Customer Record', kind: 'evidence_record' },
        { object_id: 'ns-secondary', label: 'Secondary Coverage', kind: 'work_item' },
        { object_id: 'ns-pr491', label: 'PR #491', kind: 'work_item' },
        { object_id: 'ns-impl', label: 'Implementation', kind: 'work_item' },
        { object_id: 'ns-release', label: 'Release', kind: 'work_item' },
        { object_id: 'ns-failure', label: 'Observed Failure', kind: 'execution_trace' },
      ],
      edges: [
        { from: 'ns-workflow', to: 'ns-assumption', label: 'Assumes' },
        { from: 'ns-assumption', to: 'ns-record', label: 'Leads to' },
        { from: 'ns-record', to: 'ns-secondary', label: 'Triggers' },
        { from: 'ns-secondary', to: 'ns-failure', label: 'Results in' },
        { from: 'ns-impl', to: 'ns-secondary', label: 'Depends on' },
        { from: 'ns-release', to: 'ns-impl', label: 'Depends on' },
      ],
    }),
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-impl',
      invocation_id: 'inv-graph-render',
      run_id: 'run-graph-sep19',
    },
  },

  'evi-coverage-trace': {
    evidence_id: 'evi-coverage-trace',
    name: 'Reality trace: Secondary coverage validation',
    digest: sha256('coverage-trace-analysis'),
    content_type: 'application/json',
    content: JSON.stringify({
      schema: 'realitytrace',
      question: {
        question_id: 'q-secondary-coverage',
        predicate: 'ASSERT_SECONDARY_COVERAGE_VALIDATED',
        target_entity: 'services/eligibility/coverage.ts',
      },
      claim: {
        proposition: 'Every claim with secondary coverage is validated before submission.',
        asserted_by: 'implementation-plan',
      },
      evaluation: {
        discrepancy_count: 2,
        discrepancies: [
          {
            attribution_locus: 'Locus2_Configuration',
            declared_state: 'secondary coverage validated before submit',
            observed_state: 'validation skipped when primary payer present',
          },
          {
            attribution_locus: 'Locus5_ObservedOutcome',
            declared_state: 'claim accepted',
            observed_state: 'claim rejected: coordination of benefits missing',
          },
        ],
      },
    }),
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-secondary',
      invocation_id: 'inv-trace-analysis',
      run_id: 'run-trace-sep18',
    },
  },

  'evi-case-timeline': {
    evidence_id: 'evi-case-timeline',
    name: 'Case timeline: Northstar pilot progression',
    digest: sha256('northstar-timeline-sep19'),
    content_type: 'application/json',
    content: JSON.stringify({
      schema: 'timeline',
      events: [
        { at: '2026-09-17T09:00:00Z', label: 'Sep 17', summary: 'Pilot scoped', cursor: '1.0000000001' },
        { at: '2026-09-17T14:40:00Z', label: 'Sep 17', summary: 'Before secondary coverage was identified', cursor: '1.0000000002' },
        { at: '2026-09-18T09:15:00Z', label: 'Sep 18', summary: 'Customer record examined', cursor: '1.0000000003' },
        { at: '2026-09-18T16:30:00Z', label: 'Sep 18', summary: 'Coverage gap confirmed', cursor: '1.0000000004' },
        { at: '2026-09-19T08:05:00Z', label: 'Sep 19', summary: 'PR #491 opened', cursor: '1.0000000005' },
        { at: '2026-09-19T10:24:00Z', label: 'Now', summary: 'Latest', cursor: '1.0000000006' },
      ],
    }),
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'northstar',
      invocation_id: 'inv-timeline-build',
      run_id: 'run-timeline-sep19',
    },
  },

  'evi-release-log': {
    evidence_id: 'evi-release-log',
    name: 'Release staging log',
    digest: sha256('release-log-sep19'),
    content_type: 'text/plain',
    content: `2026-09-19T10:15:00Z [INFO] Starting v0.3.0 staging verification
2026-09-19T10:15:45Z [INFO] Checking artifact signatures... 12 artifacts verified
2026-09-19T10:16:12Z [INFO] Running pre-release tests... 487 passed, 2 skipped
2026-09-19T10:16:58Z [INFO] Validating secondary coverage guard activation
2026-09-19T10:17:23Z [INFO] Secondary coverage validation: READY
2026-09-19T10:17:45Z [INFO] Checking feature flags... all ENABLED
2026-09-19T10:18:02Z [INFO] Staging environment health check: HEALTHY
2026-09-19T10:18:30Z [INFO] Load test 1: 100 RPS - PASS (p99: 247ms)
2026-09-19T10:19:15Z [INFO] Load test 2: 250 RPS - PASS (p99: 312ms)
2026-09-19T10:20:01Z [INFO] Canary deployment prepared for 5% traffic
2026-09-19T10:24:00Z [INFO] Release v0.3.0 ready for approval
`,
    provenance: {
      case_id: 'case-northstar',
      plan_item_id: 'ns-release',
      invocation_id: 'inv-release-staging',
      run_id: 'run-release-v0.3.0',
    },
  },
}

export const ARTIFACT_TITLES: Record<string, string> = {
  'evi-diff-491': 'PR #491 · Add secondary coverage guard',
  'evi-record': 'Customer record EHR-77392',
  'evi-assumption': 'Initial assumption: Primary-only scope',
  'evi-checks-table': 'Nightly checks: Coverage validation suite',
  'evi-rejections-chart': 'Claim rejections by day',
  'evi-impl-graph': 'Northstar dependency graph',
  'evi-coverage-trace': 'Reality trace: Secondary coverage validation',
  'evi-case-timeline': 'Case timeline: Northstar pilot progression',
  'evi-release-log': 'Release staging log',
}
