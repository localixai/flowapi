//! The SDK's runtime: what every operation's `<name>_request` and `<name>_response` call. No
//! I/O: send the request with any HTTP client, give its answer back. It needs serde and
//! serde_json, as the types do, and nothing else; percent-encoding is written here, by byte.
//! Printed once as `pub mod sdk` after the types, never compiled in flowapi itself.

use serde::Serialize;
use serde::de::DeserializeOwned;

/// A request to send. `path` goes after the server's base URL, percent-encoded already;
/// `query` is the query string without its `?`, empty for none; `body` is its content type and
/// bytes. Nothing more is added on the way: a content type's parameters (`; charset=utf-8`) are
/// the caller's to add to `body`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub method: &'static str,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<(String, Vec<u8>)>,
}

/// Which response of the description an answer was read as, as the description writes it.
pub trait Response {
    /// The status as written: `200`, `4XX` or `default`.
    fn response(&self) -> &'static str;
    /// The media type as written, `None` for a response with no body.
    fn media_type(&self) -> Option<&'static str>;
}

/// How a style writes a value: RFC 6570's operator (`first`, `sep`, `named`, `ifemp`; 3.2,
/// Appendix C), `explode`, what joins an unexploded list (`,`, `%20`, `%7C`, `%09`) and the
/// encoder; `nest`, Swagger 2.0's, what joins the lists inside a list, outermost first.
#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub first: &'static str,
    pub sep: &'static str,
    pub named: bool,
    pub ifemp: bool,
    pub explode: bool,
    pub join: &'static str,
    pub enc: Enc,
    pub shape: Shape,
    pub nest: &'static [&'static str],
}

/// The styles, by what they write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Simple,
    Label,
    Matrix,
    Form,
    SpaceDelimited,
    PipeDelimited,
    TabDelimited,
    Cookie,
    Deep,
}

/// What a value's text is written as: RFC 3986 percent-encoding of all but the unreserved
/// characters; that with the reserved ones and percent-encoded triples kept (`allowReserved`);
/// as it is (headers and `style: cookie`, 3.2 Appendix D); or by WHATWG's
/// `application/x-www-form-urlencoded` rules, all but the letters, digits and `*-._` encoded and
/// a space as `+` (a form's content, 3.2 Appendix E).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enc {
    Plain,
    Reserved,
    Raw,
    Form,
}

impl Style {
    pub const fn new(shape: Shape, explode: bool, enc: Enc) -> Self {
        let (first, sep, named, ifemp, join) = match shape {
            Shape::Simple => ("", ",", false, false, ","),
            Shape::Label => (".", ".", false, false, ","),
            Shape::Matrix => (";", ";", true, false, ","),
            Shape::Form | Shape::Deep => ("", "&", true, true, ","),
            Shape::SpaceDelimited => ("", "&", true, true, "%20"),
            Shape::PipeDelimited => ("", "&", true, true, "%7C"),
            Shape::TabDelimited => ("", "&", true, true, "%09"),
            Shape::Cookie => ("", "; ", true, true, ","),
        };
        Self {
            first,
            sep,
            named,
            ifemp,
            explode,
            join,
            enc,
            shape,
            nest: &[],
        }
    }

    /// The style with an unexploded list joined by `join` (Swagger 2.0's delimited formats in
    /// a path or a header).
    pub const fn joined(mut self, join: &'static str) -> Self {
        self.join = join;
        self
    }

    /// The style with the lists inside a list joined by `nest`, outermost first.
    pub const fn nested(mut self, nest: &'static [&'static str]) -> Self {
        self.nest = nest;
        self
    }
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~')
}

fn reserved(b: u8) -> bool {
    matches!(
        b,
        b':' | b'/'
            | b'?'
            | b'#'
            | b'['
            | b']'
            | b'@'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'='
    )
}

