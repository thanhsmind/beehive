// bee dev render-skill-trees — Rust port of
// scripts/render_plugin_skill_trees.mjs, plus the four renderer primitives it
// imports from packages/bee/scripts/onboard_bee.mjs.
//
// PROVENANCE. Every function below names its .mjs source. The onboard_bee.mjs
// half (`RENDER_RUNTIMES`, `RENDER_SCHEMA`, `RENDER_SIDECAR`,
// `validateSkillMarkers`, `renderSkillBytes`, `manifestFingerprint`,
// `skillDigest`, `buildRenderSidecar`) is RE-DERIVED here rather than shared:
// the onboard port lands in its own `onboard/` module, which this port may
// not edit, and a dev tool that cannot regenerate the committed trees on its
// own is not a port. Both ports answer to the same .mjs; `render_matches_the_
// committed_trees` below re-renders the REAL skills/ tree and byte-compares
// it against the committed projections, which is the pin that catches either
// port drifting.
//
// WHAT IT DOES. Regenerates the two committed plugin skill-route trees:
//   .claude-plugin/skills/ = render(canonical skills/, "claude")
//   .codex-plugin/skills/  = render(canonical skills/, "codex")
// Whole-tree marker grammar is validated BEFORE any write; each tree is
// rendered into a tmp sibling and swapped in via two renames, under ONE
// withStoreLock(repo, "plugin-render") critical section.
//
// STRANGLER ROUTING. Takes no arguments (neither does the .mjs), so any
// argument returns None. Every failure detectable BEFORE output — a symlink
// in the skill source, an unsupported dirent, an unreadable file, a source
// tree that isn't there — also returns None rather than inventing bytes for
// what Node reports as a V8 stack. Once the lock is held and writing has
// begun, a failure prints `render_plugin_skill_trees: <message>` and exits 1
// where Node prints the same message followed by `at` frames (see the
// devtools/mod.rs header on this documented error-path divergence).

use super::{rel_platform, sha256_hex, sort_by_locale};
use crate::jsjson;
use crate::lock;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// provenance: onboard_bee.mjs RENDER_RUNTIMES.
const RENDER_RUNTIMES: [&str; 2] = ["claude", "codex"];
/// provenance: onboard_bee.mjs RENDER_SCHEMA (bee-render/2, D7).
const RENDER_SCHEMA: &str = "bee-render/2";
/// provenance: onboard_bee.mjs RENDER_SIDECAR.
const RENDER_SIDECAR: &str = ".bee-render.json";
/// provenance: render_plugin_skill_trees.mjs TMP_STALE_MS.
const TMP_STALE_MS: f64 = 5.0 * 60.0 * 1000.0;
/// provenance: render_plugin_skill_trees.mjs SWAP_DIR_PREFIXES.
const SWAP_DIR_PREFIXES: [&str; 2] = ["tmp-", "old-"];

/// provenance: render_plugin_skill_trees.mjs TARGET_ROOTS.
fn target_root(root: &Path, runtime: &str) -> PathBuf {
    let dir = if runtime == "claude" { ".claude-plugin" } else { ".codex-plugin" };
    root.join(dir).join("skills")
}

fn source_root(root: &Path) -> PathBuf {
    root.join("skills")
}

/// "Needs Node": a shape this port has not proven — the caller returns None
/// before any output.
#[derive(Debug)]
struct Nd;
type R<T> = Result<T, Nd>;

// ═══ marker grammar (onboard_bee.mjs) ══════════════════════════════════════

/// provenance: onboard_bee.mjs NEAR_MARKER_RE = /^[ \t]*<!--[ \t]*bee:(only|end)\b/
fn is_near_marker(line: &str) -> bool {
    let rest = line.trim_start_matches([' ', '\t']);
    let Some(rest) = rest.strip_prefix("<!--") else { return false };
    let rest = rest.trim_start_matches([' ', '\t']);
    let Some(rest) = rest.strip_prefix("bee:") else { return false };
    for kw in ["only", "end"] {
        if let Some(after) = rest.strip_prefix(kw) {
            // `\b`: the keyword's last char is a word char, so the boundary
            // holds iff the next char is not one (or there is no next char).
            let boundary = after
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'));
            if boundary {
                return true;
            }
        }
    }
    false
}

enum MarkerClass {
    Only(String),
    End,
    Error(String),
}

/// provenance: onboard_bee.mjs classifyMarkerLine, with MARKER_ONLY_RE =
/// /^<!-- bee:only (\S+) -->[ \t]*$/ and MARKER_END_RE = /^<!-- bee:end -->[ \t]*$/.
fn classify_marker_line(line: &str) -> MarkerClass {
    let body = line.trim_end_matches([' ', '\t']);
    if let Some(inner) = body.strip_prefix("<!-- bee:only ") {
        if let Some(label) = inner.strip_suffix(" -->") {
            // `\S+`: at least one char, none of them whitespace.
            if !label.is_empty() && !label.chars().any(char::is_whitespace) {
                if !RENDER_RUNTIMES.contains(&label) {
                    return MarkerClass::Error(format!(
                        "unknown runtime label \"{label}\" (expected {})",
                        RENDER_RUNTIMES.join(" or ")
                    ));
                }
                return MarkerClass::Only(label.to_string());
            }
        }
    }
    if body == "<!-- bee:end -->" {
        return MarkerClass::End;
    }
    MarkerClass::Error(format!(
        "ambiguous near-marker \"{}\" (not an exact full-line bee marker)",
        js_trim(line)
    ))
}

