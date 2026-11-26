// The texts a generated SDK shows its users, held to its client: for each repository kept in
// tests/repo/ beside the client kept in tests/client/ under the same name,
// - every TypeScript block of README.md compiles against the client (the package's name read as
//   its src/index.ts; a block that uses `client` without making one is given a `Shop`-like
//   `declare const client`), the headline example first among them;
// - every method line of api.md is a signature the client's method has: `const m: (params…) =>
//   R = client.<resource>.<method>` compiles, with the types api.md says to import;
// - api.md lists as many methods as src/resources/ defines, and README.md names every resource
//   api.md has;
// - no text holds a placeholder: `TODO` or `FIXME`, `"string"` as an example value, a comment
//   standing for arguments (`/* … */`), or a section with nothing under its heading.
// The compiler is the oracles' (conformance/ts), under strict and exactOptionalPropertyTypes.
// Run from the repository root, after `(cd conformance/ts && npm ci)`:
//   node crates/flowapi-sdk/tests/texts.mjs
// Exits 1 when anything fails.
import { execFileSync } from 'node:child_process'
import { cpSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'

const tsc = resolve('conformance/ts/node_modules/.bin/tsc')
const repos = 'crates/flowapi-sdk/tests/repo'
const clients = 'crates/flowapi-sdk/tests/client'
const core = 'crates/flowapi-sdk/src/client/core'
const strict = ['--ignoreConfig', '--strict', '--exactOptionalPropertyTypes', '--target', 'es2022', '--module', 'nodenext', '--lib', 'es2022,dom', '--noEmit', '--pretty', 'false']
const failures = []
const index = "'../src/index.js'"

/** The texts of a generated repository a user reads. */
const TEXTS = ['README.md', 'api.md', 'AGENTS.md', 'CHANGELOG.md', 'package.json']

/** The placeholders no text may hold, each with what it is. */
const PLACEHOLDERS = [
  [/\b(TODO|FIXME)\b/, 'a TODO left in'],
  [/[:=(,[]\s*["']string["']/, '"string" as an example value'],
  [/\/\*\s*(…|\.\.\.)\s*\*\//, 'a comment standing for arguments'],
]

/** Headings with nothing under them before the next heading of the same level or above. */
function emptySections(text) {
  const lines = text.split('\n')
  const out = []
  lines.forEach((line, i) => {
    const h = /^(#{1,6}) /.exec(line)
    if (h === null) return
    const next = lines.slice(i + 1).find((l) => l.trim() !== '')
    const nh = next === undefined ? null : /^(#{1,6}) /.exec(next)
    if (next === undefined || (nh !== null && nh[1].length <= h[1].length)) out.push(line)
  })
  return out
}

const root = mkdtempSync(join(tmpdir(), 'flowapi-texts-'))
for (const name of readdirSync(repos).filter((n) => existsSync(join(clients, n, 'src'))).sort()) {
  const repo = join(repos, name)
  const dir = join(root, name)
  cpSync(join(clients, name, 'src'), join(dir, 'src'), { recursive: true })
  cpSync(core, join(dir, 'src', 'core'), { recursive: true })
  mkdirSync(join(dir, 'docs'))
  writeFileSync(join(dir, 'package.json'), '{"type":"module"}')
  const pkg = JSON.parse(readFileSync(join(repo, 'package.json'), 'utf8')).name
  const client = /export class (\w+)/.exec(readFileSync(join(dir, 'src', 'client.ts'), 'utf8'))[1]
  const ofPackage = (code) => code.replaceAll(`'${pkg}'`, index).replaceAll(`"${pkg}"`, index)
  const readme = readFileSync(join(repo, 'README.md'), 'utf8')
  const api = readFileSync(join(repo, 'api.md'), 'utf8')
  const docs = []

  // README.md: each block a module of its own.
  const blocks = [...readme.matchAll(/```ts\n([\s\S]*?)```/g)].map((m) => ofPackage(m[1]))
  if (blocks.length === 0) failures.push(`${name}: README.md has no TypeScript example`)
  blocks.forEach((code, i) => {
    const prelude = []
    if (!new RegExp(`import \\{[^}]*\\b${client}\\b`).test(code)) prelude.push(`import { ${client} } from ${index}`)
    if (!/\bconst client\b/.test(code)) prelude.push(`declare const client: ${client}`)
    const file = join(dir, 'docs', `readme-${i}.ts`)
    writeFileSync(file, `${prelude.join('\n')}\n${code}export {}\n`)
    docs.push(file)
  })

  // api.md: each method line as a value of the signature it writes.
  const imports = ofPackage(/```ts\n([\s\S]*?)```/.exec(api)?.[1] ?? '')
  const lines = [...api.matchAll(/^- `client\.([\w.]+)\((.*)\): (.+)`  $/gm)]
  const values = new Set(lines.flatMap((m) => [...`${m[2]} ${m[3]}`.matchAll(/typeof (\w+)/g)].map((t) => t[1])))
  const checks = lines.map((m, i) => `export const m${i}: (${m[2]}) => ${m[3]} = client.${m[1]}`)
  const apiFile = join(dir, 'docs', 'api.ts')
  writeFileSync(apiFile, [
    imports.trim(),
    `import { ${client}${values.size > 0 ? `, ${[...values].join(', ')}` : ''} } from ${index}`,
    `declare const client: ${client}`,
    ...checks,
  ].join('\n') + '\n')
  docs.push(apiFile)
  const files = readdirSync(join(dir, 'src'), { recursive: true }).filter((f) => f.endsWith('.ts')).map((f) => join(dir, 'src', f))
  try {
    execFileSync(tsc, [...strict, ...files, ...docs], { encoding: 'utf8' })
  } catch (e) {
    failures.push(`${name}: an example of README.md or a signature of api.md does not compile against the client:\n${e.stdout}`)
  }

  // Every method in api.md, every resource in README.md.
  const resources = readdirSync(join(dir, 'src', 'resources'), { recursive: true }).filter((f) => f.endsWith('.ts'))
  const methods = resources
    .map((f) => readFileSync(join(dir, 'src', 'resources', f), 'utf8'))
    .reduce((n, text) => n + [...text.matchAll(/^ {2}(?!constructor)\w+\(params/gm)].length, 0)
  if (methods !== lines.length) failures.push(`${name}: api.md lists ${lines.length} methods, the client has ${methods}`)
  for (const [, access] of api.matchAll(/^## (client\.[\w.]+)$/gm)) {
    if (!readme.includes(`\`${access}\``)) failures.push(`${name}: README.md does not name the resource ${access}`)
  }

  // No placeholder in any text, nor in the client's doc comments.
  const texts = TEXTS.filter((t) => existsSync(join(repo, t))).map((t) => [t, readFileSync(join(repo, t), 'utf8')])
  for (const f of ['client.ts', ...resources.map((r) => `resources/${r}`)]) {
    texts.push([`src/${f}`, readFileSync(join(dir, 'src', f), 'utf8')])
  }
  for (const [file, text] of texts) {
    for (const [pattern, what] of PLACEHOLDERS) {
      const hit = pattern.exec(text)
      if (hit !== null) failures.push(`${name}/${file}: ${what}: ${JSON.stringify(hit[0])}`)
    }
    if (file.endsWith('.md')) {
      for (const h of emptySections(text)) failures.push(`${name}/${file}: the section "${h}" is empty`)
    }
  }
  console.log(`${name}: ${blocks.length} README examples and ${lines.length} api.md signatures compiled`)
}
rmSync(root, { recursive: true, force: true })
if (failures.length > 0) {
  console.error(failures.join('\n\n'))
  process.exit(1)
}
console.log('the texts hold to their clients')
