// The disk around `flowapi generate`, as the Rust command reads and writes it
// (crates/flowapi-cli/src/generate/disk.rs and apply.rs): what is at each path the plan asks about
// (a file's SHA-256 by node:crypto, a path through a symbolic link refused), and a plan made true,
// all or nothing as far as a file system allows. What to write, keep or delete is decided in the
// module (`flowapi_sdk::plan`), never here; the paths it names are inside the folder already.

import { createHash } from 'node:crypto'
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmdirSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from 'node:fs'
import { join, parent } from './paths.ts'
import type { Change, Disk } from './types.ts'

/** A refusal: the message the command prints before it exits with 1. */
export class Refused extends Error {}

export function reason(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** `path` joined to `dir`, refused when a part of it on the way is a symbolic link. */
function inside(dir: string, path: string): string {
  let at = dir
  for (const part of path.split('/')) {
    at = join(at, part)
    let link = false
    try {
      link = lstatSync(at).isSymbolicLink()
    } catch {
      // Not there (or not readable): no link to refuse, as `symlink_metadata(..).is_ok_and(..)`.
    }
    if (link) {
      throw new Refused(
        `flowapi: ${at} is a symbolic link: generate writes no file through one, nothing was written`,
      )
    }
  }
  return at
}

/** What is at each path inside `dir`, for the plan; a path it cannot read is an `error`. */
export function readDisk(dir: string, paths: readonly string[]): (Disk & { path: string })[] {
  return paths.map((path) => {
    try {
      return { path, ...disk(dir, path) }
    } catch (error) {
      if (!(error instanceof Refused)) throw error
      return { path, kind: 'error', message: error.message }
    }
  })
}

function disk(dir: string, path: string): Disk {
  const at = inside(dir, path)
  let stat
  try {
    stat = lstatSync(at)
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return { kind: 'absent' }
    throw new Refused(`flowapi: cannot read ${at}: ${reason(error)}`)
  }
  if (!stat.isFile()) return { kind: 'other' }
  let bytes
  try {
    bytes = readFileSync(at)
  } catch (error) {
    throw new Refused(`flowapi: cannot read ${at}: ${reason(error)}`)
  }
  const sha256 = createHash('sha256').update(bytes).digest('hex')
  return { kind: 'file', sha256, executable: executable(at) }
}

/** Whether a file is run as a program; on Windows every file is, as the Rust command says. */
function executable(path: string): boolean {
  if (process.platform === 'win32') return true
  try {
    return (statSync(path).mode & 0o111) !== 0
  } catch {
    // Unreadable now: not executable, as `metadata(..).is_ok_and(..)` says.
    return false
  }
}

function setExecutable(path: string): void {
  if (process.platform === 'win32') return
  let mode
  try {
    mode = statSync(path).mode
  } catch (error) {
    throw new Refused(`flowapi: cannot read ${path}: ${reason(error)}`)
  }
  try {
    chmodSync(path, mode | 0o755)
  } catch (error) {
    throw new Refused(`flowapi: cannot make ${path} executable: ${reason(error)}`)
  }
}

/** A file written under its temporary name. */
interface Staged {
  readonly temp: string
  readonly target: string
}

/**
 * The plan's changes made in `dir`: every write first staged beside its target as
 * `.<name>.flowapi-new` (a failure there removes them and the folders made for them), then
 * renamed over it, the lock last; then the deletions, with the folders they leave empty, and the
 * modes. `text` gives each write's text.
 */
export function apply(dir: string, changes: readonly Change[], text: (path: string) => string) {
  const made: string[] = []
  const staged: Staged[] = []
  try {
    stage(dir, changes, text, made, staged)
  } catch (error) {
    if (!(error instanceof Refused)) throw error
    for (const s of staged) attempt(() => unlinkSync(s.temp))
    for (const folder of [...made].reverse()) attempt(() => rmdirSync(folder))
    throw new Refused(`${error.message}; nothing was written`)
  }
  for (const s of staged) {
    try {
      renameSync(s.temp, s.target)
    } catch (error) {
      throw new Refused(`flowapi: cannot write ${s.target}: ${reason(error)}`)
    }
  }
  for (const change of changes) {
    if (change.change === 'delete') {
      const at = inside(dir, change.path)
      try {
        unlinkSync(at)
      } catch (error) {
        throw new Refused(`flowapi: cannot delete ${at}: ${reason(error)}`)
      }
      prune(dir, change.path)
    } else if (change.change === 'mode') setExecutable(inside(dir, change.path))
  }
}

function stage(
  dir: string,
  changes: readonly Change[],
  text: (path: string) => string,
  made: string[],
  staged: Staged[],
): void {
  for (const change of changes) {
    if (change.change !== 'write') continue
    const target = inside(dir, change.path)
    const folder = parent(target) || dir
    makeFolders(folder, made)
    const name = target.slice(target.lastIndexOf('/') + 1)
    const temp = join(folder, `.${name}.flowapi-new`)
    try {
      writeFileSync(temp, text(change.path))
    } catch (error) {
      throw new Refused(`flowapi: cannot write ${temp}: ${reason(error)}`)
    }
    staged.push({ temp, target })
    if (change.executable) setExecutable(temp)
  }
}

/** A folder and those above it that are missing, made; each one made is recorded. */
function makeFolders(folder: string, made: string[]): void {
  const missing: string[] = []
  for (let at = folder; at !== '' && !existsSync(at); at = parent(at)) missing.push(at)
  for (const f of missing.reverse()) {
    try {
      mkdirSync(f)
    } catch (error) {
      throw new Refused(`flowapi: cannot make ${f}: ${reason(error)}`)
    }
    made.push(f)
  }
}

/** The folders a deleted file leaves empty, removed up to `dir`. */
function prune(dir: string, path: string): void {
  const parts = path.split('/')
  for (let n = parts.length - 1; n > 0; n--) {
    try {
      rmdirSync(join(dir, parts.slice(0, n).join('/')))
    } catch {
      // Not empty (or not removable): the folders above it are not empty either.
      return
    }
  }
}

/** A cleanup step whose failure leaves nothing worse than it was. */
function attempt(step: () => void): void {
  try {
    step()
  } catch {
    // Best effort, as the Rust command drops these errors: the refusal is what is reported.
  }
}
