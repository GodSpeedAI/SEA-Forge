// Internal model of the cognitive environment: the projection of contract snapshots
// (src/ports/contract.ts → src/ports/project.ts) that layouts and renderers read.
// Spec: .agents/specs/cognitive-environment/GodSpeed_Cognitive_Environment_DESIGN.md §0.1 + Appendix A.
// Everything here is plain data. Nothing references React, the DOM or Three.js.

import type {
  ActionIntentKind,
  CausalRepresentation,
  CognitiveArtifact,
  ObjectExtensions,
  OperationalSettlement,
  TemplateEntryOption,
  ThothAnswerView,
} from '../ports/contract'

export type Id = string
export type Theme = 'light' | 'dark'

/** Semantic tone of a status line. Colour is never the only carrier: every tone also has a label. */
export type Tone = 'ok' | 'progress' | 'attention' | 'critical' | 'hypothesis' | 'muted'

export interface Status {
  label: string
  tone: Tone
}

export type ObjectKind =
  | 'core' // the Home singularity
  | 'category' // Home orbit regions: Projects, People, Knowledge...
  | 'case' // a governed world you can enter (Northstar)
  | 'facet' // a part of a case: Goal, Evidence, Implementation, Release...
  | 'item' // concrete thing: PR, record, observation, test path
  | 'person'
  | 'run' // delegated/autonomous execution

/** An action the backend lists on an object (contract ActionDescriptor). The UI never computes authority. */
export interface ObjectAction {
  id: string
  label: string
  intent: ActionIntentKind
  /** Consequential actions cross the authority boundary and open the judgment surface. */
  consequential: boolean
  variant?: 'PRIMARY' | 'SECONDARY' | 'DANGER' | 'WARNING' | 'GHOST'
  requiresJustification?: boolean
}

/** Authored position of an object inside its parent's frame (contract spatial_layout, mock px). */
export interface Slot {
  x: number
  y: number
  size: number
  label?: 'right' | 'left' | 'above'
}

export interface WorldObject {
  id: Id
  kind: ObjectKind
  parent: Id | null
  title: string
  subtitle?: string
  /** Short count/metric line, e.g. "5 tasks". Shown instead of subtitle when present at LOD 1. */
  metric?: string
  status?: Status
  /** 0..1. Drives scale, clarity and distance in layouts. */
  salience: number
  /** Visual mass multiplier for sphere size (default 1). */
  mass?: number
  /** Present but not yet real at this time (dashed ghost, e.g. "Not yet identified"). */
  ghost?: boolean
  /** Exists but has not started (rendered as a pale, inactive sphere). */
  dormant?: boolean
  actions?: ObjectAction[]
  /** Artifact ids bound to this object (reachable evidence). */
  artifacts?: Id[]
  /** Core-center residue shown when this object is the center of gravity, e.g. "1 issue needs review". */
  residue?: Status
  /** Design-time glyph drawn inside the sphere (case design mode only). */
  icon?: 'target' | 'layers' | 'people' | 'doc' | 'flag' | 'shield' | 'bolt' | 'history' | 'eye'
  /** Rim accent for design-time nodes. */
  accent?: 'orange' | 'blue' | 'green' | 'dark'
  /** Contract kind and status, kept for honest outline/inspection text. */
  contractKind?: string
  contractStatus?: string
  slot?: Slot
  causal?: CausalRepresentation
  /** Authoritative settlement reported by the backend. Completion alone never sets this. */
  settlement?: OperationalSettlement
  templateCaseId?: string
  designItems?: NonNullable<ObjectExtensions['design']>['items']
}

export type RelationKind =
  | 'contains'
  | 'depends'
  | 'evidences'
  | 'assumes'
  | 'leads-to'
  | 'triggers'
  | 'results-in'
  | 'relates'

export interface Relationship {
  id: Id
  from: Id
  to: Id
  kind: RelationKind
  label?: string
}

export interface Revision {
  id: string
  /** ISO timestamp */
  at: string
  label: string
  /** e.g. "Before secondary coverage was identified" */
  summary?: string
}

export interface WorldSnapshot {
  /** Contract cursor of this snapshot. */
  revision: string
  caseId: string
  objects: Record<Id, WorldObject>
  relationships: Relationship[]
  /** Artifact descriptors bound to objects in this snapshot, by ref. */
  artifacts: Record<string, CognitiveArtifact>
  /** Object the backend says deserves attention. */
  attention?: Id
}

export interface WorldHistory {
  /** Oldest first; the last one is "now". */
  revisions: Revision[]
  snapshots: Record<string, WorldSnapshot>
  /** Where the data came from, shown honestly in the status bar. */
  provenance: 'local-contract' | 'go'
  caseId: string
}

// ---------------------------------------------------------------------------
// Camera and layout

