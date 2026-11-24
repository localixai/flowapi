//! The README's chart, drawn from the results: how often each generator's types are right on the
//! official JSON Schema Test Suite (`../conformance/results/compare.json`), and how long each takes
//! and how much memory it holds on real descriptions (`results/speed.json`). Panels three to a row,
//! each with its own scale; dark, as the lab shows every demo; an SVG and a PNG at twice the size.

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use serde_json::Value;
use std::fmt::Write as _;
use std::fs;

const TOOLS: [&str; 3] = ["flowapi", "openapi-typescript", "hey-api"];

/// The lab's neutral dark: greys with no blue cast, ours in violet.
const PAPER: &str = "#0b0b0b";
const EDGE_INK: &str = "#2e2e2c";
const INK: &str = "#f4f4f2";
const QUIET: &str = "#b0b0ad";
const OURS: &str = "#a594f9";
const THEIRS: &str = "#3d3d3a";

/// A bar: its length's value, and what is written beside it; `None` when the tool failed.
type Bar = Option<(f64, String)>;

struct Panel {
    title: String,
    under: String,
    bars: Vec<Bar>,
}

struct Section {
    title: &'static str,
    under: String,
    panels: Vec<Panel>,
}

fn read(path: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("run the bench and the oracles first"))
        .expect("JSON")
}

