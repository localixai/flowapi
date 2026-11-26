#!/usr/bin/env node
// The judge's command line. Usage in HELP below; the steps are lib/judge.mjs.
// Exit code: 0 when every step run passed, 1 when one failed or was skipped, 2 on wrong usage.

import { existsSync, statSync } from 'node:fs'
import { resolve } from 'node:path'
import { STEPS, judge } from '../lib/judge.mjs'

const HELP = `usage: judge.mjs <sdk-dir> [--steps 0-6] [--json] [--keep-going]
                 [--generate "<command>"] [--timeout <seconds>]

Checks a TypeScript SDK repository generated from flowapi-sdk.yml:
${STEPS.map((s) => `  ${s.id}  ${s.name}`).join('\n')}

  --steps       which steps: "0-6" (the default), "1,3", "2-4,6"
  --json        the report as JSON on stdout
  --keep-going  run every step even after one failed
  --generate    the generate command for step 0, run in <sdk-dir> by the shell; {out} is the
                empty folder to write into, {flowapi} the version from .flowapi/lock.json
                (default: FLOWAPI_PLUGIN_FLOWAPI, else npx -y @localix-ai/flowapi@{flowapi}, then
                generate flowapi-sdk.yml --out {out})
  --timeout     the limit for each command, in seconds (default 600)
`

function usage(message) {
  process.stderr.write(`${message}\n\n${HELP}`)
  process.exit(2)
}

function parseSteps(text) {
  const ids = new Set()
  for (const part of text.split(',')) {
    const match = /^\s*(\d+)\s*(?:-\s*(\d+)\s*)?$/.exec(part)
    if (!match) throw new Error(`not a step list: ${text}`)
    const from = Number(match[1])
    const to = match[2] === undefined ? from : Number(match[2])
    if (to < from) throw new Error(`not a step range: ${part.trim()}`)
    for (let id = from; id <= to; id++) {
      if (!STEPS.some((s) => s.id === id))
        throw new Error(
          `no step ${id}: steps are 0-${STEPS.length - 1} (7-9 are not part of the judge yet)`,
        )
      ids.add(id)
    }
  }
  return [...ids].sort((a, b) => a - b)
}

function parse(argv) {
  const options = { ids: STEPS.map((s) => s.id), json: false, keepGoing: false }
  let dir = null
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    const value = () => (i + 1 < argv.length ? argv[++i] : usage(`${arg} needs a value`))
    if (arg === '-h' || arg === '--help') {
      process.stdout.write(HELP)
      process.exit(0)
    } else if (arg === '--json') options.json = true
    else if (arg === '--keep-going') options.keepGoing = true
    else if (arg === '--steps') {
      try {
        options.ids = parseSteps(value())
      } catch (error) {
        usage(error.message)
      }
    } else if (arg === '--generate') {
      options.generate = value()
      if (!options.generate.includes('{out}'))
        usage('--generate must contain {out}, the folder to write into')
    } else if (arg === '--timeout') {
      const seconds = Number(value())
      if (!(seconds > 0)) usage('--timeout takes a number of seconds')
      options.timeoutMs = seconds * 1000
    } else if (arg.startsWith('-')) usage(`unknown option ${arg}`)
    else if (dir === null) dir = arg
    else usage(`one <sdk-dir> only, got ${dir} and ${arg}`)
  }
  if (dir === null) usage('<sdk-dir> is missing')
  dir = resolve(dir)
  if (!existsSync(dir) || !statSync(dir).isDirectory()) usage(`not a directory: ${dir}`)
  return { dir, options }
}

function table(report) {
  const lines = [`judge: ${report.dir}`, '']
  const seconds = (ms) => `${(ms / 1000).toFixed(1)}s`
  for (const s of report.steps) {
    const note = s.reason ?? ''
    lines.push(
      `${String(s.step).padEnd(3)}${s.name.padEnd(13)}${s.status.padEnd(6)}${seconds(s.duration_ms).padStart(7)}  ${note}`.trimEnd(),
    )
    if (s.status === 'fail') {
      if (s.command) lines.push(`     $ ${s.command}`)
      if (s.output) lines.push(...s.output.split('\n').map((line) => `     | ${line}`))
    }
  }
  lines.push('', `verdict: ${report.verdict}`)
  return `${lines.join('\n')}\n`
}

const { dir, options } = parse(process.argv.slice(2))
const report = await judge(dir, options)
process.stdout.write(options.json ? `${JSON.stringify(report, null, 2)}\n` : table(report))
process.exitCode = report.verdict === 'pass' ? 0 : 1
