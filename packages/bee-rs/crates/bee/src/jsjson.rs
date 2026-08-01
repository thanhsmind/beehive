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
///
/// Implements ECMA-262 `Number::toString` for the whole finite range, not just
/// the comfortable middle. Rust's `{}` and JS agree on shortest-round-trip
/// DIGITS but disagree on when to switch to exponential form: JS switches at
/// |n| >= 1e21 and at |n| < 1e-6, Rust never does. Until cutover the port
/// dodged the difference by refusing such numbers (`js_numberify` returned
/// Exotic and the verb delegated to Node); with Node gone the formatting is
/// implemented instead, so a store file holding 1e21 renders as `1e+21` the
/// way it always did and no read has to bail.
pub fn js_f64_to_string(v: f64) -> String {
    if v == 0.0 {
        return "0".to_string(); // covers -0.0, which JS prints as "0"
    }
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if v < 0.0 {
        return format!("-{}", js_f64_to_string(-v));
    }
    if v.fract() == 0.0 && v < 9.007_199_254_740_992e15 {
        // 2^53: the cast stays exact below the boundary, and JS prints these
        // exponent-free — the common case for every count and timestamp bee
        // stores.
        return format!("{}", v as i64);
    }
    // Shortest round-trip digits and decimal exponent, via Rust's `{:e}`
    // (`d1[.d2…dk]e<E>`), where the spec's `n` is E + 1.
    let exp = format!("{v:e}");
    let (mantissa, e) = exp.split_once('e').expect("LowerExp always emits 'e'");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i32;
    let n = e.parse::<i32>().expect("LowerExp exponent is an integer") + 1;

    if k <= n && n <= 21 {
        // Integer with trailing zeros.
        return format!("{}{}", digits, "0".repeat((n - k) as usize));
    }
    if 0 < n && n <= 21 {
        // Decimal point inside the digits.
        return format!("{}.{}", &digits[..n as usize], &digits[n as usize..]);
    }
    if -6 < n && n <= 0 {
        return format!("0.{}{}", "0".repeat((-n) as usize), digits);
    }
    // Exponential form. JS always signs the exponent.
    let sign = if n - 1 >= 0 { "+" } else { "-" };
    let mag = (n - 1).abs();
    if k == 1 {
        format!("{digits}e{sign}{mag}")
    } else {
        format!("{}.{}e{}{}", &digits[..1], &digits[1..], sign, mag)
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

    /// The formatting that used to force a delegation. Every expectation here
    /// is what `node -p "String(x)"` prints.
    #[test]
    fn extreme_magnitudes_print_like_js() {
        assert_eq!(js_f64_to_string(1e21), "1e+21");
        assert_eq!(js_f64_to_string(-1e21), "-1e+21");
        assert_eq!(js_f64_to_string(1.5e22), "1.5e+22");
        assert_eq!(js_f64_to_string(1.2345e21), "1.2345e+21");
        assert_eq!(js_f64_to_string(1e-7), "1e-7");
        assert_eq!(js_f64_to_string(1.5e-7), "1.5e-7");
        assert_eq!(js_f64_to_string(1e-6), "0.000001");
        assert_eq!(js_f64_to_string(0.000123), "0.000123");
        assert_eq!(js_f64_to_string(1e20), "100000000000000000000");
        assert_eq!(js_f64_to_string(0.0), "0");
        assert_eq!(js_f64_to_string(-0.0), "0");
        assert_eq!(js_f64_to_string(1.5), "1.5");
        assert_eq!(js_f64_to_string(-3.0), "-3");
        assert_eq!(js_f64_to_string(1e300), "1e+300");
        assert_eq!(js_f64_to_string(5e-324), "5e-324");
    }

    /// And the same through the serializer, where the value would land in a
    /// store file.
    #[test]
    fn stringify_carries_extreme_magnitudes() {
        let v: Value = serde_json::from_str(r#"{"big":1e21,"tiny":1e-7}"#).unwrap();
        assert_eq!(stringify(&v), r#"{"big":1e+21,"tiny":1e-7}"#);
    }

    #[test]
    fn js_string_coercion() {
        assert_eq!(js_to_string(&json!("x")), "x");
        assert_eq!(js_to_string(&json!(true)), "true");
        assert_eq!(js_to_string(&json!({"a": 1})), "[object Object]");
        assert_eq!(js_to_string(&json!([1, null, "b"])), "1,,b");
    }
}
