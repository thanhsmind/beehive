// verbs/triggers — the deferred-decision trigger registry (D2,
// docs/history/knowledge-distill-trigger/plan.md's C2).
//
// A deferred decision ("revisit when upstream lands", "for now, until…")
// used to sink into decisions.jsonl prose with nothing watching it. This
// module gives it a persistent record instead: one JSON file per trigger
// at the CONTROL root (shared across worktrees — a worktree session must
// hit the shared store, never a per-worktree copy, exactly the
// no_route_claim_counts_dir pattern, cells/handlers_write.rs:815-870):
//
//   .bee/triggers/<slug>__<short8>.json
//
// Record shape: `id` (the filename stem), `decision` (short8 of the
// deferring decision), `condition` (prose), `tier` (`predicate` or
// `manual`, derived at `add` time — present `--predicate` makes it
// `predicate`, absent makes it `manual`), `predicate` (optional
// `path-exists:<p>` | `path-missing:<p>` | `path-changed:<p>[,<p>...]`),
// `anchored_at` (path-changed only: the HEAD sha of the tree where `add`
// ran), `status` (`waiting` | `due` | `resolved`),
// `created_at`/`updated_at`, `outcome` (set only by `resolve`).
//
// `path-changed` (finding-recheck-trigger D1) goes true once a commit in
// `<anchored_at>..HEAD` at the CONTROL root touches one of its
// repo-relative paths. The anchor is the HEAD of the tree where `add` ran
// (a feature worktree, usually), so the feature's own commits never fire
// it when that branch merges. A git error or a missing anchor reads as
// true: a watched condition never sinks silently.
//
// Verbs:
//   triggers add     --decision <id> --condition <text> [--predicate <spec>] [--json]
//   triggers list     [--due] [--json]
//   triggers resolve  --id <id> --outcome <text> [--json]
//
// Evaluation happens ON READ (`list`, the `due_and_manual_counts` door
// `bee orient` calls, and the per-prompt `due_count_for_prompt`, which
// re-evaluates only when the control root HEAD moved since its last run —
// cached in `.bee/cache/triggers-last-eval-head`, outside the tracked
// store): a `predicate`-tier trigger still `waiting`
// has its predicate checked, and a true predicate flips it to `due` AND
// PERSISTS that flip — the same write-on-read shape `bee orient`'s own
// `sweep_on_orient` already uses (status_full/orient.rs:260-289).
// `manual`-tier triggers NEVER auto-fire; they only ever surface as
// awaiting confirmation. `resolve` writes `outcome` + `status: resolved`
// into the trigger record ONLY — it never logs a decision itself; the
// capture discipline owns any follow-up decision that outcome implies.
//
// A corrupt or shape-invalid trigger file fails OPEN to a visible
// "unreadable trigger <file> — remedy: delete the file" line, never a
// crash and never a silent skip (fsutil.rs's own Missing/Corrupt split).
//
// Root topology: WORKTREE-NATIVE (`resolve_store_root_worktree`, the same
// door `verbs/reservations` uses) — a worker in a feature worktree must
// be able to run `triggers add`/`list`/`resolve` directly. The verb's OWN
// root (`Ctx.root`, used for timings/drift/emit) stays the worktree's;
// only the trigger STORE itself re-roots onto the control root via
// `rsv::control_root_for`, reservations.mjs's cycle-free git walk
// (`verbs/reservations/leases.rs`), the same one `deferred_queue.rs`'s
// `control_root_string` already reuses for the identical reason.

use super::feedback::{emit_error, emit_success, js_trim, now_iso, parse_shape, ParsedArgs};
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::textutil::truncate_chars_head;
use crate::verbs::reservations as rsv;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── store paths ────────────────────────────────────────────────────────

fn triggers_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("triggers")
}

fn control_root_path(root: &Path) -> PathBuf {
    let root_s = root.to_string_lossy().into_owned();
    PathBuf::from(rsv::control_root_for(&root_s).unwrap_or(root_s))
}

// ─── record ─────────────────────────────────────────────────────────────

pub(crate) struct TriggerRecord {
    pub(crate) id: String,
    /// The SHORT8 of the deferring decision — the first 8 characters of
    /// its id, never the full id (written at `add`, below). Every join
    /// from a decision onto its triggers goes through this one field.
    pub(crate) decision: String,
    condition: String,
    pub(crate) tier: String,
    predicate: Option<String>,
    /// `path-changed` only: the HEAD sha of the tree where `add` ran.
    anchored_at: Option<String>,
    pub(crate) status: String,
    created_at: String,
    updated_at: String,
    outcome: Option<String>,
}

const KNOWN_TIERS: [&str; 2] = ["predicate", "manual"];
const KNOWN_STATUSES: [&str; 3] = ["waiting", "due", "resolved"];

