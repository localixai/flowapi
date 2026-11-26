//! The readers of the texts the SDK follows, each on a small fixture: OpenAPI 2.0's Markdown
//! (tables without a leading `|`), an RFC's plain text and an RFCXML v3 source.

use flowapi_ledger::markers::ids_in;
use flowapi_ledger::requirements::Requirement;
use flowapi_ledger::sources::{Format, SOURCES, Source};
use flowapi_ledger::{extract, markdown, rfc_text, rfc_xml, xml};

fn source(format: Format, version: &str) -> Source {
    SOURCES
        .iter()
        .find(|s| s.format == format && s.version == version)
        .copied()
        .unwrap()
}

fn ids(found: &[Requirement]) -> Vec<&str> {
    found.iter().map(|r| r.id.as_str()).collect()
}

#[test]
fn openapi_2_tables_without_a_leading_pipe_give_fields() {
    let text = "# OpenAPI Specification\n\n### Schema\n\n#### Security Scheme Object\n\n##### Fixed Fields\nField Name | Type | Validity | Description\n---|:---:|---|---\n<a name=\"securitySchemeType\"></a>type | `string` | Any | **Required.** The type. Valid values are `\"basic\"`.\n<a name=\"securitySchemeTokenUrl\"></a>tokenUrl | `string` | `oauth2` | **Required.** The token URL. This SHOULD be in the form of a URL.\n\nA plain line with a | in it is prose.\n";
    let found = extract::requirements(&markdown::read(text), &source(Format::Markdown, "2.0"));
    assert_eq!(
        ids(&found),
        [
            "oas-2.0.security-scheme-object.type",
            "oas-2.0.security-scheme-object.tokenUrl",
            "oas-2.0.security-scheme-object.tokenUrl.should-form-url",
        ]
    );
    assert_eq!(
        found.get(1).map(|r| r.anchor.as_str()),
        Some("securitySchemeTokenUrl")
    );
    assert_eq!(found.get(2).map(|r| r.line), Some(11));
}

const RFC_TEXT: &str = "Internet Engineering Task Force (IETF)                          A. Author
Request for Comments: 9999

Abstract

   This document MUST be ignored: it is the abstract.

Status of This Memo

   The text of the memo SHALL not count.

1.  Introduction

   The key words \"MUST\", \"MUST NOT\" and \"MAY\" in this document are to be
   interpreted as described in [RFC2119].

4.4.  A Grant

   The client MUST send the user-
   agent away.  A client must not count in lower case.

     POST /token HTTP/1.1
   The server MAY answer.

   o  The token is OPTIONAL.

   grant_type
         REQUIRED.  Value MUST be set to \"client_credentials\".  The client
         MUST keep the



Author                       Standards Track                    [Page 4]
\u{c}
RFC 9999                         A Grant                     October 2012


         secret.

Appendix A.  Syntax

   A.1 states nothing.

A.1.  The Name

   A name SHOULD be short.

5.  References

   [RFC2119]  Bradner, S., \"Key words\" MUST be read.

Author's Address

   A. Author MUST be reachable.
";

#[test]
fn an_rfc_text_gives_numbered_sections_and_skips_the_boilerplate() {
    let doc = rfc_text::read(RFC_TEXT);
    let found = extract::requirements(&doc, &source(Format::RfcText, "6749"));
    assert_eq!(
        ids(&found),
        [
            "rfc6749.4.4.must-client-send-user-agent-away",
            "rfc6749.4.4.may-server-answer",
            "rfc6749.4.4.optional-token",
            "rfc6749.4.4.required-grant-type",
            "rfc6749.4.4.must-value-set-client-credentials",
            "rfc6749.4.4.must-client-keep-secret",
            "rfc6749.A.1.should-name-short",
        ]
    );
    let texts: Vec<&str> = found.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(
        texts.first(),
        Some(&"The client MUST send the user-agent away.")
    );
    assert_eq!(texts.get(5), Some(&"The client MUST keep the secret."));
    let last = found.last().unwrap();
    assert_eq!(
        (last.anchor.as_str(), last.section.as_str()),
        ("appendix-A.1", "Syntax > The Name")
    );
    assert_eq!(found.first().map(|r| r.line), Some(19));
}

