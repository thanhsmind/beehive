// bee help surfaces — top-level `--help`, and group/command-scoped help
// (`bee <group> [verb] --help` [--json], GH #23).
//
// Registry data: crate::registry::REGISTRY_PAYLOAD is the
// `{schema_version, commands}` JSON string compiled into the binary — parsed
// ONCE here (serde_json preserve_order keeps registry order) and rendered.
//
// Semantics:
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
// Routing: only proven shapes are served —
//   - `--help` with every following token in {--json, --all};
//   - `<leading...> --help` where every flag token is in {--help, --json},
//     the leading tokens resolve (longest registry prefix, or the legacy
//     fallback shape) with NO extra tokens, and the resolved name matches at
//     least one registry entry (itself or a "<name>." prefix).
// Anything else — junk flags, `--json=x` forms, unresolved groups (the
// nearest-match machinery owns those bytes), stray positionals, non-unicode
// argv, linked-worktree roots — returns None before ANY output, and the
// unknown-command refusal reports it.

use crate::jsjson;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::verbs::{emit_unsupported_root, record_timing};
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
        if !strs[1..]
            .iter()
            .all(|t| *t == "--json" || *t == "--all" || *t == "--names")
        {
            return None;
        }
        return top_level(
            strs.contains(&"--json"),
            strs.contains(&"--all"),
            strs.contains(&"--names"),
            t0,
        );
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

fn top_level(json: bool, all: bool, names: bool, t0: Instant) -> Option<ExitCode> {
    let reg = registry()?;
    // Timing root: the resolved repo root, falling back to cwd. Help reads
    // NOTHING but the embedded registry, so the WIDE door serves both grant
    // states; only a broken link is left to emit.
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::None => cwd.clone(),
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "unknown", json, t0, &why))
        }
    };

    let total = reg.commands.len();
    if names {
        let rows: Vec<&Map<String, Value>> = if all {
            reg.commands.iter().collect()
        } else {
            reg.commands.iter().filter(|e| is_porcelain(e) && !is_unavailable(e)).collect()
        };
        let surface = if all { "all" } else { "porcelain" };
        if json {
            let mut manifest = Map::new();
            manifest.insert("schema_version".into(), reg.schema_version.clone());
            manifest.insert("surface".into(), Value::from(surface));
            manifest.insert("view".into(), Value::from("names"));
            manifest.insert("total_commands".into(), Value::from(total as u64));
            manifest.insert(
                "commands".into(),
                Value::Array(rows.iter().map(|e| Value::Object(names_entry(e))).collect()),
            );
            print!("{}\n", jsjson::stringify_pretty(&Value::Object(manifest)));
        } else {
            for entry in &rows {
                let invoke = entry.get("invoke").and_then(Value::as_str).unwrap_or("");
                let mark = if is_unavailable(entry) { " [not built]" } else { "" };
                println!("{invoke}{mark} — {}", first_sentence(entry));
            }
            println!(
                "{} command(s){}. Full text for one: `bee <command> --help`; everything at once: \
                 `bee --help --all`.",
                rows.len(),
                if all { String::new() } else { format!(" of {total}") }
            );
        }
        record_timing(&root, "unknown", t0, true);
        return Some(ExitCode::SUCCESS);
    }
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
        // The flow surface lists only what this binary can actually run. An
        // unavailable porcelain verb (`doctor`, after the Node deletion) used
        // to sit here looking like any other command; the whole point of the
        // porcelain list is that an agent can call everything on it.
        let porcelain: Vec<&Map<String, Value>> = reg
            .commands
            .iter()
            .filter(|e| is_porcelain(e) && !is_unavailable(e))
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
            let unavailable = reg.commands.iter().filter(|e| is_unavailable(e)).count();
            let footer = help_footer_lines(total, porcelain.len(), unavailable);
            print!("{}", render_help_text(&commands, &footer, &reg.schema_version));
        }
    }
    record_timing(&root, "unknown", t0, true);
    Some(ExitCode::SUCCESS)
}

