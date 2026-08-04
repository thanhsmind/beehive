// bee tmp — native port of the tmp verb group (bee.mjs handleTmpSweep +
// lib/scratch.mjs runSweep/computeSweepPlan and friends).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   tmp sweep [--feature F] [--before ISO] [--all] [--dry-run] [--json]
// Nothing in this group is left permanently delegated. Within the accepted
// shapes the deterministic refusals are served natively byte-identical: the
// no-flag "no default purge" refusal and the invalid --before ISO refusal.
//
// CUTOVER (2026-08-01): a corrupt .bee/lanes/*.json no longer delegates. It
// warns (crate::fsutil::warn_corrupt_json, then readLane's own
// skipping-corrupt-lane-record line, both of which Node printed) and reads as
// "no lane" — `readJson(file, null)`'s own fallback, so the sweep plan is
// unchanged. A corrupt .bee/state.json fails open inside
// crate::state::read_state_brief, which was converted in its own file.
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens, --all=<not true/false>
//   - --before values outside the strict-ISO subset js_date_parse proves
//     (V8's legacy Date fallback could still accept them)
//   - paths that cannot be represented as UTF-8 strings for the result JSON
//
// DIVERGENCE NOTES (documented, unreachable-different for real bee data):
//   - a mid-sweep fs.rmSync failure surfaces a Rust io message where Node
//     would print the V8 message (removals already performed forbid
//     delegation at that point).
//   - realpath uses dunce::canonicalize, which mirrors Node's
//     fs.realpathSync.native (\\?\-prefix-free) for every path bee touches.
//   - on Windows every reparse point (junction included) counts as a
//     symlink, matching libuv's lstat classification.
//
// SAFETY: the port keeps scratch.mjs's whole doctrine — the literal
// (non-symlinked) roots are the only delete authority, every candidate is
// canonically re-proved contained immediately before removal, and an
// escaping candidate is refused, never followed.
//
// Provenance: bee.mjs handleTmpSweep, lib/scratch.mjs (SCRATCH_TMP_REL/
// SCRATCH_SPIKES_REL/TERMINAL_PHASES/realpathOrNull/isUnderRoot/
// literalRootInfo/scratchRoots/inspectScratchRoots/containedRoot/dirSize/
// countFiles/isLiveFeature/hasRecord/listEntries/FEATURE_PREFIX_SEPARATORS/
// matchFeature/computeSweepPlan/runSweep), lib/state.mjs readState/readLane
// (phase slice).

use crate::fsutil::{read_json, ReadJson};
use crate::state::read_state_brief;
use crate::verbs::knowledge::{g_prelude, js_bool_flag, pre_json_scan, GPre};
use crate::verbs::reservations::{js_date_parse, js_trim, keys_known, now_ms, parse_flags, FlagV};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const SCRATCH_ROOT_RELS: [&str; 2] = [".bee/tmp", ".bee/spikes"];

fn is_terminal_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if s == "idle" || s == "compounding-complete")
}

/// lstat-level symlink test matching Node's isSymbolicLink(): on Windows any
/// reparse point (symlink OR junction) counts, like libuv.
fn md_is_symlink(md: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        (md.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(not(windows))]
    {
        md.file_type().is_symlink()
    }
}

fn realpath_or_null(p: &Path) -> Option<PathBuf> {
    dunce::canonicalize(p).ok()
}

/// isUnderRoot — the real root itself counts; anything else must be a strict
/// descendant. Case-insensitive on Windows like path.relative's win32 compare.
fn is_under_root(parent: &Path, child: &Path) -> bool {
    let norm = |p: &Path| -> Vec<String> {
        p.components()
            .map(|c| {
                let s = c.as_os_str().to_string_lossy().into_owned();
                if cfg!(windows) {
                    s.to_lowercase()
                } else {
                    s
                }
            })
            .collect()
    };
    let pv = norm(parent);
    let cv = norm(child);
    cv.len() >= pv.len() && cv[..pv.len()] == pv[..]
}

// ─── state / lane phase reads (the slice the sweep predicates need) ────────

struct StateLite {
    phase: Value,
    feature: Value,
}

/// readState(root) — fail-open defaults. A corrupt file warns and reads as
/// defaults inside crate::state::read_state_brief (converted in its own
/// file), so `Err(())` is now unreachable for that cause.
fn read_state_lite(root: &Path) -> Result<StateLite, ()> {
    let brief = read_state_brief(root);
    Ok(StateLite { phase: brief.phase, feature: brief.feature })
}