impl TriggerRecord {
    /// `None` for anything JSON-shaped but missing a required field or
    /// carrying an unknown tier/status — treated exactly like a parse
    /// failure by every caller (fail open to the unreadable-trigger line).
    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        let id = m.get("id")?.as_str()?.to_string();
        let decision = m.get("decision")?.as_str()?.to_string();
        let condition = m.get("condition")?.as_str()?.to_string();
        let tier = m.get("tier")?.as_str()?.to_string();
        if !KNOWN_TIERS.contains(&tier.as_str()) {
            return None;
        }
        let status = m.get("status")?.as_str()?.to_string();
        if !KNOWN_STATUSES.contains(&status.as_str()) {
            return None;
        }
        let created_at = m.get("created_at").and_then(Value::as_str).unwrap_or("").to_string();
        let updated_at = m.get("updated_at").and_then(Value::as_str).unwrap_or("").to_string();
        let predicate = m.get("predicate").and_then(Value::as_str).map(str::to_string);
        let anchored_at = m.get("anchored_at").and_then(Value::as_str).map(str::to_string);
        let outcome = m.get("outcome").and_then(Value::as_str).map(str::to_string);
        Some(Self { id, decision, condition, tier, predicate, anchored_at, status, created_at, updated_at, outcome })
    }

    fn to_value(&self) -> Value {
        let mut v = json!({
            "id": self.id,
            "decision": self.decision,
            "condition": self.condition,
            "tier": self.tier,
            "predicate": self.predicate,
            "status": self.status,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "outcome": self.outcome,
        });
        if let Some(sha) = &self.anchored_at {
            v["anchored_at"] = json!(sha);
        }
        v
    }
}

// ─── predicate evaluation ───────────────────────────────────────────────

fn is_valid_predicate(spec: &str) -> bool {
    if let Some(list) = spec.strip_prefix("path-changed:") {
        return list.split(',').map(js_trim).all(|p| {
            !p.is_empty() && !Path::new(p).is_absolute() && !p.starts_with('/') && !p.starts_with('\\')
        });
    }
    let after = spec.strip_prefix("path-exists:").or_else(|| spec.strip_prefix("path-missing:"));
    matches!(after, Some(p) if !js_trim(p).is_empty())
}

/// The trimmed HEAD sha of the checkout at `root`, `None` when git cannot
/// name one (no repo, no commit yet).
fn head_sha(root: &Path) -> Option<String> {
    let out = crate::verbs::worktree::run_git(root, &["rev-parse", "HEAD"]);
    let sha = js_trim(out.stdout.as_deref().unwrap_or("")).to_string();
    (out.status == Some(0) && !sha.is_empty()).then_some(sha)
}

/// True when a commit in `<anchor>..HEAD` at `control` touches one of the
/// comma-separated `paths`. A missing anchor or any git failure is ALSO
/// true — a watched condition never reads as waiting because git broke.
fn paths_changed_since(control: &Path, anchor: Option<&str>, paths: &str) -> bool {
    let Some(anchor) = anchor.filter(|a| !a.is_empty()) else { return true };
    let range = format!("{anchor}..HEAD");
    let mut args = vec!["log", "-1", "--format=%H", range.as_str(), "--"];
    args.extend(paths.split(',').map(js_trim).filter(|p| !p.is_empty()));
    let out = crate::verbs::worktree::run_git(control, &args);
    out.status != Some(0) || !js_trim(out.stdout.as_deref().unwrap_or("")).is_empty()
}

/// A relative predicate path resolves against the CONTROL root (where the
/// trigger store itself lives) — the same root a `path-exists`/
/// `path-missing` condition almost always means ("has this file landed in
/// the repo yet"). An absolute path is used as-is. An unrecognized spec
/// (should not happen past `add`'s own validation, but a hand-edited
/// store file could carry one) never fires — fail closed on the
/// PREDICATE, fail open on the READ.
fn predicate_true(control: &Path, spec: &str, anchored_at: Option<&str>) -> bool {
    let resolve = |raw: &str| -> PathBuf {
        let p = Path::new(raw);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            control.join(p)
        }
    };
    if let Some(p) = spec.strip_prefix("path-exists:") {
        resolve(p).exists()
    } else if let Some(p) = spec.strip_prefix("path-missing:") {
        !resolve(p).exists()
    } else if let Some(paths) = spec.strip_prefix("path-changed:") {
        paths_changed_since(control, anchored_at, paths)
    } else {
        false
    }
}

// ─── slug + filename ────────────────────────────────────────────────────

/// Kebab-slug of `s`, capped at 40 chars: lowercase, runs of
/// non-alphanumeric characters collapse to one `-`, no leading/trailing
/// dash. `"trigger"` when `s` carries no letters or digits at all — a
/// filename stem is never empty.
fn slug_from_text(s: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in s.chars() {
        if out.chars().count() >= 40 {
            break;
        }
        if c.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.extend(c.to_lowercase());
        } else {
            pending_dash = true;
        }
    }
    if out.is_empty() {
        "trigger".to_string()
    } else {
        out
    }
}

/// `<slug>__<short8>.json`, disambiguated with a `-<n>` suffix on the rare
/// collision (two triggers off the same decision with near-identical
/// condition text) so a second `add` never clobbers an existing,
/// different trigger record.
fn trigger_path_new(dir: &Path, slug: &str, short8: &str) -> PathBuf {
    let base = format!("{slug}__{short8}");
    let mut candidate = dir.join(format!("{base}.json"));
    let mut n = 2u32;
    while candidate.exists() {
        candidate = dir.join(format!("{base}-{n}.json"));
        n += 1;
    }
    candidate
}

