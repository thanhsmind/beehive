// the pure guards.mjs ports and the command tokenizer
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── guards.mjs pure ports ─────────────────────────────────────────────────

/// provenance: guards.mjs normalizeRel.
pub(crate) fn normalize_rel(rel: &str) -> String {
    let s = rel.replace('\\', "/");
    if s.starts_with("./") {
        s[1..].trim_start_matches('/').to_string()
    } else {
        s
    }
}

pub(crate) const GATE_ALLOWED_PREFIXES: [&str; 4] = [".bee/", "docs/", "plans/", "AGENTS.md"];

/// provenance: guards.mjs underAllowedPrefix.
pub(crate) fn under_allowed_prefix(rel: &str) -> bool {
    let normalized = normalize_rel(rel);
    GATE_ALLOWED_PREFIXES.iter().any(|prefix| {
        if let Some(bare) = prefix.strip_suffix('/') {
            normalized == bare || normalized.starts_with(prefix)
        } else {
            normalized == *prefix
        }
    })
}

/// provenance: guards.mjs DIRECT_EDIT_DENY. E2 (guard-hardening) extends the
/// set to the cells/lanes stores and the onboarding marker.
pub(crate) fn direct_edit_verb(normalized: &str) -> Option<&'static str> {
    // E2: whole CLI-owned stores — any .json under the prefix is owned.
    let prefix_json =
        |prefix: &str| normalized.starts_with(prefix) && normalized.ends_with(".json");
    if prefix_json(".bee/cells/") {
        return Some("bee cells add/finish");
    }
    if prefix_json(".bee/lanes/") {
        return Some("bee state start-feature --as-lane / bee state set --lane");
    }
    match normalized {
        ".bee/state.json" => Some("bee state set --owner <selected pre-mutation phase>, or the dedicated state gate/worker/scribing-run verb"),
        ".bee/backlog.jsonl" => Some("bee backlog add"),
        "docs/backlog.md" => Some("bee backlog pbi add / bee backlog pbi status / bee backlog pbi amend to change data, or bee backlog render --write to regenerate the view"),
        ".bee/runtime/cross-worktree-holds.json" => Some("bee reservations reserve/release (holds are mirrored into the ledger automatically)"),
        ".bee/runtime/worktree-grants.json" => Some("bee worktree register / unregister"),
        ".bee/companion-session.json" => Some("bee worktree new --with-companion (started/ended automatically by the companion lifecycle)"),
        ".bee/onboarding.json" => Some("bee onboard"),
        _ => None,
    }
}

/// provenance: guards.mjs HISTORY_CODE_EXTENSIONS — also the "is this a code
/// file" test for the credentials-prefix secret arm and the scratch
/// verdict-/probe-/digest- prefix rule, both of which must not fire on a
/// real source file that merely starts with a matching word.
pub(crate) const CODE_FILE_EXTENSIONS: [&str; 22] = [
    ".sh", ".bash", ".zsh", ".fish", ".ps1", ".bat", ".cmd", ".mjs", ".cjs", ".js", ".jsx", ".ts",
    ".tsx", ".py", ".rb", ".go", ".rs", ".java", ".php", ".pl", ".lua", ".r",
];

/// provenance: guards.mjs docsHistoryCodeDeny.
pub(crate) fn docs_history_code_deny(normalized: &str) -> Option<String> {
    if !normalized.starts_with("docs/history/") {
        return None;
    }
    let dot = normalized.rfind('.')?;
    let ext = normalized[dot..].to_lowercase();
    if CODE_FILE_EXTENSIONS.contains(&ext.as_str()) {
        Some(ext)
    } else {
        None
    }
}

