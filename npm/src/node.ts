// What the command reads, for Node, Bun and Deno: a description and every file its references
// reach by a relative path, read as `crates/flowapi-cli/src/load.rs` reads them (the same URIs
// in the same order), so the output is the command's byte for byte. A YAML file is read by the
// module as the command reads it (`crates/flowapi-yaml`): which file is YAML, the JSON it is read
// into and the YAML line a diagnostic names are decided there, never here.

import { readFile, stat } from 'node:fs/promises'
import { dirname, join, normalize } from 'node:path'
import { load, type Source } from './index.ts'
import type { Document, Options, Result } from './types.ts'

/** A file the package cannot read. */
export class ReadError extends Error {
  override readonly name = 'ReadError'
}

/**
 * The description at `path` and every file its `$ref`s and `$schema`s name by a relative path,
 * the description first. A file a reference names that does not exist is left to the generator,
 * which reports the reference it cannot resolve.
 */
export async function readDescription(path: string, source?: Source): Promise<Document[]> {
  const flowapi = await load(source)
  const out: Document[] = []
  const queue = [path]
  for (let next = queue.pop(); next !== undefined; next = queue.pop()) {
    const uri = next
    if (out.some((d) => d.uri === uri)) continue
    const text = await read(uri)
    for (const reference of flowapi.references(text, uri)) {
      const hash = reference.indexOf('#')
      const resource = hash === -1 ? reference : reference.slice(0, hash)
      if (resource === '' || resource.includes('://') || resource.startsWith('urn:')) continue
      const reached = normalize(join(dirname(uri), resource))
      if (await isFile(reached)) queue.push(reached)
    }
    out.push({ uri, text })
  }
  return out
}

/** Generates TypeScript from the description at `path` and the files its references reach. */
export async function generateFile(path: string, options?: Options): Promise<Result> {
  const flowapi = await load()
  return flowapi.generate(await readDescription(path), options)
}

async function read(path: string): Promise<string> {
  try {
    return await readFile(path, 'utf8')
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error)
    throw new ReadError(`cannot read ${path}: ${reason}`, { cause: error })
  }
}

async function isFile(path: string): Promise<boolean> {
  try {
    return (await stat(path)).isFile()
  } catch {
    // Not there, or not readable: the generator names the reference it cannot resolve.
    return false
  }
}
