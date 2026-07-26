//! adapter — Rust port of `.bee/bin/hooks/adapter.mjs` (rust-port-7, CONTEXT.md
//! D2/D7). `.bee/bin/hooks/adapter.mjs` is FROZEN for the duration of the
//! rust-port feature (D1) — this module is conformance-checked against it
//! (`crates/queen-bee/tests/hook_conformance.rs`), never edited to "improve"
//! on it. Every public item here mirrors an exported function/const from the
//! mjs source; see that file's own header comment for the responsibilities
//! (stdin normalization, root discovery, output encoding, fail-open
//! crash/coverage-gap logging).
//!
//! One deliberate API shape difference from the mjs source, noted here so it
//! is never "fixed" back by accident: [`read_hook_context`] takes the raw
//! stdin text as a parameter instead of reading `process.stdin` itself. The
//! observable hook behavior (exit code, stdout, log lines) is unchanged —
//! `queen-bee`'s `main.rs` reads stdin exactly once, the same way the mjs
//! wrappers do, and passes it in. This keeps the function pure and directly
//! testable without process plumbing.

use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

/// Events whose non-empty stdout must be a parseable JSON systemMessage —
/// mirrors `adapter.mjs`'s `ADVISORY_EVENTS`.
pub const ADVISORY_EVENTS: [&str; 3] = ["PreCompact", "SubagentStop", "Stop"];

const SOURCE_IDENTITIES: [&str; 2] = ["plugin", "repo"];
const DETAIL_MAX: usize = 300;

