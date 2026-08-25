// bee dev statusline — the per-model token/cost segment of the status line.
//
// PROVENANCE. A Rust port of `statusline-usage.mjs`, the Node half of the old
// status-display pair, deleted at the R6 cutover. The `provenance:` notes
// through this file still name its functions because they are what this
// behaviour is bound to; nothing in the tree ships that file any more.
//
// Reads the statusline JSON on stdin, aggregates usage from the session's main
// transcript AND every subagent transcript, prints two lines. Fail-open on
// EVERY error: print nothing, exit 0, never break the line.
//
// THE .sh CONTRACT (packages/bee/statusline/statusline-command.sh). The script
// renders line one itself, then resolves a bee binary — `.bee/bin/bee` (or
// `bee.exe`) under `$CLAUDE_PROJECT_DIR` first, else the same path beside the
// main checkout that `git rev-parse --git-common-dir` names from a linked
// worktree, else `bee` on PATH — and appends this command's output as a
// second line only when it is non-empty:
//
//     usage_seg=$(echo "$input" | "$BEE" dev statusline 2>/dev/null)
//     [ -n "$usage_seg" ] && line="${line}\n${yellow}${usage_seg}${reset}"
//
// So the contract this command must keep is exactly: JSON on stdin, the segment
// (or NOTHING) on stdout, stderr ignored, exit code ignored. The leg is
// optional by design — a host that resolves no binary renders line one and no
// usage segment, because a status line must never be the reason a prompt fails
// to render. `packages/bee-rs/crates/bee/tests/statusline_contract.rs` holds
// that lookup and this contract; the .sh is not edited here.
//
// SHARED CACHE. The signature cache file in os.tmpdir() is READ AND WRITTEN
// BY BOTH runtimes during the two-runtime window, so `js_tmpdir`, the cache
// filename, the `sig` string and the JSON body are reproduced exactly — a
// line cached by Node is served by Rust and vice versa.

use crate::jsjson;
use serde_json::Value;
use std::cmp::Ordering;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// provenance: statusline-usage.mjs CACHE_VERSION.
const CACHE_VERSION: u32 = 6;

// ─── JS number/string formatting primitives ────────────────────────────────

/// provenance: `Number.prototype.toFixed` (ECMA-262 §21.1.3.3) — pick the
/// integer n minimising |n/10^f − x|, ties to the LARGER n. Rust's
/// `format!("{:.*}")` rounds half-to-EVEN instead, which diverges on exactly
/// the values this tool produces (`(1_250_000/1e6).toFixed(1)` is "1.3" in
/// JS and "1.2" under Rust's formatter — 1.25 is exactly representable, so
/// the tie is real and reachable for any token count that is a multiple of
/// 50k). Rendering the double's EXACT decimal expansion first and rounding
/// half-up on the digit string reproduces the spec.
fn js_to_fixed(x: f64, digits: usize) -> String {
    if !x.is_finite() {
        return format!("{x}");
    }
    let negative = x.is_sign_negative() && x != 0.0;
    let ax = x.abs();
    // 100 fraction digits is exact here: a double ≥ 1e-3 has at most ~62
    // significant binary fraction bits, so its decimal expansion terminates
    // well inside that width and the tail is genuine zeros.
    let exact = format!("{ax:.100}");
    let (int_part, frac) = exact.split_once('.').unwrap_or((exact.as_str(), ""));
    let mut int_digits: Vec<u8> = int_part.bytes().collect();
    let mut keep: Vec<u8> = frac.bytes().take(digits).collect();
    while keep.len() < digits {
        keep.push(b'0');
    }
    let round_up = frac.as_bytes().get(digits).is_some_and(|d| *d >= b'5');
    if round_up {
        let mut carry = true;
        for d in keep.iter_mut().rev() {
            if !carry {
                break;
            }
            if *d == b'9' {
                *d = b'0';
            } else {
                *d += 1;
                carry = false;
            }
        }
        if carry {
            for d in int_digits.iter_mut().rev() {
                if *d == b'9' {
                    *d = b'0';
                } else {
                    *d += 1;
                    carry = false;
                    break;
                }
            }
            if carry {
                int_digits.insert(0, b'1');
            }
        }
    }
    let int_s = String::from_utf8(int_digits).unwrap();
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(&int_s);
    if digits > 0 {
        out.push('.');
        out.push_str(std::str::from_utf8(&keep).unwrap());
    }
    out
}

