// Comparing two generated trees byte for byte: the judge's determinism step.

import { lstatSync, readFileSync, readdirSync, readlinkSync } from 'node:fs'
import { join } from 'node:path'

/** Every entry under `root` as `relative path → { kind, bytes | target }`, paths with `/`. */
export function listTree(root) {
  const entries = new Map()
  const walk = (dir, prefix) => {
    for (const name of readdirSync(dir).sort()) {
      const full = join(dir, name)
      const rel = prefix ? `${prefix}/${name}` : name
      const stat = lstatSync(full)
      if (stat.isDirectory()) walk(full, rel)
      else if (stat.isSymbolicLink()) entries.set(rel, { kind: 'link', target: readlinkSync(full) })
      else entries.set(rel, { kind: 'file', bytes: readFileSync(full) })
    }
  }
  walk(root, '')
  return entries
}

/** The differences between two trees, one line each, sorted; empty when they are equal. */
export function compareTrees(first, second) {
  const a = listTree(first)
  const b = listTree(second)
  const differences = []
  for (const [rel, entry] of a) {
    const other = b.get(rel)
    if (!other) differences.push(`only in the first run: ${rel}`)
    else if (!sameEntry(entry, other)) differences.push(`differs: ${rel}`)
  }
  for (const rel of b.keys()) if (!a.has(rel)) differences.push(`only in the second run: ${rel}`)
  return { files: a.size, differences: differences.sort() }
}

function sameEntry(x, y) {
  if (x.kind !== y.kind) return false
  if (x.kind === 'link') return x.target === y.target
  return x.bytes.equals(y.bytes)
}
