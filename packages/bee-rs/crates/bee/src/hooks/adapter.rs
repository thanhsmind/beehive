// hooks/adapter — Rust port of hooks/adapter.mjs: the shared fail-open
// substrate every bee hook stands on. Responsibilities and ordering match
// the .mjs exactly:
//   1. stdin normalization to a plain object before ANY property access;
//   2. root discovery inside the fail-open boundary (realpath'd, symlinks
//      resolved — the hook flavor differs from the CLI's resolveRoots here);
//   3. per-event output encoding (advisory events emit JSON systemMessage,
//      context events plain stdout; encodeBlock is Stop-only);
//   4. crash/coverage-gap logging to .bee/logs/hooks.jsonl that can NEVER
//      change a hook's decision or exit code.

#![allow(dead_code)]

use crate::jsjson;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub const ADVISORY_EVENTS: [&str; 3] = ["PreCompact", "SubagentStop", "Stop"];
const DETAIL_MAX: usize = 300;

fn truncate_detail(text: &str) -> String {
    if text.chars().count() <= DETAIL_MAX {
        text.to_string()
    } else {
        format!("{}...", text.chars().take(DETAIL_MAX).collect::<String>())
    }
}

// --- source identity -------------------------------------------------------

pub struct SourceIdentity {
    pub source: Option<&'static str>,
    pub invalid: Option<String>,
}

pub fn parse_source_identity(argv: &[String]) -> SourceIdentity {
    for (i, arg) in argv.iter().enumerate() {
        let value = if let Some(v) = arg.strip_prefix("--source=") {
            v.trim().to_string()
        } else if arg == "--source" {
            argv.get(i + 1).map(|s| s.trim().to_string()).unwrap_or_default()
        } else {
            continue;
        };
        return match value.as_str() {
            "plugin" => SourceIdentity { source: Some("plugin"), invalid: None },
            "repo" => SourceIdentity { source: Some("repo"), invalid: None },
            _ => SourceIdentity {
                source: None,
                invalid: Some(if value.is_empty() { "<missing>".to_string() } else { value }),
            },
        };
    }
    SourceIdentity { source: None, invalid: None }
}

// --- root discovery (hook flavor: realpath'd, never throws) ---------------

fn realpath_or_none(p: &Path) -> Option<PathBuf> {
    // fs.realpathSync.native resolves symlinks and returns a drive-letter
    // path on Windows; dunce strips the \\?\ prefix std's canonicalize adds.
    dunce::canonicalize(p).ok()
}

fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = std::fs::read_to_string(file).ok()?;
    let mut raw = raw.trim();
    if let Some(rest) = raw.strip_prefix("gitdir:") {
        raw = rest.trim();
    }
    if raw.is_empty() {
        return None;
    }
    let sep_fixed = if cfg!(windows) { raw.replace('\\', "\\") } else { raw.replace('\\', "/") };
    Some(std::path::absolute(base.join(sep_fixed)).ok()?)
}

