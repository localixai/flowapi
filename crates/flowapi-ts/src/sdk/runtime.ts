
// The SDK: for each operation, a function that writes its request and one that reads its
// response. No I/O: send the request with any HTTP client, give its answer back.

/**
 * A request to send. `path` goes after the server's base URL; `query` is the query string
 * without its `?`, empty for none; `body` text, or bytes (a multipart body, bytes given).
 */
export type $Request = {
  method: string;
  path: string;
  query: string;
  headers: Record<string, string>;
  body?: string | Uint8Array;
};

/**
 * A request `<op>Request` refuses to write: a value that would change what the request means
 * rather than what it says. `location` is where it goes (`path`, `query`, `querystring`,
 * `header`, `cookie` or `body`), `parameter` the parameter's name as the description writes it
 * (`null` for the body; a multipart part's name), `reason` why: a CR, LF or NUL in a header, a
 * cookie or a part's head (it would start another header), a path segment left empty or made `.`
 * or `..` (a client or a server would remove it and the one before it), NaN or an infinity (JSON
 * has no number for them). In a multipart body also: `null` where a part goes (`null`), a member
 * of an array of parts that is not an object (`unnamed-part`), a Blob not read yet
 * (`unread-blob`: `flowapiBlob` reads it), a `"` or `\` in a name or a file name (`quote`), a
 * file name that is empty, `.` or `..` (`file-name`), a part header's name that is not a token or
 * is `Content-Disposition` (`header-name`), a list or a range for a part's type (`media-range`),
 * a boundary RFC 2046 does not allow or that a part holds (`boundary`).
 */
export class FlowapiRequestError extends Error {
  readonly location: string;
  readonly parameter: string | null;
  readonly reason:
    | "control-character" | "empty-segment" | "dot-segment" | "non-finite" | "null" | "unnamed-part"
    | "unread-blob" | "quote" | "file-name" | "header-name" | "media-range" | "boundary";
  constructor(location: string, parameter: string | null, reason: FlowapiRequestError["reason"]) {
    const what = location === "body" ? "the body" + (parameter === null ? "" : `'s part \`${parameter}\``) : `the ${location} parameter \`${parameter}\``;
    super(`${what}: ${reason}`);
    this.name = "FlowapiRequestError";
    this.location = location;
    this.parameter = parameter;
    this.reason = reason;
  }
}