/// `t` written into `out` by `enc`: runs that need nothing are copied whole, each other byte
/// of its UTF-8 is `%XX`.
pub fn encode(out: &mut String, t: &str, enc: Enc) {
    if enc == Enc::Raw {
        out.push_str(t);
        return;
    }
    let bytes = t.as_bytes();
    let mut run = 0;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let triple = enc == Enc::Reserved
            && b == b'%'
            && bytes.get(i + 1).is_some_and(u8::is_ascii_hexdigit)
            && bytes.get(i + 2).is_some_and(u8::is_ascii_hexdigit);
        let kept = if enc == Enc::Form {
            b.is_ascii_alphanumeric() || matches!(b, b'*' | b'-' | b'.' | b'_')
        } else {
            unreserved(b) || (enc == Enc::Reserved && reserved(b)) || triple
        };
        if kept {
            i += if triple { 3 } else { 1 };
            continue;
        }
        // `run..i` is ASCII up to a character's start: never cut inside one.
        out.push_str(t.get(run..i).unwrap_or(""));
        if enc == Enc::Form && b == b' ' {
            out.push('+');
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(b >> 4)]));
            out.push(char::from(HEX[usize::from(b & 15)]));
        }
        i += 1;
        run = i;
    }
    out.push_str(t.get(run..).unwrap_or(""));
}

/// One parameter's value written into `out` by its style, after `sep` when `out` holds
/// something already; `false` when there was nothing to write (an empty list or object). A
/// value written raw (a header, `style: cookie`) is refused if it holds a CR, LF or NUL, and
/// NaN or an infinity anywhere in a value is refused.
pub fn put<T: Serialize + ?Sized>(
    out: &mut String,
    sep: &str,
    name: &str,
    s: Style,
    v: &T,
) -> Result<bool, Reason> {
    let start = out.len();
    let lead = if out.is_empty() { "" } else { sep };
    let mut w = write::W::new(out, lead, name, s);
    let wrote = match v.serialize(expand::Ser {
        w: &mut w,
        at: write::At::Top,
    }) {
        Ok(wrote) => wrote,
        Err(expand::Error::NonFinite) => return Err(Reason::NonFinite),
        Err(expand::Error::Compound | expand::Error::Custom(_)) => false,
    };
    if s.enc == Enc::Raw {
        control(out.get(start..).unwrap_or(""))?;
    }
    Ok(wrote)
}

/// A CR, LF or NUL, which would end a header and start another.
fn control(text: &str) -> Result<(), Reason> {
    if text.bytes().any(|b| matches!(b, b'\r' | b'\n' | 0)) {
        Err(Reason::ControlCharacter)
    } else {
        Ok(())
    }
}

/// A header, set when its value writes something.
pub fn header<T: Serialize + ?Sized>(
    headers: &mut Vec<(String, String)>,
    name: &str,
    s: Style,
    v: &T,
) -> Result<(), Reason> {
    let mut text = String::new();
    if put(&mut text, "", name, s, v)? {
        headers.push((name.to_owned(), text));
    }
    Ok(())
}

/// The path segment written from `start` to the path's end, refused when it is empty or a dot
/// segment: `.`, `..`, or either with a dot as `%2E` (WHATWG's URL parser reads those as dots
/// too). `pieces`: a parameter of it keeps `/` (`allowReserved`), and each piece between them is
/// held to the same.
#[inline]
pub fn segment(path: &str, start: usize, pieces: bool) -> Result<(), Reason> {
    let len = path.len().saturating_sub(start);
    // A dot segment is at most 6 bytes (`%2e%2e`): a longer one is none, read no further.
    if len > 6 && !pieces {
        return Ok(());
    }
    if len == 0 {
        return Err(Reason::EmptySegment);
    }
    let s = path.get(start..).unwrap_or("");
    let dot = |p: &str| after_dot(p).is_some_and(|r| r.is_empty() || after_dot(r) == Some(""));
    if dot(s) || (pieces && s.split('/').any(dot)) {
        return Err(Reason::DotSegment);
    }
    Ok(())
}

/// What follows a leading `.` or `%2E`.
fn after_dot(p: &str) -> Option<&str> {
    match p.get(..3) {
        Some(h) if h.eq_ignore_ascii_case("%2e") => p.get(3..),
        _ => p.strip_prefix('.'),
    }
}

/// A value as `application/x-www-form-urlencoded`, after `sep` when `out` holds something:
/// each member a pair, a list's items pairs of the same key, an object member as JSON, `null`
/// an empty value (the Encoding Object's defaults), encoded by WHATWG's rules. NaN or an infinity
/// is refused.
pub fn form<T: Serialize + ?Sized>(out: &mut String, sep: &str, v: &T) -> Result<(), Reason> {
    let lead = if out.is_empty() { "" } else { sep };
    let s = Style::new(Shape::Form, true, Enc::Form);
    let mut w = write::W::new(out, lead, "", s);
    match v.serialize(expand::Ser {
        w: &mut w,
        at: write::At::Form,
    }) {
        Err(expand::Error::NonFinite) => Err(Reason::NonFinite),
        Ok(_) | Err(expand::Error::Compound | expand::Error::Custom(_)) => Ok(()),
    }
}

