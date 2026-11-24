//! What the SDK refuses to write and what it could not read: printed inside the generated
//! `sdk` module after the runtime (these lines left out), never compiled here.

/// A request `<name>_request` refuses to write: a value that would change what the request
/// means, rather than what it says. Nothing of it is sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestError {
    /// Where the value goes: `path`, `query`, `querystring`, `header`, `cookie` or `body`.
    pub location: &'static str,
    /// The parameter's name as the description writes it; `None` for the body.
    pub parameter: Option<&'static str>,
    pub reason: Reason,
}

/// Why a request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// A CR, LF or NUL in a header's or a cookie's value, which would end the header it is in
    /// and start another (header injection).
    ControlCharacter,
    /// A path parameter that makes a whole segment empty: `/files/` names another resource.
    EmptySegment,
    /// A path parameter that makes a segment `.` or `..` (or `%2E` for a dot): a client or a
    /// server removes it and the one before it (RFC 3986, 5.2.4), so the path names another
    /// resource (path traversal).
    DotSegment,
    /// NaN or an infinity, which JSON has no number for: serde_json would write `null`, and a
    /// parameter the text `NaN`.
    NonFinite,
}

impl Reason {
    /// The reason as a code, the same as the TypeScript SDK's `FlowapiRequestError.reason`.
    pub fn code(self) -> &'static str {
        match self {
            Self::ControlCharacter => "control-character",
            Self::EmptySegment => "empty-segment",
            Self::DotSegment => "dot-segment",
            Self::NonFinite => "non-finite",
        }
    }
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let why = match self.reason {
            Reason::ControlCharacter => "a CR, LF or NUL in its value",
            Reason::EmptySegment => "an empty path segment",
            Reason::DotSegment => "a path segment `.` or `..`",
            Reason::NonFinite => "NaN or an infinity, which JSON has no number for",
        };
        match self.parameter {
            Some(name) => write!(f, "the {} parameter `{name}`: {why}", self.location),
            None => write!(f, "the body: {why}"),
        }
    }
}

impl std::error::Error for RequestError {}

/// What turns a refusal into the error of a parameter.
pub fn at(location: &'static str, parameter: &'static str) -> impl FnOnce(Reason) -> RequestError {
    move |reason| RequestError {
        location,
        parameter: Some(parameter),
        reason,
    }
}

/// A refusal of the body.
pub fn in_body(reason: Reason) -> RequestError {
    RequestError {
        location: "body",
        parameter: None,
        reason,
    }
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
