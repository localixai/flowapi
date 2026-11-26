// `results/agent.md`, written from `results/agent.json` (the latest run of each scenario), never
// by hand.

const seconds = (ms) => (typeof ms === 'number' ? `${(ms / 1000).toFixed(0)} s` : '-')
const dollars = (usd) => (typeof usd === 'number' ? `$${usd.toFixed(2)}` : '-')
const cell = (text) => String(text ?? '').replaceAll('|', '\\|').replaceAll('\n', ' ')

/** The Markdown of every run kept. */
export function agentReport(all) {
  const out = ['# The agent plugin driven by a model: live runs', '']
  out.push(
    'Written by `bench/agent/run.sh`; do not edit. Each scenario is one `claude -p` run with the',
    'plugin of this repository (`--plugin-dir plugins/flowapi`) and flowapi built here',
    '(`FLOWAPI_PLUGIN_FLOWAPI`), in a fresh copy of what it starts from; after it, the runner runs',
    "the judge and `config check` itself and never reads the agent's own table. A model's runs are",
    'not deterministic: the same scenario may take other turns, cost and time, or fail, on another',
    'run. Only a judge that does not pass, or a scenario expectation that does not hold, fails a run.',
    '',
    '| Scenario | Date | flowapi | Model | Turns | Cost | Time | Judge | Expectations |',
    '| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |',
  )
  for (const run of all.runs) {
    const held = run.checks.filter((c) => c.ok).length
    out.push(
      `| ${run.scenario} | ${run.date} | ${run.flowapi} (${run.commit}${run.uncommitted ? ', changed' : ''}) | ${cell(run.model)} | ${run.turns ?? '-'} | ${dollars(run.costUsd)} | ${seconds(run.wallMs)} | ${run.judge?.verdict ?? '-'} | ${held} of ${run.checks.length} |`,
    )
  }
  for (const run of all.runs) {
    out.push('', `## ${run.scenario}`, '', cell(run.about), '')
    out.push(`claude ${run.claude ?? '?'}; budget ${dollars(run.maxBudgetUsd)}; ended ${run.subtype ?? '?'}.`, '')
    out.push('| Judge step | Status | Time |', '| --- | --- | ---: |')
    for (const s of run.judge?.steps ?? []) out.push(`| ${s.step} ${s.name} | ${s.status} | ${seconds(s.duration_ms)} |`)
    const warnings = run.warnings.map((w) => `\`${w.code}\` ${w.seen ? 'seen' : 'NOT seen'}`)
    out.push('', `Final \`config check\`: ${run.config?.errors ?? '?'} errors, ${run.config?.warnings ?? '?'} warnings${warnings.length ? ` (${warnings.join(', ')})` : ''}.`)
    out.push(`Issue codes the agent was shown: ${run.seenCodes.length ? run.seenCodes.map((c) => `\`${c}\``).join(', ') : 'none'}.`, '')
    for (const c of run.checks) out.push(`- ${c.ok ? 'holds' : 'BROKEN'}: ${cell(c.what)}${!c.ok && c.said ? ` (${cell(c.said)})` : ''}`)
  }
  return `${out.join('\n')}\n`
}