/// provenance: onboard_bee.mjs FRONTMATTER_DELIM_RE = /^---[ \t]*$/.
fn is_frontmatter_delim(line: &str) -> bool {
    line.strip_prefix("---")
        .is_some_and(|rest| rest.chars().all(|c| c == ' ' || c == '\t'))
}

/// provenance: onboard_bee.mjs fenceChar — /^[ \t]*(`{3,}|~{3,})/, first char.
fn fence_char(line: &str) -> Option<char> {
    let rest = line.trim_start_matches([' ', '\t']);
    for c in ['`', '~'] {
        if rest.chars().take(3).filter(|x| *x == c).count() == 3 {
            return Some(c);
        }
    }
    None
}

/// JS `String#trim` (whitespace + BOM).
fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// provenance: `text.split(/\r\n|\n/)`.
fn split_lines(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let end = if i > 0 && bytes[i - 1] == b'\r' { i - 1 } else { i };
            out.push(&text[start..end]);
            start = i + 1;
        }
        i += 1;
    }
    out.push(&text[start..]);
    out
}

/// provenance: onboard_bee.mjs splitLinesPreserving — [content, terminator]
/// pairs whose concatenation rebuilds the input byte for byte.
fn split_lines_preserving(text: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut last = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            let (content_end, term_start) = if i > 0 && bytes[i - 1] == b'\r' {
                (i - 1, i - 1)
            } else {
                (i, i)
            };
            out.push((&text[last..content_end], &text[term_start..i + 1]));
            last = i + 1;
        }
        i += 1;
    }
    out.push((&text[last..], ""));
    out
}

/// provenance: onboard_bee.mjs validateSkillMarkers — whole-file grammar
/// check; an empty result means well-formed (or marker-free).
fn validate_skill_markers(text: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let lines = split_lines(text);

    // Frontmatter span: only when the very first line is a `---` delimiter.
    let mut frontmatter_end: i64 = -1;
    if !lines.is_empty() && is_frontmatter_delim(lines[0]) {
        for (i, line) in lines.iter().enumerate().skip(1) {
            if is_frontmatter_delim(line) {
                frontmatter_end = i as i64;
                break;
            }
        }
    }

    let mut fence: Option<char> = None;
    let mut open_runtime: Option<String> = None;
    let mut open_line: i64 = -1;
    let mut first_frontmatter_opener: i64 = -1;

    for (idx, line) in lines.iter().enumerate() {
        let i = idx as i64;
        if i > frontmatter_end {
            let fc = fence_char(line);
            match fence {
                None => {
                    if let Some(c) = fc {
                        fence = Some(c);
                    }
                }
                Some(open) => {
                    if fc == Some(open) {
                        fence = None;
                    }
                }
            }
        }
        if !is_near_marker(line) {
            if first_frontmatter_opener == -1
                && fence.is_none()
                && is_frontmatter_delim(line)
                && i > 0
                && frontmatter_end == -1
            {
                first_frontmatter_opener = i;
            }
            continue;
        }
        if fence.is_some() {
            errors.push(format!(
                "marker inside a fenced code block at line {}: \"{}\"",
                i + 1,
                js_trim(line)
            ));
            continue;
        }
        if frontmatter_end != -1 && i <= frontmatter_end {
            errors.push(format!(
                "marker inside YAML frontmatter at line {}: \"{}\"",
                i + 1,
                js_trim(line)
            ));
            continue;
        }
        match classify_marker_line(line) {
            MarkerClass::Error(e) => {
                errors.push(format!("{e} at line {}", i + 1));
            }
            MarkerClass::Only(runtime) => {
                if open_runtime.is_some() {
                    errors.push(format!(
                        "nested bee:only block at line {} (block opened at line {} not closed)",
                        i + 1,
                        open_line + 1
                    ));
                    continue;
                }
                open_runtime = Some(runtime);
                open_line = i;
                if first_frontmatter_opener != -1 {
                    errors.push(format!("marker before YAML frontmatter at line {}", i + 1));
                }
            }
            MarkerClass::End => {
                if open_runtime.is_none() {
                    errors.push(format!("stray bee:end with no open block at line {}", i + 1));
                    continue;
                }
                open_runtime = None;
            }
        }
    }
    if open_runtime.is_some() {
        errors.push(format!("unclosed bee:only block opened at line {}", open_line + 1));
    }
    errors
}

/// provenance: onboard_bee.mjs bufHasMarkerBytes — the cheap gate that keeps
/// the no-marker path from ever decoding (the byte-identity guarantee).
fn buf_has_marker_bytes(buf: &[u8]) -> bool {
    fn contains(h: &[u8], n: &[u8]) -> bool {
        h.windows(n.len()).any(|w| w == n)
    }
    contains(buf, b"bee:only") || contains(buf, b"bee:end")
}

