//! A string token's escapes decoded.

/// Decodes the inside of a string token that the reader accepted. `None` for a lone surrogate,
/// which JSON's grammar allows and a Rust string cannot hold.
pub(super) fn decode(inner: &str) -> Option<String> {
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let decoded = match chars.next()? {
            '"' => '"',
            '\\' => '\\',
            '/' => '/',
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => unicode(&mut chars)?,
            _ => return None,
        };
        out.push(decoded);
    }
    Some(out)
}

fn unicode(chars: &mut std::str::Chars<'_>) -> Option<char> {
    let first = hex4(chars)?;
    if (0xd800..0xdc00).contains(&first) {
        if chars.next()? != '\\' || chars.next()? != 'u' {
            return None;
        }
        let second = hex4(chars)?;
        if !(0xdc00..0xe000).contains(&second) {
            return None;
        }
        return char::from_u32(0x10000 + ((first - 0xd800) << 10) + (second - 0xdc00));
    }
    char::from_u32(first)
}

fn hex4(chars: &mut std::str::Chars<'_>) -> Option<u32> {
    let mut value = 0;
    for _ in 0..4 {
        value = value * 16 + chars.next()?.to_digit(16)?;
    }
    Some(value)
}