fn number(v: &Value, path: &[&str]) -> f64 {
    let mut at = v;
    for key in path {
        at = &at[*key];
    }
    at.as_f64()
        .or_else(|| at.as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

fn correctness(compare: &Value) -> Section {
    let totals = |tool: &str| &compare["tools"][tool]["totals"];
    let percent = |key: &'static str| -> Vec<Bar> {
        TOOLS
            .iter()
            .map(|t| {
                let p = number(totals(t), &[key]);
                Some((p, format!("{p:.1}%")))
            })
            .collect()
    };
    let broken: Vec<Bar> = TOOLS
        .iter()
        .map(|t| {
            let n = number(totals(t), &["generation_errors"])
                + number(totals(t), &["invalid_typescript"]);
            Some((n, format!("{n:.0}")))
        })
        .collect();
    let valid = number(totals("flowapi"), &["valid_cases"]);
    let invalid = number(totals("flowapi"), &["invalid_cases"]);
    let groups = number(totals("flowapi"), &["groups_run"]);
    Section {
        title: "Correct types",
        under: format!(
            "the official JSON Schema Test Suite, each case a literal of the type printed for it, checked by TypeScript {}",
            compare["typescript"].as_str().unwrap_or("")
        ),
        panels: vec![
            Panel {
                title: "Valid values accepted".into(),
                under: format!("of {valid:.0} · under 100% is a type that lies"),
                bars: percent("soundness_percent"),
            },
            Panel {
                title: "Invalid values rejected".into(),
                under: format!("of {invalid:.0} · higher is better"),
                bars: percent("precision_percent"),
            },
            Panel {
                title: "Broken output".into(),
                under: format!("of {groups:.0} groups: not generated, or not TS"),
                bars: broken,
            },
        ],
    }
}

fn speed(speed: &Value) -> Section {
    let rows = speed["rows"].as_array().cloned().unwrap_or_default();
    let row = |file: &str, tool: &str| {
        rows.iter()
            .find(|r| r["file"] == file && r["tool"] == tool)
            .cloned()
    };
    let time = |file: &str| -> Vec<Bar> {
        TOOLS
            .iter()
            .map(|t| {
                row(file, t).filter(|r| r["ok"] == true).map(|r| {
                    let ms = number(&r, &["medianMs"]);
                    (
                        ms,
                        if ms >= 1000.0 {
                            format!("{:.2} s", ms / 1000.0)
                        } else {
                            format!("{ms:.0} ms")
                        },
                    )
                })
            })
            .collect()
    };
    let size = |file: &str| {
        let bytes = row(file, "flowapi").map_or(0.0, |r| number(&r, &["bytes"]));
        format!("{:.1} MB of JSON", bytes / 1e6)
    };
    let mut panels: Vec<Panel> = [
        ("slack", "Slack"),
        ("stripe", "Stripe"),
        ("github", "GitHub"),
        ("cloudflare", "Cloudflare"),
    ]
    .iter()
    .map(|(file, name)| Panel {
        title: (*name).into(),
        under: size(file),
        bars: time(file),
    })
    .collect();
    let memory: Vec<Bar> = TOOLS
        .iter()
        .map(|t| {
            row("cloudflare", t).filter(|r| r["ok"] == true).map(|r| {
                let mb = number(&r, &["peakRssKb"]) / 1024.0;
                (mb, format!("{mb:.0} MB"))
            })
        })
        .collect();
    panels.push(Panel {
        title: "Peak memory".into(),
        under: "on Cloudflare's description".into(),
        bars: memory,
    });
    let checks: Vec<Bar> = TOOLS
        .iter()
        .map(|t| {
            let all = rows.iter().filter(|r| r["tool"] == *t).count() as f64;
            let ok = rows
                .iter()
                .filter(|r| r["tool"] == *t && r["typeChecks"] == true)
                .count() as f64;
            Some((ok, format!("{ok:.0} of {all:.0}")))
        })
        .collect();
    panels.push(Panel {
        title: "Output that type-checks".into(),
        under: "descriptions · more is better".into(),
        bars: checks,
    });
    Section {
        title: "Speed",
        under: "a fresh process per run, the whole process timed · shorter is better".into(),
        panels,
    }
}

const WIDE: f64 = 880.0;
const EDGE: f64 = 28.0;
const COLUMN: f64 = (WIDE - 2.0 * EDGE) / 3.0;
const LABELS: f64 = 116.0;
const BAR: f64 = COLUMN - LABELS - 8.0 - 70.0;
const ROW: f64 = 22.0;
const PANEL: f64 = 46.0 + 3.0 * ROW + 26.0;

fn text(svg: &mut String, x: f64, y: f64, fill: &str, look: &str, words: &str) {
    let words = words.replace('&', "&amp;").replace('<', "&lt;");
    let _ = writeln!(
        svg,
        "<text x=\"{x:.1}\" y=\"{y:.1}\" fill=\"{fill}\" {look}>{words}</text>"
    );
}

fn draw(svg: &mut String, left: f64, top: f64, panel: &Panel) {
    text(
        svg,
        left,
        top + 14.0,
        INK,
        "font-size=\"14\" font-weight=\"600\"",
        &panel.title,
    );
    text(
        svg,
        left,
        top + 31.0,
        QUIET,
        "font-size=\"11\"",
        &panel.under,
    );
    let most = panel
        .bars
        .iter()
        .flatten()
        .map(|(v, _)| *v)
        .fold(f64::MIN_POSITIVE, f64::max);
    for (index, (name, bar)) in TOOLS.iter().zip(&panel.bars).enumerate() {
        let y = top + 46.0 + index as f64 * ROW;
        let ours = index == 0;
        let (fill, ink, weight) = if ours {
            (OURS, OURS, " font-weight=\"600\"")
        } else {
            (THEIRS, INK, "")
        };
        text(
            svg,
            left + LABELS,
            y + 11.5,
            ink,
            &format!("font-size=\"12\" text-anchor=\"end\"{weight}"),
            name,
        );
        let Some((value, shown)) = bar else {
            text(
                svg,
                left + LABELS + 8.0,
                y + 11.5,
                QUIET,
                "font-size=\"12\"",
                "failed",
            );
            continue;
        };
        let wide = (value / most * BAR).max(3.0);
        let _ = writeln!(
            svg,
            "<rect x=\"{:.1}\" y=\"{y:.1}\" width=\"{wide:.1}\" height=\"14\" rx=\"3\" fill=\"{fill}\"/>",
            left + LABELS + 8.0
        );
        text(
            svg,
            left + LABELS + 8.0 + wide + 7.0,
            y + 11.5,
            ink,
            &format!("font-size=\"12\"{weight}"),
            shown,
        );
    }
}

const PAGE: &str = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif";
const DRAWN: &str = "'System Font', 'Helvetica Neue', Arial, 'DejaVu Sans', sans-serif";

fn chart(sections: &[Section], notes: &[String], fonts: &str) -> String {
    let rows = |s: &Section| s.panels.len().div_ceil(3) as f64;
    let high = EDGE
        + sections.iter().map(|s| 62.0 + rows(s) * PANEL).sum::<f64>()
        + 14.0
        + notes.len() as f64 * 17.0
        + 14.0;
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {WIDE:.0} {high:.0}\" width=\"{WIDE:.0}\" height=\"{high:.0}\" font-family=\"{fonts}\">\n\
         <rect x=\"0.5\" y=\"0.5\" width=\"{:.0}\" height=\"{:.0}\" rx=\"12\" fill=\"{PAPER}\" stroke=\"{EDGE_INK}\"/>\n",
        WIDE - 1.0,
        high - 1.0
    );
    let rule = |svg: &mut String, y: f64| {
        let _ = writeln!(
            svg,
            "<rect x=\"{EDGE:.0}\" y=\"{y:.1}\" width=\"{:.0}\" height=\"1\" fill=\"{EDGE_INK}\"/>",
            WIDE - 2.0 * EDGE
        );
    };
    let mut top = EDGE;
    for section in sections {
        text(
            &mut svg,
            EDGE,
            top + 16.0,
            INK,
            "font-size=\"17\" font-weight=\"700\"",
            section.title,
        );
        text(
            &mut svg,
            EDGE,
            top + 34.0,
            QUIET,
            "font-size=\"12\"",
            &section.under,
        );
        rule(&mut svg, top + 44.0);
        top += 62.0;
        for (index, panel) in section.panels.iter().enumerate() {
            let (column, row) = ((index % 3) as f64, (index / 3) as f64);
            draw(&mut svg, EDGE + column * COLUMN, top + row * PANEL, panel);
        }
        top += rows(section) * PANEL;
    }
    rule(&mut svg, top - 8.0);
    for (index, note) in notes.iter().enumerate() {
        text(
            &mut svg,
            EDGE,
            top + 12.0 + index as f64 * 17.0,
            QUIET,
            "font-size=\"11.5\"",
            note,
        );
    }
    svg.push_str("</svg>\n");
    svg
}

