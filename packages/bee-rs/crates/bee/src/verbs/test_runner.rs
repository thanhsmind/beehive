// bee test — Rust port of bee.mjs handleTest (~7745) +
// renderTestCommandLines (~7601), backed by lib/test-runner.mjs
// (declaredTestCommands / runDeclaredTests / firstFailureLine /
// spawnDeclaredCommand / TEST_RESULTS_RELATIVE) and lib/state.mjs
// normalizeCommands (~211). Spec: docs/specs/test-simple.md.
//
// Mirrors bee.mjs's dispatch end to end for the accepted shapes:
//   root resolution -> manifest-drift check (cache write) -> handleTest
//   (runDeclaredTests: run each declared command through a POSIX sh, capture
//   combined output, write .bee/logs/test-results.json) -> emit (drift
//   stderr line, stdout payload) -> timing ("[bee] test Nms").
//
// Verified semantics (bee.mjs + lib/test-runner.mjs):
//   - Declaration: merged .bee/config.json + config.local.json ->
//     commands.test; normalizeCommands keeps a trimmed non-empty string, or
//     (test only) a trimmed non-empty-string array; declaredTestCommands then
//     drops the "none" sentinel entries; empty -> undeclared.
//   - Runner: every command runs sequentially (never short-circuits), cwd =
//     root, via `bash -c <command>` on win32 (probed once — Git Bash) and the
//     platform POSIX sh otherwise. Output is stdout-then-stderr concatenated;
//     failure_excerpt is the last ≤500 UTF-16 units of the JS-trimmed output
//     (null on pass), or "(no output; exit <exit>)" when empty; a signal
//     death records exit null.
//   - Record: {ran_at, green, commands:[{command, exit, duration_ms,
//     failure_excerpt}]} written via writeJsonAtomic (pretty + "\n").
//   - Green: text = ✓ lines + "next: green (record: ...)" line, exit 0.
//     Red: ✗ lines carry ", exit N" (or "spawn-failed"), "next: <first
//     failure line> — fix before capping", exit 1 (record still written).
//     Undeclared: 4 fixed lines, {green:null, undeclared:true}, exit 0.
//
// Strangler routing: only `bee test` and `bee test --json` (bare --json
// tokens) are served natively. Everything else — other flags, --json=x
// forms, positionals, non-unicode argv, linked-worktree roots, corrupt
// config/drift-cache JSON (Node's warn-with-V8-message paths), and win32
// hosts without a POSIX shell (Node's cmd.exe fallback) — returns None
// before ANY output and before the drift-cache write, so the Node re-run
// owns the whole command including the manifest_changed line.
//
// Known divergences (documented, unreachable in practice): a spawn error
// AFTER the successful shell probe embeds Rust's io error text where Node
// embeds "spawnSync ... ENOENT"; Node's 64 MiB maxBuffer kill and its
// 10-second probe timeout are not replicated; a results-file write failure
// after the run reports Rust's io message where Node reports the V8 one
// (the .bee/logs dir is pre-flighted before the drift write to keep that
// path all but impossible).

use crate::fsutil::{ensure_dir, write_json_atomic};
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, record_timing};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

/// TEST_RESULTS_RELATIVE (lib/test-runner.mjs:30).
const TEST_RESULTS_RELATIVE: &str = ".bee/logs/test-results.json";
/// FAILURE_EXCERPT_MAX_CHARS (lib/test-runner.mjs:35) — UTF-16 units, like
/// JS String.prototype.slice.
const FAILURE_EXCERPT_MAX: usize = 500;

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "test" {
        return None;
    }
    let mut use_json = false;
    for a in &args[1..] {
        match a.to_str()? {
            "--json" => use_json = true,
            _ => return None, // unknown flags/positionals/--json=x — Node's error paths
        }
    }
    run(use_json, t0)
}

