// bee dev release-manifest (DIST-01/DIST-03/D-03, decision ed0b2920).
//
// Enumerates the release-identity file set for the bee distribution and
// hashes it. `--write` regenerates
// docs/history/codex-harness-hardening/release-manifest.json, `--check`
// recomputes and compares (the shape a full verify run checks on every
// verify), `--selftest` proves the comparison logic actually bites.
//
// Two details carry the whole byte-identity of the output file:
//
//   1. THE SORT. The path sort is ICU collation, not code units. Sorting the
//      real 326-path set by code unit produces a DIFFERENT file (measured),
//      so devtools::locale_compare is mandatory here — see its header for
//      the proof. The manifest comparison mixes the two comparators
//      deliberately: `missing`/`added` use bare code-unit sort, `changed`
//      uses locale collation. Both are reproduced as written.
//   2. THE MODE. The git index, not the filesystem: `100755` records `"755"`,
//      anything else records `"644"`. It used to be `statSync(p).mode & 0o777`,
//      which is not a portable fact — Windows libuv synthesises 0666 (0444
//      when FILE_ATTRIBUTE_READONLY) and never reports the executable bit, so
//      the manifest could only ever agree with the platform that wrote it:
//      every one of its 205 records read as drifted on the other. The
//      executable bit as git records it is the same on every clone, which is
//      what a distribution proof needs. `observed_mode` is the matching
//      reader for a checkout with no index (an installed package), and it
//      answers `None` on Windows rather than guessing.
//
// ROUTING. Exactly one of --write/--check/--selftest (an extra unrecognised
// argument is ignored). Every failure refuses natively in the
// `FAIL release_manifest: …` shape with a non-zero exit. The one surviving
// None is a path outside the proven collation alphabet.

use super::{rel_posix, sha256_hex, sort_by_locale};
use crate::jsjson;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Bumped to 2 when the file gained a second top-level key
/// (`unhashedArtifacts`) and the `runtime_lib` / `distribution_test` roles
/// left the inventory, so a stored v1 manifest is not comparable to a v2
/// build.
const SCHEMA_VERSION: u64 = 2;

/// THE BINARY, and the one thing in the shipped frame that cannot be hashed.
///
/// A host receives `.bee/bin/bee` — it is the whole point of the R6 cutover —
/// but the file is GITIGNORED and per-platform: it is produced by
/// `cargo build --release` on the installing machine, so its bytes differ
/// between every host, every toolchain and every build. Three options were
/// weighed:
///
///   a) omit it. Rejected: the frame would claim to describe what a host
///      receives while silently missing the only executable in it.
///   b) hash it. Rejected: `--check` would then fail on every machine except
///      the one that last ran `--write` — a check that always fails is a check
///      everyone learns to ignore.
///   c) record it as PRESENCE-ONLY. Chosen. The manifest names the artifact
///      and says why it carries no hash; `--check` asserts the file EXISTS and
///      refuses loudly when it does not.
///
/// Both spellings are accepted because the same manifest is checked on Windows
/// and on POSIX; finding either satisfies the requirement.
const UNHASHED_ARTIFACTS: &[(&str, &[&str], &str)] = &[(
    ".bee/bin/bee",
    &[".bee/bin/bee.exe", ".bee/bin/bee"],
    "built by `cargo build --release` on the installing host and gitignored, so it has no stable content hash; presence is checked, bytes are not",
)];

/// Repo-relative roots the manifest's inventory covers. Read by
/// `verbs/cells.rs`'s regen obligation so "what the release manifest covers"
/// is DERIVED from the manifest builder rather than pasted beside it, by
/// sharing the definition instead of re-reading it (D2).
pub(crate) const INVENTORY_ROOTS: &[&str] = &[
    ".bee/bin/bee",
    ".bee/expertise",
    ".claude-plugin/marketplace.json",
    ".claude-plugin/plugin.json",
    ".claude-plugin/skills",
    ".codex-plugin/plugin.json",
    ".codex-plugin/skills",
    ".opencode/plugins",
    "expertise",
    "packages/bee",
    "packages/bee/hooks",
    "scripts/install.ps1",
    "scripts/install.sh",
    "skills",
];

/// The manifest file itself, repo-relative — the file a cell that touches any
/// covered root must also list in `files`.
pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";

fn manifest_path(root: &Path) -> PathBuf {
    root.join("docs")
        .join("history")
        .join("codex-harness-hardening")
        .join("release-manifest.json")
}

enum BuildErr {
    /// `FAIL release_manifest: ${message}` + exit 1.
    Refuse(String),
    /// Unproven bytes — the probe returns None. Every I/O arm reports through
    /// `Refuse` (see `io_refuse`); what remains is `sort_records`, whose
    /// subject is locale collation over free prose — the one case still left
    /// unproven.
    Nd,
}
type R<T> = Result<T, BuildErr>;

#[derive(Clone)]
struct Record {
    path: String,
    sha256: String,
    mode: String,
    role: String,
    package_path: Option<String>,
}

impl Record {
    fn to_value(&self) -> Value {
        let mut o = Map::new();
        o.insert("path".into(), Value::String(self.path.clone()));
        o.insert("sha256".into(), Value::String(self.sha256.clone()));
        o.insert("mode".into(), Value::String(self.mode.clone()));
        o.insert("role".into(), Value::String(self.role.clone()));
        if let Some(p) = &self.package_path {
            o.insert("packagePath".into(), Value::String(p.clone()));
        }
        Value::Object(o)
    }
}

/// `(mode & 0o777)` formatted as 3 zero-padded octal digits — the raw stat
/// bits, kept for same-machine before/after snapshots only.
///
/// It is NOT what the manifest records: raw bits are not comparable across
/// machines (Windows synthesises 0666, or 0444 for FILE_ATTRIBUTE_READONLY,
/// and never reports the executable bit at all), so a manifest built on one
/// platform reported every record as drifted on the other. `index_mode` is
/// the recorded value; `observed_mode` is the only thing a checker may
/// compare it against.
pub(super) fn mode_octal(meta: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    let bits = {
        use std::os::unix::fs::MetadataExt;
        meta.mode() & 0o777
    };
    #[cfg(windows)]
    // libuv's uv__stat: FILE_ATTRIBUTE_READONLY -> 0444, otherwise 0666.
    let bits: u32 = if meta.permissions().readonly() { 0o444 } else { 0o666 };
    format!("{bits:03o}")
}

