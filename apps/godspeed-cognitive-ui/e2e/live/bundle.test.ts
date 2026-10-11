import { describe, expect, test } from 'bun:test'
import { assetRefs, FORBIDDEN_TOKENS, scanSources } from './bundle'

const clean = { 'index-abc.js': 'import("./httpCaseworkAdapter-XyZ_1.js")', 'httpCaseworkAdapter-XyZ_1.js': 'class HttpCaseworkAdapter{}' }

describe('bundle scan', () => {
  test('a clean build with the HTTP adapter chunk passes', () => {
    const r = scanSources(clean)
    expect(r.ok).toBe(true)
    expect(r.hasHttpAdapter).toBe(true)
    expect(r.violations).toEqual([])
  })
  test.each([...FORBIDDEN_TOKENS])('fails when a chunk contains %s', (token) => {
    const r = scanSources({ ...clean, 'x-1.js': `const a = "${token}"` })
    expect(r.ok).toBe(false)
    expect(r.violations).toEqual([{ file: 'x-1.js', token }])
  })
  test('fails when the HTTP adapter chunk is missing', () => {
    const r = scanSources({ 'index-abc.js': 'x' })
    expect(r.ok).toBe(false)
    expect(r.problems.join(';')).toContain('httpCaseworkAdapter')
  })
  test('fails closed on an empty scan and ignores non-JS files', () => {
    expect(scanSources({}).ok).toBe(false)
    expect(scanSources({ 'a.css': 'LocalContractAdapter', ...clean }).ok).toBe(true)
  })
  test('assetRefs finds same-origin asset scripts in html and dynamic imports', () => {
    const refs = assetRefs('<script src="/assets/index-A.js"></script> import("./JsonRenderer-B.js")')
    expect(refs.sort()).toEqual(['JsonRenderer-B.js', 'index-A.js'])
  })
})