/// readLane(root, feature) — fail-open display read; Ok(Some(phase)) when a
/// matching lane record exists. CUTOVER: corrupt JSON no longer delegates —
/// it warns (readJson's line, then readLane's own) and reads as "no lane",
/// which is readJson's `null` fallback. The mismatched-record console.warn is
/// replicated byte-identically.
fn read_lane_phase(root: &Path, feature: &str) -> Result<Option<Value>, ()> {
    // requireLaneFeature (throws are caught fail-open in readLane -> null).
    let trimmed = js_trim(feature);
    if trimmed.is_empty() || trimmed.contains(['\\', '/']) || trimmed.contains("..") {
        return Ok(None);
    }
    let file = root.join(".bee").join("lanes").join(format!("{trimmed}.json"));
    if !file.exists() {
        return Ok(None);
    }
    let parsed = match read_json(&file) {
        ReadJson::Missing => return Ok(None),
        // CUTOVER: readJson's `null` fallback makes laneRecordFrom answer
        // null, so Node printed ITS warning and then readLane's own line and
        // returned null. Both lines are reproduced; the answer is unchanged.
        ReadJson::Corrupt => {
            crate::fsutil::warn_corrupt_json(&file);
            let rel = format!(
                ".bee{sep}lanes{sep}{trimmed}.json",
                sep = std::path::MAIN_SEPARATOR
            );
            eprintln!(
                "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
            );
            return Ok(None);
        }
        ReadJson::Parsed(v) => v,
    };
    // laneRecordFrom: object, feature field must strictly equal the trimmed name.
    let record_ok = matches!(&parsed, Value::Object(_))
        && parsed.get("feature").unwrap_or(&Value::Null) == &Value::String(trimmed.to_string());
    if !record_ok {
        let rel = format!(
            ".bee{sep}lanes{sep}{trimmed}.json",
            sep = std::path::MAIN_SEPARATOR
        );
        eprintln!(
            "readLane: skipping corrupt lane record \"{rel}\" for display — mutations through readLaneStrict will refuse loudly. FIX: inspect/restore the file (e.g. \"git checkout -- {rel}\")."
        );
        return Ok(None);
    }
    // merged.phase = parsed.phase (key present wins) else 'idle'; legacy coerce.
    let mut phase = match parsed.get("phase") {
        Some(v) => v.clone(),
        None => Value::String("idle".to_string()),
    };
    if phase == Value::String("validating".to_string()) {
        phase = Value::String("planning".to_string());
    }
    Ok(Some(phase))
}

fn is_live_feature(root: &Path, state: &StateLite, name: &str) -> Result<bool, ()> {
    if &state.feature == &Value::String(name.to_string()) && !is_terminal_phase(&state.phase) {
        return Ok(true);
    }
    match read_lane_phase(root, name)? {
        Some(phase) => Ok(!is_terminal_phase(&phase)),
        None => Ok(false),
    }
}

fn has_record(root: &Path, state: &StateLite, name: &str) -> Result<bool, ()> {
    if &state.feature == &Value::String(name.to_string()) {
        return Ok(true);
    }
    Ok(read_lane_phase(root, name)?.is_some())
}

// ─── literal roots + containment (the safety core) ─────────────────────────

struct RootInfo {
    rel: &'static str,
    abs: PathBuf,
    real: PathBuf,
}

struct RefusedRoot {
    rel: &'static str,
    path: PathBuf,
    reason: &'static str,
}

/// literalRootInfo — walk rel one segment at a time from the repo's realpath,
/// lstat-ing WITHOUT resolving symlinks; any symlinked segment refuses the
/// whole root.
fn literal_root_info(root_real: &Path, rel: &'static str) -> Result<RootInfo, RefusedRoot> {
    let mut cursor = root_real.to_path_buf();
    for seg in rel.split('/') {
        cursor.push(seg);
        let md = match std::fs::symlink_metadata(&cursor) {
            Ok(md) => md,
            Err(_) => {
                return Err(RefusedRoot { rel, path: join_root(root_real, rel), reason: "missing" })
            }
        };
        if md_is_symlink(&md) {
            return Err(RefusedRoot { rel, path: join_root(root_real, rel), reason: "symlinked_root" });
        }
        if !md.is_dir() {
            return Err(RefusedRoot { rel, path: join_root(root_real, rel), reason: "not_a_directory" });
        }
    }
    Ok(RootInfo { rel, abs: cursor.clone(), real: cursor })
}

fn join_root(root_real: &Path, rel: &str) -> PathBuf {
    let mut p = root_real.to_path_buf();
    for seg in rel.split('/') {
        p.push(seg);
    }
    p
}

struct RootsInspection {
    roots: Vec<RootInfo>,
    refused: Vec<RefusedRoot>,
}

fn inspect_scratch_roots(root: &Path) -> RootsInspection {
    let Some(root_real) = realpath_or_null(root) else {
        return RootsInspection { roots: Vec::new(), refused: Vec::new() };
    };
    let mut roots = Vec::new();
    let mut refused = Vec::new();
    for rel in SCRATCH_ROOT_RELS {
        match literal_root_info(&root_real, rel) {
            Ok(info) => roots.push(info),
            Err(r) => {
                if r.reason != "missing" {
                    refused.push(r);
                }
            }
        }
    }
    RootsInspection { roots, refused }
}

/// containedRoot — canonical containment proof; None refuses (never follows).
fn contained_root<'a>(candidate: &Path, roots: &'a [RootInfo]) -> Option<&'a RootInfo> {
    let real = realpath_or_null(candidate)?;
    roots.iter().find(|r| is_under_root(&r.real, &real))
}

// ─── sizing (never dereferences a symlink) ─────────────────────────────────

fn dir_size(abs: &Path) -> u64 {
    let Ok(md) = std::fs::symlink_metadata(abs) else { return 0 };
    if md_is_symlink(&md) {
        return 0;
    }
    if !md.is_dir() {
        return md.len();
    }
    let Ok(entries) = std::fs::read_dir(abs) else { return 0 };
    entries.flatten().map(|e| dir_size(&e.path())).sum()
}

fn count_files(abs: &Path) -> u64 {
    let Ok(md) = std::fs::symlink_metadata(abs) else { return 0 };
    if md_is_symlink(&md) {
        return 0;
    }
    if !md.is_dir() {
        return 1;
    }
    let Ok(entries) = std::fs::read_dir(abs) else { return 0 };
    entries.flatten().map(|e| count_files(&e.path())).sum()
}