/// The one permission fact that survives a trip through git, a tarball and a
/// second operating system: whether the file is executable. `"755"` when it
/// is, `"644"` when it is not.
///
/// On Windows the bit does not exist to be read, so an observer there can
/// only say "I cannot tell" — `None`. A comparison against `None` is not a
/// mismatch; it is a check that platform cannot perform.
pub(super) fn observed_mode(meta: &std::fs::Metadata) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(if meta.mode() & 0o111 != 0 { "755".to_string() } else { "644".to_string() })
    }
    #[cfg(windows)]
    {
        let _ = meta;
        None
    }
}

/// The recorded mode, read from the git index rather than the filesystem —
/// `100755` is the executable bit as every clone of this repository receives
/// it, on every platform, whatever the local umask or filesystem did.
///
/// Memoized per root: one `git ls-files` per process instead of one per
/// record. A manifest build is a single pass over a tree nobody is staging
/// into at the same time, so a cached index cannot go stale inside one.
fn index_modes(root: &Path) -> std::sync::Arc<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<HashMap<String, String>>>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(hit) = cache.lock().unwrap().get(root) {
        return Arc::clone(hit);
    }
    let mut map = HashMap::new();
    if let Ok(out) = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-s", "-z"])
        .output()
    {
        if out.status.success() {
            // Each entry is "<mode> <sha> <stage>\t<path>\0".
            for entry in String::from_utf8_lossy(&out.stdout).split('\0') {
                let Some((meta, path)) = entry.split_once('\t') else { continue };
                let Some(mode) = meta.split(' ').next() else { continue };
                let recorded = if mode == "100755" { "755" } else { "644" };
                map.insert(path.to_string(), recorded.to_string());
            }
        }
    }
    let shared = Arc::new(map);
    cache.lock().unwrap().insert(root.to_path_buf(), Arc::clone(&shared));
    shared
}

/// The recorded mode for one file: the git index when the file is tracked,
/// and the observed executable bit when it is not (an untracked file in an
/// inventory root is already a drift the caller will report; guessing its
/// mode is not the interesting failure). Windows, which cannot observe the
/// bit, records the non-executable default.
fn index_mode(root: &Path, rel: &str, meta: &std::fs::Metadata) -> String {
    if let Some(mode) = index_modes(root).get(rel) {
        return mode.clone();
    }
    observed_mode(meta).unwrap_or_else(|| "644".to_string())
}

/// A filesystem failure, worded by us. The error KIND is named rather than
/// the OS message string, which varies by platform and locale.
fn io_refuse(action: &str, path: &Path, err: &std::io::Error) -> BuildErr {
    BuildErr::Refuse(format!(
        "release_manifest: cannot {action} {} ({})",
        path.display(),
        err.kind()
    ))
}

fn build_record(root: &Path, abs: &Path, role: &str, with_package_path: bool) -> R<Record> {
    let data = std::fs::read(abs).map_err(|e| io_refuse("read", abs, &e))?;
    let meta = std::fs::metadata(abs).map_err(|e| io_refuse("stat", abs, &e))?;
    let path = rel_posix(root, abs);
    Ok(Record {
        sha256: sha256_hex(&data),
        mode: index_mode(root, &path, &meta),
        role: role.to_string(),
        package_path: if with_package_path { Some(path.clone()) } else { None },
        path,
    })
}

fn sort_records(records: &mut [Record]) -> R<()> {
    if sort_by_locale(records, |r| r.path.as_str()) {
        Ok(())
    } else {
        Err(BuildErr::Nd) // a path outside the proven collation alphabet
    }
}

/// Recursive; `packagePath` set, `exclude_top` skipping only IMMEDIATE
/// children.
fn enumerate_tree(root: &Path, dir: &Path, role: &str, exclude_top: &[&str]) -> R<Vec<Record>> {
    if !dir.exists() {
        return Err(BuildErr::Refuse(format!(
            "release_manifest: expected directory missing: {}",
            dir.display()
        )));
    }
    let mut records = Vec::new();
    walk(root, dir, dir, role, exclude_top, &mut records)?;
    sort_records(&mut records)?;
    Ok(records)
}

