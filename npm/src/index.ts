// flowapi for JavaScript: the generator of the `flowapi` command, its Rust core and TypeScript
// target compiled to WebAssembly, the same bytes out for the same description. It reads no file
// and fetches nothing but its own module: the host gives it the documents' text (`./node` reads
// them from disk, as the command does).

import { call, type Call } from './wasm.ts'
import type { Diagnostic, Document, Input, Options, Result } from './types.ts'

export type { Diagnostic, Document, File, Input, Options, Result, Severity } from './types.ts'

/**
 * Where the module comes from: compiled already, its bytes, a response, or its URL. By default
 * it is `flowapi.wasm` beside this file, which bundlers that follow
 * `new URL('…', import.meta.url)` (Vite, webpack, Parcel, Rollup's plugin) copy for you; with
 * another, serve `@localix-ai/flowapi/flowapi.wasm` and give its URL.
 */
export type Source =
  | WebAssembly.Module
  | BufferSource
  | Response
  | PromiseLike<Response>
  | URL
  | string

/** The generator, loaded: its calls are synchronous. */
export interface Flowapi {
  /** Generates TypeScript from a description and the documents its references reach. */
  generate(input: Input, options?: Options): Result
  /**
   * Every `$ref` and `$schema` of a document, JSON or YAML, that names another document, in the
   * order the command reads them: what to read and pass to `generate` after the description.
   * `uri` says whether the text is YAML, as a document's does (`.yaml`, `.yml`; any name but
   * `.json` whose text does not open with `{` or `[`); without one the text is JSON. A text that
   * cannot be read has none: `generate` says why.
   */
  references(text: string, uri?: string): string[]
}

/** The URI a description given as bare text is read under. */
const BARE_URI = 'input.json'

let compiled: Promise<WebAssembly.Module> | undefined

/**
 * Loads the module, once: later calls return the same generator whatever their `source`. Each
 * call of the generator runs in a fresh instance, so no memory is held between calls.
 */
export async function load(source?: Source): Promise<Flowapi> {
  return loaded(await module(source))
}

/**
 * Generates TypeScript from a description, loading the module first if need be. In a browser's
 * main thread this is the call to use: its instance is made asynchronously.
 */
export async function generate(input: Input, options?: Options): Promise<Result> {
  const instance = await WebAssembly.instantiate(await module(), {})
  return answer(call(instance, 'flowapi_generate', request(input, options)))
}

/** The module compiled, once; a failed load is tried again on the next call. */
async function module(source?: Source): Promise<WebAssembly.Module> {
  compiled ??= compile(source ?? new URL('./flowapi.wasm', import.meta.url))
  try {
    return await compiled
  } catch (error) {
    compiled = undefined
    throw error
  }
}

/** A diagnostic as the command prints it: `error[code]: message`, then `  at uri:line:column`. */
export function formatDiagnostic(d: Diagnostic): string {
  return `${d.severity}[${d.code}]: ${d.message}\n  at ${d.uri}:${d.line}:${d.column}`
}

function loaded(module: WebAssembly.Module): Flowapi {
  const run = (name: Call, text: string): string =>
    call(new WebAssembly.Instance(module, {}), name, text)
  return {
    generate: (input, options) => answer(run('flowapi_generate', request(input, options))),
    references: (text, uri = BARE_URI) =>
      JSON.parse(run('flowapi_references', JSON.stringify({ uri, text }))) as string[],
  }
}

function request(input: Input, options: Options | undefined): string {
  const documents: readonly Document[] =
    typeof input === 'string'
      ? [{ uri: BARE_URI, text: input }]
      : 'text' in input
        ? [input as Document]
        : (input as readonly Document[])
  const list = documents.map(({ uri, text }) => ({ uri, text }))
  return JSON.stringify({ documents: list, options: { output: options?.output, sdk: options?.sdk === true } })
}

function answer(json: string): Result {
  return JSON.parse(json) as Result
}

async function compile(source: Source): Promise<WebAssembly.Module> {
  if (source instanceof WebAssembly.Module) return source
  if (source instanceof ArrayBuffer || ArrayBuffer.isView(source)) {
    return WebAssembly.compile(source as BufferSource)
  }
  if (typeof source === 'string' || source instanceof URL) {
    const url = new URL(source, import.meta.url)
    if (url.protocol === 'file:') return WebAssembly.compile(await readFile(url))
    return compile(fetch(url))
  }
  const response = await source
  if (!response.ok) {
    throw new Error(`flowapi: cannot load its module from ${response.url}: ${response.status}`)
  }
  return WebAssembly.compile(await response.arrayBuffer())
}

/** Reads the module from disk in Node, Bun and Deno; the name is hidden from bundlers, which
 * would otherwise pull `node:fs` into a page that never takes this branch. */
async function readFile(url: URL): Promise<Uint8Array<ArrayBuffer>> {
  const name = 'node:fs/promises'
  const fs = (await import(/* @vite-ignore */ /* webpackIgnore: true */ name)) as {
    readFile(url: URL): Promise<Uint8Array<ArrayBuffer>>
  }
  return fs.readFile(url)
}