export interface Camera {
  /** World-space point at the focal point of the screen. */
  x: number
  y: number
  /** Screen pixels per world unit. Home default is 1. */
  zoom: number
  /** Plane tilt in radians (0 = looking straight down). Bounded, restrained 2.5D. */
  tilt: number
  /** In-screen rotation of the plane in radians. Small. */
  roll: number
}

export interface Vec3 {
  x: number
  y: number
  /** Depth: >0 toward the viewer (foreground bokeh), <0 away (background). */
  z: number
}

export type SurfaceId = 'orbital' | 'causal' | 'compare' | 'judgment'

/** Per-object result of layout(). Anything not in the map is not visible in this arrangement. */
export interface Placement {
  pos: Vec3
  /** Presentation role in this surface, e.g. "2. Key assumption". */
  role?: string
  /** 0..1, multiplies opacity; <1 means receded. */
  emphasis: number
  /** Overrides for this surface only (identity stays the same). */
  title?: string
  subtitle?: string
  status?: Status
  /** Render as the local center of gravity (singularity visual). */
  center?: boolean
  /** Node diameter in world units. Apparent size (size × projected scale) drives LOD. */
  size: number
  /** Where the label sits relative to the node. */
  labelSide?: 'right' | 'left' | 'above'
  /** Drawn as a tiny satellite dot of its parent (no label until hover). */
  satellite?: boolean
  /** Center title drawn inside the void (workbench modes, mocks 11/12). */
  labelInside?: boolean
  /** Comparison representation: this object's state on side A and side B. */
  compare?: { change: 'added' | 'removed' | 'changed' | 'same'; a?: Status; b?: Status; aTitle?: string; bTitle?: string; inside?: number; fields?: string[] }
  /** Narration annotation shown beside the node. */
  annotation?: string
  /** Narration highlight. */
  highlight?: boolean
}

export interface Orbit {
  cx: number
  cy: number
  rx: number
  ry: number
  /** radians */
  rot: number
  opacity: number
}

export interface Link {
  id: Id
  from: Id
  to: Id
  label?: string
  tone?: Tone
  /** 'spoke' = center-to-part line; 'causal' = labelled arrow; 'relation' = revealed relationship. */
  style: 'spoke' | 'causal' | 'relation'
}

export interface LayoutResult {
  placements: Record<Id, Placement>
  orbits: Orbit[]
  /** Links drawn in this arrangement. Hover/focus reveals are added by the renderer. */
  links: Link[]
  /** Camera the surface wants (the camera flies here when the surface/focus changes). */
  camera: Camera
  /** The object acting as the center of gravity. */
  center: Id
}

/** Semantic level of detail. 0 = dot, 1 = labeled sphere + status, 2 = card with parts, 3 = source. */
export type Lod = 0 | 1 | 2 | 3

// ---------------------------------------------------------------------------
// Artifacts

export interface DiffLine {
  sign: '+' | '-' | ' ' | '@'
  text: string
  /** Line number shown in the gutter. */
  n?: number
}

export interface DiffFile {
  path: string
  added: number
  removed: number
  lines: DiffLine[]
}

export type ArtifactExcerpt =
  | { type: 'diff'; lines: DiffLine[] }
  | { type: 'quote'; text: string; ref?: string }
  | { type: 'table'; columns: string[]; rows: string[][] }
  | { type: 'chart'; unit: string; points: { label: string; value: number }[] }
  | { type: 'doc'; text: string; version?: string }

export type ArtifactBody =
  | {
      type: 'diff'
      breadcrumb: string[]
      title: string
      status?: Status
      description: string
      meta: { state: string; author: string; date: string; branch: string }
      tabs: { id: string; label: string; count: number }[]
      files: DiffFile[]
    }
  | { type: 'doc'; breadcrumb: string[]; title: string; sections: { heading?: string; text: string }[] }
  | { type: 'table'; breadcrumb: string[]; title: string; columns: string[]; rows: string[][] }
  | { type: 'chart'; breadcrumb: string[]; title: string; unit: string; points: { label: string; value: number }[] }

export interface ArtifactSpec {
  id: Id
  /** Object the excerpt is anchored beside. */
  source: Id
  kind: 'diff' | 'quote' | 'table' | 'chart' | 'doc' | 'record'
  title: string
  /** Right-aligned meta in the excerpt header, e.g. "EHR-77392". */
  ref?: string
  excerpt: ArtifactExcerpt
  body: ArtifactBody
}

export type ArtifactPhase = 'excerpt' | 'expanded'

export interface OpenArtifact {
  id: Id
  phase: ArtifactPhase
  pinned: boolean
  /** Object the excerpt was disclosed from (an artifact can be bound to several objects). */
  source?: Id
}