// ─── feature matching ──────────────────────────────────────────────────────

/// matchFeature — exact name always; `<feature><sep>...` prefix unless the
/// sibling is itself live. `Err(())` is now unreachable through the lane read
/// (a corrupt record reads as absent); the fallible signature is kept for the
/// caller's `?` plumbing.
fn match_feature(
    root: &Path,
    state: &StateLite,
    name: &str,
    feature: &str,
) -> Result<(bool, Option<&'static str>), ()> {
    if name == feature {
        return Ok((true, None));
    }
    if !name.starts_with(feature) {
        return Ok((false, None));
    }
    let next = name[feature.len()..].chars().next();
    if !matches!(next, Some('-' | '.' | '_')) {
        return Ok((false, None));
    }
    if is_live_feature(root, state, name)? {
        return Ok((false, Some("live_sibling")));
    }
    Ok((true, None))
}

// ─── plan + execution ──────────────────────────────────────────────────────

struct Included {
    scratch_root: &'static str,
    name: String,
    path: PathBuf,
    bytes: u64,
    files: u64,
}

struct SweepPlan {
    included: Vec<Included>,
    skipped: Vec<(String, String, Option<&'static str>)>, // (scratchRoot, name, reason)
    refused_escapes: Vec<(String, String, PathBuf)>,      // (scratchRoot, name, path)
    refused_roots: Vec<RefusedRoot>,
}

enum SweepErr {
    Delegate,
    Thrown(String),
}

fn compute_sweep_plan(
    root: &Path,
    feature: Option<&str>,
    before: Option<&str>,
    all: bool,
) -> Result<SweepPlan, SweepErr> {
    let inspection = inspect_scratch_roots(root);
    let before_ms: Option<f64> = match before.filter(|b| !b.is_empty()) {
        None => None,
        Some(b) => match js_date_parse(b) {
            Err(_) => return Err(SweepErr::Delegate), // V8's legacy fallback might parse it
            Ok(None) => {
                return Err(SweepErr::Thrown(format!(
                    "tmp sweep: --before \"{b}\" is not a valid ISO date."
                )))
            }
            Ok(Some(ms)) => Some(ms),
        },
    };
    let state = read_state_lite(root).map_err(|_| SweepErr::Delegate)?;

    let mut plan = SweepPlan {
        included: Vec::new(),
        skipped: Vec::new(),
        refused_escapes: Vec::new(),
        refused_roots: inspection.refused,
    };

    for root_info in &inspection.roots {
        let entries = match std::fs::read_dir(&root_info.abs) {
            Ok(rd) => rd,
            Err(_) => continue, // listEntries catch -> []
        };
        for entry in entries.flatten() {
            let name = match entry.file_name().to_str() {
                Some(n) => n.to_string(),
                None => return Err(SweepErr::Delegate),
            };
            let abs_path = root_info.abs.join(&name);
            let proof = contained_root(&abs_path, &inspection.roots);
            if proof.is_none() {
                plan.refused_escapes
                    .push((root_info.rel.to_string(), name, abs_path));
                continue;
            }

            let (qualifies, skip_reason): (bool, Option<&'static str>) = if all {
                (true, None)
            } else if let Some(feature) = feature.filter(|f| !f.is_empty()) {
                let (q, r) = match_feature(root, &state, &name, feature).map_err(|_| SweepErr::Delegate)?;
                (q, r)
            } else {
                let live = is_live_feature(root, &state, &name).map_err(|_| SweepErr::Delegate)?;
                if live {
                    (false, Some("live"))
                } else if has_record(root, &state, &name).map_err(|_| SweepErr::Delegate)? {
                    (true, None)
                } else if let Some(before_ms) = before_ms {
                    let mtime_ms = std::fs::metadata(&abs_path)
                        .ok()
                        .and_then(|md| md.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs_f64() * 1000.0)
                        .unwrap_or_else(now_ms);
                    if mtime_ms < before_ms {
                        (true, None)
                    } else {
                        (false, Some("absent_not_old_enough"))
                    }
                } else {
                    (false, Some("absent_no_before"))
                }
            };

            if !qualifies {
                let default_mode = !all && feature.filter(|f| !f.is_empty()).is_none();
                if default_mode || skip_reason == Some("live_sibling") {
                    plan.skipped.push((root_info.rel.to_string(), name, skip_reason));
                }
                continue;
            }

            plan.included.push(Included {
                scratch_root: root_info.rel,
                name,
                bytes: dir_size(&abs_path),
                files: count_files(&abs_path),
                path: abs_path,
            });
        }
    }
    Ok(plan)
}

fn rm_recursive_force(path: &Path) -> std::io::Result<()> {
    let md = match std::fs::symlink_metadata(path) {
        Ok(md) => md,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()), // force: true
        Err(e) => return Err(e),
    };
    if md.is_dir() && !md_is_symlink(&md) {
        std::fs::remove_dir_all(path)
    } else if cfg!(windows) && md.is_dir() {
        // a directory-symlink/junction entry: remove the link itself
        std::fs::remove_dir(path)
    } else {
        std::fs::remove_file(path)
    }
}

