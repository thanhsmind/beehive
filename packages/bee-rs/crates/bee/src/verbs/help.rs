// bee help surfaces — Rust port of bee.mjs's handleHelp (~8334),
// renderHelpText (~8316), toManifestEntries (~8281), toSurfacedManifestEntries
// (~8304), helpFooterLine (~8311), and main()'s group/command-scoped help
// block (~8409, GH #23), backed by splitCommandTokens (~8110) and
// resolveCommand (~8129).
//
// Registry data: crate::registry::REGISTRY_PAYLOAD is the byte-exact
// `JSON.stringify({schema_version, commands})` string Node hashes — parsed
// ONCE here (serde_json preserve_order keeps registry order) and rendered.
//
// Verified semantics (bee.mjs + live Node runs, 2026-08-01):
//   - Top-level `--help` fires on `argv[0] === '--help'` ONLY, BEFORE root
//     resolution and BEFORE the drift-cache write (no manifest_changed line,
//     no .bee/cache/manifest-hash.json touch). The isDirectRun wrapper still
//     times the call: splitCommandTokens on a flag-only argv yields no
//     leading tokens, resolveCommand returns null, so the timing record and
//     the stderr line carry cmd 'unknown' ("[bee] unknown Nms"), appended to
//     <findRepoRoot(cwd) || cwd>/.bee/logs/timings.jsonl.
//   - Group/command-scoped help (`bee <group> [verb] --help` [--json]) also
//     fires BEFORE root resolution / drift; it times under the RESOLVED
//     command name with dots as spaces ("[bee] cells ready Nms").
//   - Field selection: toManifestEntries picks exactly {name, invoke,
//     description, parameters, examples, deprecated} in that literal order —
//     registry entries carry MORE keys (surface); a key absent on the source
//     entry destructures to undefined and JSON.stringify drops it, so
//     present-keys-only are copied here. --all appends surface LAST
//     (`entry.surface === 'porcelain' ? 'porcelain' : 'plumbing'`).
//   - JSON manifests are `JSON.stringify(manifest, null, 2) + "\n"`:
//     top-level {schema_version, surface, total_commands, commands} (surface
//     'porcelain'|'all'; total_commands is ALWAYS the full registry length),
//     group-scoped {schema_version, commands}.
//
// Strangler routing: only proven shapes are served —
//   - `--help` with every following token in {--json, --all};
//   - `<leading...> --help` where every flag token is in {--help, --json},
//     the leading tokens resolve (longest registry prefix, or the legacy
//     fallback shape) with NO extra tokens, and the resolved name matches at
//     least one registry entry (itself or a "<name>." prefix).
// Anything else — junk flags, `--json=x` forms, unresolved groups (Node's
// GROUP_USAGE_FALLBACKS / nearest-match machinery owns those bytes), stray
// positionals, non-unicode argv, linked-worktree roots — returns None before
// ANY output and the whole command re-runs under Node.

use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::verbs::record_timing;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::process::ExitCode;
use std::sync::OnceLock;
use std::time::Instant;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    let strs: Vec<&str> = args
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if strs.first() == Some(&"--help") {
        // handleHelp reads json/all via argv.includes(...) — any OTHER token
        // is ignored by Node; those unproven shapes delegate instead.
        if !strs[1..].iter().all(|t| *t == "--json" || *t == "--all") {
            return None;
        }
        return top_level(strs.contains(&"--json"), strs.contains(&"--all"), t0);
    }
    group_scoped(&strs, t0)
}

// ── embedded registry, parsed once ─────────────────────────────────────────

struct ParsedRegistry {
    schema_version: Value,
    commands: Vec<Map<String, Value>>,
}

fn parse_registry() -> Option<ParsedRegistry> {
    let payload: Value = serde_json::from_str(crate::registry::REGISTRY_PAYLOAD).ok()?;
    let obj = payload.as_object()?;
    let schema_version = obj.get("schema_version")?.clone();
    let commands = obj
        .get("commands")?
        .as_array()?
        .iter()
        .map(|e| e.as_object().cloned())
        .collect::<Option<Vec<_>>>()?;
    Some(ParsedRegistry {
        schema_version,
        commands,
    })
}

fn registry() -> Option<&'static ParsedRegistry> {
    static CELL: OnceLock<Option<ParsedRegistry>> = OnceLock::new();
    CELL.get_or_init(parse_registry).as_ref()
}

// ── top-level --help (handleHelp) ──────────────────────────────────────────

