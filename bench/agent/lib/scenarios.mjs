// The scenarios of `scenarios/*.json`, read and held to their shape, and the folder each run
// starts in: a description copied alone (the agent makes the SDK from it) or a committed SDK
// repository copied whole (the agent changes it), npm's files and build output left out.

import { createHash } from 'node:crypto'
import { cpSync, copyFileSync, existsSync, readFileSync, readdirSync } from 'node:fs'
import { basename, join } from 'node:path'

const KEYS = ['id', 'about', 'start', 'sdk', 'prompt', 'model', 'maxBudgetUsd', 'expect']
const EXPECT = ['warningsSeen', 'reportMatches', 'configMatches', 'fileMatches', 'unchanged', 'refused']

/** Every scenario, by file name; an error names the file and what is wrong. */
export function loadScenarios(dir) {
  return readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .sort()
    .map((name) => checkScenario(JSON.parse(readFileSync(join(dir, name), 'utf8')), name))
}

/** A scenario as read, or an error saying what in it is wrong. */
export function checkScenario(s, name) {
  const wrong = (what) => {
    throw new Error(`scenarios/${name}: ${what}`)
  }
  for (const key of Object.keys(s)) if (!KEYS.includes(key)) wrong(`unknown key ${key}`)
  for (const key of KEYS) if (s[key] === undefined) wrong(`${key} is missing`)
  if (`${s.id}.json` !== name) wrong(`id ${s.id} is not the file's name`)
  const start = s.start
  if (typeof start.repository !== 'string' && !(typeof start.description === 'string' && typeof start.as === 'string'))
    wrong('start is { description, as } or { repository }')
  if (!(s.maxBudgetUsd > 0)) wrong('maxBudgetUsd is a positive number of dollars')
  for (const key of Object.keys(s.expect)) if (!EXPECT.includes(key)) wrong(`unknown expect.${key}`)
  return s
}

/** What the run starts from, relative to the repository's root, and whether it is there. */
export function startOf(s, root) {
  const from = s.start.repository ?? s.start.description
  return { from, present: existsSync(join(root, from)) }
}

const LEFT_OUT = new Set(['node_modules', 'dist', 'package-lock.json'])

/** Copies what the scenario starts from into the empty folder `dir`. */
export function setUp(s, root, dir) {
  if (s.start.repository) {
    cpSync(join(root, s.start.repository), dir, {
      recursive: true,
      filter: (path) => !LEFT_OUT.has(basename(path)),
    })
  } else {
    copyFileSync(join(root, s.start.description), join(dir, s.start.as))
  }
}

/** The sha256 of each file, `null` when it is missing. */
export function hashes(dir, paths) {
  const out = {}
  for (const path of paths) {
    const file = join(dir, path)
    out[path] = existsSync(file) ? createHash('sha256').update(readFileSync(file)).digest('hex') : null
  }
  return out
}
