//! The requirements ledger of flowapi (oracle O1).
//!
//! Every normative statement of the pinned specification texts (`conformance/specs/`: OpenAPI,
//! JSON Schema and the RFCs the SDK follows) becomes a requirement with a stable id: each sentence
//! carrying an RFC 2119 / RFC 8174 keyword, each row of an OpenAPI field table, each JSON Schema
//! keyword. `scope.json` classifies the ids, `req:` markers
//! in flowapi's tests cover them, `lossy:` markers say where the type is wider and why, and
//! `REPORT.md` counts what is covered, what is lossy and what is missing; `MISSING.md` lists the
//! missing ones.

pub mod doc;
pub mod extract;
pub mod ids;
pub mod markdown;
pub mod markers;
pub mod missing;
pub mod paths;
pub mod report;
pub mod requirements;
pub mod rfc;
pub mod rfc_text;
pub mod rfc_xml;
pub mod scope;
pub mod sources;
pub mod terms;
pub mod text;
pub mod xml;