/// provenance: `Math.round` — floor(x + 0.5) for the non-negative token
/// counts this tool feeds it (ties toward +∞, unlike Rust's `f64::round`
/// which is ties-away-from-zero; identical for x ≥ 0, spelled out anyway).
fn js_math_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// provenance: statusline-usage.mjs fmtTok.
fn fmt_tok(n: f64) -> String {
    if n >= 1e6 {
        format!("{}M", js_to_fixed(n / 1e6, 1))
    } else if n >= 1e3 {
        format!("{}k", jsjson::js_f64_to_string(js_math_round(n / 1e3)))
    } else {
        jsjson::js_f64_to_string(n)
    }
}

/// provenance: statusline-usage.mjs fmtUsd.
fn fmt_usd(n: f64) -> String {
    if n >= 10.0 {
        format!("${}", js_to_fixed(n, 0))
    } else if n >= 0.1 {
        format!("${}", js_to_fixed(n, 2))
    } else {
        format!("${}", js_to_fixed(n, 3))
    }
}

/// provenance: statusline-usage.mjs PRICES / priceFor — first regex hit wins,
/// default [5, 25]. Every pattern is a bare unanchored alternation/substring.
fn price_for(model: &str) -> (f64, f64) {
    if model.contains("fable") || model.contains("mythos") {
        (10.0, 50.0)
    } else if model.contains("opus") {
        (5.0, 25.0)
    } else if model.contains("sonnet-5") {
        (2.0, 10.0) // intro pricing through 2026-08-31 (standard 3/15)
    } else if model.contains("sonnet") {
        (3.0, 15.0)
    } else if model.contains("haiku") {
        (1.0, 5.0)
    } else {
        (5.0, 25.0)
    }
}

/// provenance: statusline-usage.mjs shortName —
/// `.replace(/^claude-/, "").replace(/-\d{8}$/, "")`.
fn short_name(model: &str) -> String {
    let stripped = model.strip_prefix("claude-").unwrap_or(model);
    let b = stripped.as_bytes();
    if b.len() >= 9 && b[b.len() - 9] == b'-' && b[b.len() - 8..].iter().all(u8::is_ascii_digit) {
        return stripped[..stripped.len() - 9].to_string();
    }
    stripped.to_string()
}

// ─── Node os primitives ────────────────────────────────────────────────────
//
// `path.dirname` / `path.basename` / `path.join` come from devtools::jspath
// (Node's lexical rules, win32 flavor on Windows).

use super::jspath::{basename as path_basename, dirname as path_dirname, join as path_join};

/// provenance: `os.tmpdir()` (node lib/os.js) — reproduced rather than using
/// `std::env::temp_dir` because the cache file is SHARED with the Node
/// implementation and a different directory would silently halve the cache
/// hit rate. Empty env vars are falsy in JS, hence the `is_empty` filters.
fn js_tmpdir() -> String {
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    if cfg!(windows) {
        let mut p = env("TEMP")
            .or_else(|| env("TMP"))
            .unwrap_or_else(|| {
                let root = env("SystemRoot").or_else(|| env("windir")).unwrap_or_default();
                format!("{root}\\temp")
            });
        if p.len() > 1 && p.ends_with('\\') && !p.ends_with(":\\") {
            p.pop();
        }
        p
    } else {
        let mut p = env("TMPDIR")
            .or_else(|| env("TMP"))
            .or_else(|| env("TEMP"))
            .unwrap_or_else(|| "/tmp".to_string());
        if p.len() > 1 && p.ends_with('/') {
            p.pop();
        }
        p
    }
}

/// provenance: Node's `fs.Stats.mtimeMs` — libuv's `st_mtim` rendered as
/// `sec * 1e3 + nsec / 1e6` in f64, the exact expression node_file.cc uses.
fn mtime_ms(meta: &std::fs::Metadata) -> Option<f64> {
    let t = meta.modified().ok()?;
    let d = t.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some((d.as_secs() as f64) * 1e3 + (d.subsec_nanos() as f64) / 1e6)
}

// ─── the aggregation ───────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct Sums {
    r#in: f64,
    out: f64,
    c5: f64,
    c1: f64,
    read: f64,
}