fn walk(
    root: &Path,
    top: &Path,
    current: &Path,
    role: &str,
    exclude_top: &[&str],
    out: &mut Vec<Record>,
) -> R<()> {
    let entries = std::fs::read_dir(current).map_err(|e| io_refuse("list", current, &e))?;
    for entry in entries {
        let entry = entry.map_err(|e| io_refuse("list", current, &e))?;
        let ft = entry.file_type().map_err(|e| io_refuse("stat", &entry.path(), &e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if current == top && ft.is_dir() && exclude_top.contains(&name.as_str()) {
            continue;
        }
        let abs = entry.path();
        if ft.is_symlink() {
            return Err(BuildErr::Refuse(format!(
                "release_manifest: symlink forbidden in package inventory: {}",
                abs.display()
            )));
        }
        if ft.is_dir() {
            walk(root, top, &abs, role, exclude_top, out)?;
        } else if ft.is_file() {
            out.push(build_record(root, &abs, role, true)?);
        } else {
            return Err(BuildErr::Refuse(format!(
                "release_manifest: unsupported package entry: {}",
                abs.display()
            )));
        }
    }
    Ok(())
}

/// Files with `ext` directly inside `dir` (no recursion), no `packagePath`,
/// sorted.
fn enumerate_flat_dir(root: &Path, dir: &Path, role: &str, ext: &str) -> R<Vec<Record>> {
    if !dir.exists() {
        return Err(BuildErr::Refuse(format!(
            "release_manifest: expected directory missing: {}",
            dir.display()
        )));
    }
    let mut records = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| io_refuse("list", dir, &e))? {
        let entry = entry.map_err(|e| io_refuse("list", dir, &e))?;
        let ft = entry.file_type().map_err(|e| io_refuse("stat", &entry.path(), &e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if ft.is_file() && name.ends_with(ext) {
            records.push(build_record(root, &entry.path(), role, false)?);
        }
    }
    sort_records(&mut records)?;
    Ok(records)
}

/// THE SHIPPED FRAME (owner decision). Every root below answers one
/// question: what does a HOST actually receive when it installs bee?
///
///   * the binary            `.bee/bin/bee` — see UNHASHED_ARTIFACTS
///   * the prompts           packages/bee/prompts/** (inside the payload tree)
///   * the expertise         expertise/**  +  .bee/expertise/**
///   * the skills            skills/**  +  the two committed render trees
///   * the hook manifests    packages/bee/hooks/**
///   * the plugin identity   the two plugin.json + marketplace.json
///   * the installers        scripts/install.sh / .ps1
///   * the third belt        .opencode/plugins/** (opencode-support oc-14) —
///     `bee-guard.ts`, vendored into a host by `bee onboard --apply`; without
///     this root a release that lost the file stayed green on `--check`
///     while installing nothing, the exact silent-drop the manifest exists
///     to catch for every other shipped artifact
///
/// DROPPED, with reasons:
///   * the vendored Node library under `.bee/bin/lib/` (`runtime_lib`, 38
///     records) — it is deleted; a host receives a binary instead, and the
///     frame must describe what ships, not what used to.
///   * the Node test suites' own distribution tests (`distribution_test`, 2
///     records) — deleted with the suites. They were never SHIPPED in the
///     first place; they were the tests OF the distribution, pinned here so
///     a release could not quietly drop its own guard. That job now belongs
///     to `cargo test`, which cannot be dropped without the build noticing.
///
/// `packages/bee/**` and `packages/bee/hooks/**` stay as roots and simply
/// shrink: after the deletion they carry the prompts, the statusline, the
/// agent templates, AGENTS.block.md and the two hook JSON manifests. Keeping
/// the roots rather than enumerating survivors means a file ADDED back to
/// those trees is caught by `--check`, which a hand-listed survivor set would
/// not do.
///
/// Roots are also read by `verbs/cells.rs`'s regen obligation (see
/// `inventory_roots`), so "what the manifest covers" has exactly one
/// definition in the binary.
fn build_current_records(root: &Path) -> R<Vec<Record>> {
    let mut records: Vec<Record> = Vec::new();
    records.extend(enumerate_flat_dir(root, &root.join("expertise"), "expertise_guide", ".md")?);
    records.extend(enumerate_flat_dir(
        root,
        &root.join(".bee").join("expertise"),
        "runtime_expertise",
        ".md",
    )?);
    records.extend(enumerate_tree(
        root,
        &root.join("packages").join("bee"),
        "package_payload",
        &["hooks"],
    )?);
    records.extend(enumerate_tree(root, &root.join("skills"), "plugin_skill", &[])?);
    // provenance: PLUGIN_SKILL_RENDER_ROOTS (D9/cnr2-12) — the committed
    // per-runtime projections, under their own roles so managedSkillNames()
    // never sees them.
    for (dir, role) in [
        (root.join(".claude-plugin").join("skills"), "plugin_skill_claude_render"),
        (root.join(".codex-plugin").join("skills"), "plugin_skill_codex_render"),
    ] {
        records.extend(enumerate_tree(root, &dir, role, &[])?);
    }
    records.extend(enumerate_tree(
        root,
        &root.join("packages").join("bee").join("hooks"),
        "plugin_hook",
        &[],
    )?);
    // The OpenCode guard plugin (opencode-support D2/oc-14) — the third
    // enforcement belt. Not a rendered projection like the skills roots
    // above; a hand-written, hand-vendored TypeScript file, but it ships the
    // same way everything else in this inventory does.
    records.extend(enumerate_tree(
        root,
        &root.join(".opencode").join("plugins"),
        "opencode_plugin",
        &[],
    )?);
    for abs in [
        root.join(".claude-plugin").join("plugin.json"),
        root.join(".codex-plugin").join("plugin.json"),
    ] {
        if !abs.exists() {
            return Err(BuildErr::Refuse(format!(
                "release_manifest: expected plugin manifest missing: {}",
                abs.display()
            )));
        }
        records.push(build_record(root, &abs, "plugin_manifest", true)?);
    }
    records.push(build_record(
        root,
        &root.join(".claude-plugin").join("marketplace.json"),
        "plugin_marketplace",
        true,
    )?);
    for abs in [
        root.join("scripts").join("install.sh"),
        root.join("scripts").join("install.ps1"),
    ] {
        records.push(build_record(root, &abs, "distribution_tool", false)?);
    }

    sort_records(&mut records)?;
    let duplicates: Vec<String> = records
        .iter()
        .enumerate()
        .filter(|(i, r)| *i > 0 && r.path == records[i - 1].path)
        .map(|(_, r)| r.path.clone())
        .collect();
    if !duplicates.is_empty() {
        return Err(BuildErr::Refuse(format!(
            "release_manifest: duplicate inventory path(s): {}",
            duplicates.join(", ")
        )));
    }
    Ok(records)
}

/// `{schemaVersion, unhashedArtifacts, files}`, pretty-printed with a
/// trailing newline.
fn manifest_bytes(records: &[Record]) -> String {
    let mut m = Map::new();
    m.insert("schemaVersion".into(), Value::Number(SCHEMA_VERSION.into()));
    m.insert("unhashedArtifacts".into(), unhashed_artifacts_value());
    m.insert(
        "files".into(),
        Value::Array(records.iter().map(Record::to_value).collect()),
    );
    format!("{}\n", jsjson::stringify_pretty(&Value::Object(m)))
}

/// The `unhashedArtifacts` block, self-documenting so a reader of the manifest
/// never has to guess why one shipped file carries no hash.
fn unhashed_artifacts_value() -> Value {
    Value::Array(
        UNHASHED_ARTIFACTS
            .iter()
            .map(|(path, accepts, reason)| {
                let mut r = Map::new();
                r.insert("path".into(), Value::String((*path).to_string()));
                r.insert(
                    "accepts".into(),
                    Value::Array(accepts.iter().map(|a| Value::String((*a).to_string())).collect()),
                );
                r.insert("check".into(), Value::String("presence".into()));
                r.insert("reason".into(), Value::String((*reason).to_string()));
                Value::Object(r)
            })
            .collect(),
    )
}

/// The presence half of `--check`: every unhashed artifact must exist under at
/// least one of its accepted spellings. Returns the failures, loudly named.
fn unhashed_artifact_failures(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for (path, accepts, _) in UNHASHED_ARTIFACTS {
        let present = accepts.iter().any(|rel| {
            let mut p = root.to_path_buf();
            for seg in rel.split('/') {
                p = p.join(seg);
            }
            p.is_file()
        });
        if !present {
            out.push(format!(
                "MISSING unhashed artifact {path} (looked for {}) - the shipped frame claims a \
                 host receives this binary and it is not here. Run `cargo build --release` in \
                 packages/bee-rs and copy the result into .bee/bin/.",
                accepts.join(" or ")
            ));
        }
    }
    out
}

// ─── comparison ────────────────────────────────────────────────────────────

#[derive(Default)]
struct Diff {
    missing: Vec<String>,
    added: Vec<String>,
    changed: Vec<(String, Vec<&'static str>)>,
}

impl Diff {
    fn ok(&self) -> bool {
        self.missing.is_empty() && self.added.is_empty() && self.changed.is_empty()
    }
}

/// One record's comparable fields, read the way JS reads them off a parsed
/// object: an absent key is `undefined`, which `!==` distinguishes from every
/// present value, and `packagePath` is normalised through `?? null`.
struct Fields<'a> {
    sha256: Option<&'a Value>,
    mode: Option<&'a Value>,
    role: Option<&'a Value>,
    package_path: Option<&'a Value>,
}

fn fields(v: &Value) -> Fields<'_> {
    Fields {
        sha256: v.get("sha256"),
        mode: v.get("mode"),
        role: v.get("role"),
        package_path: v.get("packagePath").filter(|p| !p.is_null()),
    }
}

/// NOTE the two different sorts: `missing`/`added` use bare code-unit sort,
/// `changed` uses locale collation.
fn compare_manifests(stored: &[Value], current: &[Value]) -> Option<Diff> {
    // `new Map(arr.map(r => [r.path, r]))` — a duplicate path keeps the LAST.
    let index = |arr: &[Value]| -> Vec<(String, Value)> {
        let mut out: Vec<(String, Value)> = Vec::new();
        for r in arr {
            let key = match r.get("path").and_then(Value::as_str) {
                Some(s) => s.to_string(),
                None => "undefined".to_string(),
            };
            match out.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = r.clone(),
                None => out.push((key, r.clone())),
            }
        }
        out
    };
    let stored_by = index(stored);
    let current_by = index(current);

    let mut diff = Diff::default();
    diff.missing = stored_by
        .iter()
        .map(|(k, _)| k.clone())
        .filter(|k| !current_by.iter().any(|(c, _)| c == k))
        .collect();
    super::js_default_sort(&mut diff.missing);
    diff.added = current_by
        .iter()
        .map(|(k, _)| k.clone())
        .filter(|k| !stored_by.iter().any(|(s, _)| s == k))
        .collect();
    super::js_default_sort(&mut diff.added);

    for (p, stored_record) in &stored_by {
        let Some((_, current_record)) = current_by.iter().find(|(c, _)| c == p) else { continue };
        let (s, c) = (fields(stored_record), fields(current_record));
        let mut reasons: Vec<&'static str> = Vec::new();
        if s.sha256 != c.sha256 {
            reasons.push("sha256");
        }
        if s.mode != c.mode {
            reasons.push("mode");
        }
        if s.role != c.role {
            reasons.push("role");
        }
        if s.package_path != c.package_path {
            reasons.push("packagePath");
        }
        if !reasons.is_empty() {
            diff.changed.push((p.clone(), reasons));
        }
    }
    if !sort_by_locale(&mut diff.changed, |c| c.0.as_str()) {
        return None;
    }
    Some(diff)
}

// ─── verbs ─────────────────────────────────────────────────────────────────

fn fail(message: &str) -> ExitCode {
    eprintln!("FAIL release_manifest: {message}");
    ExitCode::FAILURE
}

/// Unwrap a build result into either the records or a finished ExitCode/None.
fn resolve<T>(r: R<T>) -> Result<T, Option<ExitCode>> {
    match r {
        Ok(v) => Ok(v),
        Err(BuildErr::Refuse(m)) => Err(Some(fail(&m))),
        Err(BuildErr::Nd) => Err(None),
    }
}

fn run_write(root: &Path) -> Result<ExitCode, Option<ExitCode>> {
    let records = resolve(build_current_records(root))?;
    let file = manifest_path(root);
    // CUTOVER: both write failures used to delegate (Node printed a libuv
    // message). They refuse natively now, same channel and exit code.
    if let Some(dir) = file.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            return Err(Some(fail(&format!(
                "release_manifest: cannot create {} ({})",
                dir.display(),
                e.kind()
            ))));
        }
    }
    if let Err(e) = std::fs::write(&file, manifest_bytes(&records)) {
        return Err(Some(fail(&format!(
            "release_manifest: cannot write {} ({})",
            file.display(),
            e.kind()
        ))));
    }
    println!("WROTE {}: {} file(s)", rel_posix(root, &file), records.len());
    Ok(ExitCode::SUCCESS)
}

