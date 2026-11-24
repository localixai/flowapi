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

/// An answer that is not a response the description names, or whose body is not what it says;
/// the status, the content type and the body are kept, to read by hand.
#[derive(Debug)]
pub struct Error {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
    pub kind: ErrorKind,
}

/// Why an answer was not read.
#[derive(Debug)]
pub enum ErrorKind {
    /// No response of the operation is for this status, and it has no `default`.
    Status,
    /// The response names no media type that the `Content-Type` is (or none was sent).
    MediaType,
    /// The body is not the JSON of the media type's type.
    Json(serde_json::Error),
    /// A `text/*` body that is not UTF-8.
    Text,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ct = self.content_type.as_deref().unwrap_or("no content type");
        match &self.kind {
            ErrorKind::Status => write!(
                f,
                "status {} is not a response of the operation",
                self.status
            ),
            ErrorKind::MediaType => write!(
                f,
                "status {}: {ct} is not a media type of the response",
                self.status
            ),
            ErrorKind::Json(e) => write!(
                f,
                "status {} ({ct}): the body is not its type: {e}",
                self.status
            ),
            ErrorKind::Text => write!(f, "status {} ({ct}): the body is not UTF-8", self.status),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(status: u16, content_type: Option<&str>, body: &[u8], kind: ErrorKind) -> Self {
        Self {
            status,
            content_type: content_type.map(str::to_owned),
            body: body.to_vec(),
            kind,
        }
    }
}

/// How a style writes a value: RFC 6570's operator (`first`, `sep`, `named`, `ifemp`; 3.2,
/// Appendix C), `explode`, what joins an unexploded list (`,`, `%20`, `%7C`) and the encoder.
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
    Cookie,
    Deep,
}

/// What a value's text is written as: RFC 3986 percent-encoding of all but the unreserved
/// characters; that with the reserved ones and percent-encoded triples kept (`allowReserved`);
/// or as it is (headers and `style: cookie`, 3.2 Appendix D).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enc {
    Plain,
    Reserved,
    Raw,
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
        }
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
        if unreserved(b) || (enc == Enc::Reserved && reserved(b)) || triple {
            i += if triple { 3 } else { 1 };
            continue;
        }
        // `run..i` is ASCII up to a character's start: never cut inside one.
        out.push_str(t.get(run..i).unwrap_or(""));
        out.push('%');
        out.push(char::from(HEX[usize::from(b >> 4)]));
        out.push(char::from(HEX[usize::from(b & 15)]));
        i += 1;
        run = i;
    }
    out.push_str(t.get(run..).unwrap_or(""));
}

/// One parameter's value written into `out` by its style, after `sep` when `out` holds
/// something already; `false` when there was nothing to write (an empty list or object).
pub fn put<T: Serialize + ?Sized>(
    out: &mut String,
    sep: &str,
    name: &str,
    s: Style,
    v: &T,
) -> bool {
    let lead = if out.is_empty() { "" } else { sep };
    let mut w = write::W::new(out, lead, name, s);
    v.serialize(expand::Ser {
        w: &mut w,
        at: write::At::Top,
    })
    .unwrap_or(false)
}

/// A header, set when its value writes something.
pub fn header<T: Serialize + ?Sized>(
    headers: &mut Vec<(String, String)>,
    name: &str,
    s: Style,
    v: &T,
) {
    let mut text = String::new();
    if put(&mut text, "", name, s, v) {
        headers.push((name.to_owned(), text));
    }
}

/// A value as `application/x-www-form-urlencoded`, after `sep` when `out` holds something:
/// each member a pair, a list's items pairs of the same key, an object member as JSON, `null`
/// an empty value (the Encoding Object's defaults).
pub fn form<T: Serialize + ?Sized>(out: &mut String, sep: &str, v: &T) {
    let lead = if out.is_empty() { "" } else { sep };
    let s = Style::new(Shape::Form, true, Enc::Plain);
    let mut w = write::W::new(out, lead, "", s);
    let _ = v.serialize(expand::Ser {
        w: &mut w,
        at: write::At::Form,
    });
}

/// A form body's bytes.
pub fn form_body<T: Serialize + ?Sized>(v: &T) -> Vec<u8> {
    let mut out = String::new();
    form(&mut out, "&", v);
    out.into_bytes()
}

/// A JSON body's bytes. The generated types always serialize (their maps' keys are strings).
pub fn json_body<T: Serialize + ?Sized>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).unwrap_or_default()
}

/// A parameter with `content`: its JSON for a JSON media type, else its text (a string as it
/// is, a number or boolean as written, anything else as JSON); then written as a string.
pub fn content<T: Serialize + ?Sized>(v: &T, json: bool) -> String {
    if json {
        return serde_json::to_string(v).unwrap_or_default();
    }
    let mut out = String::new();
    let s = Style::new(Shape::Simple, false, Enc::Raw);
    let mut w = write::W::new(&mut out, "", "", s);
    match v.serialize(expand::Ser {
        w: &mut w,
        at: write::At::Top,
    }) {
        Err(expand::Error::Compound) => serde_json::to_string(v).unwrap_or_default(),
        Ok(_) | Err(_) => out,
    }
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

/// A JSON body read as its type; an empty or broken body is an error, never a default.
pub fn json<T: DeserializeOwned>(status: u16, ct: Option<&str>, body: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(body).map_err(|e| Error::new(status, ct, body, ErrorKind::Json(e)))
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