/// provenance: guards.mjs SCRATCH_* / scratchShapeDeny.
pub(crate) fn scratch_shape_deny(normalized: &str) -> Option<String> {
    let under_any = |prefixes: &[&str]| {
        prefixes.iter().any(|p| normalized == &p[..p.len() - 1] || normalized.starts_with(p))
    };
    if under_any(&[".bee/tmp/", ".bee/spikes/", ".bee/logs/", ".bee/workers/"]) {
        return None;
    }
    if normalized == ".bee/decisions.jsonl" {
        return None;
    }
    if under_any(&[
        "docs/", ".bee/cells/", ".claude-plugin/skills/", ".codex-plugin/skills/",
        ".claude/skills/", ".agents/skills/",
    ]) {
        return None;
    }
    let basename = &normalized[normalized.rfind('/').map(|i| i + 1).unwrap_or(0)..];
    let lower = basename.to_lowercase();
    // SCRATCH_DOTFILE_RE: ^\.[^/]*(?:debug|stress|scratch)[^/]*$ (i)
    if lower.starts_with('.')
        && (lower.contains("debug") || lower.contains("stress") || lower.contains("scratch"))
    {
        return Some("a dotfile named like a debug/stress/scratch script".to_string());
    }
    // SCRATCH_PREFIX_RE: ^(?:verdict|probe|digest)- (i), exempted when the
    // extension marks it a real source file rather than a scratch payload.
    if lower.starts_with("verdict-") || lower.starts_with("probe-") || lower.starts_with("digest-") {
        let is_code_file = lower
            .rfind('.')
            .map(|dot| CODE_FILE_EXTENSIONS.contains(&&lower[dot..]))
            .unwrap_or(false);
        if !is_code_file {
            return Some("a verdict-/probe-/digest- style scratch payload".to_string());
        }
    }
    // SCRATCH_EXT_RE: \.(tmp|log|bak)$ (i), exempted in test/fixture dirs.
    let ext_hit = [".tmp", ".log", ".bak"].iter().any(|e| lower.ends_with(e));
    if ext_hit && !in_test_fixture_dir(normalized) {
        let dot = basename.rfind('.').unwrap();
        return Some(format!("a {} scratch file", &basename[dot..]));
    }
    None
}

/// provenance: guards.mjs TEST_FIXTURE_DIR_RE — (^|/)(test|tests|__tests__|
/// fixtures|__fixtures__|testdata|examples)(/|$) case-insensitive.
pub(crate) fn in_test_fixture_dir(normalized: &str) -> bool {
    const NAMES: [&str; 7] = ["test", "tests", "__tests__", "fixtures", "__fixtures__", "testdata", "examples"];
    let lower = normalized.to_lowercase();
    let segments: Vec<&str> = lower.split('/').collect();
    segments.iter().any(|s| NAMES.contains(s))
}

/// provenance: guards.mjs SECRET_PATTERNS + checkRead privacy half.
pub(crate) fn is_secret_path(normalized: &str) -> bool {
    let lower = normalized.to_lowercase();
    let base = &lower[lower.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0)..];
    // /(^|[\\/])\.env(\.[A-Za-z0-9._-]+)?$/i
    if base == ".env" {
        return true;
    }
    if let Some(rest) = base.strip_prefix(".env.") {
        if !rest.is_empty()
            && rest.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        {
            return true;
        }
    }
    if lower.ends_with(".pem") || lower.ends_with(".key") || lower.ends_with(".p12") {
        return true;
    }
    if base.starts_with("id_rsa") {
        return true;
    }
    // "credentials*" is secret unless the extension marks it a real source
    // file (src/credentials.rs, credentials_test.go, credentials.py, ...).
    if base.starts_with("credentials") {
        let is_code_file = base
            .rfind('.')
            .map(|dot| CODE_FILE_EXTENSIONS.contains(&&base[dot..]))
            .unwrap_or(false);
        if !is_code_file {
            return true;
        }
    }
    // /(^|[\\/])secrets\.[^\\/]+$/i
    if let Some(rest) = base.strip_prefix("secrets.") {
        if !rest.is_empty() {
            return true;
        }
    }
    false
}

pub(crate) const SCOUT_DIRS: [&str; 8] = [
    "node_modules/", "dist/", "build/", ".git/objects", "vendor/", "coverage/", ".next/",
    "__pycache__/",
];

pub(crate) enum ReadVerdict {
    Allow,
    Deny { reason: String, marker: Option<String> },
}

/// provenance: guards.mjs checkRead.
pub(crate) fn check_read(rel: &str) -> ReadVerdict {
    let normalized = normalize_rel(rel);
    if is_secret_path(&normalized) {
        let question = format!(
            "\"{}\" looks like a secret/credential file. Ask the user before reading it.",
            normalized
        );
        let mut obj = Map::new();
        obj.insert("file".into(), Value::String(normalized.clone()));
        obj.insert("question".into(), Value::String(question.clone()));
        let marker = format!("@@BEE_PRIVACY@@{}@@END@@", jsjson::stringify(&Value::Object(obj)));
        return ReadVerdict::Deny {
            reason: format!("bee privacy guard: {}", question),
            marker: Some(marker),
        };
    }
    if let Some(hit) = SCOUT_DIRS
        .iter()
        .find(|dir| normalized.starts_with(*dir) || normalized.contains(&format!("/{}", dir)))
    {
        return ReadVerdict::Deny {
            reason: format!(
                "bee scout guard: \"{}\" is inside \"{}\" — generated/vendored content. Read the source or lockfile instead.",
                normalized, hit
            ),
            marker: None,
        };
    }
    ReadVerdict::Allow
}

