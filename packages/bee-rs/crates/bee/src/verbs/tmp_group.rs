// bee tmp — native port of the tmp verb group (bee.mjs handleTmpSweep +
// lib/scratch.mjs runSweep/computeSweepPlan and friends).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   tmp sweep [--feature F] [--before ISO] [--all] [--dry-run] [--json]
// Nothing in this group is left permanently delegated. Within the accepted
// shapes the deterministic refusals are served natively byte-identical: the
// no-flag "no default purge" refusal and the invalid --before ISO refusal.
//
// Additional delegation triggers (None before any output/write):
//   - --help anywhere, unknown flags, non-flag tokens, --all=<not true/false>
//   - corrupt .bee/state.json or .bee/lanes/*.json (Node warns with the V8
//     parse message)
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
use crate::verbs::reservations::{js_date_parse, js_strict_eq, js_trim, keys_known, now_ms, parse_flags, FlagV};
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

/// readState(root) — fail-open defaults; Err(()) => corrupt file, delegate.
fn read_state_lite(root: &Path) -> Result<StateLite, ()> {
    let brief = read_state_brief(root).map_err(|_| ())?;
    Ok(StateLite { phase: brief.phase, feature: brief.feature })
}

/// readLane(root, feature) — fail-open display read; Ok(Some(phase)) when a
/// matching lane record exists. Err(()) => corrupt JSON, delegate. The
/// mismatched-record console.warn is replicated byte-identically.
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
        ReadJson::Corrupt => return Err(()), // Node's readJson warns with the V8 message
        ReadJson::Parsed(v) => v,
    };
    // laneRecordFrom: object, feature field must strictly equal the trimmed name.
    let record_ok = matches!(&parsed, Value::Object(_))
        && js_strict_eq(
            parsed.get("feature").unwrap_or(&Value::Null),
            &Value::String(trimmed.to_string()),
        );
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
    if js_strict_eq(&state.feature, &Value::String(name.to_string())) && !is_terminal_phase(&state.phase) {
        return Ok(true);
    }
    match read_lane_phase(root, name)? {
        Some(phase) => Ok(!is_terminal_phase(&phase)),
        None => Ok(false),
    }
}

fn has_record(root: &Path, state: &StateLite, name: &str) -> Result<bool, ()> {
    if js_strict_eq(&state.feature, &Value::String(name.to_string())) {
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
/// sibling is itself live. Err(()) => delegate (corrupt lane record).
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
            return; // environment cannot create junctions — skip
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
        // Corrupt JSON: delegate.
        write_lane(root, "f2", "{broken");
        assert!(read_lane_phase(root, "f2").is_err());
        // Path-separator names read as "no lane" without touching disk.
        assert_eq!(read_lane_phase(root, "a/b").unwrap(), None);
    }
}
