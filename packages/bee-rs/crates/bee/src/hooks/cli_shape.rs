// cli_shape — write-guard check (d), the CLI-shape schema guard. Recognizes a
// bee-CLI-shaped token in a Bash command and validates it against the
// embedded command registry — a malformed bee CLI call is denied rather than
// executing unguarded.
//
// The command registry is NOT re-derived here: crate::registry::
// REGISTRY_PAYLOAD is the `{schema_version, commands}` JSON string compiled
// into the binary (shape and freshness pinned by
// tests/registry_contracts.rs). It is parsed once, insertion-ordered, the
// same way verbs/help.rs parses it for the help surface.
//
// RECOGNIZED SPELLINGS: both the legacy `node .bee/bin/bee.mjs <verb>` /
// `node .bee/bin/bee_<group>.mjs <verb>` shapes (DISPATCHER_RE /
// LEGACY_HELPER_RE) and the current BINARY spelling (`.bee/bin/bee <verb>`,
// or a bare `bee <verb>` on PATH) are recognized identically by
// `recognize_script`, so a malformed call is denied the same way regardless
// of which spelling it uses.
//
// The bare (path-less) `bee` token is only honoured in COMMAND position — the
// first token of a shell segment, or the first token after a run of
// `NAME=value` environment assignments (`BEE_AGENT_NAME=w1 bee cells cap …`,
// the spelling AGENTS.md prescribes). A `bee` appearing as a mid-command
// argument (`echo bee cells cap`) is left alone, which keeps recognition from
// inventing denials for prose. A PATH-SPELLED token (`.bee/bin/bee` or
// `.bee/bin/bee.mjs`) is recognized anywhere in the segment.
//
// Never fails: every arm is deterministic, so unlike most guard branches this
// one never needs the `Nd` delegate refusal.

use crate::hooks::write_guard::{js_trim, tokenize};
use serde_json::{Map, Value};
use std::sync::OnceLock;

// ─── embedded registry ─────────────────────────────────────────────────────

pub(crate) struct Entry {
    pub(crate) name: String,
    /// `${entry.invoke}` as a template string would render it — a missing key
    /// interpolates the literal "undefined" in JS, so that is what a missing
    /// `invoke` produces here too.
    pub(crate) invoke: String,
    /// `entry.parameters` verbatim (absent → Value::Null, which fails
    /// isValidParameterSchema exactly as `undefined` does).
    pub(crate) parameters: Value,
}

/// `String(v)` for the value kinds this module ever interpolates: a registry
/// `invoke` (always a string — pinned by tests/registry_contracts.rs) and an
/// `enum` member (always a string — same pin). The remaining arms exist so a
/// hand-edited registry degrades the way JS would rather than panicking; an
/// array/object would render as `1,2` / `[object Object]` in JS, and those are
/// spelled out here rather than left to serde.
fn js_string_of(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => crate::jsjson::stringify(&Value::Number(n.clone())),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| match i {
                Value::Null => String::new(), // Array.join renders null/undefined as ""
                other => js_string_of(Some(other)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".to_string(),
    }
}

fn parse_registry() -> Vec<Entry> {
    let Ok(payload) = serde_json::from_str::<Value>(crate::registry::REGISTRY_PAYLOAD) else {
        return Vec::new();
    };
    let Some(commands) = payload.get("commands").and_then(Value::as_array) else {
        return Vec::new();
    };
    commands
        .iter()
        .filter_map(|e| {
            let obj = e.as_object()?;
            // `registry.map((e) => e.name)` / `registry.find(c => c.name === …)`
            // — an entry without a string name can never be matched by either.
            let name = match obj.get("name") {
                Some(Value::String(s)) => s.clone(),
                _ => return None,
            };
            Some(Entry {
                name,
                invoke: js_string_of(obj.get("invoke")),
                parameters: obj.get("parameters").cloned().unwrap_or(Value::Null),
            })
        })
        .collect()
}

pub(crate) fn registry() -> &'static [Entry] {
    static CELL: OnceLock<Vec<Entry>> = OnceLock::new();
    CELL.get_or_init(parse_registry)
}

// ─── JS Number(string) finiteness (validate-args typeMatches) ──────────────

/// `Number.isFinite(Number(s))` for a string `s`. The StringNumericLiteral
/// grammar in full: optional sign + `Infinity`, the 0x/0o/0b radix forms (no
/// sign permitted), or a decimal literal. Anything else is NaN → not finite.
/// The caller has already excluded the all-whitespace case, whose `Number("")
/// === 0` would be finite.
fn js_number_is_finite(s: &str) -> bool {
    let t = js_trim(s);
    if t.is_empty() {
        return true; // Number("") === 0
    }
    let (body, signed) = match t.strip_prefix(['+', '-']) {
        Some(rest) => (rest, true),
        None => (t, false),
    };
    if body == "Infinity" {
        return false;
    }
    if !signed && body.len() > 2 {
        let radix = match &body[..2] {
            "0x" | "0X" => Some(16u32),
            "0o" | "0O" => Some(8),
            "0b" | "0B" => Some(2),
            _ => None,
        };
        if let Some(radix) = radix {
            let digits = &body[2..];
            let mut acc = 0f64;
            for c in digits.chars() {
                let Some(d) = c.to_digit(radix) else { return false };
                acc = acc * f64::from(radix) + f64::from(d);
                if !acc.is_finite() {
                    return false; // overflowed to Infinity
                }
            }
            return true;
        }
    }
    // DecimalLiteral: digits [ '.' digits? ] | '.' digits , optional exponent.
    let b = body.as_bytes();
    let mut i = 0usize;
    let int_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    let mut frac_len = 0usize;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let fs = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        frac_len = i - fs;
    }
    if int_len == 0 && frac_len == 0 {
        return false;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if matches!(b.get(i), Some(b'+') | Some(b'-')) {
            i += 1;
        }
        let es = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == es {
            return false;
        }
    }
    if i != b.len() {
        return false;
    }
    // Rust's f64 FromStr accepts this grammar exactly; overflow yields inf,
    // matching JS.
    t.parse::<f64>().map(f64::is_finite).unwrap_or(false)
}