// ── `bee internal --help` — the plumbing namespace's own surface ───────────

/// Everything that is NOT on the flow surface, rendered under the `bee
/// internal …` spelling. The list is the same registry rows `--help --all`
/// shows minus the flow verbs; what this adds is the NAME for them, so a
/// reader learns the namespace exists from the tool rather than from prose.
pub fn internal_surface(json: bool, t0: Instant) -> Option<ExitCode> {
    let reg = registry()?;
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::None => cwd.clone(),
        Roots::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, "internal", json, t0, &why))
        }
    };
    let plumbing: Vec<&Map<String, Value>> = reg
        .commands
        .iter()
        .filter(|e| !is_porcelain(e) && !is_unavailable(e))
        .collect();
    let entries: Vec<Value> = plumbing
        .iter()
        .map(|e| {
            let mut m = manifest_entry(e);
            // Spelled the way the namespace is called, not the way the
            // legacy top-level spelling reads.
            if let Some(Value::String(invoke)) = m.get_mut("invoke") {
                *invoke = format!("bee internal {}", invoke.trim_start_matches("bee "));
            }
            Value::Object(m)
        })
        .collect();
    if json {
        let mut manifest = Map::new();
        manifest.insert("schema_version".into(), reg.schema_version.clone());
        manifest.insert("surface".into(), Value::from("internal"));
        manifest.insert("total_commands".into(), Value::from(reg.commands.len() as u64));
        manifest.insert("commands".into(), Value::Array(entries));
        print!("{}\n", jsjson::stringify_pretty(&Value::Object(manifest)));
    } else {
        let footer = vec![format!(
            "{} plumbing command(s). Each also answers to its bare top-level spelling (`bee state gate` == `bee internal state gate`); `bee internal` is the one that says which surface it belongs to. Run \"bee --help\" for the flow surface.",
            plumbing.len()
        )];
        print!("{}", render_help_text(&entries, &footer, &reg.schema_version));
    }
    record_timing(&root, "internal", t0, true);
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
        Roots::None => cwd.clone(),
        Roots::Unsupported(why) => {
            let cmd = command_name.replace('.', " ");
            let j = rest.iter().any(|t| *t == "--json");
            return Some(emit_unsupported_root(&cwd, &cmd, j, t0, &why));
        }
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

