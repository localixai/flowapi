// The module's ABI (crates/flowapi-wasm/src/abi.rs): a request written into a buffer the module
// allocates, an answer read from the buffer it returns (four bytes of length, then UTF-8 JSON)
// and freed. Each call runs in a fresh instance: nothing is kept between calls, and the memory a
// large description took is returned with the instance, as the command returns it on exit.

interface Exports {
  readonly memory: WebAssembly.Memory
  flowapi_alloc(len: number): number
  flowapi_free(ptr: number, len: number): void
  flowapi_generate(ptr: number, len: number): number
  flowapi_references(ptr: number, len: number): number
}

/** An export that takes a request and returns an answer. */
export type Call = 'flowapi_generate' | 'flowapi_references'

const encoder = new TextEncoder()
const decoder = new TextDecoder('utf-8', { fatal: true })

/** One call into an instance made for it. */
export function call(instance: WebAssembly.Instance, name: Call, request: string): string {
  const x = instance.exports as unknown as Exports
  try {
    const bytes = encoder.encode(request)
    const at = x.flowapi_alloc(bytes.length)
    new Uint8Array(x.memory.buffer, at, bytes.length).set(bytes)
    const answer = x[name](at, bytes.length)
    const length = new DataView(x.memory.buffer).getUint32(answer, true)
    const text = decoder.decode(new Uint8Array(x.memory.buffer, answer + 4, length))
    x.flowapi_free(answer, length + 4)
    return text
  } catch (error) {
    throw new Error(
      `flowapi: the WebAssembly module stopped, which is a bug in flowapi; please report it with the description that caused it (${String(error)})`,
      { cause: error },
    )
  }
}
