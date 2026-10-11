import { describe, expect, test } from 'bun:test'
import { stubGatewayToothProblem } from './teeth'

const l1 = (name: string, error: string) => [{ journey_id: 'L1', status: 'FAIL', steps: [{ name: 'ui', ok: true }, { name, ok: false, error }] }]

describe('stubGatewayToothProblem', () => {
  test('bites: durable step timed out and the cell gained no case', () => {
    expect(stubGatewayToothProblem(l1('durable: case_created in case-events.jsonl', 'timed out after 30s'), [], [])).toBeNull()
  })
  test('a case directory in the cell means the stub was not a stub', () => {
    expect(stubGatewayToothProblem(l1('durable: x', 'timed out'), [], ['case_1'])).toMatch(/gained case directories: case_1|case_1/)
  })
  test('pre-existing case directories are not new', () => {
    expect(stubGatewayToothProblem(l1('durable: x', 'timed out'), ['old'], ['old'])).toBeNull()
  })
  test('failing at a UI step or for another reason does not count', () => {
    expect(stubGatewayToothProblem(l1('ui: open', 'timed out'), [], [])).toMatch(/not at a durable step/)
    expect(stubGatewayToothProblem(l1('durable: x', 'boom'), [], [])).toMatch(/another reason/)
  })
  test('a passing or missing L1 is not a bite', () => {
    expect(stubGatewayToothProblem([{ journey_id: 'L1', status: 'PASS', steps: [] }], [], [])).toMatch(/no failed step/)
    expect(stubGatewayToothProblem([], [], [])).toMatch(/no failed step/)
  })
})
