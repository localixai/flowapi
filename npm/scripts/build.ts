// Builds the package into dist/: the WebAssembly module from crates/flowapi-wasm (release, the
// workspace's profile), `wasm-opt -O3` over it when it is installed (never required), its custom
// sections dropped (names, producers: debugging aids nothing loads), then the TypeScript.
//
//   bun scripts/build.ts

import { spawnSync } from 'node:child_process'
import { chmodSync, copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { stripCustomSections } from './wasm-sections.ts'

const npm = join(import.meta.dir, '..')
const root = join(npm, '..')
const target = process.env.CARGO_TARGET_DIR ?? join(root, 'target')
const built = join(target, 'wasm32-unknown-unknown', 'release', 'flowapi_wasm.wasm')
const dist = join(npm, 'dist')
const wasm = join(dist, 'flowapi.wasm')

function run(command: string, args: readonly string[], cwd: string): void {
  const done = spawnSync(command, args, { cwd, stdio: 'inherit' })
  if (done.error) throw done.error
  if (done.status !== 0) process.exit(done.status ?? 1)
}

function has(command: string): boolean {
  return spawnSync(command, ['--version'], { stdio: 'ignore' }).status === 0
}

run(
  'cargo',
  ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown', '-p', 'flowapi-wasm'],
  root,
)
rmSync(dist, { recursive: true, force: true })
mkdirSync(dist, { recursive: true })
// wasm-opt reads the features rustc used from the `target_features` section: it runs first.
copyFileSync(built, wasm)
if (has('wasm-opt')) run('wasm-opt', ['-O3', wasm, '-o', wasm], npm)
writeFileSync(wasm, stripCustomSections(readFileSync(wasm)), { mode: 0o644 })
chmodSync(wasm, 0o644)
run(join(npm, 'node_modules', '.bin', 'tsc'), ['-p', 'tsconfig.build.json'], npm)
chmodSync(join(dist, 'cli.js'), 0o755)
