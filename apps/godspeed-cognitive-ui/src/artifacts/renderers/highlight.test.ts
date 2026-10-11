import { describe, it, expect } from 'bun:test'
import { highlight } from './highlight'

describe('highlight', () => {
  it('empty query returns one part with hit false and count 0', () => {
    const result = highlight('hello world', '')
    expect(result).toEqual({
      parts: [{ text: 'hello world', hit: false }],
      count: 0,
    })
  })

  it('case-insensitive matching', () => {
    const result = highlight('Hello World', 'hello')
    expect(result.count).toBe(1)
    expect(result.parts.length).toBe(2)
    expect(result.parts[0]).toEqual({ text: 'Hello', hit: true })
  })

  it('multiple hits', () => {
    const result = highlight('foo bar foo baz foo', 'foo')
    expect(result.count).toBe(3)
    // Should have 7 parts: 'foo' 'bar foo baz' 'foo'... but split properly
    const hitParts = result.parts.filter((p) => p.hit)
    expect(hitParts).toHaveLength(3)
    expect(hitParts.every((p) => p.text === 'foo')).toBe(true)
  })

  it('special regex characters are literal', () => {
    const result = highlight('a.b.c', 'a.b')
    expect(result.count).toBe(1)
    expect(result.parts[0]).toEqual({ text: 'a.b', hit: true })
  })

  it('query with brackets', () => {
    const result = highlight('[test]', '[test]')
    expect(result.count).toBe(1)
    expect(result.parts[0]).toEqual({ text: '[test]', hit: true })
  })

  it('no match returns single non-hit part', () => {
    const result = highlight('hello world', 'xyz')
    expect(result.count).toBe(0)
    expect(result.parts).toEqual([{ text: 'hello world', hit: false }])
  })

  it('query at start', () => {
    const result = highlight('hello world', 'hello')
    expect(result.count).toBe(1)
    expect(result.parts[0]).toEqual({ text: 'hello', hit: true })
  })

  it('query at end', () => {
    const result = highlight('hello world', 'world')
    expect(result.count).toBe(1)
    const lastPart = result.parts[result.parts.length - 1]
    expect(lastPart).toEqual({ text: 'world', hit: true })
  })

  it('non-overlapping patterns', () => {
    const result = highlight('aaa', 'aa')
    // Regex exec finds 'aa' at position 0, then continues from position 2, so count 1
    expect(result.count).toBe(1)
    expect(result.parts[0]).toEqual({ text: 'aa', hit: true })
  })
})
