// bee dev release-manifest — Rust port of scripts/release_manifest.mjs
// (DIST-01/DIST-03/D-03, decision ed0b2920).
//
// Enumerates the release-identity file set for the bee distribution and
// hashes it. `--write` regenerates
// docs/history/codex-harness-hardening/release-manifest.json, `--check`
// recomputes and compares (the shape run_verify.mjs runs on every verify),
// `--selftest` proves the comparison logic actually bites.
//
// PROVENANCE: every function names its .mjs source. Two details carry the
// whole byte-identity of the output file:
//
//   1. THE SORT. `records.sort((a, b) => a.path.localeCompare(b.path))` is
//      ICU collation, not code units. Sorting the real 326-path set by code
//      unit produces a DIFFERENT file (measured), so devtools::locale_compare
//      is mandatory here — see its header for the proof. `compareManifests`
//      mixes the two comparators deliberately: `missing`/`added` use bare
//      `.sort()` (code units), `changed` uses localeCompare. Both are
//      reproduced as written.
//   2. THE MODE. `statSync(p).mode & 0o777` — on Windows libuv synthesises
//      0666 (0444 when FILE_ATTRIBUTE_READONLY), which is what the committed
//      manifest carries; on Unix it is the real permission bits.
//
// STRANGLER ROUTING. Exactly one of --write/--check/--selftest, matched with
// the .mjs's own `args.includes(...)` semantics (so an extra unrecognised
// argument is ignored there and here). Failures that are DETERMINISTIC in
// Node (`throw new Error("release_manifest: …")`, surfaced as
// `FAIL release_manifest: ${error.message}`) are reproduced byte for byte;
// failures whose text WAS a V8/libuv message (a corrupt stored manifest, an
// unreadable file) refuse natively at cutover, in the same
// `FAIL release_manifest: …` shape and with the same exit code, worded by us.
// The one surviving None is a path outside the proven collation alphabet.

use super::{rel_posix, sha256_hex, sort_by_locale};
use crate::jsjson;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// provenance: release_manifest.mjs SCHEMA_VERSION.
const SCHEMA_VERSION: u64 = 1;

/// provenance: release_manifest.mjs MANIFEST_PATH.
fn manifest_path(root: &Path) -> PathBuf {
    root.join("docs")
        .join("history")
        .join("codex-harness-hardening")
        .join("release-manifest.json")
}

/// A deterministic `throw new Error(...)` from the .mjs (reproduced), versus
/// a V8/libuv-worded failure (delegated as None before any output).
enum BuildErr {
    /// `FAIL release_manifest: ${message}` + exit 1.
    Refuse(String),
    /// Unproven bytes — the probe returns None. CUTOVER: every I/O arm that
    /// used this became a `Refuse` (see `io_refuse`). What remains is
    /// `sort_records`, whose subject is `localeCompare` collation over free
    /// prose — a different delegate class from V8 message text, and the one
    /// this sweep deliberately left alone.
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

/// provenance: release_manifest.mjs modeOctal —
/// `(statSync(p).mode & 0o777).toString(8).padStart(3, "0")`.
fn mode_octal(meta: &std::fs::Metadata) -> String {
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

/// A filesystem failure, worded by us. Node let the libuv error throw, which
/// is why every one of these used to be `BuildErr::Nd` (delegate). The error
/// KIND is named rather than the OS message string, which varies by platform
/// and locale.
fn io_refuse(action: &str, path: &Path, err: &std::io::Error) -> BuildErr {
    BuildErr::Refuse(format!(
        "release_manifest: cannot {action} {} ({})",
        path.display(),
        err.kind()
    ))
}

/// provenance: release_manifest.mjs buildRecord (+ sha256File).
fn build_record(root: &Path, abs: &Path, role: &str, with_package_path: bool) -> R<Record> {
    let data = std::fs::read(abs).map_err(|e| io_refuse("read", abs, &e))?;
    let meta = std::fs::metadata(abs).map_err(|e| io_refuse("stat", abs, &e))?;
    let path = rel_posix(root, abs);
    Ok(Record {
        sha256: sha256_hex(&data),
        mode: mode_octal(&meta),
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

/// provenance: release_manifest.mjs enumerateTree — recursive, `packagePath`
/// set, `excludeTopDirNames` skipping only IMMEDIATE children.
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

/// provenance: release_manifest.mjs enumerateFlatDir — files with `ext`
/// directly inside `dir` (no recursion), no `packagePath`, sorted.
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

/// provenance: release_manifest.mjs buildCurrentRecords — the whole
/// inventory, in the .mjs's own concatenation order, then sorted, then
/// duplicate-checked.
fn build_current_records(root: &Path) -> R<Vec<Record>> {
    let mut records: Vec<Record> = Vec::new();
    records.extend(enumerate_flat_dir(
        root,
        &root.join(".bee").join("bin").join("lib"),
        "runtime_lib",
        ".mjs",
    )?);
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
    for abs in [
        root.join("scripts").join("tests").join("test_verify_manifest.mjs"),
        root.join("scripts").join("tests").join("test_release_tuple.mjs"),
    ] {
        records.push(build_record(root, &abs, "distribution_test", false)?);
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

/// provenance: release_manifest.mjs writeManifestFile — `{schemaVersion,
/// files}` as `JSON.stringify(m, null, 2) + "\n"`.
fn manifest_bytes(records: &[Record]) -> String {
    let mut m = Map::new();
    m.insert("schemaVersion".into(), Value::Number(SCHEMA_VERSION.into()));
    m.insert(
        "files".into(),
        Value::Array(records.iter().map(Record::to_value).collect()),
    );
    format!("{}\n", jsjson::stringify_pretty(&Value::Object(m)))
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

/// provenance: release_manifest.mjs compareManifests. NOTE the two different
/// sorts: `missing`/`added` use bare `.sort()` (code units), `changed` uses
/// `localeCompare`.
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
    match parsed.get("files") {
        Some(Value::Array(files)) => Ok(files.clone()),
        _ => Err(Some(fail(&format!(
            "release_manifest: stored manifest malformed: {}",
            file.display()
        )))),
    }
}

fn run_check(root: &Path) -> Result<ExitCode, Option<ExitCode>> {
    let stored = read_stored(root)?;
    let current = resolve(build_current_records(root))?;
    let current_values: Vec<Value> = current.iter().map(Record::to_value).collect();
    let Some(diff) = compare_manifests(&stored, &current_values) else { return Err(None) };
    if diff.ok() {
        println!(
            "release_manifest --check: {} file(s) match stored manifest",
            current.len()
        );
        return Ok(ExitCode::SUCCESS);
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

/// provenance: release_manifest.mjs runSelftest — take the REAL manifest as a
/// baseline, mutate ONE covered file's content in a temp copy (never the real
/// tree), and assert compareManifests flags exactly that file.
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
        assert_eq!(stored["schemaVersion"], 1);
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
            path: "a/b.mjs".into(),
            sha256: "deadbeef".into(),
            mode: "666".into(),
            role: "runtime_lib".into(),
            package_path: None,
        }];
        assert_eq!(
            manifest_bytes(&records),
            "{\n  \"schemaVersion\": 1,\n  \"files\": [\n    {\n      \"path\": \"a/b.mjs\",\n      \"sha256\": \"deadbeef\",\n      \"mode\": \"666\",\n      \"role\": \"runtime_lib\"\n    }\n  ]\n}\n"
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
