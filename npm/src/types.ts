/** One document: the URI its references resolve against, and its text, JSON or YAML. */
export interface Document {
  /**
   * A path, a URL or any name: a relative `$ref` in another document resolves against it. It says
   * whether the text is YAML, as the command decides: `.yaml` and `.yml` are, `.json` is not, and
   * under any other name a text is YAML when it does not open with `{` or `[`.
   */
  readonly uri: string
  /** The document's JSON or YAML (YAML 1.2, one document). */
  readonly text: string
}

/**
 * What to generate from: a description's JSON text, one document, or the description followed
 * by every document its references reach. A YAML description is given as a document whose `uri`
 * names it so: bare text is read as JSON.
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
  /**
   * A requirement id of the specifications, or a code of flowapi's own (`json.syntax`, and
   * `yaml.syntax` for a YAML text that JSON cannot hold).
   */
  readonly code: string
  readonly message: string
  /** The document's URI; empty when the request itself could not be read. */
  readonly uri: string
  /**
   * The byte offset into the UTF-8 JSON the generator read: the document's text, or for a YAML
   * document the JSON it was read into (for `yaml.syntax`, into the YAML text).
   */
  readonly offset: number
  /** The line, from 1: in a YAML document, the YAML's. */
  readonly line: number
  /** The column, in characters (Unicode scalar values), from 1. */
  readonly column: number
}

/** What generation gives: the files (none when a diagnostic is an error) and what it found. */
export interface Result {
  readonly files: readonly File[]
  readonly diagnostics: readonly Diagnostic[]
}

/** Which operations `summary` gives, as the command's `--tag`, `--path` and `--max-bytes`. */
export interface SummaryFilter {
  /** Only the operations with this tag. */
  readonly tag?: string
  /** Only the operations whose path starts so. */
  readonly path?: string
  /** Over this many bytes, the tags alone with their counts, to be read on by `tag` or `path`.
   * 60000 by default. */
  readonly maxBytes?: number
}

/**
 * What `summary` and `configDraft` give: the text the command prints, `null` when the documents
 * are not an OpenAPI or Swagger description; or, with no text, the document that could not be
 * read.
 */
export interface Output {
  readonly text: string | null
  readonly diagnostics: readonly Diagnostic[]
}

/** One issue of a config, as `config check --json` gives it. */
export interface Issue {
  readonly severity: Severity
  /** The rule: `config.<what>`. */
  readonly code: string
  /** The member it is about, as a JSON Pointer into the config. */
  readonly pointer: string
  /** The line, from 1: in a YAML config, the YAML's. */
  readonly line: number
  /** The column, in characters, from 1. */
  readonly column: number
  readonly message: string
  /** What to write instead (the closest operation for a typo), where one is known. */
  readonly hint?: string
}

/**
 * What `configCheck` finds. `text` is the issues as `config check` prints them (empty when there
 * are none) and `json` as `config check --json` prints them; both are `null`, and `ok` false,
 * when the config or a document could not be read (`diagnostics` says why).
 */
export interface Check {
  /** No issue is an error. */
  readonly ok: boolean
  readonly errors: number
  readonly warnings: number
  readonly issues: readonly Issue[]
  readonly text: string | null
  readonly json: string | null
  readonly diagnostics: readonly Diagnostic[]
}
