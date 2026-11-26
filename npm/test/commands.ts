// The two commands the parity tests hold to each other, built from the same sources: the Rust
// one (target/release/flowapi, built by `buildRust` when stale) and the package's (dist/cli.js,
// after `bun run build`); and GitHub's description, the largest the tests run them on.

import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { join } from 'node:path'

export const npm = join(import.meta.dir, '..')
export const root = join(npm, '..')
const target = process.env.CARGO_TARGET_DIR ?? join(root, 'target')
export const rust = join(target, 'release', 'flowapi')
export const js = join(npm, 'dist', 'cli.js')

export interface Run {
  readonly status: number | null
  readonly stdout: string
  readonly stderr: string
}

export function run(command: string, args: readonly string[], cwd: string): Run {
  const done = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 1 << 28 })
  if (done.error) throw done.error
  return { status: done.status, stdout: done.stdout, stderr: done.stderr }
}

/** Both commands on the same arguments, from the same directory. */
export function both(args: readonly string[], cwd = root): { rust: Run; js: Run } {
  return { rust: run(rust, args, cwd), js: run('node', [js, ...args], cwd) }
}

/** The Rust command, built (cargo does nothing when it is fresh). */
export function buildRust(): void {
  const built = run('cargo', ['build', '--locked', '--release', '-p', 'flowapi-cli'], root)
  if (built.status !== 0) throw new Error(`the Rust command did not build:\n${built.stderr}`)
}

/** GitHub's description: the benchmarks' pinned copy, else the workspace's reference clone. */
const clone = join('.references', 'codegen', 'corpus', 'github', 'descriptions', 'api.github.com')
export const github = [
  join(root, 'bench', 'data', 'github.json'),
  ...['..', join('..', '..')].map((up) => join(root, up, clone, 'api.github.com.json')),
].find((path) => existsSync(path))