fn read_stored(root: &Path) -> Result<Vec<Value>, Option<ExitCode>> {
    let file = manifest_path(root);
    if !file.exists() {
        return Err(Some(fail(&format!(
            "release_manifest: stored manifest missing: {} (run --write first)",
            file.display()
        ))));
    }
    // CUTOVER: a read/parse failure was a V8/libuv message in Node, so this
    // returned None (delegate). Both refuse natively now, through the same
    // `FAIL release_manifest: …` channel and exit code as the missing and
    // malformed cases above and below — only the wording is ours.
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) => {
            return Err(Some(fail(&format!(
                "release_manifest: cannot read stored manifest: {} ({})",
                file.display(),
                e.kind()
            ))))
        }
    };
    let Ok(parsed) = serde_json::from_str::<Value>(&text) else {
        return Err(Some(fail(&format!(
            "release_manifest: stored manifest is not valid JSON: {}",
            file.display()
        ))));
    };
    // A stored manifest from before the frame was redrawn describes a
    // DIFFERENT inventory. Say so in one line instead of printing forty
    // mismatches that all mean the same thing.
    match parsed.get("schemaVersion").and_then(Value::as_u64) {
        Some(v) if v == SCHEMA_VERSION => {}
        Some(v) => {
            return Err(Some(fail(&format!(
                "release_manifest: stored manifest is schemaVersion {v}, this bee writes {SCHEMA_VERSION}. \
                 The inventory changed at the R6 Node cutover (the vendored .mjs roots left it and \
                 an unhashedArtifacts block joined it), so the two are not comparable. Run \
                 `bee dev release-manifest --write` and review the diff."
            ))))
        }
        None => {
            return Err(Some(fail(&format!(
                "release_manifest: stored manifest has no schemaVersion: {}",
                file.display()
            ))))
        }
    }
    match parsed.get("files") {
        Some(Value::Array(files)) => Ok(files.clone()),
        _ => Err(Some(fail(&format!(
            "release_manifest: stored manifest malformed: {}",
            file.display()
        )))),
    }
}