// ─── read + evaluate-on-read ────────────────────────────────────────────

pub(crate) enum TriggerEntry {
    Ok(TriggerRecord),
    Unreadable(PathBuf),
}

fn unreadable_line(path: &Path) -> String {
    format!("unreadable trigger {} — remedy: delete the file", path.display())
}

/// The ONE walk of the trigger store, shared by both readers. `evaluate`
/// is the only difference between them: ON, a `predicate`-tier trigger
/// still `waiting` whose predicate is true is flipped to `due` and that
/// flip is PERSISTED (the write-on-read `triggers list` has always
/// performed); OFF, nothing under `.bee/triggers/` is written at all.
///
/// One body, never two copies — a rule checked at two points needs one
/// shared read, or the two points drift and only one of them is right.
///
/// Fail-open throughout, in BOTH modes: a missing directory is simply
/// empty, a file that raced away between listing and reading is skipped,
/// and a corrupt or shape-invalid file becomes `Unreadable` rather than a
/// crash — callers surface that as the delete-remedy line.
fn read_entries(control: &Path, evaluate: bool) -> Vec<TriggerEntry> {
    let dir = triggers_dir(control);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.ends_with(".json").then_some(name)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let path = dir.join(&name);
        match read_json(&path) {
            ReadJson::Missing => continue,
            ReadJson::Corrupt => out.push(TriggerEntry::Unreadable(path)),
            ReadJson::Parsed(v) => match TriggerRecord::from_value(&v) {
                None => out.push(TriggerEntry::Unreadable(path)),
                Some(mut rec) => {
                    if evaluate && rec.tier == "predicate" && rec.status == "waiting" {
                        if let Some(spec) = rec.predicate.clone() {
                            if predicate_true(control, &spec, rec.anchored_at.as_deref()) {
                                rec.status = "due".to_string();
                                rec.updated_at = now_iso();
                                let _ = write_json_atomic(&path, &rec.to_value());
                            }
                        }
                    }
                    out.push(TriggerEntry::Ok(rec));
                }
            },
        }
    }
    out
}

/// The evaluating reader — `triggers list` and `bee orient`'s counts. The
/// predicate flip is ON, so this call can WRITE the store it reads.
fn read_and_evaluate(control: &Path) -> Vec<TriggerEntry> {
    read_entries(control, true)
}

/// The read-only reader: the same walk with the predicate flip OFF, so it
/// promises ZERO mutation — every file under `.bee/triggers/` is
/// byte-identical after the call, including a `predicate`-tier trigger
/// still `waiting` whose predicate is true. A refusal path that writes is
/// not a refusal path, so the derived contract-status read
/// (`verbs/decisions/read.rs`, slp-contract D1/D2) reaches the trigger
/// store through here and NEVER through `read_and_evaluate`.
///
/// `root` is the caller's OWN root (a decisions-store root, possibly a
/// worktree); this re-roots onto the control root itself, exactly the way
/// `trigger_registered` and every other trigger-store access in this
/// module does.
pub(crate) fn read_without_evaluating(root: &Path) -> Vec<TriggerEntry> {
    read_entries(&control_root_path(root), false)
}

/// `bee orient`'s door (status_full/orient.rs): `(due, awaiting
/// confirmation)` — `due` counts `predicate`-tier triggers whose flip has
/// landed, `awaiting confirmation` counts `manual`-tier triggers still
/// `waiting` (a manual trigger never reaches `due`; it only ever waits
/// for a human, then gets `resolve`d). Read-only from the caller's
/// perspective; the predicate flip this performs is the same
/// write-on-read `triggers list` performs.
/// D2's write-path law (`decisions log`'s `--trigger` flag,
/// knowledge-distill-trigger's kdt-3): true when `id` names a shape-valid
/// trigger record file under `control`'s `.bee/triggers/` — the same
/// fail-open shape check `read_and_evaluate` applies, so a corrupt or
/// malformed file never counts as "registered" (it only ever surfaces
/// separately, via `triggers list`'s unreadable line). `root` is the
/// caller's OWN root (a decisions-store root, possibly a worktree); this
/// re-roots onto the control root itself, the same way every other
/// trigger-store access in this module does.
pub(crate) fn trigger_registered(root: &Path, id: &str) -> bool {
    if !is_plain_id(id) {
        return false;
    }
    let control = control_root_path(root);
    let path = triggers_dir(&control).join(format!("{id}.json"));
    match read_json(&path) {
        ReadJson::Parsed(v) => TriggerRecord::from_value(&v).is_some(),
        _ => false,
    }
}

