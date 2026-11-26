// The live agent runs: each scenario of `scenarios/` run once by `claude -p` with this repository's
// plugin and flowapi, in a fresh copy of what it starts from; then the judge and `config check`
// run by the runner, the scenario's expectations held, and `results/agent.json` and `.md`
// written (the latest run of each scenario kept). Usage: `run.sh [--dry-run] <scenario>... | all`.
// `--dry-run` prints each command and runs nothing. Exit 1 when a run's judge or an expectation
// fails, 2 on wrong usage. The runs cost money: they are the owner's to start.

import { spawn, spawnSync } from 'node:child_process'
import { createWriteStream, existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { agentReport } from './report.mjs'
import { hashes, loadScenarios, setUp, startOf } from './scenarios.mjs'
import { parseStream } from './stream.mjs'
import { verdict, warningsSeen } from './verdict.mjs'

const AGENT = fileURLToPath(new URL('..', import.meta.url))
const ROOT = join(AGENT, '..', '..')
const FLOWAPI = join(ROOT, 'target', 'release', 'flowapi')
const PLUGIN = join(ROOT, 'plugins', 'flowapi')
const RESULTS = join(ROOT, 'bench', 'results')
const TOOLS = ['Bash', 'Read', 'Write', 'Edit', 'Glob', 'Grep', 'Skill']

const quote = (arg) => (/^[\w./:=@+-]+$/.test(arg) ? arg : `'${arg.replaceAll("'", "'\\''")}'`)

/** The arguments of `claude` for one scenario. */
export function claudeArgs(s) {
  return [
    '-p', s.prompt, '--model', s.model, '--plugin-dir', PLUGIN,
    '--permission-mode', 'acceptEdits', '--allowedTools', ...TOOLS,
    '--max-budget-usd', String(s.maxBudgetUsd), '--output-format', 'stream-json', '--verbose',
  ]
}

function output(program, args, cwd = ROOT) {
  const done = spawnSync(program, args, { cwd, encoding: 'utf8', env: { ...process.env, FLOWAPI_PLUGIN_FLOWAPI: FLOWAPI } })
  return done.status === 0 ? done.stdout.trim() : null
}

function json(text) {
  try {
    return JSON.parse(text)
  } catch {
    return null
  }
}

/** `claude` run to its end, its events into `log`; resolves to the wall time. */
function agent(s, dir, log) {
  const started = Date.now()
  return new Promise((resolve, reject) => {
    const child = spawn('claude', claudeArgs(s), {
      cwd: dir,
      env: { ...process.env, FLOWAPI_PLUGIN_FLOWAPI: FLOWAPI },
      stdio: ['ignore', 'pipe', 'inherit'],
    })
    child.stdout.pipe(createWriteStream(log))
    child.on('error', reject)
    child.on('close', () => resolve(Date.now() - started))
  })
}

async function runOne(s) {
  const dir = mkdtempSync(join(tmpdir(), `flowapi-agent-${s.id}-`))
  setUp(s, ROOT, dir)
  const sdk = join(dir, s.sdk)
  const before = hashes(dir, s.expect.unchanged ?? [])
  const logs = join(ROOT, 'bench', 'data', 'agent', s.id)
  mkdirSync(logs, { recursive: true })
  process.stderr.write(`agent: ${s.id} in ${dir}\n`)
  const wallMs = await agent(s, dir, join(logs, 'stream.jsonl'))
  const stream = parseStream(readFileSync(join(logs, 'stream.jsonl'), 'utf8'))
  writeFileSync(join(logs, 'report.txt'), stream.result)
  let installMs = null
  if (existsSync(join(sdk, 'package.json')) && !existsSync(join(sdk, 'node_modules'))) {
    const started = Date.now()
    spawnSync('npm', ['install', '--no-audit', '--no-fund'], { cwd: sdk, stdio: 'ignore' })
    installMs = Date.now() - started
  }
  // The judge exits 1 when a step fails; its report is on stdout either way.
  const judged = spawnSync('node', [join(PLUGIN, 'bin', 'judge.mjs'), sdk, '--steps', '0-6', '--json', '--keep-going'], {
    encoding: 'utf8', env: { ...process.env, FLOWAPI_PLUGIN_FLOWAPI: FLOWAPI },
  })
  const judge = existsSync(sdk) ? json(judged.stdout ?? '') : null
  const checked = spawnSync(FLOWAPI, ['config', 'check', 'flowapi-sdk.yml', '--json'], { cwd: sdk, encoding: 'utf8' })
  const config = json(checked.stdout ?? '')
  return {
    scenario: s.id, about: s.about, date: new Date().toISOString().slice(0, 10),
    flowapi: output(FLOWAPI, ['--version']), commit: output('git', ['rev-parse', '--short', 'HEAD']),
    uncommitted: (output('git', ['status', '--porcelain']) ?? '') !== '',
    claude: output('claude', ['--version']), maxBudgetUsd: s.maxBudgetUsd,
    model: stream.model, turns: stream.turns, costUsd: stream.costUsd, wallMs, subtype: stream.subtype,
    installMs, judge: judge && { verdict: judge.verdict, steps: judge.steps.map(({ output: _, ...step }) => step) },
    config: config && { errors: config.errors, warnings: config.warnings, codes: config.issues.map((i) => i.code) },
    warnings: warningsSeen(config, stream), seenCodes: stream.seenCodes, refusals: stream.refusals,
    checks: verdict({ scenario: s, stream, judge, dir, sdk, before }),
  }
}

function usage(message) {
  process.stderr.write(`${message}\nusage: run.sh [--dry-run] <scenario>... | all\n`)
  process.exit(2)
}

async function main(argv) {
  const dryRun = argv.includes('--dry-run')
  const names = argv.filter((a) => a !== '--dry-run')
  const all = loadScenarios(join(AGENT, 'scenarios'))
  if (names.length === 0) usage('name a scenario, or all')
  const chosen = names.includes('all') ? all : names.map((n) => all.find((s) => s.id === n) ?? usage(`no scenario ${n}`))
  if (dryRun) {
    for (const s of chosen) {
      const start = startOf(s, ROOT)
      const missing = start.present ? '' : `  # ${start.from} is missing: examples/fetch.sh`
      process.stdout.write(`# ${s.id}: a copy of ${start.from} in a new folder:${missing}\n`)
      process.stdout.write(`FLOWAPI_PLUGIN_FLOWAPI=${quote(FLOWAPI)} claude ${claudeArgs(s).map(quote).join(' ')}\n\n`)
    }
    return 0
  }
  if (!existsSync(FLOWAPI)) usage('no target/release/flowapi: cargo build --release -p flowapi-cli')
  const missing = chosen.map((s) => startOf(s, ROOT)).filter((st) => !st.present)
  if (missing.length) usage(`missing: ${missing.map((m) => m.from).join(', ')} (examples/fetch.sh)`)
  const file = join(RESULTS, 'agent.json')
  const kept = existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : { runs: [] }
  let failed = false
  for (const s of chosen) {
    const run = await runOne(s)
    failed ||= run.checks.some((c) => !c.ok)
    kept.runs = [...kept.runs.filter((r) => r.scenario !== s.id), run].sort((a, b) => a.scenario.localeCompare(b.scenario))
    writeFileSync(file, `${JSON.stringify(kept, null, 2)}\n`)
    writeFileSync(join(RESULTS, 'agent.md'), agentReport(kept))
  }
  return failed ? 1 : 0
}

if (process.argv[1] === fileURLToPath(import.meta.url)) process.exitCode = await main(process.argv.slice(2))