/// provenance: onboard_bee.mjs renderSkillBytes — filter one file's bytes for
/// `runtime`. A file with no marker LINE is returned byte-identical (never
/// decoded); a marked file is rebuilt with exact line endings, markers
/// stripped and off-runtime blocks dropped.
fn render_skill_bytes(buf: &[u8], runtime: &str) -> Vec<u8> {
    if !buf_has_marker_bytes(buf) {
        return buf.to_vec();
    }
    let text = String::from_utf8_lossy(buf).into_owned();
    if !split_lines(&text).iter().any(|l| is_near_marker(l)) {
        return buf.to_vec(); // literal "bee:only"/"bee:end" in prose
    }
    let mut out = String::new();
    let mut open_runtime: Option<String> = None;
    for (content, term) in split_lines_preserving(&text) {
        if is_near_marker(content) {
            match classify_marker_line(content) {
                MarkerClass::Only(r) => {
                    open_runtime = Some(r);
                    continue;
                }
                MarkerClass::End => {
                    open_runtime = None;
                    continue;
                }
                // Validation guarantees well-formedness before render is ever
                // reached; a malformed near-marker falls through as content.
                MarkerClass::Error(_) => {}
            }
        }
        if open_runtime.as_deref().is_none_or(|r| r == runtime) {
            out.push_str(content);
            out.push_str(term);
        }
    }
    out.into_bytes()
}

// ═══ sidecar (onboard_bee.mjs D7) ══════════════════════════════════════════

/// provenance: onboard_bee.mjs manifestFingerprint —
/// `JSON.stringify([...files.entries()].sort((a,b) => a[0] < b[0] ? -1 : 1))`.
/// Keys are unique, so that comparator is a plain code-unit ascending sort.
fn manifest_fingerprint(files: &[(String, String)]) -> String {
    let mut sorted: Vec<&(String, String)> = files.iter().collect();
    sorted.sort_by(|a, b| super::code_unit_cmp(&a.0, &b.0));
    let arr = Value::Array(
        sorted
            .into_iter()
            .map(|(k, v)| Value::Array(vec![Value::String(k.clone()), Value::String(v.clone())]))
            .collect(),
    );
    jsjson::stringify(&arr)
}

/// provenance: onboard_bee.mjs skillDigest — sha256 over manifestFingerprint.
fn skill_digest(files: &[(String, String)]) -> String {
    sha256_hex(manifest_fingerprint(files).as_bytes())
}

/// provenance: onboard_bee.mjs buildRenderSidecar — {schema, target_runtime,
/// skills:[{name, sha256}]}, skills sorted by name with the `<`/`>`
/// comparator (code units).
fn build_render_sidecar(target_runtime: &str, entries: &[(String, Vec<(String, String)>)]) -> Value {
    let mut skills: Vec<(String, String)> = entries
        .iter()
        .map(|(name, files)| (name.clone(), skill_digest(files)))
        .collect();
    skills.sort_by(|a, b| super::code_unit_cmp(&a.0, &b.0));
    let mut root = Map::new();
    root.insert("schema".into(), Value::String(RENDER_SCHEMA.into()));
    root.insert("target_runtime".into(), Value::String(target_runtime.into()));
    root.insert(
        "skills".into(),
        Value::Array(
            skills
                .into_iter()
                .map(|(name, sha)| {
                    let mut o = Map::new();
                    o.insert("name".into(), Value::String(name));
                    o.insert("sha256".into(), Value::String(sha));
                    Value::Object(o)
                })
                .collect(),
        ),
    );
    Value::Object(root)
}

/// provenance: render_plugin_skill_trees.mjs groupRenderedBySkill — one entry
/// per skill, in the rendered map's insertion order, each carrying its
/// per-file sha256 of the RENDERED bytes.
fn group_rendered_by_skill(rendered: &[(String, Vec<u8>)]) -> Vec<(String, Vec<(String, String)>)> {
    let mut by_skill: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (skill_rel, bytes) in rendered {
        let slash = skill_rel.find('/').unwrap_or(skill_rel.len());
        let name = skill_rel[..slash].to_string();
        let rel = skill_rel[(slash + 1).min(skill_rel.len())..].to_string();
        let idx = match by_skill.iter().position(|(n, _)| *n == name) {
            Some(i) => i,
            None => {
                by_skill.push((name, Vec::new()));
                by_skill.len() - 1
            }
        };
        by_skill[idx].1.push((rel, sha256_hex(bytes)));
    }
    by_skill
}

/// provenance: render_plugin_skill_trees.mjs sidecarBytes —
/// `JSON.stringify(obj, null, 2) + "\n"`.
fn sidecar_bytes(runtime: &str, rendered: &[(String, Vec<u8>)]) -> Vec<u8> {
    let obj = build_render_sidecar(runtime, &group_rendered_by_skill(rendered));
    format!("{}\n", jsjson::stringify_pretty(&obj)).into_bytes()
}

// ═══ the canonical walk ════════════════════════════════════════════════════

