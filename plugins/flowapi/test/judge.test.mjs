// The judge on a stand-in SDK repository: its scripts are tiny shell scripts that pass or fail,
// `npx` is a stand-in that records its arguments (FLOWAPI_PLUGIN_NPX), and generate is a small Node
// script that writes the same files every run or different ones. Nothing reaches the network.
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'

const JUDGE = new URL('../bin/judge.mjs', import.meta.url).pathname
const posix = process.platform !== 'win32'

const GENERATOR = `import { mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
const [out, mode] = process.argv.slice(2)
mkdirSync(join(out, 'src'), { recursive: true })
writeFileSync(join(out, 'package.json'), '{"name":"@acme/sdk"}\\n')
writeFileSync(join(out, 'src', 'client.ts'), mode === 'random' ? String(Math.random()) : 'export {}\\n')
if (mode === 'fail') { console.error('config check: 2 errors'); process.exit(3) }
`

function executable(file, text) {
  writeFileSync(file, text)
  chmodSync(file, 0o755)
}

/** A stand-in SDK repository; `scripts` maps a script name to its exit code (absent: missing). */
function sdk({
  scripts = { lint: 0, build: 0, test: 0 },
  files = ['flowapi-sdk.yml', 'tsconfig.json', 'tsconfig.dist.json', 'package.json'],
  deps = true,
} = {}) {
  const dir = mkdtempSync(join(tmpdir(), 'flowapi-plugin-judge-'))
  for (const file of files) writeFileSync(join(dir, file), '{}\n')
  if (deps) mkdirSync(join(dir, 'node_modules'))
  mkdirSync(join(dir, 'scripts'))
  for (const [name, code] of Object.entries(scripts)) {
    executable(
      join(dir, 'scripts', name),
      `#!/bin/sh\necho "${name} line 1"\necho "${name} trouble" >&2\nexit ${code}\n`,
    )
  }
  const tools = mkdtempSync(join(tmpdir(), 'flowapi-plugin-tools-'))
  const gen = join(tools, 'gen.mjs')
  writeFileSync(gen, GENERATOR)
  const trace = join(tools, 'npx.trace')
  // Fails when its arguments contain $NPX_FAIL.
  executable(
    join(tools, 'npx'),
    `#!/bin/sh\necho "$*" >> '${trace}'\ncase " $* " in *" $NPX_FAIL "*) echo "npx $1 failed"; exit 1;; esac\nexit 0\n`,
  )
  const calls = () => (existsSync(trace) ? readFileSync(trace, 'utf8').trim().split('\n') : [])
  return { dir, gen, calls, npx: join(tools, 'npx') }
}

function judge(world, args, { mode = 'same', fail } = {}) {
  const generate = `"${process.execPath}" "${world.gen}" {out} ${mode}`
  const env = { ...process.env, FLOWAPI_PLUGIN_NPX: world.npx, NPX_FAIL: fail ?? '' }
  const run = spawnSync(process.execPath, [JUDGE, world.dir, '--generate', generate, ...args], {
    env,
    encoding: 'utf8',
  })
  return { ...run, report: args.includes('--json') ? JSON.parse(run.stdout) : null }
}

const statuses = (report) => report.steps.map((s) => `${s.step}:${s.status}`).join(' ')

test(
  'every step passes: verdict pass, exit code 0, the tools called as written',
  { skip: !posix },
  () => {
    const world = sdk()
    const { status, report } = judge(world, ['--json'])
    assert.equal(status, 0)
    assert.equal(report.verdict, 'pass')
    assert.equal(statuses(report), '0:pass 1:pass 2:pass 3:pass 4:pass 5:pass 6:pass')
    assert.match(report.steps[0].reason, /2 files, equal in both runs/)
    for (const step of report.steps) assert.equal(typeof step.duration_ms, 'number')
    assert.deepEqual(world.calls(), [
      '--no -- tsc --noEmit',
      '--no -- tsc -p tsconfig.dist.json',
      '-y publint@0.3.25 --strict',
      '-y @arethetypeswrong/cli@0.18.5 --pack .',
    ])
  },
)

test('determinism: two runs that differ fail step 0 and name the file', { skip: !posix }, () => {
  const { status, report } = judge(sdk(), ['--json'], { mode: 'random' })
  assert.equal(status, 1)
  assert.equal(report.verdict, 'fail')
  assert.equal(report.steps[0].status, 'fail')
  assert.match(report.steps[0].reason, /two runs differ in 1 of 2 paths/)
  assert.equal(report.steps[0].output, 'differs: src/client.ts')
  assert.ok(
    report.steps.slice(1).every((s) => s.status === 'skip' && /earlier step failed/.test(s.reason)),
  )
})