fn truncate_detail(text: &str) -> String {
    let count = text.chars().count();
    if count <= DETAIL_MAX {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(DETAIL_MAX).collect();
        format!("{truncated}...")
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Mirrors `new Date().toISOString()` at the call site — reuses bee-core's
/// already-published `iso8601_millis` (D2, "use bee-core's existing
/// published API only") rather than re-deriving the same civil-date math.
pub fn timestamp_now() -> String {
    bee_core::lock::iso8601_millis(now_ms())
}

// --- source identity ---------------------------------------------------

/// Port of `parseSourceIdentity`. Returns `(source, invalid)`: `source` is
/// `Some("plugin"|"repo")` on a recognized value, `invalid` carries the raw
/// (or `"<missing>"`) value when a `--source`/`--source=` flag was present
/// but not one of the two recognized identities. Scans in argv order and
/// stops at the FIRST `--source`/`--source=` occurrence, exactly like the
/// mjs `for` loop's early return.
pub fn parse_source_identity(argv: &[String]) -> (Option<String>, Option<String>) {
    for (i, arg) in argv.iter().enumerate() {
        let value = if let Some(v) = arg.strip_prefix("--source=") {
            v.trim().to_string()
        } else if arg == "--source" {
            argv.get(i + 1).map(|s| s.trim().to_string()).unwrap_or_default()
        } else {
            continue;
        };
        if SOURCE_IDENTITIES.contains(&value.as_str()) {
            return (Some(value), None);
        }
        let invalid = if value.is_empty() { "<missing>".to_string() } else { value };
        return (None, Some(invalid));
    }
    (None, None)
}

// --- root discovery ------------------------------------------------------

fn resolve_abs(p: &Path) -> PathBuf {
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(p)
    };
    normalize_lexical(&joined)
}

/// `path.resolve`-shaped lexical normalization: collapses `.`/`..`
/// components WITHOUT touching the filesystem (no symlink resolution) —
/// matches node's `path.resolve`/`path.join` semantics, as distinct from
/// `realpath` which the adapter calls separately where the mjs source does.
fn normalize_lexical(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn resolve_path(base: &Path, rel: &Path) -> PathBuf {
    let joined = if rel.is_absolute() { rel.to_path_buf() } else { base.join(rel) };
    normalize_lexical(&joined)
}

fn realpath_or_none(p: &Path) -> Option<PathBuf> {
    fs::canonicalize(p).ok()
}

fn read_gitdir_file(file: &Path, base: &Path) -> Option<PathBuf> {
    let raw = fs::read_to_string(file).ok()?;
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let raw = raw.strip_prefix("gitdir:").map(str::trim).unwrap_or(raw);
    if raw.is_empty() {
        return None;
    }
    let normalized = raw.replace('\\', &std::path::MAIN_SEPARATOR.to_string());
    Some(resolve_path(base, Path::new(&normalized)))
}

fn locate_git_root(start_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut candidate = resolve_abs(start_dir);
    loop {
        let marker = candidate.join(".git");
        if marker.exists() {
            return Some((candidate, marker));
        }
        match candidate.parent() {
            Some(parent) if parent != candidate => candidate = parent.to_path_buf(),
            _ => return None,
        }
    }
}

fn locate_onboarded_root(start_dir: &Path) -> Option<PathBuf> {
    let mut candidate = resolve_abs(start_dir);
    loop {
        if candidate.join(".bee").join("onboarding.json").exists() {
            return Some(candidate);
        }
        match candidate.parent() {
            Some(parent) if parent != candidate => candidate = parent.to_path_buf(),
            _ => return None,
        }
    }
}

/// `worktreeResolution` values mirroring the mjs source's string literals.
pub type WorktreeResolution = &'static str;

/// Non-throwing hook-side twin of `state.mjs`'s `resolveRoots` — port of
/// `adapter.mjs`'s `resolveRoots`. `work_root` remains the physical
/// checkout; `store_root` is what write-guard-style consumers would use for
/// coordination state; `main_root` is set only on the `"linked-valid"`
/// branch.
#[derive(Debug, Clone)]
pub struct Roots {
    pub store_root: Option<PathBuf>,
    pub work_root: Option<PathBuf>,
    pub worktree_resolution: WorktreeResolution,
    pub id: Option<String>,
    pub main_root: Option<PathBuf>,
    pub worktree_root: Option<PathBuf>,
}

impl Roots {
    fn ordinary(root: Option<PathBuf>) -> Self {
        Roots {
            store_root: root.clone(),
            work_root: root,
            worktree_resolution: "ordinary",
            id: None,
            main_root: None,
            worktree_root: None,
        }
    }

    fn linked_invalid(work_root: Option<PathBuf>) -> Self {
        Roots {
            store_root: None,
            work_root,
            worktree_resolution: "linked-invalid",
            id: None,
            main_root: None,
            worktree_root: None,
        }
    }

    fn none() -> Self {
        Roots {
            store_root: None,
            work_root: None,
            worktree_resolution: "ordinary",
            id: None,
            main_root: None,
            worktree_root: None,
        }
    }
}

pub fn resolve_roots(start_dir: &Path) -> Roots {
    let onboarded = locate_onboarded_root(start_dir);
    if let Some(ref ob) = onboarded {
        if !ob.join(".git").exists() {
            return Roots::ordinary(realpath_or_none(ob));
        }
    }

    let Some((work_root_raw, marker)) = locate_git_root(start_dir) else {
        return Roots::ordinary(onboarded.as_deref().and_then(realpath_or_none));
    };

    let Some(work_root) = realpath_or_none(&work_root_raw) else {
        return Roots::none();
    };

    let marker_stat = match fs::metadata(&marker) {
        Ok(m) => m,
        Err(_) => return Roots::linked_invalid(Some(work_root)),
    };
    if !marker_stat.is_file() {
        return Roots::ordinary(Some(work_root));
    }

    let Some(gitdir) = read_gitdir_file(&marker, &work_root_raw) else {
        return Roots::linked_invalid(Some(work_root));
    };

    let worktrees_root = resolve_path(&gitdir, Path::new(".."));
    let common_git_dir = resolve_path(&worktrees_root, Path::new(".."));
    let linked_shape = worktrees_root.file_name() == Some(std::ffi::OsStr::new("worktrees"))
        && common_git_dir.file_name() == Some(std::ffi::OsStr::new(".git"));
    if !linked_shape {
        return Roots::ordinary(Some(work_root));
    }

    let reverse = read_gitdir_file(&gitdir.join("gitdir"), &gitdir);
    let cwd = std::env::current_dir().unwrap_or_default();
    let marker_real = resolve_path(&cwd, &marker);
    let reverse_ok = reverse.as_ref().map(|r| resolve_path(&cwd, r) == marker_real).unwrap_or(false);
    if !reverse_ok {
        return Roots::linked_invalid(Some(work_root));
    }

    let Some(main_root) = common_git_dir.parent().and_then(realpath_or_none) else {
        return Roots::linked_invalid(Some(work_root));
    };

    let id = gitdir.file_name().map(|s| s.to_string_lossy().into_owned());
    let mut store_root = main_root.clone();
    if let Some(ref id_s) = id {
        if let Ok(text) = fs::read_to_string(main_root.join(".bee").join("runtime").join("worktree-grants.json")) {
            if let Ok(Value::Object(grants)) = serde_json::from_str::<Value>(&text) {
                if grants.get(id_s).and_then(Value::as_bool) == Some(true) {
                    store_root = work_root.clone();
                }
            }
        }
    }

    Roots {
        store_root: Some(store_root),
        work_root: Some(work_root.clone()),
        worktree_resolution: "linked-valid",
        id,
        main_root: Some(main_root),
        worktree_root: Some(work_root),
    }
}

pub fn find_repo_root(start_dir: &Path) -> Option<PathBuf> {
    resolve_roots(start_dir).work_root
}

/// Port of `controlRootFor` — `root` is expected to already be a resolved
/// physical checkout path (typically `ctx.root`).
pub fn control_root_for(root: &Path) -> PathBuf {
    let roots = resolve_roots(root);
    roots.main_root.or(roots.work_root).unwrap_or_else(|| root.to_path_buf())
}

// --- fail-open logging -----------------------------------------------------

/// Port of `appendHookLog` — a log failure is swallowed: logging never
/// changes a hook's decision or exit code.
pub fn append_hook_log(root: &Path, entry: &Value) {
    let logs_dir = root.join(".bee").join("logs");
    if fs::create_dir_all(&logs_dir).is_err() {
        return;
    }
    let Ok(mut line) = serde_json::to_string(entry) else {
        return;
    };
    line.push('\n');
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(logs_dir.join("hooks.jsonl")) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn log_crash(root: Option<&Path>, hook_name: &str, error: &str, source: Option<&str>) {
    let Some(root) = root else { return };
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(timestamp_now()));
    entry.insert("hook".into(), Value::String(hook_name.to_string()));
    if let Some(s) = source {
        entry.insert("source".into(), Value::String(s.to_string()));
    }
    entry.insert("error".into(), Value::String(error.to_string()));
    append_hook_log(root, &Value::Object(entry));
}

/// A coverage gap is a host path the hook could not fully cover (malformed
/// payload, unprovable patch target, ...). Logged VISIBLY; the hook proceeds
/// fail-open — the gap line never alters the allow/deny result.
pub fn log_coverage_gap(root: Option<&Path>, hook_name: &str, gap: &str, detail: &str, source: Option<&str>) {
    let Some(root) = root else { return };
    let mut entry = Map::new();
    entry.insert("ts".into(), Value::String(timestamp_now()));
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
    pub payload: Value,
    pub cwd: String,
    pub root: Option<PathBuf>,
    pub store_root: Option<PathBuf>,
    pub control_root: Option<PathBuf>,
    pub worktree_resolution: WorktreeResolution,
    pub source: Option<String>,
    pub event: String,
    pub gaps: Vec<Gap>,
}

fn js_typeof_for_cwd(v: &Value) -> &'static str {
    if v.is_array() {
        "an array"
    } else if v.is_boolean() {
        "boolean"
    } else if v.is_number() {
        "number"
    } else {
        // typeof null === "object" in JS, and a real object also reads
        // "object" — both collapse to the same detail string as the mjs
        // source (Value::String is never reached here: the caller already
        // handles the string case before falling into this helper).
        "object"
    }
}

/// Port of `readHookContext` — NEVER panics. `raw_stdin` is the hook's full
/// stdin text, already read by the caller (see module doc comment for why
/// this takes it as a parameter instead of reading `process.stdin` itself).
pub fn read_hook_context(hook_name: &str, argv: &[String], raw_stdin: &str) -> HookContext {
    let mut gaps: Vec<Gap> = Vec::new();

    let mut payload = Value::Object(Map::new());
    if !raw_stdin.trim().is_empty() {
        match serde_json::from_str::<Value>(raw_stdin) {
            Ok(parsed) => {
                if parsed.is_object() {
                    payload = parsed;
                } else {
                    let kind = match &parsed {
                        Value::Null => "null",
                        Value::Array(_) => "array",
                        Value::String(_) => "string",
                        Value::Number(_) => "number",
                        Value::Bool(_) => "boolean",
                        Value::Object(_) => unreachable!("object handled above"),
                    };
                    gaps.push(Gap {
                        gap: "malformed-payload",
                        detail: format!("top-level {kind} payload — normalized to {{}}"),
                    });
                }
            }
            Err(_) => gaps.push(Gap {
                gap: "malformed-payload",
                detail: "stdin is not parseable JSON — normalized to {}".to_string(),
            }),
        }
    }

    let mut cwd = std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
    if let Some(cwd_val) = payload.get("cwd") {
        match cwd_val.as_str() {
            Some(s) if !s.trim().is_empty() => cwd = s.to_string(),
            Some(_empty) => gaps.push(Gap {
                gap: "invalid-cwd",
                detail: "payload.cwd is string, not a usable string — fell back to process.cwd()".to_string(),
            }),
            None => gaps.push(Gap {
                gap: "invalid-cwd",
                detail: format!(
                    "payload.cwd is {}, not a usable string — fell back to process.cwd()",
                    js_typeof_for_cwd(cwd_val)
                ),
            }),
        }
    }

    let (source, invalid_source) = parse_source_identity(argv);
    if let Some(inv) = &invalid_source {
        gaps.push(Gap {
            gap: "invalid-source",
            detail: format!("--source \"{inv}\" is not plugin|repo — recorded as unknown"),
        });
    }

    let roots = resolve_roots(Path::new(&cwd));
    let root = roots.work_root.clone();
    let control_root = roots.main_root.clone().or_else(|| root.clone());

    if let Some(ref r) = root {
        for g in &gaps {
            log_coverage_gap(Some(r), hook_name, g.gap, &g.detail, source.as_deref());
        }
    }

    let event = payload.get("hook_event_name").and_then(Value::as_str).unwrap_or("").to_string();

    HookContext {
        payload,
        cwd,
        root,
        store_root: roots.store_root,
        control_root,
        worktree_resolution: roots.worktree_resolution,
        source,
        event,
        gaps,
    }
}

// --- output encoding -------------------------------------------------------

pub fn is_advisory_event(event: &str) -> bool {
    ADVISORY_EVENTS.contains(&event)
}

/// Port of `encodeAdvisory`. NEVER `decision:"block"` — see the mjs source's
/// header comment for why (a block on SubagentStop/Stop is NOT advisory).
pub fn encode_advisory(text: &str) -> String {
    serde_json::to_string(&serde_json::json!({ "systemMessage": text })).expect("json serialize")
}

/// Port of `encodeBlock` — deliberate inverse of [`encode_advisory`];
/// callers must restrict this to a `Stop` event only (see mjs source).
pub fn encode_block(reason: &str) -> String {
    serde_json::to_string(&serde_json::json!({ "decision": "block", "reason": reason })).expect("json serialize")
}

/// Port of `emitHookOutput` — returns what the mjs source would have
/// written to stdout, or `None` for an empty/whitespace-only message (the
/// mjs source writes nothing in that case).
pub fn emit_hook_output(event: &str, text: &str) -> Option<String> {
    if text.trim().is_empty() {
        return None;
    }
    if is_advisory_event(event) {
        Some(encode_advisory(text))
    } else {
        Some(text.to_string())
    }
}
