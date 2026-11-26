// A run held to its scenario, after the agent finished: the judge, run by the runner itself (the
// agent's own table is never trusted), passes; and each of the scenario's `expect` holds. Only
// these fail a run; its turns, cost and time are recorded and never budgeted.

import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { hashes } from './scenarios.mjs'

function read(dir, path) {
  const file = join(dir, path)
  return existsSync(file) ? readFileSync(file, 'utf8') : null
}

/**
 * The checks of one run: `[{ what, ok, said }]`. `dir` is the run's folder, `sdk` the SDK's
 * folder in it; `before` the hashes taken before the agent started.
 */
export function verdict({ scenario, stream, judge, dir, sdk, before }) {
  const checks = []
  const add = (what, ok, said = '') => checks.push({ what, ok, said })
  const failed = (judge?.steps ?? []).filter((s) => s.status !== 'pass').map((s) => `${s.step} ${s.status}`)
  add('the judge passes on steps 0-6', judge?.verdict === 'pass' && failed.length === 0, failed.join(', '))
  const expect = scenario.expect
  for (const code of expect.warningsSeen ?? [])
    add(`the agent was shown ${code}`, stream.seenCodes.includes(code), stream.seenCodes.join(', '))
  for (const pattern of expect.reportMatches ?? [])
    add(`its report says /${pattern}/`, new RegExp(pattern, 'i').test(stream.result))
  const config = read(sdk, 'flowapi-sdk.yml') ?? ''
  for (const pattern of expect.configMatches ?? [])
    add(`flowapi-sdk.yml has /${pattern}/`, new RegExp(pattern, 'm').test(config))
  for (const [path, patterns] of Object.entries(expect.fileMatches ?? {})) {
    const text = read(sdk, path) ?? ''
    for (const pattern of patterns) add(`${path} has /${pattern}/`, new RegExp(pattern, 'm').test(text))
  }
  if (expect.unchanged) {
    const after = hashes(dir, expect.unchanged)
    for (const path of expect.unchanged) add(`${path} is unchanged`, before[path] !== null && before[path] === after[path])
  }
  if (expect.refused)
    add('the guard refused a hand edit', stream.refusals.length > 0, JSON.stringify(stream.refusals))
  return checks
}

/** The final `config check --json`'s warnings, each with whether the agent was shown it. */
export function warningsSeen(check, stream) {
  return (check?.issues ?? [])
    .filter((issue) => issue.severity === 'warning')
    .map((issue) => ({ code: issue.code, seen: stream.seenCodes.includes(issue.code) }))
}