/** Everything needed to restore the world exactly when an expanded artifact closes. */
export interface ViewSnapshot {
  camera: Camera
  focusStack: Id[]
  revision: string
  surface: SurfaceId
  beatOverrides: BeatOverrides
  compare: CompareState | null
}

// ---------------------------------------------------------------------------
// Beats and narratives

export interface Beat {
  caption: string
  /** Secondary caption line under the center title (e.g. "Why this failed"). */
  subcaption?: string
  focus?: Id
  surface?: SurfaceId
  reveal?: Id[]
  dim?: Id[]
  relationships?: Id[]
  artifacts?: Id[]
  /** Revision id to move to. */
  time?: string
  /** ms to hold before auto-advancing. Default 2600. */
  holdMs?: number
  /** Residue in the center under the title while this beat holds. */
  residue?: Status
  highlight?: Id[]
  annotate?: { target: Id; label: string }[]
  /** Open a historical comparison (cursor or 'live'). */
  compare?: { a: string; b: string }
  /** Evidence refs the beat cites (spec 04 NarrationBeat.evidenceCitations). */
  citations?: string[]
  /** Complete governed Thoth disclosure this beat was derived from (never an authority grant). */
  grounded?: ThothAnswerView
}

export interface Narrative {
  id: Id
  /** The question that started it (shown while paused). */
  question: string
  beats: Beat[]
  /** true once the agent stream has delivered its last beat. */
  complete: boolean
}

export type NarrativeStatus = 'playing' | 'paused' | 'done'

export interface NarrativeState {
  id: Id
  index: number
  status: NarrativeStatus
  /** Set when the agent adapter failed mid-explanation. */
  error?: string
}

/** Accumulated beat effects that layouts read (reset when the narrative is dismissed). */
export interface BeatOverrides {
  reveal: Id[]
  dim: Id[]
  relationships: Id[]
  caption?: string
  subcaption?: string
  residue?: Status
  highlight: Id[]
  annotations: { target: Id; label: string }[]
}

// ---------------------------------------------------------------------------
// Modes, time, comparison, judgment, intents, execution

/** Dense workbench modes are entered deliberately and never leak into Home. */
export type Mode = 'world' | 'case-design' | 'execution-inspect'

/** Comparison representation over two snapshots of the same objects. */
export interface CompareState {
  /** Which history the cursors belong to. */
  source: 'world' | 'design'
  a: string
  b: string
  /** What to go back to when comparison closes. */
  returnTo: { revision: string; surface: SurfaceId }
}

export interface Actor {
  id: string
  role: string
  name: string
  kind: 'human' | 'agent'
}

export interface PendingJudgment {
  object: Id
  /** The consequential action that opened the judgment. */
  action: ObjectAction
  /** Intent id once a choice was sent; the panel stays until the backend answers. */
  pending?: string
  outcome?: { state: 'accepted' | 'refused'; note?: string; code?: string; by: Actor }
}

export interface IntentRecord {
  id: string
  actionName: string
  target?: Id
  actor: Actor
  state: 'sending' | 'accepted' | 'refused'
  note?: string
  code?: string
  justification?: string
}

/** Execution projected from contract events. Completion is not settlement. */
export interface ExecutionState {
  object: Id
  runId: string
  phase: string
  /** 0..1 when the stream reported progress; null when only snapshot standing is known (never guessed). */
  progress: number | null
  log: string[]
  state: 'running' | 'executed' | 'settled' | 'rejected' | 'failed'
}

/** Case design from templates (T09): picker → parameters → preflight → commit. */
export interface ProposalState {
  /** Template list loading through the port, ready, or unavailable (shown honestly). */
  status: 'loading' | 'ready' | 'unavailable'
  templates: readonly TemplateEntryOption[]
  error?: string
  selected: string | null
  params: Record<string, string>
  preflighting: boolean
  preflight: { passed: boolean; digest?: string; reasons: readonly string[] } | null
  submitting: boolean
  /** The committed case, ready to focus. */
  result?: { caseId: string }
  submitError?: string
  submitCode?: string
}

/** Case design: the template world, its authoritative versions and a local proposal. */
export interface DesignState {
  caseId: string
  history: WorldHistory
  /** Cursor of the published version being shown, or 'draft'. */
  revision: string
  /** Local, unsubmitted proposal (never authoritative). */
  draft: WorldSnapshot | null
  /** Where Case Design was entered from, restored on exit. */
  returnFocus: Id[]
  selected: Id | null
}

