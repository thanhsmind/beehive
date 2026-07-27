//! Group/verb dispatch seam (rpl-1, CONTEXT.md D3/D7).
//!
//! `main.rs` dispatches the pre-seam commands (`ping`, `--version`, `hook`,
//! `status`) from a flat single-word `match`. Adding a registry group there
//! would mean rewriting `main` once per group. This module is the layer
//! beside it: `queen-bee <group> <verb> [flags]` resolves through a
//! REGISTRATION TABLE ([`Dispatcher`]), so landing a group is one line in
//! [`crate::groups::register_all`] and nothing else.
//!
//! ## What is ported here, and from where
//!
//! Everything in this module is a port of `packages/bee/bee.mjs`'s `main()`
//! argv path, which is FROZEN (D1) and is therefore the oracle — never
//! "improved" on:
//!
//! | behavior | mjs oracle |
//! |---|---|
//! | leading-token / flag split | `bee.mjs:6754` `splitCommandTokens` |
//! | longest-prefix command resolution | `bee.mjs:6773` `resolveCommand` |
//! | flag parsing + flag-alone booleans | `bee.mjs:6752,6789` |
//! | nearest-command suggestion | `bee.mjs:6823,6839` |
//! | per-group unknown-VERB usage fallback | `bee.mjs:6591-6609` `GROUP_USAGE_FALLBACKS` |
//! | missing/invalid flag refusal | `bee.mjs:7135-7159` (via `validate-args.mjs`) |
//! | unknown-flag refusal | `bee.mjs:7181-7191` |
//! | `--help` / group-scoped `--help` | `bee.mjs:6925-6963,7009-7023` |
//! | stdout/stderr channel split | `bee.mjs:6973` `emit` / `:6985` `emitError` |
//!
//! ## The unknown-flag oracle is NOT `validate_args`
//!
//! `packages/bee/lib/validate-args.mjs:90` disclaims unknown-flag rejection
//! in its own comment (`if (!propSchema) continue; // unknown-flag rejection
//! is the dispatcher's own concern`), and so does its Rust port
//! (`bee-core/src/validate_args.rs:147-151`). The real producer is
//! `bee.mjs:7186-7188`, which builds the message from the REGISTRY entry's
//! `parameters.properties` — which is why this module cannot produce that
//! text at all without a resolvable command registry, and why the parity
//! fixture must carry one (`queen-bench::fixture::write_command_registry`).
//!
//! ## Deliberate, documented divergences from `main()`
//!
//! - **Manifest drift** (`bee.mjs:7053` `checkManifestDrift`) is not ported.
//!   It is a stderr-only advisory derived from the mjs module's own
//!   in-process registry hash, and it fires only when a PRIOR
//!   `.bee/cache/manifest-hash.json` disagrees with the current hash. Every
//!   parity leg runs exactly once against a fresh clone that carries no such
//!   file, so the mjs leg never emits the line either.
//! - **A registry command with no registered Rust handler is REFUSED**, on
//!   stderr, with a queen-bee-specific message — it never falls through to
//!   an mjs-shaped answer. Impersonating mjs for an unported verb is the one
//!   failure mode that would make a parity green meaningless.
//! - **A stale registry snapshot is REFUSED** rather than used. `registry.rs`
//!   exists precisely so a snapshot that no longer matches the live
//!   `command-registry.mjs` is never silently trusted; for a dispatcher
//!   "skip the check" is not available, so the only honest option is to stop.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use bee_core::registry::{load_registry, CommandRegistryDump, CommandRegistryEntry, Registry};
use bee_core::validate_args::{validate, Problem};
use serde_json::{json, Value};

use crate::jsonout;

/// `packages/bee/lib/command-registry.mjs:27` `SCHEMA_VERSION = '1.0'`.
///
/// The dump bridge (`.bee/cache/command-registry.json`) carries the entries
/// and the source hash but not this constant, and `command-registry.mjs` is
/// frozen (D1) so it cannot be taught to export it into the dump from here.
/// The guard against drift is the `--cmd-check` `seam/help-json` scenario:
/// it diffs this binary's whole `--help --json` payload against the live mjs
/// oracle byte-for-byte, so a wrong value here is an immediate red.
pub const SCHEMA_VERSION: &str = "1.0";

