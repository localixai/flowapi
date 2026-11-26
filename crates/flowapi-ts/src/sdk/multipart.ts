
/** A file to send as a part of a `multipart/form-data` body: its bytes, the name its part's
 * `filename` is written from and the `Content-Type` it is sent as. Made by `flowapiFile` or
 * `flowapiBlob`. */
export class FlowapiFile {
  readonly data: Uint8Array;
  readonly name: string | undefined;
  readonly type: string | undefined;
  constructor(data: Uint8Array, name: string | undefined, type: string | undefined) {
    this.data = data;
    this.name = name;
    this.type = type;
  }
}

/** A file from bytes, or from text as its UTF-8, with the name and the type to send it as. */
export function flowapiFile(data: Uint8Array | string, name?: string, type?: string): FlowapiFile {
  return new FlowapiFile(typeof data === "string" ? $bytes(data) : data, name, type);
}

/** What `flowapiBlob` reads of a `Blob` or a `File`, typed here so the output type-checks
 * without the DOM's declarations. */
export type $Blob = { readonly type: string; readonly name?: string; arrayBuffer(): Promise<ArrayBuffer> };

/** A `Blob` or a `File` read into a `FlowapiFile`: its bytes, `name` or else a File's own, `type`
 * or else the Blob's. The one step that waits: `<op>Request` takes bytes and stays synchronous. */
export async function flowapiBlob(blob: $Blob, name?: string, type?: string): Promise<FlowapiFile> {
  const bytes = new Uint8Array(await blob.arrayBuffer());
  return new FlowapiFile(bytes, name ?? blob.name, type ?? (blob.type === "" ? undefined : blob.type));
}

/** Per part's name, what the description's Encoding Object says of it: the `Content-Type` its
 * value is sent as, and the headers its part carries (`Content-Type` among them is ignored, as
 * the spec says). An array's items each take their property's. */
export type FlowapiEncoding = {
  readonly [name: string]: { readonly contentType?: string; readonly headers?: { readonly [name: string]: string } } | undefined;
};

/** A `multipart/form-data` value as `<op>Request` takes it: a string member may also be a file
 * (the types cannot tell `format: binary` from text), an array member's items one part each. */
type $Upload<T> = T extends string ? T | Uint8Array | FlowapiFile : T;
type $Member<T> = T extends readonly (infer U)[] ? readonly $Upload<U>[] : $Upload<T>;
type $Fields<T> = T extends object ? { [K in keyof T]: $Member<T[K]> } : T;
export type $Form<T> = T extends readonly (infer U)[] ? readonly $Fields<U>[] : $Fields<T>;

/** A request body as the input gives it. */
type $Given = { readonly contentType: string; readonly value: unknown; readonly encoding?: FlowapiEncoding };

/** The body set on a request with its `Content-Type`: a `multipart/form-data` object (or 3.2's
 * array of objects, its parts in order) written as parts, any other value by `$body`. */
function $put(request: $Request, body: $Given): void {
  const v = body.value;
  if ($bare(body.contentType) === "multipart/form-data" && typeof v === "object" && v !== null && !ArrayBuffer.isView(v)) {
    const parts: $Part[] = [];
    for (const one of Array.isArray(v) ? v : [v]) {
      if (one === undefined) continue;
      if (one === null) throw new FlowapiRequestError("body", null, "null");
      if (typeof one !== "object" || Array.isArray(one) || ArrayBuffer.isView(one) || one instanceof FlowapiFile || $unread(one)) {
        throw new FlowapiRequestError("body", null, "unnamed-part");
      }
      for (const name of Object.keys(one)) $member(parts, name, (one as Record<string, unknown>)[name], body.encoding?.[name]);
    }
    const [type, bytes] = $multipart(body.contentType, parts);
    request.headers["content-type"] = type;
    request.body = bytes;
    return;
  }
  request.headers["content-type"] = body.contentType;
  request.body = $body(body.contentType, v);
}