fn path_str(p: &Path) -> Option<String> {
    p.to_str().map(str::to_string)
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "tmp" {
        return None;
    }
    if args.get(1)?.to_str()? != "sweep" {
        return None; // unknown verbs → group-usage fallback stays Node's
    }
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    if !keys_known(&flags, &["feature", "before", "all", "dry-run"]) {
        return None;
    }
    let all = js_bool_flag(&flags, "all")?;
    let dry_run = js_bool_flag(&flags, "dry-run")?;
    // `flags.feature !== undefined ? String(flags.feature) : undefined`
    let feature = match flags.get("feature") {
        Some(FlagV::S(s)) => Some(s.clone()),
        Some(FlagV::Present) => None, // unreachable: not a FLAG_ALONE_BOOLEAN
        None => None,
    };
    let before = match flags.get("before") {
        Some(FlagV::S(s)) => Some(s.clone()),
        _ => None,
    };

    let ctx = match g_prelude("tmp sweep", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };

    // "no default purge" refusal (typed, zero mutation).
    let feature_truthy = feature.as_deref().map(|f| !f.is_empty()).unwrap_or(false);
    let before_truthy = before.as_deref().map(|b| !b.is_empty()).unwrap_or(false);
    if !feature_truthy && !before_truthy && !all && !dry_run {
        return Some(ctx.fail(
            "tmp sweep requires at least one of --feature/--before/--all/--dry-run — no default purge (same discipline as `decisions archive`). FIX: pass --dry-run to preview the default (closed/absent-feature) target set, --feature <slug> to target one feature explicitly (even a live one), --before <ISO> to age-gate scratch with no feature/lane record, or --all to clear everything.",
        ));
    }

    let plan = match compute_sweep_plan(&ctx.root, feature.as_deref(), before.as_deref(), all) {
        Ok(p) => p,
        Err(SweepErr::Delegate) => return None,
        Err(SweepErr::Thrown(msg)) => return Some(ctx.fail(&msg)),
    };

    // runSweep: re-prove containment immediately before each removal (fresh
    // root inspection, dry-run or not), then remove.
    let fresh = inspect_scratch_roots(&ctx.root);
    let mut removed: Vec<Value> = Vec::new();
    let mut refused_escapes = plan.refused_escapes;
    let mut bytes_freed: u64 = 0;
    let mut files_freed: u64 = 0;
    for candidate in &plan.included {
        if contained_root(&candidate.path, &fresh.roots).is_none() {
            refused_escapes.push((candidate.scratch_root.to_string(), candidate.name.clone(), candidate.path.clone()));
            continue;
        }
        if !dry_run {
            if let Err(e) = rm_recursive_force(&candidate.path) {
                // DIVERGENCE (header note): removals already performed forbid
                // delegation — the Rust io message stands in for V8's.
                return Some(ctx.fail(&e.to_string()));
            }
        }
        let mut m = Map::new();
        m.insert("scratchRoot".into(), Value::String(candidate.scratch_root.to_string()));
        m.insert("name".into(), Value::String(candidate.name.clone()));
        m.insert("bytes".into(), Value::from(candidate.bytes));
        m.insert("files".into(), Value::from(candidate.files));
        removed.push(Value::Object(m));
        bytes_freed += candidate.bytes;
        files_freed += candidate.files;
    }

    let mut result = Map::new();
    result.insert("dry_run".into(), Value::Bool(dry_run));
    let removed_len = removed.len();
    result.insert("removed".into(), Value::Array(removed));
    result.insert("bytes_freed".into(), Value::from(bytes_freed));
    result.insert("files_freed".into(), Value::from(files_freed));
    result.insert(
        "skipped".into(),
        Value::Array(
            plan.skipped
                .into_iter()
                .map(|(scratch_root, name, reason)| {
                    let mut m = Map::new();
                    m.insert("scratchRoot".into(), Value::String(scratch_root));
                    m.insert("name".into(), Value::String(name));
                    m.insert(
                        "reason".into(),
                        reason.map(|r| Value::String(r.to_string())).unwrap_or(Value::Null),
                    );
                    Value::Object(m)
                })
                .collect(),
        ),
    );
    let mut escapes_json = Vec::new();
    for (scratch_root, name, path) in refused_escapes {
        let mut m = Map::new();
        m.insert("scratchRoot".into(), Value::String(scratch_root));
        m.insert("name".into(), Value::String(name));
        m.insert("path".into(), Value::String(path_str(&path)?));
        escapes_json.push(Value::Object(m));
    }
    result.insert("refused_escapes".into(), Value::Array(escapes_json));
    let mut roots_json = Vec::new();
    for r in plan.refused_roots {
        let mut m = Map::new();
        m.insert("rel".into(), Value::String(r.rel.to_string()));
        m.insert("path".into(), Value::String(path_str(&r.path)?));
        m.insert("reason".into(), Value::String(r.reason.to_string()));
        roots_json.push(Value::Object(m));
    }
    result.insert("refused_roots".into(), Value::Array(roots_json));

    let verb = if dry_run { "Would remove" } else { "Removed" };
    let text = format!(
        "{verb} {removed_len} scratch entr(y|ies) ({bytes_freed} bytes, {files_freed} files) from .bee/tmp/ and .bee/spikes/."
    );
    Some(ctx.emit(&Value::Object(result), &text, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn setup_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee").join("tmp")).unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee").join("spikes")).unwrap();
        tmp
    }

    fn write_state(root: &Path, content: &str) {
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(root.join(".bee").join("state.json"), content).unwrap();
    }

    fn write_lane(root: &Path, feature: &str, content: &str) {
        let dir = root.join(".bee").join("lanes");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{feature}.json")), content).unwrap();
    }

    /// A fixture the WHOLE command can run against: `resolve_store_root` needs
    /// an onboarding marker before g_prelude will hand a verb a root.
    fn setup_cli_repo() -> tempfile::TempDir {
        let repo = setup_repo();
        std::fs::write(
            repo.path().join(".bee").join("onboarding.json"),
            r#"{"schema_version":"1.0","bee_version":"0.1.0"}"#,
        )
        .unwrap();
        write_state(repo.path(), r#"{"phase":"idle","feature":null}"#);
        repo
    }

    fn scratch_dir(root: &Path, scratch_rel: &str) -> PathBuf {
        let mut p = root.to_path_buf();
        for seg in scratch_rel.split('/') {
            p.push(seg);
        }
        p
    }

    /// test_scratch.mjs makeScratchDir — `files` are (relative path, content).
    fn make_scratch(root: &Path, scratch_rel: &str, name: &str, files: &[(&str, &str)]) {
        let dir = scratch_dir(root, scratch_rel).join(name);
        for (rel, content) in files {
            let mut file = dir.clone();
            for seg in rel.split('/') {
                file.push(seg);
            }
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(&file, content).unwrap();
        }
    }

    // ─── whole-command harness ─────────────────────────────────────────────
    //
    // `try_native` runs the real dispatch frame, and g_prelude resolves the
    // repo root from the PROCESS cwd. Mutating cwd in-process would race every
    // other test sharing this binary, so each command runs in a child copy of
    // THIS test binary (always the freshly built code under test — never a
    // possibly stale target/<profile>/bee executable) with cwd set to the
    // fixture. The child brackets bee's own streams with markers so the parent
    // can lift them out of libtest's chatter.

    const ARGV_ENV: &str = "BEE_RS_TMP_GROUP_TEST_ARGV";
    const CHILD_TEST: &str = "verbs::tmp_group::tests::tmp_sweep_child_process";
    const OUT_OPEN: &str = "<<<bee-stdout";
    const OUT_CLOSE: &str = "bee-stdout>>>";
    const ERR_OPEN: &str = "<<<bee-stderr";
    const ERR_CLOSE: &str = "bee-stderr>>>";
    /// Tripwire: printed only when the router declined and Node would have
    /// served the command. `run_sweep` refuses to let that pass for a green.
    const DELEGATED: &str = "__DELEGATED_TO_NODE__";

    #[test]
    #[ignore = "child process of run_sweep(): needs a fixture cwd, never runs in-process"]
    fn tmp_sweep_child_process() -> ExitCode {
        let raw = std::env::var(ARGV_ENV).expect("child spawned without an argv env var");
        let argv: Vec<OsString> = raw.split('\u{1f}').map(OsString::from).collect();
        println!("{OUT_OPEN}");
        eprintln!("{ERR_OPEN}");
        let code = match try_native(&argv, Instant::now()) {
            Some(code) => code,
            None => {
                println!("{DELEGATED}");
                ExitCode::SUCCESS
            }
        };
        println!("{OUT_CLOSE}");
        eprintln!("{ERR_CLOSE}");
        code
    }

    struct Run {
        /// The command exited non-zero (ctx.fail), as Node's `status !== 0`.
        refused: bool,
        stdout: String,
        stderr: String,
    }

    impl Run {
        fn json(&self) -> Value {
            serde_json::from_str(&self.stdout)
                .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {:?}", self.stdout))
        }
    }

    fn between(hay: &str, open: &str, close: &str, whole: &str) -> String {
        let start = match hay.find(open) {
            Some(i) => i + open.len(),
            None => panic!("child never printed {open} — full child output:\n{whole}"),
        };
        let end = match hay[start..].find(close) {
            Some(i) => start + i,
            None => panic!("child never printed {close} — full child output:\n{whole}"),
        };
        hay[start..end].trim().to_string()
    }

    fn run_sweep(root: &Path, args: &[&str]) -> Run {
        let exe = std::env::current_exe().expect("test binary path");
        let out = std::process::Command::new(exe)
            .args([CHILD_TEST, "--exact", "--ignored", "--nocapture"])
            .env(ARGV_ENV, args.join("\u{1f}"))
            .current_dir(root)
            .output()
            .expect("spawn the child test binary");
        let raw_out = String::from_utf8_lossy(&out.stdout).into_owned();
        let raw_err = String::from_utf8_lossy(&out.stderr).into_owned();
        let whole = format!("--- stdout ---\n{raw_out}\n--- stderr ---\n{raw_err}");
        let stdout = between(&raw_out, OUT_OPEN, OUT_CLOSE, &whole);
        let stderr = between(&raw_err, ERR_OPEN, ERR_CLOSE, &whole);
        assert!(
            !stdout.contains(DELEGATED),
            "`bee {}` fell through to the Node delegate — the native path under test never ran",
            args.join(" ")
        );
        Run { refused: !out.status.success(), stdout, stderr }
    }

    #[test]
    fn default_sweep_protects_live_and_gates_absent_on_before() {
        let repo = setup_repo();
        let root = repo.path();
        write_state(root, r#"{"phase":"executing","feature":"live-f"}"#);
        write_lane(root, "closed-f", r#"{"feature":"closed-f","phase":"idle"}"#);
        for name in ["live-f", "closed-f", "absent-f"] {
            std::fs::create_dir_all(root.join(".bee").join("tmp").join(name)).unwrap();
            std::fs::write(root.join(".bee").join("tmp").join(name).join("x.txt"), "data").unwrap();
        }

        // No --before: closed sweeps, live and absent skip (with reasons).
        let plan = compute_sweep_plan(root, None, None, false).ok().unwrap();
        let included: Vec<&str> = plan.included.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(included, vec!["closed-f"]);
        let mut skipped: Vec<(String, Option<&'static str>)> =
            plan.skipped.iter().map(|(_, n, r)| (n.clone(), *r)).collect();
        skipped.sort();
        assert_eq!(
            skipped,
            vec![("absent-f".to_string(), Some("absent_no_before")), ("live-f".to_string(), Some("live"))]
        );
        assert_eq!(plan.included[0].bytes, 4);
        assert_eq!(plan.included[0].files, 1);

        // With a far-future --before, absent qualifies too.
        let plan = compute_sweep_plan(root, None, Some("2999-01-01"), false).ok().unwrap();
        let mut included: Vec<&str> = plan.included.iter().map(|i| i.name.as_str()).collect();
        included.sort();
        assert_eq!(included, vec!["absent-f", "closed-f"]);

        // --all takes everything, live or not.
        let plan = compute_sweep_plan(root, None, None, true).ok().unwrap();
        assert_eq!(plan.included.len(), 3);
        assert!(plan.skipped.is_empty());
    }

    #[test]
    fn feature_match_takes_cells_but_never_live_siblings() {
        let repo = setup_repo();
        let root = repo.path();
        write_state(root, r#"{"phase":"idle","feature":null}"#);
        write_lane(root, "auth-v2", r#"{"feature":"auth-v2","phase":"executing"}"#);
        for name in ["auth", "auth-1", "auth-v2", "auth2", "other"] {
            std::fs::create_dir_all(root.join(".bee").join("tmp").join(name)).unwrap();
        }
        let plan = compute_sweep_plan(root, Some("auth"), None, false).ok().unwrap();
        let mut included: Vec<&str> = plan.included.iter().map(|i| i.name.as_str()).collect();
        included.sort();
        // exact + prefix-with-separator; auth-v2 is a LIVE sibling (refused,
        // reported); auth2 has no separator; other is not a match.
        assert_eq!(included, vec!["auth", "auth-1"]);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].1, "auth-v2");
        assert_eq!(plan.skipped[0].2, Some("live_sibling"));
    }

    #[test]
    fn invalid_before_throws_and_exotic_before_delegates() {
        let repo = setup_repo();
        write_state(repo.path(), r#"{"phase":"idle"}"#);
        match compute_sweep_plan(repo.path(), None, Some("2999-99-99"), false) {
            Err(SweepErr::Thrown(msg)) => {
                assert_eq!(msg, "tmp sweep: --before \"2999-99-99\" is not a valid ISO date.")
            }
            _ => panic!("expected thrown"),
        }
        assert!(matches!(
            compute_sweep_plan(repo.path(), None, Some("next tuesday"), false),
            Err(SweepErr::Delegate)
        ));
    }

    #[cfg(windows)]
    #[test]
    fn symlinked_root_is_refused_wholesale() {
        // A junction .bee/spikes -> repo root must be excluded and reported.
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        std::fs::create_dir_all(root.join(".bee").join("tmp")).unwrap();
        let target = root.to_path_buf();
        let link = root.join(".bee").join("spikes");
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&target)
            .output();
        if !status.map(|o| o.status.success()).unwrap_or(false) {
            eprintln!(
                "SKIP (env-limited: junction creation denied — needs `mklink /J` rights, i.e. \
                 Developer Mode or an elevated shell) symlinked_root_is_refused_wholesale"
            );
            return;
        }
        let inspection = inspect_scratch_roots(root);
        assert_eq!(inspection.roots.len(), 1);
        assert_eq!(inspection.roots[0].rel, ".bee/tmp");
        assert_eq!(inspection.refused.len(), 1);
        assert_eq!(inspection.refused[0].reason, "symlinked_root");
    }

    #[test]
    fn corrupt_lane_record_warns_and_reads_as_absent() {
        let repo = setup_repo();
        let root = repo.path();
        // Mismatched feature field: deterministic warn, record skipped.
        write_lane(root, "f1", r#"{"feature":"other","phase":"executing"}"#);
        assert_eq!(read_lane_phase(root, "f1").unwrap(), None);
        // CUTOVER: corrupt JSON used to delegate (Node's readJson warning
        // carried a V8 parse message). It now warns — readJson's line AND
        // readLane's own — and reads as "no lane", readJson's null fallback.
        write_lane(root, "f2", "{broken");
        assert_eq!(read_lane_phase(root, "f2").unwrap(), None);
        // Path-separator names read as "no lane" without touching disk.
        assert_eq!(read_lane_phase(root, "a/b").unwrap(), None);
    }

    /// The sweep PLAN over a corrupt lane record: the run still succeeds and
    /// the lane simply reads as absent, so the feature is not "live".
    #[test]
    fn a_corrupt_lane_record_no_longer_delegates_the_sweep() {
        let repo = setup_repo();
        let root = repo.path();
        write_state(root, r#"{"phase":"idle","feature":null}"#);
        write_lane(root, "f2", "{broken");
        let state = read_state_lite(root).unwrap();
        assert!(!is_live_feature(root, &state, "f2").unwrap());
        assert!(!has_record(root, &state, "f2").unwrap());
    }

    // ─── whole-command contracts (oracle: packages/bee/tests/test_scratch.mjs) ─

    /// Oracle: "`bee tmp sweep` with NO flags refuses (typed), no default
    /// purge", "`--json` (json alone) still refuses", and the control
    /// "`--dry-run` (a real flag) does not refuse".
    #[test]
    fn sweep_refuses_every_targetless_invocation_and_purges_nothing() {
        let repo = setup_cli_repo();
        let root = repo.path();
        make_scratch(root, ".bee/tmp", "would-be-swept", &[("a.txt", "hello world")]);
        let victim = scratch_dir(root, ".bee/tmp").join("would-be-swept");

        // No flags at all: a typed refusal that names every way to opt in.
        let bare = run_sweep(root, &["tmp", "sweep"]);
        assert!(
            bare.refused,
            "a no-flag sweep must exit non-zero; stdout={:?} stderr={:?}",
            bare.stdout, bare.stderr
        );
        for flag in ["--feature", "--before", "--all", "--dry-run"] {
            assert!(bare.stderr.contains(flag), "the refusal must name {flag}: {:?}", bare.stderr);
        }
        assert!(bare.stderr.contains("no default purge"), "{:?}", bare.stderr);
        assert!(victim.is_dir(), "a refused no-flag call must delete nothing");

        // `--json` is not a target either — same refusal, carried on stdout.
        let json_only = run_sweep(root, &["tmp", "sweep", "--json"]);
        assert!(json_only.refused, "--json alone must still refuse: {:?}", json_only.stdout);
        let err = json_only.json();
        assert!(
            err["error"].as_str().unwrap_or_default().contains("no default purge"),
            "{err}"
        );
        assert!(victim.is_dir(), "a refused --json call must delete nothing either");

        // CONTROL — one real flag and the very same command is served, so the
        // refusals above are the flag gate and not a broken fixture. The
        // default target set is still empty (no record, no --before), which is
        // the same discipline expressed as a plan instead of an error.
        let dry = run_sweep(root, &["tmp", "sweep", "--dry-run", "--json"]);
        assert!(!dry.refused, "--dry-run must be accepted as a real flag: {:?}", dry.stderr);
        let payload = dry.json();
        assert_eq!(payload["dry_run"], json!(true));
        assert_eq!(payload["removed"], json!([]));
        assert!(victim.is_dir(), "--dry-run must delete nothing");
    }

    /// Oracle: "--dry-run removes nothing on disk and its removed[] matches a
    /// real run byte-for-byte" + "reported bytes_freed/files_freed match a
    /// manual walk of the real files removed".
    #[test]
    fn dry_run_reports_exactly_what_a_real_run_removes_and_deletes_nothing() {
        let repo = setup_cli_repo();
        let root = repo.path();
        let files = [
            ("a.txt", "hello world"),
            ("nested/b.txt", "second file, more bytes here"),
            ("nested/deeper/c.txt", "zzzzz"),
        ];
        make_scratch(root, ".bee/tmp", "closed-feat", &files);
        write_lane(root, "closed-feat", r#"{"feature":"closed-feat","phase":"compounding-complete"}"#);
        let target = scratch_dir(root, ".bee/tmp").join("closed-feat");
        // Ground truth: a manual walk of the fixture, not the port's own sizer.
        let expect_bytes: u64 = files.iter().map(|(_, c)| c.len() as u64).sum();
        let expect_files = files.len() as u64;

        let dry = run_sweep(root, &["tmp", "sweep", "--all", "--dry-run", "--json"]).json();
        assert!(target.is_dir(), "--dry-run must delete nothing");
        assert_eq!(dry["dry_run"], json!(true));
        assert_eq!(dry["bytes_freed"], json!(expect_bytes), "{dry}");
        assert_eq!(dry["files_freed"], json!(expect_files), "{dry}");
        assert_eq!(dry["removed"][0]["bytes"], json!(expect_bytes));
        assert_eq!(dry["removed"][0]["files"], json!(expect_files));

        let real = run_sweep(root, &["tmp", "sweep", "--all", "--json"]).json();
        assert!(!target.exists(), "the real run must actually remove the directory");
        assert_eq!(real["dry_run"], json!(false));
        assert_eq!(
            real["removed"], dry["removed"],
            "a real run's removed[] must be exactly what the dry run advertised"
        );
        assert_eq!(real["bytes_freed"], dry["bytes_freed"]);
        assert_eq!(real["files_freed"], dry["files_freed"]);
        assert_eq!(real["removed"][0]["name"], json!("closed-feat"));
        assert_eq!(real["removed"][0]["scratchRoot"], json!(".bee/tmp"));
    }

    /// Oracle: issue #53 — "a LOOSE FILE at the scratch root is a sweep entry
    /// — --all clears it", paired with "loose root files stay behind the same
    /// no-default-purge discipline as dirs".
    #[test]
    fn all_clears_loose_root_files_that_the_default_target_set_leaves_alone() {
        let repo = setup_cli_repo();
        let root = repo.path();
        let tmp_root = scratch_dir(root, ".bee/tmp");
        make_scratch(root, ".bee/tmp", "a-dir", &[("a.txt", "dir scratch")]);
        std::fs::write(tmp_root.join("build_helper.mjs"), "console.log(1)\n").unwrap();
        std::fs::write(tmp_root.join("f2-2-evidence.json"), "{\"x\":1}\n").unwrap();

        // CONTROL — record-less entries, loose files included, are reported and
        // left alone by the default set.
        let dry = run_sweep(root, &["tmp", "sweep", "--dry-run", "--json"]).json();
        assert_eq!(dry["removed"], json!([]));
        let mut skipped: Vec<(String, String)> = dry["skipped"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s["name"].as_str().unwrap().to_string(),
                    s["reason"].as_str().unwrap_or("null").to_string(),
                )
            })
            .collect();
        skipped.sort();
        assert_eq!(
            skipped,
            vec![
                ("a-dir".to_string(), "absent_no_before".to_string()),
                ("build_helper.mjs".to_string(), "absent_no_before".to_string()),
                ("f2-2-evidence.json".to_string(), "absent_no_before".to_string()),
            ]
        );
        assert!(tmp_root.join("build_helper.mjs").is_file(), "nothing swept without a flag");

        // --all clears the lot — "the lot" includes the loose root files.
        let all = run_sweep(root, &["tmp", "sweep", "--all", "--json"]).json();
        let mut names: Vec<&str> = all["removed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["name"].as_str().unwrap())
            .collect();
        names.sort();
        assert_eq!(names, vec!["a-dir", "build_helper.mjs", "f2-2-evidence.json"], "{all}");
        assert_eq!(all["files_freed"], json!(3), "files_freed must count the loose files");
        assert!(!tmp_root.join("build_helper.mjs").exists());
        assert!(!tmp_root.join("f2-2-evidence.json").exists());
        assert!(!tmp_root.join("a-dir").exists());
    }

    /// Oracle: "docs/**, .bee/cells/, .bee/decisions.jsonl are NEVER touched
    /// under any flag combination".
    #[test]
    fn deliverables_survive_every_flag_combination() {
        let repo = setup_cli_repo();
        let root = repo.path();
        write_lane(root, "closed-feat", r#"{"feature":"closed-feat","phase":"compounding-complete"}"#);

        let docs = root.join("docs").join("history").join("demo").join("CONTEXT.md");
        std::fs::create_dir_all(docs.parent().unwrap()).unwrap();
        std::fs::write(&docs, "# real deliverable").unwrap();
        let cell = root.join(".bee").join("cells").join("demo-1.json");
        std::fs::create_dir_all(cell.parent().unwrap()).unwrap();
        std::fs::write(&cell, "{\"id\":\"demo-1\"}").unwrap();
        let decisions = root.join(".bee").join("decisions.jsonl");
        std::fs::write(&decisions, "{\"id\":\"dec-1\"}\n").unwrap();
        let deliverables = [docs, cell, decisions];
        // The bytes ARE the contract here: an untouched file is byte-identical.
        let before: Vec<String> =
            deliverables.iter().map(|p| std::fs::read_to_string(p).unwrap()).collect();

        let combos: [&[&str]; 4] = [
            &["tmp", "sweep", "--all", "--dry-run", "--json"],
            &["tmp", "sweep", "--all", "--json"],
            &["tmp", "sweep", "--feature", "closed-feat", "--json"],
            &["tmp", "sweep", "--before", "2999-01-01", "--json"],
        ];
        let scratch = scratch_dir(root, ".bee/tmp").join("closed-feat");
        for args in combos {
            make_scratch(root, ".bee/tmp", "closed-feat", &[("a.txt", "reseeded scratch")]);
            let run = run_sweep(root, args).json();
            // PAIRED CONTROL — each combo really swept something, so the
            // survival assertions below cannot pass vacuously.
            let names: Vec<&str> = run["removed"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["name"].as_str().unwrap())
                .collect();
            assert_eq!(names, vec!["closed-feat"], "combo {args:?} swept nothing: {run}");
            let dry = run["dry_run"] == json!(true);
            assert_eq!(
                scratch.exists(),
                dry,
                "combo {args:?}: only --dry-run may leave the scratch dir behind"
            );
            for (path, want) in deliverables.iter().zip(&before) {
                assert_eq!(
                    &std::fs::read_to_string(path).unwrap(),
                    want,
                    "{} must survive combo {args:?}",
                    path.display()
                );
            }
        }
    }

    /// Oracle: "finding 4 (pinned): a CLOSED record sweeps even when --before
    /// predates its mtime — --before never age-gates closed scratch".
    #[test]
    fn closed_record_sweeps_even_when_before_predates_its_mtime() {
        let repo = setup_repo();
        let root = repo.path();
        write_state(root, r#"{"phase":"idle","feature":null}"#);
        write_lane(root, "closed-feat", r#"{"feature":"closed-feat","phase":"compounding-complete"}"#);
        for name in ["closed-feat", "absent-feat"] {
            make_scratch(root, ".bee/tmp", name, &[("a.txt", "scratch")]);
        }

        // A cutoff far in the past — every fixture entry's mtime is newer.
        let plan = compute_sweep_plan(root, None, Some("2000-01-01"), false).ok().unwrap();
        let included: Vec<&str> = plan.included.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(
            included,
            vec!["closed-feat"],
            "a closed record's closure is the signal; --before never age-gates it"
        );
        // CONTROL — the very same cutoff DOES gate the record-less sibling,
        // so the line above is about the record and not about --before being
        // ignored wholesale.
        let skipped: Vec<(&str, Option<&'static str>)> =
            plan.skipped.iter().map(|(_, n, r)| (n.as_str(), *r)).collect();
        assert_eq!(skipped, vec![("absent-feat", Some("absent_not_old_enough"))]);

        // …and a cutoff ahead of that mtime takes the absent one too.
        let plan = compute_sweep_plan(root, None, Some("2999-01-01"), false).ok().unwrap();
        let mut included: Vec<&str> = plan.included.iter().map(|i| i.name.as_str()).collect();
        included.sort();
        assert_eq!(included, vec!["absent-feat", "closed-feat"]);
    }
}