/// provenance: render_plugin_skill_trees.mjs listBeeSkillDirs — directories
/// (never symlinks) named `bee-*`, `.sort()`ed (code units, NOT localeCompare).
fn list_bee_skill_dirs(src: &Path) -> R<Vec<String>> {
    let Ok(entries) = std::fs::read_dir(src) else { return Err(Nd) };
    let mut names = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else { return Err(Nd) };
        let Ok(ft) = entry.file_type() else { return Err(Nd) };
        let name = entry.file_name().to_string_lossy().into_owned();
        if ft.is_dir() && !ft.is_symlink() && name.starts_with("bee-") {
            names.push(name);
        }
    }
    super::js_default_sort(&mut names);
    Ok(names)
}

/// provenance: render_plugin_skill_trees.mjs walkFiles — depth-first over
/// `readdirSync(...).sort((a,b) => a.name.localeCompare(b.name))`. A symlink
/// or an unsupported entry is a loud refusal in Node (a thrown stack); here
/// it is `Err(Nd)` so the probe returns None before any output.
fn walk_files(dir: &Path, rel_prefix: &str, out: &mut Vec<(String, PathBuf)>) -> R<()> {
    let Ok(read) = std::fs::read_dir(dir) else { return Err(Nd) };
    let mut entries: Vec<(String, PathBuf, std::fs::FileType)> = Vec::new();
    for entry in read {
        let Ok(entry) = entry else { return Err(Nd) };
        let Ok(ft) = entry.file_type() else { return Err(Nd) };
        entries.push((entry.file_name().to_string_lossy().into_owned(), entry.path(), ft));
    }
    if !sort_by_locale(&mut entries, |e| e.0.as_str()) {
        return Err(Nd); // a name outside the proven collation alphabet
    }
    for (name, abs, ft) in entries {
        let rel = if rel_prefix.is_empty() {
            name.clone()
        } else {
            format!("{rel_prefix}/{name}")
        };
        if ft.is_symlink() {
            return Err(Nd); // "symlink forbidden in skill source" (a V8 stack in Node)
        }
        if ft.is_dir() {
            walk_files(&abs, &rel, out)?;
        } else if ft.is_file() {
            out.push((rel, abs));
        } else {
            return Err(Nd); // "unsupported entry"
        }
    }
    Ok(())
}

/// provenance: render_plugin_skill_trees.mjs canonicalFiles.
fn canonical_files(root: &Path) -> R<Vec<(String, PathBuf)>> {
    let src = source_root(root);
    let mut files = Vec::new();
    for name in list_bee_skill_dirs(&src)? {
        let mut inner = Vec::new();
        walk_files(&src.join(&name), "", &mut inner)?;
        for (rel, abs) in inner {
            files.push((format!("{name}/{rel}"), abs));
        }
    }
    Ok(files)
}

/// provenance: render_plugin_skill_trees.mjs validateWholeTree.
fn validate_whole_tree(files: &[(String, PathBuf)]) -> R<Vec<String>> {
    let mut errors = Vec::new();
    for (skill_rel, abs) in files {
        let Ok(bytes) = std::fs::read(abs) else { return Err(Nd) };
        let text = String::from_utf8_lossy(&bytes);
        for e in validate_skill_markers(&text) {
            errors.push(format!("{skill_rel}: {e}"));
        }
    }
    Ok(errors)
}

/// provenance: render_plugin_skill_trees.mjs renderTree.
fn render_tree(runtime: &str, files: &[(String, PathBuf)]) -> R<Vec<(String, Vec<u8>)>> {
    let mut out = Vec::with_capacity(files.len());
    for (skill_rel, abs) in files {
        let Ok(bytes) = std::fs::read(abs) else { return Err(Nd) };
        out.push((skill_rel.clone(), render_skill_bytes(&bytes, runtime)));
    }
    Ok(out)
}

// ═══ tmp/backup hygiene + the swap ═════════════════════════════════════════

/// provenance: render_plugin_skill_trees.mjs randomSuffix —
/// `crypto.randomBytes(6).toString("hex")`. Only uniqueness is load-bearing
/// (the name never appears in output), so this derives 12 hex chars from
/// pid + a monotonic counter + the clock instead of pulling in an RNG crate.
fn random_suffix() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let seed = format!(
        "{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        nanos
    );
    sha256_hex(seed.as_bytes())[..12].to_string()
}

/// provenance: render_plugin_skill_trees.mjs isSwapDirRemovable — LIVE-PID
/// discipline first (a dir whose owning pid is proven alive is never touched,
/// at any age; a proven-dead pid is swept immediately, at any age), falling
/// back to age only when the pid segment cannot be parsed at all.
fn is_swap_dir_removable(entry_name: &str, prefix: &str, mtime_ms: f64, now_ms: f64) -> bool {
    // `new RegExp("\\." + prefix + "(\\d+)-")`, unanchored.
    let needle = format!(".{prefix}");
    let mut from = 0usize;
    while let Some(p) = entry_name[from..].find(&needle) {
        let after = &entry_name[from + p + needle.len()..];
        let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() && after[digits.len()..].starts_with('-') {
            let pid = digits.parse::<f64>().ok();
            return !lock::is_pid_alive(pid);
        }
        from += p + 1;
    }
    now_ms - mtime_ms > TMP_STALE_MS
}

