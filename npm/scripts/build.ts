// Builds the package into dist/: its two WebAssembly modules (release, the workspace's profile),
// `flowapi.wasm` from crates/flowapi-wasm (the generator) and `flowapi-sdk.wasm` from
// crates/flowapi-wasm-sdk (the same with the SDK config commands, loaded only by them); over each
// `wasm-opt -O3` when it is installed (never required) and its custom sections dropped (names,
// producers: debugging aids nothing loads); then the TypeScript.
//
//   bun scripts/build.ts

import { spawnSync } from 'node:child_process'
import { chmodSync, copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { stripCustomSections } from './wasm-sections.ts'

const npm = join(import.meta.dir, '..')
const root = join(npm, '..')
const target = process.env.CARGO_TARGET_DIR ?? join(root, 'target')
const dist = join(npm, 'dist')

/** Each module: its crate, the file it ships as, and whether it has the SDK commands. Built one
 * crate per run: built together, cargo would turn on flowapi-wasm's `sdk` for both. */
const modules = [
  { crate: 'flowapi-wasm', file: 'flowapi.wasm', sdk: false },
  { crate: 'flowapi-wasm-sdk', file: 'flowapi-sdk.wasm', sdk: true },
] as const

function run(command: string, args: readonly string[], cwd: string): void {
  const done = spawnSync(command, args, { cwd, stdio: 'inherit' })
  if (done.error) throw done.error
  if (done.status !== 0) process.exit(done.status ?? 1)
}

function has(command: string): boolean {
  return spawnSync(command, ['--version'], { stdio: 'ignore' }).status === 0
}

rmSync(dist, { recursive: true, force: true })
mkdirSync(dist, { recursive: true })
const optimize = has('wasm-opt')
for (const { crate, file, sdk } of modules) {
  const args = ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown', '-p', crate]
  run('cargo', args, root)
  const release = join(target, 'wasm32-unknown-unknown', 'release')
  const built = join(release, `${crate.replaceAll('-', '_')}.wasm`)
  const wasm = join(dist, file)
  // wasm-opt reads the features rustc used from the `target_features` section: it runs first.
  copyFileSync(built, wasm)
  if (optimize) run('wasm-opt', ['-O3', wasm, '-o', wasm], npm)
  const bytes = stripCustomSections(readFileSync(wasm))
  // The generator's module never holds the SDK commands, and the second module always does.
  const compiled = new WebAssembly.Module(new Uint8Array(bytes))
  const exports = WebAssembly.Module.exports(compiled).map((e) => e.name)
  if (exports.includes('flowapi_sdk') !== sdk) {
    console.error(`${file}: ${sdk ? 'no' : 'an'} flowapi_sdk export (${exports.join(', ')})`)
    process.exit(1)
  }
  writeFileSync(wasm, bytes, { mode: 0o644 })
  chmodSync(wasm, 0o644)
}
run(join(npm, 'node_modules', '.bin', 'tsc'), ['-p', 'tsconfig.build.json'], npm)
chmodSync(join(dist, 'cli.js'), 0o755)