fn top_level(json: bool, all: bool, t0: Instant) -> Option<ExitCode> {
    let reg = registry()?;
    // Timing root exactly as the wrapper computes it: findRepoRoot(cwd) ||
    // cwd. Linked worktrees (grant-registry half) stay with Node.
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::None => cwd,
        Roots::NeedsNode => return None,
    };

    let total = reg.commands.len();
    if all {
        let commands: Vec<Value> = reg
            .commands
            .iter()
            .map(|e| Value::Object(surfaced_manifest_entry(e)))
            .collect();
        if json {
            let mut manifest = Map::new();
            manifest.insert("schema_version".into(), reg.schema_version.clone());
            manifest.insert("surface".into(), Value::from("all"));
            manifest.insert("total_commands".into(), Value::from(total as u64));
            manifest.insert("commands".into(), Value::Array(commands));
            print!("{}\n", jsjson::stringify_pretty(&Value::Object(manifest)));
        } else {
            print!("{}", render_help_text(&commands, &[], &reg.schema_version));
        }
    } else {
        let porcelain: Vec<&Map<String, Value>> = reg
            .commands
            .iter()
            .filter(|e| is_porcelain(e))
            .collect();
        let commands: Vec<Value> = porcelain
            .iter()
            .map(|e| Value::Object(manifest_entry(e)))
            .collect();
        if json {
            let mut manifest = Map::new();
            manifest.insert("schema_version".into(), reg.schema_version.clone());
            manifest.insert("surface".into(), Value::from("porcelain"));
            manifest.insert("total_commands".into(), Value::from(total as u64));
            manifest.insert("commands".into(), Value::Array(commands));
            print!("{}\n", jsjson::stringify_pretty(&Value::Object(manifest)));
        } else {
            let footer = vec![help_footer_line(total, porcelain.len())];
            print!("{}", render_help_text(&commands, &footer, &reg.schema_version));
        }
    }
    record_timing(&root, "unknown", t0, true);
    Some(ExitCode::SUCCESS)
}

// ── group/command-scoped help (main() ~8409) ───────────────────────────────

fn group_scoped(argv: &[&str], t0: Instant) -> Option<ExitCode> {
    let (leading, rest) = split_command_tokens(argv);
    if leading.is_empty() || !rest.contains(&"--help") {
        return None;
    }
    // Proven-shape gate: Node ignores extra rest tokens in this branch, but
    // only the documented flag pair is served natively.
    if !rest.iter().all(|t| *t == "--help" || *t == "--json") {
        return None;
    }

    let reg = registry()?;
    let names: HashSet<&str> = reg
        .commands
        .iter()
        .filter_map(|e| e.get("name").and_then(Value::as_str))
        .collect();
    let (command_name, extra) = resolve_command(&leading, &names);
    if !extra.is_empty() {
        // Node still shows group help for e.g. `bee status foo --help`;
        // stray-positional shapes stay with Node regardless.
        return None;
    }
    let prefix = format!("{command_name}.");
    let filtered: Vec<&Map<String, Value>> = reg
        .commands
        .iter()
        .filter(|e| {
            e.get("name")
                .and_then(Value::as_str)
                .is_some_and(|n| n == command_name || n.starts_with(&prefix))
        })
        .collect();
    if filtered.is_empty() {
        // Unresolved group/verb: Node's fallback machinery owns those bytes.
        return None;
    }

    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::None => cwd,
        Roots::NeedsNode => return None,
    };

    // jsonRequested: rest.some(t === '--json' || t.startsWith('--json=')) —
    // the '=' forms never reach here (proven-shape gate above delegates them).
    let json_requested = rest.iter().any(|t| *t == "--json");
    let entries: Vec<Value> = filtered
        .iter()
        .map(|e| Value::Object(manifest_entry(e)))
        .collect();
    if json_requested {
        let mut manifest = Map::new();
        manifest.insert("schema_version".into(), reg.schema_version.clone());
        manifest.insert("commands".into(), Value::Array(entries));
        print!("{}\n", jsjson::stringify_pretty(&Value::Object(manifest)));
    } else {
        print!("{}", render_help_text(&entries, &[], &reg.schema_version));
    }
    record_timing(&root, &command_name.replace('.', " "), t0, true);
    Some(ExitCode::SUCCESS)
}

// ── splitCommandTokens / resolveCommand (bee.mjs ~8110/~8129) ──────────────

fn split_command_tokens<'a>(argv: &[&'a str]) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut i = 0;
    while i < argv.len() && !argv[i].starts_with("--") {
        i += 1;
    }
    (argv[..i].to_vec(), argv[i..].to_vec())
}

/// Longest-prefix match over registry names, with the legacy fallback shapes
/// (bare token for length 1, "<group>.<verb>" for length ≥ 2). Only called
/// with non-empty `leading`.
fn resolve_command(leading: &[&str], names: &HashSet<&str>) -> (String, Vec<String>) {
    for n in (1..=leading.len()).rev() {
        let candidate = leading[..n].join(".");
        if names.contains(candidate.as_str()) {
            return (candidate, leading[n..].iter().map(|s| s.to_string()).collect());
        }
    }
    if leading.len() == 1 {
        return (leading[0].to_string(), Vec::new());
    }
    (
        format!("{}.{}", leading[0], leading[1]),
        leading[2..].iter().map(|s| s.to_string()).collect(),
    )
}