fn run(use_json: bool, t0: Instant) -> Option<ExitCode> {
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::NeedsNode => return None,
        Roots::None => return Some(emit_no_root_error(&cwd, "test", use_json, t0)),
    };

    // Everything that can still delegate happens BEFORE the drift check: its
    // cache write would otherwise swallow the Node re-run's drift line.
    let commands = declared_test_commands(&root).ok()?;
    let shell = if commands.is_some() {
        let s = posix_shell()?; // no POSIX sh — Node's cmd.exe fallback owns it
        // Pre-flight the record dir so the post-run writeJsonAtomic (whose
        // failure text is Node-owned) can only fail on a hard race.
        ensure_dir(&root.join(".bee").join("logs")).ok()?;
        Some(s)
    } else {
        None
    };

    let drift = check_manifest_drift(&root).ok()?;

    // ── handleTest ─────────────────────────────────────────────────────────
    let (result, text, exit_code) = match commands {
        None => {
            // runDeclaredTests undeclared: nothing runs, nothing is written.
            let mut result = Map::new();
            result.insert("green".into(), Value::Null);
            result.insert("undeclared".into(), Value::Bool(true));
            let text = [
                "No commands.test declared — nothing ran.",
                "bee test runs the project's ONE declared test path: set commands.test in .bee/config.json (a string, or an array run in order). Point it at the fast suite — the full chain stays on commands.verify (worktree merge / CI).",
                "Once declared, it becomes the cap door: bee cells finish runs it before every cap, and bee close runs it as the tests door.",
                "next: declare commands.test, then re-run bee test",
            ]
            .join("\n");
            (result, text, 0u8)
        }
        Some(commands) => {
            let run = run_declared_tests(&root, &commands, shell?);
            if let Some(msg) = &run.write_error {
                // Node: writeJsonAtomic throws -> main's catch -> emitError.
                // (Rust io text, not V8's — see the divergence note above.)
                let code = emit_error(msg, use_json);
                record_timing(&root, "test", t0, false);
                return Some(code);
            }
            let mut lines = render_test_command_lines(&run);
            let green = run.green;
            let mut result = Map::new();
            result.insert("green".into(), Value::Bool(green));
            result.insert("undeclared".into(), Value::Bool(false));
            result.insert("ran_at".into(), Value::from(run.ran_at.clone()));
            result.insert(
                "commands".into(),
                Value::Array(run.commands.iter().map(command_result_value).collect()),
            );
            result.insert("results".into(), Value::from(TEST_RESULTS_RELATIVE));
            if green {
                lines.push(format!(
                    "next: green (record: {TEST_RESULTS_RELATIVE}) — back to what you were doing"
                ));
                (result, lines.join("\n"), 0u8)
            } else {
                let first = first_failure_line(&run)
                    .unwrap_or_else(|| "see the failure excerpt in the record".to_string());
                lines.push(format!("next: {first} — fix before capping"));
                (result, lines.join("\n"), 1u8)
            }
        }
    };

    // ── emit (bee.mjs ~8373) ───────────────────────────────────────────────
    if drift.manifest_changed {
        eprintln!("manifest_changed: true — {}", drift.hint);
    }
    if use_json {
        print!("{}\n", jsjson::stringify_pretty(&Value::Object(result)));
    } else {
        print!("{text}\n");
    }
    record_timing(&root, "test", t0, exit_code == 0);
    Some(ExitCode::from(exit_code))
}

/// emitError (bee.mjs ~8385): JSON -> stdout {"error": msg} compact; text ->
/// stderr line. Exit 1.
fn emit_error(message: &str, use_json: bool) -> ExitCode {
    if use_json {
        let mut m = Map::new();
        m.insert("error".into(), Value::from(message));
        print!("{}\n", jsjson::stringify(&Value::Object(m)));
    } else {
        eprintln!("{message}");
    }
    ExitCode::FAILURE
}

// ── declaration (state.mjs normalizeCommands + test-runner.mjs
//    declaredTestCommands) ──────────────────────────────────────────────────

/// Ordered declared commands, or None when the repo declares no test path
/// (absent commands.test, empty after normalization, or only the "none"
/// sentinel). Err bubbles Node-owned shapes (corrupt config JSON).
fn declared_test_commands(root: &Path) -> Result<Option<Vec<String>>, crate::state::Bail> {
    let config = read_config_raw(root)?;
    // normalizeCommands: non-object (or array) commands -> {}.
    let raw_test = config
        .get("commands")
        .and_then(Value::as_object)
        .and_then(|c| c.get("test"));
    let normalized: Vec<String> = match raw_test {
        Some(Value::String(s)) => {
            let t = js_trim(s);
            if t.is_empty() { Vec::new() } else { vec![t] }
        }
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(js_trim)
            .filter(|t| !t.is_empty())
            .collect(),
        _ => Vec::new(),
    };
    // declaredTestCommands: drop the no-test sentinel (decision 55b951e1).
    let cleaned: Vec<String> = normalized.into_iter().filter(|c| c != "none").collect();
    Ok(if cleaned.is_empty() { None } else { Some(cleaned) })
}

