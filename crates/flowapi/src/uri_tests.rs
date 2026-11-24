//! RFC 3986 §5.4's examples, against the base it gives.

use super::resolve;

const BASE: &str = "http://a/b/c/d;p?q";

#[test]
fn normal_examples() {
    for (reference, expected) in [
        ("g:h", "g:h"),
        ("g", "http://a/b/c/g"),
        ("./g", "http://a/b/c/g"),
        ("g/", "http://a/b/c/g/"),
        ("/g", "http://a/g"),
        ("//g", "http://g"),
        ("?y", "http://a/b/c/d;p?y"),
        ("g?y", "http://a/b/c/g?y"),
        ("#s", "http://a/b/c/d;p?q#s"),
        ("g#s", "http://a/b/c/g#s"),
        ("g?y#s", "http://a/b/c/g?y#s"),
        (";x", "http://a/b/c/;x"),
        ("g;x", "http://a/b/c/g;x"),
        ("", "http://a/b/c/d;p?q"),
        (".", "http://a/b/c/"),
        ("./", "http://a/b/c/"),
        ("..", "http://a/b/"),
        ("../", "http://a/b/"),
        ("../g", "http://a/b/g"),
        ("../..", "http://a/"),
        ("../../", "http://a/"),
        ("../../g", "http://a/g"),
    ] {
        assert_eq!(resolve(BASE, reference), expected, "{reference}");
    }
}

#[test]
fn abnormal_examples() {
    for (reference, expected) in [
        ("../../../g", "http://a/g"),
        ("/./g", "http://a/g"),
        ("/../g", "http://a/g"),
        ("g.", "http://a/b/c/g."),
        ("..g", "http://a/b/c/..g"),
        ("./../g", "http://a/b/g"),
        ("g/./h", "http://a/b/c/g/h"),
        ("g/../h", "http://a/b/c/h"),
        ("g;x=1/../y", "http://a/b/c/y"),
    ] {
        assert_eq!(resolve(BASE, reference), expected, "{reference}");
    }
}

#[test]
fn urns_and_bare_names() {
    assert_eq!(resolve("urn:example:root", "#foo"), "urn:example:root#foo");
    assert_eq!(resolve("schema.json", "other.json#/a"), "other.json#/a");
    assert_eq!(resolve("dir/schema.json", "other.json"), "dir/other.json");
}
