//! The time of each stage on one description: `cargo run --release --bin phases -- <file.json>`.

use flowapi::{Documents, graph, lower, names, spec};
use std::time::Instant;

fn main() {
    let path = std::env::args().nth(1).expect("a description");
    let text = std::fs::read_to_string(&path).expect("readable");
    let target = flowapi_ts::TypeScript::new("out.ts");
    let t = Instant::now();
    let mut docs = Documents::new();
    docs.add(path.clone(), &text).expect("JSON");
    let parse = t.elapsed();
    let (description, _) = spec::read(&docs);
    let index = spec::index(&docs, &description.all);
    let indexed = t.elapsed();
    let (g, _) = graph::build_with(&docs, &index, &description.named, &description.operations);
    let types = g.len();
    let built = t.elapsed();
    let l = lower::lower(g, flowapi::Target::capabilities(&target));
    let lowered = t.elapsed();
    let (program, _) = names::assign(l, &docs, flowapi::Target::identifiers(&target));
    let named_at = t.elapsed();
    let out = flowapi::Target::emit(&target, &program);
    let emitted = t.elapsed();
    let bytes: usize = out.files.iter().map(|f| f.text.len()).sum();
    println!(
        "parse {:?} | index {:?} | graph {:?} ({types} types) | lower {:?} | names {:?} | emit {:?} ({bytes} bytes) | total {:?}",
        parse,
        indexed - parse,
        built - indexed,
        lowered - built,
        named_at - lowered,
        emitted - named_at,
        emitted
    );
}