// ── runner (test-runner.mjs runDeclaredTests) ──────────────────────────────

struct CommandResult {
    command: String,
    exit: Option<i64>,
    duration_ms: u64,
    failure_excerpt: Option<String>,
}

struct TestRun {
    ran_at: String,
    green: bool,
    commands: Vec<CommandResult>,
    /// Set when the record write failed (Node throws there).
    write_error: Option<String>,
}

/// Build the `<shell> -c <command>` spawn. On Windows the child's PATH is set
/// explicitly (to the parent's own PATH): Rust's Command then resolves the
/// bare "bash" against PATH FIRST — matching libuv's spawnSync('bash', ...)
/// search, which finds Git Bash — instead of CreateProcess's System32-first
/// order, which finds WSL's bash.exe on WSL-enabled hosts (verified live:
/// bare Command::new("bash") ran Linux). The bare name also keeps bash's $0
/// as "bash", so "bash: line 1: ..." error prefixes stay byte-identical to
/// Node's. (libuv additionally searches cwd before PATH; a bash.exe sitting
/// in the repo root is not mirrored — documented divergence.)
fn shell_command(shell: &str) -> Command {
    let mut cmd = Command::new(shell);
    if cfg!(windows) {
        if let Some(path) = std::env::var_os("PATH") {
            cmd.env("PATH", path);
        }
    }
    cmd
}

/// posixShell (test-runner.mjs:51): on win32, probe `bash -c "exit 0"` once
/// (Git Bash); no bash -> None (Node falls back to cmd.exe — delegated).
/// On POSIX, Node's `shell: true` already IS /bin/sh; probe it the same way
/// so a missing sh delegates instead of embedding a Rust spawn error.
fn posix_shell() -> Option<&'static str> {
    let shell = if cfg!(windows) { "bash" } else { "/bin/sh" };
    let probe = shell_command(shell)
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match probe {
        Ok(s) if s.success() => Some(shell),
        _ => None,
    }
}

fn run_declared_tests(root: &Path, commands: &[String], shell: &str) -> TestRun {
    let ran_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    let mut results: Vec<CommandResult> = Vec::new();
    let mut green = true;
    for command in commands {
        let started = Instant::now();
        let spawned = shell_command(shell)
            .arg("-c")
            .arg(command)
            .current_dir(root)
            .stdin(Stdio::null())
            .output();
        let duration_ms = started.elapsed().as_millis() as u64;
        let (mut output, exit, spawn_err) = match &spawned {
            Ok(out) => (
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
                out.status.code().map(i64::from), // signal death -> null, like JS
                None,
            ),
            Err(e) => (String::new(), None, Some(e.to_string())),
        };
        if let Some(msg) = spawn_err {
            output.push_str(&format!("\n[bee test] spawn error: {msg}"));
        }
        let passed = spawned.is_ok() && exit == Some(0);
        if !passed {
            green = false;
        }
        let failure_excerpt = if passed {
            None
        } else {
            let trimmed = js_trim(&output);
            let tail = utf16_tail(&trimmed, FAILURE_EXCERPT_MAX);
            Some(if tail.is_empty() {
                format!("(no output; exit {})", js_exit(exit))
            } else {
                tail
            })
        };
        results.push(CommandResult {
            command: command.clone(),
            exit,
            duration_ms,
            failure_excerpt,
        });
    }
    // The ONE normalized record: {ran_at, green, commands}.
    let mut record = Map::new();
    record.insert("ran_at".into(), Value::from(ran_at.clone()));
    record.insert("green".into(), Value::Bool(green));
    record.insert(
        "commands".into(),
        Value::Array(results.iter().map(command_result_value).collect()),
    );
    let write_error = write_json_atomic(
        &root.join(".bee").join("logs").join("test-results.json"),
        &Value::Object(record),
    )
    .err()
    .map(|e| e.to_string());
    TestRun {
        ran_at,
        green,
        commands: results,
        write_error,
    }
}