fn locate_up(start: &Path, probe: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut dir = std::path::absolute(start).ok()?;
    loop {
        if probe(&dir) {
            return Some(dir);
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}

pub struct HookRoots {
    pub store_root: Option<PathBuf>,
    pub work_root: Option<PathBuf>,
    pub worktree_resolution: &'static str,
    pub main_root: Option<PathBuf>,
}

/// Non-throwing hook-side twin of state.mjs resolveRoots (adapter.mjs
/// flavor: realpath'd roots, inline grants read, fail-open everywhere).
pub fn resolve_roots(start: &Path) -> HookRoots {
    let ordinary_none = || HookRoots {
        store_root: None,
        work_root: None,
        worktree_resolution: "ordinary",
        main_root: None,
    };

    let onboarded = locate_up(start, |d| d.join(".bee").join("onboarding.json").exists());
    if let Some(ob) = &onboarded {
        if !ob.join(".git").exists() {
            let root = realpath_or_none(ob);
            return HookRoots {
                store_root: root.clone(),
                work_root: root,
                worktree_resolution: "ordinary",
                main_root: None,
            };
        }
    }
    let located = locate_up(start, |d| d.join(".git").exists());
    let Some(work_dir) = located else {
        let root = onboarded.as_deref().and_then(realpath_or_none);
        return HookRoots {
            store_root: root.clone(),
            work_root: root,
            worktree_resolution: "ordinary",
            main_root: None,
        };
    };
    let marker = work_dir.join(".git");
    let Some(work_root) = realpath_or_none(&work_dir) else {
        return ordinary_none();
    };
    let linked_invalid = |work_root: PathBuf| HookRoots {
        store_root: None,
        work_root: Some(work_root),
        worktree_resolution: "linked-invalid",
        main_root: None,
    };
    let Ok(marker_stat) = std::fs::symlink_metadata(&marker) else {
        return linked_invalid(work_root);
    };
    if !marker_stat.is_file() {
        return HookRoots {
            store_root: Some(work_root.clone()),
            work_root: Some(work_root),
            worktree_resolution: "ordinary",
            main_root: None,
        };
    }

    let Some(gitdir) = read_gitdir_file(&marker, &work_dir) else {
        return linked_invalid(work_root);
    };
    let worktrees_root = gitdir.parent().map(Path::to_path_buf).unwrap_or_default();
    let common_git_dir = worktrees_root.parent().map(Path::to_path_buf).unwrap_or_default();
    let linked_shape = worktrees_root.file_name().is_some_and(|n| n == "worktrees")
        && common_git_dir.file_name().is_some_and(|n| n == ".git");
    if !linked_shape {
        // `git init --separate-git-dir` also has a .git file — stays ordinary.
        return HookRoots {
            store_root: Some(work_root.clone()),
            work_root: Some(work_root),
            worktree_resolution: "ordinary",
            main_root: None,
        };
    }
    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    let marker_abs = std::path::absolute(&marker).unwrap_or(marker.clone());
    if reverse.as_deref() != Some(marker_abs.as_path()) {
        return linked_invalid(work_root);
    }
    let Some(main_root) = common_git_dir.parent().and_then(realpath_or_none) else {
        return linked_invalid(work_root);
    };
    // Inline grants read (never via lib import): registered id => local
    // store; anything else (missing/invalid registry) => main default.
    let id = gitdir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut store_root = main_root.clone();
    if let Ok(text) = std::fs::read_to_string(main_root.join(".bee").join("runtime").join("worktree-grants.json")) {
        if let Ok(Value::Object(grants)) = serde_json::from_str::<Value>(&text) {
            if grants.get(&id) == Some(&Value::Bool(true)) {
                store_root = work_root.clone();
            }
        }
    }
    HookRoots {
        store_root: Some(store_root),
        work_root: Some(work_root),
        worktree_resolution: "linked-valid",
        main_root: Some(main_root),
    }
}

// --- fail-open logging -----------------------------------------------------

pub fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Append to .bee/logs/hooks.jsonl; failures swallowed — logging never
/// changes a hook's decision or exit code.
pub fn append_hook_log(root: &Path, entry: &Value) {
    let logs_dir = root.join(".bee").join("logs");
    if std::fs::create_dir_all(&logs_dir).is_err() {
        return;
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs_dir.join("hooks.jsonl"))
    {
        let _ = f.write_all(format!("{}\n", jsjson::stringify(entry)).as_bytes());
    }
}

pub fn log_crash(root: Option<&Path>, hook_name: &str, error: &str, source: Option<&str>) {
    let Some(root) = root else { return };
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(now_iso()));
    entry.insert("hook".into(), Value::String(hook_name.to_string()));
    if let Some(s) = source {
        entry.insert("source".into(), Value::String(s.to_string()));
    }
    entry.insert("error".into(), Value::String(error.to_string()));
    append_hook_log(root, &Value::Object(entry));
}

pub fn log_coverage_gap(root: &Path, hook_name: &str, gap: &str, detail: &str, source: Option<&str>) {
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(now_iso()));
    entry.insert("hook".into(), Value::String(hook_name.to_string()));
    entry.insert("event".into(), Value::String("coverage-gap".to_string()));
    entry.insert("gap".into(), Value::String(gap.to_string()));
    entry.insert("detail".into(), Value::String(truncate_detail(detail)));
    if let Some(s) = source {
        entry.insert("source".into(), Value::String(s.to_string()));
    }
    append_hook_log(root, &Value::Object(entry));
}

