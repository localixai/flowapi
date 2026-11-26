// The judge: the checks a generated SDK repository passes before the agent may call it done
// (steps 0-6). Cheap ones first; it stops at the first failure unless
// asked to go on. A step whose script or file is missing is a skip with its reason, never a pass.
// It installs nothing and publishes nothing; the network is reached only by `npx` fetching a tool.

import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { head, run, shellQuote } from './run.mjs'
import { compareTrees } from './tree.mjs'
import { ATTW, FLOWAPI_VERSION, PUBLINT, defaultGenerate, npx } from './tools.mjs'

export const STEPS = [
  { id: 0, name: 'determinism', check: determinism },
  { id: 1, name: 'typecheck', check: typecheck },
  { id: 2, name: 'lint', check: (ctx) => script(ctx, 'scripts/lint') },
  { id: 3, name: 'build', check: (ctx) => script(ctx, 'scripts/build') },
  { id: 4, name: 'dist types', check: distTypes },
  { id: 5, name: 'package', check: packageShape },
  { id: 6, name: 'test', check: (ctx) => script(ctx, 'scripts/test') },
]

/**
 * Runs the steps `ids` on the repository `dir`. Options: `keepGoing`, `generate` (the command
 * template), `timeoutMs` per command, `env`. Resolves to the report.
 */
export async function judge(
  dir,
  { ids, keepGoing = false, generate, timeoutMs, env = process.env } = {},
) {
  const ctx = {
    dir,
    generate: generate ?? defaultGenerate(env),
    timeoutMs,
    env: { ...env, CI: env.CI ?? '1' },
  }
  const steps = []
  let failed = false
  for (const step of STEPS.filter((s) => ids.includes(s.id))) {
    if (failed && !keepGoing) {
      steps.push({
        step: step.id,
        name: step.name,
        command: null,
        status: 'skip',
        reason: 'not run: an earlier step failed',
        duration_ms: 0,
      })
      continue
    }
    const started = Date.now()
    const result = await step.check(ctx)
    const entry = {
      step: step.id,
      name: step.name,
      command: result.command ?? null,
      status: result.status,
    }
    if (result.reason) entry.reason = result.reason
    entry.duration_ms = Date.now() - started
    if (result.output) entry.output = head(result.output)
    steps.push(entry)
    if (result.status === 'fail') failed = true
  }
  const verdict = failed ? 'fail' : steps.every((s) => s.status === 'pass') ? 'pass' : 'incomplete'
  return { dir, verdict, steps }
}

// Step 0: two generations into empty folders, compared byte for byte.
async function determinism(ctx) {
  const command = ctx.generate
  if (!existsSync(join(ctx.dir, 'flowapi-sdk.yml'))) {
    return {
      command,
      status: 'fail',
      reason: 'flowapi-sdk.yml is missing: the judge checks a repository generated from one',
    }
  }
  const outs = [0, 1].map((i) => mkdtempSync(join(tmpdir(), `flowapi-plugin-gen${i}-`)))
  try {
    for (const out of outs) {
      const line = command
        .replaceAll('{out}', shellQuote(out))
        .replaceAll('{flowapi}', flowapiVersion(ctx.dir))
      const done = await run(line, [], {
        cwd: ctx.dir,
        env: ctx.env,
        timeoutMs: ctx.timeoutMs,
        shell: true,
      })
      const failure = failureOf(done)
      if (failure)
        return { command, status: 'fail', reason: `generate: ${failure}`, output: done.output }
    }
    const { files, differences } = compareTrees(outs[0], outs[1])
    if (files === 0)
      return { command, status: 'fail', reason: 'generate wrote no files into {out}' }
    if (differences.length > 0) {
      return {
        command,
        status: 'fail',
        reason: `two runs differ in ${differences.length} of ${files} paths`,
        output: differences.join('\n'),
      }
    }
    return { command, status: 'pass', reason: `${files} files, equal in both runs` }
  } finally {
    for (const out of outs) rmSync(out, { recursive: true, force: true })
  }
}