/// {command, exit, duration_ms, failure_excerpt} — frozen key order.
fn command_result_value(c: &CommandResult) -> Value {
    let mut m = Map::new();
    m.insert("command".into(), Value::from(c.command.clone()));
    m.insert(
        "exit".into(),
        match c.exit {
            Some(code) => Value::from(code),
            None => Value::Null,
        },
    );
    m.insert("duration_ms".into(), Value::from(c.duration_ms));
    m.insert(
        "failure_excerpt".into(),
        match &c.failure_excerpt {
            Some(s) => Value::from(s.clone()),
            None => Value::Null,
        },
    );
    Value::Object(m)
}

// ── rendering (bee.mjs renderTestCommandLines ~7601, firstFailureLine) ─────

fn render_test_command_lines(run: &TestRun) -> Vec<String> {
    run.commands
        .iter()
        .map(|c| {
            // `${((c.duration_ms || 0) / 1000).toFixed(1)}s`
            let secs = format!("{:.1}s", c.duration_ms as f64 / 1000.0);
            match &c.failure_excerpt {
                None => format!("✓ {} ({})", c.command, secs),
                Some(_) => format!(
                    "✗ {} ({}, exit {})",
                    c.command,
                    secs,
                    // c.exit ?? 'spawn-failed'
                    c.exit
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "spawn-failed".to_string())
                ),
            }
        })
        .collect()
}

fn first_failure_line(run: &TestRun) -> Option<String> {
    let failing = run
        .commands
        .iter()
        .find(|c| c.failure_excerpt.as_deref().is_some_and(|s| !s.is_empty()))?;
    failing
        .failure_excerpt
        .as_deref()?
        .split('\n')
        .map(js_trim)
        .find(|l| !l.is_empty())
}

// ── JS string primitives ───────────────────────────────────────────────────

/// JS `${exit}` for the "(no output; exit N)" excerpt: null -> "null".
fn js_exit(exit: Option<i64>) -> String {
    exit.map(|e| e.to_string()).unwrap_or_else(|| "null".to_string())
}

/// ECMA WhiteSpace ∪ LineTerminator — String.prototype.trim's char set
/// (includes U+FEFF and NBSP, which Rust's str::trim does not).
fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}' | '\u{000A}' | '\u{000B}' | '\u{000C}' | '\u{000D}' | '\u{0020}'
            | '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{2028}' | '\u{2029}'
            | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{FEFF}'
    )
}

fn js_trim(s: &str) -> String {
    s.trim_matches(is_js_whitespace).to_string()
}