/// THE `--check` CONTRACT. Two obligations, both of which must hold:
///
///   1. HASH PARITY over `files` — every record in the stored manifest is
///      present in the current tree with the same sha256/mode/role/packagePath,
///      and the current tree adds nothing the manifest does not know about.
///      Unchanged from v1, and still the bulk of the check.
///   2. PRESENCE over `unhashedArtifacts` — new. `.bee/bin/bee` is part of the
///      shipped frame but cannot be hashed (per-host build, gitignored), so the
///      manifest asserts it EXISTS. A frame that promises a binary and cannot
///      find one fails the check; it does not pass quietly with the promise
///      unverified.
///
/// A schemaVersion mismatch is its own refusal (see `read_stored`): a v1
/// manifest describes an inventory that included the vendored library under
/// `.bee/bin/lib/`, and diffing it against a v2 build would report 40
/// spurious mismatches instead of the one real fact — that the file needs
/// regenerating.
fn run_check(root: &Path) -> Result<ExitCode, Option<ExitCode>> {
    let stored = read_stored(root)?;
    let current = resolve(build_current_records(root))?;
    let current_values: Vec<Value> = current.iter().map(Record::to_value).collect();
    let Some(diff) = compare_manifests(&stored, &current_values) else { return Err(None) };
    let unhashed_failures = unhashed_artifact_failures(root);
    if diff.ok() && unhashed_failures.is_empty() {
        println!(
            "release_manifest --check: {} file(s) match stored manifest, {} unhashed artifact(s) present",
            current.len(),
            UNHASHED_ARTIFACTS.len()
        );
        return Ok(ExitCode::SUCCESS);
    }
    for line in &unhashed_failures {
        eprintln!("{line}");
    }
    if diff.ok() {
        eprintln!(
            "release_manifest --check: FAIL ({} unhashed artifact(s) missing)",
            unhashed_failures.len()
        );
        return Ok(ExitCode::FAILURE);
    }
    // provenance: printDiff
    for p in &diff.missing {
        eprintln!("MISMATCH missing (in stored manifest, absent from current tree): {p}");
    }
    for p in &diff.added {
        eprintln!("MISMATCH added (in current tree, absent from stored manifest): {p}");
    }
    for (p, reasons) in &diff.changed {
        eprintln!("MISMATCH {p}: {} differ", reasons.join(", "));
    }
    eprintln!(
        "release_manifest --check: FAIL ({} missing, {} added, {} changed)",
        diff.missing.len(),
        diff.added.len(),
        diff.changed.len()
    );
    Ok(ExitCode::FAILURE)
}

