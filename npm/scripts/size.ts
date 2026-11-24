// The size budget of the WebAssembly module the package ships, raw and gzipped (level 9, as a
// CDN serves it at most). Fails when dist/flowapi.wasm is over either; run after the build.
//
//   bun scripts/size.ts

import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { gzipSync } from 'node:zlib'

/** The budgets in bytes: the module as built on 2026-11-24 (297 KB raw, 120 KB gzipped) and
 * some room for the core to grow. A change that passes one is a change to read. */
const BUDGET = { raw: 360_000, gzip: 145_000 }

const wasm = readFileSync(join(import.meta.dir, '..', 'dist', 'flowapi.wasm'))
const size = { raw: wasm.length, gzip: gzipSync(wasm, { level: 9 }).length }
const kb = (n: number): string => `${(n / 1024).toFixed(1)} KiB`
let over = false
for (const key of ['raw', 'gzip'] as const) {
  const fits = size[key] <= BUDGET[key]
  over ||= !fits
  console.log(`flowapi.wasm ${key}: ${kb(size[key])} (budget ${kb(BUDGET[key])})${fits ? '' : ' OVER'}`)
}
if (over) process.exit(1)