// --- stdin normalization + context -----------------------------------------

pub struct Gap {
    pub gap: &'static str,
    pub detail: String,
}

pub struct HookContext {
    pub payload: Map<String, Value>,
    pub cwd: PathBuf,
    pub root: Option<PathBuf>,
    pub store_root: Option<PathBuf>,
    pub control_root: Option<PathBuf>,
    pub worktree_resolution: &'static str,
    pub source: Option<&'static str>,
    pub event: String,
    pub gaps: Vec<Gap>,
}

/// The one entry point every hook calls first. Never fails. `raw` is the
/// full stdin, read ONCE by the hook dispatcher (hooks/mod.rs) so a native
/// hook that bails can re-feed the identical bytes to the Node wrapper.
pub fn read_hook_context(hook_name: &str, argv: &[String], raw: &str) -> HookContext {
    let mut gaps: Vec<Gap> = Vec::new();
    let mut payload: Map<String, Value> = Map::new();
    if !raw.trim().is_empty() {
        match serde_json::from_str::<Value>(&raw) {
            Ok(Value::Object(m)) => payload = m,
            Ok(parsed) => {
                let kind = match parsed {
                    Value::Null => "null",
                    Value::Array(_) => "array",
                    Value::Bool(_) => "boolean",
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Object(_) => unreachable!(),
                };
                gaps.push(Gap {
                    gap: "malformed-payload",
                    detail: format!("top-level {kind} payload — normalized to {{}}"),
                });
            }
            Err(_) => gaps.push(Gap {
                gap: "malformed-payload",
                detail: "stdin is not parseable JSON — normalized to {}".to_string(),
            }),
        }
    }

    let mut cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match payload.get("cwd") {
        Some(Value::String(s)) if !s.trim().is_empty() => cwd = PathBuf::from(s),
        None => {}
        Some(other) => {
            let kind = match other {
                Value::Array(_) => "an array",
                Value::Null => "object", // typeof null === "object" in JS
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string", // present but empty/whitespace
                Value::Object(_) => "object",
            };
            gaps.push(Gap {
                gap: "invalid-cwd",
                detail: format!("payload.cwd is {kind}, not a usable string — fell back to process.cwd()"),
            });
        }
    }

    let parsed_source = parse_source_identity(argv);
    if let Some(invalid) = &parsed_source.invalid {
        gaps.push(Gap {
            gap: "invalid-source",
            detail: format!("--source \"{invalid}\" is not plugin|repo — recorded as unknown"),
        });
    }

    let roots = resolve_roots(&cwd);
    let root = roots.work_root.clone();
    let control_root = roots.main_root.clone().or_else(|| root.clone());

    if let Some(r) = &root {
        for g in &gaps {
            log_coverage_gap(r, hook_name, g.gap, &g.detail, parsed_source.source);
        }
    }

    let event = payload
        .get("hook_event_name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    HookContext {
        payload,
        cwd,
        root,
        store_root: roots.store_root,
        control_root,
        worktree_resolution: roots.worktree_resolution,
        source: parsed_source.source,
        event,
        gaps,
    }
}

// --- output encoding -------------------------------------------------------

pub fn is_advisory_event(event: &str) -> bool {
    ADVISORY_EVENTS.contains(&event)
}

pub fn encode_advisory(text: &str) -> String {
    jsjson::stringify(&serde_json::json!({ "systemMessage": text }))
}

/// Stop-event BLOCK — callers must restrict to ctx.event == "Stop".
pub fn encode_block(reason: &str) -> String {
    jsjson::stringify(&serde_json::json!({ "decision": "block", "reason": reason }))
}

/// Advisory events get JSON systemMessage; context events plain stdout.
pub fn emit_hook_output(ctx: &HookContext, text: &str, default_event: &str) {
    if text.trim().is_empty() {
        return;
    }
    let event = if ctx.event.is_empty() { default_event } else { &ctx.event };
    use std::io::Write;
    let out = if is_advisory_event(event) { encode_advisory(text) } else { text.to_string() };
    let _ = std::io::stdout().write_all(out.as_bytes());
}
