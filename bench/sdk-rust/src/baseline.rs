//! The same requests written the way progenitor's generated clients write them
//! (progenitor-client/src/progenitor_client.rs: `encode_path` with its path set, 560-578;
//! `QueryParam` and its serializer, 599-640, each parameter's pairs given to serde_urlencoded
//! on one form_urlencoded serializer, as reqwest's `.query` does; serde_json for a body). A
//! deepObject, which progenitor does not write, is its `name[key]` pairs.

use serde::Serialize;

/// A request's four parts, as `sdk::Request` holds them.
pub(crate) struct Raw {
    pub(crate) path: String,
    pub(crate) query: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Option<Vec<u8>>,
}

const PATH_SET: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'%');

fn encode_path(pc: &str) -> String {
    percent_encoding::utf8_percent_encode(pc, PATH_SET).to_string()
}

/// One parameter's pairs appended, as `QueryParam` does for a scalar and for each item.
fn pair<T: Serialize>(q: &mut form_urlencoded::Serializer<'_, String>, name: &str, v: T) {
    let _ = [(name, v)].serialize(serde_urlencoded::Serializer::new(q));
}

fn done(path: String, query: String, headers: Vec<(String, String)>, body: Option<Vec<u8>>) -> Raw {
    Raw {
        path,
        query,
        headers,
        body,
    }
}

pub(crate) fn get_repo(owner: &str, repo: &str, verbose: Option<bool>) -> Raw {
    let path = format!("/repos/{}/{}", encode_path(owner), encode_path(repo));
    let mut q = form_urlencoded::Serializer::new(String::new());
    if let Some(v) = verbose {
        pair(&mut q, "verbose", v);
    }
    done(path, q.finish(), Vec::new(), None)
}

pub(crate) fn search(tag: Option<&[String]>, filter: &[(&str, &str)], page: Option<i64>) -> Raw {
    let mut q = form_urlencoded::Serializer::new(String::new());
    for t in tag.into_iter().flatten() {
        pair(&mut q, "tag", t);
    }
    for (k, v) in filter {
        pair(&mut q, &format!("filter[{k}]"), v);
    }
    if let Some(p) = page {
        pair(&mut q, "page", p);
    }
    done(String::from("/search"), q.finish(), Vec::new(), None)
}

pub(crate) fn put_pet(
    id: i64,
    dry: Option<bool>,
    request_id: Option<&str>,
    body: &serde_json::Value,
) -> Raw {
    let path = format!("/pets/{}", encode_path(&id.to_string()));
    let mut q = form_urlencoded::Serializer::new(String::new());
    if let Some(v) = dry {
        pair(&mut q, "dry", v);
    }
    let mut headers = Vec::new();
    if let Some(r) = request_id {
        headers.push((String::from("X-Request-Id"), r.to_owned()));
    }
    let body = serde_json::to_vec(body).ok();
    done(path, q.finish(), headers, body)
}