// ─── tokenizer (provenance: hooks/tokenize-command.mjs == guards.mjs
// tokenize — byte-identical algorithm, hand-synced there, single port here) ──

pub(crate) fn tokenize(command: &str) -> Vec<String> {
    let chars: Vec<char> = command.chars().collect();
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut has_current = false;
    let mut i = 0usize;
    macro_rules! flush {
        () => {
            if has_current {
                tokens.push(std::mem::take(&mut current));
                #[allow(unused_assignments)]
                {
                    has_current = false;
                }
            }
        };
    }
    while i < chars.len() {
        let ch = chars[i];
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            flush!();
            i += 1;
            continue;
        }
        if ch == '\\' && i + 1 < chars.len() {
            current.push(chars[i + 1]);
            has_current = true;
            i += 2;
            continue;
        }
        if ch == '"' || ch == '\'' {
            let close = chars[i + 1..].iter().position(|&c| c == ch).map(|p| p + i + 1);
            let end = close.unwrap_or(chars.len());
            for &c in &chars[i + 1..end] {
                current.push(c);
            }
            has_current = true;
            i = end + 1;
            continue;
        }
        if (ch == '&' && chars.get(i + 1) == Some(&'&')) || (ch == '|' && chars.get(i + 1) == Some(&'|')) {
            flush!();
            tokens.push(format!("{}{}", ch, ch));
            i += 2;
            continue;
        }
        if ch == ';' || ch == '&' || ch == '|' {
            flush!();
            tokens.push(ch.to_string());
            i += 1;
            continue;
        }
        current.push(ch);
        has_current = true;
        i += 1;
    }
    flush!();
    tokens
}

pub(crate) fn is_separator(t: &str) -> bool {
    matches!(t, "&&" | "||" | ";" | "|" | "&")
}

// ─── deep tokenizer: sees through sh/bash/dash/zsh/ksh -c and eval wrappers ─
//
// `tokenize` folds a quoted span into one token and stops there — it never
// learns that `sh -c '<payload>'` or `eval '<payload>'` hands that payload to
// the shell to execute. `tokenize_deep` re-tokenizes and splices in every
// wrapper payload it finds, so every consumer that reasons over the token
// list (git classification, bash target extraction, the node inline-eval
// detector) sees the commands a wrapper hides.
//
// A wrapper is either a token whose BASENAME equals sh/bash/dash/zsh/ksh
// immediately followed by `-c` and one payload token, or the token `eval`
// followed by its arguments up to the next separator (joined the way a real
// shell joins eval's argv before re-parsing it). The payload is tokenized
// and spliced in fenced by `;` on both sides, so a wrapper can never merge
// two separator-delimited segments into one — `a && sh -c 'b; c' && d`
// still keeps `a` and `d` as their own segments.
//
// Recursion is bounded: a wrapper is only unwrapped `WRAPPER_MAX_DEPTH`
// levels deep. A wrapper nested past the bound is left as an unexpanded,
// still-quoted token and `truncated` is set — every consumer treats that as
// an undecidable payload and fails open (delegates) rather than silently
// allowing a command it could not fully see through.

pub(crate) const WRAPPER_MAX_DEPTH: usize = 4;

const WRAPPER_SHELL_NAMES: [&str; 5] = ["sh", "bash", "dash", "zsh", "ksh"];

fn wrapper_basename(token: &str) -> String {
    let normalized = token.replace('\\', "/");
    normalized.rsplit('/').next().unwrap_or("").to_string()
}

fn is_wrapper_shell_name(basename: &str) -> bool {
    WRAPPER_SHELL_NAMES.contains(&basename)
}

