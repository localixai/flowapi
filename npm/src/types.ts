/** One document: the URI its references resolve against, and its JSON text. */
export interface Document {
  /** A path, a URL or any name: a relative `$ref` in another document resolves against it. */
  readonly uri: string
  /** The document's JSON. */
  readonly text: string
}

/**
 * What to generate from: a description's JSON text, one document, or the description followed
 * by every document its references reach.
 */
export type Input = string | Document | readonly Document[]

/** How to generate. */
export interface Options {
  /** The path of the file written, as `-o` gives it to the command. `types.ts` by default. */
  readonly output?: string
  /** Also write each operation's request and response functions after the types (`--sdk`). */
  readonly sdk?: boolean
}

/** A file generated. */
export interface File {
  readonly path: string
  readonly text: string
}

/** How much a diagnostic matters: an error means no file was generated. */
export type Severity = 'error' | 'warning'

/** One finding, where it is and the rule it is about. */
export interface Diagnostic {
  readonly severity: Severity
  /** A requirement id of the specifications, or a code of flowapi's own (`json.syntax`). */
  readonly code: string
  readonly message: string
  /** The document's URI; empty when the request itself could not be read. */
  readonly uri: string
  /** The byte offset into the document's UTF-8 text. */
  readonly offset: number
  /** The line, from 1. */
  readonly line: number
  /** The column, in characters (Unicode scalar values), from 1. */
  readonly column: number
}

/** What generation gives: the files (none when a diagnostic is an error) and what it found. */
export interface Result {
  readonly files: readonly File[]
  readonly diagnostics: readonly Diagnostic[]
}
