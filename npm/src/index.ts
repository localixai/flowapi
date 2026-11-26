// flowapi for JavaScript: the generator of the `flowapi` command, its Rust core and TypeScript
// target compiled to WebAssembly, the same bytes out for the same description. It reads no file
// and fetches nothing but its own modules: the host gives it the documents' text (`./node` reads
// them from disk, as the command does). The commands about an SDK's config are a second module,
// `flowapi-sdk.wasm`, compiled the first time one of them is called: `generate` never loads it.

import { call, type Call } from './wasm.ts'
import * as req from './request.ts'
import type {
  Check,
  Diagnostic,
  Document,
  Input,
  Options,
  Output,
  Result,
  SummaryFilter,
} from './types.ts'

export type {
  Check,
  Diagnostic,
  Document,
  File,
  Input,
  Issue,
  Options,
  Output,
  Result,
  Severity,
  SummaryFilter,
} from './types.ts'

/**
 * Where the module comes from: compiled already, its bytes, a response, or its URL. By default
 * it is `flowapi.wasm` beside this file, which bundlers that follow
 * `new URL('…', import.meta.url)` (Vite, webpack, Parcel, Rollup's plugin) copy for you; with
 * another, serve `@localix-ai/flowapi/flowapi.wasm` and give its URL. The same holds for the
 * second module, `flowapi-sdk.wasm`, given to `loadSdk`.
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

/**
 * The commands about an SDK's config, loaded: the second module, `flowapi-sdk.wasm`, which only
 * they load. Its calls are synchronous.
 */
export interface FlowapiSdk {
  /**
   * The description in compact JSON, as `flowapi summary` prints it: its operations with their
   * parameters, media types and the pagination they look like they have; over `maxBytes` the
   * tags alone. `text` is `null` when the documents are not an OpenAPI or Swagger description.
   */
  summary(input: Input, filter?: SummaryFilter): Output
  /**
   * A first `sdk-anything.yml`, every operation in a resource, as `flowapi config draft` prints
   * it. `spec` is the description's path as the config names it, relative to where the config
   * will be (the description's `uri` by default).
   */
  configDraft(input: Input, spec?: string): Output
  /** The config's format as JSON Schema (2020-12), as `flowapi config schema` prints it. */
  configSchema(): string
  /**
   * The description a config (JSON or YAML) names, its `spec` as written, for the host to read
   * before `configCheck`: relative to the config, as the command reads it.
   */
  configSpec(config: Document): Output
  /**
   * Every issue of a config against its description (the description followed by the documents
   * its references reach; none for a config that names none), as `flowapi config check` finds
   * them. The text names the config by its `uri`.
   */
  configCheck(config: Document, input?: Input): Check
}

/** A module compiled once, from its `source` or its default URL; a failed load is tried again
 * on the next call. */
function cached(url: () => URL): (source?: Source) => Promise<WebAssembly.Module> {
  let compiled: Promise<WebAssembly.Module> | undefined
  return async (source) => {
    compiled ??= compile(source ?? url())
    try {
      return await compiled
    } catch (error) {
      compiled = undefined
      throw error
    }
  }
}

// Each URL written out whole, as bundlers find a module to copy.
const generator = cached(() => new URL('./flowapi.wasm', import.meta.url))
const sdkModule = cached(() => new URL('./flowapi-sdk.wasm', import.meta.url))

/**
 * Loads the module, once: later calls return the same generator whatever their `source`. Each
 * call of the generator runs in a fresh instance, so no memory is held between calls.
 */
export async function load(source?: Source): Promise<Flowapi> {
  return loaded(await generator(source))
}

/**
 * Loads the second module, `flowapi-sdk.wasm`, once, as `load` loads the first (with another
 * bundler, serve `@localix-ai/flowapi/flowapi-sdk.wasm` and give its URL). `generate` never
 * loads it.
 */
export async function loadSdk(source?: Source): Promise<FlowapiSdk> {
  return loadedSdk(await sdkModule(source))
}

/**
 * Generates TypeScript from a description, loading the module first if need be. In a browser's
 * main thread this is the call to use: its instance is made asynchronously.
 */
export async function generate(input: Input, options?: Options): Promise<Result> {
  const request = req.generateRequest(input, options)
  return req.generateAnswer(await once(generator, 'flowapi_generate', request))
}

/** `FlowapiSdk.summary`, loading the second module first if need be. */
export async function summary(input: Input, filter?: SummaryFilter): Promise<Output> {
  return req.outputAnswer(await once(sdkModule, 'flowapi_sdk', req.summaryRequest(input, filter)))
}

/** `FlowapiSdk.configDraft`, loading the second module first if need be. */
export async function configDraft(input: Input, spec?: string): Promise<Output> {
  return req.outputAnswer(await once(sdkModule, 'flowapi_sdk', req.draftRequest(input, spec)))
}

/** `FlowapiSdk.configSchema`, loading the second module first if need be. */
export async function configSchema(): Promise<string> {
  return req.outputAnswer(await once(sdkModule, 'flowapi_sdk', req.schemaRequest)).text ?? ''
}

/** `FlowapiSdk.configCheck`, loading the second module first if need be. */
export async function configCheck(config: Document, input?: Input): Promise<Check> {
  const request = req.checkRequest(config, input)
  return req.checkAnswer(await once(sdkModule, 'flowapi_sdk', request))
}

/** One call in an instance made asynchronously, as a browser's main thread needs. */
async function once(
  module: () => Promise<WebAssembly.Module>,
  name: Call,
  request: string,
): Promise<string> {
  const instance = await WebAssembly.instantiate(await module(), {})
  return call(instance, name, request)
}

/** A diagnostic as the command prints it: `error[code]: message`, then `  at uri:line:column`. */
export function formatDiagnostic(d: Diagnostic): string {
  return `${d.severity}[${d.code}]: ${d.message}\n  at ${d.uri}:${d.line}:${d.column}`
}

/** An export called in a fresh instance of `module`. */
function runner(module: WebAssembly.Module): (name: Call, text: string) => string {
  return (name, text) => call(new WebAssembly.Instance(module, {}), name, text)
}

function loaded(module: WebAssembly.Module): Flowapi {
  const run = runner(module)
  return {
    generate: (input, options) =>
      req.generateAnswer(run('flowapi_generate', req.generateRequest(input, options))),
    references: (text, uri = req.BARE_URI) =>
      JSON.parse(run('flowapi_references', JSON.stringify({ uri, text }))) as string[],
  }
}

function loadedSdk(module: WebAssembly.Module): FlowapiSdk {
  const run = runner(module)
  const sdk = (text: string): string => run('flowapi_sdk', text)
  return {
    summary: (input, filter) => req.outputAnswer(sdk(req.summaryRequest(input, filter))),
    configDraft: (input, spec) => req.outputAnswer(sdk(req.draftRequest(input, spec))),
    configSchema: () => req.outputAnswer(sdk(req.schemaRequest)).text ?? '',
    configSpec: (config) => req.outputAnswer(sdk(req.specRequest(config))),
    configCheck: (config, input) => req.checkAnswer(sdk(req.checkRequest(config, input))),
  }
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