/// JS `.slice(-n)` counts UTF-16 code units. When the boundary would split a
/// surrogate pair (where JS keeps a lone low surrogate no Rust String can
/// hold) the window widens by one unit to keep the pair — the sole excerpt
/// divergence, unreachable for ASCII/BMP output.
fn utf16_tail(s: &str, n: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() <= n {
        return s.to_string();
    }
    let mut start = units.len() - n;
    if (0xDC00..=0xDFFF).contains(&units[start]) {
        start -= 1;
    }
    String::from_utf16_lossy(&units[start..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_config(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("config.json"), content).unwrap();
    }

    #[test]
    fn declaration_normalizes_strings_arrays_and_sentinel() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Absent commands.test -> undeclared.
        write_config(root, r#"{"commands":{}}"#);
        assert!(declared_test_commands(root).ok().unwrap().is_none());
        // String trims; empty/whitespace normalizes away.
        write_config(root, r#"{"commands":{"test":"  npm test  "}}"#);
        assert_eq!(declared_test_commands(root).ok().unwrap(), Some(vec!["npm test".to_string()]));
        write_config(root, r#"{"commands":{"test":"   "}}"#);
        assert!(declared_test_commands(root).ok().unwrap().is_none());
        // Array filters non-strings and empties, keeps order, trims.
        write_config(root, r#"{"commands":{"test":[" a ",1,""," b "]}}"#);
        assert_eq!(
            declared_test_commands(root).ok().unwrap(),
            Some(vec!["a".to_string(), "b".to_string()])
        );
        // The "none" sentinel (decision 55b951e1) declares no-test.
        write_config(root, r#"{"commands":{"test":"none"}}"#);
        assert!(declared_test_commands(root).ok().unwrap().is_none());
        write_config(root, r#"{"commands":{"test":["none"," none "]}}"#);
        assert!(declared_test_commands(root).ok().unwrap().is_none());
        // Non-object commands -> {} -> undeclared.
        write_config(root, r#"{"commands":["x"]}"#);
        assert!(declared_test_commands(root).ok().unwrap().is_none());
        // Corrupt config bails to Node.
        write_config(root, "{broken");
        assert!(declared_test_commands(root).is_err());
    }

    #[test]
    fn green_run_writes_record_and_renders_check_lines() {
        let Some(shell) = posix_shell() else {
            return; // no POSIX sh on this host — the verb delegates anyway
        };
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
        let run = run_declared_tests(root, &["echo hi".to_string()], shell);
        assert!(run.write_error.is_none());
        assert!(run.green);
        assert_eq!(run.commands.len(), 1);
        assert_eq!(run.commands[0].exit, Some(0));
        assert!(run.commands[0].failure_excerpt.is_none());
        let lines = render_test_command_lines(&run);
        assert!(lines[0].starts_with("✓ echo hi ("));
        assert!(lines[0].ends_with("s)"));
        // Record file: {ran_at, green, commands} pretty + trailing newline.
        let raw =
            std::fs::read_to_string(root.join(".bee").join("logs").join("test-results.json"))
                .unwrap();
        assert!(raw.ends_with("\n"));
        let record: Value = serde_json::from_str(&raw).unwrap();
        let obj = record.as_object().unwrap();
        assert_eq!(
            obj.keys().collect::<Vec<_>>(),
            vec!["ran_at", "green", "commands"]
        );
        assert_eq!(obj.get("green"), Some(&json!(true)));
        let cmd = &obj.get("commands").unwrap().as_array().unwrap()[0];
        assert_eq!(
            cmd.as_object().unwrap().keys().collect::<Vec<_>>(),
            vec!["command", "exit", "duration_ms", "failure_excerpt"]
        );
        assert_eq!(cmd.get("failure_excerpt"), Some(&Value::Null));
    }

    #[test]
    fn red_run_records_excerpt_and_first_failure_line() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let run = run_declared_tests(
            root,
            &[
                "echo boom-line; echo more 1>&2; exit 3".to_string(),
                "echo second-ok".to_string(), // never short-circuits on a red
            ],
            shell,
        );
        assert!(!run.green);
        assert_eq!(run.commands[0].exit, Some(3));
        // stdout then stderr, JS-trimmed.
        assert_eq!(run.commands[0].failure_excerpt.as_deref(), Some("boom-line\nmore"));
        assert_eq!(run.commands[1].exit, Some(0));
        assert_eq!(first_failure_line(&run).as_deref(), Some("boom-line"));
        let lines = render_test_command_lines(&run);
        assert!(lines[0].starts_with("✗ "));
        assert!(lines[0].ends_with(", exit 3)"));
        assert!(lines[1].starts_with("✓ "));
    }

    #[test]
    fn missing_command_is_a_red_result_with_exit_127() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let run = run_declared_tests(root, &["definitely-not-a-command-xyz".to_string()], shell);
        assert!(!run.green);
        assert_eq!(run.commands[0].exit, Some(127)); // sh's command-not-found
        let excerpt = run.commands[0].failure_excerpt.as_deref().unwrap();
        assert!(!excerpt.is_empty());
    }

    #[test]
    fn silent_red_gets_the_no_output_excerpt() {
        let Some(shell) = posix_shell() else { return };
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let run = run_declared_tests(root, &["exit 7".to_string()], shell);
        assert_eq!(
            run.commands[0].failure_excerpt.as_deref(),
            Some("(no output; exit 7)")
        );
    }

    #[test]
    fn excerpt_keeps_only_the_last_500_utf16_units() {
        let long = "x".repeat(650);
        assert_eq!(utf16_tail(&long, FAILURE_EXCERPT_MAX).len(), 500);
        assert_eq!(utf16_tail("short", FAILURE_EXCERPT_MAX), "short");
        // Astral chars count 2 units each, like JS .slice.
        let astral = "🐝".repeat(300); // 600 units
        let tail = utf16_tail(&astral, FAILURE_EXCERPT_MAX);
        assert_eq!(tail.encode_utf16().count(), 500); // boundary lands on a pair
    }

    #[test]
    fn js_trim_covers_the_ecma_whitespace_set() {
        assert_eq!(js_trim("\u{FEFF}\u{00A0} boom \n\t"), "boom");
    }
}