// ── manifest entry shaping (toManifestEntries / toSurfacedManifestEntries) ─

/// toManifestEntries' object-literal key order. A key absent on the source
/// entry destructures to undefined in JS and JSON.stringify drops it — so
/// only present keys are copied; extra source keys (surface, ...) never leak.
const MANIFEST_KEYS: [&str; 6] = [
    "name",
    "invoke",
    "description",
    "parameters",
    "examples",
    "deprecated",
];

fn manifest_entry(entry: &Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for key in MANIFEST_KEYS {
        if let Some(v) = entry.get(key) {
            out.insert(key.to_string(), v.clone());
        }
    }
    out
}

fn is_porcelain(entry: &Map<String, Value>) -> bool {
    // `e.surface === 'porcelain'` — strict string equality.
    entry.get("surface").and_then(Value::as_str) == Some("porcelain")
}

fn surfaced_manifest_entry(entry: &Map<String, Value>) -> Map<String, Value> {
    let mut out = manifest_entry(entry);
    let surface = if is_porcelain(entry) { "porcelain" } else { "plumbing" };
    out.insert("surface".to_string(), Value::from(surface));
    out
}

fn help_footer_line(total: usize, porcelain: usize) -> String {
    format!(
        "{} more command(s) are plumbing, hidden here — run \"bee --help --all\" for the full surface.",
        total - porcelain
    )
}

// ── renderHelpText (bee.mjs ~8316) ─────────────────────────────────────────

/// JS template-literal interpolation for a manifest field: missing key →
/// "undefined", null → "null", strings raw.
fn js_display(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// ECMA WhiteSpace ∪ LineTerminator — String.prototype.trimEnd's char set.
fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}' | '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
            | '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}'
            | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{FEFF}'
    )
}

