import { describe, expect, test } from 'bun:test'
import { HttpRefusalError } from '../adapters/http/httpCaseworkAdapter'
import { artifactErrorText } from './service'

describe('artifactErrorText', () => {
  test('a typed integrity refusal reads as a digest mismatch, not an outage', () => {
    for (const kind of ['integrity_mismatch', 'INTEGRITY_MISMATCH']) {
      const text = artifactErrorText(new HttpRefusalError(kind, 'the stored artifact no longer matches its digest; it was not served'))
      expect(text).toMatch(/no longer matches its digest/)
      expect(text).toMatch(/not opened/)
    }
  })
  test('other errors keep their own message', () => {
    expect(artifactErrorText(new HttpRefusalError('unavailable', 'the governed artifact authority is unavailable'))).toBe('the governed artifact authority is unavailable')
    expect(artifactErrorText('boom')).toBe('boom')
  })
})
