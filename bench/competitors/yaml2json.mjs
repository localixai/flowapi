// A YAML description read and written again as JSON, two spaces an indent, so that a generator
// that reads only JSON (flowapi does) is measured on the same document as the others:
// `node yaml2json.mjs <in.yaml> <out.json>`. YAML 1.2's core schema, as the `yaml` package reads
// it by default; aliases are expanded. The same bytes on every run: corpus.tsv pins their sha256.
import { readFileSync, writeFileSync } from 'node:fs'
import { parse } from 'yaml'

const [input, output] = process.argv.slice(2)
if (!input || !output) {
  console.error('usage: node yaml2json.mjs <in.yaml> <out.json>')
  process.exit(2)
}
const doc = parse(readFileSync(input, 'utf8'), { maxAliasCount: -1 })
writeFileSync(output, `${JSON.stringify(doc, null, 2)}\n`)
