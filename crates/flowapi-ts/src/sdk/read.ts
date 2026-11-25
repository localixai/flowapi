
/** A response read: which response of the description it is (`null` for none), and its body. */
type $Read = { response: string | null; status: number; mediaType: string | null; body: unknown };

/** Per response, as written: its status, its media types as written and their bare forms. */
type $Table = readonly (readonly [string, readonly string[], readonly string[]])[];

/** The platform's UTF-8 coders (every browser, Node, Deno and Bun has them), looked up on
 * `globalThis` so the output type-checks without the DOM's or Node's declarations. */
type $Coders = {
  TextDecoder: new (label: string, options: { fatal: boolean; ignoreBOM: boolean }) => {
    decode(bytes: Uint8Array): string;
  };
  TextEncoder: new () => { encode(text: string): Uint8Array };
};

let $decoder: { decode(bytes: Uint8Array): string } | undefined;

/** Bytes as UTF-8 text, a byte order mark kept as the character; `null` when they are not
 * UTF-8 (the Rust SDK refuses them too, where `Response.text()` would replace them). */
function $utf8(bytes: Uint8Array): string | null {
  $decoder ??= new (globalThis as unknown as $Coders).TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
  try {
    return $decoder.decode(bytes);
  } catch {
    return null;
  }
}

/** A `text/*` type (a JSON one is read as JSON first). */
function $text(bare: string): boolean {
  return bare.startsWith("text/");
}

/**
 * Whether `<op>Response` keeps a body of this `Content-Type` as bytes: any type but JSON and
 * `text/*`, and a body with none. Read such a body with `arrayBuffer()`, the others with
 * `text()`; bytes are always safe to give, as they are decoded where the type is text.
 */
export function flowapiReadsBytes(contentType: string | null | undefined): boolean {
  if (contentType === null || contentType === undefined) return true;
  return !$json(contentType) && !$text($bare(contentType));
}

/** A header, its name compared without case. */
function $header(headers: Record<string, string>, name: string): string | null {
  const direct = headers[name];
  if (direct !== undefined) return direct;
  for (const key of Object.keys(headers)) {
    if (key.toLowerCase() === name) return headers[key] ?? null;
  }
  return null;
}

/**
 * The response a status picks (its code, then its range `NXX`, then `default`), the media type
 * the `Content-Type` picks (exact, then `type/*`, then `*\/*`), and the body read by the type
 * sent: JSON parsed, `text/*` the text, any other the bytes (a string given for it encoded as
 * UTF-8: its bytes were lost before). Bytes given for JSON or text are decoded as UTF-8. A
 * status or a media type the description does not name, and a JSON or text body that is not
 * UTF-8 or not JSON, read as `response: null` with the body as given.
 */
function $read(table: $Table, status: number, headers: Record<string, string>, body: string | Uint8Array): $Read {
  const code = String(status);
  const range = code.charAt(0) + "XX";
  let hit: (typeof table)[number] | undefined;
  for (const r of table) if (r[0] === code) hit = hit ?? r;
  if (hit === undefined) for (const r of table) if (r[0].toUpperCase() === range) hit = hit ?? r;
  if (hit === undefined) for (const r of table) if (r[0] === "default") hit = hit ?? r;
  const type = $header(headers, "content-type");
  if (hit === undefined) return { response: null, status, mediaType: type, body };
  if (hit[1].length === 0) return { response: hit[0], status, mediaType: null, body };
  if (type === null) return { response: null, status, mediaType: null, body };
  const bare = $bare(type);
  const slash = bare.indexOf("/");
  const major = bare.slice(0, slash + 1) + "*";
  let at = hit[2].indexOf(bare);
  if (at < 0) at = hit[2].indexOf(major);
  if (at < 0) at = hit[2].indexOf("*/*");
  const written = hit[1][at];
  if (written === undefined) return { response: null, status, mediaType: type, body };
  const json = $json(type);
  if (!json && !$text(bare)) {
    const bytes = typeof body === "string" ? $bytes(body) : body;
    return { response: hit[0], status, mediaType: written, body: bytes };
  }
  const text = typeof body === "string" ? body : $utf8(body);
  if (text === null) return { response: null, status, mediaType: type, body };
  if (!json) return { response: hit[0], status, mediaType: written, body: text };
  // An empty or broken JSON body is not the response described: the body as given, unnamed. A
  // leading byte order mark is skipped (RFC 8259, 8.1), as `JSON.parse` would refuse it.
  try {
    return { response: hit[0], status, mediaType: written, body: JSON.parse(text.charCodeAt(0) === 0xfeff ? text.slice(1) : text) };
  } catch {
    return { response: null, status, mediaType: type, body };
  }
}
