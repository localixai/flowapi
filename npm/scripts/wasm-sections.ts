// A WebAssembly module with its custom sections (id 0: `name`, `producers`,
// `target_features`) dropped. They hold debugging aids and nothing the module runs; the code is
// the bytes cargo built.

/** The module's bytes without its custom sections. */
export function stripCustomSections(module: Uint8Array): Uint8Array {
  const magic = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
  if (module.length < 8 || magic.some((b, i) => module[i] !== b)) {
    throw new Error('not a WebAssembly 1.0 module')
  }
  const kept: Uint8Array[] = [module.subarray(0, 8)]
  let at = 8
  while (at < module.length) {
    const start = at
    const id = module[at++] as number
    let size = 0
    for (let shift = 0; ; shift += 7) {
      const byte = module[at++]
      if (byte === undefined || shift > 28) throw new Error('a section size past the module')
      size += (byte & 0x7f) * 2 ** shift
      if ((byte & 0x80) === 0) break
    }
    const end = at + size
    if (end > module.length) throw new Error('a section past the module')
    if (id !== 0) kept.push(module.subarray(start, end))
    at = end
  }
  const out = new Uint8Array(kept.reduce((n, part) => n + part.length, 0))
  let offset = 0
  for (const part of kept) {
    out.set(part, offset)
    offset += part.length
  }
  return out
}