// ── split_command_tokens / resolve_command ──────────────────────────────────

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
/// `unavailable` is the seventh key and the one addition to Node's list: it is
/// present ONLY on entries this binary declares but does not implement, so
/// every other entry's manifest is byte-identical to what it was. Without it
/// `--help --json` — the surface agents read to decide what to call — would go
/// on advertising 23 commands the dispatcher refuses.
const MANIFEST_KEYS: [&str; 7] = [
    "name",
    "invoke",
    "description",
    "parameters",
    "examples",
    "deprecated",
    "unavailable",
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

// ── `--names`: the index view ──────────────────────────────────────────────
//
// `bee --help --all --json` is the map an agent is told to read, and it is
// 212 KB — roughly 53k tokens, most of a small context window, to answer
// "what may I call". Every byte of it is real (each entry's description is
// the contract for that verb), so the fix is not to shorten the descriptions
// but to offer an INDEX: the invocation, whether it is built, and one
// sentence. ~7 KB for the whole registry. An agent reads the index, then
// spends the tokens on `bee <command> --help` for the one verb it is about
// to call.
//
// The `unavailable` marker rides along even here: an index that lists a verb
// this binary cannot run would recreate exactly the drift the registry laws
// exist to catch.

/// The first sentence of a description — up to the first ". ", em-dash clause
/// break, or 160 chars, whichever comes first.
fn first_sentence(entry: &Map<String, Value>) -> String {
    let desc = entry.get("description").and_then(Value::as_str).unwrap_or("");
    let cut = desc
        .find(". ")
        .map(|i| i + 1)
        .or_else(|| desc.find(" — ").or_else(|| desc.find(" - ")))
        .unwrap_or(desc.len());
    let mut s = desc[..cut.min(desc.len())].trim().to_string();
    if s.chars().count() > 160 {
        let end = s.char_indices().nth(157).map(|(i, _)| i).unwrap_or(s.len());
        s.truncate(end);
        s.push('…');
    }
    s
}

fn names_entry(entry: &Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for key in ["name", "invoke"] {
        if let Some(v) = entry.get(key) {
            out.insert(key.to_string(), v.clone());
        }
    }
    out.insert(
        "surface".into(),
        Value::from(if is_porcelain(entry) { "porcelain" } else { "plumbing" }),
    );
    out.insert("summary".into(), Value::from(first_sentence(entry)));
    if is_unavailable(entry) {
        out.insert("unavailable".into(), Value::Bool(true));
    }
    out
}

fn is_porcelain(entry: &Map<String, Value>) -> bool {
    // `e.surface === 'porcelain'` — strict string equality.
    entry.get("surface").and_then(Value::as_str) == Some("porcelain")
}

/// Declared in the registry, not built into this binary. See crate::catalog.
fn is_unavailable(entry: &Map<String, Value>) -> bool {
    entry.get("unavailable").and_then(Value::as_object).is_some()
}

fn surfaced_manifest_entry(entry: &Map<String, Value>) -> Map<String, Value> {
    let mut out = manifest_entry(entry);
    let surface = if is_porcelain(entry) { "porcelain" } else { "plumbing" };
    out.insert("surface".to_string(), Value::from(surface));
    out
}

fn help_footer_lines(total: usize, porcelain: usize, unavailable: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "{} more command(s) are plumbing — run \"bee internal --help\" for that namespace, or \"bee --help --all\" for every command at once.",
        total - porcelain - unavailable
    )];
    if unavailable > 0 {
        // Counted separately and said out loud. Folding them into "plumbing"
        // would keep the old lie in a new place: they are not hidden because
        // they are low-level, they are hidden because they do not run.
        lines.push(format!(
            "{unavailable} command(s) are declared in the registry but NOT built into this binary; \
             \"bee --help --all\" marks each one with its reason."
        ));
    }
    // Named here because a reader who has not seen this line will reach for
    // `--all` — which is the full contract text for 139 commands, and by far
    // the most expensive thing this CLI can print into a context window.
    lines.push(
        "For an index instead of full text — one line per command — add \"--names\" to either \
         form."
            .to_string(),
    );
    lines
}

// ── render_help_text ─────────────────────────────────────────────────────

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

/// First three characters of a JSON-schema type name (`"string"` → `"str"`),
/// the same abbreviation `command_surface_lines` (session_preamble/budget.rs)
/// renders with — kept as its own copy here since this module reads
/// registry-shaped `Value` maps, never the parsed `catalog::Entry` that
/// section works from.
fn type_abbrev(schema_type: &str) -> String {
    schema_type.chars().take(3).collect()
}

