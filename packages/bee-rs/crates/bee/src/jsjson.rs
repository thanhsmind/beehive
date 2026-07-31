// jsjson — JSON serialization matching Node's JSON.stringify byte-for-byte
// for the values bee round-trips (contract C2).
//
// serde_json's default output differs from JS in exactly one load-bearing
// place: float-typed whole numbers. `JSON.parse("1.0")` collapses to the JS
// number 1 and re-stringifies as "1"; serde keeps f64 1.0 and prints "1.0".
// The custom Formatter below applies the JS rule. (Rust's shortest-round-trip
// float printing otherwise agrees with V8's for realistic values; neither
// side's exotic-exponent forms appear in bee state files.)
//
// Object key order: bee relies on JS insertion order everywhere, so the
// serde_json "preserve_order" feature (IndexMap-backed) is mandatory — the
// workspace enables it.

use serde_json::ser::{CompactFormatter, Formatter, PrettyFormatter, Serializer};
use serde_json::Value;
use std::io;

/// JS Number-to-string for a finite f64 (JSON never carries NaN/Inf).
/// Whole values inside i64 range print without a decimal point, like JS.
pub fn js_f64_to_string(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 9.007_199_254_740_992e15 {
        // 2^53: beyond it JS itself prints exponent-free integers too, but
        // f64 can no longer represent every integer — the cast stays exact
        // below the boundary, which is all bee's state files ever hold.
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

struct JsFloats<F>(F);

impl<F: Formatter> Formatter for JsFloats<F> {
    fn write_f64<W: ?Sized + io::Write>(&mut self, writer: &mut W, value: f64) -> io::Result<()> {
        writer.write_all(js_f64_to_string(value).as_bytes())
    }
    // Everything else delegates to the wrapped formatter.
    fn begin_array<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.begin_array(w)
    }
    fn end_array<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.end_array(w)
    }
    fn begin_array_value<W: ?Sized + io::Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        self.0.begin_array_value(w, first)
    }
    fn end_array_value<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.end_array_value(w)
    }
    fn begin_object<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.begin_object(w)
    }
    fn end_object<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.end_object(w)
    }
    fn begin_object_key<W: ?Sized + io::Write>(&mut self, w: &mut W, first: bool) -> io::Result<()> {
        self.0.begin_object_key(w, first)
    }
    fn end_object_key<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.end_object_key(w)
    }
    fn begin_object_value<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.begin_object_value(w)
    }
    fn end_object_value<W: ?Sized + io::Write>(&mut self, w: &mut W) -> io::Result<()> {
        self.0.end_object_value(w)
    }
}

/// JSON.stringify(value) — compact.
pub fn stringify(value: &Value) -> String {
    let mut out = Vec::new();
    let mut ser = Serializer::with_formatter(&mut out, JsFloats(CompactFormatter));
    serde::Serialize::serialize(value, &mut ser).expect("in-memory JSON serialization");
    String::from_utf8(out).expect("serde_json emits UTF-8")
}

/// JSON.stringify(value, null, 2) — 2-space pretty.
pub fn stringify_pretty(value: &Value) -> String {
    let mut out = Vec::new();
    let mut ser = Serializer::with_formatter(&mut out, JsFloats(PrettyFormatter::with_indent(b"  ")));
    serde::Serialize::serialize(value, &mut ser).expect("in-memory JSON serialization");
    String::from_utf8(out).expect("serde_json emits UTF-8")
}

/// JS String(value) coercion for JSON values — used by warning interpolations
/// (`config: unrecognized ship_visibility "${raw}" ...`).
pub fn js_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                js_f64_to_string(f)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        // JS Array.prototype.toString: elements joined by ",", null/undefined
        // render empty.
        Value::Array(items) => items
            .iter()
            .map(|v| match v {
                Value::Null => String::new(),
                other => js_to_string(other),
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn float_whole_numbers_print_like_js() {
        let v: Value = serde_json::from_str(r#"{"a":1.0,"b":1.5,"c":2,"d":-3.0}"#).unwrap();
        assert_eq!(stringify(&v), r#"{"a":1,"b":1.5,"c":2,"d":-3}"#);
    }

    #[test]
    fn pretty_matches_js_layout() {
        let v = json!({"hash": "abc", "n": [1, 2]});
        assert_eq!(
            stringify_pretty(&v),
            "{\n  \"hash\": \"abc\",\n  \"n\": [\n    1,\n    2\n  ]\n}"
        );
    }

    #[test]
    fn preserves_insertion_order() {
        let v: Value = serde_json::from_str(r#"{"z":1,"a":2,"m":3}"#).unwrap();
        assert_eq!(stringify(&v), r#"{"z":1,"a":2,"m":3}"#);
    }

    #[test]
    fn js_string_coercion() {
        assert_eq!(js_to_string(&json!("x")), "x");
        assert_eq!(js_to_string(&json!(true)), "true");
        assert_eq!(js_to_string(&json!({"a": 1})), "[object Object]");
        assert_eq!(js_to_string(&json!([1, null, "b"])), "1,,b");
    }
}
