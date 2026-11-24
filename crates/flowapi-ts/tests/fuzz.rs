//! Oracle O7: no input makes flowapi panic (G1). Random JSON texts, random bytes and random
//! schemas (every keyword, references to random places, cycles, wrong shapes) are generated from
//! fixed seeds, so a failure is the same on every run; each must end in output or diagnostics.

#![expect(
    clippy::expect_used,
    reason = "a generator's own bug fails the test by panicking"
)]

use std::fmt::Write;

/// A small deterministic generator (xorshift64*), so the cases need no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % (n as u64)).expect("fits")
    }

    fn pick<'a>(&mut self, list: &[&'a str]) -> &'a str {
        list[self.below(list.len())]
    }
}

const KEYWORDS: &[&str] = &[
    "type",
    "enum",
    "const",
    "properties",
    "required",
    "additionalProperties",
    "patternProperties",
    "items",
    "prefixItems",
    "allOf",
    "anyOf",
    "oneOf",
    "not",
    "if",
    "then",
    "else",
    "$ref",
    "$defs",
    "$id",
    "$anchor",
    "$dynamicRef",
    "$dynamicAnchor",
    "minItems",
    "maxItems",
    "unevaluatedProperties",
    "unevaluatedItems",
    "dependentRequired",
    "dependentSchemas",
    "propertyNames",
    "contains",
    "nullable",
    "discriminator",
    "format",
    "minimum",
    "$schema",
];
const TYPES: &[&str] = &[
    "\"null\"",
    "\"boolean\"",
    "\"integer\"",
    "\"number\"",
    "\"string\"",
    "\"array\"",
    "\"object\"",
    "[\"string\",\"null\"]",
    "\"nope\"",
    "7",
];
const REFS: &[&str] = &[
    "\"#\"",
    "\"#/$defs/a\"",
    "\"#/$defs/b\"",
    "\"#a\"",
    "\"#/properties/x\"",
    "\"other.json\"",
    "\"#/$defs/a/properties/x\"",
    "\"#/nowhere\"",
    "\"%%\"",
    "3",
];

fn value(r: &mut Rng, depth: u32, out: &mut String) {
    match r.below(if depth == 0 { 4 } else { 7 }) {
        0 => out.push_str(r.pick(&["null", "true", "false", "0", "-1.5e3", "1.0"])),
        1 => out.push_str(r.pick(&["\"a\"", "\"x-y\"", "\"\\u00e9\"", "\"\\ud800\"", "\"\""])),
        2 => out.push_str(r.pick(TYPES)),
        3 => out.push_str(r.pick(&["true", "false", "{}"])),
        4 => {
            out.push('[');
            for i in 0..r.below(4) {
                if i > 0 {
                    out.push(',');
                }
                schema(r, depth - 1, out);
            }
            out.push(']');
        }
        _ => schema(r, depth - 1, out),
    }
}

fn schema(r: &mut Rng, depth: u32, out: &mut String) {
    if depth == 0 || r.below(6) == 0 {
        out.push_str(r.pick(&["true", "false", "{}", "{\"type\":\"string\"}"]));
        return;
    }
    out.push('{');
    for i in 0..r.below(5) {
        if i > 0 {
            out.push(',');
        }
        let key = r.pick(KEYWORDS);
        let _ = write!(out, "\"{key}\":");
        match key {
            "$ref" | "$dynamicRef" => out.push_str(r.pick(REFS)),
            "type" => out.push_str(r.pick(TYPES)),
            "properties" | "$defs" | "patternProperties" | "dependentSchemas" => {
                out.push('{');
                for (j, name) in ["a", "x", "^x-", "b"]
                    .iter()
                    .take(r.below(4) + 1)
                    .enumerate()
                {
                    if j > 0 {
                        out.push(',');
                    }
                    let _ = write!(out, "\"{name}\":");
                    schema(r, depth - 1, out);
                }
                out.push('}');
            }
            "required" => out.push_str(r.pick(&["[\"a\"]", "[\"x\",\"y\"]", "[]", "[1]", "\"a\""])),
            _ => value(r, depth, out),
        }
    }
    out.push('}');
}

/// Whether the text was read and generated from (the generators must make JSON, or the test
/// proves nothing).
fn generate(text: &str, other: Option<&str>) -> bool {
    let mut docs = flowapi::Documents::new();
    if docs.add("root.json", text).is_err() {
        return false;
    }
    if let Some(o) = other {
        let _ = docs.add("other.json", o);
    }
    let target = flowapi_ts::TypeScript::new("out.ts");
    let (output, diags) = flowapi::generate(&docs, &target);
    assert!(
        output.is_some() || diags.has_errors(),
        "no output and no error for {text}"
    );
    true
}

#[test]
fn random_schemas_never_panic() {
    let mut r = Rng(0x9e37_79b9_7f4a_7c15);
    let mut read = 0;
    for _ in 0..20_000 {
        let mut root = String::new();
        schema(&mut r, 5, &mut root);
        let mut other = String::new();
        schema(&mut r, 3, &mut other);
        read += usize::from(generate(&root, Some(&other)));
    }
    assert_eq!(read, 20_000, "every generated schema is JSON");
}

#[test]
fn random_descriptions_never_panic() {
    let mut r = Rng(42);
    for _ in 0..5_000 {
        let mut s = String::new();
        schema(&mut r, 4, &mut s);
        let version = r.pick(&["3.0.3", "3.1.0", "3.2.0", "4.0", "3"]);
        let text = format!(
            "{{\"openapi\":\"{version}\",\"info\":{{\"title\":\"t\",\"version\":\"1\"}},\"paths\":{{\"/a/{{id}}\":{{\"get\":{{\"parameters\":[{{\"name\":\"id\",\"in\":\"path\",\"schema\":{s}}},{{\"$ref\":\"#/components/parameters/p\"}}],\"responses\":{{\"200\":{{\"content\":{{\"application/json\":{{\"schema\":{s}}}}}}},\"default\":{{\"$ref\":\"#/nowhere\"}}}}}}}}}},\"components\":{{\"schemas\":{{\"S\":{s},\"T\":{{\"$ref\":\"#/components/schemas/S\"}}}}}}}}"
        );
        // `4.0` and `3` are versions no reader takes: refused at reading, as they must be.
        let read = generate(&text, None);
        assert!(
            read || !version.starts_with("3."),
            "a generated description is read: {text}"
        );
    }
}

#[test]
fn random_bytes_never_panic_the_reader() {
    let mut r = Rng(7);
    let alphabet = b"{}[]:,\"\\ ntrufalse0123456789.eE+-\x00\xff";
    for _ in 0..20_000 {
        let len = r.below(64);
        let bytes: Vec<u8> = (0..len)
            .map(|_| alphabet[r.below(alphabet.len())])
            .collect();
        if let Ok(text) = std::str::from_utf8(&bytes) {
            let _ = flowapi::json::parse(text);
        }
    }
}
