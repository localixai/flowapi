//! URI references resolved against a base (RFC 3986 §5.2), and split from their fragment. The
//! documents' own URIs and every `$id` are only names: nothing is ever fetched (G5).

/// `reference` resolved against `base`, both as written; the result keeps the reference's
/// fragment, if any.
#[must_use]
pub fn resolve(base: &str, reference: &str) -> String {
    let r = Parts::of(reference);
    if r.scheme.is_some() {
        return r.join(&remove_dots(r.path));
    }
    let b = Parts::of(base);
    let (authority, path, query) = if r.authority.is_some() {
        (r.authority, remove_dots(r.path), r.query)
    } else if r.path.is_empty() {
        (b.authority, b.path.to_owned(), r.query.or(b.query))
    } else if r.path.starts_with('/') {
        (b.authority, remove_dots(r.path), r.query)
    } else {
        (b.authority, remove_dots(&merge(&b, r.path)), r.query)
    };
    Parts {
        scheme: b.scheme,
        authority,
        path: &path,
        query,
        fragment: r.fragment,
    }
    .join(&path)
}

/// A URI split at its fragment: the resource, and the fragment without its `#`.
#[must_use]
pub fn split_fragment(uri: &str) -> (&str, &str) {
    uri.split_once('#').unwrap_or((uri, ""))
}

/// A URI with its percent-escapes decoded, for comparing two names of one resource; an escape
/// that is not UTF-8 or not two hex digits is kept as written.
#[must_use]
pub fn decoded(uri: &str) -> std::borrow::Cow<'_, str> {
    if !uri.contains('%') {
        return std::borrow::Cow::Borrowed(uri);
    }
    let bytes = uri.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (b, hex) {
            (b'%', Some(v)) => {
                out.push(v);
                i += 3;
            }
            _ => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_or(std::borrow::Cow::Borrowed(uri), std::borrow::Cow::Owned)
}

struct Parts<'u> {
    scheme: Option<&'u str>,
    authority: Option<&'u str>,
    path: &'u str,
    query: Option<&'u str>,
    fragment: Option<&'u str>,
}

impl<'u> Parts<'u> {
    fn of(uri: &'u str) -> Self {
        let (rest, fragment) = match uri.split_once('#') {
            Some((rest, f)) => (rest, Some(f)),
            None => (uri, None),
        };
        let (rest, query) = match rest.split_once('?') {
            Some((rest, q)) => (rest, Some(q)),
            None => (rest, None),
        };
        let (scheme, rest) = match rest.split_once(':') {
            Some((s, rest)) if is_scheme(s) => (Some(s), rest),
            Some(_) | None => (None, rest),
        };
        let (authority, path) = match rest.strip_prefix("//") {
            Some(after) => {
                let end = after.find('/').unwrap_or(after.len());
                (after.get(..end), after.get(end..).unwrap_or(""))
            }
            None => (None, rest),
        };
        Self {
            scheme,
            authority,
            path,
            query,
            fragment,
        }
    }

    fn join(&self, path: &str) -> String {
        let mut out = String::new();
        if let Some(s) = self.scheme {
            out.push_str(s);
            out.push(':');
        }
        if let Some(a) = self.authority {
            out.push_str("//");
            out.push_str(a);
        }
        out.push_str(path);
        if let Some(q) = self.query {
            out.push('?');
            out.push_str(q);
        }
        if let Some(f) = self.fragment {
            out.push('#');
            out.push_str(f);
        }
        out
    }
}

fn is_scheme(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

fn merge(base: &Parts<'_>, path: &str) -> String {
    if base.authority.is_some() && base.path.is_empty() {
        return format!("/{path}");
    }
    match base.path.rfind('/') {
        Some(i) => format!("{}{path}", base.path.get(..=i).unwrap_or("")),
        None => path.to_owned(),
    }
}

/// RFC 3986 §5.2.4.
fn remove_dots(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    let absolute = path.starts_with('/');
    let segments: Vec<&str> = path.split('/').collect();
    let last = segments.len().saturating_sub(1);
    for (i, segment) in segments.iter().enumerate() {
        match *segment {
            "." => {
                if i == last {
                    out.push("");
                }
            }
            ".." => {
                if out.len() > usize::from(absolute) {
                    out.pop();
                }
                if i == last {
                    out.push("");
                }
            }
            other => out.push(other),
        }
    }
    let joined = out.join("/");
    if absolute && !joined.starts_with('/') {
        format!("/{joined}")
    } else {
        joined
    }
}

#[cfg(test)]
#[path = "uri_tests.rs"]
mod tests;
