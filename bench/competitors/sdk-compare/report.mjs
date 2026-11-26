// results/sdk-compare.md, written from what `sdk-compare.mjs` measured (the same JSON it writes to
// results/sdk-compare.json), never by hand.

const time = (ms) => (ms === undefined ? '-' : ms < 10 ? `${ms.toFixed(1)} ms` : ms < 1000 ? `${ms.toFixed(0)} ms` : `${(ms / 1000).toFixed(2)} s`)
const size = (n) => (n === undefined ? '-' : n < 1e6 ? `${(n / 1e3).toFixed(0)} KB` : `${(n / 1e6).toFixed(1)} MB`)
const memory = (kb) => (kb === undefined ? '-' : `${(kb / 1024).toFixed(0)} MB`)

/** One cell of the feature table: what every description that tests the feature observed. */
function cell(results, tool, feature) {
  const seen = results.rows.filter((r) => r.tool === tool && r.behavior?.[feature])
  if (seen.length === 0) return results.rows.some((r) => r.tool === tool && !r.ok) ? 'not generated' : '-'
  const by = (what) => seen.filter((r) => r.behavior[feature].result === what)
  const fails = by('fail')
  if (fails.length > 0) return `fail (${fails.map((r) => r.file).join(', ')})`
  if (by('pass').length === 0) return 'n/a'
  const na = by('n/a')
  return `pass (${by('pass').length} of ${seen.length - na.length})`
}

export function report(results) {
  const { rows, tools } = results
  const lines = [
    '# The whole SDK: flowapi against the other client generators',
    '',
    `Written by \`node bench/competitors/sdk-compare.mjs\`; do not edit. ${results.date}, commit ${results.commit}, ${results.machineLine}, Node.js ${results.node}.`,
    '',
    `Versions: ${Object.entries(results.versions).map(([k, v]) => `${k} ${v}`).join(', ')}.`,
    '',
  ]
  if (results.only) lines.push(`**A partial run** (\`--only ${results.only.join(',')}\`): not every description was measured.`, '')
  lines.push(
    `Each generation is a fresh process into an empty folder, the median of ${results.rounds} with its peak resident memory; flowapi runs \`generate\` on the config \`config draft\` wrote (the whole repository: client, tests, mock, README), the others their client mode. Files and bytes are all a generator wrote. \`tsc\` errors are those of \`tsc ${results.tscFlags.join(' ')}\` on the SDK's TypeScript (flowapi's \`src/\`; openapi-typescript's types with the five lines that compose them with openapi-fetch).`,
    '',
  )
  for (const d of results.descriptions) {
    lines.push(`## ${d.name}`, '', `${d.about} (\`${d.file}\`, ${size(d.bytes)}).`, '', '| Generator | Generate | Peak memory | Files | Bytes | TypeScript | tsc errors |', '| --- | ---: | ---: | ---: | ---: | ---: | ---: |')
    for (const tool of tools) {
      const r = rows.find((x) => x.file === d.name && x.tool === tool)
      if (!r) continue
      if (!r.ok) {
        lines.push(`| ${tool} | failed: ${r.failure.replaceAll('|', '\\|').slice(0, 160)} | | | | | |`)
        continue
      }
      lines.push(`| ${tool} | ${time(r.medianMs)} | ${memory(r.peakRssKb)} | ${r.files} | ${size(r.bytes)} | ${size(r.tsBytes)} | ${r.tscErrors} |`)
    }
    lines.push('')
  }
  lines.push(
    '## Behavior',
    '',
    'Each SDK called through its own surface against one mock server made from the description, which records the requests as they arrive. `pass (k of n)`: passed on every description that tests it; `fail (where)`: the generated code has the feature and the test failed; `n/a`: no surface for it in the generated code (an option, a method, a credential), the caller does it; `-`: no description here tests it. Every cell is an observation; none is read from documentation.',
    '',
    `| Feature | ${tools.join(' | ')} |`,
    `| --- | ${tools.map(() => '---').join(' | ')} |`,
    ...results.features.map(([id, label]) => `| ${label} | ${tools.map((t) => cell(results, t, id)).join(' | ')} |`),
    '',
    '### What each failure sent',
    '',
  )
  const failed = rows.flatMap((r) => Object.entries(r.behavior ?? {}).filter(([, b]) => b.result === 'fail').map(([id, b]) => `- ${r.tool}, ${r.file}, \`${id}\`: ${b.observed.replaceAll('\n', ' ')}`))
  const broken = rows.filter((r) => r.loadFailure || r.emitFailure).map((r) => `- ${r.tool}, ${r.file}: ${r.loadFailure ?? ''} ${r.emitFailure ?? ''}`.trim())
  lines.push(...(failed.length ? failed : ['None.']), '')
  if (broken.length) lines.push('### SDKs that did not load', '', ...broken, '')
  lines.push('### The first tsc errors', '')
  const errors = rows.filter((r) => r.tscErrors > 0).map((r) => `- ${r.tool}, ${r.file} (${r.tscErrors}): ${r.tscSample.map((s) => `\`${s.replaceAll('`', "'")}\``).join('; ')}`)
  lines.push(...(errors.length ? errors : ['None.']), '', '## Not measured', '', ...results.excluded.map((e) => `- ${e.tool}: ${e.why}.`), '')
  return lines.join('\n')
}