// ─── parsed argv values ────────────────────────────────────────────────────

/// parseCliFlags only ever produces a string or the boolean `true` — argv
/// parsing never yields a real number or array. Modelling exactly that domain
/// keeps typeMatches total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ArgVal {
    Str(String),
    True,
}

/// Insertion-ordered `{flag: value}` object. A repeated flag OVERWRITES in
/// place (JS keeps a property's original insertion position on re-assignment).
#[derive(Debug, Default, Clone)]
pub(crate) struct ParsedArgs(Vec<(String, ArgVal)>);

impl ParsedArgs {
    fn set(&mut self, key: String, value: ArgVal) {
        match self.0.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = value,
            None => self.0.push((key, value)),
        }
    }

    fn get(&self, key: &str) -> Option<&ArgVal> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// `Object.entries(args)` order: array-index-like keys first in ascending
    /// numeric order, then every other key in insertion order.
    fn entries(&self) -> Vec<(&String, &ArgVal)> {
        let mut indexed: Vec<(u32, &(String, ArgVal))> = Vec::new();
        let mut rest: Vec<&(String, ArgVal)> = Vec::new();
        for pair in &self.0 {
            match array_index_key(&pair.0) {
                Some(n) => indexed.push((n, pair)),
                None => rest.push(pair),
            }
        }
        indexed.sort_by_key(|(n, _)| *n);
        indexed
            .into_iter()
            .map(|(_, p)| p)
            .chain(rest)
            .map(|(k, v)| (k, v))
            .collect()
    }
}

/// An "array index" property key: the canonical decimal spelling of a u32
/// below 2^32-1.
fn array_index_key(key: &str) -> Option<u32> {
    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if key.len() > 1 && key.starts_with('0') {
        return None;
    }
    let n: u32 = key.parse().ok()?;
    (n != u32::MAX).then_some(n)
}

// ─── parameter-schema validation ────────────────────────────────────────────

/// isValidParameterSchema — the structural D3 check.
pub(crate) fn is_valid_parameter_schema(schema: &Value) -> bool {
    let Value::Object(obj) = schema else { return false };
    if !matches!(obj.get("type"), Some(Value::String(t)) if t == "object") {
        return false;
    }
    let Some(Value::Object(props)) = obj.get("properties") else {
        return false;
    };
    let Some(Value::Array(required)) = obj.get("required") else {
        return false;
    };
    for field in required {
        match field {
            Value::String(name) if props.contains_key(name) => {}
            _ => return false,
        }
    }
    for prop_schema in props.values() {
        // `!propSchema || typeof propSchema.type !== 'string'` — every
        // non-object (and every object without a string `type`) fails, because
        // reading `.type` off it yields undefined.
        match prop_schema {
            Value::Object(m) if matches!(m.get("type"), Some(Value::String(_))) => {}
            _ => return false,
        }
    }
    true
}

/// isPresent — `value !== undefined && value !== null && value !== ''`.
fn is_present(value: Option<&ArgVal>) -> bool {
    match value {
        None => false,
        Some(ArgVal::Str(s)) => !s.is_empty(),
        Some(ArgVal::True) => true,
    }
}

