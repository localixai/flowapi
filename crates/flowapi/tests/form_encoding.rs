//! A request's form members by their Encoding Objects, read into `spec::Media::encoding` through
//! the public API: by style where `style`, `explode` or `allowReserved` is written, by a
//! `contentType` alone otherwise, nothing for a member with neither, another media type or a
//! response; Swagger 2.0's array `formData` fields by their `collectionFormat`.
#![expect(
    clippy::expect_used,
    reason = "a description that is not JSON fails the test by panicking"
)]

use flowapi::Documents;
use flowapi::spec::{Description, Style};

/// Each media type's members: name, style, explode, allowReserved and media type.
type Members = Vec<(String, Style, bool, bool, Option<String>)>;

fn read(text: &str) -> Description {
    let mut docs = Documents::new();
    docs.add("t.json", text).expect("JSON");
    let (description, diags) = flowapi::spec::read(&docs);
    assert!(diags.all().is_empty(), "{:?}", diags.all());
    description
}

fn members(media: &flowapi::spec::Media) -> Members {
    let s = media.encoding.iter();
    s.map(|(n, s)| {
        let t = s.media_type.clone();
        (n.clone(), s.style, s.explode, s.allow_reserved, t)
    })
    .collect()
}

// req: oas-3.2.encoding-object.style — tags by its style, n's contentType unused beside a style
#[test]
fn a_forms_encoding_objects_are_read_for_its_request_body_alone() {
    let d = read(
        r#"{"openapi":"3.2.0","info":{"title":"t","version":"1"},"paths":{"/f":{"post":{
        "requestBody":{"content":{
          "application/x-www-form-urlencoded":{"encoding":{
            "tags":{"style":"pipeDelimited"},"path":{"allowReserved":true},
            "n":{"contentType":"application/json","style":"form"},
            "id":{"contentType":" application/json "},"a":{"headers":{}},"e":{"contentType":""}}},
          "multipart/form-data":{"encoding":{"x":{"contentType":"text/plain"}}}}},
        "responses":{"200":{"description":"ok","content":{"application/x-www-form-urlencoded":{
          "encoding":{"y":{"style":"form"}}}}}}}}}}"#,
    );
    let op = &d.operations[0];
    let content = &op.body.as_ref().expect("a body").content;
    let json = Some(String::from("application/json"));
    assert_eq!(
        members(&content[0]),
        [
            ("tags".into(), Style::PipeDelimited, false, false, None),
            ("path".into(), Style::Form, true, true, None),
            ("n".into(), Style::Form, true, false, None),
            ("id".into(), Style::Form, true, false, json),
        ]
    );
    assert!(content[1].encoding.is_empty(), "multipart/form-data");
    assert!(op.responses[0].content[0].encoding.is_empty(), "a response");
}

// req: oas-2.0.parameter-object.collectionFormat — a formData array by its format, as in a query
#[test]
fn swagger_form_arrays_are_written_by_their_collection_format() {
    let d = read(
        r#"{"swagger":"2.0","info":{"title":"t","version":"1"},"paths":{"/f":{"post":{
        "parameters":[{"name":"name","in":"formData","type":"string"},
          {"name":"tags","in":"formData","type":"array","items":{"type":"string"}},
          {"name":"m","in":"formData","type":"array","items":{"type":"string"},
           "collectionFormat":"multi"}],"responses":{}}}}}"#,
    );
    let content = &d.operations[0].body.as_ref().expect("a body").content;
    assert_eq!(
        members(&content[0]),
        [
            ("tags".into(), Style::Form, false, false, None),
            ("m".into(), Style::Form, true, false, None),
        ]
    );
}