/// `bee.mjs:6752` `FLAG_ALONE_BOOLEANS` — flags that parse as a bare boolean
/// instead of consuming the next token as their value. Ported verbatim,
/// order-insensitive (it is a Set on both sides).
pub const FLAG_ALONE_BOOLEANS: &[&str] = &[
    "json",
    "stdin",
    "behavior-change",
    "evidence-stdin",
    "active-only",
    "dry-run",
    "write",
    "as-lane",
    "no-lane",
    "waive-scribing-debt",
    "html",
    "string",
    "cleanup",
    "force-ownership",
    "local",
    "all",
    "untagged",
    "check",
    "with-companion",
    "lanes-full",
    "strict",
    "queue-submit",
    "show",
    "isolate",
];

/// `bee.mjs:7181` `DISPATCH_ALWAYS_ALLOWED_FLAGS`.
const DISPATCH_ALWAYS_ALLOWED_FLAGS: &[&str] = &["json", "help"];

// ─── registration table ────────────────────────────────────────────────────

/// One registered verb. `handler` receives the fully parsed, already
/// validated call — every refusal above it has already been ruled out.
pub type HandlerFn = fn(&Call) -> ExitCode;

/// The per-group unknown-VERB usage fallback (`bee.mjs:6591-6609`). Receives
/// the FULL leading tokens (not just the extras) so a fallback can
/// reconstruct the attempted verb / sub-action exactly as mjs does.
pub type UsageFallbackFn = fn(&[String]) -> String;

pub struct VerbDef {
    pub verb: &'static str,
    pub handler: HandlerFn,
}

pub struct GroupDef {
    pub group: &'static str,
    /// The group's byte-exact legacy `Unknown command "…". Use: …` line.
    /// `None` means the group has no legacy fallback in mjs either, and an
    /// unknown verb falls through to the nearest-match path.
    pub usage_fallback: Option<UsageFallbackFn>,
    pub verbs: Vec<VerbDef>,
}

/// The registration table. A group cell adds ONE `register` call in
/// [`crate::groups::register_all`]; nothing in `main.rs` or in this file
/// changes.
#[derive(Default)]
pub struct Dispatcher {
    groups: Vec<GroupDef>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Refuses a duplicate group name and a group that registers the same
    /// verb twice — a silently shadowed handler is exactly the kind of hole
    /// that keeps every downstream parity suite green while one verb is
    /// quietly unreachable.
    pub fn register(&mut self, group: GroupDef) -> Result<(), String> {
        if self.groups.iter().any(|g| g.group == group.group) {
            return Err(format!("dispatch: group `{}` is already registered", group.group));
        }
        for (i, v) in group.verbs.iter().enumerate() {
            if group.verbs[..i].iter().any(|prev| prev.verb == v.verb) {
                return Err(format!(
                    "dispatch: group `{}` registers verb `{}` twice",
                    group.group, v.verb
                ));
            }
        }
        self.groups.push(group);
        Ok(())
    }

    /// Every registered group name, in registration order.
    pub fn group_names(&self) -> Vec<&'static str> {
        self.groups.iter().map(|g| g.group).collect()
    }

    /// Every registered verb of `group`, in registration order.
    pub fn verb_names(&self, group: &str) -> Option<Vec<&'static str>> {
        self.find_group(group).map(|g| g.verbs.iter().map(|v| v.verb).collect())
    }

    /// Every registered command as its dotted registry name (`group.verb`).
    pub fn command_names(&self) -> Vec<String> {
        self.groups
            .iter()
            .flat_map(|g| g.verbs.iter().map(move |v| format!("{}.{}", g.group, v.verb)))
            .collect()
    }

    fn find_group(&self, group: &str) -> Option<&GroupDef> {
        self.groups.iter().find(|g| g.group == group)
    }

    pub fn handler(&self, group: &str, verb: &str) -> Option<HandlerFn> {
        self.find_group(group)?
            .verbs
            .iter()
            .find(|v| v.verb == verb)
            .map(|v| v.handler)
    }

    pub fn usage_fallback(&self, group: &str) -> Option<UsageFallbackFn> {
        self.find_group(group).and_then(|g| g.usage_fallback)
    }
}

/// A fully resolved, parsed and validated call handed to a verb handler.
pub struct Call {
    /// The dotted registry name, e.g. `"intent.show"`.
    pub command_name: String,
    /// The resolved store root (`resolveRoots(cwd).store_root`).
    pub root: PathBuf,
    /// Parsed flags in argv order. `--json` is siphoned into [`Call::json`],
    /// exactly as `parseFlags` does.
    pub flags: Vec<(String, Value)>,
    pub json: bool,
    /// The registry entry this call resolved to.
    pub entry: CommandRegistryEntry,
}

impl Call {
    pub fn flag(&self, name: &str) -> Option<&Value> {
        self.flags.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }

    pub fn flag_str(&self, name: &str) -> Option<&str> {
        self.flag(name).and_then(Value::as_str)
    }
}