/// The per-prompt door (finding-recheck-trigger D2): the count of
/// `predicate`-tier triggers at `due`. Evaluation (and so any `git log`)
/// runs only when the control root HEAD differs from the sha cached in
/// `.bee/cache/triggers-last-eval-head`; an unchanged HEAD counts the stored
/// statuses as they are. An unreadable HEAD always evaluates and caches
/// nothing. No trigger store at all is zero, with nothing written.
pub(crate) fn due_count_for_prompt(control: &Path) -> usize {
    let dir = triggers_dir(control);
    if !dir.is_dir() {
        return 0;
    }
    let cache = eval_head_cache(control);
    let head = head_sha(control);
    let cached = std::fs::read_to_string(&cache).ok();
    let entries = match &head {
        Some(sha) if cached.as_deref().map(js_trim) == Some(sha.as_str()) => read_entries(control, false),
        _ => {
            let entries = read_and_evaluate(control);
            if let Some(sha) = &head {
                if let Some(parent) = cache.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&cache, sha);
            }
            entries
        }
    };
    entries
        .iter()
        .filter(|e| matches!(e, TriggerEntry::Ok(r) if r.tier == "predicate" && r.status == "due"))
        .count()
}

/// The prompt door's HEAD cache. It lives in `.bee/cache/`, which every
/// onboarded repo git-ignores, never in the tracked `.bee/triggers/` store:
/// a HEAD sha that changes on every commit must not show as a repo change.
fn eval_head_cache(control: &Path) -> PathBuf {
    control.join(".bee").join("cache").join("triggers-last-eval-head")
}

pub(crate) fn due_and_manual_counts(control: &Path) -> (usize, usize) {
    let mut due = 0usize;
    let mut manual_waiting = 0usize;
    for entry in read_and_evaluate(control) {
        if let TriggerEntry::Ok(rec) = entry {
            if rec.tier == "predicate" && rec.status == "due" {
                due += 1;
            } else if rec.tier == "manual" && rec.status == "waiting" {
                manual_waiting += 1;
            }
        }
    }
    (due, manual_waiting)
}

// ─── argv plumbing ──────────────────────────────────────────────────────

struct Ctx {
    root: PathBuf,
    control: PathBuf,
    drift: crate::registry::Drift,
}

fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r.root,
        RootsWt::Unsupported(why) => return Err(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why)),
        RootsWt::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let control = control_root_path(&root);
    let drift = check_manifest_drift(&root);
    Ok(Some(Ctx { root, control, drift }))
}

fn flag<'a>(parsed: &'a ParsedArgs, name: &str) -> Option<&'a str> {
    parsed.flags.get(name).map(|s| js_trim(s)).filter(|s| !s.is_empty())
}

struct ListFlags {
    due: bool,
    json: bool,
}

/// `triggers list`'s own tiny parser: `parse_shape` (feedback.rs) only
/// ever produces a bare boolean for `--json`, and `--due` needs the same
/// shape. Any other token refuses the whole parse (`None`), which — same
/// as every `parse_shape(...)?` call site in this crate — falls through
/// `try_native` to the top-level "unsupported command shape" emission.
fn parse_list_flags(rest: &[OsString]) -> Option<ListFlags> {
    let mut due = false;
    let mut json = false;
    for tok in rest {
        match tok.to_str()? {
            "--due" => due = true,
            "--json" => json = true,
            _ => return None,
        }
    }
    Some(ListFlags { due, json })
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "triggers" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "add" => run_add(parse_shape(rest, &["decision", "condition", "predicate"])?, t0),
        "list" => run_list(rest, t0),
        "resolve" => run_resolve(parse_shape(rest, &["id", "outcome"])?, t0),
        _ => None,
    }
}

// ─── add ─────────────────────────────────────────────────────────────────

/// Validate, anchor, and write one trigger record; `Err` is the refusal
/// line and means nothing was written. A `path-changed` predicate anchors
/// on the HEAD of `root` — the tree where `add` runs, NOT the control
/// root — so the reviewed branch's own commits never fire it after merge.
fn add_record(
    root: &Path,
    control: &Path,
    decision: &str,
    condition: &str,
    predicate: Option<&str>,
) -> Result<TriggerRecord, String> {
    let cmd = "triggers add";
    if let Some(p) = predicate {
        if !is_valid_predicate(p) {
            return Err(format!(
                "bee {cmd}: --predicate must be path-exists:<path>, path-missing:<path>, or \
                 path-changed:<path>[,<path>...] with repo-relative paths (a path containing a \
                 comma cannot be watched), got {p:?}."
            ));
        }
    }
    let anchored_at = match predicate {
        Some(p) if p.starts_with("path-changed:") => match head_sha(root) {
            Some(sha) => Some(sha),
            None => {
                return Err(format!(
                    "bee {cmd}: --predicate path-changed needs a git HEAD at {} to anchor on.",
                    root.display()
                ))
            }
        },
        _ => None,
    };
    let tier = if predicate.is_some() { "predicate" } else { "manual" };
    let short8 = truncate_chars_head(decision, 8);
    let dir = triggers_dir(control);
    let path = trigger_path_new(&dir, &slug_from_text(condition), &short8);
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "trigger".to_string());
    let ts = now_iso();
    let rec = TriggerRecord {
        id,
        decision: short8,
        condition: condition.to_string(),
        tier: tier.to_string(),
        predicate: predicate.map(str::to_string),
        anchored_at,
        status: "waiting".to_string(),
        created_at: ts.clone(),
        updated_at: ts,
        outcome: None,
    };
    if write_json_atomic(&path, &rec.to_value()).is_err() {
        return Err(format!("bee {cmd}: could not write trigger record."));
    }
    // The next prompt must evaluate the new trigger even if HEAD is still.
    let _ = std::fs::remove_file(eval_head_cache(control));
    Ok(rec)
}

