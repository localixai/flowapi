// Writes conformance/suites/style-examples.json: the table "Style Examples" of OpenAPI 3.2.1
// (conformance/specs/openapi/3.2.1.md, section "Style Examples" under the Parameter Object) as
// data, each row a style and an explode with the serialization of each of the section's four
// values. A cell the spec marks _n/a_ (undefined behaviour) is left out; _empty_ is "".
// Run from the repository root: node conformance/sdk/extract.mjs [--check]
import { readFileSync, writeFileSync } from 'node:fs'

const spec = 'conformance/specs/openapi/3.2.1.md'
const target = 'conformance/suites/style-examples.json'
const lines = readFileSync(spec, 'utf8').split('\n')
const heading = lines.findIndex((l) => l === '#### Style Examples')
if (heading < 0) throw new Error(`${spec}: no "#### Style Examples" heading`)
const header = lines.findIndex((l, i) => i > heading && l.startsWith('| [`style`]'))
// The values the section assumes for a parameter named `color`, as its `dataValue`s.
const values = { undefined: null, string: 'blue', array: ['blue', 'black', 'brown'], object: { R: 100, G: 200, B: 150 } }
const columns = ['undefined', 'string', 'array', 'object']
const rows = []
for (let i = header + 2; i < lines.length && lines[i].startsWith('|'); i++) {
  const cells = lines[i].split('|').slice(1, -1).map((c) => c.trim())
  const [style, explode, ...rest] = cells
  const examples = []
  rest.forEach((cell, k) => {
    if (cell === '_n/a_') return
    examples.push({ value: columns[k], serialized: cell === '_empty_' ? '' : cell })
  })
  rows.push({ line: i + 1, style, explode: explode === '_n/a_' ? null : explode === 'true', examples })
}
const out = JSON.stringify({
  about: `The table "Style Examples" of ${spec} (OpenAPI 3.2.1, Parameter Object, "Style Examples"; https://spec.openapis.org/oas/v3.2.1.html#style-examples), written by conformance/sdk/extract.mjs; do not edit. The parameter is named color; each example's value is one of values, serialized as the spec shows it.`,
  source: spec,
  heading: heading + 1,
  values,
  rows,
}, null, 1) + '\n'
if (process.argv.includes('--check')) {
  if (readFileSync(target, 'utf8') !== out) throw new Error(`${target} is stale: node conformance/sdk/extract.mjs`)
} else {
  writeFileSync(target, out)
}
