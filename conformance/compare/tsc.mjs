// One run of the TypeScript compiler over a tool's files, its diagnostics read back by file and
// line, and each case judged, as conformance/suite/src/tsc.rs and evaluate.rs do.
import { spawnSync } from 'node:child_process'

// Checks `files` (names inside `dir`) under the oracle's options, from inside `dir`.
export function check(tsc, dir, files) {
  const args = ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--noEmit']
  args.push('--target', 'es2022', '--skipLibCheck', '--pretty', 'false', ...files)
  const run = spawnSync(tsc, args, { cwd: dir, encoding: 'utf8', maxBuffer: 1 << 30 })
  if (run.error) throw new Error(`cannot run ${tsc}: ${run.error.message} (npm ci in conformance/ts)`)
  return parse(`${run.stdout}${run.stderr}`)
}

// `file(line,column): error TSnnnn: message`, then lines indented under it.
export function parse(text) {
  const byFile = new Map()
  const other = []
  let last = null
  for (const line of text.split(/\r?\n/)) {
    if (line.startsWith(' ')) {
      const messages = last && byFile.get(last[0])?.get(last[1])
      if (messages) messages[messages.length - 1] += `\n${line.trimEnd()}`
      else if (!last) other.push(line)
      continue
    }
    const m = located(line)
    if (m) {
      const [file, n, message] = m
      if (!byFile.has(file)) byFile.set(file, new Map())
      const lines = byFile.get(file)
      if (!lines.has(n)) lines.set(n, [])
      lines.get(n).push(message)
      last = [file, n]
    } else {
      if (line.trim() !== '') other.push(line)
      last = null
    }
  }
  return { byFile, other }
}

function located(line) {
  const split = line.indexOf('): ')
  if (split < 0) return null
  const head = line.slice(0, split)
  const open = head.lastIndexOf('(')
  if (open < 0) return null
  const position = head.slice(open + 1)
  const comma = position.indexOf(',')
  if (comma < 0 || !/^[0-9]+$/.test(position.slice(0, comma))) return null
  return [head.slice(0, open), Number(position.slice(0, comma)), line.slice(split + 3)]
}

// One group's verdict: `{ invalidTs }` with the messages above the first case, or the counts
// and the valid cases rejected.
export function judge(group, file, firstCaseLine, diagnostics) {
  const lines = diagnostics.byFile.get(file) ?? new Map()
  const onTypes = [...lines]
    .filter(([n]) => n < firstCaseLine)
    .sort(([a], [b]) => a - b)
    .flatMap(([n, ms]) => ms.map((m) => `line ${n}: ${m}`))
  if (onTypes.length > 0) return { invalidTs: onTypes.join('\n') }
  const r = { valid: 0, validAccepted: 0, invalid: 0, invalidRejected: 0, unsound: [] }
  group.cases.forEach((c, i) => {
    const errors = lines.get(firstCaseLine + i)
    if (c.valid) {
      r.valid++
      if (!errors) r.validAccepted++
      else r.unsound.push({ description: c.description, data: c.data, error: errors.join('\n') })
    } else {
      r.invalid++
      if (errors) r.invalidRejected++
    }
  })
  return r
}