/// True when `tokens` (a single separator-delimited segment or a whole
/// token list — separators inside are simply skipped) starts a wrapper shape
/// this module recognizes. Used only to decide whether hitting the depth
/// bound left something genuinely undecided.
fn contains_wrapper_shape(tokens: &[String]) -> bool {
    let mut i = 0usize;
    while i < tokens.len() {
        let token = &tokens[i];
        if is_separator(token) {
            i += 1;
            continue;
        }
        let base = wrapper_basename(token);
        if is_wrapper_shell_name(&base)
            && tokens.get(i + 1).map(String::as_str) == Some("-c")
            && tokens.get(i + 2).is_some()
        {
            return true;
        }
        if token == "eval" && tokens.get(i + 1).is_some() && !is_separator(&tokens[i + 1]) {
            return true;
        }
        i += 1;
    }
    false
}

pub(crate) struct DeepTokens {
    pub(crate) tokens: Vec<String>,
    /// True when a wrapper nested past `WRAPPER_MAX_DEPTH` was left
    /// unexpanded — the token list may hide a command no consumer can see.
    pub(crate) truncated: bool,
}

/// `tokenize`, then expand every `sh`/`bash`/`dash`/`zsh`/`ksh -c` and `eval`
/// wrapper payload in place and re-scan, bounded at `WRAPPER_MAX_DEPTH`.
pub(crate) fn tokenize_deep(command: &str) -> DeepTokens {
    let mut truncated = false;
    let tokens = expand_wrappers(tokenize(command), 0, &mut truncated);
    DeepTokens { tokens, truncated }
}

fn expand_wrappers(tokens: Vec<String>, depth: usize, truncated: &mut bool) -> Vec<String> {
    if depth >= WRAPPER_MAX_DEPTH {
        if contains_wrapper_shape(&tokens) {
            *truncated = true;
        }
        return tokens;
    }
    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0usize;
    while i < tokens.len() {
        let token = &tokens[i];
        if is_separator(token) {
            out.push(token.clone());
            i += 1;
            continue;
        }
        let base = wrapper_basename(token);
        if is_wrapper_shell_name(&base) && tokens.get(i + 1).map(String::as_str) == Some("-c") {
            if let Some(payload) = tokens.get(i + 2) {
                let inner = expand_wrappers(tokenize(payload), depth + 1, truncated);
                out.push(";".to_string());
                out.extend(inner);
                out.push(";".to_string());
                i += 3;
                continue;
            }
        }
        if token == "eval" {
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                j += 1;
            }
            if j > i + 1 {
                let payload = tokens[i + 1..j].join(" ");
                let inner = expand_wrappers(tokenize(&payload), depth + 1, truncated);
                out.push(";".to_string());
                out.extend(inner);
                out.push(";".to_string());
                i = j;
                continue;
            }
        }
        out.push(token.clone());
        i += 1;
    }
    out
}