fn run_add(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "triggers add";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(decision) = flag(&parsed, "decision") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --decision is required."), t0));
    };
    let Some(condition) = flag(&parsed, "condition") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --condition is required."), t0));
    };
    let rec = match add_record(&ctx.root, &ctx.control, decision, condition, flag(&parsed, "predicate")) {
        Ok(rec) => rec,
        Err(msg) => return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0)),
    };
    let text = format!("Registered {} trigger {} for decision {}.", rec.tier, rec.id, rec.decision);
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &rec.to_value(), &text, t0))
}

// ─── list ────────────────────────────────────────────────────────────────

fn run_list(rest: &[OsString], t0: Instant) -> Option<ExitCode> {
    let cmd = "triggers list";
    let ListFlags { due, json } = parse_list_flags(rest)?;
    let ctx = match preamble(cmd, json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let entries = read_and_evaluate(&ctx.control);
    let mut lines: Vec<String> = Vec::new();
    let mut trigger_values: Vec<Value> = Vec::new();
    let mut unreadable_values: Vec<Value> = Vec::new();
    for entry in entries {
        match entry {
            TriggerEntry::Unreadable(path) => {
                lines.push(unreadable_line(&path));
                unreadable_values.push(json!(path.display().to_string()));
            }
            TriggerEntry::Ok(rec) => {
                let surfaced = rec.status == "due" || (rec.tier == "manual" && rec.status == "waiting");
                if due && !surfaced {
                    continue;
                }
                lines.push(format!("- {} [{}/{}] {}", rec.id, rec.tier, rec.status, rec.condition));
                trigger_values.push(rec.to_value());
            }
        }
    }
    let text = if lines.is_empty() { "No triggers.".to_string() } else { lines.join("\n") };
    let result = json!({ "triggers": trigger_values, "unreadable": unreadable_values });
    Some(emit_success(&ctx.root, cmd, json, &ctx.drift, &result, &text, t0))
}

// ─── resolve ─────────────────────────────────────────────────────────────

fn is_plain_id(s: &str) -> bool {
    !s.is_empty() && !s.contains('/') && !s.contains('\\') && !s.contains("..")
}

fn run_resolve(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "triggers resolve";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let Some(id) = flag(&parsed, "id") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --id is required."), t0));
    };
    let Some(outcome) = flag(&parsed, "outcome") else {
        return Some(emit_error(&ctx.root, cmd, parsed.json, &format!("bee {cmd}: --outcome is required."), t0));
    };
    if !is_plain_id(id) {
        let msg = format!("bee {cmd}: trigger id must be a plain id (no path separators).");
        return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
    }
    let path = triggers_dir(&ctx.control).join(format!("{id}.json"));
    match read_json(&path) {
        ReadJson::Missing => {
            let msg = format!("bee {cmd}: no trigger with id \"{id}\".");
            Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0))
        }
        ReadJson::Corrupt => Some(emit_error(&ctx.root, cmd, parsed.json, &unreadable_line(&path), t0)),
        ReadJson::Parsed(v) => {
            let Some(mut rec) = TriggerRecord::from_value(&v) else {
                return Some(emit_error(&ctx.root, cmd, parsed.json, &unreadable_line(&path), t0));
            };
            rec.status = "resolved".to_string();
            rec.outcome = Some(outcome.to_string());
            rec.updated_at = now_iso();
            if write_json_atomic(&path, &rec.to_value()).is_err() {
                let msg = format!("bee {cmd}: could not write trigger record.");
                return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0));
            }
            let text = format!("Resolved trigger {id}: {outcome}");
            Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &rec.to_value(), &text, t0))
        }
    }
}

