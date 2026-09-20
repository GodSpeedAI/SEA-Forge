/**
 * Fixture world data (T03).
 *
 * A representative casework world in the application's own projection terms: surfaces, objects with
 * kinds and salience, relationships, and per-object artifacts disclosed in bounded levels. This data
 * exists so the core can be exercised without any backend; NONE of its content or shape is known to
 * the core. A second, structurally different provider lives in `alt-provider.ts` and drives the same
 * interaction tests.
 */

export interface FixtureArtifactLevel {
  readonly level: 'minimal' | 'summary' | 'source'
  readonly mediaType: string
  readonly text: string
}

export interface FixtureArtifact {
  readonly ref: string
  readonly kind: string
  readonly title: string
  readonly boundObject: string
  readonly levels: readonly FixtureArtifactLevel[]
}

export interface FixtureWorldRevision {
  readonly cursor: number
  readonly at: string
  readonly summary: string
  readonly surfaces: readonly { id: string; label: string; objectIds: readonly string[] }[]
  readonly objects: readonly {
    id: string
    kind: string
    label: string
    position: { x: number; y: number; depth: number }
    salience: number
  }[]
  readonly relationships: readonly { from: string; to: string; kind: string }[]
}

export const harborDredgingCase: FixtureWorldRevision = {
  cursor: 1042,
  at: '2026-09-18T09:24:00Z',
  summary: 'case activated; evidence review underway',
  surfaces: [
    { id: 'surface-cases', label: 'Cases', objectIds: ['obj-case', 'obj-approval'] },
    {
      id: 'surface-evidence',
      label: 'Evidence',
      objectIds: ['obj-licence', 'obj-sounding', 'obj-impact'],
    },
    { id: 'surface-runs', label: 'Runs', objectIds: ['obj-run-a', 'obj-run-b'] },
  ],
  objects: [
    {
      id: 'obj-case',
      kind: 'case',
      label: 'Harbour dredging permit — renewal',
      position: { x: 0, y: 0, depth: 0 },
      salience: 1,
    },
    {
      id: 'obj-approval',
      kind: 'approval',
      label: 'Harbour authority sign-off',
      position: { x: 3, y: 0, depth: 1 },
      salience: 0.6,
    },
    {
      id: 'obj-licence',
      kind: 'document',
      label: 'Existing licence (2024)',
      position: { x: 0, y: 2, depth: 1 },
      salience: 0.8,
    },
    {
      id: 'obj-sounding',
      kind: 'document',
      label: 'Bathymetric survey',
      position: { x: 1.5, y: 2, depth: 2 },
      salience: 0.9,
    },
    {
      id: 'obj-impact',
      kind: 'document',
      label: 'Environmental impact statement',
      position: { x: 3, y: 2, depth: 2 },
      salience: 0.7,
    },
    {
      id: 'obj-run-a',
      kind: 'run',
      label: 'Evidence completeness check',
      position: { x: 0, y: 4, depth: 1 },
      salience: 0.5,
    },
    {
      id: 'obj-run-b',
      kind: 'run',
      label: 'Draft determination',
      position: { x: 1.5, y: 4, depth: 1 },
      salience: 0.4,
    },
  ],
  relationships: [
    { from: 'obj-case', to: 'obj-licence', kind: 'governed-by-evidence' },
    { from: 'obj-case', to: 'obj-sounding', kind: 'governed-by-evidence' },
    { from: 'obj-case', to: 'obj-impact', kind: 'governed-by-evidence' },
    { from: 'obj-case', to: 'obj-approval', kind: 'awaits' },
    { from: 'obj-run-a', to: 'obj-sounding', kind: 'observes' },
    { from: 'obj-run-b', to: 'obj-case', kind: 'produces-draft-for' },
  ],
}

export const harborDredgingHistory: readonly FixtureWorldRevision[] = [
  {
    ...harborDredgingCase,
    cursor: 997,
    at: '2026-09-11T14:02:00Z',
    summary: 'case opened; licence evidence only',
    objects: harborDredgingCase.objects.map((o) =>
      o.id === 'obj-sounding' || o.id === 'obj-run-b'
        ? { ...o, salience: Math.max(0.1, o.salience - 0.4) }
        : o,
    ),
  },
  {
    ...harborDredgingCase,
    cursor: 1020,
    at: '2026-09-15T08:47:00Z',
    summary: 'survey arrived; impact statement requested',
  },
  harborDredgingCase,
]

export const harborArtifacts: readonly FixtureArtifact[] = [
  {
    ref: 'fx-art-licence',
    kind: 'licence-document',
    title: 'Existing licence (2024)',
    boundObject: 'obj-licence',
    levels: [
      { level: 'minimal', mediaType: 'text/plain', text: 'Licence, 2024 season.' },
      {
        level: 'summary',
        mediaType: 'text/plain',
        text: 'One-page renewal licence for the 2024 dredging season; expires end of Q4.',
      },
      {
        level: 'source',
        mediaType: 'text/plain',
        text: 'LICENCE 2024 — granted to Harbour Works Ltd; permitted aggregate removal 40,000 t; depth restriction −7.2 m; three conditions on spoil disposal; renewal requires updated survey and impact statement.',
      },
    ],
  },
  {
    ref: 'fx-art-sounding',
    kind: 'survey-document',
    title: 'Bathymetric survey',
    boundObject: 'obj-sounding',
    levels: [
      { level: 'minimal', mediaType: 'text/plain', text: 'Survey, September.' },
      {
        level: 'summary',
        mediaType: 'text/plain',
        text: 'Multi-beam survey of the approach channel; confirms shoaling near berth 3.',
      },
      {
        level: 'source',
        mediaType: 'text/plain',
        text: 'SURVEY 2026-09 — 14 transects, coverage 98.2%, mean depth −6.4 m, shoal to −5.1 m at berth 3 approach; appendix lists tide corrections; signed by the surveyor of record.',
      },
    ],
  },
  {
    ref: 'fx-art-impact',
    kind: 'statement-document',
    title: 'Environmental impact statement',
    boundObject: 'obj-impact',
    levels: [
      { level: 'minimal', mediaType: 'text/plain', text: 'Impact statement, draft.' },
      {
        level: 'summary',
        mediaType: 'text/plain',
        text: 'Draft statement covering spoil plume, noise, and benthic recovery; two mitigations proposed.',
      },
      {
        level: 'source',
        mediaType: 'text/plain',
        text: 'EIS DRAFT v3 — sections: plume modelling (unsupported beyond 400 m), marine mammal watch protocol, benthic recovery 24–36 months; mitigation M1 timed extraction, M2 silt curtain at berth 3; outstanding peer review.',
      },
    ],
  },
]