/// Take the REAL manifest as a baseline, mutate ONE covered file's content in
/// a temp copy (never the real tree), and assert compare_manifests flags
/// exactly that file.
fn run_selftest(root: &Path) -> Result<ExitCode, Option<ExitCode>> {
    let baseline = resolve(build_current_records(root))?;
    if baseline.is_empty() {
        eprintln!("FAIL release_manifest --selftest: baseline manifest is empty, cannot prove anything");
        return Ok(ExitCode::FAILURE);
    }
    let target = baseline
        .iter()
        .find(|r| r.role == "package_payload" || r.role == "runtime_lib")
        .unwrap_or(&baseline[0])
        .clone();

    let Ok(temp_dir) = tempdir_for_selftest() else {
        // CUTOVER: was a delegate (Node's libuv message).
        return Err(Some(fail(
            "release_manifest --selftest: cannot create a scratch directory",
        )));
    };
    let real_abs = {
        let mut p = root.to_path_buf();
        for seg in target.path.split('/') {
            p = p.join(seg);
        }
        p
    };
    let temp_copy = temp_dir.join(
        target
            .path
            .rsplit('/')
            .next()
            .unwrap_or(target.path.as_str()),
    );
    let outcome = (|| -> Result<ExitCode, Option<ExitCode>> {
        // CUTOVER: both were delegates (Node's libuv messages).
        let Ok(mut content) = std::fs::read(&real_abs) else {
            return Err(Some(fail(&format!(
                "release_manifest --selftest: cannot read {}",
                real_abs.display()
            ))));
        };
        content.extend_from_slice(b"\n// release_manifest --selftest mutation marker\n");
        if let Err(e) = std::fs::write(&temp_copy, &content) {
            return Err(Some(fail(&format!(
                "release_manifest --selftest: cannot write {} ({})",
                temp_copy.display(),
                e.kind()
            ))));
        }
        let mutated_hash = sha256_hex(&content);
        if mutated_hash == target.sha256 {
            eprintln!("FAIL release_manifest --selftest: mutation did not change the file's hash");
            return Ok(ExitCode::FAILURE);
        }
        let baseline_values: Vec<Value> = baseline.iter().map(Record::to_value).collect();
        let mutated_values: Vec<Value> = baseline
            .iter()
            .map(|r| {
                let mut c = r.clone();
                if c.path == target.path {
                    c.sha256 = mutated_hash.clone();
                }
                c.to_value()
            })
            .collect();
        let Some(diff) = compare_manifests(&baseline_values, &mutated_values) else {
            return Err(None);
        };
        let flagged = diff.changed.iter().find(|(p, _)| *p == target.path);
        let bites = !diff.ok()
            && diff.missing.is_empty()
            && diff.added.is_empty()
            && diff.changed.len() == 1
            && flagged.is_some_and(|(_, reasons)| reasons.contains(&"sha256"));
        if !bites {
            eprintln!(
                "FAIL release_manifest --selftest: comparison logic did not flag mutated file {} as expected",
                target.path
            );
            // Node also dumps `JSON.stringify(diffResult)` here; the shape is
            // an internal debug aid on a path this port proves cannot be
            // reached, so it is not reconstructed.
            return Ok(ExitCode::FAILURE);
        }
        println!(
            "PASS release_manifest --selftest: comparison logic correctly flags a mutated file ({}) as sha256 mismatch, exit 1",
            target.path
        );
        Ok(ExitCode::SUCCESS)
    })();
    let _ = std::fs::remove_dir_all(&temp_dir);
    outcome
}

