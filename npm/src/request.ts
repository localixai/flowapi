// The requests the module reads and the answers it gives, as JSON text (crates/flowapi-wasm's
// `request.rs`, `answer.rs`, `sdk.rs` and `repo.rs` say what each holds).

import type {
  Check,
  Diagnostic,
  Document,
  Input,
  Issue,
  LockFile,
  Options,
  Output,
  Plan,
  PlanInput,
  Repo,
  Result,
  SummaryFilter,
} from './types.ts'

/** The URI a description given as bare text is read under. */
export const BARE_URI = 'input.json'

/** The documents an input names, the description first. */
function documents(input: Input): { uri: string; text: string }[] {
  const list: readonly Document[] =
    typeof input === 'string'
      ? [{ uri: BARE_URI, text: input }]
      : 'text' in input
        ? [input as Document]
        : (input as readonly Document[])
  return list.map(({ uri, text }) => ({ uri, text }))
}

function document({ uri, text }: Document): { uri: string; text: string } {
  return { uri, text }
}

export function generateRequest(input: Input, options: Options | undefined): string {
  const output = options?.output
  const sdk = options?.sdk === true
  return JSON.stringify({ documents: documents(input), options: { output, sdk } })
}

export function generateAnswer(json: string): Result {
  return JSON.parse(json) as Result
}

export function summaryRequest(input: Input, filter: SummaryFilter | undefined): string {
  const { tag, path, maxBytes } = filter ?? {}
  const request = { command: 'summary', documents: documents(input), filter: { tag, path, maxBytes } }
  return JSON.stringify(request)
}

export function draftRequest(input: Input, spec: string | undefined): string {
  return JSON.stringify({ command: 'draft', documents: documents(input), spec })
}

export const schemaRequest = JSON.stringify({ command: 'schema' })

export function specRequest(config: Document): string {
  return JSON.stringify({ command: 'spec', config: document(config) })
}

export function checkRequest(config: Document, input: Input | undefined): string {
  const list = input === undefined ? [] : documents(input)
  return JSON.stringify({ command: 'check', documents: list, config: document(config) })
}

interface Answer {
  readonly text: string | null
  readonly json: string | null
  readonly diagnostics: readonly Diagnostic[]
}

export function outputAnswer(json: string): Output {
  const { text, diagnostics } = JSON.parse(json) as Answer
  return { text, diagnostics }
}

export function checkAnswer(answer: string): Check {
  const { text, json, diagnostics } = JSON.parse(answer) as Answer
  if (json === null) {
    return { ok: false, errors: 0, warnings: 0, issues: [], text, json, diagnostics }
  }
  const { ok, errors, warnings, issues } = JSON.parse(json) as {
    ok: boolean
    errors: number
    warnings: number
    issues: readonly Issue[]
  }
  return { ok, errors, warnings, issues, text, json, diagnostics }
}

export function repoRequest(config: Document, input: Input, lock: LockFile): string {
  const { path, text, error } = lock
  const request = { command: 'repo', config: document(config), documents: documents(input) }
  return JSON.stringify({ ...request, lock: { path, text, error } })
}

export function planRequest(input: PlanInput): string {
  const { out, check, lock, previous } = input
  const files = input.files.map(({ path, owner, executable }) => ({ path, owner, executable }))
  return JSON.stringify({ command: 'plan', out, check, files, lock, previous, disk: input.disk })
}

/** A `repo` or `plan` answer; a request the module could not read has its fields empty. */
export function repoAnswer(json: string): Repo {
  const answer = JSON.parse(json) as Partial<Repo> & Pick<Repo, 'diagnostics'>
  const { text, error, files, lock, read, diagnostics } = answer
  return {
    text: text ?? '',
    error: error ?? null,
    files: files ?? [],
    lock: lock ?? null,
    read: read ?? [],
    diagnostics,
  }
}

export function planAnswer(json: string): Plan {
  const answer = JSON.parse(json) as Partial<Plan> & Pick<Plan, 'diagnostics'>
  const { text, next, changes, lock, summary, diagnostics } = answer
  return {
    text: text ?? '',
    next: next ?? 'failed',
    changes: changes ?? [],
    lock: lock ?? '',
    summary: summary ?? '',
    diagnostics,
  }
}
