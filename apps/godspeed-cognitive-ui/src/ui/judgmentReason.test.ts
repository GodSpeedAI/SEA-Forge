import { describe, expect, test } from 'bun:test'
import { decisionKeyOf, reasonFor } from './JudgmentPanel'

describe('the reason typed in the judgment panel belongs to one decision', () => {
  const opt = (id: string) => ({ id })
  const approve = decisionKeyOf('Approve: draft change note?', [opt('act-approve_human_task-task_draft'), opt('act-reject_human_task-task_draft')])
  const terminate = decisionKeyOf('Terminate case: case_1?', [opt('act-terminate_case-case_1')])

  test('is read back for the decision it was typed for', () => {
    expect(reasonFor({ key: approve, text: 'checked' }, approve)).toBe('checked')
  })

  test('is not carried into a different decision', () => {
    expect(reasonFor({ key: approve, text: 'checked the draft' }, terminate)).toBe('')
  })

  test('starts empty', () => {
    expect(reasonFor(null, terminate)).toBe('')
  })

  test('two decisions with the same question but different offers are different decisions', () => {
    const a = decisionKeyOf('Same?', [opt('x')])
    const b = decisionKeyOf('Same?', [opt('y')])
    expect(a).not.toBe(b)
    expect(reasonFor({ key: a, text: 'r' }, b)).toBe('')
  })
})