const $unreserved = /^[A-Za-z0-9\-._~]*$/;
const $marks = /[!'()*]/g;
const $marked = /[!'()*]/;
const $kept = /%[0-9A-Fa-f]{2}|[:/?#[\]@!$&'()*+,;=]/g;

function $mark(c: string): string {
  return "%" + c.charCodeAt(0).toString(16).toUpperCase();
}

/** RFC 3986: every character but the unreserved ones percent-encoded, as UTF-8. */
function $enc(s: string): string {
  if ($unreserved.test(s)) return s;
  const e = encodeURIComponent(s);
  return $marked.test(e) ? e.replace($marks, $mark) : e;
}

/** RFC 6570's reserved expansion: reserved characters and percent-encoded triples kept. */
function $res(s: string): string {
  if ($unreserved.test(s)) return s;
  $kept.lastIndex = 0;
  let out = "";
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = $kept.exec(s)) !== null) {
    out += $enc(s.slice(last, m.index)) + m[0];
    last = $kept.lastIndex;
  }
  return out + $enc(s.slice(last));
}

/** Headers and `style: cookie` take values as they are (3.2, Appendix D). */
function $raw(s: string): string {
  return s;
}

const $dot = /^(?:\.|%2e){1,2}$/i;

/** A value written raw, refused if it holds a CR, LF or NUL: it would end its header. */
function $ctl(s: string | undefined, at: string, name: string): string | undefined {
  if (s !== undefined && (s.indexOf("\r") >= 0 || s.indexOf("\n") >= 0 || s.indexOf("\0") >= 0)) {
    throw new FlowapiRequestError(at, name, "control-character");
  }
  return s;
}

/** A path segment a parameter wrote, refused when empty or a dot segment (`.`, `..`, a dot as
 * `%2E` too, as WHATWG's URL parser reads it). `pieces`: a parameter of it keeps `/`
 * (`allowReserved`), and each piece between them is held to the same. */
function $seg(s: string, name: string, pieces: boolean): string {
  // A dot segment is at most 6 characters (`%2e%2e`) and starts with `.` or `%`: a longer one
  // is none, and is not read (reading a string made by `+` would flatten it).
  if (!pieces && s.length > 6) return s;
  if (s.length === 0) throw new FlowapiRequestError("path", name, "empty-segment");
  const c = s.charCodeAt(0);
  if (!pieces && c !== 46 && c !== 37) return s;
  if ($dot.test(s) || (pieces && s.split("/").some((p) => $dot.test(p)))) {
    throw new FlowapiRequestError("path", name, "dot-segment");
  }
  return s;
}

/** Whether a value holds NaN or an infinity anywhere. */
function $bad(v: unknown): boolean {
  if (typeof v === "number") return v - v !== 0;
  if (v === null || typeof v !== "object") return false;
  if (Array.isArray(v)) {
    for (let i = 0; i < v.length; i++) if ($bad(v[i])) return true;
    return false;
  }
  for (const key in v) if ($bad((v as Record<string, unknown>)[key])) return true;
  return false;
}

/** A parameter's value, refused when it holds NaN or an infinity: JSON has no number for them,
 * and their text would reach the server as a string. */
function $fin<T>(v: T, at: string, name: string): T {
  if (typeof v === "number" ? v - v !== 0 : typeof v === "object" && v !== null && $bad(v)) {
    throw new FlowapiRequestError(at, name, "non-finite");
  }
  return v;
}

/** A value as text: a string as it is, a number, boolean or bigint by `String`, else JSON. */
function $str(v: unknown): string {
  switch (typeof v) {
    case "string":
      return v;
    case "number":
    case "boolean":
    case "bigint":
      return String(v);
    default:
      return JSON.stringify(v) ?? "";
  }
}

/**
 * One variable expanded by RFC 6570 (§3.2, Appendix A): `first` and `sep` are the operator's,
 * `named` and `ifemp` too; `join` is what separates the members of an unexploded array or
 * object (`,`, or `%20` and `%7C` for the space and pipe styles). `undefined` is nothing to
 * write: an absent value, an empty array or object (§2.3). `null` writes the "undefined" column
 * of the spec's Style Examples: `name=` for form and cookie, nothing else. NaN or an infinity
 * anywhere in the value is refused, as the parameter `pname` in `at`.
 */
function $x(
  v: unknown,
  name: string,
  first: string,
  sep: string,
  named: boolean,
  ifemp: string,
  explode: boolean,
  join: string,
  enc: (s: string) => string,
  at: string,
  pname: string,
): string | undefined {
  if (v === undefined) return undefined;
  if (v === null) return ifemp === "=" ? name + "=" : "";
  let out = "";
  let n = 0;
  if (Array.isArray(v)) {
    for (let i = 0; i < v.length; i++) {
      const item: unknown = v[i];
      if (item === null || item === undefined) continue;
      if (typeof item === "number" ? item - item !== 0 : typeof item === "object" && $bad(item)) {
        throw new FlowapiRequestError(at, pname, "non-finite");
      }
      const s = enc($str(item));
      if (explode && named) out += (n++ ? sep : "") + name + (s === "" ? ifemp : "=" + s);
      else out += (n++ ? (explode ? sep : join) : "") + s;
    }
  } else if (typeof v === "object") {
    for (const key of Object.keys(v)) {
      const item = (v as Record<string, unknown>)[key];
      if (item === null || item === undefined) continue;
      if (typeof item === "number" ? item - item !== 0 : typeof item === "object" && $bad(item)) {
        throw new FlowapiRequestError(at, pname, "non-finite");
      }
      const k = enc(key);
      const s = enc($str(item));
      if (explode) out += (n++ ? sep : "") + k + (s === "" && named ? ifemp : "=" + s);
      else out += (n++ ? join : "") + k + join + s;
    }
  } else {
    if (typeof v === "number" && v - v !== 0) throw new FlowapiRequestError(at, pname, "non-finite");
    const s = enc($str(v));
    return first + (named ? name + (s === "" ? ifemp : "=" + s) : s);
  }
  if (n === 0) return undefined;
  return first + (named && !explode ? name + "=" + out : out);
}

/** `deepObject`: `name[key]=value` for each member; a nested object goes a level deeper, an
 * array's items repeat their key. */
function $deep(v: unknown, name: string, enc: (s: string) => string): string | undefined {
  if (v === null || typeof v !== "object" || Array.isArray(v)) return undefined;
  const out = $members(v as Record<string, unknown>, name, enc, "");
  return out === "" ? undefined : out;
}

function $members(v: Record<string, unknown>, path: string, enc: (s: string) => string, acc: string): string {
  for (const key in v) {
    if (!Object.prototype.hasOwnProperty.call(v, key)) continue;
    const item = v[key];
    if (item === null || item === undefined) continue;
    const at = path + "%5B" + enc(key) + "%5D";
    if (Array.isArray(item)) {
      for (let i = 0; i < item.length; i++) {
        const one: unknown = item[i];
        if (one !== null && one !== undefined) acc += (acc === "" ? "" : "&") + at + "=" + enc($str(one));
      }
    } else if (typeof item === "object") {
      acc = $members(item as Record<string, unknown>, at, enc, acc);
    } else {
      acc += (acc === "" ? "" : "&") + at + "=" + enc($str(item));
    }
  }
  return acc;
}

/** `s` added to a query string or a cookie header, after `sep`. */
function $add(acc: string, s: string | undefined, sep: string): string {
  return s === undefined ? acc : acc === "" ? s : acc + sep + s;
}

/** The media type without its parameters, in lower case. */
function $bare(type: string): string {
  const semi = type.indexOf(";");
  return (semi < 0 ? type : type.slice(0, semi)).trim().toLowerCase();
}

/** `application/json` and every `+json` type (RFC 6839). */
function $json(type: string): boolean {
  const bare = $bare(type);
  return bare === "application/json" || bare.endsWith("+json");
}

/** A body in its media type: bytes as given, JSON, a form (by the Encoding Objects `enc`), or
 * text as given (anything else as JSON). NaN or an infinity in a JSON or form body is refused:
 * `JSON.stringify` would write `null`. */
function $body(type: string, v: unknown, enc?: FlowapiEncoding): string | Uint8Array {
  if (v instanceof Uint8Array) return v;
  if (!$json(type)) {
    if ($bare(type) === "application/x-www-form-urlencoded") {
      if ($bad(v)) throw new FlowapiRequestError("body", null, "non-finite");
      return $form(v, enc);
    }
    if (typeof v === "string") return v;
  }
  // `JSON.stringify` writes NaN and the infinities as `null`: with no `null` in the text there is
  // none, and the value is walked only when there is one.
  const text = JSON.stringify(v);
  if (text !== undefined && text.indexOf("null") >= 0 && $bad(v)) {
    throw new FlowapiRequestError("body", null, "non-finite");
  }
  return text;
}

/** A parameter with `content`: its value as its media type's text, then written as a string. */
function $content(v: unknown, type: string): string | undefined {
  if (v === undefined) return undefined;
  return $json(type) ? JSON.stringify(v) : $str(v);
}

/** `in: querystring`: the whole query string, a form as one, any other type percent-encoded. */
function $qs(v: unknown, type: string): string | undefined {
  if (v === undefined) return undefined;
  if ($bare(type) === "application/x-www-form-urlencoded") return $form(v);
  return $enc($json(type) ? JSON.stringify(v) : $str(v));
}

/** A header set when it has a value; a CR, LF or NUL in it is refused (it would end the header). */
function $set(headers: Record<string, string>, name: string, s: string | undefined): void {
  if (s === undefined) return;
  if (s.indexOf("\r") >= 0 || s.indexOf("\n") >= 0 || s.indexOf("\0") >= 0) {
    throw new FlowapiRequestError("header", name, "control-character");
  }
  headers[name] = s;
}