/// `fs.mkdtempSync(path.join(os.tmpdir(), "release-manifest-selftest-"))`.
fn tempdir_for_selftest() -> std::io::Result<PathBuf> {
    let base = std::env::temp_dir();
    for _ in 0..64 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = base.join(format!(
            "release-manifest-selftest-{}{}",
            std::process::id(),
            &sha256_hex(nanos.to_string().as_bytes())[..6]
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::other("mkdtemp exhausted"))
}

pub(super) fn run(args: &[&str]) -> Option<ExitCode> {
    let root = super::bee_source_root()?;
    // provenance: main() — `["--write","--check","--selftest"].filter(hasFlag)`,
    // where hasFlag is `args.includes(name)`; any other argument is ignored.
    let flags: Vec<&str> = ["--write", "--check", "--selftest"]
        .into_iter()
        .filter(|f| args.contains(f))
        .collect();
    if flags.len() != 1 {
        eprintln!("usage: bee dev release-manifest (--write | --check | --selftest)");
        return Some(ExitCode::FAILURE);
    }
    let result = match flags[0] {
        "--write" => run_write(&root),
        "--check" => run_check(&root),
        _ => run_selftest(&root),
    };
    match result {
        Ok(code) => Some(code),
        Err(code) => code, // None => delegate (nothing printed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// INVENTORY_ROOTS is READ by verbs/cells.rs to decide when a cell owes a
    /// manifest regen. If it drifts from what `build_current_records` actually
    /// enumerates, that obligation stops firing for the uncovered files — the
    /// exact silent-no-op failure the R6 cutover had to avoid. So: every path
    /// the builder produces must fall under a declared root.
    #[test]
    fn every_inventory_root_covers_what_the_builder_enumerates() {
        let root = repo_root();
        let records = match build_current_records(&root) {
            Ok(r) => r,
            Err(_) => return, // not a source checkout (packaged build) — nothing to prove
        };
        assert!(!records.is_empty(), "the live tree must produce a non-empty inventory");
        let uncovered: Vec<&str> = records
            .iter()
            .map(|r| r.path.as_str())
            .filter(|p| {
                !INVENTORY_ROOTS
                    .iter()
                    .any(|root| *p == *root || p.starts_with(&format!("{root}/")))
            })
            .collect();
        assert!(
            uncovered.is_empty(),
            "release_manifest INVENTORY_ROOTS does not cover {} enumerated path(s): {:?}. \
             verbs/cells.rs reads this list to raise the regen obligation, so an uncovered \
             path means a cell can edit a manifested file and never be told to regenerate.",
            uncovered.len(),
            &uncovered[..uncovered.len().min(10)]
        );
    }

    /// …and the converse: a root nobody enumerates is a root that would raise
    /// a regen obligation for files the manifest does not actually cover.
    /// `.bee/bin/bee` is the one deliberate exception — it is the unhashed
    /// artifact, present in the frame but never in `files`.
    #[test]
    fn every_inventory_root_is_actually_enumerated() {
        let root = repo_root();
        let records = match build_current_records(&root) {
            Ok(r) => r,
            Err(_) => return,
        };
        let unhashed: Vec<&str> = UNHASHED_ARTIFACTS.iter().map(|(p, _, _)| *p).collect();
        let idle: Vec<&&str> = INVENTORY_ROOTS
            .iter()
            .filter(|root| !unhashed.contains(*root))
            .filter(|root| {
                !records
                    .iter()
                    .any(|r| r.path == **root || r.path.starts_with(&format!("{root}/")))
            })
            .collect();
        assert!(
            idle.is_empty(),
            "INVENTORY_ROOTS declares {idle:?}, which the builder never enumerates - either the \
             root is stale (drop it) or the builder stopped covering it (a real regression)."
        );
    }

    /// The manifest path the regen obligation names must be the one the tool
    /// actually writes.
    /// The presence half of the `--check` contract. `cargo test` proves hash
    /// parity (`rebuild_reproduces_the_committed_manifest`); nothing else
    /// proves this half, so it is proven here rather than only in whatever
    /// tree happens to have a binary lying around.
    #[test]
    fn the_unhashed_artifact_check_bites_and_accepts_either_spelling() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Absent -> loud, and it names both the artifact and the fix.
        let failures = unhashed_artifact_failures(root);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains(".bee/bin/bee"), "{}", failures[0]);
        assert!(failures[0].contains("cargo build --release"), "{}", failures[0]);

        // Either spelling satisfies it — the same manifest is checked on
        // Windows and on POSIX.
        std::fs::create_dir_all(root.join(".bee").join("bin")).unwrap();
        for name in ["bee.exe", "bee"] {
            std::fs::write(root.join(".bee").join("bin").join(name), b"binary").unwrap();
            assert!(
                unhashed_artifact_failures(root).is_empty(),
                "{name} must satisfy the presence check"
            );
            std::fs::remove_file(root.join(".bee").join("bin").join(name)).unwrap();
        }

        // A DIRECTORY at the path is not a binary.
        std::fs::create_dir_all(root.join(".bee").join("bin").join("bee")).unwrap();
        assert_eq!(unhashed_artifact_failures(root).len(), 1);
    }

    #[test]
    fn manifest_rel_matches_the_path_the_tool_writes() {
        let root = Path::new("/repo");
        assert_eq!(rel_posix(root, &manifest_path(root)), MANIFEST_REL);
    }
    use crate::devtools::locale_compare;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("..")
    }

    #[test]
    fn mode_octal_is_three_digits() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("x");
        std::fs::write(&f, b"x").unwrap();
        let m = mode_octal(&std::fs::metadata(&f).unwrap());
        assert_eq!(m.len(), 3);
        assert!(m.chars().all(|c| ('0'..='7').contains(&c)));
        #[cfg(windows)]
        assert_eq!(m, "666");
    }

    // ── THE PIN: the committed manifest ───────────────────────────────────

    /// The equality proof, run against the real tree.
    ///
    /// The COMMITTED manifest is regenerated by hand and can legitimately lag
    /// the working tree — Node's own `--check` reports the same lag when it
    /// does, so an unconditional byte-equality assertion here would pin this
    /// port to whether someone remembered to re-run `--write`, not to whether
    /// the port is faithful. What IS asserted, unconditionally:
    ///
    ///   * the ENUMERATION — same path set, in the same order (this is the
    ///     localeCompare sort, the enumerate/exclude rules, and the role
    ///     assignment, all at once);
    ///   * every non-hash field of every record;
    ///   * that each differing sha256 is a REAL content difference (the file
    ///     on disk hashes to what this port computed), never a hashing bug;
    ///   * byte-equality of the whole file whenever the tree is in sync.
    #[test]
    fn rebuild_reproduces_the_committed_manifest() {
        let root = repo_root();
        if !manifest_path(&root).is_file() {
            return; // not a source checkout
        }
        let records = match build_current_records(&root) {
            Ok(r) => r,
            Err(BuildErr::Refuse(m)) => panic!("inventory refused: {m}"),
            Err(BuildErr::Nd) => panic!("inventory hit an unproven shape"),
        };
        let rebuilt = manifest_bytes(&records);
        let committed = std::fs::read_to_string(manifest_path(&root)).unwrap();
        let stored: Value = serde_json::from_str(&committed).unwrap();
        assert_eq!(stored["schemaVersion"], 2);
        let stored_files = stored["files"].as_array().unwrap();
        assert_eq!(
            stored_files.len(),
            records.len(),
            "record count drifted — the enumeration rules diverged"
        );
        let mut hash_lag = 0usize;
        for (i, (want, got)) in stored_files.iter().zip(records.iter()).enumerate() {
            assert_eq!(
                want["path"].as_str().unwrap(),
                got.path,
                "record {i}: path/order drifted (localeCompare sort)"
            );
            assert_eq!(want["mode"], got.mode, "record {i} ({}): mode", got.path);
            assert_eq!(want["role"], got.role, "record {i} ({}): role", got.path);
            assert_eq!(
                want.get("packagePath").cloned(),
                got.package_path.clone().map(Value::String),
                "record {i} ({}): packagePath",
                got.path
            );
            if want["sha256"].as_str() != Some(got.sha256.as_str()) {
                // Prove the difference is the file's, not the hasher's.
                let mut abs = root.clone();
                for seg in got.path.split('/') {
                    abs = abs.join(seg);
                }
                let on_disk = sha256_hex(&std::fs::read(&abs).unwrap());
                assert_eq!(
                    on_disk, got.sha256,
                    "record {i} ({}): this port's sha256 does not match the file on disk",
                    got.path
                );
                hash_lag += 1;
            }
        }
        if hash_lag == 0 {
            assert_eq!(rebuilt, committed, "manifest bytes drifted");
        } else {
            // Same condition Node's `--check` reports; recorded, not hidden.
            eprintln!(
                "note: committed manifest lags the tree on {hash_lag} file(s) — \
                 `bee dev release-manifest --check` reports the same set"
            );
        }
    }

    /// The localeCompare sort is load-bearing: a code-unit sort of the SAME
    /// path set produces a different file.
    #[test]
    fn code_unit_sort_would_not_reproduce_the_manifest() {
        let root = repo_root();
        if !manifest_path(&root).is_file() {
            return;
        }
        let stored: Value =
            serde_json::from_str(&std::fs::read_to_string(manifest_path(&root)).unwrap()).unwrap();
        let paths: Vec<String> = stored["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["path"].as_str().unwrap().to_string())
            .collect();
        let mut by_code_unit = paths.clone();
        super::super::js_default_sort(&mut by_code_unit);
        assert_ne!(
            by_code_unit, paths,
            "if these ever agree the localeCompare port is no longer being exercised"
        );
        let mut by_locale = paths.clone();
        assert!(sort_by_locale(&mut by_locale, |s| s.as_str()));
        assert_eq!(by_locale, paths, "stored order IS the localeCompare order");
    }

    // ── compareManifests ──────────────────────────────────────────────────

    fn rec(path: &str, sha: &str, role: &str) -> Value {
        Record {
            path: path.into(),
            sha256: sha.into(),
            mode: "666".into(),
            role: role.into(),
            package_path: None,
        }
        .to_value()
    }

    #[test]
    fn compare_reports_missing_added_and_every_changed_reason() {
        let stored = vec![
            rec("a.mjs", "h1", "runtime_lib"),
            rec("b.mjs", "h2", "runtime_lib"),
            rec("gone.mjs", "h3", "runtime_lib"),
        ];
        let current = vec![
            rec("a.mjs", "h1", "runtime_lib"),
            rec("b.mjs", "CHANGED", "package_payload"),
            rec("new.mjs", "h4", "runtime_lib"),
        ];
        let diff = compare_manifests(&stored, &current).unwrap();
        assert!(!diff.ok());
        assert_eq!(diff.missing, ["gone.mjs"]);
        assert_eq!(diff.added, ["new.mjs"]);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].0, "b.mjs");
        assert_eq!(diff.changed[0].1, ["sha256", "role"]);
    }

    #[test]
    fn compare_treats_absent_package_path_as_null() {
        let mut with = Record {
            path: "a".into(),
            sha256: "h".into(),
            mode: "666".into(),
            role: "r".into(),
            package_path: Some("a".into()),
        };
        let without = Record { package_path: None, ..with.clone() };
        let d = compare_manifests(&[with.to_value()], &[without.to_value()]).unwrap();
        assert_eq!(d.changed[0].1, ["packagePath"]);
        // …and an explicit null equals absent (`?? null`).
        with.package_path = None;
        let mut explicit_null = with.to_value();
        explicit_null
            .as_object_mut()
            .unwrap()
            .insert("packagePath".into(), Value::Null);
        let d = compare_manifests(&[explicit_null], &[with.to_value()]).unwrap();
        assert!(d.ok());
    }

    #[test]
    fn identical_inputs_compare_clean() {
        let a = vec![rec("x", "h", "r"), rec("y", "h2", "r")];
        assert!(compare_manifests(&a, &a.clone()).unwrap().ok());
    }

    /// The selftest's own claim, proven on a fixture: a single mutated hash
    /// must produce exactly one `changed` entry naming sha256.
    #[test]
    fn selftest_bite_condition_holds() {
        let baseline = vec![rec("a", "h1", "package_payload"), rec("b", "h2", "runtime_lib")];
        let mut mutated = baseline.clone();
        mutated[0].as_object_mut().unwrap().insert("sha256".into(), Value::String("h9".into()));
        let diff = compare_manifests(&baseline, &mutated).unwrap();
        let bites = !diff.ok()
            && diff.missing.is_empty()
            && diff.added.is_empty()
            && diff.changed.len() == 1
            && diff.changed[0].1.contains(&"sha256");
        assert!(bites);
    }

    #[test]
    fn manifest_bytes_shape_matches_node() {
        let records = vec![Record {
            path: "a/b.md".into(),
            sha256: "deadbeef".into(),
            mode: "666".into(),
            role: "package_payload".into(),
            package_path: None,
        }];
        // Key ORDER is load-bearing: JSON.stringify emits insertion order and
        // the committed file is byte-compared. schemaVersion, then the R6
        // unhashedArtifacts block, then files.
        let bytes = manifest_bytes(&records);
        assert!(bytes.starts_with("{\n  \"schemaVersion\": 2,\n  \"unhashedArtifacts\": [\n"), "{bytes}");
        assert!(bytes.contains("\"path\": \".bee/bin/bee\""), "{bytes}");
        assert!(bytes.contains("\"check\": \"presence\""), "{bytes}");
        assert!(
            bytes.ends_with(
                "  \"files\": [\n    {\n      \"path\": \"a/b.md\",\n      \"sha256\": \"deadbeef\",\n      \"mode\": \"666\",\n      \"role\": \"package_payload\"\n    }\n  ]\n}\n"
            ),
            "{bytes}"
        );
    }

    #[test]
    fn enumerate_tree_excludes_only_immediate_children() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("pkg").join("hooks")).unwrap();
        std::fs::create_dir_all(root.join("pkg").join("lib").join("hooks")).unwrap();
        std::fs::write(root.join("pkg").join("top.mjs"), b"1").unwrap();
        std::fs::write(root.join("pkg").join("hooks").join("h.mjs"), b"2").unwrap();
        std::fs::write(root.join("pkg").join("lib").join("hooks").join("deep.mjs"), b"3").unwrap();
        let recs = enumerate_tree(root, &root.join("pkg"), "package_payload", &["hooks"])
            .map_err(|_| ())
            .unwrap();
        let paths: Vec<&str> = recs.iter().map(|r| r.path.as_str()).collect();
        assert_eq!(paths, ["pkg/lib/hooks/deep.mjs", "pkg/top.mjs"]);
        assert_eq!(recs[0].package_path.as_deref(), Some("pkg/lib/hooks/deep.mjs"));
    }

    #[test]
    fn a_missing_directory_is_a_deterministic_refusal() {
        let tmp = tempfile::tempdir().unwrap();
        match enumerate_flat_dir(tmp.path(), &tmp.path().join("nope"), "r", ".md") {
            Err(BuildErr::Refuse(m)) => {
                assert!(m.starts_with("release_manifest: expected directory missing: "))
            }
            _ => panic!("expected a deterministic refusal"),
        }
    }

    #[test]
    fn changed_uses_locale_order_while_missing_uses_code_units() {
        // `_z` vs `Az`: code units put `A` (0x41) first, ICU puts `_` first.
        let stored = vec![rec("Az", "h", "r"), rec("_z", "h", "r")];
        let current: Vec<Value> = vec![];
        let diff = compare_manifests(&stored, &current).unwrap();
        assert_eq!(diff.missing, ["Az", "_z"]); // code units
        let current2 = vec![rec("Az", "X", "r"), rec("_z", "X", "r")];
        let diff2 = compare_manifests(&stored, &current2).unwrap();
        let order: Vec<&str> = diff2.changed.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(order, ["_z", "Az"]); // localeCompare
        assert_eq!(locale_compare("_z", "Az"), Some(std::cmp::Ordering::Less));
    }
}
