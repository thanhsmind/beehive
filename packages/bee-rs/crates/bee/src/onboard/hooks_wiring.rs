// onboard::hooks_wiring — the two rendered hook projections and their merges.
//
// Provenance: onboard_bee.mjs vendoredBeeBinary (l. 2242), repoHookCommand
// (l. 2249), renderRepoHookEntries (l. 2258), isBeeHookEntry (l. 2281),
// mergeRepoSettings (l. 2296), repoOwnsHookCatalog (l. 2344),
// runtimeCoversCodex (l. 2360), codexHookCommand (l. 2364),
// codexWindowsBootstrap (l. 2400), renderCodexHookEntries (l. 2420),
// isBeeCodexHookEntry (l. 2493), mergeCodexHooks (l. 2504),
// codexHookWriteBlocker (l. 2540), codexUserConfigPath (l. 2574),
// codexStatuslineMissing (l. 2578) and statuslineOptIn (l. 1883).
//
// The binary/node feature detection in repoHookCommand + isBeeHookEntry is
// the rust-port R2 behaviour and is ported verbatim: a host WITHOUT a
// vendored `.bee/bin/bee[.exe]` keeps byte-identical node wiring, and a
// re-render replaces EITHER spelling instead of stacking duplicates.

use super::templates::{
    CODEX_BINARY_MISSING_DIAGNOSTIC, CODEX_STATUS_LINE_BLOCK, CODEX_TRANSPORT_DIAGNOSTIC,
};
use super::util::{
    exists, home_dir, lstat_if_exists, read_json_if_exists, read_text_if_exists, split_lines,
};
use crate::jsjson;
use serde_json::{json, Map, Value};
use std::path::Path;

// ── Claude repo projection (.claude/settings.json) ─────────────────────────

/// The main-checkout arm, appended to every projection that resolves the
/// binary from a directory the host hands it.
///
/// A linked git worktree materialises TRACKED files only, and the vendored
/// binary is deliberately untracked (decision 1f4262ca) — so a worktree starts
/// with every hook wired to a path that does not exist, and each one fails
/// open. Nothing bee-side can warn about it either: the thing that would do
/// the warning is the binary that is missing. `--git-common-dir` is the one
/// question git answers usefully from both sides — it names the MAIN
/// repository's `.git`, whose parent is the checkout holding the binary.
/// (`--show-toplevel` does not: from a linked worktree it returns the worktree
/// itself. Measured, git 2.51.2.)
///
/// It runs ONLY after the direct candidates have missed, so an ordinary
/// checkout never pays for the git process.
pub(crate) fn main_checkout_arm(hook_name: &str, dir_expr: &str, args: &str) -> String {
    format!(
        "g=\"$(git -C {dir_expr} rev-parse --path-format=absolute --git-common-dir 2>/dev/null)\"; \
[ -n \"$g\" ] && for b in \"$g/../.bee/bin/bee\" \"$g/../.bee/bin/bee.exe\"; \
do [ -x \"$b\" ] && exec \"$b\" hook {hook_name}{args}; done"
    )
}

/// repoHookCommand (l. 2249).
///
/// ONE shape, whether or not the host already carries a binary. The old
/// vendored-binary fast path rendered a bare `"$CLAUDE_PROJECT_DIR"/.bee/bin/
/// bee hook X`, which in a linked worktree is not a missing candidate the
/// runner can fall past — it is a command that does not exist, so no fallback
/// could ever run behind it. Probing costs two `[ -x ]` tests against a
/// process spawn; the fast path was never worth what it cost here.
fn repo_hook_command(file_name: &str, _repo_root: Option<&Path>) -> String {
    let hook_name =
        file_name.strip_prefix("bee-").unwrap_or(file_name).strip_suffix(".mjs").unwrap_or(file_name);
    let arm = main_checkout_arm(hook_name, "\"$CLAUDE_PROJECT_DIR\"", "");
    format!(
        "for b in \"$CLAUDE_PROJECT_DIR/.bee/bin/bee\" \"$CLAUDE_PROJECT_DIR/.bee/bin/bee.exe\"; \
do [ -x \"$b\" ] && exec \"$b\" hook {hook_name}; done; {arm}; \
echo \"{CODEX_BINARY_MISSING_DIAGNOSTIC}\" >&2; exit 0"
    )
}

fn repo_entry(file_name: &str, repo_root: Option<&Path>) -> Value {
    json!({"type": "command", "command": repo_hook_command(file_name, repo_root)})
}