#[test]
fn an_rfc_heading_is_a_number_at_column_0() {
    assert_eq!(
        rfc_text::heading("4.4.2.  Access Token Response"),
        Some(("4.4.2".to_owned(), "Access Token Response"))
    );
    assert_eq!(
        rfc_text::heading("Appendix B.  Use of a Media Type"),
        Some(("B".to_owned(), "Use of a Media Type"))
    );
    assert!(rfc_text::heading("   4.4.  Indented").is_none());
    assert!(rfc_text::heading("Request for Comments: 6749").is_none());
    assert!(rfc_text::heading("A. Author MUST be reachable.").is_none());
}

const RFC_XML: &str = r#"<?xml version='1.0' encoding='utf-8'?>
<rfc version="3" number="9110">
  <front>
    <boilerplate>
      <section numbered="false" pn="section-boilerplate.1"><name>Status of This Memo</name>
        <t>This memo MUST not count.</t></section>
    </boilerplate>
  </front>
  <middle>
    <section anchor="auth" numbered="true" pn="section-11">
      <name slugifiedName="name-http-authentication">HTTP Authentication</name>
      <section anchor="credentials" numbered="true" pn="section-11.4">
        <name>Credentials</name>
        <iref item="Grammar" subitem="credentials"/>
        <t>A user agent <bcp14>MUST</bcp14> send them as <xref target="RFC7617" format="default" derivedContent="RFC7617"/> says, per <xref target="field.authorization" format="default" derivedContent="Section 11.6.2"/>.</t>
        <sourcecode type="abnf9110">credentials = auth-scheme MUST</sourcecode>
        <ul><li><t>A token <bcp14>SHOULD NOT</bcp14> be reused.</t></li><li>Each list item <bcp14>MAY</bcp14> be read.</li></ul>
        <dl><dt>realm:</dt><dd>The realm <bcp14>MUST</bcp14> be quoted.</dd></dl>
      </section>
    </section>
  </middle>
  <back>
    <references pn="section-19"><name>References</name></references>
    <section numbered="true" pn="section-appendix.b.1"><name>Changes</name>
      <t>An appendix <bcp14>MAY</bcp14> state it.</t></section>
    <section numbered="false" pn="section-appendix.e"><name>Authors' Addresses</name>
      <t>An author <bcp14>MUST</bcp14> not count.</t></section>
  </back>
</rfc>
"#;

#[test]
fn an_rfcxml_v3_source_gives_its_numbered_sections() {
    let root = xml::parse(RFC_XML).unwrap();
    let found = extract::requirements(&rfc_xml::read(&root), &source(Format::RfcXml, "9110"));
    assert_eq!(
        ids(&found),
        [
            "rfc9110.11.4.must-user-agent-send-them-rfc7617-says",
            "rfc9110.11.4.should-not-token-reused",
            "rfc9110.11.4.may-each-list-item-read",
            "rfc9110.11.4.must-realm-realm-quoted",
            "rfc9110.B.1.may-appendix-state",
        ]
    );
    assert_eq!(
        found.first().map(|r| r.text.as_str()),
        Some("A user agent MUST send them as [RFC7617] says, per Section 11.6.2.")
    );
    assert_eq!(
        found
            .first()
            .map(|r| (r.anchor.as_str(), r.section.as_str())),
        Some(("section-11.4", "HTTP Authentication > Credentials"))
    );
    assert_eq!(
        found.last().map(|r| r.anchor.as_str()),
        Some("appendix-B.1")
    );
    assert_eq!(
        rfc_xml::number("section-appendix.b.1").as_deref(),
        Some("B.1")
    );
}

#[test]
fn markers_name_rfc_ids_too() {
    assert_eq!(
        ids_in("// req: rfc6750.2.1.must-clients-use-bearer-authentication-scheme — sent"),
        ["rfc6750.2.1.must-clients-use-bearer-authentication-scheme"]
    );
    assert!(ids_in("req: rfcs are texts").is_empty());
}