/** One part: its head (`Content-Disposition` and the other headers, each line ended) and data. */
type $Part = { head: string; data: Uint8Array };

type $One = { readonly contentType?: string; readonly headers?: { readonly [name: string]: string } } | undefined;

let $encoder: { encode(text: string): Uint8Array } | undefined;

/** Text as UTF-8 bytes. */
function $bytes(text: string): Uint8Array {
  $encoder ??= new (globalThis as unknown as $Coders).TextEncoder();
  return $encoder.encode(text);
}

const $breaks = /[\r\n\0]/;
const $quotes = /["\\]/;
const $token = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;

/** `s` refused when it holds a CR, LF or NUL (it would end its header line), and where it goes
 * between quotes, a `"` or `\`: RFC 7578 and WHATWG's FormData escape them differently, so a
 * receiver could read another name. */
function $safe(s: string, part: string, quoted: boolean): string {
  if ($breaks.test(s)) throw new FlowapiRequestError("body", part, "control-character");
  if (quoted && $quotes.test(s)) throw new FlowapiRequestError("body", part, "quote");
  return s;
}

/** A file's name as its part's `filename`: its last path segment (a sender's directories are not
 * the receiver's), refused when that is empty, `.` or `..`, and when the name holds a CR, LF or
 * NUL anywhere (cut off with a directory, it would still be meant). */
function $filename(name: string, part: string): string {
  $safe(name, part, false);
  const base = name.slice(Math.max(name.lastIndexOf("/"), name.lastIndexOf("\\")) + 1);
  if (base === "" || base === "." || base === "..") throw new FlowapiRequestError("body", part, "file-name");
  return $safe(base, part, true);
}

/** A member's parts: an array's items one part each in order, `undefined` none. */
function $member(parts: $Part[], name: string, v: unknown, e: $One): void {
  if (v === undefined) return;
  $safe(name, name, true);
  if (!Array.isArray(v)) return $part(parts, name, v, e);
  for (let i = 0; i < v.length; i++) if (v[i] !== undefined) $part(parts, name, v[i], e);
}

/** One part, by the Encoding Object's defaults for the value: bytes and files as files (WHATWG's
 * `blob` for a name not given, `application/octet-stream` for a type not given), strings,
 * numbers and booleans as text (`text/plain`, the part's default, so not written), objects and
 * arrays as JSON. `encoding.contentType` replaces the default; a file's own type replaces it. */
function $part(parts: $Part[], name: string, v: unknown, e: $One): void {
  if (v === null) throw new FlowapiRequestError("body", name, "null");
  // Bytes in any other view are the same bytes.
  if (v instanceof ArrayBuffer) v = new Uint8Array(v);
  else if (ArrayBuffer.isView(v) && !(v instanceof Uint8Array)) v = new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
  let head = `Content-Disposition: form-data; name="${name}"`;
  let type = e?.contentType;
  let data: Uint8Array;
  if (v instanceof FlowapiFile || v instanceof Uint8Array) {
    const file = v instanceof FlowapiFile ? v : undefined;
    head += `; filename="${$filename(file?.name ?? "blob", name)}"`;
    type = file?.type ?? type ?? "application/octet-stream";
    data = file?.data ?? (v as Uint8Array);
  } else if ($unread(v)) {
    throw new FlowapiRequestError("body", name, "unread-blob");
  } else {
    if ($bad(v)) throw new FlowapiRequestError("body", name, "non-finite");
    type ??= typeof v === "object" ? "application/json" : undefined;
    // An object in its type (JSON, or a form), a scalar as its text unless the type is JSON.
    const s = typeof v === "object" || (type !== undefined && $json(type)) ? $body(type ?? "application/json", v) : $str(v);
    data = typeof s === "string" ? $bytes(s) : s;
  }
  if (type !== undefined) {
    if (type.indexOf(",") >= 0 || $bare(type).indexOf("*") >= 0) throw new FlowapiRequestError("body", name, "media-range");
    head += `\r\nContent-Type: ${$safe(type, name, false)}`;
  }
  const headers = e?.headers ?? {};
  for (const key of Object.keys(headers)) {
    const value = headers[key];
    const lower = key.toLowerCase();
    if (value === undefined || lower === "content-type") continue;
    if (!$token.test(key) || lower === "content-disposition") throw new FlowapiRequestError("body", name, "header-name");
    head += `\r\n${key}: ${$safe(value, name, false)}`;
  }
  parts.push({ head: head + "\r\n", data });
}

/** A `Blob`, a `File` or a `Response` given unread: its bytes come only by waiting
 * (`flowapiBlob`), and as JSON it would be `{}`. */
function $unread(v: unknown): boolean {
  return typeof v === "object" && v !== null && typeof (v as { arrayBuffer?: unknown }).arrayBuffer === "function";
}

const $bchars = /^[0-9A-Za-z'()+_,\-./:=? ]{0,69}[0-9A-Za-z'()+_,\-./:=?]$/;

/** Whether `needle` occurs in `data`. */
function $holds(data: Uint8Array, needle: Uint8Array): boolean {
  const first = needle[0] ?? 0;
  for (let i = data.indexOf(first); i >= 0 && i + needle.length <= data.length; i = data.indexOf(first, i + 1)) {
    let j = 1;
    while (j < needle.length && data[i + j] === needle[j]) j++;
    if (j === needle.length) return true;
  }
  return false;
}

/** Whether a boundary occurs in any part: a delimiter in a part would end it there. */
function $collides(parts: $Part[], boundary: string): boolean {
  const b = $bytes(boundary);
  return parts.some((p) => p.head.indexOf(boundary) >= 0 || $holds(p.data, b));
}

/** A boundary of 128 random bits from `globalThis.crypto`, so no content can be made to hold it. */
function $random(): string {
  const c = (globalThis as unknown as { crypto?: { getRandomValues(b: Uint8Array): Uint8Array } }).crypto;
  if (c === undefined) throw new Error("flowapi: no globalThis.crypto to draw a multipart boundary from; give one: `multipart/form-data; boundary=...`");
  let s = "flowapi-";
  for (const x of c.getRandomValues(new Uint8Array(16))) s += (x < 16 ? "0" : "") + x.toString(16);
  return s;
}

/** The parts written with a boundary (RFC 2046, 5.1.1; RFC 7578): the one the content type
 * gives (`boundary=`), refused when RFC 2046 does not allow it or a part holds it, or one drawn
 * at random and added to the content type, drawn again while a part holds it. */
function $multipart(contentType: string, parts: $Part[]): [string, Uint8Array] {
  const given = /;\s*boundary\s*=\s*(?:"([^"]*)"|([^;\s]*))/i.exec(contentType);
  let boundary: string;
  let type = contentType;
  if (given !== null) {
    boundary = given[1] ?? given[2] ?? "";
    if (!$bchars.test(boundary) || $collides(parts, boundary)) throw new FlowapiRequestError("body", null, "boundary");
  } else {
    boundary = $random();
    for (let tries = 1; $collides(parts, boundary); tries++) {
      if (tries === 8) throw new FlowapiRequestError("body", null, "boundary");
      boundary = $random();
    }
    type += "; boundary=" + boundary;
  }
  const chunks: Uint8Array[] = [];
  for (const p of parts) chunks.push($bytes(`--${boundary}\r\n${p.head}\r\n`), p.data, $bytes("\r\n"));
  chunks.push($bytes(`--${boundary}--\r\n`));
  let size = 0;
  for (const c of chunks) size += c.length;
  const out = new Uint8Array(size);
  let at = 0;
  for (const c of chunks) {
    out.set(c, at);
    at += c.length;
  }
  return [type, out];
}