/// renderRepoHookEntries (l. 2258) — event order is the literal's, which
/// drives the merged object's key order for a fresh settings.json.
pub fn render_repo_hook_entries(repo_root: &Path) -> Vec<(&'static str, Value)> {
    let r = Some(repo_root);
    vec![
        (
            "SessionStart",
            json!([{ "matcher": "startup|resume|clear|compact", "hooks": [repo_entry("bee-session-init.mjs", r)] }]),
        ),
        ("UserPromptSubmit", json!([{ "hooks": [repo_entry("bee-prompt-context.mjs", r)] }])),
        (
            "PreToolUse",
            json!([
                { "matcher": "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion", "hooks": [repo_entry("bee-write-guard.mjs", r)] },
                { "matcher": "Agent|Task", "hooks": [repo_entry("bee-model-guard.mjs", r)] }
            ]),
        ),
        (
            "PostToolUse",
            json!([
                { "matcher": "update_plan|TaskCreate|TaskUpdate|TodoWrite", "hooks": [repo_entry("bee-state-sync.mjs", r)] },
                { "hooks": [repo_entry("bee-tools-logger.mjs", r)] }
            ]),
        ),
        (
            "SubagentStop",
            json!([{ "hooks": [repo_entry("bee-state-sync.mjs", r), repo_entry("bee-chain-nudge.mjs", r)] }]),
        ),
        ("PreCompact", json!([{ "hooks": [repo_entry("bee-session-close.mjs", r)] }])),
        (
            "Stop",
            json!([{ "hooks": [repo_entry("bee-state-sync.mjs", r), repo_entry("bee-session-close.mjs", r)] }]),
        ),
    ]
}

