// Paths as the Rust command's `Path` joins and splits them, so every path the package prints
// (a diagnostic's URI, a draft's `spec`, a file `generate` refuses) is the Rust command's.

/** `Path::parent` as text: `''` for a bare name, `'/'` for a file at the root. */
export function parent(path: string): string {
  const trimmed = path.replace(/(?<=.)\/+$/, '')
  const slash = trimmed.lastIndexOf('/')
  if (slash === -1) return ''
  const head = trimmed.slice(0, slash).replace(/\/+$/, '')
  return head === '' ? '/' : head
}

/** `Path::join`: an absolute path replaces, an empty base adds nothing. */
export function join(base: string, path: string): string {
  if (path.startsWith('/') || base === '') return path
  return base.endsWith('/') ? `${base}${path}` : `${base}/${path}`
}
