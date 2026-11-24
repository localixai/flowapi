//! What writing a request costs: the Rust SDK flowapi generates (`<op>_request`, the whole
//! request: path, query, headers, body) against the way progenitor's generated clients write
//! the same request (`progenitor_client::encode_path` for each path value, each query
//! parameter's pairs through serde_urlencoded into one form_urlencoded serializer, as its
//! `QueryParam` does, serde_json for the body). The shapes are bench/competitors/sdk.mjs's: a
//! path and a query flag; an array, a deepObject and an integer in the query; a path, a query,
//! a header and a JSON body. Each pair is first checked to mean the same request (the same
//! path decoded, the same query pairs decoded, the same body). 100,000 requests a round, 5
//! rounds after a warm-up, the median kept. Writes `bench/results/sdk-rust.md`.

#[allow(dead_code, clippy::all)]
mod api {
    include!(concat!(env!("OUT_DIR"), "/api.rs"));
}
mod baseline;

use serde_json::json;
use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 100_000;
const ROUNDS: usize = 5;

/// What a request is, decoded: the path, the query's pairs, the headers, the body.
type Meaning = (
    String,
    Vec<(String, String)>,
    Vec<(String, String)>,
    Option<Vec<u8>>,
);

fn meaning(path: &str, query: &str, headers: &[(String, String)], body: Option<&[u8]>) -> Meaning {
    let pairs = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let path = percent_encoding::percent_decode_str(path)
        .decode_utf8_lossy()
        .into_owned();
    (path, pairs, headers.to_vec(), body.map(<[u8]>::to_vec))
}

fn ours(r: &api::sdk::Request) -> Meaning {
    meaning(
        &r.path,
        &r.query,
        &r.headers,
        r.body.as_ref().map(|b| b.1.as_slice()),
    )
}

fn theirs(r: &baseline::Raw) -> Meaning {
    meaning(&r.path, &r.query, &r.headers, r.body.as_deref())
}

/// The median of `ROUNDS` rounds of `N` calls, after one round of warm-up, in nanoseconds a call.
fn time(mut f: impl FnMut() -> usize) -> f64 {
    let mut rounds = Vec::with_capacity(ROUNDS);
    for round in 0..=ROUNDS {
        let started = Instant::now();
        let mut sink = 0;
        for _ in 0..N {
            sink += f();
        }
        black_box(sink);
        if round > 0 {
            rounds.push(started.elapsed().as_nanos() as f64 / N as f64);
        }
    }
    rounds.sort_by(f64::total_cmp);
    rounds[ROUNDS / 2]
}

fn main() {
    let repo: api::GetRepoInput = serde_json::from_value(json!({
        "parameters": { "path": { "owner": "flowapi org", "repo": "sdk/rust" }, "query": { "verbose": true } }
    }))
    .expect("the getRepo input");
    let search: api::SearchInput = serde_json::from_value(json!({
        "parameters": { "query": { "tag": ["a b", "c&d", "e"], "filter": { "color": "blue", "size": "L" }, "page": 3 } }
    }))
    .expect("the search input");
    let body = json!({ "name": "Rex", "tags": ["a", "b"] });
    let pet: api::PutPetInput = serde_json::from_value(json!({
        "parameters": { "path": { "id": 42 }, "query": { "dry": false }, "header": { "X-Request-Id": "abc-123" } },
        "body": { "application/json": body }
    }))
    .expect("the putPet input");
    let tags = vec![String::from("a b"), String::from("c&d"), String::from("e")];
    let filter = [("color", "blue"), ("size", "L")];

    let pairs = [
        (
            ours(&api::get_repo_request(&repo)),
            theirs(&baseline::get_repo("flowapi org", "sdk/rust", Some(true))),
        ),
        (
            ours(&api::search_request(&search)),
            theirs(&baseline::search(Some(&tags), &filter, Some(3))),
        ),
        (
            ours(&api::put_pet_request(&pet)),
            theirs(&baseline::put_pet(42, Some(false), Some("abc-123"), &body)),
        ),
    ];
    for (i, (a, b)) in pairs.iter().enumerate() {
        assert_eq!(a, b, "shape {i}: the two requests differ");
    }

    let rows = [
        (
            "getRepo: path and a query flag",
            time(|| api::get_repo_request(&repo).path.len()),
            time(|| {
                baseline::get_repo("flowapi org", "sdk/rust", Some(true))
                    .path
                    .len()
            }),
        ),
        (
            "search: an array, a deepObject, an integer",
            time(|| api::search_request(&search).query.len()),
            time(|| baseline::search(Some(&tags), &filter, Some(3)).query.len()),
        ),
        (
            "putPet: path, query, header, JSON body",
            time(|| api::put_pet_request(&pet).body.map_or(0, |b| b.1.len())),
            time(|| {
                baseline::put_pet(42, Some(false), Some("abc-123"), &body)
                    .body
                    .map_or(0, |b| b.len())
            }),
        ),
    ];
    let cores = std::thread::available_parallelism().map_or(0, usize::from);
    let mut out = format!(
        "# Writing a request: the Rust SDK against progenitor's way\n\nWritten by `cargo run \
         --release -j 1` in `bench/sdk-rust`; do not edit. {N} requests a round, the median of \
         {ROUNDS} rounds after a warm-up, nanoseconds a request, one thread on a machine of \
         {cores} cores. flowapi's is the whole `<op>_request` call into its `sdk::Request`; \
         progenitor's is its `encode_path`, its `QueryParam` pairs through serde_urlencoded \
         and serde_json for the body, into the same four parts (src/baseline.rs). Both mean \
         the same request (checked first: path, query pairs and body decoded alike).\n\n\
         | Shape | flowapi (ns) | progenitor's way (ns) | Ratio |\n| --- | ---: | ---: | ---: |\n"
    );
    for (name, a, b) in rows {
        let _ = writeln!(out, "| {name} | {a:.0} | {b:.0} | {:.2}x |", b / a);
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../results/sdk-rust.md");
    std::fs::write(path, &out).expect("the results are writable");
    print!("{out}");
}