/// provenance: render_plugin_skill_trees.mjs cleanStaleTmpDirs. Runs BEFORE
/// the lock, deliberately: a leaked dir from a dead pid must not need the
/// lock to clean up.
fn clean_stale_tmp_dirs(target_root: &Path) {
    let Some(parent) = target_root.parent() else { return };
    let Some(base) = target_root.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(parent) else { return };
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as f64)
        .unwrap_or(0.0);
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        if !ft.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(prefix) = SWAP_DIR_PREFIXES
            .iter()
            .find(|p| name.starts_with(&format!("{base}.{p}")))
        else {
            continue;
        };
        let abs = parent.join(&name);
        // `fs.statSync` follows; a vanished entry is skipped.
        let Ok(meta) = std::fs::metadata(&abs) else { continue };
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64() * 1000.0)
            .unwrap_or(0.0);
        if is_swap_dir_removable(&name, prefix, mtime, now_ms) {
            let _ = std::fs::remove_dir_all(&abs);
        }
    }
}

/// provenance: render_plugin_skill_trees.mjs writeTree — render into a fresh
/// tmp SIBLING (never touching targetRoot), sidecar included pre-swap, then
/// two renames. Caller MUST hold the "plugin-render" lock.
fn write_tree(target_root: &Path, rendered: &[(String, Vec<u8>)], runtime: &str) -> std::io::Result<()> {
    let parent = target_root.parent().ok_or_else(|| {
        std::io::Error::other("render_plugin_skill_trees: target root has no parent")
    })?;
    let base = target_root.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let tmp_root = parent.join(format!("{base}.tmp-{}-{}", std::process::id(), random_suffix()));
    let _ = std::fs::remove_dir_all(&tmp_root);
    std::fs::create_dir_all(&tmp_root)?;

    let mut backup_root: Option<PathBuf> = None;
    let result = (|| -> std::io::Result<()> {
        for (rel, bytes) in rendered {
            let mut dest = tmp_root.clone();
            for seg in rel.split('/') {
                dest = dest.join(seg);
            }
            if let Some(dir) = dest.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(&dest, bytes)?;
        }
        std::fs::write(tmp_root.join(RENDER_SIDECAR), sidecar_bytes(runtime, rendered))?;

        if target_root.exists() {
            let backup =
                parent.join(format!("{base}.old-{}-{}", std::process::id(), random_suffix()));
            std::fs::rename(target_root, &backup)?;
            backup_root = Some(backup);
        }
        if let Err(e) = std::fs::rename(&tmp_root, target_root) {
            // Swap failed partway — restore rather than leave targetRoot gone.
            if let Some(backup) = backup_root.take() {
                std::fs::rename(&backup, target_root)?;
            }
            return Err(e);
        }
        Ok(())
    })();

    // `finally`: runs on every exit path. tmp removal is a no-op once
    // renamed away; a still-set backup means the swap committed but cleanup
    // had not run yet.
    let _ = std::fs::remove_dir_all(&tmp_root);
    if let Some(backup) = backup_root {
        let _ = std::fs::remove_dir_all(&backup);
    }
    result
}

// ═══ CLI ═══════════════════════════════════════════════════════════════════

fn fail(message: &str) -> ExitCode {
    // Node prints `${error.stack}` here — this is the message without the V8
    // `at` frames (devtools/mod.rs header, error-path divergence).
    eprintln!("render_plugin_skill_trees: {message}");
    ExitCode::FAILURE
}