struct Part {
    model: String,
    new_tokens: f64,
    cached_tokens: f64,
    cost: f64,
}

/// JS truthiness for a JSON value (`if (!m.usage) continue`).
fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `usage[key] ?? 0` for a JSON number.
fn num(v: Option<&Value>) -> f64 {
    match v {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        // JS `??` only defaults on null/undefined; a non-number would make
        // `+=` produce NaN. Non-numbers never appear in a transcript's usage
        // block, and NaN would poison the whole line, so a non-number is
        // treated as absent here (documented divergence, unreachable in
        // practice).
        _ => 0.0,
    }
}

/// provenance: statusline-usage.mjs main(input). `None` means "print
/// nothing" — every early `return` in the .mjs plus every `catch`.
///
/// `tmpdir` is `os.tmpdir()` at the call site; it is a parameter rather than
/// a call so the tests can isolate the SHARED signature cache without
/// mutating process env (which would race across the test harness's threads).
fn compute_line(input: &Value, tmpdir: &str) -> Option<String> {
    let transcript = input.get("transcript_path")?.as_str()?;
    if transcript.is_empty() || !Path::new(transcript).exists() {
        return None;
    }

    // files = [transcript, ...subagents/*.jsonl]
    let mut files: Vec<String> = vec![transcript.to_string()];
    let sub_dir = path_join(&[
        &path_dirname(transcript),
        &path_basename(transcript, ".jsonl"),
        "subagents",
    ]);
    if Path::new(&sub_dir).exists() {
        // readdirSync order = the platform's directory order; Node does not
        // sort. std::fs::read_dir is the same syscall stream.
        if let Ok(entries) = std::fs::read_dir(&sub_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.ends_with(".jsonl") {
                    files.push(path_join(&[&sub_dir, &name]));
                }
            }
        }
    }

    // Signature cache: `${CACHE_VERSION};` then `${f}:${size}:${round(mtimeMs)};`
    // per file. statSync THROWS on a vanished file, and the .mjs's outer try
    // turns that into exit 0 with no output — hence the `?` here.
    let mut sig = format!("{CACHE_VERSION};");
    for f in &files {
        let meta = std::fs::metadata(f).ok()?;
        let ms = mtime_ms(&meta)?;
        sig.push_str(&format!(
            "{}:{}:{};",
            f,
            meta.len(),
            jsjson::js_f64_to_string(js_math_round(ms))
        ));
    }
    let cache_file = PathBuf::from(tmpdir).join(format!(
        "claude-usage-{}.json",
        path_basename(transcript, ".jsonl")
    ));
    if let Ok(text) = std::fs::read_to_string(&cache_file) {
        if let Ok(cached) = serde_json::from_str::<Value>(&text) {
            if cached.get("sig").and_then(Value::as_str) == Some(sig.as_str()) {
                // A non-string `line` makes process.stdout.write throw, which
                // the .mjs's catch turns into a silent exit 0.
                return cached.get("line").and_then(Value::as_str).map(str::to_string);
            }
        }
    }

    // Streaming appends several lines per message id with cumulative usage —
    // keep the LAST occurrence so nothing is double-counted.
    let mut by_id: Vec<(String, String, Value)> = Vec::new(); // (key, model, usage)
    let mut seq: u64 = 0;
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else { continue };
        for line_raw in text.split('\n') {
            if !line_raw.contains("\"usage\"") {
                continue;
            }
            let Ok(obj) = serde_json::from_str::<Value>(line_raw) else { continue };
            let Some(m) = obj.get("message").filter(|v| js_truthy(v)) else { continue };
            let usage = m.get("usage").filter(|u| js_truthy(u));
            // A non-string `model` would be truthy in JS and then coerced by
            // the price regexes; transcripts never carry one, and treating it
            // as absent is the fail-open direction (documented divergence).
            let model = m.get("model").and_then(Value::as_str).filter(|s| !s.is_empty());
            let (Some(usage), Some(model)) = (usage, model) else { continue };
            if model == "<synthetic>" {
                continue;
            }
            // `m.id ?? \`${f}#${seq++}\`` — the fallback (and the increment)
            // fire only when id is null/undefined. Keys are tagged by JSON
            // form so a numeric id can never collide with a string one, the
            // way a JS Map's SameValueZero keying behaves.
            let key = match m.get("id").filter(|v| !v.is_null()) {
                Some(id) => format!("id:{}", jsjson::stringify(id)),
                None => {
                    let k = format!("gen:{f}#{seq}");
                    seq += 1;
                    k
                }
            };
            match by_id.iter_mut().find(|(k, _, _)| *k == key) {
                Some(slot) => {
                    slot.1 = model.to_string();
                    slot.2 = usage.clone();
                }
                None => by_id.push((key, model.to_string(), usage.clone())),
            }
        }
    }
    if by_id.is_empty() {
        return None;
    }

    let mut per_model: Vec<(String, Sums)> = Vec::new();
    for (_, model, usage) in &by_id {
        let idx = match per_model.iter().position(|(m, _)| m == model) {
            Some(i) => i,
            None => {
                per_model.push((model.clone(), Sums::default()));
                per_model.len() - 1
            }
        };
        let s = &mut per_model[idx].1;
        s.r#in += num(usage.get("input_tokens"));
        s.out += num(usage.get("output_tokens"));
        s.read += num(usage.get("cache_read_input_tokens"));
        let cc = usage.get("cache_creation").filter(|v| js_truthy(v));
        let tiered = cc.is_some_and(|c| {
            !matches!(c.get("ephemeral_5m_input_tokens"), None | Some(Value::Null))
                || !matches!(c.get("ephemeral_1h_input_tokens"), None | Some(Value::Null))
        });
        if tiered {
            let cc = cc.unwrap();
            s.c5 += num(cc.get("ephemeral_5m_input_tokens"));
            s.c1 += num(cc.get("ephemeral_1h_input_tokens"));
        } else {
            s.c5 += num(usage.get("cache_creation_input_tokens"));
        }
    }

    let mut parts: Vec<Part> = per_model
        .iter()
        .map(|(model, s)| {
            let (in_p, out_p) = price_for(model);
            // Left-to-right exactly as the .mjs writes it: each product is
            // formed first, the sums accumulate left-assoc, then /1e6.
            let cost = (s.r#in * in_p + s.out * out_p + s.c5 * in_p * 1.25 + s.c1 * in_p * 2.0
                + s.read * in_p * 0.1)
                / 1e6;
            Part {
                model: short_name(model),
                new_tokens: s.r#in + s.out + s.c5 + s.c1,
                cached_tokens: s.read,
                cost,
            }
        })
        .collect();
    // `parts.sort((a, b) => b.cost - a.cost)` — descending, and stable in
    // both V8 (TimSort) and Rust (`sort_by`).
    parts.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap_or(Ordering::Equal));

    let total = parts.iter().fold(0.0f64, |acc, p| acc + p.cost);
    let usage_line = parts
        .iter()
        .map(|p| {
            format!(
                "{} {} new/{} cached",
                p.model,
                fmt_tok(p.new_tokens),
                fmt_tok(p.cached_tokens)
            )
        })
        .collect::<Vec<_>>()
        .join(" + ");
    let cost_line = format!(
        "{}{}",
        parts
            .iter()
            .map(|p| format!("{} {}", p.model, fmt_usd(p.cost)))
            .collect::<Vec<_>>()
            .join(" + "),
        if parts.len() > 1 {
            format!(" = {} billed", fmt_usd(total))
        } else {
            " billed".to_string()
        }
    );
    let line = format!("{usage_line}\n{cost_line}");

    // Best-effort cache write (the .mjs swallows every failure).
    let body = serde_json::json!({ "sig": sig, "line": line });
    let _ = std::fs::write(&cache_file, jsjson::stringify(&body));

    Some(line)
}