/// The full declared flag surface for one manifest entry, in the compact
/// shape the session preamble's `### Command surface` section uses:
/// `--flag*:type ...`, `*` marking a required flag, `--json` always dropped
/// (its omission is stated once, in the render's own header note, never
/// repeated per line). `None` when the entry declares no properties at all
/// (or only `json`) — the fka-4 required-only line's "only a non-empty
/// list prints" behavior, folded to cover optional flags too.
fn flags_line(entry: &Value) -> Option<String> {
    let properties = entry
        .as_object()?
        .get("parameters")?
        .get("properties")?
        .as_object()?;
    let required: HashSet<&str> = entry
        .as_object()
        .and_then(|m| m.get("parameters"))
        .and_then(|p| p.get("required"))
        .and_then(Value::as_array)
        .map(|r| r.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let flags: Vec<String> = properties
        .iter()
        .filter(|(k, _)| k.as_str() != "json")
        .map(|(k, v)| {
            let star = if required.contains(k.as_str()) { "*" } else { "" };
            let t = v.get("type").and_then(Value::as_str).unwrap_or("value");
            format!("--{k}{star}:{}", type_abbrev(t))
        })
        .collect();
    if flags.is_empty() {
        None
    } else {
        Some(flags.join(" "))
    }
}

/// Detailed per-flag breakdown for a single-entry render (`bee <verb> --help`).
/// One indented line per flag: `--name`, `*` when required, the type
/// abbreviation in parens, an em-dash, and the registry description. A flag
/// with no description falls back to `--name*:typ`. `--json` is omitted.
/// `None` when the entry declares no properties at all (or only `json`).
fn flags_detail_lines(entry: &Value) -> Option<Vec<String>> {
    let properties = entry
        .as_object()?
        .get("parameters")?
        .get("properties")?
        .as_object()?;
    let required: HashSet<&str> = entry
        .as_object()
        .and_then(|m| m.get("parameters"))
        .and_then(|p| p.get("required"))
        .and_then(Value::as_array)
        .map(|r| r.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let flags: Vec<String> = properties
        .iter()
        .filter(|(k, _)| k.as_str() != "json")
        .map(|(k, v)| {
            let star = if required.contains(k.as_str()) { "*" } else { "" };
            let t = v.get("type").and_then(Value::as_str).unwrap_or("value");
            let abbrev = type_abbrev(t);
            match v.get("description").and_then(Value::as_str) {
                Some(desc) if !desc.trim().is_empty() => {
                    format!("--{k}{star} ({abbrev}) — {desc}")
                }
                _ => format!("--{k}{star}:{abbrev}"),
            }
        })
        .collect();
    if flags.is_empty() {
        None
    } else {
        Some(flags)
    }
}

/// True when at least one rendered entry declares a `json` property — the
/// trigger for the one, render-wide header note that states its omission
/// instead of repeating it on every flags line.
fn any_entry_takes_json(entries: &[Value]) -> bool {
    entries.iter().any(|e| {
        e.as_object()
            .and_then(|m| m.get("parameters"))
            .and_then(|p| p.get("properties"))
            .and_then(Value::as_object)
            .is_some_and(|props| props.contains_key("json"))
    })
}

fn render_help_text(entries: &[Value], footer_lines: &[String], schema_version: &Value) -> String {
    let mut lines: Vec<String> = vec![format!(
        "bee — unified CLI dispatcher (schema_version {})",
        jsjson::js_to_string(schema_version)
    )];
    if any_entry_takes_json(entries) {
        lines.push(
            "Nearly every command below also takes a `json` flag for machine-readable output \
             — omitted from every flags line."
                .to_string(),
        );
    }
    lines.push(String::new());
    let is_single = entries.len() == 1;
    for entry in entries {
        let get = |k: &str| entry.as_object().and_then(|m| m.get(k));
        lines.push(js_display(get("invoke")));
        lines.push(format!("    {}", js_display(get("description"))));
        // Single-entry renders print a detail block with descriptions;
        // multi-entry renders keep the compact flags_line.
        if is_single {
            if let Some(detail) = flags_detail_lines(entry) {
                lines.push("    flags:".to_string());
                for flag in detail {
                    lines.push(format!("      {flag}"));
                }
            }
        } else if let Some(flags) = flags_line(entry) {
            lines.push(format!("    flags: {flags}"));
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
        // Same shape as the DEPRECATED line, and deliberately louder: a
        // deprecated command still runs, an unavailable one does not.
        if let Some(gap) = get("unavailable") {
            if js_truthy(gap) {
                lines.push(format!(
                    "    NOT BUILT INTO THIS BINARY — {}.",
                    js_display(gap.get("reason"))
                ));
                lines.push(format!("    instead: {}", js_display(gap.get("fix"))));
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
        // The seventh key rides along ONLY where the registry set it, so an
        // ordinary entry's manifest is unchanged.
        assert!(m.get("unavailable").is_none());
    }

    #[test]
    fn an_unavailable_entry_carries_the_marker_last() {
        let e = obj(
            r#"{"name":"doctor","invoke":"bee doctor","surface":"porcelain","description":"d",
                "parameters":{"type":"object","properties":{},"required":[]},
                "examples":["bee doctor"],"deprecated":null,
                "unavailable":{"reason":"never ported","fix":"use bee status"}}"#,
        );
        let m = manifest_entry(&e);
        assert_eq!(m.keys().last().map(String::as_str), Some("unavailable"));
        let text = render_help_text(&[Value::Object(m)], &[], &json!("1.0"));
        assert!(text.contains("NOT BUILT INTO THIS BINARY — never ported."), "{text}");
        assert!(text.contains("instead: use bee status"), "{text}");
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
            help_footer_lines(123, 17, 0),
            vec![
                "106 more command(s) are plumbing — run \"bee internal --help\" for that namespace, or \"bee --help --all\" for every command at once."
                    .to_string(),
                // The index pointer is unconditional: a reader who never sees
                // it reaches for `--all`, which is the most expensive thing
                // this CLI prints.
                "For an index instead of full text — one line per command — add \"--names\" to either form."
                    .to_string(),
            ]
        );
    }

    /// Unavailable commands are subtracted from the plumbing count and named
    /// on their own line — "hidden because it is low-level" and "hidden
    /// because it does not run" are different facts.
    #[test]
    fn footer_separates_unavailable_from_plumbing() {
        let lines = help_footer_lines(123, 16, 23);
        assert!(lines[0].starts_with("84 more command(s) are plumbing"), "{lines:?}");
        assert!(lines[1].starts_with("23 command(s) are declared in the registry but NOT built"), "{lines:?}");
    }

    /// The live payload's own numbers: the flow surface must contain nothing
    /// the dispatcher would refuse by declaration.
    #[test]
    fn the_porcelain_surface_lists_no_unavailable_command() {
        let reg = registry().expect("embedded payload parses");
        let bad: Vec<&str> = reg
            .commands
            .iter()
            .filter(|e| is_porcelain(e) && is_unavailable(e))
            .filter_map(|e| e.get("name").and_then(Value::as_str))
            .collect();
        // The assertion that used to stand here REQUIRED at least one
        // unavailable porcelain command - written with `doctor` in mind, and
        // carrying its own instruction to drop it once doctor was ported.
        // Doctor is ported and no porcelain command is unavailable now, so
        // the requirement inverts: the flow surface is what an agent may
        // call, and nothing on it may be unbuilt.
        assert!(bad.is_empty(), "a porcelain command is marked unavailable: {bad:?}");
        // …and none of them reach the rendered porcelain list.
        let listed: Vec<&str> = reg
            .commands
            .iter()
            .filter(|e| is_porcelain(e) && !is_unavailable(e))
            .filter_map(|e| e.get("name").and_then(Value::as_str))
            .collect();
        for name in bad {
            assert!(!listed.contains(&name), "{name} still appears on the flow surface");
        }
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
                "parameters":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                "examples":[],"deprecated":null}"#,
        ))];
        let text = render_help_text(&entries, &[], &json!("1.0"));
        assert_eq!(
            text,
            "bee — unified CLI dispatcher (schema_version 1.0)\n\nbee cells show\n    Show one cell.\n    flags:\n      --id*:str\n"
        );
        // Footer path: body trimmed, two newlines, footer lines, one trailing
        // \n. Each footer line is its own paragraph, so the index pointer that
        // now follows the plumbing count must not run into it.
        let with_footer = render_help_text(&entries, &help_footer_lines(2, 1, 0), &json!("1.0"));
        assert!(with_footer.contains(
            "\n\n1 more command(s) are plumbing — run \"bee internal --help\" for that namespace, or \"bee --help --all\" for every command at once."
        ), "{with_footer}");
        assert!(with_footer.ends_with(
            "For an index instead of full text — one line per command — add \"--names\" to either form.\n"
        ), "{with_footer}");
        // Surfaced entry prints its surface line.
        let surfaced = vec![Value::Object(surfaced_manifest_entry(&obj(
            r#"{"name":"a","invoke":"bee a","description":"d","parameters":{"required":[]},"examples":[],"deprecated":null}"#,
        )))];
        let text = render_help_text(&surfaced, &[], &json!("1.0"));
        assert!(text.contains("\nbee a\n    d\n    surface: plumbing\n"));
    }

    /// hah-4: text help used to print only a `required:` line, so an
    /// optional flag like `--claim`/`--purpose` never appeared even though
    /// the registry declares it — pins the full-surface rendering the fix
    /// adds, on the live registry entry that first exposed the gap.
    #[test]
    fn dispatch_prepare_help_text_shows_the_full_flag_surface() {
        let reg = registry().expect("embedded payload parses");
        let entry = reg
            .commands
            .iter()
            .find(|e| e.get("name").and_then(Value::as_str) == Some("dispatch.prepare"))
            .expect("dispatch.prepare is registered");
        let text = render_help_text(&[Value::Object(manifest_entry(entry))], &[], &json!("1.0"));
        // Optional flags are visible at all.
        assert!(text.contains("--claim"), "{text}");
        assert!(text.contains("--purpose"), "{text}");
        // Required flags are marked distinct from optional ones.
        assert!(text.contains("--runtime* (str) —"), "{text}");
        assert!(text.contains("--kind* (str) —"), "{text}");
        assert!(text.contains("--claim (boo) —"), "{text}");
        assert!(text.contains("--purpose (str) —"), "{text}");
        // --json is dropped from the line and stated once instead.
        assert!(!text.contains("--json"), "{text}");
        assert!(text.contains("also takes a `json` flag"), "{text}");
    }

    #[test]
    fn single_entry_renders_flag_details_and_multi_entry_renders_compact() {
        let entry1 = Value::Object(obj(
            r#"{"name":"cmd.a","invoke":"bee cmd a","description":"Command A.",
                "parameters":{"type":"object","properties":{"task":{"type":"string","description":"Task text"},"dry-run":{"type":"boolean","description":"Dry run"},"undesc":{"type":"string"}},"required":["task"]},
                "examples":[],"deprecated":null}"#,
        ));
        let entry2 = Value::Object(obj(
            r#"{"name":"cmd.b","invoke":"bee cmd b","description":"Command B.",
                "parameters":{"type":"object","properties":{"id":{"type":"string"}},"required":["id"]},
                "examples":[],"deprecated":null}"#,
        ));

        // Single-entry: detailed block with descriptions, required stars, and fallback for undescribed flags.
        let single_text = render_help_text(&[entry1.clone()], &[], &json!("1.0"));
        assert!(single_text.contains("    flags:\n      --task* (str) — Task text\n      --dry-run (boo) — Dry run\n      --undesc:str"), "{single_text}");

        // Multi-entry: compact single line.
        let multi_text = render_help_text(&[entry1, entry2], &[], &json!("1.0"));
        assert!(multi_text.contains("    flags: --task*:str --dry-run:boo --undesc:str"), "{multi_text}");
        assert!(multi_text.contains("    flags: --id*:str"), "{multi_text}");
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
        // The six ALWAYS-present keys. `unavailable` (the seventh) is set
        // only on entries this binary does not implement — asserting it here
        // would demand every command be broken.
        let core = &MANIFEST_KEYS[..6];
        let all_have_six_keys =
            reg.commands.iter().all(|e| core.iter().all(|k| e.contains_key(*k)));
        assert!(all_have_six_keys, "registry entries carry the six manifest keys");
    }
}
