#!/usr/bin/env node
// The `flowapi` skill's references, copied from flowapi's own docs. The docs stay where they are
// (the README, the npm package's README, the commands' help) and are the source of truth; an
// installed plugin is a copy of this folder alone and cannot read a file outside it (Agent Plugins
// core 1.1.0 §4.1), so the skill carries copies. Each starts with one line naming its source.
// Relative links are made absolute on GitHub so they still work in the copy.
//
//   node scripts/docs.mjs           write skills/flowapi/references/*.md
//   node scripts/docs.mjs --check   write nothing; exit 1 and name each reference that differs

import { readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { join, posix } from 'node:path'
import { fileURLToPath } from 'node:url'

/** This plugin's folder, and the repository it sits in. */
export const PLUGIN = fileURLToPath(new URL('..', import.meta.url))
export const REPO = join(PLUGIN, '..', '..')
export const OUT = join(PLUGIN, 'skills', 'flowapi', 'references')
const BLOB = 'https://github.com/localixai/flowapi/blob/main/'

/** Each reference: its file name and the parts it is made of, in order. */
export const REFERENCES = [
  { name: 'use.md', parts: [{ kind: 'section', path: 'README.md', heading: '## Use' }] },
  {
    name: 'conformance.md',
    parts: [{ kind: 'section', path: 'README.md', heading: '## How it holds to the specification' }],
  },
  { name: 'npm.md', parts: [{ kind: 'file', path: 'npm/README.md' }] },
  {
    name: 'cli.md',
    title: '# The `flowapi` command',
    parts: [
      { kind: 'help', path: 'crates/flowapi-cli/src/args.rs', heading: '## The Rust command' },
      { kind: 'help', path: 'npm/src/cli.ts', heading: '## The npm command' },
    ],
  },
]

/** Every reference's text, by file name. Throws when a source is not where it is expected. */
export function render(repo = REPO) {
  const out = new Map()
  for (const ref of REFERENCES) {
    const sources = ref.parts.map((part) => part.path)
    const header = `<!-- generated from ${[...new Set(sources)].join(' and ')} by plugins/flowapi/scripts/docs.mjs: edit the source, not this file -->`
    const body = ref.parts.map((part) => renderPart(repo, part))
    const blocks = [header, ...(ref.title ? [ref.title] : []), ...body]
    out.set(ref.name, `${blocks.join('\n\n')}\n`)
  }
  return out
}

function renderPart(repo, part) {
  const text = readFileSync(join(repo, part.path), 'utf8').replace(/\r\n/g, '\n')
  if (part.kind === 'file') return absoluteLinks(text.trimEnd(), part.path)
  if (part.kind === 'section') return absoluteLinks(section(text, part), part.path)
  return `${part.heading}\n\n\`\`\`text\n${helpText(text, part.path)}\n\`\`\``
}

/** The section that starts at `heading`, up to the next heading of the same level. */
function section(text, { path, heading }) {
  const lines = text.split('\n')
  const start = lines.indexOf(heading)
  if (start < 0) throw new Error(`${path}: no line "${heading}"`)
  const level = heading.indexOf(' ')
  const prefix = `${'#'.repeat(level)} `
  let end = lines.findIndex((line, i) => i > start && line.startsWith(prefix))
  if (end < 0) end = lines.length
  return lines.slice(start, end).join('\n').trimEnd()
}

/** The help text a command prints: the Rust `HELP` string, or the TypeScript template literal. */
function helpText(text, path) {
  const rust = /const HELP: &str = "\\\n([\s\S]*?)";\n/.exec(text)
  if (rust) {
    if (rust[1].includes('\\')) throw new Error(`${path}: HELP holds an escape; read it by hand`)
    return rust[1]
  }
  const ts = /const HELP = `([\s\S]*?)`\n/.exec(text)
  if (!ts) throw new Error(`${path}: no HELP text`)
  if (/\\[^`]|\$\{/.test(ts[1])) throw new Error(`${path}: HELP holds more than \\\` escapes`)
  return ts[1].replaceAll('\\`', '`')
}

/** Markdown links relative to `from` (and bare `#anchor`s) as absolute links on GitHub. */
function absoluteLinks(text, from) {
  return text.replace(/\]\(([^)\s]+)\)/g, (whole, target) => {
    if (/^[a-z][a-z0-9+.-]*:/i.test(target)) return whole
    if (target.startsWith('#')) return `](${BLOB}${from}${target})`
    return `](${BLOB}${posix.normalize(posix.join(posix.dirname(from), target))})`
  })
}

function main(argv) {
  const check = argv.includes('--check')
  const wanted = render()
  const stale = []
  for (const [name, text] of wanted) {
    const file = join(OUT, name)
    let current = null
    try {
      current = readFileSync(file, 'utf8')
    } catch {
      current = null
    }
    if (current === text) continue
    stale.push(name)
    if (!check) writeFileSync(file, text)
  }
  const extra = readdirSync(OUT).filter((name) => !wanted.has(name))
  if (check) {
    for (const name of stale) process.stderr.write(`stale: ${join('skills/flowapi/references', name)}\n`)
    for (const name of extra) process.stderr.write(`not generated: ${join('skills/flowapi/references', name)}\n`)
    if (stale.length + extra.length > 0) {
      process.stderr.write('run `node scripts/docs.mjs` in plugins/flowapi and commit the result\n')
      process.exitCode = 1
    }
  } else {
    for (const name of stale) process.stdout.write(`wrote ${join('skills/flowapi/references', name)}\n`)
    for (const name of extra) process.stderr.write(`not generated, remove it: ${name}\n`)
    if (extra.length > 0) process.exitCode = 1
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) main(process.argv.slice(2))
