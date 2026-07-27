//! JS-compatible JSON emission (rpl-1, CONTEXT.md D3 byte-compatibility).
//!
//! `serde_json` with `preserve_order` reproduces JS object key order for the
//! ordinary case — insertion order — but NOT for the one case where JS
//! itself reorders: **integer-index keys**. `OrdinaryOwnPropertyKeys` (ECMA-262
//! 10.1.11.1) enumerates array-index keys FIRST, in ascending numeric order,
//! and only then the remaining string keys in property-creation order.
//! `JSON.stringify` walks the object through that same ordering, so
//!
//! ```text
//! JSON.stringify({"10":1,"2":2,"zeta":3,"1":4})
//!   -> {"1":4,"2":2,"10":1,"zeta":3}
//! ```
//!
//! while `serde_json` + `preserve_order` emits the insertion order verbatim:
//! `{"10":1,"2":2,"zeta":3,"1":4}`.
//!
//! MEASURED, not assumed (rpl-1 obligation (c)): a lane record seeded with
//! numeric-string keys under `approved_gates` and read back through `status
//! --json` produced exactly that divergence between the frozen `bee.mjs`
//! oracle and `queen-bee` before this module existed. The divergence is
//! reachable from a real ledger store — `bee reviews record` persists
//! caller-supplied JSON into `.bee/reviews/<id>.json` with no key
//! sanitization (`packages/bee/lib/reviews.mjs:283-327`, fed by
//! `packages/bee/bee.mjs:4315-4322`/`:4398-4404`) — so this is a port
//! correctness requirement, not a theoretical one.
//!
//! Every JSON payload `queen-bee` writes to stdout goes through
//! [`stringify_pretty`] so the reordering can never be forgotten at one call
//! site while being applied at another.

use serde_json::{Map, Value};

/// The largest valid JS array index: 2^32 - 2. `String(2**32-1)` is a
/// perfectly good property key but is NOT an array index, so it enumerates
/// with the string keys.
const MAX_ARRAY_INDEX: u64 = (u32::MAX as u64) - 1;

/// Is `key` a JS *array index* — i.e. is `String(ToUint32(key)) === key` with
/// the value in `[0, 2^32-2]`? This is the exact predicate that decides
/// whether a property enumerates in the numeric-first bucket.
///
/// Deliberately strict, matching the canonical-numeric-string rule:
/// `"01"`, `"+1"`, `"1.0"`, `"-1"`, `" 1"` and `"1e2"` are all ordinary
/// string keys, because none of them round-trips through `ToUint32`.
pub fn is_array_index(key: &str) -> bool {
    if key.is_empty() || key.len() > 10 {
        return false;
    }
    if !key.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    // "0" is canonical; "01", "007" are not.
    if key.len() > 1 && key.starts_with('0') {
        return false;
    }
    key.parse::<u64>().map(|n| n <= MAX_ARRAY_INDEX).unwrap_or(false)
}

/// Rewrite `value` recursively into JS own-property order: array-index keys
/// ascending first, then every other key in its original (insertion) order.
/// Arrays keep their order; scalars are untouched.
pub fn to_js_key_order(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut indexed: Vec<(u64, &String, &Value)> = Vec::new();
            let mut named: Vec<(&String, &Value)> = Vec::new();
            for (k, v) in map {
                match k.parse::<u64>() {
                    Ok(n) if is_array_index(k) => indexed.push((n, k, v)),
                    _ => named.push((k, v)),
                }
            }
            // `sort_by_key` is stable, and array-index keys are unique within
            // one object anyway, so this is a total order.
            indexed.sort_by_key(|(n, _, _)| *n);
            let mut out = Map::new();
            for (_, k, v) in indexed {
                out.insert(k.clone(), to_js_key_order(v));
            }
            for (k, v) in named {
                out.insert(k.clone(), to_js_key_order(v));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(to_js_key_order).collect()),
        other => other.clone(),
    }
}

/// `JSON.stringify(value, null, 2)` — two-space indent, JS key order. Does
/// NOT append the trailing newline: callers that mirror a `process.stdout
/// .write(`${...}\n`)` add it themselves, so a call site that must not emit
/// one cannot be forced to.
pub fn stringify_pretty(value: &Value) -> String {
    serde_json::to_string_pretty(&to_js_key_order(value)).unwrap_or_else(|_| "null".to_string())
}

/// `JSON.stringify(value)` — compact, JS key order.
pub fn stringify_compact(value: &Value) -> String {
    serde_json::to_string(&to_js_key_order(value)).unwrap_or_else(|_| "null".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn array_index_predicate_matches_the_canonical_numeric_string_rule() {
        for yes in ["0", "1", "2", "10", "4294967294"] {
            assert!(is_array_index(yes), "{yes} should be an array index");
        }
        for no in ["", "01", "007", "-1", "+1", "1.0", " 1", "1e2", "4294967295", "12345678901", "a"] {
            assert!(!is_array_index(no), "{no} should NOT be an array index");
        }
    }

    #[test]
    fn integer_like_keys_sort_ascending_before_string_keys() {
        // The exact shape measured against the frozen mjs oracle.
        let v: Value = serde_json::from_str(r#"{"10":true,"2":false,"zeta":true,"1":true}"#).unwrap();
        assert_eq!(stringify_compact(&v), r#"{"1":true,"2":false,"10":true,"zeta":true}"#);
    }

    #[test]
    fn string_keys_keep_insertion_order_among_themselves() {
        let v: Value = serde_json::from_str(r#"{"zeta":1,"alpha":2,"3":3,"mid":4,"0":5}"#).unwrap();
        assert_eq!(
            stringify_compact(&v),
            r#"{"0":5,"3":3,"zeta":1,"alpha":2,"mid":4}"#
        );
    }

    #[test]
    fn reordering_is_recursive_through_objects_and_arrays() {
        let v: Value = serde_json::from_str(
            r#"{"outer":{"10":1,"2":2},"list":[{"5":"a","1":"b"}]}"#,
        )
        .unwrap();
        assert_eq!(
            stringify_compact(&v),
            r#"{"outer":{"2":2,"10":1},"list":[{"1":"b","5":"a"}]}"#
        );
    }

    #[test]
    fn non_canonical_numeric_keys_stay_in_the_string_bucket() {
        // "01" is NOT an array index in JS, so it must NOT be hoisted.
        let v: Value = serde_json::from_str(r#"{"b":1,"01":2,"2":3}"#).unwrap();
        assert_eq!(stringify_compact(&v), r#"{"2":3,"b":1,"01":2}"#);
    }

    #[test]
    fn ordinary_objects_are_untouched() {
        let v = json!({"schema_version": "1.0", "phase": "idle", "workers": []});
        assert_eq!(
            stringify_compact(&v),
            r#"{"schema_version":"1.0","phase":"idle","workers":[]}"#
        );
    }

    #[test]
    fn pretty_matches_json_stringify_two_space_indent() {
        let v = json!({"a": 1, "b": [1, 2]});
        assert_eq!(stringify_pretty(&v), "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2\n  ]\n}");
    }

    #[test]
    fn empty_containers_render_like_json_stringify() {
        let v = json!({"o": {}, "a": []});
        assert_eq!(stringify_pretty(&v), "{\n  \"o\": {},\n  \"a\": []\n}");
    }
}
