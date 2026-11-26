// The size budgets of the WebAssembly modules the package ships, raw and gzipped (level 9, as a
// CDN serves them at most). Fails when a module in dist/ is over either; run after the build.
//
//   bun scripts/size.ts

import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { gzipSync } from 'node:zlib'

/** `flowapi.wasm`, the generator, in bytes: the module as built on 2026-11-24 (297 KB raw, 120 KB gzipped), then
 * the SDK printer and its runtime added the same day (372 KB, 150 KB: the `sdk` option), then the
 * core's precision passes (propertyNames, enum bounds, contains, if/then/else, oneOf) and the SDK's
 * refusals of header, path and number values (407 KB, 164 KB), then the YAML reader on
 * 2026-11-26 (537 KB, 207 KB: saphyr-parser is most of it; three of four real descriptions in the
 * corpus are YAML, and the package refused them), then the SDK's runtime for bytes and
 * multipart bodies on 2026-11-26 (553 KB, 213 KB: the TypeScript it prints, held as text), and
 * some room for the core to grow. A change that passes one is a change to read. */
const GENERATOR = { raw: 580_000, gzip: 224_000 }

/** `flowapi-sdk.wasm`, the second module: the first with the SDK layer's commands about a config
 * (`summary`, `config draft|schema|check`), as built on 2026-11-26 (864 KB, 323 KB: the config's
 * checker and its draft are most of the difference, with a copy of the core, the TypeScript target
 * and the YAML reader), then the `generate` command for whole SDK repositories on 2026-11-26
 * (1,177 KB, 424 KB: the repository's texts it prints, the client's runtime, scripts, mock server
 * and licenses held as text, and the plan of its writes), then the SDK layer made to the
 * requirements ledger on 2026-11-26 (1,370 KB, 490 KB: every security scheme kind, client
 * credentials, servers and their variables, the Encoding Object, the RFC 9110 retries and
 * redirects, and the README's texts; 32 KB of it is the client runtime every SDK carries), and
 * about 5% room. It is a module of its own so that a user of the types alone (the API's
 * `generate`, the command without a subcommand) never loads it: only the SDK commands do. */
const SDK = { raw: 1_439_000, gzip: 514_000 }

const modules = [
  ['flowapi.wasm', GENERATOR],
  ['flowapi-sdk.wasm', SDK],
] as const
const kb = (n: number): string => `${(n / 1024).toFixed(1)} KiB`
let over = false
for (const [file, budget] of modules) {
  const wasm = readFileSync(join(import.meta.dir, '..', 'dist', file))
  const size = { raw: wasm.length, gzip: gzipSync(wasm, { level: 9 }).length }
  for (const key of ['raw', 'gzip'] as const) {
    const fits = size[key] <= budget[key]
    over ||= !fits
    console.log(`${file} ${key}: ${kb(size[key])} (budget ${kb(budget[key])})${fits ? '' : ' OVER'}`)
  }
}
if (over) process.exit(1)
