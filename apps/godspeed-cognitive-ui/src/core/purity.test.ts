/**
 * Core purity (T03): the interaction model contains no backend, renderer, transport, or agent
 * framework authority. Enforced mechanically INSIDE the test suite so `just casework-ui-check`
 * observes it on every run — not as a one-off grep.
 */
import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const CORE_DIR = join(import.meta.dir)
const FORBIDDEN_IMPORTS = [
  'react',
  'react-dom',
  'three',
  '@react-three',
  '@copilotkit',
  'copilotkit',
  'vite',
]
const FORBIDDEN_IDENTIFIERS = [
  'sfwp',
  'gauntlet',
  'sea-forge-server',
  'godspeed-casework-go',
  'ndjson',
  'XMLHttpRequest',
  'WebSocket',
  'EventSource',
  'copilotkit',
  'usestate',
  'useeffect',
  'jsx',
]
const FORBIDDEN_CALLS = ['fetch(']

function coreSources(): readonly { file: string; text: string }[] {
  return readdirSync(CORE_DIR)
    .filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))
    .map((f) => ({ file: f, text: readFileSync(join(CORE_DIR, f), 'utf8') }))
}

describe('the UI core is free of renderer, transport and backend authority', () => {
  const sources = coreSources()

  test('the scan itself is non-vacuous', () => {
    expect(sources.length).toBeGreaterThanOrEqual(4)
    expect(sources.some((s) => s.file.includes('engine'))).toBe(true)
  })

  test('no forbidden import appears in any core file', () => {
    for (const { file, text } of sources) {
      const imports = [...text.matchAll(/from\s+['"]([^'"]+)['"]/g)].map((m) => m[1] ?? '')
      for (const imp of imports) {
        for (const bad of FORBIDDEN_IMPORTS) {
          expect(imp.includes(bad) ? `${file}: imports ${imp} (matches ${bad})` : '').toBe('')
        }
      }
    }
  })

  test('no provider, transport or renderer-library identifier appears in any core file', () => {
    for (const { file, text } of sources) {
      const lower = text.toLowerCase()
      for (const bad of FORBIDDEN_IDENTIFIERS) {
        expect(lower.includes(bad.toLowerCase()) ? `${file}: names ${bad}` : '').toBe('')
      }
      for (const call of FORBIDDEN_CALLS) {
        expect(text.includes(call) ? `${file}: calls ${call}` : '').toBe('')
      }
    }
  })
})