fn render_help_text(entries: &[Value], footer_lines: &[String], schema_version: &Value) -> String {
    let mut lines: Vec<String> = vec![
        format!(
            "bee — unified CLI dispatcher (schema_version {})",
            jsjson::js_to_string(schema_version)
        ),
        String::new(),
    ];
    for entry in entries {
        let get = |k: &str| entry.as_object().and_then(|m| m.get(k));
        lines.push(js_display(get("invoke")));
        lines.push(format!("    {}", js_display(get("description"))));
        // entry.parameters?.required || [] — only a non-empty array prints.
        let required = get("parameters")
            .and_then(|p| p.get("required"))
            .and_then(Value::as_array);
        if let Some(req) = required {
            if !req.is_empty() {
                let flags = req
                    .iter()
                    .map(|r| format!("--{}", jsjson::js_to_string(r)))
                    .collect::<Vec<_>>()
                    .join(", ");
                lines.push(format!("    required: {flags}"));
            }
        }
        if let Some(surface) = get("surface") {
            if js_truthy(surface) {
                lines.push(format!("    surface: {}", jsjson::js_to_string(surface)));
            }
        }
        if let Some(dep) = get("deprecated") {
            if js_truthy(dep) {
                lines.push(format!(
                    "    DEPRECATED since {} — use \"{}\" instead.",
                    js_display(dep.get("since")),
                    js_display(dep.get("use_instead"))
                ));
            }
        }
        lines.push(String::new());
    }
    let joined = lines.join("\n");
    let body = joined.trim_end_matches(is_js_whitespace);
    let footer = if footer_lines.is_empty() {
        String::new()
    } else {
        format!("\n\n{}", footer_lines.join("\n"))
    };
    format!("{body}{footer}\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(s: &str) -> Map<String, Value> {
        serde_json::from_str::<Value>(s).unwrap().as_object().unwrap().clone()
    }

    #[test]
    fn manifest_entry_selects_six_keys_in_order_and_drops_extras() {
        let e = obj(
            r#"{"name":"x","invoke":"bee x","surface":"porcelain","description":"d",
                "parameters":{"type":"object","properties":{},"required":[]},
                "examples":["bee x"],"deprecated":null,"internal":true}"#,
        );
        let m = manifest_entry(&e);
        assert_eq!(
            m.keys().collect::<Vec<_>>(),
            vec!["name", "invoke", "description", "parameters", "examples", "deprecated"]
        );
        assert!(m.get("surface").is_none());
        assert!(m.get("internal").is_none());
        assert_eq!(m.get("deprecated"), Some(&Value::Null));
    }

    #[test]
    fn manifest_entry_omits_absent_keys_like_json_stringify_drops_undefined() {
        let e = obj(r#"{"name":"x","invoke":"bee x","description":"d"}"#);
        let m = manifest_entry(&e);
        assert_eq!(m.keys().collect::<Vec<_>>(), vec!["name", "invoke", "description"]);
    }

    #[test]
    fn surfaced_entry_appends_surface_last_with_plumbing_default() {
        let p = surfaced_manifest_entry(&obj(
            r#"{"name":"a","invoke":"bee a","description":"d","parameters":null,"examples":[],"deprecated":null,"surface":"porcelain"}"#,
        ));
        assert_eq!(p.keys().last().map(String::as_str), Some("surface"));
        assert_eq!(p.get("surface"), Some(&json!("porcelain")));
        // No surface key (and any non-'porcelain' value) → 'plumbing'.
        let q = surfaced_manifest_entry(&obj(
            r#"{"name":"b","invoke":"bee b","description":"d","parameters":null,"examples":[],"deprecated":null}"#,
        ));
        assert_eq!(q.get("surface"), Some(&json!("plumbing")));
    }

    #[test]
    fn footer_counts_hidden_commands() {
        assert_eq!(
            help_footer_line(123, 17),
            "106 more command(s) are plumbing, hidden here — run \"bee --help --all\" for the full surface."
        );
    }

    #[test]
    fn resolve_command_longest_prefix_then_legacy_fallback() {
        let names: HashSet<&str> =
            ["status", "cells.ready", "state.worker.add"].into_iter().collect();
        assert_eq!(
            resolve_command(&["state", "worker", "add"], &names),
            ("state.worker.add".to_string(), vec![])
        );
        assert_eq!(
            resolve_command(&["status", "foo"], &names),
            ("status".to_string(), vec!["foo".to_string()])
        );
        // No prefix match: legacy shapes.
        assert_eq!(resolve_command(&["state"], &names), ("state".to_string(), vec![]));
        assert_eq!(
            resolve_command(&["state", "worker"], &names),
            ("state.worker".to_string(), vec![])
        );
        assert_eq!(
            resolve_command(&["a", "b", "c"], &names),
            ("a.b".to_string(), vec!["c".to_string()])
        );
    }

    #[test]
    fn split_command_tokens_stops_at_first_double_dash() {
        let (leading, rest) = split_command_tokens(&["cells", "ready", "--help", "x"]);
        assert_eq!(leading, vec!["cells", "ready"]);
        assert_eq!(rest, vec!["--help", "x"]);
        let (leading, rest) = split_command_tokens(&["--help", "--json"]);
        assert!(leading.is_empty());
        assert_eq!(rest, vec!["--help", "--json"]);
    }

    #[test]
    fn render_help_text_matches_node_shape() {
        let entries = vec![Value::Object(obj(
            r#"{"name":"cells.show","invoke":"bee cells show","description":"Show one cell.",
                "parameters":{"type":"object","properties":{},"required":["id"]},
                "examples":[],"deprecated":null}"#,
        ))];
        let text = render_help_text(&entries, &[], &json!("1.0"));
        assert_eq!(
            text,
            "bee — unified CLI dispatcher (schema_version 1.0)\n\nbee cells show\n    Show one cell.\n    required: --id\n"
        );
        // Footer path: body trimmed, two newlines, footer, one trailing \n.
        let with_footer = render_help_text(&entries, &[help_footer_line(2, 1)], &json!("1.0"));
        assert!(with_footer.ends_with(
            "\n\n1 more command(s) are plumbing, hidden here — run \"bee --help --all\" for the full surface.\n"
        ));
        // Surfaced entry prints its surface line.
        let surfaced = vec![Value::Object(surfaced_manifest_entry(&obj(
            r#"{"name":"a","invoke":"bee a","description":"d","parameters":{"required":[]},"examples":[],"deprecated":null}"#,
        )))];
        let text = render_help_text(&surfaced, &[], &json!("1.0"));
        assert!(text.contains("\nbee a\n    d\n    surface: plumbing\n"));
    }

    #[test]
    fn group_filter_matches_exact_name_and_dotted_prefix() {
        let reg = registry().expect("embedded payload parses");
        let cells: Vec<&str> = reg
            .commands
            .iter()
            .filter_map(|e| e.get("name").and_then(Value::as_str))
            .filter(|n| *n == "cells" || n.starts_with("cells."))
            .collect();
        assert!(cells.iter().all(|n| n.starts_with("cells")));
        assert!(cells.contains(&"cells.ready"));
        // A porcelain-only surface never filters group help: plumbing verbs
        // (surface absent) appear too.
        let all_have_six_keys = reg.commands.iter().all(|e| {
            MANIFEST_KEYS.iter().all(|k| e.contains_key(*k))
        });
        assert!(all_have_six_keys, "registry entries carry the six manifest keys");
    }
}