test('determinism: a failing generate is a failure with its output', { skip: !posix }, () => {
  const { report } = judge(sdk(), ['--json', '--steps', '0'], { mode: 'fail' })
  assert.equal(report.steps[0].reason, 'generate: exit code 3')
  assert.match(report.steps[0].output, /config check: 2 errors/)
})

test(
  'a failing script stops the judge; its output is kept, stdout and stderr',
  { skip: !posix },
  () => {
    const { status, report } = judge(sdk({ scripts: { lint: 0, build: 4, test: 0 } }), ['--json'])
    assert.equal(status, 1)
    assert.equal(statuses(report), '0:pass 1:pass 2:pass 3:fail 4:skip 5:skip 6:skip')
    const build = report.steps[3]
    assert.equal(build.reason, 'exit code 4')
    assert.equal(build.command, 'scripts/build')
    assert.match(build.output, /build line 1/)
    assert.match(build.output, /build trouble/)
  },
)

test('--keep-going runs the rest after a failure', { skip: !posix }, () => {
  const { report } = judge(
    sdk({ scripts: { lint: 1, build: 0, test: 0 } }),
    ['--json', '--keep-going'],
    { fail: 'publint@0.3.25' },
  )
  assert.equal(statuses(report), '0:pass 1:pass 2:fail 3:pass 4:pass 5:fail 6:pass')
  assert.match(report.steps[5].output, /npx -y failed/)
})

test(
  'a missing script is a skip with its reason, and the verdict is incomplete',
  { skip: !posix },
  () => {
    const { status, report } = judge(sdk({ scripts: { build: 0 } }), ['--json'])
    assert.equal(status, 1)
    assert.equal(report.verdict, 'incomplete')
    assert.equal(statuses(report), '0:pass 1:pass 2:skip 3:pass 4:pass 5:pass 6:skip')
    assert.equal(report.steps[2].reason, 'scripts/lint is missing')
    assert.equal(report.steps[6].reason, 'scripts/test is missing')
  },
)

test('missing config, tsconfig and dependencies fail with what to do', { skip: !posix }, () => {
  const noConfig = judge(sdk({ files: ['package.json'] }), ['--json', '--keep-going'])
  assert.match(noConfig.report.steps[0].reason, /flowapi-sdk\.yml is missing/)
  assert.match(noConfig.report.steps[1].reason, /tsconfig\.json is missing/)
  assert.equal(noConfig.report.steps[4].reason, 'tsconfig.dist.json is missing')
  const noDeps = judge(sdk({ deps: false }), ['--json', '--steps', '1'])
  assert.match(noDeps.report.steps[0].reason, /node_modules is missing.*installs nothing/)
})

test('--steps picks steps; the table names the failure and its command', { skip: !posix }, () => {
  const world = sdk({ scripts: { lint: 0, build: 0, test: 2 } })
  const picked = judge(world, ['--json', '--steps', '2,6'])
  assert.equal(statuses(picked.report), '2:pass 6:fail')
  const text = judge(world, ['--steps', '6'])
  assert.equal(text.status, 1)
  assert.match(text.stdout, /^6 +test +fail .*exit code 2$/m)
  assert.match(text.stdout, /\$ scripts\/test/)
  assert.match(text.stdout, /\| test trouble/)
  assert.match(text.stdout, /verdict: fail/)
})

test('wrong usage exits 2 and runs nothing', { skip: !posix }, () => {
  const world = sdk()
  for (const args of [
    ['--steps', '7'],
    ['--steps', '3-1'],
    ['--steps', 'x'],
    ['--nope'],
    ['--timeout', '0'],
  ]) {
    const run = judge(world, args)
    assert.equal(run.status, 2, args.join(' '))
    assert.equal(run.stdout, '')
  }
  const noOut = spawnSync(process.execPath, [JUDGE, world.dir, '--generate', 'true'], {
    encoding: 'utf8',
  })
  assert.equal(noOut.status, 2)
  assert.match(noOut.stderr, /must contain \{out\}/)
  assert.deepEqual(world.calls(), [])
})

test('a time limit ends a script that hangs', { skip: !posix }, () => {
  const world = sdk()
  executable(join(world.dir, 'scripts', 'lint'), '#!/bin/sh\nsleep 30\n')
  const started = Date.now()
  const { report } = judge(world, ['--json', '--steps', '2', '--timeout', '0.5'])
  assert.equal(report.steps[0].reason, 'timed out')
  assert.ok(Date.now() - started < 10_000)
})