// ─── tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn git(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixture");
        assert!(out.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr));
    }

    /// A real main checkout plus one real linked (granted-shaped, in the
    /// sense that its store must still re-root to main for a control-plane
    /// store) worktree — mirrors status_full/tests.rs's `worktree_fixture`,
    /// re-derived locally since that helper is private to its own module.
    fn worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf) {
        // On Windows the tempdir can come back in 8.3 short form
        // (RUNNER~1); `n()` only repairs paths that exist, so a not-yet-created
        // `.bee/triggers` keeps the raw spelling and a byte compare fails on
        // spelling alone. One canonical root keeps every derived path in the
        // long form git itself reports.
        let tmp = dunce::canonicalize(tmp).unwrap_or_else(|_| tmp.to_path_buf());
        let main = tmp.join("main");
        std::fs::create_dir_all(&main).unwrap();
        write(&main, ".bee/onboarding.json", "{}");
        write(&main, "f.txt", "x");
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let wt = tmp.join("wt");
        git(&main, &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "wt/one"]);
        write(&wt, ".bee/onboarding.json", "{}");
        (main, wt)
    }

    fn n(p: &Path) -> String {
        dunce::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()).to_string_lossy().into_owned()
    }

    #[test]
    fn store_path_resolves_to_control_root_from_a_linked_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = worktree_fixture(tmp.path());
        assert_eq!(n(&control_root_path(&main)), n(&main));
        assert_eq!(n(&control_root_path(&wt)), n(&main));
        assert_eq!(n(&triggers_dir(&control_root_path(&wt))), n(&main.join(".bee").join("triggers")));
    }

    #[test]
    fn add_list_resolve_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let dir = triggers_dir(control);
        let path = trigger_path_new(&dir, &slug_from_text("revisit when upstream lands"), "c2a7bd4f");
        let rec = TriggerRecord {
            id: path.file_stem().unwrap().to_str().unwrap().to_string(),
            decision: "c2a7bd4f".to_string(),
            condition: "revisit when upstream lands".to_string(),
            tier: "manual".to_string(),
            predicate: None,
            anchored_at: None,
            status: "waiting".to_string(),
            created_at: "2026-08-16T00:00:00.000Z".to_string(),
            updated_at: "2026-08-16T00:00:00.000Z".to_string(),
            outcome: None,
        };
        write_json_atomic(&path, &rec.to_value()).unwrap();

        // list: the manual trigger surfaces as awaiting confirmation.
        let entries = read_and_evaluate(control);
        assert_eq!(entries.len(), 1);
        let TriggerEntry::Ok(read_back) = &entries[0] else { panic!("expected Ok") };
        assert_eq!(read_back.id, rec.id);
        assert_eq!(read_back.status, "waiting");
        let (due, manual) = due_and_manual_counts(control);
        assert_eq!((due, manual), (0, 1));

        // resolve: writes outcome + status, never touches decisions.jsonl
        // (there is none in this fixture — a crash here would prove a
        // decision got logged where none should be).
        let ReadJson::Parsed(v) = read_json(&path) else { panic!("expected Parsed") };
        let mut resolved = TriggerRecord::from_value(&v).unwrap();
        resolved.status = "resolved".to_string();
        resolved.outcome = Some("upstream landed; proceeding".to_string());
        write_json_atomic(&path, &resolved.to_value()).unwrap();
        let ReadJson::Parsed(v2) = read_json(&path) else { panic!("expected Parsed") };
        let final_rec = TriggerRecord::from_value(&v2).unwrap();
        assert_eq!(final_rec.status, "resolved");
        assert_eq!(final_rec.outcome.as_deref(), Some("upstream landed; proceeding"));
        assert!(!tmp.path().join(".bee").join("decisions.jsonl").exists());
    }

    #[test]
    fn predicate_flip_persists_across_reads() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let watched = control.join("watched.txt");
        let dir = triggers_dir(control);
        let path = trigger_path_new(&dir, "watched-file-lands", "aaaaaaaa");
        let rec = TriggerRecord {
            id: path.file_stem().unwrap().to_str().unwrap().to_string(),
            decision: "aaaaaaaa".to_string(),
            condition: "watched.txt lands".to_string(),
            tier: "predicate".to_string(),
            predicate: Some("path-exists:watched.txt".to_string()),
            anchored_at: None,
            status: "waiting".to_string(),
            created_at: "2026-08-16T00:00:00.000Z".to_string(),
            updated_at: "2026-08-16T00:00:00.000Z".to_string(),
            outcome: None,
        };
        write_json_atomic(&path, &rec.to_value()).unwrap();

        // Not yet true: stays waiting, never due.
        let (due, _) = due_and_manual_counts(control);
        assert_eq!(due, 0);
        let ReadJson::Parsed(v) = read_json(&path) else { panic!("expected Parsed") };
        assert_eq!(TriggerRecord::from_value(&v).unwrap().status, "waiting");

        // The path lands: the very next read flips AND persists.
        std::fs::write(&watched, "x").unwrap();
        let (due, manual) = due_and_manual_counts(control);
        assert_eq!((due, manual), (1, 0));
        let ReadJson::Parsed(v) = read_json(&path) else { panic!("expected Parsed") };
        assert_eq!(TriggerRecord::from_value(&v).unwrap().status, "due");

        // Persisted: a fresh read (no re-evaluation needed) still sees due.
        let ReadJson::Parsed(v2) = read_json(&path) else { panic!("expected Parsed") };
        assert_eq!(TriggerRecord::from_value(&v2).unwrap().status, "due");
    }

    #[test]
    fn manual_tier_never_auto_fires_even_with_a_predicate_shaped_condition() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let dir = triggers_dir(control);
        let path = trigger_path_new(&dir, "manual-one", "bbbbbbbb");
        let rec = TriggerRecord {
            id: path.file_stem().unwrap().to_str().unwrap().to_string(),
            decision: "bbbbbbbb".to_string(),
            condition: "when the team decides".to_string(),
            tier: "manual".to_string(),
            predicate: None,
            anchored_at: None,
            status: "waiting".to_string(),
            created_at: "2026-08-16T00:00:00.000Z".to_string(),
            updated_at: "2026-08-16T00:00:00.000Z".to_string(),
            outcome: None,
        };
        write_json_atomic(&path, &rec.to_value()).unwrap();
        let (due, manual) = due_and_manual_counts(control);
        assert_eq!((due, manual), (0, 1));
        // A second read never flips it to due — manual stays manual.
        let (due, manual) = due_and_manual_counts(control);
        assert_eq!((due, manual), (0, 1));
    }

    #[test]
    fn corrupt_file_yields_the_delete_remedy_line_never_a_crash() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let dir = triggers_dir(control);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("broken__cccccccc.json"), "not json at all").unwrap();

        let entries = read_and_evaluate(control);
        assert_eq!(entries.len(), 1);
        let TriggerEntry::Unreadable(path) = &entries[0] else { panic!("expected Unreadable") };
        let line = unreadable_line(path);
        assert!(line.starts_with("unreadable trigger "));
        assert!(line.ends_with(" — remedy: delete the file"));

        // Never a crash, and the queue-summary door stays fail-open too.
        let (due, manual) = due_and_manual_counts(control);
        assert_eq!((due, manual), (0, 0));
    }

    #[test]
    fn a_shape_valid_json_file_missing_a_required_field_is_also_unreadable() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let dir = triggers_dir(control);
        std::fs::create_dir_all(&dir).unwrap();
        // Valid JSON, but no `tier` — still fails open to the remedy line
        // rather than panicking on an unwrap deep in a caller.
        std::fs::write(
            dir.join("shapeless__dddddddd.json"),
            r#"{"id":"shapeless__dddddddd","decision":"dddddddd","condition":"x","status":"waiting"}"#,
        )
        .unwrap();
        let entries = read_and_evaluate(control);
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0], TriggerEntry::Unreadable(_)));
    }

    #[test]
    fn predicate_validation_accepts_only_the_two_known_specs() {
        assert!(is_valid_predicate("path-exists:foo/bar.txt"));
        assert!(is_valid_predicate("path-missing:foo/bar.txt"));
        assert!(!is_valid_predicate("path-exists:"));
        assert!(!is_valid_predicate("something-else:foo"));
        assert!(!is_valid_predicate(""));
    }

    // ─── path-changed (finding-recheck-trigger D1/D2) ──────────────────

    /// A committed git repo at `root` (canonical form) with `a.txt` and
    /// `b.txt` tracked.
    fn change_repo(tmp: &Path) -> PathBuf {
        let root = dunce::canonicalize(tmp).unwrap_or_else(|_| tmp.to_path_buf());
        write(&root, "a.txt", "a");
        write(&root, "b.txt", "b");
        git(&root, &["init", "-q", "-b", "main", "."]);
        git(&root, &["config", "user.email", "a@b.c"]);
        git(&root, &["config", "user.name", "t"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-qm", "init"]);
        root
    }

    fn commit_edit(root: &Path, rel: &str, content: &str) {
        write(root, rel, content);
        git(root, &["add", rel]);
        git(root, &["commit", "-qm", &format!("edit {rel}")]);
    }

    fn stored_status(control: &Path, rec: &TriggerRecord) -> String {
        let ReadJson::Parsed(v) = read_json(&triggers_dir(control).join(format!("{}.json", rec.id))) else {
            panic!("expected Parsed")
        };
        TriggerRecord::from_value(&v).unwrap().status
    }

    #[test]
    fn predicate_validation_for_path_changed() {
        assert!(is_valid_predicate("path-changed:a.txt"));
        assert!(is_valid_predicate("path-changed:a.txt, src/b.rs"));
        assert!(!is_valid_predicate("path-changed:"));
        assert!(!is_valid_predicate("path-changed:a.txt,,b.txt"));
        assert!(!is_valid_predicate("path-changed:a.txt, "));
        assert!(!is_valid_predicate("path-changed:/etc/passwd"));
        assert!(!is_valid_predicate("path-changed:a.txt,/abs/b.txt"));
    }

    #[test]
    fn add_refuses_bad_path_changed_specs_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = change_repo(tmp.path());
        for bad in ["path-changed:", "path-changed:a.txt,,b.txt", "path-changed:/abs.txt"] {
            let err = add_record(&root, &root, "deadbeef00", "x", Some(bad)).err().expect("refused");
            assert!(err.contains("path-changed:<path>[,<path>...]"), "{err}");
            assert!(err.contains("path-exists:<path>") && err.contains("path-missing:<path>"), "{err}");
            assert!(err.contains("comma cannot be watched"), "{err}");
        }
        assert!(!triggers_dir(&root).exists());
    }

    #[test]
    fn add_anchors_path_changed_on_the_tree_head() {
        let tmp = tempfile::tempdir().unwrap();
        let root = change_repo(tmp.path());
        let rec = add_record(&root, &root, "deadbeef00", "a changes", Some("path-changed:a.txt")).unwrap();
        assert_eq!(rec.anchored_at, head_sha(&root));
        assert!(rec.anchored_at.is_some());
        let ReadJson::Parsed(v) = read_json(&triggers_dir(&root).join(format!("{}.json", rec.id))) else {
            panic!("expected Parsed")
        };
        assert_eq!(v["anchored_at"], json!(head_sha(&root).unwrap()));
        // path-exists records keep their old shape: no anchored_at key.
        let plain = add_record(&root, &root, "deadbeef00", "b lands", Some("path-exists:b.txt")).unwrap();
        assert!(plain.to_value().get("anchored_at").is_none());
    }

    #[test]
    fn add_outside_a_git_repo_refuses_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let err = add_record(root, root, "deadbeef00", "x", Some("path-changed:a.txt")).err().expect("refused");
        assert_eq!(
            err,
            format!("bee triggers add: --predicate path-changed needs a git HEAD at {} to anchor on.", root.display())
        );
        assert!(!triggers_dir(root).exists());
    }

    #[test]
    fn path_changed_waits_until_a_commit_touches_a_watched_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root = change_repo(tmp.path());
        let rec = add_record(&root, &root, "deadbeef00", "a changes", Some("path-changed:c.txt, a.txt")).unwrap();
        // No commit since the anchor.
        assert_eq!(due_and_manual_counts(&root), (0, 0));
        assert_eq!(stored_status(&root, &rec), "waiting");
        // A commit touching an unrelated file.
        commit_edit(&root, "b.txt", "b2");
        assert_eq!(due_and_manual_counts(&root), (0, 0));
        assert_eq!(stored_status(&root, &rec), "waiting");
        // A commit touching a watched path flips and persists.
        commit_edit(&root, "a.txt", "a2");
        assert_eq!(due_and_manual_counts(&root), (1, 0));
        assert_eq!(stored_status(&root, &rec), "due");
    }

    #[test]
    fn path_changed_with_a_bogus_or_missing_anchor_reads_as_due() {
        let tmp = tempfile::tempdir().unwrap();
        let root = change_repo(tmp.path());
        assert!(predicate_true(&root, "path-changed:a.txt", Some("0123456789abcdef0123456789abcdef01234567")));
        assert!(predicate_true(&root, "path-changed:a.txt", Some("not-a-sha")));
        assert!(predicate_true(&root, "path-changed:a.txt", None));
        let head = head_sha(&root).unwrap();
        assert!(!predicate_true(&root, "path-changed:a.txt", Some(&head)));
    }

    #[test]
    fn path_changed_added_in_a_feature_worktree_stays_waiting_after_its_merge() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = worktree_fixture(tmp.path());
        // The feature branch's own commits touch the watched path.
        commit_edit(&wt, "f.txt", "feature edit");
        let rec = add_record(&wt, &control_root_path(&wt), "deadbeef00", "f changes", Some("path-changed:f.txt"))
            .unwrap();
        assert_eq!(rec.anchored_at, head_sha(&wt));
        // An unrelated commit on main, then a real merge commit.
        commit_edit(&main, "other.txt", "o");
        git(&main, &["merge", "--no-ff", "-q", "-m", "merge wt/one", "wt/one"]);
        assert_eq!(due_and_manual_counts(&main), (0, 0));
        assert_eq!(stored_status(&main, &rec), "waiting");
        // A later change to the path on main does fire it.
        commit_edit(&main, "f.txt", "post-merge edit");
        assert_eq!(due_and_manual_counts(&main), (1, 0));
    }

    #[test]
    fn due_count_for_prompt_evaluates_only_when_head_moves() {
        let tmp = tempfile::tempdir().unwrap();
        let root = change_repo(tmp.path());
        // No store yet: zero, nothing written.
        assert_eq!(due_count_for_prompt(&root), 0);
        assert!(!triggers_dir(&root).exists());
        let rec = add_record(&root, &root, "deadbeef00", "c lands", Some("path-exists:c.txt")).unwrap();
        let cache = root.join(".bee").join("cache").join("triggers-last-eval-head");
        assert_eq!(due_count_for_prompt(&root), 0);
        assert_eq!(std::fs::read_to_string(&cache).unwrap(), head_sha(&root).unwrap());
        // The cache lives in the git-ignored cache dir, never in the tracked store.
        assert!(!triggers_dir(&root).join(".last-eval-head").exists());
        // The predicate turns true, but HEAD has not moved: no evaluation.
        write(&root, "c.txt", "c");
        assert_eq!(due_count_for_prompt(&root), 0);
        assert_eq!(stored_status(&root, &rec), "waiting");
        // HEAD moves: the next prompt evaluates and counts it.
        commit_edit(&root, "b.txt", "b2");
        assert_eq!(due_count_for_prompt(&root), 1);
        assert_eq!(stored_status(&root, &rec), "due");
        assert_eq!(std::fs::read_to_string(&cache).unwrap(), head_sha(&root).unwrap());
        // A new add drops the cache so the next prompt evaluates it.
        add_record(&root, &root, "deadbeef00", "d lands", Some("path-exists:d.txt")).unwrap();
        assert!(!cache.exists());
    }

    #[test]
    fn slug_from_text_folds_case_and_punctuation_and_never_empty() {
        assert_eq!(slug_from_text("Revisit When Upstream Lands!"), "revisit-when-upstream-lands");
        assert_eq!(slug_from_text("!!!"), "trigger");
        assert_eq!(slug_from_text(""), "trigger");
    }

    #[test]
    fn trigger_path_new_disambiguates_a_collision() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let first = trigger_path_new(dir, "slug", "eeeeeeee");
        std::fs::write(&first, "{}").unwrap();
        let second = trigger_path_new(dir, "slug", "eeeeeeee");
        assert_ne!(first, second);
        assert_eq!(second.file_name().unwrap().to_str().unwrap(), "slug__eeeeeeee-2.json");
    }
}
