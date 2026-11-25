//! One document as the core is given it: which text is YAML (the rule both hosts follow), the
//! JSON it is read into, the line and column of a byte offset, and where a refused YAML text is.

use flowapi_yaml::{Text, is_yaml};
use std::borrow::Cow;

#[test]
fn yaml_is_known_by_the_name_then_by_the_text() {
    assert!(is_yaml("a/openapi.yaml", "{}"));
    assert!(is_yaml("openapi.YML", "[]"));
    assert!(is_yaml("C:\\specs\\openapi.yml", "{}"));
    assert!(!is_yaml("openapi.json", "openapi: 3.1.0"));
    assert!(!is_yaml("x/openapi.JSON", "openapi: 3.1.0"));
    // Any other name, or none: YAML unless the text opens with `{` or `[`.
    assert!(is_yaml("openapi", "openapi: 3.1.0"));
    assert!(is_yaml("openapi.txt", "openapi: 3.1.0"));
    assert!(!is_yaml("openapi", " \n\u{feff}{\"openapi\":\"3.1.0\"}"));
    assert!(!is_yaml("https://example.com/spec", "[1]"));
    // A dot that opens a name is no extension, as for a path; a dot in a directory is not one.
    assert!(!is_yaml(".yaml", "{}"));
    assert!(!is_yaml("dir.yaml/spec", "{}"));
}

#[test]
fn json_is_given_as_it_is() {
    let text = "{\n  \"a\": [1,\n  2]\n}";
    let read = Text::read("a.json", Cow::Borrowed(text)).expect("JSON is not read");
    assert_eq!(read.json(), text);
    let offset = u32::try_from(text.find('2').expect("a 2")).expect("small");
    assert_eq!(read.place(offset), (3, 3));
    assert_eq!(read.place(0), (1, 1));
}

#[test]
fn yaml_is_read_into_json_placed_at_its_lines() {
    let read = Text::read("a.yaml", Cow::Borrowed("a:\n  - 1\n  - 2\n")).expect("YAML");
    assert_eq!(read.json(), r#"{"a":[1,2]}"#);
    let offset = u32::try_from(read.json().find('2').expect("a 2")).expect("small");
    assert_eq!(read.place(offset), (3, 5));
}

#[test]
fn a_refused_yaml_text_names_its_byte_offset_line_and_column() {
    // The parser counts characters; the offset is in bytes: `é` is two.
    let text = "é: 1\nb:\n  c: 2\n d: 3\n";
    let e = Text::read("bad.yaml", Cow::Borrowed(text)).expect_err("a bad indentation");
    assert_eq!((e.line, e.column), (4, 2));
    assert_eq!(e.offset, text.find("d: 3").expect("the key"));
}