/// typeMatches, over the argv value domain (string | true).
fn type_matches(json_type: &str, value: &ArgVal) -> bool {
    match json_type {
        "string" => matches!(value, ArgVal::Str(_)),
        "boolean" => match value {
            ArgVal::True => true,
            ArgVal::Str(s) => s == "true" || s == "false",
        },
        "number" | "integer" => match value {
            ArgVal::True => false, // typeof true is neither 'number' nor 'string'
            ArgVal::Str(s) => !js_trim(s).is_empty() && js_number_is_finite(s),
        },
        // Array.isArray(true) is false and typeof true !== 'string'.
        "array" => matches!(value, ArgVal::Str(_)),
        _ => true,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Problem {
    /// `null` for the schema-invalid problem, otherwise the flag name.
    pub(crate) field: Option<String>,
    pub(crate) reason: String,
}

/// validate() — `None` is `{ok:true}`; `Some(problems)` is the batched
/// refusal, `problems[0]` being the `error` shape Node reports.
pub(crate) fn validate(entry: &Entry, args: &ParsedArgs) -> Option<Vec<Problem>> {
    if !is_valid_parameter_schema(&entry.parameters) {
        return Some(vec![Problem {
            field: None,
            reason: "command has no valid JSON-Schema parameters".to_string(),
        }]);
    }
    let schema = entry.parameters.as_object().expect("validated above");
    let props = schema
        .get("properties")
        .and_then(Value::as_object)
        .expect("validated above");
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("validated above")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    let mut problems: Vec<Problem> = Vec::new();

    for field in &required {
        if !is_present(args.get(field)) {
            problems.push(Problem {
                field: Some((*field).to_string()),
                reason: "required, missing".to_string(),
            });
        }
    }

    for (field, value) in args.entries() {
        // `schema.properties[field]` reaching a prototype method (toString,
        // constructor, __proto__) yields something whose `.type` is undefined,
        // so typeMatches falls to its `default: true` arm and the enum arm
        // cannot fire either (the name is never in `required`) — indistinguish-
        // able from the `continue` below. Own properties only is exact.
        let Some(prop_schema) = props.get(field.as_str()) else {
            continue;
        };
        let prop_type = prop_schema.get("type").and_then(Value::as_str).unwrap_or("");
        if !type_matches(prop_type, value) {
            problems.push(Problem {
                field: Some(field.clone()),
                reason: format!("invalid type, expected {prop_type}"),
            });
            continue;
        }
        // DB3 guard: enum is enforced on REQUIRED fields only.
        if required.contains(&field.as_str()) {
            if let Some(Value::Array(choices)) = prop_schema.get("enum") {
                let hit = choices.iter().any(|c| match (c, value) {
                    (Value::String(c), ArgVal::Str(v)) => c == v,
                    (Value::Bool(true), ArgVal::True) => true,
                    _ => false,
                });
                if !hit {
                    let joined = choices
                        .iter()
                        .map(|c| js_string_of(Some(c)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    problems.push(Problem {
                        field: Some(field.clone()),
                        reason: format!("invalid value, expected one of {joined}"),
                    });
                }
            }
        }
    }

    (!problems.is_empty()).then_some(problems)
}

// ─── check_cli_shape ────────────────────────────────────────────────────────

const CLI_SEGMENT_SEPARATORS: [&str; 5] = ["&&", "||", ";", "|", "&"];

/// splitCliSegments.
fn split_cli_segments(tokens: Vec<String>) -> Vec<Vec<String>> {
    let mut segments = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for token in tokens {
        if CLI_SEGMENT_SEPARATORS.contains(&token.as_str()) {
            if !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
        } else {
            current.push(token);
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// Which CLI spelling a token's basename is, if any.
enum Script {
    /// LEGACY_HELPER_RE — carries `match[1]` in its ORIGINAL case (the regex is
    /// case-insensitive, so `bee_Cells.mjs` yields the group "Cells", which
    /// then fails to match any registry name and correctly fails open).
    Legacy(String),
    /// DISPATCHER_RE, or (R6a widening) the `bee` / `bee.exe` binary.
    Dispatcher,
}

/// `token.replace(/\\/g, '/').split('/').pop()`.
fn basename(token: &str) -> &str {
    let cut = token.rfind(['/', '\\']).map_or(0, |i| i + 1);
    &token[cut..]
}

fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.eq_ignore_ascii_case(b)
}

/// `/^bee_([a-z]+)\.mjs$/i` — the `i` flag widens `[a-z]` to both cases.
fn legacy_group(base: &str) -> Option<String> {
    let mid = base.strip_prefix("bee_").or_else(|| {
        base.get(..4)
            .filter(|p| eq_ignore_ascii_case(p, "bee_"))
            .and_then(|_| base.get(4..))
    })?;
    let mid = match mid.len().checked_sub(4).and_then(|n| mid.get(..n)) {
        Some(m) if eq_ignore_ascii_case(&mid[mid.len() - 4..], ".mjs") => m,
        _ => return None,
    };
    if mid.is_empty() || !mid.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(mid.to_string())
}

/// `NAME=value` — the only tokens allowed to precede a bare `bee` in command
/// position (POSIX per-command environment assignments).
fn is_env_assignment(token: &str) -> bool {
    let Some(eq) = token.find('=') else { return false };
    let name = &token[..eq];
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn recognize_script(segment: &[String], index: usize) -> Option<Script> {
    let token = &segment[index];
    let base = basename(token);
    if let Some(group) = legacy_group(base) {
        return Some(Script::Legacy(group));
    }
    if eq_ignore_ascii_case(base, "bee.mjs") {
        return Some(Script::Dispatcher);
    }
    // The binary spelling, recognized identically to the dispatcher shape.
    if eq_ignore_ascii_case(base, "bee") || eq_ignore_ascii_case(base, "bee.exe") {
        let path_spelled = token.contains('/') || token.contains('\\');
        let command_position =
            segment[..index].iter().all(|t| is_env_assignment(t));
        if path_spelled || command_position {
            return Some(Script::Dispatcher);
        }
    }
    None
}

struct Resolved {
    command_name: String,
    consumed: usize,
}

/// resolveCliCommandName — longest-prefix match over the registry's own names.
fn resolve_cli_command_name(
    script: &Script,
    positional: &[String],
    registry: &[Entry],
) -> Option<Resolved> {
    let is_dispatcher = matches!(script, Script::Dispatcher);
    let group: String = match script {
        Script::Legacy(g) => g.clone(),
        Script::Dispatcher => match positional.first() {
            // `!group` — an absent OR empty first token both fail the check.
            Some(g) if !g.is_empty() => g.clone(),
            _ => return None,
        },
    };
    if let Script::Legacy(_) = script {
        if group == "status" {
            return Some(Resolved { command_name: "status".into(), consumed: 0 });
        }
    } else {
        if group.starts_with('-') {
            return None;
        }
        if group == "status" {
            return Some(Resolved { command_name: "status".into(), consumed: 1 });
        }
    }

    let scan_from: &[String] = if is_dispatcher { &positional[1..] } else { positional };
    let mut verb_tokens: Vec<&String> = Vec::new();
    for token in scan_from {
        if token.starts_with('-') {
            break;
        }
        verb_tokens.push(token);
    }
    if verb_tokens.is_empty() {
        return None; // no verb token at all: ambiguous, fail open
    }

    let mut name_segments: Vec<&str> = vec![group.as_str()];
    name_segments.extend(verb_tokens.iter().map(|t| t.as_str()));
    for n in (2..=name_segments.len()).rev() {
        let candidate = name_segments[..n].join(".");
        if registry.iter().any(|e| e.name == candidate) {
            return Some(Resolved {
                command_name: candidate,
                consumed: if is_dispatcher { n } else { n - 1 },
            });
        }
    }
    None
}

/// parseCliFlags — the resolved entry's own schema decides whether a `--flag`
/// is boolean (consumes nothing) or value-taking (consumes the next token
/// UNCONDITIONALLY, even when that token itself starts with `--`).
fn parse_cli_flags(flag_tokens: &[String], properties: Option<&Map<String, Value>>) -> ParsedArgs {
    let mut parsed = ParsedArgs::default();
    let mut i = 0usize;
    while i < flag_tokens.len() {
        let token = &flag_tokens[i];
        if !token.starts_with("--") {
            i += 1;
            continue;
        }
        if let Some(eq) = token.find('=') {
            parsed.set(token[2..eq].to_string(), ArgVal::Str(token[eq + 1..].to_string()));
            i += 1;
            continue;
        }
        let name = token[2..].to_string();
        let is_boolean = properties
            .and_then(|p| p.get(&name))
            .and_then(|s| s.get("type"))
            .and_then(Value::as_str)
            == Some("boolean");
        if is_boolean {
            parsed.set(name, ArgVal::True);
        } else if let Some(next) = flag_tokens.get(i + 1) {
            parsed.set(name, ArgVal::Str(next.clone()));
            i += 1;
        } else {
            parsed.set(name, ArgVal::True);
        }
        i += 1;
    }
    parsed
}

/// checkCliShape — scan every shell segment for a recognizable bee-CLI
/// invocation and validate it. `Some(reason)` on the first structural mismatch.
pub(crate) fn check_cli_shape(command: &str) -> Option<String> {
    if command.is_empty() {
        return None;
    }
    let registry = registry();
    if registry.is_empty() {
        return None; // `!Array.isArray(registry)` — nothing to resolve against
    }
    for segment in split_cli_segments(tokenize(command)) {
        for i in 0..segment.len() {
            let Some(script) = recognize_script(&segment, i) else {
                continue;
            };
            let positional = &segment[i + 1..];
            // `break` on every non-match below: one bee-CLI call per segment.
            let Some(resolved) = resolve_cli_command_name(&script, positional, registry) else {
                break;
            };
            let Some(entry) = registry.iter().find(|e| e.name == resolved.command_name) else {
                break;
            };
            let flag_tokens = &positional[resolved.consumed.min(positional.len())..];
            let parsed = parse_cli_flags(
                flag_tokens,
                entry.parameters.get("properties").and_then(Value::as_object),
            );
            // `--help` is never a schema parameter — no registry entry declares a
            // `help` property — so a parsed `help` key can only be the caller
            // asking for the help surface. Required-parameter validation must not
            // stand in front of it: the denial's own Correction line names
            // `bee <cmd> --help --json`, which the guard would deny in turn.
            // The test is the PARSED key, not a raw token scan, so a `--help`
            // swallowed as another flag's value never disarms the guard.
            if parsed.get("help").is_some() {
                break;
            }
            if let Some(problems) = validate(entry, &parsed) {
                return Some(render_denial(command, entry, &problems));
            }
            break;
        }
    }
    None
}

/// The exact refusal bytes (ce-1: every problem rendered, the pinned
/// substrings unmoved — "bee CLI-shape guard", the entry name, and
/// "field: <first>" still naming the FIRST problem at the end).
fn render_denial(command: &str, entry: &Entry, problems: &[Problem]) -> String {
    let detail = problems
        .iter()
        .map(|p| match &p.field {
            Some(f) if !f.is_empty() => format!("{} (--{f})", p.reason),
            _ => p.reason.clone(),
        })
        .collect::<Vec<_>>()
        .join("; ");
    let first_field = problems
        .first()
        .and_then(|p| p.field.as_deref())
        .filter(|f| !f.is_empty());
    let field_clause = match first_field {
        Some(f) => format!(" (field: {f})"),
        None => String::new(),
    };
    format!(
        "bee CLI-shape guard: \"{}\" does not match {}'s schema — {detail}{field_clause}. \
Correction: run `{}` with the required parameters (see `{} --help --json`).",
        js_trim(command),
        entry.name,
        entry.invoke,
        entry.invoke
    )
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn deny(command: &str) -> String {
        check_cli_shape(command).unwrap_or_else(|| panic!("expected a denial for: {command}"))
    }

    fn allow(command: &str) {
        if let Some(r) = check_cli_shape(command) {
            panic!("expected fail-open for {command}, got: {r}");
        }
    }

    // ── the registry is really there ───────────────────────────────────────

    #[test]
    fn the_embedded_registry_parses_and_carries_the_commands_the_guard_resolves() {
        let reg = registry();
        assert!(reg.len() > 100, "registry parsed {} entries", reg.len());
        // `cells.tier` used to stand in this list; model-role-split D4 (store
        // `97ce5225`) retired it, and `cells.escalate` — the escalation half
        // of that verb, under its own name — stands in its place. Retargeted,
        // not dropped: the assertion is still "a real cells verb the guard
        // resolves is in the embedded registry".
        for name in ["cells.show", "cells.cap", "cells.escalate", "state.worker.add", "status"] {
            assert!(reg.iter().any(|e| e.name == name), "missing {name}");
        }
        let cap = reg.iter().find(|e| e.name == "cells.cap").unwrap();
        assert_eq!(cap.invoke, "bee cells cap");
        assert!(is_valid_parameter_schema(&cap.parameters));
    }

    // ── legacy/dispatcher spellings resolve identically (rows 5/5b/5c/5d) ──

    #[test]
    fn row5_plain_legacy_helper_invocations_fail_open() {
        allow("node .bee/bin/bee_state.mjs set --phase swarming");
        // bah-2: `--layer` joined the declared required set when
        // `backlog.add`'s registry entry was corrected to match its handler.
        // A well-shaped legacy call still fails open; row5c below is the
        // paired case proving a legacy call MISSING a required flag denies.
        allow("node .bee/bin/bee_backlog.mjs add --type bug --title \"x\" --severity P2 --layer cli");
    }

    #[test]
    fn row5c_legacy_cells_cap_missing_id_resolves_and_denies() {
        let reason = deny("node .bee/bin/bee_cells.mjs cap --outcome \"done\"");
        assert!(reason.contains("bee CLI-shape guard"), "{reason}");
        assert!(reason.contains("cells.cap"), "{reason}");
        assert!(reason.contains("field: id"), "{reason}");
    }

    #[test]
    fn row5d_dispatcher_cells_cap_missing_id_resolves_and_denies() {
        let reason = deny("node .bee/bin/bee.mjs cells cap --outcome \"done\"");
        assert!(reason.contains("cells.cap"), "{reason}");
        assert!(reason.contains("field: id"), "{reason}");
    }

    // ── check (a): required-flag denial ─────────────────────────────────────

    #[test]
    fn a_malformed_call_missing_a_required_flag_is_denied() {
        let reason = deny("node .bee/bin/bee_cells.mjs show");
        assert_eq!(
            reason,
            "bee CLI-shape guard: \"node .bee/bin/bee_cells.mjs show\" does not match cells.show's \
schema — required, missing (--id) (field: id). Correction: run `bee cells show` with the required \
parameters (see `bee cells show --help --json`)."
        );
    }

    /// Retargeted by model-role-split D4 (store `97ce5225`), which retired
    /// `cells tier`. `backlog.pbi.status` is its exact structural twin — two
    /// required fields, `id` first, the second carrying an enum — so every
    /// assertion below is the original one, moved to a subject that still
    /// exists. Nothing here was weakened or dropped.
    #[test]
    fn ce1_every_problem_is_rendered_and_the_first_still_owns_the_field_clause() {
        let reason = deny("node .bee/bin/bee.mjs backlog pbi status");
        assert!(reason.contains("bee CLI-shape guard"), "{reason}");
        assert!(reason.contains("backlog.pbi.status"), "{reason}");
        assert!(reason.contains("field: id"), "{reason}");
        assert_eq!(
            reason.matches("required, missing").count(),
            2,
            "both missing fields must be reported: {reason}"
        );
        assert!(reason.contains("(--to)"), "{reason}");
    }

    #[test]
    fn a_well_formed_call_is_allowed() {
        allow("node .bee/bin/bee_cells.mjs show --id demo-1");
    }

    #[test]
    fn a_flag_value_beginning_with_dash_dash_is_consumed_as_the_value() {
        allow("node .bee/bin/bee_decisions.mjs log --decision \"--foo\" --rationale bar --relation none");
    }

    #[test]
    fn a_boolean_flag_never_over_consumes_a_trailing_positional() {
        allow("node .bee/bin/bee_reservations.mjs sweep --json notaboolean");
    }

    #[test]
    fn three_token_longest_prefix_resolution_passes_a_valid_call() {
        allow("node .bee/bin/bee_state.mjs worker add --nickname w1 --cell c1 --json");
        allow("node .bee/bin/bee.mjs state worker add --nickname w1 --cell c1 --json");
    }

    #[test]
    fn three_token_resolution_denies_a_mistyped_boolean() {
        let reason = deny(
            "node .bee/bin/bee_state.mjs worker add --nickname w1 --cell c1 --json=notaboolean",
        );
        assert!(reason.contains("bee CLI-shape guard"), "{reason}");
        assert!(reason.contains("state.worker.add"), "{reason}");
        assert!(reason.contains("field: json"), "{reason}");
        assert!(reason.contains("invalid type, expected boolean"), "{reason}");
    }

    #[test]
    fn an_unrecognized_bash_call_is_left_alone() {
        allow("echo hello");
        allow("git status");
        allow("node scripts/whatever.mjs --id");
        allow("rm -rf build");
    }

    #[test]
    fn a_segment_whose_shape_is_ambiguous_fails_open() {
        allow("node .bee/bin/bee.mjs"); // no group at all
        allow("node .bee/bin/bee.mjs --help"); // group starts with '-'
        allow("node .bee/bin/bee.mjs cells"); // no verb token after the group
        allow("node .bee/bin/bee.mjs nosuchgroup nosuchverb --x y"); // resolves to nothing
    }

    #[test]
    fn asking_a_subcommand_for_its_help_reaches_the_help_surface() {
        // Reported from a Windows host: the guard denied `bee config set --help
        // --json` for the very parameters help would have explained, and its own
        // Correction line named that same denied command. Help now passes.
        allow("node .bee/bin/bee.mjs config set --help");
        allow("node .bee/bin/bee.mjs config set --help --json");
        // But only when `--help` was PARSED as help. Here it is consumed as the
        // value of `--key`, so the missing `--value` is still a denial.
        let reason = deny("node .bee/bin/bee.mjs config set --key --help");
        assert!(reason.contains("config.set"), "{reason}");
        assert!(reason.contains("(--value)"), "{reason}");
    }

    #[test]
    fn status_is_the_one_single_segment_name_both_shapes_special_case() {
        allow("node .bee/bin/bee.mjs status --json");
        allow("node .bee/bin/bee_status.mjs --json");
        // `close` has a required `feature` but is single-segment: the n>=2
        // longest-prefix loop can never reach it, so it fails open in BOTH
        // runtimes. Pinned so a future "improvement" is a deliberate one.
        allow("node .bee/bin/bee.mjs close");
    }

    #[test]
    fn only_the_first_bee_token_of_a_segment_is_examined() {
        // The first token resolves and validates clean; the second (malformed)
        // is never reached, because checkCliShape breaks out of the segment.
        allow("node .bee/bin/bee.mjs cells show --id a node .bee/bin/bee.mjs cells cap");
    }

    #[test]
    fn each_segment_is_scanned_independently() {
        let reason = deny("echo hi && node .bee/bin/bee_cells.mjs cap --outcome done");
        assert!(reason.contains("cells.cap"), "{reason}");
        // and the whole (untrimmed-per-segment) command is echoed back
        assert!(
            reason.contains("\"echo hi && node .bee/bin/bee_cells.mjs cap --outcome done\""),
            "{reason}"
        );
    }

    // ── R6a widening: the BINARY spelling ──────────────────────────────────

    #[test]
    fn the_binary_spelling_resolves_exactly_like_the_dispatcher() {
        for command in [
            ".bee/bin/bee cells cap --outcome done",
            "'.bee\\bin\\bee.exe' cells cap --outcome done",
            "bee.exe cells cap --outcome done",
            "bee cells cap --outcome done",
            "BEE_AGENT_NAME=w1 bee cells cap --outcome done",
            "BEE_AGENT_NAME=w1 FOO=bar .bee/bin/bee cells cap --outcome done",
        ] {
            let reason = check_cli_shape(command)
                .unwrap_or_else(|| panic!("expected a denial for: {command}"));
            assert!(reason.contains("cells.cap"), "{command}: {reason}");
            assert!(reason.contains("field: id"), "{command}: {reason}");
            assert!(
                reason.contains("Correction: run `bee cells cap`"),
                "{command}: {reason}"
            );
        }
    }

    #[test]
    fn the_binary_denial_bytes_are_the_dispatcher_denial_bytes_modulo_the_echoed_command() {
        let mjs = deny("node .bee/bin/bee.mjs cells show");
        let bin = deny(".bee/bin/bee cells show");
        assert_eq!(
            mjs.replace("\"node .bee/bin/bee.mjs cells show\"", "<cmd>"),
            bin.replace("\".bee/bin/bee cells show\"", "<cmd>")
        );
    }

    #[test]
    fn a_well_formed_binary_call_is_allowed() {
        allow(".bee/bin/bee cells cap --id demo-1 --outcome done --report \"cargo test -p bee — green:unit — touched close.rs\"");
        allow("bee status --json");
        allow("bee cells ready --json");
        allow("bee state worker add --nickname w1 --cell c1 --json");
        allow("bee decisions log --decision d --rationale r --relation none");
    }

    #[test]
    fn a_bare_bee_outside_command_position_is_never_recognized() {
        // Prose and arguments must not become denials — the widening only
        // claims the command position (or a path-spelled token).
        allow("echo bee cells cap");
        allow("rg \"bee cells cap\" docs/");
        allow("git commit -m bee cells cap");
        allow("npx bee cells cap");
        // ...but the path-spelled form IS claimed anywhere, exactly as Node
        // claimed `.bee/bin/bee.mjs` anywhere.
        assert!(check_cli_shape("npx .bee/bin/bee cells cap").is_some());
    }

    #[test]
    fn a_bare_bee_at_the_head_of_a_later_segment_is_command_position() {
        let reason = deny("echo hi && bee cells cap --outcome done");
        assert!(reason.contains("cells.cap"), "{reason}");
    }

    // ── tokenizer / segment interaction (D9 forms) ─────────────────────────

    #[test]
    fn every_separator_form_starts_a_fresh_segment() {
        for sep in ["&&", "||", ";", "|", "&"] {
            let glued = format!("echo hi{sep}bee cells cap --outcome done");
            let spaced = format!("echo hi {sep} bee cells cap --outcome done");
            for command in [glued, spaced] {
                let reason = check_cli_shape(&command)
                    .unwrap_or_else(|| panic!("expected a denial for: {command}"));
                assert!(reason.contains("cells.cap"), "{command}: {reason}");
            }
        }
    }

    #[test]
    fn a_quoted_separator_is_not_a_boundary() {
        // "bee" stays an argument of echo — no segment break, no denial.
        allow("echo 'hi;' bee cells cap");
    }

    // ── parameter validation unit table ─────────────────────────────────────

    #[test]
    fn type_matches_models_the_argv_value_domain() {
        assert!(type_matches("string", &ArgVal::Str("x".into())));
        assert!(!type_matches("string", &ArgVal::True));
        assert!(type_matches("boolean", &ArgVal::True));
        assert!(type_matches("boolean", &ArgVal::Str("true".into())));
        assert!(type_matches("boolean", &ArgVal::Str("false".into())));
        assert!(!type_matches("boolean", &ArgVal::Str("TRUE".into())));
        assert!(type_matches("number", &ArgVal::Str("12".into())));
        assert!(type_matches("number", &ArgVal::Str(" 12 ".into())));
        assert!(type_matches("number", &ArgVal::Str("1e3".into())));
        assert!(type_matches("number", &ArgVal::Str("0x10".into())));
        assert!(type_matches("number", &ArgVal::Str(".5".into())));
        assert!(!type_matches("number", &ArgVal::Str("".into())));
        assert!(!type_matches("number", &ArgVal::Str("  ".into())));
        assert!(!type_matches("number", &ArgVal::Str("12abc".into())));
        assert!(!type_matches("number", &ArgVal::Str("Infinity".into())));
        assert!(!type_matches("number", &ArgVal::Str("1e999".into())));
        assert!(!type_matches("number", &ArgVal::True));
        assert!(type_matches("array", &ArgVal::Str("a,b".into())));
        assert!(!type_matches("array", &ArgVal::True));
        assert!(type_matches("unknown-type", &ArgVal::True));
    }

    #[test]
    fn is_present_treats_only_absent_and_empty_string_as_missing() {
        assert!(!is_present(None));
        assert!(!is_present(Some(&ArgVal::Str(String::new()))));
        assert!(is_present(Some(&ArgVal::Str("x".into()))));
        assert!(is_present(Some(&ArgVal::True)));
    }

    #[test]
    fn an_empty_valued_required_flag_is_reported_missing() {
        let reason = deny("node .bee/bin/bee.mjs cells show --id=");
        assert!(reason.contains("required, missing (--id)"), "{reason}");
    }

    /// Retargeted by model-role-split D4 (store `97ce5225`): `cells.tier`'s
    /// required enum was the subject here until the verb retired with the
    /// selector it wrote. `backlog.pbi.status`'s `to` is required and carries
    /// an enum in exactly the same shape, so the assertion is unchanged —
    /// only the command it is taken against moved.
    #[test]
    fn enum_enforcement_is_scoped_to_required_fields() {
        let reason = deny("node .bee/bin/bee.mjs backlog pbi status --id p1 --to nope");
        assert!(
            reason.contains(
                "invalid value, expected one of proposed, in-flight, parked, done, declined"
            ),
            "{reason}"
        );
        assert!(reason.contains("field: to"), "{reason}");
        allow("node .bee/bin/bee.mjs backlog pbi status --id p1 --to done");
    }

    #[test]
    fn a_repeated_flag_overwrites_in_place_and_keeps_its_first_position() {
        let mut args = ParsedArgs::default();
        args.set("a".into(), ArgVal::Str("1".into()));
        args.set("b".into(), ArgVal::Str("2".into()));
        args.set("a".into(), ArgVal::Str("3".into()));
        let keys: Vec<&str> = args.entries().into_iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["a", "b"]);
        assert_eq!(args.get("a"), Some(&ArgVal::Str("3".into())));
        // the LAST value wins, so a valid re-spelling rescues an empty first
        allow("node .bee/bin/bee.mjs cells show --id= --id demo-1");
    }

    #[test]
    fn object_entries_order_hoists_array_index_keys() {
        let mut args = ParsedArgs::default();
        args.set("z".into(), ArgVal::True);
        args.set("10".into(), ArgVal::True);
        args.set("2".into(), ArgVal::True);
        args.set("a".into(), ArgVal::True);
        args.set("07".into(), ArgVal::True); // not canonical → a string key
        let keys: Vec<&str> = args.entries().into_iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["2", "10", "z", "a", "07"]);
    }

    #[test]
    fn is_valid_parameter_schema_rejects_every_malformed_shape() {
        use serde_json::json;
        assert!(is_valid_parameter_schema(
            &json!({"type":"object","properties":{"a":{"type":"string"}},"required":["a"]})
        ));
        assert!(!is_valid_parameter_schema(&Value::Null));
        assert!(!is_valid_parameter_schema(&json!([])));
        assert!(!is_valid_parameter_schema(&json!({"type":"array","properties":{},"required":[]})));
        assert!(!is_valid_parameter_schema(&json!({"type":"object","required":[]})));
        assert!(!is_valid_parameter_schema(&json!({"type":"object","properties":{}})));
        // a required name with no property is a validator that can never pass
        assert!(!is_valid_parameter_schema(
            &json!({"type":"object","properties":{},"required":["a"]})
        ));
        // a property without a string `type`
        assert!(!is_valid_parameter_schema(
            &json!({"type":"object","properties":{"a":{}},"required":[]})
        ));
        assert!(!is_valid_parameter_schema(
            &json!({"type":"object","properties":{"a":"string"},"required":[]})
        ));
    }

    #[test]
    fn legacy_group_matches_the_regex_including_its_case_insensitivity() {
        assert_eq!(legacy_group("bee_cells.mjs").as_deref(), Some("cells"));
        assert_eq!(legacy_group("BEE_Cells.MJS").as_deref(), Some("Cells"));
        assert_eq!(legacy_group("bee_.mjs"), None);
        assert_eq!(legacy_group("bee_cells2.mjs"), None);
        assert_eq!(legacy_group("bee_cells.js"), None);
        assert_eq!(legacy_group("bee.mjs"), None);
        // a mixed-case group resolves to no registry name → fail open
        allow("node .bee/bin/bee_CELLS.mjs cap --outcome done");
    }

    #[test]
    fn an_unquoted_windows_path_is_eaten_by_the_tokenizer() {
        // The tokenizer treats `\` as escaping the next character, so
        // `.bee\bin\bee.exe` tokenizes to `.beebinbee.exe` and resolves to
        // nothing. Pinned as a known limitation, not a defect.
        assert_eq!(tokenize(".bee\\bin\\bee.exe cells cap")[0], ".beebinbee.exe");
        allow(".bee\\bin\\bee.exe cells cap");
        // A quoted path keeps its backslashes and IS recognized.
        assert!(check_cli_shape("\".bee\\bin\\bee.exe\" cells cap").is_some());
    }

    #[test]
    fn basename_splits_on_both_separators() {
        assert_eq!(basename("a/b/c"), "c");
        assert_eq!(basename("a\\b\\c"), "c");
        assert_eq!(basename("c"), "c");
        assert_eq!(basename("a/"), "");
    }

    #[test]
    fn env_assignment_recognition_is_narrow() {
        assert!(is_env_assignment("A=1"));
        assert!(is_env_assignment("_a9=x"));
        assert!(is_env_assignment("BEE_AGENT_NAME="));
        assert!(!is_env_assignment("9A=1"));
        assert!(!is_env_assignment("A-B=1"));
        assert!(!is_env_assignment("plain"));
    }
}

// ─── the widening's safety fence ───────────────────────────────────────────
// Recognizing the binary spelling turns previously-unguarded calls into
// potential denials, so the corpus that matters is the instruction layer every
// agent copies from: if a SHIPPED command spelling would be refused, the
// widening is wrong, not the doc. This sweep is what makes that a test failure
// instead of a field incident.

#[cfg(test)]
mod documented_invocations {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        // crates/bee -> crates -> bee-rs -> packages -> <repo>
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap()
            .to_path_buf()
    }

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "md") {
                out.push(p);
            }
        }
    }

    /// Every runnable `bee …` / `.bee/bin/bee …` / `node .bee/bin/bee… …`
    /// command spelling on a FENCED CODE-BLOCK line.
    ///
    /// The scope is deliberate. A fenced block is a transcript — the literal
    /// bytes an agent copies into a shell — so a refusal there is a real
    /// incident. An inline backtick span in prose is usually a command NAME or
    /// a flag under discussion ("re-enter via `bee worktree new
    /// --with-companion`"); that is not a complete call and SHOULD be refused
    /// if run verbatim, so asserting the opposite would invert the guard.
    /// Templates carrying placeholders (`<id>`, `[--json]`, `{a|b}`) are not
    /// runnable bytes either and are excluded on the same reasoning.
    fn invocations(line: &str) -> Vec<String> {
        const HEADS: [&str; 3] = ["node .bee/bin/bee", ".bee/bin/bee ", "bee "];
        let mut out = Vec::new();
        for head in HEADS {
            let mut from = 0usize;
            while let Some(rel) = line[from..].find(head) {
                let start = from + rel;
                from = start + head.len();
                // a head must begin a word
                if start > 0 {
                    let prev = line[..start].chars().next_back().unwrap();
                    if prev.is_alphanumeric() || prev == '/' || prev == '-' || prev == '_' {
                        continue;
                    }
                }
                let rest = &line[start..];
                let end = rest.find('`').unwrap_or(rest.len());
                let candidate = rest[..end].trim_end_matches(['.', ',', ')', ']']).trim();
                // A real INVOCATION, not a prose mention of a command name: it
                // must carry at least one flag and no placeholder syntax. A
                // bare `bee decisions log` names the command; run literally it
                // is exactly the malformed call this guard exists to refuse,
                // so it must NOT be in the must-not-refuse corpus.
                if candidate.contains("--")
                    && !candidate.contains(['<', '>', '[', ']', '{', '}', '\u{2026}'])
                {
                    out.push(candidate.to_string());
                }
            }
        }
        out
    }

    /// A handful of pinned exceptions: fenced lines inside `docs/history/**`
    /// that are transcript BYTES of a command someone actually ran in the
    /// past, not a spelling anyone would copy going forward — that history
    /// is immutable (cited, never reinterpreted), so a later required-flag
    /// addition naturally leaves an old real invocation behind. Each entry
    /// says which cell dated it obsolete, so a fixed extractor or a rewritten
    /// history file makes this row go red and the exception comes out.
    const KNOWN_HISTORICAL_EXCEPTIONS: [&str; 2] = [
        // kdt-3 (knowledge-distill-trigger): `decisions log` gained a
        // required `--relation` after this codex-native-runtime-v2 advisor
        // session ran; the report is a raw shell-transcript line, not a
        // spelling to keep current.
        r#"node .bee/bin/bee.mjs decisions log --decision "auto-approved Gate 3 (bypass): proceed with required validation repairs" --rationale "Advisor verdict was PROCEED-WITH-CHANGES; repairs are bounded and required before implementation."' in /home/thanhsmind/projects/goglbe/beegog"#,
        // bah-2 (backlog-add-honest-refusal): `backlog add` declared
        // `required: []` while its handler always demanded four flags; the
        // declaration was corrected, which makes this walkthrough line
        // refusable. The line is a transcript of a call the walkthrough
        // shows BEING REJECTED ("# rejected, exit 1") — refusing it is the
        // documented behavior, not a stale spelling to repair.
        r#"node .bee/bin/bee_backlog.mjs add --type kind --title x   # rejected, exit 1"#,
    ];

    #[test]
    fn no_shipped_command_spelling_is_refused_by_the_widened_guard() {
        let root = repo_root();
        let mut files = Vec::new();
        for sub in ["skills", "expertise", "docs"] {
            walk(&root.join(sub), &mut files);
        }
        for top in ["AGENTS.md", "README.md", "CLAUDE.md", "LLM.md", "INSTALL.md"] {
            files.push(root.join(top));
        }
        let mut checked = 0usize;
        let mut denied: Vec<String> = Vec::new();
        let mut known_hits = 0usize;
        for f in &files {
            let Ok(text) = std::fs::read_to_string(f) else { continue };
            let mut fenced = false;
            for line in text.lines() {
                if line.trim_start().starts_with("```") {
                    fenced = !fenced;
                    continue;
                }
                if !fenced {
                    continue;
                }
                for command in invocations(line) {
                    checked += 1;
                    if let Some(reason) = check_cli_shape(&command) {
                        if KNOWN_HISTORICAL_EXCEPTIONS.contains(&command.as_str()) {
                            known_hits += 1;
                            continue;
                        }
                        denied.push(format!("{}\n    {command}\n    {reason}", f.display()));
                    }
                }
            }
        }
        assert_eq!(
            known_hits,
            KNOWN_HISTORICAL_EXCEPTIONS.len(),
            "a pinned historical-transcript exception stopped being refused — delete it from \
KNOWN_HISTORICAL_EXCEPTIONS instead of leaving a dead exception"
        );
        // Never vacuous: a corpus this small would mean the extractor broke or
        // the tree is not the repo checkout.
        println!("fenced runnable invocations checked: {checked}");
        assert!(
            checked > 50,
            "expected the shipped instruction layer to carry dozens of fenced bee invocations, \
saw {checked} (is this a repo checkout? root={})",
            root.display()
        );
        assert!(
            denied.is_empty(),
            "the widened CLI-shape guard would refuse {} SHIPPED command spelling(s):\n{}",
            denied.len(),
            denied.join("\n")
        );
    }
}

#[cfg(test)]
mod registry_examples {
    use super::*;
    use serde_json::Value;

    /// The registry's own `examples` are the canonical spellings of every
    /// command — `bee --help --json` publishes them and agents copy them
    /// verbatim. The guard resolves against that SAME registry, so a refusal
    /// here would mean the registry contradicts itself. This is the widening's
    /// most stable fence: it does not depend on prose, and it grows with the
    /// command surface automatically.
    /// ONE known registry defect, pinned rather than papered over: this
    /// example omits `--budget`, which `knowledge.context` declares required
    /// — either the example needs `--budget` or `budget` should not be
    /// required. When the registry is fixed this row goes red and the
    /// exception comes out.
    const KNOWN_SELF_CONTRADICTING_EXAMPLES: [&str; 1] =
        ["bee knowledge context --work okf-foundation --lane standard --json"];

    #[test]
    fn no_registry_example_is_refused_by_the_guard() {
        let payload: Value = serde_json::from_str(crate::registry::REGISTRY_PAYLOAD).unwrap();
        let commands = payload["commands"].as_array().unwrap();
        let mut checked = 0usize;
        let mut denied = Vec::new();
        let mut known_hits = 0usize;
        for entry in commands {
            for example in entry["examples"].as_array().into_iter().flatten() {
                let Some(example) = example.as_str() else { continue };
                checked += 1;
                if let Some(reason) = check_cli_shape(example) {
                    if KNOWN_SELF_CONTRADICTING_EXAMPLES.contains(&example) {
                        known_hits += 1;
                        continue;
                    }
                    denied.push(format!("  {example}\n    {reason}"));
                }
            }
        }
        assert_eq!(
            known_hits,
            KNOWN_SELF_CONTRADICTING_EXAMPLES.len(),
            "a pinned known-defect example stopped being refused — delete it from \
KNOWN_SELF_CONTRADICTING_EXAMPLES instead of leaving a dead exception"
        );
        println!("registry examples checked: {checked}");
        assert!(checked > 100, "expected 100+ registry examples, saw {checked}");
        assert!(
            denied.is_empty(),
            "the CLI-shape guard refuses {} of the registry's OWN examples:\n{}",
            denied.len(),
            denied.join("\n")
        );
    }

    /// The control: the fence above must be able to fail. Strip a required
    /// flag off a real example and the same guard must refuse it — otherwise
    /// the sweep is proving nothing.
    #[test]
    fn the_example_fence_is_not_vacuous() {
        assert!(check_cli_shape("bee cells show --id demo-1").is_none());
        assert!(check_cli_shape("bee cells show").is_some());
    }
}