export interface UiState {
  theme: Theme
  mode: Mode
  history: WorldHistory
  /** Revision currently shown. The last revision is "now". */
  revision: string
  /** Focus stack, outermost first. Empty = Home. */
  focusStack: Id[]
  surface: SurfaceId
  selection: Id | null
  hover: Id | null
  artifacts: OpenArtifact[]
  /** Snapshot taken when an artifact expanded; restored on collapse. */
  expandedFrom: ViewSnapshot | null
  narrative: NarrativeState | null
  narratives: Record<Id, Narrative>
  overrides: BeatOverrides
  judgment: PendingJudgment | null
  /** Discretionary-work drawer (T09): a typed add-work surface on a case/stage. */
  drawer: { object: Id; action: ObjectAction } | null
  /** Template-based case design (T09): picker → params → preflight → commit. */
  proposal: ProposalState | null
  intents: IntentRecord[]
  /** Temporal inspection is open (history strip shown). */
  timeline: boolean
  /** Up to two history positions marked for comparison. */
  timeMarks: string[]
  compare: CompareState | null
  executions: Record<Id, ExecutionState>
  design: DesignState | null
  /** Narration/agent adapter availability. Direct UI never depends on it. */
  agent: 'available' | 'unavailable'
  /** Event-stream state; interrupted means the last source standing may be stale. */
  connection: 'live' | 'reconnecting' | 'interrupted'
  /** Monotonic counter; bump to ask the camera to fly to the current layout's camera. */
  cameraRequest: number
  /** When set with a cameraRequest, fly here instead of the layout camera (artifact collapse). */
  cameraRestore: Camera | null
  /** Pointer activity wakes the idle world (0..1, decays in the render loop). */
  awake: boolean
}

// ---------------------------------------------------------------------------
// One action vocabulary for humans and agents. Agents never produce coordinates.

export type Action =
  | { type: 'focus'; id: Id }
  | { type: 'back' }
  | { type: 'home' }
  | { type: 'select'; id: Id | null }
  | { type: 'hover'; id: Id | null }
  | { type: 'reveal'; ids: Id[] }
  | { type: 'connect'; relationships: Id[] }
  | { type: 'arrange'; surface: SurfaceId }
  | { type: 'setTime'; revision: string }
  | { type: 'returnToNow' }
  | { type: 'openTimeline'; open: boolean }
  | { type: 'markTime'; revision: string }
  | { type: 'openCompare'; a: string; b: string; source?: 'world' | 'design' }
  | { type: 'closeCompare' }
  | { type: 'materializeArtifact'; id: Id; source?: Id }
  | { type: 'expandArtifact'; id: Id; snapshot: ViewSnapshot }
  | { type: 'collapseArtifact' }
  | { type: 'dismissArtifact'; id: Id }
  | { type: 'pinArtifact'; id: Id; pinned: boolean }
  | { type: 'invoke'; object: Id; action: ObjectAction }
  | { type: 'openDrawer'; object: Id; action: ObjectAction }
  | { type: 'closeDrawer' }
  | { type: 'openProposals' }
  | { type: 'proposalsLoaded'; templates: readonly TemplateEntryOption[] }
  | { type: 'proposalsUnavailable'; error: string }
  | { type: 'selectTemplate'; templateRef: string | null }
  | { type: 'setProposalParam'; name: string; value: string }
  | { type: 'preflightStarted' }
  | { type: 'preflightResult'; result: { passed: boolean; digest?: string; reasons: readonly string[] } }
  | { type: 'proposalSubmitStarted' }
  | { type: 'proposalSubmitted'; caseId: string }
  | { type: 'proposalError'; error: string; code?: string }
  | { type: 'closeProposals' }
  | { type: 'connectionState'; connection: 'live' | 'reconnecting' | 'interrupted' }
  | { type: 'intentSent'; record: IntentRecord; judgment?: boolean }
  | { type: 'closeJudgment' }
  | { type: 'intentSettled'; id: string; state: 'accepted' | 'refused'; note?: string; code?: string }
  | { type: 'execution'; exec: ExecutionState }
  | { type: 'narrativeStarted'; narrative: Narrative }
  | { type: 'narrativeBeat'; id: Id; beat: Beat; complete: boolean }
  | { type: 'narrativeComplete'; id: Id }
  | { type: 'narrativeFailed'; id: Id; error: string }
  | { type: 'applyBeat'; beat: Beat; index: number }
  | { type: 'pauseNarrative' }
  | { type: 'resumeNarrative' }
  | { type: 'endNarrative' }
  | { type: 'agentAvailability'; available: boolean }
  | { type: 'setTheme'; theme: Theme }
  | { type: 'setMode'; mode: Mode }
  | { type: 'enterDesign'; design: DesignState }
  | { type: 'designSelect'; id: Id | null }
  | { type: 'designShow'; revision: string }
  | { type: 'designDraft'; draft: WorldSnapshot | null }
  | { type: 'loadHistory'; history: WorldHistory }
  | { type: 'wake'; awake: boolean }