pub(super) fn run(args: &[&str]) -> Option<ExitCode> {
    if !args.is_empty() {
        return None; // the .mjs takes no arguments
    }
    let root = super::bee_source_root()?;
    if !source_root(&root).is_dir() {
        return None;
    }

    // provenance: main() — validate the WHOLE tree before any write.
    let files = canonical_files(&root).ok()?;
    let errors = validate_whole_tree(&files).ok()?;
    if !errors.is_empty() {
        eprintln!(
            "render_plugin_skill_trees: refused (marker grammar):\n{}",
            errors.join("\n")
        );
        return Some(ExitCode::FAILURE);
    }
    // Pre-render both trees before touching disk, so an unreadable source
    // file is still a "return None before any output" failure.
    let mut trees = Vec::new();
    for runtime in RENDER_RUNTIMES {
        trees.push((runtime, render_tree(runtime, &files).ok()?));
    }

    for runtime in RENDER_RUNTIMES {
        clean_stale_tmp_dirs(&target_root(&root, runtime));
    }

    let _guard = match lock::acquire_store_lock(&root, "plugin-render", lock::MAX_ATTEMPTS) {
        Ok(g) => g,
        Err(busy) => return Some(fail(&busy.message())),
    };
    let mut out = String::new();
    for (runtime, rendered) in &trees {
        let target = target_root(&root, runtime);
        if let Err(e) = write_tree(&target, rendered, runtime) {
            print!("{out}");
            return Some(fail(&e.to_string()));
        }
        out.push_str(&format!(
            "WROTE {}: {} file(s) + {RENDER_SIDECAR}\n",
            rel_platform(&root, &target),
            rendered.len()
        ));
    }
    print!("{out}");
    Some(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devtools::locale_compare;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("..")
    }

    // ── marker grammar ────────────────────────────────────────────────────

    #[test]
    fn near_marker_respects_the_word_boundary() {
        assert!(is_near_marker("<!-- bee:only claude -->"));
        assert!(is_near_marker("   <!--\tbee:end -->"));
        assert!(is_near_marker("<!-- bee:end -->"));
        assert!(!is_near_marker("<!-- bee:onlyish -->")); // \b fails
        assert!(!is_near_marker("<!-- bee:ending -->"));
        assert!(!is_near_marker("text <!-- bee:only claude -->")); // not line-leading
        assert!(!is_near_marker("bee:only claude"));
    }

    #[test]
    fn classify_pins_the_three_outcomes() {
        assert!(matches!(classify_marker_line("<!-- bee:only claude -->"), MarkerClass::Only(r) if r == "claude"));
        assert!(matches!(classify_marker_line("<!-- bee:only codex -->  "), MarkerClass::Only(r) if r == "codex"));
        assert!(matches!(classify_marker_line("<!-- bee:end -->"), MarkerClass::End));
        match classify_marker_line("<!-- bee:only python -->") {
            MarkerClass::Error(e) => {
                assert_eq!(e, "unknown runtime label \"python\" (expected claude or codex)")
            }
            _ => panic!("expected an unknown-runtime error"),
        }
        match classify_marker_line("  <!-- bee:only claude -->") {
            MarkerClass::Error(e) => assert!(e.starts_with("ambiguous near-marker")),
            _ => panic!("an indented marker is ambiguous, not valid"),
        }
    }

    /// Every expected value here was CAPTURED from the live
    /// `onboard_bee.mjs::validateSkillMarkers` over the same inputs (a Node
    /// harness, 13 cases) — this is the ported grammar pinned against its
    /// original, not against my reading of it.
    #[test]
    fn validate_catches_every_grammar_error() {
        let none: Vec<String> = Vec::new();
        assert_eq!(validate_skill_markers("plain text\nno markers\n"), none);
        assert_eq!(
            validate_skill_markers("a\n<!-- bee:only claude -->\nx\n<!-- bee:end -->\nb"),
            none
        );
        assert_eq!(
            validate_skill_markers("a\n<!-- bee:end -->\n"),
            ["stray bee:end with no open block at line 2"]
        );
        assert_eq!(
            validate_skill_markers("<!-- bee:only claude -->\n"),
            ["unclosed bee:only block opened at line 1"]
        );
        assert_eq!(
            validate_skill_markers(
                "<!-- bee:only claude -->\n<!-- bee:only codex -->\n<!-- bee:end -->\n"
            ),
            ["nested bee:only block at line 2 (block opened at line 1 not closed)"]
        );
        assert_eq!(
            validate_skill_markers("```\n<!-- bee:only claude -->\n```\n"),
            ["marker inside a fenced code block at line 2: \"<!-- bee:only claude -->\""]
        );
        assert_eq!(
            validate_skill_markers("---\n<!-- bee:only claude -->\n---\nbody"),
            ["marker inside YAML frontmatter at line 2: \"<!-- bee:only claude -->\""]
        );
        // A marker that CLOSES before the first `---` opener is fine…
        assert_eq!(
            validate_skill_markers("x\n<!-- bee:only claude -->\n<!-- bee:end -->\n---\ny\n---\n"),
            none
        );
        // …but one that OPENS after it is "before YAML frontmatter".
        assert_eq!(
            validate_skill_markers("x\n---\ny\n<!-- bee:only claude -->\n<!-- bee:end -->\n"),
            ["marker before YAML frontmatter at line 4"]
        );
        assert_eq!(
            validate_skill_markers("<!-- bee:only python -->\n<!-- bee:end -->\n"),
            [
                "unknown runtime label \"python\" (expected claude or codex) at line 1",
                "stray bee:end with no open block at line 2"
            ]
        );
        assert_eq!(
            validate_skill_markers("  <!-- bee:only claude -->\n<!-- bee:end -->\n"),
            [
                "ambiguous near-marker \"<!-- bee:only claude -->\" (not an exact full-line bee marker) at line 1",
                "stray bee:end with no open block at line 2"
            ]
        );
        // `\b` fails, so this is not a marker line at all.
        assert_eq!(validate_skill_markers("<!-- bee:onlyish -->\n"), none);
        assert_eq!(
            validate_skill_markers("<!-- bee:only claude -->x\n<!-- bee:end -->\n"),
            [
                "ambiguous near-marker \"<!-- bee:only claude -->x\" (not an exact full-line bee marker) at line 1",
                "stray bee:end with no open block at line 2"
            ]
        );
    }

    // ── renderSkillBytes ──────────────────────────────────────────────────

    #[test]
    fn render_is_byte_identity_without_markers() {
        // Arbitrary bytes (BOM, CRLF, invalid UTF-8) survive untouched.
        let raw: Vec<u8> = b"\xef\xbb\xbfhead\r\n\xff\xfe body\n".to_vec();
        assert_eq!(render_skill_bytes(&raw, "claude"), raw);
        // A literal mention with no marker LINE is also identity.
        let prose = b"talks about bee:only in prose\n".to_vec();
        assert_eq!(render_skill_bytes(&prose, "codex"), prose);
    }

    #[test]
    fn render_strips_markers_and_drops_off_runtime_blocks() {
        let src = "top\n<!-- bee:only claude -->\nC\n<!-- bee:end -->\n<!-- bee:only codex -->\nX\n<!-- bee:end -->\nend\n";
        assert_eq!(
            String::from_utf8(render_skill_bytes(src.as_bytes(), "claude")).unwrap(),
            "top\nC\nend\n"
        );
        assert_eq!(
            String::from_utf8(render_skill_bytes(src.as_bytes(), "codex")).unwrap(),
            "top\nX\nend\n"
        );
        // CRLF terminators are preserved exactly on kept lines.
        let crlf = "a\r\n<!-- bee:only claude -->\r\nb\r\n<!-- bee:end -->\r\nc";
        assert_eq!(
            String::from_utf8(render_skill_bytes(crlf.as_bytes(), "claude")).unwrap(),
            "a\r\nb\r\nc"
        );
    }

    // ── sidecar ───────────────────────────────────────────────────────────

    #[test]
    fn sidecar_shape_and_digest_match_onboard() {
        let rendered = vec![
            ("bee-b/SKILL.md".to_string(), b"B".to_vec()),
            ("bee-a/SKILL.md".to_string(), b"A".to_vec()),
            ("bee-a/refs/x.md".to_string(), b"X".to_vec()),
        ];
        let bytes = sidecar_bytes("claude", &rendered);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.ends_with("}\n"));
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["schema"], "bee-render/2");
        assert_eq!(v["target_runtime"], "claude");
        // skills sorted by name even though the render order was b, a
        assert_eq!(v["skills"][0]["name"], "bee-a");
        assert_eq!(v["skills"][1]["name"], "bee-b");
        // key order inside the object is schema, target_runtime, skills
        assert!(text.starts_with("{\n  \"schema\": \"bee-render/2\",\n  \"target_runtime\""));

        // skillDigest = sha256(JSON.stringify(sorted [rel, sha256(bytes)] pairs))
        let sha_a = sha256_hex(b"A");
        let sha_x = sha256_hex(b"X");
        let fingerprint = format!(
            "[[\"SKILL.md\",\"{sha_a}\"],[\"refs/x.md\",\"{sha_x}\"]]"
        );
        assert_eq!(v["skills"][0]["sha256"], sha256_hex(fingerprint.as_bytes()));
    }

    // ── the swap-dir sweep ────────────────────────────────────────────────

    #[test]
    fn swap_dir_sweep_is_pid_first_then_age() {
        let me = std::process::id();
        // Own pid is alive -> never removable, no matter the age.
        assert!(!is_swap_dir_removable(
            &format!("skills.tmp-{me}-abc"),
            "tmp-",
            0.0,
            1e18
        ));
        // A pid that cannot be parsed falls back to age.
        assert!(!is_swap_dir_removable("skills.tmp-xyz", "tmp-", 1000.0, 2000.0));
        assert!(is_swap_dir_removable("skills.tmp-xyz", "tmp-", 0.0, TMP_STALE_MS + 1.0));
        // Both prefixes are recognised.
        assert!(!is_swap_dir_removable(
            &format!("skills.old-{me}-abc"),
            "old-",
            0.0,
            1e18
        ));
    }

    // ── end-to-end over a fixture tree ────────────────────────────────────

    #[test]
    fn renders_both_trees_and_swaps_them_in() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let skills = root.join("skills");
        std::fs::create_dir_all(skills.join("bee-alpha").join("references")).unwrap();
        std::fs::create_dir_all(skills.join("bee-beta")).unwrap();
        std::fs::create_dir_all(skills.join("not-a-bee-skill")).unwrap();
        std::fs::write(skills.join("not-a-bee-skill").join("x.md"), b"ignored").unwrap();
        std::fs::write(
            skills.join("bee-alpha").join("SKILL.md"),
            "shared\n<!-- bee:only claude -->\nclaude only\n<!-- bee:end -->\n<!-- bee:only codex -->\ncodex only\n<!-- bee:end -->\ntail\n",
        )
        .unwrap();
        std::fs::write(skills.join("bee-alpha").join("references").join("r.md"), b"plain\n").unwrap();
        std::fs::write(skills.join("bee-beta").join("SKILL.md"), b"beta\n").unwrap();

        let files = canonical_files(root).unwrap();
        // Walk order is localeCompare, which folds case at the primary level:
        // `references` sorts BEFORE `SKILL.md` (r < s), the opposite of the
        // code-unit answer. Pinned here because it is the difference the
        // committed trees were written under.
        assert_eq!(
            files.iter().map(|(r, _)| r.as_str()).collect::<Vec<_>>(),
            ["bee-alpha/references/r.md", "bee-alpha/SKILL.md", "bee-beta/SKILL.md"]
        );
        assert!(validate_whole_tree(&files).unwrap().is_empty());

        for runtime in RENDER_RUNTIMES {
            let rendered = render_tree(runtime, &files).unwrap();
            let target = target_root(root, runtime);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            write_tree(&target, &rendered, runtime).unwrap();

            let skill = std::fs::read_to_string(target.join("bee-alpha").join("SKILL.md")).unwrap();
            assert_eq!(skill, format!("shared\n{runtime} only\ntail\n"));
            assert_eq!(
                std::fs::read(target.join("bee-alpha").join("references").join("r.md")).unwrap(),
                b"plain\n"
            );
            let sidecar = std::fs::read(target.join(RENDER_SIDECAR)).unwrap();
            assert_eq!(sidecar, sidecar_bytes(runtime, &rendered));
            // no tmp/backup siblings leaked
            let leftovers: Vec<String> = std::fs::read_dir(target.parent().unwrap())
                .unwrap()
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.contains(".tmp-") || n.contains(".old-"))
                .collect();
            assert!(leftovers.is_empty(), "leaked swap dirs: {leftovers:?}");
        }

        // Re-running over an existing target is idempotent (the swap path).
        let rendered = render_tree("claude", &files).unwrap();
        write_tree(&target_root(root, "claude"), &rendered, "claude").unwrap();
        assert_eq!(
            std::fs::read_to_string(target_root(root, "claude").join("bee-beta").join("SKILL.md"))
                .unwrap(),
            "beta\n"
        );
    }

    #[test]
    fn a_symlink_in_the_source_refuses_before_any_output() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("skills").join("bee-x")).unwrap();
        std::fs::write(root.join("skills").join("bee-x").join("SKILL.md"), b"x").unwrap();
        assert!(canonical_files(root).is_ok());

        let link = root.join("skills").join("bee-x").join("linked.md");
        let made = {
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_file(root.join("skills").join("bee-x").join("SKILL.md"), &link).is_ok()
            }
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(root.join("skills").join("bee-x").join("SKILL.md"), &link).is_ok()
            }
        };
        if !made {
            return; // host cannot create symlinks (win32 without the privilege)
        }
        assert!(canonical_files(root).is_err(), "a symlink must refuse the whole walk");
    }

    // ── THE PIN: the committed trees ──────────────────────────────────────

    /// The whole point of the tool: re-render the REAL skills/ tree and
    /// byte-compare against the two COMMITTED projections, sidecars included.
    /// This is what catches this port, the onboard port, or a hand-edited
    /// tree drifting from `render(canonical, runtime)`.
    #[test]
    fn render_matches_the_committed_trees() {
        let root = repo_root();
        if !source_root(&root).is_dir() {
            return; // not a source checkout
        }
        let files = canonical_files(&root).expect("canonical walk");
        assert!(!files.is_empty(), "skills/ must hold bee-* skills");
        assert!(
            validate_whole_tree(&files).expect("read").is_empty(),
            "the live skills/ tree must satisfy the marker grammar"
        );
        for runtime in RENDER_RUNTIMES {
            let rendered = render_tree(runtime, &files).expect("render");
            let target = target_root(&root, runtime);
            for (rel, bytes) in &rendered {
                let mut committed = target.clone();
                for seg in rel.split('/') {
                    committed = committed.join(seg);
                }
                let on_disk = std::fs::read(&committed)
                    .unwrap_or_else(|e| panic!("committed {} missing: {e}", committed.display()));
                assert_eq!(
                    on_disk,
                    *bytes,
                    "{runtime} projection drifted at {rel} — run `bee dev render-skill-trees`"
                );
            }
            let committed_sidecar = std::fs::read(target.join(RENDER_SIDECAR)).expect("sidecar");
            assert_eq!(
                committed_sidecar,
                sidecar_bytes(runtime, &rendered),
                "{runtime} sidecar drifted"
            );
            // And the committed tree holds nothing the render did not produce.
            let mut committed_files = Vec::new();
            walk_files(&target, "", &mut committed_files).expect("walk committed tree");
            let produced: Vec<&str> = rendered.iter().map(|(r, _)| r.as_str()).collect();
            for (rel, _) in &committed_files {
                if rel == RENDER_SIDECAR {
                    continue;
                }
                assert!(
                    produced.contains(&rel.as_str()),
                    "{runtime} tree carries {rel}, which render() does not produce"
                );
            }
        }
    }

    /// Node's answers, captured from `String#localeCompare` on the real skill
    /// dir names — the pin for the walk's comparator.
    #[test]
    fn locale_order_is_what_the_walk_uses() {
        use std::cmp::Ordering::*;
        assert_eq!(locale_compare("SKILL.md", "references"), Some(Greater));
        assert_eq!(locale_compare("SKILL.md", "templates"), Some(Less));
        assert_eq!(locale_compare("SKILL.md", ".bee-render.json"), Some(Greater));
        assert_eq!(locale_compare("agents", "references"), Some(Less));
        // The real bee-hive dir order: agents | references | SKILL.md
        let mut v = vec!["SKILL.md".to_string(), "references".into(), "agents".into()];
        assert!(sort_by_locale(&mut v, |s| s.as_str()));
        assert_eq!(v, ["agents", "references", "SKILL.md"]);
    }
}
