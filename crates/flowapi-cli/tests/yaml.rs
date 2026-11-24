//! YAML read into JSON: each scalar form of the core schema, keys, escapes, anchors and aliases,
//! what JSON cannot hold refused, an alias bomb refused, and the YAML place of a JSON node.
#![expect(
    clippy::panic,
    reason = "a helper fails its test by panicking, as a test does"
)]

use flowapi_cli::yaml::{self, MAX_ALIAS_NODES};
use std::fmt::Write as _;

fn json(text: &str) -> String {
    match yaml::to_json(text) {
        Ok(j) => j.text,
        Err(e) => panic!("{text:?} was refused: {e}"),
    }
}

fn refused(text: &str) -> String {
    match yaml::to_json(text) {
        Ok(j) => panic!("{text:?} was read as {}", j.text),
        Err(e) => e.to_string(),
    }
}

#[test]
fn nulls_and_booleans() {
    for null in ["~", "null", "Null", "NULL", ""] {
        assert_eq!(json(&format!("a: {null}\n")), r#"{"a":null}"#, "{null:?}");
    }
    for (yes, value) in [("true", "true"), ("True", "true"), ("TRUE", "true")] {
        assert_eq!(json(yes), value);
    }
    for no in ["false", "False", "FALSE"] {
        assert_eq!(json(no), "false");
    }
    // Not the core schema's: YAML 1.1's yes, no, on, off are strings.
    assert_eq!(
        json("[yes, no, on, Off, nULL]"),
        r#"["yes","no","on","Off","nULL"]"#
    );
}

#[test]
fn integers() {
    let cases = [
        ("0", "0"),
        ("42", "42"),
        ("-17", "-17"),
        ("+12", "12"),
        ("007", "7"),
        ("-0", "-0"),
        ("0x1F", "31"),
        ("0xff", "255"),
        ("0o17", "15"),
        (
            "123456789012345678901234567890",
            "123456789012345678901234567890",
        ),
    ];
    for (text, want) in cases {
        assert_eq!(json(text), want, "{text}");
    }
    // A sign is not part of the core schema's hex and octal: these are strings.
    assert_eq!(
        json("[-0x1, 0x, 0o8, 1_000]"),
        r#"["-0x1","0x","0o8","1_000"]"#
    );
}

#[test]
fn floats() {
    let cases = [
        ("1.5", "1.5"),
        ("-0.25", "-0.25"),
        ("+3.0", "3.0"),
        (".5", "0.5"),
        ("-.5", "-0.5"),
        ("1.", "1.0"),
        ("00.5", "0.5"),
        ("1e3", "1e3"),
        ("1.5E-3", "1.5e-3"),
        ("2e+10", "2e10"),
    ];
    for (text, want) in cases {
        assert_eq!(json(text), want, "{text}");
    }
    assert_eq!(json("[1e, ., 1.2.3, e5]"), r#"["1e",".","1.2.3","e5"]"#);
}

#[test]
fn infinities_and_nan_are_refused() {
    for text in [".inf", "-.Inf", "+.INF", ".nan", ".NaN", "a: [1, .NAN]"] {
        assert!(refused(text).contains("no number"), "{text}");
    }
    // Quoted, they are strings.
    assert_eq!(json("'.inf'"), r#"".inf""#);
}

#[test]
fn strings_and_their_escapes() {
    assert_eq!(
        json(r#"["123", '-1.5', "true", 'null', abc]"#),
        r#"["123","-1.5","true","null","abc"]"#
    );
    let text = "\"q\\\" b\\\\ n\\n t\\t r\\r c\\u0001 d\\x7f bell\\a é \\u263A \\U0001F600\"";
    let want = "\"q\\\" b\\\\ n\\n t\\t r\\r c\\u0001 d\u{7f} bell\\u0007 é \u{263a} \u{1f600}\"";
    assert_eq!(json(text), want);
    assert_eq!(
        json("a: |\n  one\n  two\nb: >\n  one\n  two\n"),
        r#"{"a":"one\ntwo\n","b":"one two\n"}"#
    );
    assert_eq!(json("'it''s'"), r#""it's""#);
}

#[test]
fn tags() {
    assert_eq!(json("!!str 12"), r#""12""#);
    assert_eq!(json("! 12"), r#""12""#);
    assert_eq!(json("!!int '12'"), "12");
    assert_eq!(json("!!float 1"), "1");
    assert_eq!(json("!!null ''"), "null");
    assert_eq!(json("!!bool \"true\""), "true");
    assert_eq!(json("!local 12"), "12");
    assert_eq!(json("!<tag:yaml.org,2002:str> 12"), r#""12""#);
    assert!(refused("!!int abc").contains("`!!int`"));
}

#[test]
fn keys_are_strings() {
    assert_eq!(
        json("200: a\ntrue: b\n~: c\n0x10: d\n1.50: e\n'x': f\n"),
        r#"{"200":"a","true":"b","null":"c","16":"d","1.50":"e","x":"f"}"#
    );
    assert!(refused("? [a, b]\n: c\n").contains("as a key"));
    assert!(refused("? {a: b}\n: c\n").contains("as a key"));
    assert!(refused("a: 1\nb: 2\na: 3\n").contains("twice"));
    assert!(refused("1: x\n'1': y\n").contains("twice"));
    assert!(refused("base: &b {x: 1}\nmore:\n  <<: *b\n").contains("merge"));
    assert_eq!(json("'<<': 1"), r#"{"<<":1}"#);
}

#[test]
fn anchors_and_aliases_are_expanded() {
    let text = "\
base: &b {type: object, required: [id]}
copy: *b
list: &l [1, &one 'one', {x: *one}]
again: *l
name: &n key
*n : value
";
    assert_eq!(
        json(text),
        concat!(
            r#"{"base":{"type":"object","required":["id"]},"copy":{"type":"object","required":["id"]},"#,
            r#""list":[1,"one",{"x":"one"}],"again":[1,"one",{"x":"one"}],"name":"key","key":"value"}"#
        )
    );
    // An anchor named again names the newer node from there on.
    assert_eq!(json("[&a 1, *a, &a 2, *a]"), "[1,1,2,2]");
    assert!(refused("a: *nowhere").contains("unknown anchor"));
    assert!(refused("a: &x [1, *x]").contains("inside the node it names"));
    assert!(refused("&k [a]: 1\nb: {*k : 2}").contains("as a key"));
}

#[test]
fn an_alias_bomb_is_refused() {
    let mut text = String::from("a: &a [lol, lol, lol, lol, lol, lol, lol, lol, lol]\n");
    let names = ["b", "c", "d", "e", "f", "g", "h", "i"];
    let mut previous = "a";
    for name in names {
        let items = vec![format!("*{previous}"); 9].join(", ");
        writeln!(text, "{name}: &{name} [{items}]").unwrap();
        previous = name;
    }
    let message = refused(&text);
    assert!(message.contains("alias bomb"), "{message}");
    assert!(message.contains(&MAX_ALIAS_NODES.to_string()), "{message}");
    // A long string copied many times is bounded by bytes.
    let long = "x".repeat(1 << 20);
    let mut text = format!("a: &a '{long}'\nb: [");
    text.push_str(&["*a"; 70].join(", "));
    text.push_str("]\n");
    assert!(refused(&text).contains("alias bomb"));
}

#[test]
fn one_document_only() {
    assert!(refused("a: 1\n---\nb: 2\n").contains("second document"));
    assert!(refused("").contains("no document"));
    assert!(refused("# only a comment\n").contains("no document"));
    assert_eq!(json("---\na: 1\n...\n"), r#"{"a":1}"#);
}

#[test]
fn syntax_errors_name_the_line() {
    let e = yaml::to_json("a: 1\nb: [1, 2\nc: 3\n").expect_err("an unclosed flow sequence");
    assert!(e.line >= 2, "{e}");
    let e = yaml::to_json("a:\n  b: 1\n c: 2\n").expect_err("a bad indentation");
    assert_eq!(e.line, 3, "{e}");
}

#[test]
fn deep_nesting_is_refused_without_exhausting_the_stack() {
    let deep = format!("{}{}", "[".repeat(100_000), "]".repeat(100_000));
    assert!(refused(&deep).contains("recursion limit"));
    let fine = format!("{}{}", "[".repeat(200), "]".repeat(200));
    assert_eq!(json(&fine), fine);
    // Block nesting has no such bound; the core refuses JSON deeper than 512 levels.
    let mut block = String::new();
    for level in 0..2_000 {
        block.push_str(&" ".repeat(level));
        block.push_str("a:\n");
    }
    let j = yaml::to_json(&block).expect("deep block mappings");
    assert_eq!(j.text.matches('{').count(), 2_000);
}

#[test]
fn json_nodes_are_placed_at_their_yaml_line() {
    let text =
        "openapi: 3.1.0\ninfo:\n  title: t\n  version: '1'\npaths: {}\nlist:\n  - x\n  -   y\n";
    let j = yaml::to_json(text).expect("yaml");
    let at = |needle: &str| {
        let offset = j.text.find(needle).expect(needle);
        j.marks
            .place(u32::try_from(offset).expect("small"))
            .expect("a place")
    };
    assert_eq!(at(r#""3.1.0""#), (1, 10));
    assert_eq!(at(r#""title""#), (3, 3));
    assert_eq!(at(r#""t""#), (3, 10));
    assert_eq!(at(r#""1""#), (4, 12));
    assert_eq!(at(r#""y""#), (8, 7));
}
