// compare.json and compare.md: the same input gives the same bytes (files by name, groups and
// cases by index, no time, no path).

// The listed unsound cases per tool; the totals count all of them.
const UNSOUND_CAP = 50

// A share with one decimal, rounded down so that 100.0 means all of them (evaluate.rs).
export function percent(part, whole) {
  if (whole === 0) return '-'
  const tenths = Math.floor((part * 1000) / whole)
  return `${Math.floor(tenths / 10)}.${tenths % 10}`
}

function totals(tool) {
  const t = { groups_run: 0, generation_errors: 0, invalid_typescript: 0, valid_cases: 0, valid_accepted: 0 }
  Object.assign(t, { invalid_cases: 0, invalid_rejected: 0 })
  for (const f of tool.files) {
    t.groups_run += f.run
    t.generation_errors += f.generationErrors.length
    t.invalid_typescript += f.invalidTs.length
    t.valid_cases += f.valid
    t.valid_accepted += f.validAccepted
    t.invalid_cases += f.invalid
    t.invalid_rejected += f.invalidRejected
  }
  t.soundness_percent = percent(t.valid_accepted, t.valid_cases)
  t.precision_percent = percent(t.invalid_rejected, t.invalid_cases)
  return t
}

// Soundness over the groups that every tool generated into compiling TypeScript, so the
// denominators are the same.
function common(tools) {
  let groups = 0
  const per = tools.map(() => ({ valid: 0, accepted: 0 }))
  tools[0].files.forEach((f, fi) => {
    for (const [index] of f.ok) {
      const all = tools.map((t) => t.files[fi].ok.get(index))
      if (all.some((x) => x === undefined)) continue
      groups++
      all.forEach((x, ti) => {
        per[ti].valid += x.valid
        per[ti].accepted += x.accepted
      })
    }
  })
  return { groups, per }
}

const firstLine = (s) => s.split('\n')[0]
const failures = (tool, key) =>
  tool.files.flatMap((f) => f[key].map((x) => ({ file: f.name, group: x.group, description: x.description, message: firstLine(x.message) })))
const unsound = (tool) =>
  tool.files.flatMap((f) =>
    f.unsound.map((u) => ({ file: f.name, group: u.group, group_description: u.group_description, description: u.description, data: u.data, error: firstLine(u.error) })),
  )

export function json(tools, meta) {
  const c = common(tools)
  const out = {
    suite: 'json-schema-test-suite',
    dialect: '2020-12',
    typescript: meta.typescript,
    common_groups: c.groups,
    tools: Object.fromEntries(
      tools.map((t, ti) => {
        const all = unsound(t)
        return [
          t.id,
          {
            version: t.version,
            totals: totals(t),
            common_groups: { valid_cases: c.per[ti].valid, valid_accepted: c.per[ti].accepted, soundness_percent: percent(c.per[ti].accepted, c.per[ti].valid) },
            unsound_total: all.length,
            unsound: all.slice(0, UNSOUND_CAP),
            generation_errors: failures(t, 'generationErrors'),
            invalid_typescript: failures(t, 'invalidTs'),
            compiler_other: t.other,
          },
        ]
      }),
    ),
    files: Object.fromEntries(
      tools[0].files.map((f, fi) => [
        f.name,
        Object.fromEntries(
          tools.map((t) => {
            const x = t.files[fi]
            return [t.id, { groups_run: x.run, generation_errors: x.generationErrors.length, invalid_typescript: x.invalidTs.length, valid_cases: x.valid, valid_accepted: x.validAccepted, invalid_cases: x.invalid, invalid_rejected: x.invalidRejected }]
          }),
        ),
      ]),
    ),
  }
  return `${JSON.stringify(out, null, 2)}\n`
}

const cell = (s) => String(s).replace(/\|/g, '\\|').replace(/\n/g, ' ')

export function markdown(tools, meta) {
  const c = common(tools)
  const lines = ['# O3 for every generator: JSON-Schema-Test-Suite through the TypeScript compiler', '']
  lines.push('Written by `node conformance/compare/compare.mjs`; do not edit. How it works and its caveats: `conformance/compare/README.md`.', '')
  lines.push(`TypeScript ${meta.typescript}; ${tools.map((t) => `${t.id} ${t.version}`).join(', ')}.`, '')
  lines.push('| Tool | Version | Groups run | Gen. errors | Invalid TS | Valid | Accepted | Sound % | Invalid | Rejected | Precision % |')
  lines.push('| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |')
  for (const t of tools) {
    const x = totals(t)
    lines.push(`| ${t.id} | ${t.version} | ${x.groups_run} | ${x.generation_errors} | ${x.invalid_typescript} | ${x.valid_cases} | ${x.valid_accepted} | ${x.soundness_percent} | ${x.invalid_cases} | ${x.invalid_rejected} | ${x.precision_percent} |`)
  }
  lines.push('', `On the ${c.groups} groups every tool generated into compiling TypeScript:`, '')
  lines.push('| Tool | Valid | Accepted | Sound % |', '| --- | ---: | ---: | ---: |')
  tools.forEach((t, ti) => lines.push(`| ${t.id} | ${c.per[ti].valid} | ${c.per[ti].accepted} | ${percent(c.per[ti].accepted, c.per[ti].valid)} |`))

  lines.push('', '## Soundness per keyword file', '', 'Accepted of valid cases, then the share; files with no group run are left out.', '')
  lines.push(`| File | ${tools.map((t) => t.id).join(' | ')} |`, `| --- |${tools.map(() => ' ---: |').join('')}`)
  tools[0].files.forEach((f, fi) => {
    if (f.run === 0) return
    const cells = tools.map((t) => {
      const x = t.files[fi]
      return x.valid === 0 ? '-' : `${x.validAccepted}/${x.valid} (${percent(x.validAccepted, x.valid)})`
    })
    lines.push(`| ${f.name} | ${cells.join(' | ')} |`)
  })

  for (const t of tools) {
    const all = unsound(t)
    lines.push('', `## ${t.id}: unsound cases (${all.length}${all.length > UNSOUND_CAP ? `, the first ${UNSOUND_CAP}` : ''})`, '')
    if (all.length === 0) lines.push('None.')
    for (const u of all.slice(0, UNSOUND_CAP)) {
      lines.push(`- \`${u.file}\` #${u.group} ${cell(u.group_description)} / ${cell(u.description)}: \`${u.data}\`  \n  ${cell(u.error)}`)
    }
    for (const [key, title] of [['generationErrors', 'generation errors'], ['invalidTs', 'invalid TypeScript']]) {
      const list = failures(t, key)
      if (list.length === 0) continue
      lines.push('', `### ${t.id}: ${title} (${list.length})`, '')
      for (const g of list) lines.push(`- \`${g.file}\` #${g.group} ${cell(g.description)}: ${cell(g.message)}`)
    }
  }
  return `${lines.join('\n')}\n`
}