// ─── argv parsing (ports of bee.mjs's own exported helpers) ────────────────

/// `bee.mjs:6754` `splitCommandTokens`.
pub fn split_command_tokens(argv: &[String]) -> (Vec<String>, Vec<String>) {
    let mut i = 0usize;
    while i < argv.len() && !argv[i].starts_with("--") {
        i += 1;
    }
    (argv[..i].to_vec(), argv[i..].to_vec())
}

/// `bee.mjs:6773` `resolveCommand` — longest-prefix match over the registry
/// names, with the legacy shaping fallback (bare token for one leading
/// token, `<group>.<verb>` for two or more).
pub fn resolve_command(leading: &[String], names: &[String]) -> (Option<String>, Vec<String>) {
    if leading.is_empty() {
        return (None, Vec::new());
    }
    for n in (1..=leading.len()).rev() {
        let candidate = leading[..n].join(".");
        if names.iter().any(|name| *name == candidate) {
            return (Some(candidate), leading[n..].to_vec());
        }
    }
    if leading.len() == 1 {
        return (Some(leading[0].clone()), Vec::new());
    }
    (Some(format!("{}.{}", leading[0], leading[1])), leading[2..].to_vec())
}

/// The `{field, reason}` shape `parseFlags` returns on refusal.
#[derive(Debug, Clone, PartialEq)]
pub struct FlagParseError {
    pub field: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParsedFlags {
    pub flags: Vec<(String, Value)>,
    pub json: bool,
}

/// `bee.mjs:6789` `parseFlags`. Never panics; returns the mjs error shape.
pub fn parse_flags(tokens: &[String]) -> Result<ParsedFlags, FlagParseError> {
    let mut out = ParsedFlags::default();
    let mut i = 0usize;
    while i < tokens.len() {
        let tok = &tokens[i];
        if !tok.starts_with("--") {
            return Err(FlagParseError { field: None, reason: format!("unexpected argument \"{tok}\"") });
        }
        let (name, inline_value) = match tok.find('=') {
            Some(eq) => (tok[2..eq].to_string(), Some(tok[eq + 1..].to_string())),
            None => (tok[2..].to_string(), None),
        };
        let value: Value = if let Some(v) = inline_value {
            Value::String(v)
        } else if FLAG_ALONE_BOOLEANS.contains(&name.as_str()) {
            Value::Bool(true)
        } else {
            match tokens.get(i + 1) {
                Some(v) => {
                    i += 1;
                    Value::String(v.clone())
                }
                None => {
                    return Err(FlagParseError {
                        field: Some(name.clone()),
                        reason: format!("flag --{name} requires a value"),
                    })
                }
            }
        };
        i += 1;
        if name == "json" {
            out.json = true;
            continue;
        }
        // mjs assigns into a plain object, so a repeated flag overwrites in
        // place and keeps its ORIGINAL key position.
        match out.flags.iter_mut().find(|(k, _)| *k == name) {
            Some(slot) => slot.1 = value,
            None => out.flags.push((name, value)),
        }
    }
    Ok(out)
}

/// `bee.mjs:6823` `levenshtein` — plain edit distance over chars.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut cur = vec![0usize; n + 1];
    for i in 1..=m {
        cur[0] = i;
        for j in 1..=n {
            cur[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1]
            } else {
                1 + prev[j - 1].min(prev[j]).min(cur[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[n]
}

/// `bee.mjs:6839` `nearestCommandName` — first strictly-closest name wins
/// (`dist < bestDist`), so registry order is the tiebreak, exactly as in mjs.
pub fn nearest_command_name(name: &str, names: &[String]) -> Option<String> {
    let mut best: Option<(&String, usize)> = None;
    for candidate in names {
        let dist = levenshtein(name, candidate);
        if best.map(|(_, d)| dist < d).unwrap_or(true) {
            best = Some((candidate, dist));
        }
    }
    best.map(|(c, _)| c.clone())
}

// ─── emission (bee.mjs:6973 emit / :6985 emitError) ────────────────────────

/// `emit({result, text, exitCode}, useJson)`: `--json` puts the RESULT on
/// stdout; the text path puts `text` on stdout. Drift is never emitted here
/// (see the module doc).
/// `pub(crate)` since rpl-2: a registered group's handler is the one place
/// outside this module that has to emit a response, and it MUST reuse this
/// exact function rather than growing its own. Re-implementing it at a group
/// call site would mean a bare `serde_json::to_string_pretty` escaping
/// [`jsonout`] — precisely the JS-key-order break rpl-1 measured and closed.
pub(crate) fn emit(result: &Value, text: &str, exit_code: u8, use_json: bool) -> ExitCode {
    if use_json {
        print!("{}\n", jsonout::stringify_pretty(result));
    } else {
        println!("{text}");
    }
    ExitCode::from(exit_code)
}

/// `emitError(message, useJson)`: `--json` puts `{"error": …}` on stdout as
/// COMPACT JSON; the text path writes to STDERR. Exit 1 either way.
pub(crate) fn emit_error(message: &str, use_json: bool) -> ExitCode {
    if use_json {
        println!("{}", jsonout::stringify_compact(&json!({ "error": message })));
    } else {
        eprintln!("{message}");
    }
    ExitCode::from(1)
}

// ─── help rendering (bee.mjs:6925-6963) ────────────────────────────────────

fn manifest_entries(entries: &[CommandRegistryEntry]) -> Value {
    // `toManifestEntries` picks exactly {name, invoke, description,
    // parameters, examples, deprecated} in that order — which is this
    // struct's own field order, so a plain serialize reproduces it.
    Value::Array(entries.iter().map(|e| serde_json::to_value(e).unwrap_or(Value::Null)).collect())
}

/// `renderHelpText(entries)`.
fn render_help_text(entries: &[CommandRegistryEntry]) -> String {
    let mut lines: Vec<String> = vec![
        format!("bee — unified CLI dispatcher (schema_version {SCHEMA_VERSION})"),
        String::new(),
    ];
    for entry in entries {
        lines.push(entry.invoke.clone());
        lines.push(format!("    {}", entry.description));
        let required: Vec<String> = entry
            .parameters
            .get("required")
            .and_then(Value::as_array)
            .map(|r| r.iter().filter_map(Value::as_str).map(|s| format!("--{s}")).collect())
            .unwrap_or_default();
        if !required.is_empty() {
            lines.push(format!("    required: {}", required.join(", ")));
        }
        if let Some(dep) = entry.deprecated.as_object() {
            lines.push(format!(
                "    DEPRECATED since {} — use \"{}\" instead.",
                dep.get("since").and_then(Value::as_str).unwrap_or("undefined"),
                dep.get("use_instead").and_then(Value::as_str).unwrap_or("undefined"),
            ));
        }
        lines.push(String::new());
    }
    format!("{}\n", lines.join("\n").trim_end())
}

fn render_help(entries: &[CommandRegistryEntry], json: bool) -> ExitCode {
    if json {
        let manifest = json!({ "schema_version": SCHEMA_VERSION, "commands": manifest_entries(entries) });
        print!("{}\n", jsonout::stringify_pretty(&manifest));
    } else {
        print!("{}", render_help_text(entries));
    }
    ExitCode::SUCCESS
}

// ─── the dispatch entry point ──────────────────────────────────────────────

/// Outcome of resolving the registry bridge for this process.
enum RegistrySource {
    Loaded { root: PathBuf, dump: CommandRegistryDump },
    Refused(String),
}

fn load_registry_for(cwd: &Path) -> RegistrySource {
    let Some(root) = crate::adapter::resolve_roots(cwd).store_root else {
        return RegistrySource::Refused(
            "No bee repo root found (no .bee/onboarding.json or .git up the tree). Run bee-hive onboarding."
                .to_string(),
        );
    };
    let dump_path = root.join(".bee").join("cache").join("command-registry.json");
    let source_path = root.join(".bee").join("bin").join("lib").join("command-registry.mjs");
    match load_registry(&dump_path, &source_path) {
        Ok(Registry::Fresh(dump)) => RegistrySource::Loaded { root, dump },
        Ok(Registry::Stale { embedded_sha256, live_sha256, .. }) => RegistrySource::Refused(format!(
            "queen-bee: command registry snapshot {} (source_sha256 {embedded_sha256}) no longer matches {} ({live_sha256}) — refusing to dispatch against a stale registry; regenerate with `node scripts/dump_command_registry.mjs`",
            dump_path.display(),
            source_path.display(),
        )),
        Err(err) => RegistrySource::Refused(format!(
            "queen-bee: command registry bridge unavailable at {}: {err}",
            dump_path.display()
        )),
    }
}

/// Run one `queen-bee <group> <verb> [flags]` invocation through the seam.
///
/// Returns the process exit code. `main.rs` routes everything its own flat
/// match did not claim here, so this is also the binary's usage path.
pub fn run(argv: &[String]) -> ExitCode {
    let dispatcher = crate::groups::dispatcher();
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(err) => return emit_error(&format!("queen-bee: could not resolve the current directory: {err}"), false),
    };
    let (root, dump) = match load_registry_for(&cwd) {
        RegistrySource::Loaded { root, dump } => (root, dump),
        RegistrySource::Refused(message) => return emit_error(&message, false),
    };
    run_with(argv, &dispatcher, &root, &dump)
}

/// The testable core of [`run`]: everything resolved, nothing ambient.
pub fn run_with(
    argv: &[String],
    dispatcher: &Dispatcher,
    root: &Path,
    dump: &CommandRegistryDump,
) -> ExitCode {
    let names: Vec<String> = dump.entries.iter().map(|e| e.name.clone()).collect();

    // 1. Top-level `--help` (bee.mjs:6994).
    if argv.first().map(String::as_str) == Some("--help") {
        return render_help(&dump.entries, argv.iter().any(|t| t == "--json"));
    }

    let (leading, rest) = split_command_tokens(argv);
    let (command_name, extra) = resolve_command(&leading, &names);
    let json_requested = rest.iter().any(|t| t == "--json" || t.starts_with("--json="));

    // 2. Group/command-scoped `--help` (bee.mjs:7009).
    if let Some(name) = &command_name {
        if rest.iter().any(|t| t == "--help") {
            let filtered: Vec<CommandRegistryEntry> = dump
                .entries
                .iter()
                .filter(|e| e.name == *name || e.name.starts_with(&format!("{name}.")))
                .cloned()
                .collect();
            if !filtered.is_empty() {
                return render_help(&filtered, json_requested);
            }
        }
    }

    // 3. No command at all (bee.mjs:7025).
    let Some(command_name) = command_name else {
        return emit(
            &json!({"ok": false, "error": {"field": null, "reason": "no command given", "command": null}}),
            "No command given. Try \"bee --help\".",
            1,
            json_requested,
        );
    };

    let entry = dump.find(&command_name).cloned();

    // 4. No registry entry: group usage fallback, else nearest match
    //    (bee.mjs:7056-7082).
    let Some(entry) = entry else {
        let group = command_name.split('.').next().unwrap_or(&command_name).to_string();
        if let Some(fallback) = dispatcher.usage_fallback(&group) {
            return emit_error(&fallback(&leading), json_requested);
        }
        // A group the REGISTRY knows but this binary has not registered is
        // NOT ours to answer for: mjs would either serve that group's legacy
        // fallback line or a nearest match, and guessing which would make a
        // parity green meaningless. Refuse visibly instead.
        if names.iter().any(|n| n.starts_with(&format!("{group}."))) {
            return emit_error(
                &format!(
                    "queen-bee: `{group}` is a known bee command group but is not ported into this binary yet — run it through `node .bee/bin/bee.mjs {group} …`"
                ),
                false,
            );
        }
        let suggestion = nearest_command_name(&command_name, &names);
        let suggestion_json = suggestion.clone().map(Value::String).unwrap_or(Value::Null);
        return emit(
            &json!({
                "ok": false,
                "error": {"field": null, "reason": format!("unknown command \"{command_name}\""), "command": null},
                "suggestion": suggestion_json,
            }),
            &format!(
                "Unknown command \"{command_name}\". Did you mean \"{}\"?",
                suggestion.unwrap_or_default()
            ),
            1,
            json_requested,
        );
    };

    // 5. Stray leading tokens after a resolved entry (bee.mjs:7088).
    if let Some(first_extra) = extra.first() {
        return emit(
            &json!({
                "ok": false,
                "error": {
                    "field": null,
                    "reason": format!("unexpected argument \"{first_extra}\""),
                    "command": command_name,
                },
            }),
            &format!("Unexpected argument \"{first_extra}\" after \"{command_name}\"."),
            1,
            json_requested,
        );
    }

    // 6. Deprecated redirect (bee.mjs:7103). No entry is deprecated today;
    //    the dispatch logic exists regardless, same as in mjs.
    if let Some(dep) = entry.deprecated.as_object() {
        let since = dep.get("since").cloned().unwrap_or(Value::Null);
        let use_instead = dep.get("use_instead").cloned().unwrap_or(Value::Null);
        let since_txt = since.as_str().map(|s| format!(" since {s}")).unwrap_or_default();
        let use_txt = use_instead.as_str().unwrap_or("null");
        return emit(
            &json!({
                "ok": false,
                "deprecated": true,
                "since": since,
                "use_instead": use_instead,
                "message": format!("\"{}\" is deprecated{since_txt}; use \"{use_txt}\" instead.", entry.name),
            }),
            &format!("\"{}\" is deprecated{since_txt} — use \"{use_txt}\" instead.", entry.name),
            1,
            json_requested,
        );
    }

    // 7. Flag parsing (bee.mjs:7106).
    let parsed = match parse_flags(&rest) {
        Ok(p) => p,
        Err(err) => {
            let field_txt = err.field.as_ref().map(|f| format!(" (--{f})")).unwrap_or_default();
            return emit(
                &json!({
                    "ok": false,
                    "error": {
                        "field": err.field.clone().map(Value::String).unwrap_or(Value::Null),
                        "reason": err.reason,
                        "command": command_name,
                    },
                }),
                &format!("Invalid call to \"{command_name}\": {}{field_txt}.", err.reason),
                1,
                json_requested,
            );
        }
    };
    // After a successful parse the authoritative signal is `parsed.json`,
    // NOT the pre-parse scan — a non-boolean flag can consume the `--json`
    // token as its value (bee.mjs:7121-7127).
    let use_json = parsed.json;

    // 8. Schema validation (bee.mjs:7135).
    let problems = validate(&entry.parameters, &parsed.flags);
    if let Some(first) = problems.first() {
        let detail = problems
            .iter()
            .map(|p| {
                let field = p.field.as_ref().map(|f| format!(" (--{f})")).unwrap_or_default();
                format!("{}{field}", p.reason)
            })
            .collect::<Vec<_>>()
            .join("; ");
        let example_line = entry
            .examples
            .first()
            .map(|e| format!(" Example: {e}"))
            .unwrap_or_default();
        return emit(
            &json!({
                "ok": false,
                "error": problem_json(first, Some(&command_name)),
                "problems": problems.iter().map(|p| problem_json(p, None)).collect::<Vec<_>>(),
            }),
            &format!("Invalid call to \"{command_name}\": {detail}.{example_line}"),
            1,
            use_json,
        );
    }

    // 9. Dispatcher-level unknown-flag rejection (bee.mjs:7181-7191). Built
    //    from the REGISTRY entry's own `parameters.properties`, plus the two
    //    always-allowed flags, de-duplicated and sorted.
    let known_flag_names: Vec<String> = entry
        .parameters
        .get("properties")
        .and_then(Value::as_object)
        .map(|p| p.keys().cloned().collect())
        .unwrap_or_default();
    for (flag_name, _) in &parsed.flags {
        if known_flag_names.contains(flag_name) || DISPATCH_ALWAYS_ALLOWED_FLAGS.contains(&flag_name.as_str()) {
            continue;
        }
        let mut known: Vec<String> = known_flag_names.clone();
        for extra in DISPATCH_ALWAYS_ALLOWED_FLAGS {
            if !known.iter().any(|k| k == extra) {
                known.push((*extra).to_string());
            }
        }
        // JS `Array.prototype.sort()` with no comparator sorts by UTF-16
        // code unit; every flag name in the registry is ASCII, where Rust's
        // byte ordering is the same ordering.
        known.sort();
        return emit_error(
            &format!(
                "{}: unknown flag --{flag_name} (known: {}).",
                command_name.split('.').collect::<Vec<_>>().join(" "),
                known.join(", ")
            ),
            use_json,
        );
    }

    // 10. Handler dispatch — or an honest refusal.
    let mut parts = command_name.splitn(2, '.');
    let group = parts.next().unwrap_or_default();
    let verb = parts.next().unwrap_or_default();
    match dispatcher.handler(group, verb) {
        Some(handler) => handler(&Call {
            command_name,
            root: root.to_path_buf(),
            flags: parsed.flags,
            json: parsed.json,
            entry,
        }),
        None => emit_error(
            &format!(
                "queen-bee: `{command_name}` is a known bee command but is not ported into this binary yet — run it through `node .bee/bin/bee.mjs {}`",
                command_name.split('.').collect::<Vec<_>>().join(" ")
            ),
            false,
        ),
    }
}

fn problem_json(p: &Problem, command: Option<&str>) -> Value {
    let field = p.field.clone().map(Value::String).unwrap_or(Value::Null);
    match command {
        // validate-args.mjs's `error` shape is {field, reason, command}.
        Some(c) => json!({ "field": field, "reason": p.reason, "command": c }),
        None => json!({ "field": field, "reason": p.reason }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn toks(v: &[&str]) -> Vec<String> {
        names(v)
    }

    #[test]
    fn split_stops_at_the_first_double_dash_token() {
        let (leading, rest) = split_command_tokens(&toks(&["cells", "show", "--id", "x"]));
        assert_eq!(leading, toks(&["cells", "show"]));
        assert_eq!(rest, toks(&["--id", "x"]));
    }

    #[test]
    fn resolve_prefers_the_longest_registry_prefix() {
        let reg = names(&["state.worker.add", "state.worker", "cells.ready", "status"]);
        assert_eq!(
            resolve_command(&toks(&["state", "worker", "add"]), &reg),
            (Some("state.worker.add".to_string()), Vec::new())
        );
        assert_eq!(
            resolve_command(&toks(&["cells", "ready", "foo"]), &reg),
            (Some("cells.ready".to_string()), toks(&["foo"]))
        );
        // Legacy shaping when nothing matches.
        assert_eq!(
            resolve_command(&toks(&["zzz", "list"]), &reg),
            (Some("zzz.list".to_string()), Vec::new())
        );
        assert_eq!(resolve_command(&[], &reg), (None, Vec::new()));
    }

    #[test]
    fn flag_alone_booleans_do_not_eat_the_next_token() {
        let p = parse_flags(&toks(&["--dry-run", "--id", "x"])).unwrap();
        assert_eq!(p.flags, vec![("dry-run".into(), Value::Bool(true)), ("id".into(), Value::String("x".into()))]);
        assert!(!p.json);
    }

    #[test]
    fn json_is_siphoned_out_of_the_flag_map() {
        let p = parse_flags(&toks(&["--id", "x", "--json"])).unwrap();
        assert!(p.json);
        assert_eq!(p.flags, vec![("id".into(), Value::String("x".into()))]);
    }

    #[test]
    fn a_non_boolean_flag_can_swallow_the_json_token() {
        // bee.mjs:7121-7127's exact hazard: `--lane` is not flag-alone, so
        // it consumes `--json` as its VALUE and --json was never requested.
        let p = parse_flags(&toks(&["--lane", "--json"])).unwrap();
        assert!(!p.json);
        assert_eq!(p.flags, vec![("lane".into(), Value::String("--json".into()))]);
    }

    #[test]
    fn a_value_less_trailing_flag_is_refused_with_the_mjs_shape() {
        let err = parse_flags(&toks(&["--id"])).unwrap_err();
        assert_eq!(err.field.as_deref(), Some("id"));
        assert_eq!(err.reason, "flag --id requires a value");
    }

    #[test]
    fn inline_equals_values_parse() {
        let p = parse_flags(&toks(&["--id=abc"])).unwrap();
        assert_eq!(p.flags, vec![("id".into(), Value::String("abc".into()))]);
    }

    #[test]
    fn nearest_match_keeps_registry_order_as_the_tiebreak() {
        let reg = names(&["reservations.list", "cells.list"]);
        // Both are the same distance from "zzz.list"? No — but the rule is
        // `dist < best`, so the FIRST strictly-closest name wins.
        let got = nearest_command_name("reservations.lst", &reg).unwrap();
        assert_eq!(got, "reservations.list");
    }

    #[test]
    fn dispatcher_refuses_a_duplicate_group() {
        fn h(_: &Call) -> ExitCode {
            ExitCode::SUCCESS
        }
        let mut d = Dispatcher::new();
        d.register(GroupDef { group: "demo", usage_fallback: None, verbs: vec![VerbDef { verb: "a", handler: h }] })
            .unwrap();
        let err = d
            .register(GroupDef { group: "demo", usage_fallback: None, verbs: vec![VerbDef { verb: "b", handler: h }] })
            .unwrap_err();
        assert!(err.contains("already registered"), "{err}");
    }

    #[test]
    fn dispatcher_refuses_a_duplicate_verb_within_one_group() {
        fn h(_: &Call) -> ExitCode {
            ExitCode::SUCCESS
        }
        let mut d = Dispatcher::new();
        let err = d
            .register(GroupDef {
                group: "demo",
                usage_fallback: None,
                verbs: vec![VerbDef { verb: "a", handler: h }, VerbDef { verb: "a", handler: h }],
            })
            .unwrap_err();
        assert!(err.contains("registers verb `a` twice"), "{err}");
    }

    // ─── the registration surface, proven by execution ────────────────────
    //
    // must_have: "the group registration surface is proven by a TEST, not a
    // comment: a test registers a throwaway group and asserts it is
    // dispatchable and enumerable with no edit to main.rs". Nothing below
    // touches `main.rs`, `groups.rs`, or the real registry — a throwaway
    // group is registered into a local `Dispatcher` and driven end to end
    // through the SAME `run_with` the binary uses.

    use std::sync::atomic::{AtomicUsize, Ordering};

    static THROWAWAY_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn throwaway_handler(call: &Call) -> ExitCode {
        // Proves the handler receives the fully parsed call, not raw argv.
        assert_eq!(call.command_name, "throwaway.echo");
        assert_eq!(call.flag_str("id"), Some("abc"));
        THROWAWAY_CALLS.fetch_add(1, Ordering::SeqCst);
        ExitCode::SUCCESS
    }

    fn throwaway_usage_fallback(leading: &[String]) -> String {
        let verb = leading.get(1).cloned().unwrap_or_else(|| "(missing)".to_string());
        format!("Unknown command \"{verb}\". Use: echo.")
    }

    fn throwaway_group() -> GroupDef {
        GroupDef {
            group: "throwaway",
            usage_fallback: Some(throwaway_usage_fallback),
            verbs: vec![VerbDef { verb: "echo", handler: throwaway_handler }],
        }
    }

    /// A synthetic registry carrying exactly the throwaway command, in the
    /// real `CommandRegistryDump` shape.
    fn throwaway_dump() -> CommandRegistryDump {
        let entry: CommandRegistryEntry = serde_json::from_value(json!({
            "name": "throwaway.echo",
            "invoke": "bee throwaway echo",
            "description": "throwaway verb registered by a test",
            "parameters": {
                "type": "object",
                "properties": { "id": { "type": "string" } },
                "required": ["id"]
            },
            "examples": ["bee throwaway echo --id abc"],
            "deprecated": null
        }))
        .unwrap();
        CommandRegistryDump {
            source_sha256: "test".to_string(),
            generated_at: "2026-07-27T00:00:00.000Z".to_string(),
            entries: vec![entry],
        }
    }

    #[test]
    fn a_throwaway_group_is_dispatchable_and_enumerable_without_touching_main() {
        let mut d = Dispatcher::new();
        d.register(throwaway_group()).expect("throwaway group registers");

        // ENUMERABLE: the table reports the group, its verbs, and the dotted
        // command name — this is what a `--cmd-check --group` count reads.
        assert_eq!(d.group_names(), vec!["throwaway"]);
        assert_eq!(d.verb_names("throwaway"), Some(vec!["echo"]));
        assert_eq!(d.command_names(), vec!["throwaway.echo".to_string()]);
        assert!(d.verb_names("nope").is_none());

        // DISPATCHABLE: the real `run_with` path reaches the handler.
        let before = THROWAWAY_CALLS.load(Ordering::SeqCst);
        let dump = throwaway_dump();
        let code = run_with(&toks(&["throwaway", "echo", "--id", "abc"]), &d, Path::new("/nonexistent"), &dump);
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS));
        assert_eq!(THROWAWAY_CALLS.load(Ordering::SeqCst), before + 1);
    }

    #[test]
    fn an_unregistered_verb_of_a_registered_group_serves_that_groups_usage_fallback() {
        let mut d = Dispatcher::new();
        d.register(throwaway_group()).unwrap();
        let dump = throwaway_dump();
        // `throwaway.frobnicate` is not a registry entry, so the group
        // fallback mechanism (bee.mjs:6591-6609) is what answers.
        let code = run_with(&toks(&["throwaway", "frobnicate"]), &d, Path::new("/nonexistent"), &dump);
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(1)));
        assert_eq!(
            throwaway_usage_fallback(&toks(&["throwaway", "frobnicate"])),
            "Unknown command \"frobnicate\". Use: echo."
        );
        assert_eq!(
            throwaway_usage_fallback(&toks(&["throwaway"])),
            "Unknown command \"(missing)\". Use: echo."
        );
    }

    /// rpl-1 shipped the seam with NO group registered, and pinned that fact
    /// so every later cell has to update this expectation deliberately rather
    /// than drift into it. It is now cashed one group at a time: `intent`
    /// (rpl-2), `capture` (rpl-3) and `decisions` (rpl-4).
    ///
    /// Kept as an EXACT set, not a `contains`: the point of the pin is that a
    /// group cannot appear in the shipped binary without someone saying so
    /// here, and a `contains` check would let an unintended registration
    /// through.
    ///
    /// rpl-4 note: `decisions` appears here having registered only its three
    /// WRITE verbs. Group registration and verb coverage are separate facts —
    /// `decisions active` still takes the unported-command refusal until
    /// rpl-5 lands the read side, which the `decisions` group's own test
    /// module pins from the other side.
    #[test]
    fn the_shipped_table_registers_exactly_the_ported_ledger_groups() {
        let mut names = crate::groups::dispatcher().group_names();
        names.sort();
        assert_eq!(names, vec!["capture", "decisions", "intent"]);
    }
}