fn entry_hook_commands(entry: &Value) -> Vec<String> {
    // `entry?.hooks || []` then `String(hook?.command || "")`.
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .map(|hooks| {
            hooks
                .iter()
                .map(|h| match h.get("command") {
                    Some(Value::String(s)) => s.clone(),
                    Some(v) if !v.is_null() && v != &Value::Bool(false) => jsjson::stringify(v),
                    _ => String::new(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// isBeeHookEntry (l. 2281): recognizes BOTH wirings (node script and Rust
/// binary) so a re-render replaces either shape instead of stacking.
fn is_bee_hook_entry(entry: &Value) -> bool {
    for command in entry_hook_commands(entry) {
        if command.contains(".bee/bin/hooks/bee-") {
            return true;
        }
        if command.contains(".bee/bin/bee") && command.contains(" hook ") {
            return true;
        }
    }
    false
}

pub struct Merged {
    pub text: String,
    pub changed: bool,
}

/// The shared merge body of mergeRepoSettings / mergeCodexHooks: non-bee
/// entries preserved verbatim, stale bee entries replaced, a second apply a
/// no-op. `{...existing, hooks: merged}` keeps `existing`'s key order and
/// overwrites `hooks` in place (appending it only when absent) — IndexMap
/// insert has exactly those semantics.
fn merge_hooks_file(
    existing: Option<Value>,
    rendered: Vec<(&'static str, Value)>,
    is_bee: fn(&Value) -> bool,
) -> Merged {
    let existing = match existing {
        Some(Value::Object(m)) => m,
        _ => Map::new(),
    };
    let hooks = match existing.get("hooks") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    let mut merged = hooks;
    let mut changed = false;
    for (event_name, entries) in rendered {
        let current: Vec<Value> = match merged.get(event_name) {
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        let mut next: Vec<Value> = current.iter().filter(|e| !is_bee(e)).cloned().collect();
        next.extend(entries.as_array().unwrap().iter().cloned());
        if jsjson::stringify(&Value::Array(current)) != jsjson::stringify(&Value::Array(next.clone()))
        {
            changed = true;
        }
        merged.insert(event_name.to_string(), Value::Array(next));
    }
    let mut out = existing;
    out.insert("hooks".into(), Value::Object(merged));
    Merged { text: format!("{}\n", jsjson::stringify_pretty(&Value::Object(out))), changed }
}

/// mergeRepoSettings (l. 2296). `<repo>/.claude/settings.json -> <repo>` for
/// the vendored-binary probe.
pub fn merge_repo_settings(settings_path: &Path) -> Merged {
    let repo_root = settings_path
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| Path::new(".").to_path_buf());
    merge_hooks_file(
        read_json_if_exists(settings_path),
        render_repo_hook_entries(&repo_root),
        is_bee_hook_entry,
    )
}

// ── Codex repo projection (.codex/hooks.json) ──────────────────────────────

/// repoOwnsHookCatalog (l. 2344): a repo that ships the hook catalog IS bee,
/// and that catalog — not this projection — owns its `.codex/hooks.json`.
/// Writing this HOST projection into bee's own repo silently clobbers the
/// catalog rendering; that is how a release once broke, and this self-skip is
/// what prevents it.
///
/// CUTOVER: the catalog moved into the binary
/// (devtools/hook_manifests.rs), so the marker is now the plugin projection it
/// generates — `packages/bee/hooks/claude-hooks.json`, a file only bee ships.
/// The two `catalog.mjs` locations stay as OR arms so a checkout still on an
/// older bee (pre-cutover, or pre packages-restructure) is correctly
/// self-identified: backward-compatible detection, never forceable.
pub fn repo_owns_hook_catalog(repo_root: &Path) -> bool {
    exists(&repo_root.join("packages").join("bee").join("hooks").join("claude-hooks.json"))
        || exists(&repo_root.join("packages").join("bee").join("hooks").join("catalog.mjs"))
        || exists(&repo_root.join("hooks").join("catalog.mjs"))
}

/// runtimeCoversCodex (l. 2360): reads the PASSED --runtime only.
pub fn runtime_covers_codex(runtime: &str) -> bool {
    runtime == "codex" || runtime == "both"
}

/// `bee-write-guard.mjs` -> `write-guard`. Both arms derive from the hook
/// NAME now that neither names a wrapper file.
fn codex_hook_name(file_name: &str) -> &str {
    file_name.strip_prefix("bee-").unwrap_or(file_name).strip_suffix(".mjs").unwrap_or(file_name)
}

/// codexHookCommand (l. 2364).
///
/// rust-port R6 (cutover): the HOST Codex projection was the FOURTH wiring
/// surface and the last one still naming node. It launches the vendored
/// binary — the only runtime — with a VISIBLE fail-open arm when the host has
/// not built one, matching the no-git-root arm's discipline (spec R2:
/// fail-open must be visible, never silent). Detection is at HOOK TIME, in the
/// host's own shell, because `$r` does not exist until the command runs.
pub(crate) fn codex_hook_command(file_name: &str) -> String {
    let name = codex_hook_name(file_name);
    [
        "r=\"$(git rev-parse --show-toplevel 2>/dev/null)\"".to_string(),
        format!("[ -n \"$r\" ] || {{ echo \"{CODEX_TRANSPORT_DIAGNOSTIC}\" >&2; exit 0; }}"),
        format!(
            "for b in \"$r\"/.bee/bin/bee \"$r\"/.bee/bin/bee.exe; do [ -x \"$b\" ] && exec \"$b\" hook {name} --source=repo; done"
        ),
        // `$r` is --show-toplevel, which in a linked worktree is the WORKTREE,
        // not the checkout carrying the binary. Reach the main one.
        main_checkout_arm(&name, "\"$r\"", " --source=repo"),
        format!("echo \"{CODEX_BINARY_MISSING_DIAGNOSTIC}\" >&2; exit 0"),
    ]
    .join("\n")
}

/// codexWindowsBootstrap (l. 2400): one string that cmd.exe and
/// powershell.exe must parse IDENTICALLY, because nothing tells us which of
/// them Codex hands it to. That is the whole constraint, and it bans `$`
/// (PowerShell variables), `%` (cmd variables) and backtick (PowerShell
/// escape) — which also bans both shells' command substitution.
///
/// NODE IS GONE FROM THIS ARM. It used to be `node -e SCRIPT <name>`, kept on
/// the reasoning that "no command substitution" meant the string could not ask
/// git for the repo root, and without the root it could not name the binary —
/// so it needed an interpreter that could do the lookup itself. That reasoning
/// had a hole: it assumed the string must COMPUTE the root. It does not. A git
/// alias prefixed with `!` is run **from the top level of the repository**, so
/// git resolves the root for us and the body can name the binary by a path
/// relative to it. No substitution, no interpreter, no Node — bee ships one
/// binary and now its Windows hook transport needs nothing else installed that
/// git itself did not already require.
///
/// The body runs in the shell git uses for `!` aliases (Git for Windows'
/// bundled `sh`, never the WSL launcher a bare `bash` would reach), so it is
/// the same POSIX shape as the arm above with two deliberate differences:
///
///   * **No `$`.** The two candidate binary paths are unrolled into two
///     statements instead of a `for b in …; do [ -x "$b" ] …` loop, and the
///     no-root check reads `git rev-parse`'s EXIT STATUS rather than capturing
///     its output.
///   * **Single quotes only.** The alias value is wrapped in the one pair of
///     double quotes the outer shell consumes, so the body may not contain
///     another `"`. Both diagnostics are single-quoted; `windows_transport_
///     stays_parseable_by_both_shells` pins that neither ever grows a `'`.
///
/// VISIBLE FAIL-OPEN ON THIS TRANSPORT TOO. Both bail arms print the SAME
/// constants the POSIX arm uses — the wordings cannot drift apart — and exit
/// 0. They once were a bare `process.exit(0)`, which meant a Codex session on
/// Windows that could not resolve its repo or binary lost every guard, the
/// write guard included, without printing one character.
///
/// Measured on Windows 11 / git 2.51.2 across the full matrix — {cmd.exe,
/// powershell.exe} x {repo root, repo subdirectory, repo with no vendored
/// binary, not a repo at all} — all eight launch the binary or print the right
/// diagnostic and exit 0, byte-identically per shell.
pub(crate) fn codex_hook_command_windows(file_name: &str) -> String {
    let name = codex_hook_name(file_name);
    let body = [
        // `|| { … }` on the exit status: capturing the output would need `$`.
        format!(
            "git rev-parse --show-toplevel >/dev/null 2>&1 || {{ echo '{CODEX_TRANSPORT_DIAGNOSTIC}' >&2; exit 0; }}"
        ),
        // Unrolled rather than looped, for the same no-`$` reason. Relative to
        // the repo top level, which is where git runs a `!` alias.
        format!("[ -x .bee/bin/bee ] && exec .bee/bin/bee hook {name} --source=repo"),
        format!("[ -x .bee/bin/bee.exe ] && exec .bee/bin/bee.exe hook {name} --source=repo"),
        format!("echo '{CODEX_BINARY_MISSING_DIAGNOSTIC}' >&2; exit 0"),
    ]
    .join("; ");
    format!("git -c alias.beehook=\"!{body}\" beehook")
}

fn codex_entry(file_name: &str, status_message: &str) -> Value {
    json!({
        "type": "command",
        "command": codex_hook_command(file_name),
        "commandWindows": codex_hook_command_windows(file_name),
        "statusMessage": status_message,
    })
}

/// renderCodexHookEntries (l. 2420).
pub fn render_codex_hook_entries() -> Vec<(&'static str, Value)> {
    vec![
        (
            "SessionStart",
            json!([{ "matcher": "startup|resume|clear|compact", "hooks": [codex_entry("bee-session-init.mjs", "bee: session bootstrap")] }]),
        ),
        (
            "UserPromptSubmit",
            json!([{ "hooks": [codex_entry("bee-prompt-context.mjs", "bee: phase reminder")] }]),
        ),
        (
            "PreToolUse",
            json!([
                { "matcher": "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion", "hooks": [codex_entry("bee-write-guard.mjs", "bee: write guard")] },
                { "matcher": "spawn_agent", "hooks": [codex_entry("bee-model-guard.mjs", "bee: model-tier guard")] }
            ]),
        ),
        (
            "PostToolUse",
            json!([
                { "matcher": "update_plan|TaskCreate|TaskUpdate|TodoWrite", "hooks": [codex_entry("bee-state-sync.mjs", "bee: state sync")] },
                { "hooks": [codex_entry("bee-tools-logger.mjs", "bee: tools logger")] }
            ]),
        ),
        (
            "SubagentStart",
            json!([{ "hooks": [codex_entry("bee-codex-subagent-audit.mjs", "bee: subagent start audit")] }]),
        ),
        (
            "SubagentStop",
            json!([
                { "hooks": [codex_entry("bee-state-sync.mjs", "bee: state sync"), codex_entry("bee-chain-nudge.mjs", "bee: chain nudge")] },
                { "hooks": [codex_entry("bee-codex-subagent-audit.mjs", "bee: subagent stop audit")] }
            ]),
        ),
        (
            "PreCompact",
            json!([{ "hooks": [codex_entry("bee-session-close.mjs", "bee: pre-compact flush check")] }]),
        ),
        (
            "Stop",
            json!([{ "hooks": [codex_entry("bee-state-sync.mjs", "bee: state sync"), codex_entry("bee-session-close.mjs", "bee: session close check")] }]),
        ),
    ]
}

/// `/hooks\/bee-[a-z-]+\.mjs/` — a bee entry in ANY historical Codex
/// transport shape (isBeeCodexHookEntry, l. 2493).
fn matches_codex_bee_command(command: &str) -> bool {
    let bytes = command.as_bytes();
    let needle = b"hooks/bee-";
    let mut i = 0usize;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            let mut j = i + needle.len();
            let start = j;
            while j < bytes.len() && (bytes[j].is_ascii_lowercase() || bytes[j] == b'-') {
                j += 1;
            }
            if j > start && command[j..].starts_with(".mjs") {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// rust-port R6: also recognizes the BINARY spelling (`.bee/bin/bee … hook
/// <name>`), which carries no wrapper filename at all — without this arm a
/// re-render stacks a second copy beside the first and every event
/// double-fires.
fn is_bee_codex_hook_entry(entry: &Value) -> bool {
    entry_hook_commands(entry)
        .iter()
        .any(|c| matches_codex_bee_command(c) || (c.contains(".bee/bin/bee") && c.contains(" hook ")))
}

/// mergeCodexHooks (l. 2504).
pub fn merge_codex_hooks(hooks_path: &Path) -> Merged {
    merge_hooks_file(
        read_json_if_exists(hooks_path),
        render_codex_hook_entries(),
        is_bee_codex_hook_entry,
    )
}

/// The `.codex/hooks.json` pseudo-entry hashed into managed.repo_hooks /
/// managed.codex_hooks: `sha256(JSON.stringify(renderCodexHookEntries()))`.
pub fn codex_hook_entries_json() -> String {
    let mut m = Map::new();
    for (k, v) in render_codex_hook_entries() {
        m.insert(k.to_string(), v);
    }
    jsjson::stringify(&Value::Object(m))
}

/// codexHookWriteBlocker (l. 2540): a typed preflight for the codex-hybrid
/// write path. Never forceable — there is no downgrade to force through a
/// filesystem collision.
pub fn codex_hook_write_blocker(repo_root: &Path) -> Option<(String, String)> {
    let checks = [
        (repo_root.join(".bee").join("bin").join("hooks"), ".bee/bin/hooks"),
        (repo_root.join(".codex"), ".codex"),
    ];
    for (target, label) in checks {
        if let Some(stat) = lstat_if_exists(&target) {
            if !stat.is_dir {
                return Some((
                    "blocked".to_string(),
                    format!(
                        "codex hook apply refused: \"{label}\" exists and is not a directory. FIX: remove or rename it, then retry the hybrid apply, or pass --distribution repo-copy for a plain repo-local install instead."
                    ),
                ));
            }
        }
    }
    None
}

// ── Codex user-config status line ──────────────────────────────────────────

pub fn codex_user_config_path() -> std::path::PathBuf {
    home_dir().join(".codex").join("config.toml")
}

/// `/^[ \t]*status_line[ \t]*=/m`
fn has_status_line_key(text: &str) -> bool {
    split_lines(text).into_iter().any(|line| {
        let rest = line.trim_start_matches([' ', '\t']);
        rest.strip_prefix("status_line")
            .map(|t| t.trim_start_matches([' ', '\t']).starts_with('='))
            .unwrap_or(false)
    })
}

/// codexStatuslineMissing (l. 2578): Codex absent (no user config file) ⇒
/// stay out entirely.
pub fn codex_statusline_missing() -> bool {
    let config_path = codex_user_config_path();
    if !exists(&config_path) {
        return false;
    }
    !has_status_line_key(&read_text_if_exists(&config_path))
}

/// The `[tui]`-aware insertion of ensure_codex_statusline (l. 3820).
pub fn codex_statusline_next_text(text: &str) -> String {
    // `/^\[tui\][ \t]*\r?$/m`, first match only. The matched text INCLUDES a
    // CRLF's `\r` (the regex consumes it), so the replacement must keep it
    // attached to the header line, not to the tail.
    let bytes = text.as_bytes();
    let mut line_start = 0usize;
    loop {
        let line_end = bytes[line_start..]
            .iter()
            .position(|b| *b == b'\n')
            .map(|i| line_start + i)
            .unwrap_or(bytes.len());
        let line = &text[line_start..line_end];
        let is_tui = line.strip_prefix("[tui]").is_some_and(|t| {
            let t = t.strip_suffix('\r').unwrap_or(t);
            t.chars().all(|c| c == ' ' || c == '\t')
        });
        if is_tui {
            return format!(
                "{}{line}\n{}{}",
                &text[..line_start],
                CODEX_STATUS_LINE_BLOCK.trim_end(),
                &text[line_end..]
            );
        }
        if line_end >= bytes.len() {
            break;
        }
        line_start = line_end + 1;
    }
    let sep = if text.is_empty() || text.ends_with('\n') { "" } else { "\n" };
    format!("{text}{sep}\n[tui]\n{CODEX_STATUS_LINE_BLOCK}")
}

// ── statusline opt-in ──────────────────────────────────────────────────────

/// statuslineOptIn (l. 1883): a repo opts in by ALREADY pointing its
/// .claude/settings.json statusLine at the project-level script.
pub fn statusline_opt_in(repo_root: &Path) -> bool {
    let settings = read_json_if_exists(&repo_root.join(".claude").join("settings.json"));
    let command = settings
        .as_ref()
        .and_then(|s| s.get("statusLine"))
        .filter(|v| v.is_object())
        .and_then(|v| v.get("command"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let Some(command) = command else { return false };
    if !command.contains(".claude/statusline-command.sh") {
        return false;
    }
    project_dir_anchored(&command) || bare_repo_relative(&command)
}

/// `/\$\{?CLAUDE_PROJECT_DIR[^"'\s{}]*\}?\/\.claude\/statusline-command\.sh/`
fn project_dir_anchored(command: &str) -> bool {
    let mut search = command;
    while let Some(idx) = search.find("$") {
        let after_dollar = &search[idx + 1..];
        let after_brace = after_dollar.strip_prefix('{').unwrap_or(after_dollar);
        if let Some(rest) = after_brace.strip_prefix("CLAUDE_PROJECT_DIR") {
            // `[^"'\s{}]*` greedy, then optional `}`, then the literal tail.
            let run_end = rest
                .find(|c: char| c == '"' || c == '\'' || c.is_whitespace() || c == '{' || c == '}')
                .unwrap_or(rest.len());
            // Backtrack over the greedy run looking for the literal tail,
            // with and without the optional closing brace.
            for take in (0..=run_end).rev() {
                let tail = &rest[take..];
                if tail.starts_with("/.claude/statusline-command.sh")
                    || tail.starts_with("}/.claude/statusline-command.sh")
                {
                    return true;
                }
            }
        }
        search = &search[idx + 1..];
    }
    false
}

/// `/(^|[\s"'=(])\.claude\/statusline-command\.sh/`
fn bare_repo_relative(command: &str) -> bool {
    const NEEDLE: &str = ".claude/statusline-command.sh";
    let mut from = 0usize;
    while let Some(rel) = command[from..].find(NEEDLE) {
        let idx = from + rel;
        if idx == 0 {
            return true;
        }
        let prev = command[..idx].chars().next_back().unwrap();
        if prev.is_whitespace() || prev == '"' || prev == '\'' || prev == '=' || prev == '(' {
            return true;
        }
        from = idx + 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CUTOVER: a host with no binary yet gets a runtime-detecting loop that
    /// SAYS SO rather than a `node …bee-write-guard.mjs` command with nothing
    /// behind it.
    #[test]
    fn a_host_without_a_binary_gets_a_detecting_loop_and_a_visible_diagnostic() {
        let dir = tempfile::tempdir().unwrap();
        let cmd = repo_hook_command("bee-write-guard.mjs", Some(dir.path()));
        assert!(!cmd.contains(".mjs"), "{cmd}");
        assert!(!cmd.contains("node "), "{cmd}");
        assert!(cmd.contains("exec \"$b\" hook write-guard"), "{cmd}");
        assert!(cmd.contains("bee: hook binary missing"), "{cmd}");
    }

    /// One shape whether or not a binary is already vendored — a host that
    /// carries one must still be able to fall through to the main checkout
    /// when the SAME settings file is read from a linked worktree.
    #[test]
    fn wiring_is_one_shape_and_reaches_the_main_checkout() {
        let dir = tempfile::tempdir().unwrap();
        let with_binary = {
            let bin = dir.path().join(".bee").join("bin");
            std::fs::create_dir_all(&bin).unwrap();
            std::fs::write(bin.join("bee.exe"), "x").unwrap();
            repo_hook_command("bee-write-guard.mjs", Some(dir.path()))
        };
        let without_binary = {
            std::fs::remove_file(dir.path().join(".bee").join("bin").join("bee.exe")).unwrap();
            repo_hook_command("bee-write-guard.mjs", Some(dir.path()))
        };
        assert_eq!(with_binary, without_binary, "the vendored fast path is gone");
        // Direct candidates first, main checkout only after they miss.
        assert!(with_binary.contains("$CLAUDE_PROJECT_DIR/.bee/bin/bee.exe"), "{with_binary}");
        assert!(with_binary.contains("--git-common-dir"), "{with_binary}");
        assert!(with_binary.contains("$g/../.bee/bin/bee.exe"), "{with_binary}");
        assert!(
            with_binary.find("CLAUDE_PROJECT_DIR").unwrap() < with_binary.find("git-common-dir").unwrap(),
            "the git probe must not run before the cheap candidates: {with_binary}"
        );
        assert!(with_binary.ends_with("exit 0"), "{with_binary}");
        assert!(with_binary.contains("bee: hook binary missing"), "{with_binary}");
    }

    #[test]
    fn both_wirings_are_recognized_as_bee_entries() {
        let node = json!({"hooks":[{"type":"command","command":"node \"$CLAUDE_PROJECT_DIR\"/.bee/bin/hooks/bee-write-guard.mjs"}]});
        let rust = json!({"hooks":[{"type":"command","command":"\"$CLAUDE_PROJECT_DIR\"/.bee/bin/bee hook write-guard"}]});
        let foreign = json!({"hooks":[{"type":"command","command":"node ./scripts/my-hook.mjs"}]});
        assert!(is_bee_hook_entry(&node));
        assert!(is_bee_hook_entry(&rust));
        assert!(!is_bee_hook_entry(&foreign));
    }

    #[test]
    fn merge_preserves_foreign_entries_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        std::fs::write(
            &settings,
            r#"{
  "model": "opus",
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [{ "type": "command", "command": "node ./my-own.mjs" }] }
    ],
    "Notification": [
      { "hooks": [{ "type": "command", "command": "notify" }] }
    ]
  },
  "statusLine": { "type": "command", "command": "x" }
}
"#,
        )
        .unwrap();
        let merged = merge_repo_settings(&settings);
        assert!(merged.changed);
        let v: Value = serde_json::from_str(&merged.text).unwrap();
        // Foreign top-level keys keep their order; hooks stays in place.
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["model", "hooks", "statusLine"]);
        // Foreign hook entries survive, bee entries are appended after them.
        let pre = &v["hooks"]["PreToolUse"];
        assert_eq!(pre[0]["hooks"][0]["command"], "node ./my-own.mjs");
        assert_eq!(pre.as_array().unwrap().len(), 3);
        // The unrelated event is untouched.
        assert_eq!(v["hooks"]["Notification"][0]["hooks"][0]["command"], "notify");
        // Second pass is a no-op.
        std::fs::write(&settings, &merged.text).unwrap();
        let again = merge_repo_settings(&settings);
        assert!(!again.changed);
        assert_eq!(again.text, merged.text);
    }

    #[test]
    fn merge_replaces_a_stale_bee_entry_instead_of_stacking() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        std::fs::write(
            &settings,
            r#"{"hooks":{"UserPromptSubmit":[{"hooks":[{"type":"command","command":"node \"$CLAUDE_PROJECT_DIR\"/.bee/bin/hooks/bee-prompt-context.mjs --old"}]}]}}"#,
        )
        .unwrap();
        let merged = merge_repo_settings(&settings);
        let v: Value = serde_json::from_str(&merged.text).unwrap();
        assert_eq!(v["hooks"]["UserPromptSubmit"].as_array().unwrap().len(), 1);
        assert!(!v["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("--old"));
    }

    #[test]
    fn codex_projection_shape_and_idempotence() {
        let dir = tempfile::tempdir().unwrap();
        let hooks = dir.path().join(".codex").join("hooks.json");
        let merged = merge_codex_hooks(&hooks);
        assert!(merged.changed);
        let v: Value = serde_json::from_str(&merged.text).unwrap();
        let events: Vec<&str> = v["hooks"].as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(
            events,
            vec![
                "SessionStart",
                "UserPromptSubmit",
                "PreToolUse",
                "PostToolUse",
                "SubagentStart",
                "SubagentStop",
                "PreCompact",
                "Stop"
            ]
        );
        assert_eq!(v["hooks"]["PreToolUse"][1]["matcher"], "spawn_agent");
        assert!(v["hooks"]["SessionStart"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap()
            .contains("git rev-parse --show-toplevel"));
        assert!(v["hooks"]["SessionStart"][0]["hooks"][0]["commandWindows"]
            .as_str()
            .unwrap()
            .starts_with("git -c alias.beehook=\"!"));
        std::fs::create_dir_all(hooks.parent().unwrap()).unwrap();
        std::fs::write(&hooks, &merged.text).unwrap();
        assert!(!merge_codex_hooks(&hooks).changed);
    }

    /// rust-port R6 — the FOURTH wiring surface. Both legs must name the
    /// binary and NEITHER may name a `.mjs` wrapper: after the cutover there
    /// is no wrapper to launch, so a surviving `exec node …/bee-*.mjs` arm is
    /// a hook that silently does nothing on every event.
    #[test]
    fn codex_host_projection_launches_the_binary_on_both_legs() {
        let posix = codex_hook_command("bee-write-guard.mjs");
        assert_eq!(
            posix,
            concat!(
                "r=\"$(git rev-parse --show-toplevel 2>/dev/null)\"\n",
                "[ -n \"$r\" ] || { echo \"bee: hook transport unavailable (no git root)\" >&2; exit 0; }\n",
                "for b in \"$r\"/.bee/bin/bee \"$r\"/.bee/bin/bee.exe; do [ -x \"$b\" ] && exec \"$b\" hook write-guard --source=repo; done\n",
                "g=\"$(git -C \"$r\" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)\"; ",
                "[ -n \"$g\" ] && for b in \"$g/../.bee/bin/bee\" \"$g/../.bee/bin/bee.exe\"; ",
                "do [ -x \"$b\" ] && exec \"$b\" hook write-guard --source=repo; done\n",
                "echo \"bee: hook binary missing (.bee/bin/bee)\" >&2; exit 0"
            )
        );
        let win = codex_hook_command_windows("bee-write-guard.mjs");
        assert_eq!(
            win,
            concat!(
                "git -c alias.beehook=\"!",
                "git rev-parse --show-toplevel >/dev/null 2>&1 || ",
                "{ echo 'bee: hook transport unavailable (no git root)' >&2; exit 0; }; ",
                "[ -x .bee/bin/bee ] && exec .bee/bin/bee hook write-guard --source=repo; ",
                "[ -x .bee/bin/bee.exe ] && exec .bee/bin/bee.exe hook write-guard --source=repo; ",
                "echo 'bee: hook binary missing (.bee/bin/bee)' >&2; exit 0",
                "\" beehook"
            )
        );
        for leg in [&posix, &win] {
            assert!(!leg.contains(".mjs"), "wrapper spelling survived: {leg}");
            assert!(!leg.contains("/.bee/bin/hooks/"), "wrapper path survived: {leg}");
        }
        // Node left this arm: bee ships one binary, and git's `!` alias runs
        // from the repo top level, so nothing has to COMPUTE the root.
        assert!(!win.contains("node"), "node survived the Windows leg: {win}");
    }

    /// The one property that makes `commandWindows` a single string for two
    /// shells, pinned as its own test because every clause of it is a silent
    /// breakage: the wrong character does not fail to render, it renders and
    /// then behaves differently under cmd.exe than under powershell.exe.
    #[test]
    fn windows_transport_stays_parseable_by_both_shells() {
        for script in ["bee-session-init.mjs", "bee-write-guard.mjs", "bee-prompt-context.mjs"] {
            let win = codex_hook_command_windows(script);
            // `$` = PowerShell variable, `%` = cmd variable, backtick =
            // PowerShell escape. Any of them and the two shells disagree.
            for ch in ['$', '%', '`'] {
                assert!(!win.contains(ch), "banned {ch:?} in commandWindows: {win}");
            }
            // The alias value owns the ONLY pair of double quotes — the outer
            // shell consumes them. A `"` inside the body would close it early.
            assert_eq!(win.matches('"').count(), 2, "body grew a double quote: {win}");
            // …so both diagnostics are single-quoted inside sh, which means
            // neither may ever contain a `'` of its own.
            for diagnostic in [CODEX_TRANSPORT_DIAGNOSTIC, CODEX_BINARY_MISSING_DIAGNOSTIC] {
                assert!(
                    !diagnostic.contains('\''),
                    "diagnostic {diagnostic:?} has a single quote — it cannot be sh-quoted in the Windows arm"
                );
                assert!(win.contains(diagnostic), "{diagnostic:?} missing from {win}");
            }
        }
    }

    /// The whole point of the recognizer widening: a host whose
    /// `.codex/hooks.json` still carries the OLD node wiring gets it REPLACED,
    /// not stacked beside the new one.
    #[test]
    fn a_stale_node_codex_entry_is_replaced_not_stacked() {
        let dir = tempfile::tempdir().unwrap();
        let hooks = dir.path().join(".codex").join("hooks.json");
        std::fs::create_dir_all(hooks.parent().unwrap()).unwrap();
        std::fs::write(
            &hooks,
            r#"{"hooks":{"UserPromptSubmit":[{"hooks":[{"type":"command","command":"exec node \"$r\"/.bee/bin/hooks/bee-prompt-context.mjs --source=repo"}]}]}}"#,
        )
        .unwrap();
        let merged = merge_codex_hooks(&hooks);
        let v: Value = serde_json::from_str(&merged.text).unwrap();
        assert_eq!(v["hooks"]["UserPromptSubmit"].as_array().unwrap().len(), 1);
        assert!(!merged.text.contains(".mjs"));
        // …and the fresh render is itself idempotent.
        std::fs::write(&hooks, &merged.text).unwrap();
        assert!(!merge_codex_hooks(&hooks).changed);
    }

    #[test]
    fn codex_bee_command_regex() {
        assert!(matches_codex_bee_command("exec node \"$r\"/.bee/bin/hooks/bee-write-guard.mjs --source=repo"));
        assert!(matches_codex_bee_command("node \"$CLAUDE_PROJECT_DIR\"/hooks/bee-state-sync.mjs"));
        assert!(!matches_codex_bee_command("node hooks/my-thing.mjs"));
        assert!(!matches_codex_bee_command("node hooks/bee-.js"));
    }

    #[test]
    fn statusline_opt_in_only_for_project_level_references() {
        let dir = tempfile::tempdir().unwrap();
        let settings = dir.path().join(".claude").join("settings.json");
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        let write = |cmd: &str| {
            std::fs::write(
                &settings,
                serde_json::to_string(&json!({"statusLine":{"type":"command","command":cmd}}))
                    .unwrap(),
            )
            .unwrap();
        };
        // Oracle table, produced by running onboard_bee.mjs's own two regexes
        // over each shape under Node. Note the first row: a closing quote
        // BETWEEN the variable and the path defeats both alternatives, so it
        // is NOT an opt-in — surprising, but it is the shipped behaviour.
        let oracle: &[(&str, bool)] = &[
            ("bash \"$CLAUDE_PROJECT_DIR\"/.claude/statusline-command.sh", false),
            ("bash \"$CLAUDE_PROJECT_DIR/.claude/statusline-command.sh\"", true),
            ("bash $CLAUDE_PROJECT_DIR/.claude/statusline-command.sh", true),
            ("bash ${CLAUDE_PROJECT_DIR}/.claude/statusline-command.sh", true),
            (".claude/statusline-command.sh", true),
            ("bash .claude/statusline-command.sh", true),
            // A user-level path contains the same substring but is NOT this
            // repo's script — vendoring there would shadow the user's copy.
            ("bash ~/.claude/statusline-command.sh", false),
            ("bash /home/x/.claude/statusline-command.sh", false),
            ("echo hi", false),
        ];
        for (command, expected) in oracle {
            write(command);
            assert_eq!(statusline_opt_in(dir.path()), *expected, "{command:?}");
        }
        // An unparseable / unexpected settings shape simply means "not opted
        // in" (fail-safe, never a throw).
        std::fs::write(&settings, "{not json").unwrap();
        assert!(!statusline_opt_in(dir.path()));
        std::fs::write(&settings, r#"{"statusLine":"a string"}"#).unwrap();
        assert!(!statusline_opt_in(dir.path()));
    }

    #[test]
    fn codex_statusline_insertion_respects_an_existing_tui_section() {
        assert_eq!(
            codex_statusline_next_text("[tui]\nfoo = 1\n"),
            format!("[tui]\n{}\nfoo = 1\n", CODEX_STATUS_LINE_BLOCK.trim_end())
        );
        assert_eq!(
            codex_statusline_next_text("model = \"x\"\n"),
            format!("model = \"x\"\n\n[tui]\n{CODEX_STATUS_LINE_BLOCK}")
        );
        assert_eq!(
            codex_statusline_next_text("model = \"x\""),
            format!("model = \"x\"\n\n[tui]\n{CODEX_STATUS_LINE_BLOCK}")
        );
    }

    #[test]
    fn status_line_key_detection() {
        assert!(has_status_line_key("[tui]\n  status_line = [\"a\"]\n"));
        assert!(has_status_line_key("status_line=[]\n"));
        assert!(!has_status_line_key("# status_line = x\n"));
        assert!(!has_status_line_key("status_line_use_colors = true\n"));
    }
}
