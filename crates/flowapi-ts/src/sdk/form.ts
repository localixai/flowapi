
/** WHATWG's `application/x-www-form-urlencoded` byte serializer: the UTF-8 of every character but
 * the ASCII letters and digits and `*-._` percent-encoded, a space as `+` (3.2, "Encoding the
 * `x-www-form-urlencoded` Media Type"; Appendix E, browsers). What it writes is also valid RFC
 * 3986 query text. */
function $wenc(s: string): string {
  if ($unreserved.test(s) && s.indexOf("~") < 0) return s;
  return encodeURIComponent(s).replace($wmarks, $mark).replace($space, "+");
}

const $wmarks = /[!'()~]/g;
const $space = /%20/g;

/** Whether an Encoding Object serializes its value by style: any of `style`, `explode` and
 * `allowReserved` written makes it so, and its `contentType` is then not used. */
function $styled(e: FlowapiPart): boolean {
  return e.style !== undefined || e.explode !== undefined || e.allowReserved !== undefined;
}

/** The join of an unexploded array or object in each style a form takes (2.0's `tsv` as
 * `tabDelimited`), as written in a URI. */
function $joins(style: string): string {
  return style === "spaceDelimited" ? " " : style === "pipeDelimited" ? "|" : style === "tabDelimited" ? "\t" : ",";
}

/** A member of a form serialized by style, as a `query` parameter of that style is: `style`
 * `form` unless written, `explode` true for `form` unless written, `allowReserved` keeping the
 * reserved characters, the arrays inside an array joined by `nested`; no leading `?`. */
function $stylePairs(v: unknown, key: string, e: FlowapiPart): string | undefined {
  const style = e.style ?? "form";
  const enc = e.allowReserved === true ? $res : $enc;
  const name = $enc(key);
  if (style === "deepObject") return $deep(v, name, enc);
  const join = style === "form" ? "," : $enc($joins(style));
  const nest = e.nested?.map((s) => (s === "form" ? "," : $enc($joins(s))));
  return $x(v, name, "", "&", true, "=", e.explode ?? style === "form", join, enc, "body", key, nest);
}

/** An object as `application/x-www-form-urlencoded`. A member whose Encoding Object (`enc`, by
 * its name) has a style is serialized by it; any other is serialized by its `contentType`, JSON
 * for a JSON type and for an object, else its text (a string as it is), then encoded by WHATWG's
 * rules; an array's items are pairs of the same name, `null` an empty value. */
function $form(v: unknown, enc?: FlowapiEncoding): string {
  if (v === null || typeof v !== "object") return v === undefined ? "" : $wenc($str(v));
  let out = "";
  for (const key of Object.keys(v)) {
    const item = (v as Record<string, unknown>)[key];
    if (item === undefined) continue;
    const e = enc?.[key];
    if (e !== undefined && $styled(e)) {
      out = $add(out, $stylePairs(item, key, e), "&");
      continue;
    }
    const json = e?.contentType !== undefined && $json(e.contentType);
    for (const one of Array.isArray(item) ? item : [item]) {
      const s = one === null ? "" : json ? JSON.stringify(one) : $str(one);
      out += (out === "" ? "" : "&") + $wenc(key) + "=" + $wenc(s);
    }
  }
  return out;
}

/** A `multipart/form-data` member serialized by style (3.1, 3.2): as a form member is, but each
 * pair its own part and no percent-encoding (`allowReserved` has no effect). `form` exploded
 * makes a part per item, or per member of an object under the member's name; unexploded, one
 * part of the items joined, an array among them by `nested`; `deepObject` a part per member,
 * named `name[key]`. */
function $styleParts(v: unknown, name: string, e: FlowapiPart): [string, string][] {
  const style = e.style ?? "form";
  const explode = e.explode ?? style === "form";
  const out: [string, string][] = [];
  if (Array.isArray(v)) {
    const nest = e.nested?.map($joins);
    const raw = (s: string): string => s;
    const items = v.filter((x) => x !== null && x !== undefined).map((x) => (nest !== undefined && Array.isArray(x) ? $nest(x, nest, 0, raw) : $str(x)));
    if (explode && style === "form") for (const s of items) out.push([name, s]);
    else if (items.length > 0) out.push([name, items.join($joins(style))]);
  } else if (typeof v === "object" && v !== null) {
    const keys = Object.keys(v).filter((k) => (v as Record<string, unknown>)[k] != null);
    const at = (k: string) => $str((v as Record<string, unknown>)[k]);
    if (style === "deepObject") for (const k of keys) out.push([`${name}[${k}]`, at(k)]);
    else if (explode && style === "form") for (const k of keys) out.push([k, at(k)]);
    else if (keys.length > 0) out.push([name, keys.map((k) => k + $joins(style) + at(k)).join($joins(style))]);
  } else if (v !== undefined && v !== null) {
    out.push([name, $str(v)]);
  }
  return out;
}