// Step 1: the sources and tests compile, with the repository's own TypeScript.
async function typecheck(ctx) {
  const command = `npx --no -- tsc --noEmit`
  if (!existsSync(join(ctx.dir, 'tsconfig.json')))
    return { command, status: 'fail', reason: 'tsconfig.json is missing' }
  const missing = needsDependencies(ctx, command)
  if (missing) return missing
  return tool(ctx, command, ['--no', '--', 'tsc', '--noEmit'])
}

// Step 4: the built declarations are valid on their own (skipLibCheck off in that config).
async function distTypes(ctx) {
  const command = `npx --no -- tsc -p tsconfig.dist.json`
  if (!existsSync(join(ctx.dir, 'tsconfig.dist.json')))
    return { command, status: 'skip', reason: 'tsconfig.dist.json is missing' }
  const missing = needsDependencies(ctx, command)
  if (missing) return missing
  return tool(ctx, command, ['--no', '--', 'tsc', '-p', 'tsconfig.dist.json'])
}

// Step 5: the package's exports and types for CommonJS and ESM, on the packed tarball.
async function packageShape(ctx) {
  const commands = [
    [`npx -y ${PUBLINT} --strict`, ['-y', PUBLINT, '--strict']],
    [`npx -y ${ATTW} --pack .`, ['-y', ATTW, '--pack', '.']],
  ]
  const command = commands.map(([line]) => line).join(' && ')
  if (!existsSync(join(ctx.dir, 'package.json')))
    return { command, status: 'fail', reason: 'package.json is missing' }
  for (const [line, args] of commands) {
    const result = await tool(ctx, line, args)
    if (result.status !== 'pass') return { ...result, command }
  }
  return { command, status: 'pass' }
}

async function tool(ctx, command, args) {
  const shell = process.platform === 'win32' // npx is a .cmd there
  const done = await run(npx(ctx.env), args, {
    cwd: ctx.dir,
    env: ctx.env,
    timeoutMs: ctx.timeoutMs,
    shell,
  })
  const failure = failureOf(done)
  return failure
    ? { command, status: 'fail', reason: failure, output: done.output }
    : { command, status: 'pass' }
}

// Steps 2, 3 and 6: the generated repository's own scripts.
async function script(ctx, rel) {
  const command = rel
  if (!existsSync(join(ctx.dir, rel)))
    return { command, status: 'skip', reason: `${rel} is missing` }
  const done = await run(join(ctx.dir, rel), [], {
    cwd: ctx.dir,
    env: ctx.env,
    timeoutMs: ctx.timeoutMs,
  })
  const failure = failureOf(done)
  return failure
    ? { command, status: 'fail', reason: failure, output: done.output }
    : { command, status: 'pass' }
}

function needsDependencies(ctx, command) {
  if (existsSync(join(ctx.dir, 'node_modules'))) return null
  return {
    command,
    status: 'fail',
    reason:
      'node_modules is missing: install the dependencies first (scripts/bootstrap); the judge installs nothing',
  }
}

function failureOf(done) {
  if (done.timedOut) return 'timed out'
  if (done.code === null && !done.signal) return 'could not start'
  if (done.signal) return `ended by ${done.signal}`
  if (done.code !== 0) return `exit code ${done.code}`
  return null
}

/** The flowapi that wrote the repository, from its lock; the pinned one when there is none. */
export function flowapiVersion(dir) {
  try {
    const lock = JSON.parse(readFileSync(join(dir, '.flowapi', 'lock.json'), 'utf8'))
    if (typeof lock.flowapi === 'string' && /^[0-9A-Za-z.+-]+$/.test(lock.flowapi))
      return lock.flowapi
  } catch {
    // no lock yet, or a broken one: the pinned version
  }
  return FLOWAPI_VERSION
}
