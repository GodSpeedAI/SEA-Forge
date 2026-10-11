// Bundle scan for the live ladder (T10 precondition): the production build served to the
// browser must carry the HTTP adapter and none of the fixture/local adapter machinery.
// Pure functions over { filename -> source text } so they are unit-testable without a build.

export const FORBIDDEN_TOKENS = [
  'LocalContractAdapter',
  'NORTHSTAR_CASE_ID',
  'northstarData',
  'createLocalAgent',
  'localAdapter',
  'localAgent',
  'evi-case-timeline',
  'case-northstar',
] as const

export const REQUIRED_CHUNK = /^httpCaseworkAdapter-[\w-]+\.js$/

export interface BundleScan {
  ok: boolean
  scanned: string[]
  violations: { file: string; token: string }[]
  hasHttpAdapter: boolean
  problems: string[]
}

/** Scan JS sources keyed by base filename (e.g. "index-abc.js"). Non-.js keys are ignored. */
export function scanSources(files: Record<string, string>): BundleScan {
  const scanned = Object.keys(files).filter((f) => f.endsWith('.js')).sort()
  const violations: BundleScan['violations'] = []
  for (const file of scanned) {
    for (const token of FORBIDDEN_TOKENS) {
      if (files[file]!.includes(token)) violations.push({ file, token })
    }
  }
  const hasHttpAdapter = scanned.some((f) => REQUIRED_CHUNK.test(f))
  const problems: string[] = []
  if (scanned.length === 0) problems.push('no JS assets found to scan')
  if (!hasHttpAdapter) problems.push('required chunk httpCaseworkAdapter-*.js is absent')
  for (const v of violations) problems.push(`forbidden token ${v.token} in ${v.file}`)
  return { ok: problems.length === 0, scanned, violations, hasHttpAdapter, problems }
}

/** Extract same-origin /assets/*.js references from served HTML or JS (import("./x.js") style too). */
export function assetRefs(text: string): string[] {
  const out = new Set<string>()
  for (const m of text.matchAll(/(?:\/assets\/|\.\/)([\w.-]+\.js)\b/g)) out.add(m[1]!)
  return [...out]
}