fn png(svg: &str, options: &Options) -> Vec<u8> {
    const SCALE: f32 = 2.0;
    let tree = Tree::from_str(svg, options).expect("the chart is SVG");
    let size = tree.size();
    let (wide, high) = (
        (size.width() * SCALE).ceil() as u32,
        (size.height() * SCALE).ceil() as u32,
    );
    let mut pixels = Pixmap::new(wide, high).expect("a chart has a size");
    resvg::render(
        &tree,
        Transform::from_scale(SCALE, SCALE),
        &mut pixels.as_mut(),
    );
    pixels.encode_png().expect("pixels encode")
}

fn main() {
    let compare = read("../conformance/results/compare.json");
    let measured = read("results/speed.json");
    let sections = [correctness(&compare), speed(&measured)];
    let m = &measured["machine"];
    let versions = |t: &str| {
        compare["tools"][t]["version"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    };
    let rounds = number(&measured, &["rounds"]);
    let notes = [
        "Each panel has its own scale; the figure beside a bar is the measurement. Every figure comes from bench/ and conformance/.".to_owned(),
        format!(
            "flowapi {}, openapi-typescript {}, @hey-api/openapi-ts {} (types only), each with its defaults.",
            versions("flowapi"),
            versions("openapi-typescript"),
            versions("hey-api")
        ),
        format!(
            "{}; {}, {} cores, {}.",
            if rounds > 1.0 { format!("Time: the median of {rounds:.0} rounds") } else { String::from("Time: one round, a first look") },
            m["cpu"].as_str().unwrap_or(""),
            m["cores"],
            m["os"].as_str().unwrap_or("")
        ),
    ];
    fs::create_dir_all("../assets").expect("assets/");
    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();
    fs::write(
        "../assets/compare-dark.png",
        png(&chart(&sections, &notes, DRAWN), &options),
    )
    .expect("PNG written");
    fs::write("../assets/compare-dark.svg", chart(&sections, &notes, PAGE)).expect("SVG written");
    println!("wrote ../assets/compare-dark.svg and compare-dark.png");
}
