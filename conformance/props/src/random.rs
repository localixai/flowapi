//! The random schemas and values, from a xorshift generator with a fixed seed. Values are drawn
//! near what the schema asks (its property names, its literals, its kinds), so many are valid.

use serde_json::{Map, Value, json};

pub(crate) struct Rng(pub(crate) u64);

impl Rng {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    pub(crate) fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const NAMES: [&str; 4] = ["a", "b", "x-c", "constructor"];
const TYPES: [&str; 7] = [
    "null", "boolean", "integer", "number", "string", "array", "object",
];

fn leaf(r: &mut Rng) -> Value {
    match r.below(6) {
        0 => json!({ "type": TYPES[r.below(7)] }),
        1 => json!({ "type": [TYPES[r.below(7)], TYPES[r.below(7)]] }),
        2 => {
            let all = [
                json!(1),
                json!("a"),
                Value::Null,
                json!([1]),
                json!({ "a": 1 }),
            ];
            json!({ "enum": all[..r.below(5) + 1].to_vec() })
        }
        3 => {
            let one = [json!("a"), json!(2), json!(true), Value::Null][r.below(4)].clone();
            json!({ "const": one })
        }
        4 => json!(r.below(2) == 0),
        _ => json!({}),
    }
}

pub(crate) fn schema(r: &mut Rng, depth: u32) -> Value {
    if depth == 0 {
        return leaf(r);
    }
    let mut s = Map::new();
    for _ in 0..r.below(4) + 1 {
        match r.below(18) {
            0 => drop(s.insert("type".into(), json!(TYPES[r.below(7)]))),
            1 => {
                let mut props = Map::new();
                for name in NAMES.iter().take(r.below(4) + 1) {
                    props.insert((*name).into(), schema(r, depth - 1));
                }
                s.insert("properties".into(), Value::Object(props));
            }
            2 => drop(s.insert("required".into(), json!(NAMES[..r.below(3)]))),
            3 => drop(s.insert("additionalProperties".into(), schema(r, depth - 1))),
            4 => drop(s.insert("items".into(), schema(r, depth - 1))),
            5 => drop(s.insert(
                "prefixItems".into(),
                json!([schema(r, depth - 1), schema(r, depth - 1)]),
            )),
            6 => drop(s.insert(
                "anyOf".into(),
                json!([schema(r, depth - 1), schema(r, depth - 1)]),
            )),
            7 => drop(s.insert(
                "allOf".into(),
                json!([schema(r, depth - 1), schema(r, depth - 1)]),
            )),
            8 => drop(s.insert(
                "oneOf".into(),
                json!([schema(r, depth - 1), schema(r, depth - 1)]),
            )),
            9 => drop(s.insert("minItems".into(), json!(r.below(3)))),
            10 => drop(s.insert(
                "patternProperties".into(),
                json!({ "^x-": schema(r, depth - 1) }),
            )),
            11 => drop(s.insert("unevaluatedProperties".into(), json!(false))),
            12 => {
                let names = [
                    json!(false),
                    json!({ "enum": ["a", "x-c"] }),
                    json!({ "pattern": "^x-" }),
                    json!({ "maxLength": 1 }),
                    json!({ "const": "constructor" }),
                ];
                s.insert("propertyNames".into(), names[r.below(5)].clone());
            }
            13 => {
                s.insert("contains".into(), schema(r, depth - 1));
                for key in ["minContains", "maxContains"] {
                    if r.below(2) == 0 {
                        s.insert(key.into(), json!(r.below(3)));
                    }
                }
            }
            14 => {
                let tests = [
                    json!({ "type": TYPES[r.below(7)] }),
                    json!({ "required": ["a"] }),
                    json!({ "type": "object", "required": ["a", "b"] }),
                    json!(r.below(2) == 0),
                    json!({ "const": 1 }),
                ];
                s.insert("if".into(), tests[r.below(5)].clone());
                for key in ["then", "else"] {
                    if r.below(3) > 0 {
                        s.insert(key.into(), schema(r, depth - 1));
                    }
                }
            }
            15 => {
                let keys = [
                    "minimum",
                    "maximum",
                    "exclusiveMinimum",
                    "exclusiveMaximum",
                    "multipleOf",
                ];
                let key = keys[r.below(5)];
                let values = if key == "multipleOf" {
                    [json!(1), json!(2), json!(0.25)]
                } else {
                    [json!(0), json!(1), json!(-3)]
                };
                s.insert(key.into(), values[r.below(3)].clone());
                scalars(r, &mut s);
            }
            16 => {
                let key = ["minLength", "maxLength", "pattern"][r.below(3)];
                let value = if key == "pattern" {
                    json!(["^a", "c$", "x-"][r.below(3)])
                } else {
                    json!(r.below(3))
                };
                s.insert(key.into(), value);
                scalars(r, &mut s);
            }
            _ => drop(s.insert("maxProperties".into(), json!(r.below(3)))),
        }
    }
    Value::Object(s)
}

/// Sometimes an `enum` of scalars beside a schema's bounds, which the bounds filter.
fn scalars(r: &mut Rng, s: &mut Map<String, Value>) {
    if r.below(2) == 0 {
        let all = json!([0, 1, 2, -3, 0.25, 1.5, "a", "", "x-c", "abc"]);
        s.insert("enum".into(), all);
    }
}

/// A value drawn near the schema: its kinds, names and literals.
pub(crate) fn value(r: &mut Rng, depth: u32, schema: &Value) -> Value {
    if let Some(lits) = schema
        .get("enum")
        .and_then(Value::as_array)
        .filter(|l| !l.is_empty())
        && r.below(2) == 0
    {
        return lits[r.below(lits.len())].clone();
    }
    if let Some(c) = schema.get("const")
        && r.below(2) == 0
    {
        return c.clone();
    }
    let sub = |key: &str| schema.get(key).cloned().unwrap_or(json!({}));
    match r.below(if depth == 0 { 5 } else { 7 }) {
        0 => Value::Null,
        1 => json!(r.below(2) == 0),
        2 => json!([1, -3, 0][r.below(3)]),
        3 => json!([1.5, 0.25][r.below(2)]),
        4 => json!(["a", "", "x-c"][r.below(3)]),
        5 => {
            let items = sub("items");
            Value::Array(
                (0..r.below(3))
                    .map(|_| value(r, depth - 1, &items))
                    .collect(),
            )
        }
        _ => {
            let props = schema
                .get("properties")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            let mut o = Map::new();
            for name in NAMES {
                if r.below(2) == 0 {
                    let s = props.get(name).cloned().unwrap_or(json!({}));
                    o.insert(name.into(), value(r, depth - 1, &s));
                }
            }
            Value::Object(o)
        }
    }
}
