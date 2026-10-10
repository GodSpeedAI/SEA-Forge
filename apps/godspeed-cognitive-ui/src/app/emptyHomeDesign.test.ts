import { describe, expect, test } from 'bun:test'
import { emptyWorldOffersDesign } from './proposals'

describe('empty-world design entry (T10 L1 defect: no way to start a case on a fresh live cell)', () => {
  test('a world holding only the core offers the design entry', () => {
    expect(emptyWorldOffersDesign({ core: { kind: 'core' } })).toBe(true)
  })
  test('categories alone do not count as something to focus', () => {
    expect(emptyWorldOffersDesign({ core: { kind: 'core' }, c: { kind: 'category' } })).toBe(true)
  })
  test('a world with a case object keeps the focus-first entry', () => {
    expect(emptyWorldOffersDesign({ core: { kind: 'core' }, case_1: { kind: 'case' } })).toBe(false)
  })
})
