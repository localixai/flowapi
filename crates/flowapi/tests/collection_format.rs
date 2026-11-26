//! Swagger 2.0's `collectionFormat` read into the operations' styles, through the public API:
//! each format in each location 2.0 allows it, an Items Object's own for the arrays inside the
//! value (outermost first), and a format a place does not take reported and read as `csv`.
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use flowapi::spec::{Location, Style};

/// Each parameter's name, location, style, explode and nested styles; and the diagnostics' codes.
type Read = (
    Vec<(String, Location, Style, bool, Vec<Style>)>,
    Vec<String>,
);

fn read(parameters: &str) -> Read {
    let text = format!(
        r#"{{"swagger":"2.0","info":{{"title":"t","version":"1"}},
        "paths":{{"/{{p}}":{{"get":{{"parameters":{parameters},"responses":{{}}}}}}}}}}"#
    );
    let mut docs = Documents::new();
    docs.add("t.json", &text).expect("JSON");
    let (description, diags) = flowapi::spec::read(&docs);
    let params = description
        .operations
        .iter()
        .flat_map(|op| &op.params)
        .map(|p| {
            let s = &p.serialization;
            (
                p.name.clone(),
                p.location,
                s.style,
                s.explode,
                s.nested.clone(),
            )
        })
        .collect();
    let codes = diags.all().iter().map(|d| d.code.to_owned()).collect();
    (params, codes)
}

fn array(name: &str, location: &str, format: &str) -> String {
    format!(
        r#"{{"name":"{name}","in":"{location}","required":true,"type":"array",
        "items":{{"type":"string"}},"collectionFormat":"{format}"}}"#
    )
}

// req: oas-2.0.parameter-object.collectionFormat — every format in each location that takes it
#[test]
fn each_format_in_each_location() {
    use Location::{Header, Path, Query};
    use Style::{Form, PipeDelimited, Simple, SpaceDelimited, TabDelimited};
    let mut written = Vec::new();
    let mut expected = Vec::new();
    for (location, csv) in [(Path, Simple), (Query, Form), (Header, Simple)] {
        for (format, style) in [
            ("csv", csv),
            ("ssv", SpaceDelimited),
            ("tsv", TabDelimited),
            ("pipes", PipeDelimited),
        ] {
            let name = format!("{format}-{}", location.name());
            written.push(array(&name, location.name(), format));
            expected.push((name, location, style, false, Vec::new()));
        }
    }
    written.push(array("multi", "query", "multi"));
    expected.push((String::from("multi"), Query, Form, true, Vec::new()));
    let (params, codes) = read(&format!("[{}]", written.join(",")));
    assert_eq!(params, expected);
    assert!(codes.is_empty(), "{codes:?}");
}

// req: oas-2.0.parameter-object.collectionFormat — `multi` only in a query, an unknown format nowhere
#[test]
fn a_format_a_place_does_not_take_is_csv_with_a_warning() {
    let written = [
        array("p", "path", "multi"),
        array("h", "header", "multi"),
        array("q", "query", "semicolons"),
    ];
    let (params, codes) = read(&format!("[{}]", written.join(",")));
    let styles: Vec<_> = params.iter().map(|p| (p.2, p.3)).collect();
    assert_eq!(
        styles,
        [
            (Style::Simple, false),
            (Style::Simple, false),
            (Style::Form, false)
        ]
    );
    assert_eq!(codes, ["swagger.collectionFormat"; 3]);
}

// req: oas-2.0.items-object.collectionFormat — the arrays inside the value, by their own formats
#[test]
fn an_items_object_joins_the_arrays_inside_the_value() {
    let grid = r#"[{"name":"grid","in":"query","type":"array","collectionFormat":"multi",
        "items":{"type":"array","collectionFormat":"pipes",
          "items":{"type":"array","items":{"type":"array","collectionFormat":"ssv",
            "items":{"type":"integer","collectionFormat":"tsv"}}}}},
        {"name":"bad","in":"header","type":"array",
        "items":{"type":"array","collectionFormat":"multi","items":{"type":"string"}}}]"#;
    let (params, codes) = read(grid);
    let nested: Vec<_> = params.iter().map(|p| (p.3, p.4.clone())).collect();
    // The integers' own `collectionFormat` says nothing: they are not an array.
    assert_eq!(
        nested,
        [
            (
                true,
                vec![Style::PipeDelimited, Style::Simple, Style::SpaceDelimited]
            ),
            (false, vec![Style::Simple])
        ]
    );
    assert_eq!(codes, ["swagger.collectionFormat"]);
}
