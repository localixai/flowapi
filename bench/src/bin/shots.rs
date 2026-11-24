//! The README's picture of what flowapi prints: `shots/pets.json` beside the TypeScript the
//! library makes of it, drawn as the lab draws code (dark, monospace) into `../assets/output-dark`
//! as SVG and as PNG at twice the size. The output is generated here, never pasted.

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::fmt::Write as _;
use std::fs;

const PAPER: &str = "#0b0b0b";
const PANE: &str = "#141413";
const EDGE: &str = "#2e2e2c";
const INK: &str = "#f4f4f2";
const QUIET: &str = "#8f8f8c";
const KEYWORD: &str = "#a594f9";
const STRING: &str = "#d6c58f";
const NUMBER: &str = "#8fc7b0";

const LINE: f64 = 17.0;
const CHAR: f64 = 7.22;
const PAD: f64 = 22.0;

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A line split into coloured runs: strings, numbers, comments and the words of TypeScript.
fn runs(line: &str) -> Vec<(&'static str, String)> {
    if line.trim_start().starts_with("//")
        || line.trim_start().starts_with("/*")
        || line.trim_start().starts_with('*')
    {
        return vec![(QUIET, line.to_owned())];
    }
    let mut out: Vec<(&'static str, String)> = Vec::new();
    let mut chars = line.chars().peekable();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut Vec<(&'static str, String)>| {
        if word.is_empty() {
            return;
        }
        let colour = match word.as_str() {
            "export" | "type" | "string" | "number" | "boolean" | "null" | "unknown" | "never"
            | "true" | "false" => KEYWORD,
            w if w.chars().all(|c| c.is_ascii_digit()) => NUMBER,
            _ => INK,
        };
        out.push((colour, std::mem::take(word)));
    };
    while let Some(c) = chars.next() {
        if c == '"' || c == '`' {
            flush(&mut word, &mut out);
            let mut s = String::from(c);
            for d in chars.by_ref() {
                s.push(d);
                if d == c {
                    break;
                }
            }
            out.push((STRING, s));
        } else if c.is_alphanumeric() || c == '_' || c == '$' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push((QUIET, c.to_string()));
        }
    }
    flush(&mut word, &mut out);
    out
}

fn pane(svg: &mut String, x: f64, y: f64, w: f64, h: f64, title: &str, text: &str) {
    let _ = writeln!(
        svg,
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" rx=\"8\" fill=\"{PANE}\" stroke=\"{EDGE}\"/>"
    );
    let _ = writeln!(
        svg,
        "<text x=\"{:.1}\" y=\"{:.1}\" fill=\"{QUIET}\" font-size=\"11.5\">{}</text>",
        x + PAD,
        y + 22.0,
        escape(title)
    );
    for (i, line) in text.lines().enumerate() {
        let _ = write!(
            svg,
            "<text x=\"{:.1}\" y=\"{:.1}\" font-size=\"12\" xml:space=\"preserve\">",
            x + PAD,
            y + 48.0 + i as f64 * LINE
        );
        for (colour, run) in runs(line) {
            let _ = write!(svg, "<tspan fill=\"{colour}\">{}</tspan>", escape(&run));
        }
        svg.push_str("</text>\n");
    }
}

fn main() {
    let input = fs::read_to_string("shots/pets.json").expect("the sample");
    let mut docs = flowapi::Documents::new();
    docs.add("pets.json", &input).expect("JSON");
    let (output, _) = flowapi::generate(&docs, &flowapi_ts::TypeScript::new("pets.ts"));
    let printed = output.expect("the sample generates").files.remove(0).text;
    let printed: String = printed.lines().skip(2).collect::<Vec<_>>().join("\n");
    // The description without its `openapi` and `info` lines, one level out.
    let left: String = input
        .lines()
        .skip(3)
        .take_while(|l| *l != "}")
        .map(|l| l.get(2..).unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
    let lines = left.lines().count().max(printed.lines().count()) as f64;
    let wide_left = left.lines().map(str::len).max().unwrap_or(0) as f64 * CHAR + 2.0 * PAD;
    let wide_right = printed.lines().map(str::len).max().unwrap_or(0) as f64 * CHAR + 2.0 * PAD;
    let (gap, edge) = (16.0, 24.0);
    let high = 48.0 + lines * LINE + 18.0;
    let (w, h) = (2.0 * edge + wide_left + gap + wide_right, 2.0 * edge + high);
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\" \
         font-family=\"'JetBrains Mono', 'SF Mono', Menlo, 'DejaVu Sans Mono', monospace\">\n\
         <rect x=\"0.5\" y=\"0.5\" width=\"{:.0}\" height=\"{:.0}\" rx=\"12\" fill=\"{PAPER}\" stroke=\"{EDGE}\"/>\n",
        w - 1.0,
        h - 1.0
    );
    pane(&mut svg, edge, edge, wide_left, high, "openapi.json", &left);
    pane(
        &mut svg,
        edge + wide_left + gap,
        edge,
        wide_right,
        high,
        "flowapi openapi.json -o types.ts",
        &printed,
    );
    svg.push_str("</svg>\n");
    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = Tree::from_str(&svg, &options).expect("SVG");
    let size = tree.size();
    let mut pixels = Pixmap::new(
        (size.width() * 2.0).ceil() as u32,
        (size.height() * 2.0).ceil() as u32,
    )
    .expect("a size");
    resvg::render(&tree, Transform::from_scale(2.0, 2.0), &mut pixels.as_mut());
    fs::create_dir_all("../assets").expect("assets/");
    fs::write(
        "../assets/output-dark.png",
        pixels.encode_png().expect("PNG"),
    )
    .expect("PNG written");
    fs::write("../assets/output-dark.svg", svg).expect("SVG written");
    println!("wrote ../assets/output-dark.svg and output-dark.png");
}