// ─── heredoc fencing ─────────────────────────────────────────────────────
//
// `tokenize` has no notion of a heredoc: `<<EOF ... EOF` hands the shell a
// multi-line BODY that is not command syntax at all — it is data the target
// command reads on stdin. Without fencing, a body word lands on the same
// tee/mv/cp/redirect branches that classify real command arguments, and the
// guard denies real work naming a word that only ever appeared as heredoc
// CONTENT (e.g. the bare word "it" inside a `git commit -F - <<'EOF'` body).
//
// `fence_heredocs` runs before tokenization and removes every heredoc body
// (through its terminator line) from the text, leaving every token outside
// a body — including a real redirect that sits next to the operator, as in
// `cat > out.txt <<EOF` — byte-identical to what `tokenize` would have seen
// without this pass. `<<<` (here-string) is a single token already inert
// for target extraction and is left untouched; only `<<` and `<<-` are
// heredoc operators.
pub(crate) fn fence_heredocs(command: &str) -> String {
    let chars: Vec<char> = command.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(command.len());
    let mut i = 0usize;
    // Pending heredoc terminators for the current line, in operator order —
    // bash queues bodies after the newline in the order the operators
    // appeared on the command line.
    let mut pending: Vec<(String, bool)> = Vec::new();

    while i < n {
        let ch = chars[i];
        if ch == '"' || ch == '\'' {
            let close = chars[i + 1..].iter().position(|&c| c == ch).map(|p| p + i + 1);
            let end = close.map(|e| e + 1).unwrap_or(n);
            out.extend(&chars[i..end]);
            i = end;
            continue;
        }
        if ch == '\\' && i + 1 < n {
            out.push(ch);
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if ch == '<' {
            // Count the run of consecutive `<` so a here-string (`<<<`, or
            // longer) is consumed whole rather than re-scanned two chars at
            // a time — otherwise the trailing pair of a 3-`<` run would be
            // mistaken for a fresh `<<` heredoc operator.
            let mut run_end = i;
            while run_end < n && chars[run_end] == '<' {
                run_end += 1;
            }
            let run_len = run_end - i;
            if run_len != 2 {
                out.extend(&chars[i..run_end]);
                i = run_end;
                continue;
            }
            let op_start = i;
            let mut j = i + 2;
            let strip_tabs = chars.get(j) == Some(&'-');
            if strip_tabs {
                j += 1;
            }
            while j < n && (chars[j] == ' ' || chars[j] == '\t') {
                j += 1;
            }
            let mut term = String::new();
            let mut has_term = false;
            if j < n && (chars[j] == '\'' || chars[j] == '"') {
                let q = chars[j];
                let close = chars[j + 1..].iter().position(|&c| c == q).map(|p| p + j + 1);
                let end = close.unwrap_or(n);
                term = chars[j + 1..end.min(n)].iter().collect();
                j = if end < n { end + 1 } else { n };
                has_term = true;
            } else {
                let start = j;
                while j < n && !chars[j].is_whitespace() && !matches!(chars[j], ';' | '&' | '|') {
                    j += 1;
                }
                if j > start {
                    term = chars[start..j].iter().collect();
                    has_term = true;
                }
            }
            // The operator and its terminator word ride through unchanged —
            // `<<` itself is already inert for redirect matching, so leaving
            // it in the token stream cannot create a spurious target.
            out.extend(&chars[op_start..j]);
            i = j;
            if has_term {
                pending.push((term, strip_tabs));
            }
            continue;
        }
        if ch == '\n' {
            out.push('\n');
            i += 1;
            if !pending.is_empty() {
                let ops = std::mem::take(&mut pending);
                for (term, strip_tabs) in ops {
                    loop {
                        if i >= n {
                            // Unterminated heredoc: fail safe — nothing left
                            // to consume, nothing left to extract from.
                            break;
                        }
                        let line_start = i;
                        let mut k = i;
                        while k < n && chars[k] != '\n' {
                            k += 1;
                        }
                        let line: String = chars[line_start..k].iter().collect();
                        let compare: &str =
                            if strip_tabs { line.trim_start_matches('\t') } else { &line };
                        let is_term_line = compare == term;
                        let ran_out = k >= n;
                        i = if ran_out { k } else { k + 1 };
                        if is_term_line || ran_out {
                            break;
                        }
                    }
                }
            }
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

// ─── bash target extraction (provenance: guards.mjs extractBashTargets) ────

pub(crate) struct BashTargets {
    pub(crate) paths: Vec<String>,
    pub(crate) broad_write: bool,
}

pub(crate) fn is_flag(t: &str) -> bool {
    t.starts_with('-')
}

/// provenance: guards.mjs isBroad + BROAD_TARGETS.
pub(crate) fn is_broad(target: &str) -> bool {
    const BROAD: [&str; 7] = [".", "..", "/", "~", "*", "./*", "/*"];
    let normalized = normalize_rel(target);
    BROAD.contains(&target)
        || BROAD.contains(&normalized.as_str())
        || normalized.ends_with("/*")
        || normalized.ends_with("/.")
        || normalized == "*"
}

/// provenance: guards.mjs hasGitShortFlag.
pub(crate) fn has_git_short_flag(tokens: &[String], letter: char) -> bool {
    tokens.iter().any(|t| {
        t.len() >= 2
            && t.starts_with('-')
            && !t[1..].is_empty()
            && t[1..].chars().all(|c| c.is_ascii_alphabetic())
            && t[1..].contains(letter)
    })
}

pub(crate) fn extract_bash_targets(command: &str) -> BashTargets {
    let fenced = fence_heredocs(command);
    let deep = tokenize_deep(&fenced);
    let tokens = deep.tokens;
    let mut paths: Vec<String> = Vec::new();
    // A wrapper nested past the depth bound could hide a redirect we cannot
    // see — treat it as broad/opaque rather than silently finding no target.
    let mut broad_write = deep.truncated;
    let add_target = |target: &str, broad: &mut bool, paths: &mut Vec<String>| {
        if target.is_empty() || target == "/dev/null" || target == "NUL" {
            return;
        }
        if is_broad(target) {
            *broad = true;
        }
        paths.push(target.to_string());
    };
    let mut i = 0usize;
    while i < tokens.len() {
        let token = &tokens[i];
        // Redirection: /^\d?>{1,2}(.*)$/ — fd-duplication (&-target) excluded.
        if let Some(inline) = match_redirect(token) {
            if !inline.is_empty() {
                if !inline.starts_with('&') {
                    add_target(&inline, &mut broad_write, &mut paths);
                }
            } else if let Some(next) = tokens.get(i + 1) {
                if !is_separator(next) && !next.starts_with('&') {
                    add_target(next, &mut broad_write, &mut paths);
                    i += 1;
                }
            }
            i += 1;
            continue;
        }
        if is_separator(token) {
            i += 1;
            continue;
        }
        let cmd = token.replace('\\', "/");
        let cmd = cmd.rsplit('/').next().unwrap_or("");
        if cmd == "git" && matches!(tokens.get(i + 1).map(String::as_str), Some("add") | Some("mv") | Some("rm")) {
            let git_verb = tokens[i + 1].clone();
            let mut end = i + 2;
            while end < tokens.len() && !is_separator(&tokens[end]) {
                end += 1;
            }
            let segment: Vec<String> = tokens[i + 2..end].to_vec();
            if git_verb == "add"
                && (segment.iter().any(|t| t == "--all")
                    || segment.iter().any(|t| t == "--update")
                    || has_git_short_flag(&segment, 'A')
                    || has_git_short_flag(&segment, 'u'))
            {
                broad_write = true;
            }
            for t in &segment {
                if !is_flag(t) {
                    // D8: staging a CLI-owned file with `git add` is not a
                    // direct-edit target.
                    let cli_owned_stage_only =
                        git_verb == "add" && direct_edit_verb(&normalize_rel(t)).is_some();
                    if !cli_owned_stage_only {
                        add_target(t, &mut broad_write, &mut paths);
                    }
                }
            }
            i = end;
            continue;
        }
        if cmd == "git" && tokens.get(i + 1).map(String::as_str) == Some("commit") {
            let mut end = i + 2;
            while end < tokens.len() && !is_separator(&tokens[end]) {
                end += 1;
            }
            let segment: Vec<String> = tokens[i + 2..end].to_vec();
            if segment.iter().any(|t| t == "--all") || has_git_short_flag(&segment, 'a') {
                broad_write = true;
            }
            i = end;
            continue;
        }
        if cmd == "sed" {
            let mut in_place = false;
            let mut last = i;
            let mut args: Vec<String> = Vec::new();
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                if tokens[j].starts_with("-i") {
                    in_place = true;
                } else if !is_flag(&tokens[j]) {
                    args.push(tokens[j].clone());
                }
                last = j;
                j += 1;
            }
            if in_place {
                for file in args.iter().skip(1) {
                    add_target(file, &mut broad_write, &mut paths);
                }
            }
            i = last + 1;
            continue;
        }
        if matches!(cmd, "rm" | "mv" | "cp" | "mkdir" | "touch" | "tee") {
            let mut saw_any = false;
            let mut last = i;
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                if !is_flag(&tokens[j]) {
                    add_target(&tokens[j], &mut broad_write, &mut paths);
                    saw_any = true;
                }
                last = j;
                j += 1;
            }
            if cmd == "rm" && !saw_any {
                broad_write = true;
            }
            i = last + 1;
            continue;
        }
        i += 1;
    }
    BashTargets { paths, broad_write }
}

/// /^\d?>{1,2}(.*)$/ → Some(captured tail) when the token is a redirect.
pub(crate) fn match_redirect(token: &str) -> Option<String> {
    let chars: Vec<char> = token.chars().collect();
    let mut idx = 0usize;
    if idx < chars.len() && chars[idx].is_ascii_digit() {
        // optional single digit — but only if a '>' follows it or starts the token
        if chars.get(idx + 1) == Some(&'>') {
            idx += 1;
        } else {
            return None;
        }
    }
    if chars.get(idx) != Some(&'>') {
        return None;
    }
    idx += 1;
    if chars.get(idx) == Some(&'>') {
        idx += 1;
    }
    Some(chars[idx..].iter().collect())
}