/// A form body's bytes.
pub fn form_body<T: Serialize + ?Sized>(v: &T) -> Result<Vec<u8>, Reason> {
    let mut out = String::new();
    form(&mut out, "&", v)?;
    Ok(out.into_bytes())
}

/// A JSON body's bytes; NaN or an infinity anywhere in it is refused (serde_json would write
/// `null`). The generated types always serialize (their maps' keys are strings).
#[inline]
pub fn json_body<T: Serialize + ?Sized>(v: &T) -> Result<Vec<u8>, Reason> {
    json_text(v).map(String::into_bytes)
}

/// A parameter with `content`: its JSON for a JSON media type, else its text (a string as it
/// is, a number or boolean as written, anything else as JSON); then written as a string. NaN or
/// an infinity is refused.
pub fn content<T: Serialize + ?Sized>(v: &T, json: bool) -> Result<String, Reason> {
    if json {
        return json_text(v);
    }
    let mut out = String::new();
    let s = Style::new(Shape::Simple, false, Enc::Raw);
    let mut w = write::W::new(&mut out, "", "", s);
    match v.serialize(expand::Ser {
        w: &mut w,
        at: write::At::Top,
    }) {
        Err(expand::Error::NonFinite) => Err(Reason::NonFinite),
        Err(expand::Error::Compound) => json_text(v),
        Ok(_) | Err(expand::Error::Custom(_)) => Ok(out),
    }
}

/// A value's JSON text, NaN or an infinity in it refused. serde_json writes a non-finite number
/// as `null`: with no `null` in the text there is none, and the value is walked only when there
/// is one.
fn json_text<T: Serialize + ?Sized>(v: &T) -> Result<String, Reason> {
    let text = serde_json::to_string(v).unwrap_or_default();
    if text.contains("null") && !finite::finite(v) {
        return Err(Reason::NonFinite);
    }
    Ok(text)
}

/// `in: querystring` of a media type other than a form: the text percent-encoded, after `&`.
pub fn query_string(out: &mut String, text: &str) {
    if !out.is_empty() {
        out.push('&');
    }
    encode(out, text, Enc::Plain);
}

/// The `Content-Type` of an answer, its name compared without case.
pub fn content_type(headers: &[(String, String)]) -> Option<&str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.as_str())
}

/// Which of a response's media types (each bare, in lower case) the content type is: the exact
/// one, then its `type/*`, then `*/*`.
pub fn media(content_type: Option<&str>, table: &[&str]) -> Option<usize> {
    let ct = content_type?;
    let bare = ct.split(';').next().unwrap_or("").trim();
    let major = bare.split('/').next().unwrap_or("");
    let is_range = |t: &&str| {
        t.len() == major.len() + 2
            && t.ends_with("/*")
            && t[..major.len()].eq_ignore_ascii_case(major)
    };
    table
        .iter()
        .position(|t| t.eq_ignore_ascii_case(bare))
        .or_else(|| table.iter().position(is_range))
        .or_else(|| table.iter().position(|t| *t == "*/*"))
}

/// A JSON body read as its type; an empty or broken body is an error, never a default. A
/// leading byte order mark is skipped (RFC 8259, 8.1 lets a parser ignore it).
pub fn json<T: DeserializeOwned>(status: u16, ct: Option<&str>, body: &[u8]) -> Result<T, Error> {
    let text = body.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(body);
    serde_json::from_slice(text).map_err(|e| Error::new(status, ct, body, ErrorKind::Json(e)))
}

/// A `text/*` body.
pub fn text(status: u16, ct: Option<&str>, body: &[u8]) -> Result<String, Error> {
    std::str::from_utf8(body)
        .map(str::to_owned)
        .map_err(|_| Error::new(status, ct, body, ErrorKind::Text))
}

/// A body of a media range (`image/*`, `*/*`): the content type sent and the bytes.
pub fn ranged(ct: Option<&str>, body: &[u8]) -> (String, Vec<u8>) {
    (ct.unwrap_or("").to_owned(), body.to_vec())
}