pub(super) fn run(args: &[&str]) -> Option<ExitCode> {
    // The .mjs takes no arguments at all; any flag is an unproven shape.
    if !args.is_empty() {
        return None;
    }
    let mut raw = String::new();
    if std::io::stdin().read_to_string(&mut raw).is_err() {
        return Some(ExitCode::SUCCESS);
    }
    let Ok(input) = serde_json::from_str::<Value>(&raw) else {
        return Some(ExitCode::SUCCESS); // JSON.parse threw -> exit 0, no output
    };
    if let Some(line) = compute_line(&input, &js_tmpdir()) {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(line.as_bytes());
        let _ = stdout.flush();
    }
    Some(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn to_fixed_follows_the_js_tie_rule_not_rusts() {
        // The reachable divergence: 1.25 is exact, JS rounds the tie UP.
        assert_eq!(js_to_fixed(1.25, 1), "1.3");
        assert_eq!(format!("{:.1}", 1.25f64), "1.2"); // Rust: half-to-even
        assert_eq!(js_to_fixed(10.5, 0), "11");
        assert_eq!(js_to_fixed(0.125, 2), "0.13");
        assert_eq!(js_to_fixed(2.675, 2), "2.67"); // the double is < 2.675
        assert_eq!(js_to_fixed(0.0, 3), "0.000");
        assert_eq!(js_to_fixed(9.999, 2), "10.00"); // carry out of the fraction
        assert_eq!(js_to_fixed(99.5, 0), "100");
    }

    #[test]
    fn fmt_tok_matches_node() {
        assert_eq!(fmt_tok(0.0), "0");
        assert_eq!(fmt_tok(999.0), "999");
        assert_eq!(fmt_tok(1000.0), "1k");
        assert_eq!(fmt_tok(1500.0), "2k"); // Math.round ties toward +Inf
        assert_eq!(fmt_tok(2500.0), "3k");
        assert_eq!(fmt_tok(999_999.0), "1000k");
        assert_eq!(fmt_tok(1_000_000.0), "1.0M");
        assert_eq!(fmt_tok(1_250_000.0), "1.3M"); // the toFixed tie
        assert_eq!(fmt_tok(12_340_000.0), "12.3M");
    }

    #[test]
    fn fmt_usd_matches_node() {
        assert_eq!(fmt_usd(0.0), "$0.000");
        assert_eq!(fmt_usd(0.0994), "$0.099");
        assert_eq!(fmt_usd(0.1), "$0.10");
        assert_eq!(fmt_usd(9.999), "$10.00");
        assert_eq!(fmt_usd(10.0), "$10");
        assert_eq!(fmt_usd(10.5), "$11");
        assert_eq!(fmt_usd(123.456), "$123");
    }

    #[test]
    fn short_name_strips_prefix_and_date_suffix() {
        assert_eq!(short_name("claude-opus-4-20250514"), "opus-4");
        assert_eq!(short_name("claude-sonnet-5"), "sonnet-5");
        assert_eq!(short_name("gpt-5-codex"), "gpt-5-codex");
        assert_eq!(short_name("claude-3-5-haiku-20241022"), "3-5-haiku");
        // an 8-digit run not preceded by `-` is not a date suffix
        assert_eq!(short_name("model12345678"), "model12345678");
    }

    #[test]
    fn price_table_order_puts_sonnet_5_before_sonnet() {
        assert_eq!(price_for("claude-sonnet-5-20260101"), (2.0, 10.0));
        assert_eq!(price_for("claude-sonnet-4"), (3.0, 15.0));
        assert_eq!(price_for("claude-opus-5"), (5.0, 25.0));
        assert_eq!(price_for("fable-1"), (10.0, 50.0));
        assert_eq!(price_for("mythos-x"), (10.0, 50.0));
        assert_eq!(price_for("claude-haiku-4"), (1.0, 5.0));
        assert_eq!(price_for("something-else"), (5.0, 25.0));
    }

    #[test]
    fn subagent_dir_is_built_the_way_the_mjs_builds_it() {
        // path.join(path.dirname(t), path.basename(t, ".jsonl"), "subagents")
        let sep = super::super::jspath::SEP;
        let t = format!("{sep}a{sep}b{sep}sess.jsonl");
        assert_eq!(path_basename(&t, ".jsonl"), "sess");
        assert_eq!(path_dirname(&t), format!("{sep}a{sep}b"));
        assert_eq!(
            path_join(&[&path_dirname(&t), &path_basename(&t, ".jsonl"), "subagents"]),
            format!("{sep}a{sep}b{sep}sess{sep}subagents")
        );
    }

    // ── end-to-end over a transcript fixture ──────────────────────────────

    fn write_transcript(dir: &Path, name: &str, rows: &[Value]) -> String {
        let p = dir.join(name);
        let body = rows
            .iter()
            .map(|r| jsjson::stringify(r))
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&p, body).unwrap();
        p.to_string_lossy().into_owned()
    }

    fn row(id: &str, model: &str, usage: Value) -> Value {
        json!({ "message": { "id": id, "model": model, "usage": usage } })
    }

    #[test]
    fn js_tmpdir_is_a_real_directory() {
        // The cache location is shared with Node; the exact selection rules
        // are Node's, and the only thing assertable without env mutation is
        // that the result resolves.
        assert!(Path::new(&js_tmpdir()).is_dir(), "os.tmpdir() must exist");
    }

    #[test]
    fn aggregates_main_transcript_and_subagents() {
        let tmp = tempfile::tempdir().unwrap();
        let sessions = tmp.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let transcript = write_transcript(
            &sessions,
            "sess.jsonl",
            &[
                row("m1", "claude-opus-5", json!({"input_tokens": 1000, "output_tokens": 500})),
                // last occurrence of an id wins (cumulative streaming rows)
                row("m1", "claude-opus-5", json!({"input_tokens": 2000, "output_tokens": 1000})),
                json!({"message": {"id": "s", "model": "<synthetic>", "usage": {"input_tokens": 9}}}),
                json!({"no_usage_here": true}),
            ],
        );
        let sub = sessions.join("sess").join("subagents");
        std::fs::create_dir_all(&sub).unwrap();
        write_transcript(
            &sub,
            "a.jsonl",
            &[row(
                "h1",
                "claude-haiku-4-20260101",
                json!({
                    "input_tokens": 100,
                    "output_tokens": 50,
                    "cache_read_input_tokens": 4000,
                    "cache_creation": {"ephemeral_5m_input_tokens": 200, "ephemeral_1h_input_tokens": 100}
                }),
            )],
        );

        let cache_dir = tmp.path().join("tmp");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let cache_dir_s = cache_dir.to_string_lossy().into_owned();
        let line = compute_line(&json!({ "transcript_path": transcript }), &cache_dir_s).unwrap();

        // opus: in 2000*5 + out 1000*25 = 35_000 / 1e6 = $0.035
        // haiku: 100*1 + 50*5 + 200*1*1.25 + 100*1*2 + 4000*1*0.1 = 1200/1e6 = $0.0012
        let expected = "opus-5 3k new/0 cached + haiku-4 450 new/4k cached\n\
                        opus-5 $0.035 + haiku-4 $0.001 = $0.036 billed";
        assert_eq!(line, expected);

        // Second call must hit the signature cache and return the same bytes.
        let again = compute_line(&json!({ "transcript_path": transcript }), &cache_dir_s).unwrap();
        assert_eq!(again, expected);
        let cached: Value = serde_json::from_str(
            &std::fs::read_to_string(cache_dir.join("claude-usage-sess.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(cached["line"].as_str().unwrap(), expected);
        assert!(cached["sig"].as_str().unwrap().starts_with("6;"));
    }

    #[test]
    fn single_model_omits_the_total_and_legacy_cache_field_is_used() {
        let tmp = tempfile::tempdir().unwrap();
        let transcript = write_transcript(
            tmp.path(),
            "one.jsonl",
            &[row(
                "x",
                "claude-sonnet-5",
                json!({"input_tokens": 10, "cache_creation_input_tokens": 1_000_000}),
            )],
        );
        let cache_dir = tmp.path().join("t");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let line = compute_line(
            &json!({ "transcript_path": transcript }),
            &cache_dir.to_string_lossy(),
        )
        .unwrap();
        // 10*2 + 1_000_000*2*1.25 = 2_500_020 / 1e6 = $2.50002
        assert_eq!(line, "sonnet-5 1.0M new/0 cached\nsonnet-5 $2.50 billed");
    }

    #[test]
    fn missing_or_empty_inputs_print_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("t");
        std::fs::create_dir_all(&cache_dir).unwrap();
        let c = cache_dir.to_string_lossy().into_owned();
        assert!(compute_line(&json!({}), &c).is_none());
        assert!(compute_line(&json!({ "transcript_path": "" }), &c).is_none());
        assert!(compute_line(&json!({ "transcript_path": "/nope/nope.jsonl" }), &c).is_none());
        let empty = write_transcript(tmp.path(), "e.jsonl", &[json!({"hello": 1})]);
        assert!(compute_line(&json!({ "transcript_path": empty }), &c).is_none());
    }
}
