// onboard — Rust port of packages/bee/scripts/onboard_bee.mjs (4332 lines):
// the vendoring/rendering engine that installs and updates bee in a target
// repo.
//
//   bee onboard [--repo-root <path>] [--apply] [--json] [--repo-hooks]
//               [--plugin-source] [--runtime claude|codex|both]
//               [--no-claude-md] [--claude-md] [--global-skills]
//               [--force-downgrade]
//
// The flag set is the .mjs's own parseArgs (l. 4022), verbatim.
//
// ── contracts ─────────────────────────────────────────────────────────────
// Every rendered artifact is byte-identical to Node's: the AGENTS.md block,
// .claude/settings.json (2-space JSON + trailing newline), .codex/hooks.json,
// the skill trees and their render sidecars, the statusline pair, and
// .bee/onboarding.json including the managed sha256 map (hashes of a file's
// UTF-8 STRING content — lib/fsutil.mjs hashFile — not of its raw bytes,
// which walkSkillTree uses instead). Plan mode emits the same action list and
// the same JSON payload, key order included.
//
// opencode-support D1/D3 (oc-13, S5): OpenCode is bee's third first-class
// runtime, at parity with claude/codex for onboarding specifically —
// `--apply` installs `.opencode/skills/` (a third `REPO_SKILL_TARGETS`
// entry, synced by the same runtime-agnostic writer as the other two roots)
// AND the guard plugin at `.opencode/plugins/bee-guard.ts` (vendored from
// this checkout's own installed copy — D3: the beehive repo is the first
// consumer). Both are copy-when-missing-or-drifted, unconditional on
// `--runtime` (that flag governs the claude/codex HOOK belt only; OpenCode's
// belt is a separate, third mechanism — see hook_manifests.rs's NAMED
// EXCLUSION comment).
//
// ── routing ────────────────────────────────────────────────────────────────
// `try_native` serves `onboard` only when it can locate the bee source
// checkout (see source::Engine::locate). Without one there is nothing
// authoritative to vendor FROM, so the probe returns None before any output
// and the unknown-command refusal reports it. `--help`/`-h` also returns None
// on purpose: the shared `bee --help` surface (verbs::help) owns every help
// shape in this CLI.
//
// ── DELIBERATE DIVERGENCES from Node (logged C2 exceptions) ───────────────
//
// (1) **entryIdentity inode precision — BUG FIX, filed win32 defect.**
//     onboard_bee.mjs builds a skill's physical identity as
//     `${st.dev}:${st.ino}` from fs.Stats, whose fields are JS Numbers
//     (IEEE-754 doubles). On win32 libuv fills st_ino from the 64-bit NTFS
//     file index, so any index above 2^53 loses its low bits and two
//     DIFFERENT skill directories can hash to one identity —
//     detectAliasCollisions then blocks both as a "case-insensitive alias"
//     that does not exist (and, symmetrically, a real alias can slip
//     through). util::entry_identity keeps the volume serial as u64 and the
//     file index as u128, so the comparison is exact. The value was only
//     ever a Map key, never emitted, so no output byte changes: this fixes
//     WHICH skills get blocked, not how a block is reported.
//
// (2) **encodeProjectDir drive colon — FIXED AT CUTOVER; never lived here.**
//     The second filed win32 defect (the encoded transcript-directory name
//     kept the drive colon, e.g. "D:-projects-…", an illegal NTFS directory
//     name, so recovery/transcript resolution was unreachable on win32) lived
//     in packages/bee/lib/perf.mjs `encodeProjectDir`, not in onboard_bee.mjs
//     — this engine never encodes a project dir. Its two Rust mirrors,
//     verbs/status_full.rs::encode_project_dir and
//     hooks/session_close.rs::encode_project_dir, now map ':' to '-' as well,
//     giving "D--projects-…" — what Claude Code itself writes. The fix was
//     held until cutover because it means diverging from Node.
//
// (3) **Node-runtime preflight is structurally unreachable.** main()'s
//     `nodeRuntimeStatus()` check (`missing_runtime`, exit 1) inspects
//     `process.versions.node` — the interpreter running the script. A native
//     binary has no such interpreter, so the branch cannot fire and is not
//     ported. Every other status value is reproduced.
//
// (4) **Unreadable-but-existing text reads as "".** readTextIfExists THROWS
//     on EISDIR/EACCES and main()'s catch turns that into `{"error":
//     "<V8 message>"}`. Reproducing V8 message text is a campaign non-goal
//     (rule 2: refusals whose bytes embed a V8 message delegate), and this
//     path cannot delegate mid-apply, so the port treats such a read as
//     empty. Same reasoning for classifyMigrationRecord's "local record
//     unreadable: <V8 message>" reason, which the port states without the
//     interpreter's wording.

mod agents;
mod apply;
pub(crate) mod hooks_wiring;
mod merge;
mod migration;
mod notices;
mod plan;
mod render;
mod skills;
mod source;
// opencode-support oc-14: pub(crate) so status_full::store's drift check can
// read AGENT_TIER_DEFAULTS_OPENCODE directly — the same ground truth
// agents.rs renders from, never a second hand-kept copy of the defaults.
pub(crate) mod templates;
mod util;

#[cfg(test)]
mod tests;

use crate::jsjson;
use apply::{ApplyOutcome, ApplyOk};
use plan::{compute_plan, core_changes_needed, Options};
use serde_json::{json, Map, Value};
use source::Engine;
use std::ffi::OsString;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

// ── CLI ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Args {
    repo_root: Option<String>,
    apply: bool,
    json: bool,
    repo_hooks: bool,
    claude_md: bool,
    global_skills: bool,
    plugin_source: bool,
    force_downgrade: bool,
    runtime: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            repo_root: None,
            apply: false,
            json: false,
            repo_hooks: false,
            // D1: CLAUDE.md is a default onboarding artifact; --no-claude-md
            // opts out. --claude-md is still accepted, a no-op alias.
            claude_md: true,
            global_skills: false,
            plugin_source: false,
            force_downgrade: false,
            // GH #22 P0-1: default "both" matches install.sh's own default.
            runtime: "both".to_string(),
        }
    }
}

enum ParseOutcome {
    Parsed(Box<Args>),
    Error(String),
    /// `--help`/`-h`: hand the command back to the shared help surface.
    Delegate,
}

/// parseArgs (l. 4022).
fn parse_args(argv: &[String]) -> ParseOutcome {
    let mut args = Args::default();
    let mut i = 0usize;
    while i < argv.len() {
        let arg = argv[i].as_str();
        if arg == "--repo-root" {
            args.repo_root = argv.get(i + 1).cloned();
            i += 1;
        } else if let Some(v) = arg.strip_prefix("--repo-root=") {
            args.repo_root = Some(v.to_string());
        } else if arg == "--apply" {
            args.apply = true;
        } else if arg == "--json" {
            args.json = true;
        } else if arg == "--repo-hooks" {
            args.repo_hooks = true;
        } else if arg == "--claude-md" {
            args.claude_md = true;
        } else if arg == "--no-claude-md" {
            args.claude_md = false;
        } else if arg == "--global-skills" {
            args.global_skills = true;
        } else if arg == "--plugin-source" {
            args.plugin_source = true;
        } else if arg == "--force-downgrade" {
            args.force_downgrade = true;
        } else if arg == "--runtime" {
            match argv.get(i + 1) {
                Some(v) => args.runtime = v.clone(),
                // Node assigns `undefined`, which then fails the validation
                // below with "got: undefined".
                None => args.runtime = "undefined".to_string(),
            }
            i += 1;
        } else if let Some(v) = arg.strip_prefix("--runtime=") {
            args.runtime = v.to_string();
        } else if arg == "--help" || arg == "-h" {
            return ParseOutcome::Delegate;
        } else {
            return ParseOutcome::Error(format!("Unknown argument: {arg}"));
        }
        i += 1;
    }
    if !["claude", "codex", "both"].contains(&args.runtime.as_str()) {
        return ParseOutcome::Error(format!(
            "--runtime must be claude, codex, or both (got: {})",
            args.runtime
        ));
    }
    ParseOutcome::Parsed(Box::new(args))
}

// ── output ─────────────────────────────────────────────────────────────────

/// emit (l. 4086).
fn emit(payload: &Value, as_json: bool) {
    if as_json {
        print!("{}\n", jsjson::stringify_pretty(payload));
        return;
    }
    let s = |k: &str| payload.get(k).and_then(Value::as_str).unwrap_or("");
    print!("bee onboarding - repo: {}\n", s("repo_root"));
    print!("status: {}\n", s("status"));
    let empty: Vec<Value> = Vec::new();
    let items = payload
        .get("plan")
        .and_then(Value::as_array)
        .or_else(|| payload.get("applied").and_then(Value::as_array))
        .unwrap_or(&empty);
    for item in items {
        print!(
            "  {}  {}\n",
            item.get("action").and_then(Value::as_str).unwrap_or(""),
            item.get("path").and_then(Value::as_str).unwrap_or("")
        );
    }
    if items.is_empty() {
        print!("  (nothing to do)\n");
    }
    if let Some(reason) = payload.get("reason").and_then(Value::as_str) {
        print!("reason: {reason}\n");
    }
    if let Some(v) = payload.get("versions").filter(|v| !v.is_null()) {
        print!(
            "versions: source={} host_helpers={} installed_skills={}\n",
            v.get("source").and_then(Value::as_str).unwrap_or(""),
            v.get("host_helpers").and_then(Value::as_str).unwrap_or(""),
            v.get("installed_skills").and_then(Value::as_str).unwrap_or("")
        );
    }
    if let Some(skipped) =
        payload.get("skills").and_then(|s| s.get("skipped")).and_then(Value::as_array)
    {
        for sk in skipped {
            let target = sk.get("target").and_then(Value::as_str);
            let suffix = match target {
                Some(t) if !t.is_empty() => format!(" [{t}]"),
                _ => String::new(),
            };
            print!(
                "skipped skill: {}{suffix} - {}\n",
                sk.get("skill").and_then(Value::as_str).unwrap_or(""),
                sk.get("reason").and_then(Value::as_str).unwrap_or("")
            );
        }
    }
    if let Some(notices) = payload.get("notices").and_then(Value::as_array) {
        for n in notices {
            print!("notice: {}\n", n.as_str().unwrap_or(""));
        }
    }
}

/// sourceKindForReport (l. 4118).
fn source_kind_for_report(engine: &Engine) -> &'static str {
    source::classify_source_kind(&engine.skills_root.join("bee-hive"), &util::home_dir())
}

fn error_exit(message: &str) -> ExitCode {
    print!("{}\n", jsjson::stringify(&json!({ "error": message })));
    ExitCode::from(1)
}

/// `bee onboard` was spelled correctly and this binary cannot see the source
/// checkout it would vendor from. Says so, names the two ways forward, and —
/// because the installer branches on it — carries a machine-readable `kind`
/// rather than prose a caller has to regex.
pub(crate) const NO_ENGINE_KIND: &str = "engine_not_found";

/// The one host-repo remedy that actually runs there: `scripts/install.sh`
/// is not vendored into a target repo (it writes only `$TARGET_DIR/.bee/bin`,
/// see scripts/install.sh:583), so the fetchable one-liner is the only path
/// that works outside a bee checkout. Shared by this refusal and
/// status_full's agent-file-drift remedies so the two strings can never
/// drift apart.
pub(crate) const HOST_REPO_INSTALL_ONE_LINER: &str =
    "curl -fsSL https://raw.githubusercontent.com/thanhsmind/beehive/main/scripts/install.sh | bash -s -- -y";

/// The refusal's prose — a pure function so the test suite can assert both
/// the invocation root and the missing template path are named, without
/// capturing stdout/stderr. B-P2-3: when the caller passed `--repo-root`,
/// that candidate is named too, so a refusal reports every root it tried,
/// never only the cwd walk's.
fn no_engine_message(err: &source::LocateError) -> String {
    let repo_root_clause = match &err.repo_root_candidate {
        Some(r) => format!(" --repo-root ({}) was also tried and also missed the template.", r.display()),
        None => String::new(),
    };
    format!(
        "bee onboard: no bee source checkout is visible from this binary, and onboarding \
vendors its files FROM that checkout — the skills, the expertise layer, the AGENTS.md block. \
Searched upward from the invocation root ({}) for {} and found none.{repo_root_clause} FIX: run \
this from inside a bee checkout (`cd <bee>` first), or re-run the installer, which brings its \
own: {HOST_REPO_INSTALL_ONE_LINER}",
        err.invocation_root.display(),
        err.missing_template.display()
    )
}

fn no_engine_refusal(err: &source::LocateError, as_json: bool) -> ExitCode {
    let message = no_engine_message(err);
    if as_json {
        print!(
            "{}\n",
            jsjson::stringify_pretty(&json!({
                "status": "blocked_no_engine",
                "kind": NO_ENGINE_KIND,
                "error": message,
                "invocation_root": err.invocation_root.to_string_lossy(),
                "missing_template": err.missing_template.to_string_lossy(),
                "repo_root_candidate": err
                    .repo_root_candidate
                    .as_ref()
                    .map(|r| json!(r.to_string_lossy()))
                    .unwrap_or(Value::Null),
            }))
        );
    } else {
        eprintln!("{message}");
    }
    ExitCode::from(1)
}

fn version_value(v: &Option<String>) -> Value {
    v.as_ref().map(|s| json!(s)).unwrap_or(Value::Null)
}

// ── main ───────────────────────────────────────────────────────────────────

fn run(engine: &Engine, args: &Args) -> ExitCode {
    let (code, payload) = run_inner(engine, args);
    emit(&payload, args.json);
    code
}

/// Everything main() does except writing to stdout — the seam the in-file
/// suite asserts payloads through.
fn run_inner(engine: &Engine, args: &Args) -> (ExitCode, Value) {
    let repo_root: PathBuf = match &args.repo_root {
        Some(r) => util::path_resolve(r),
        None => util::path_resolve(""),
    };
    // Captured before any apply: "first onboard" means no marker yet.
    let first_onboard = !util::exists(&repo_root.join(".bee").join("onboarding.json"));

    let opts = Options {
        // --repo-hooks opts a repo IN; it is not a re-consent owed on every
        // upgrade (the record is sticky).
        repo_hooks: if args.plugin_source {
            false
        } else {
            args.repo_hooks || notices::has_repo_hooks_recorded(&repo_root)
        },
        claude_md: args.claude_md,
        global_skills: args.global_skills,
        sync_skills: !args.plugin_source,
        force_downgrade: args.force_downgrade,
        plugin_source: args.plugin_source,
        runtime: args.runtime.clone(),
    };
    let hooks_transition_notices = notices::repo_hooks_transition_notices(
        &repo_root,
        args.plugin_source,
        args.plugin_source && hooks_wiring::runtime_covers_codex(&args.runtime),
    );
    let build_notices = |extra: &[String]| -> Value {
        let mut out: Vec<Value> = Vec::new();
        for n in notices::commands_notices(&repo_root, first_onboard) {
            out.push(json!(n));
        }
        for n in notices::stale_advisor_notices(&repo_root) {
            out.push(json!(n));
        }
        for n in notices::tracked_paths_notices(&repo_root) {
            out.push(json!(n));
        }
        for n in extra {
            out.push(json!(n));
        }
        Value::Array(out)
    };

    if !args.apply {
        let computed = compute_plan(engine, &repo_root, &opts);
        let migration_blocked = !computed.worktree_migration.conflicts.is_empty();
        let status = if migration_blocked {
            "blocked_worktree_migration_conflict".to_string()
        } else if let Some(b) = &computed.skill_sync.blocked {
            b.status.clone()
        } else if core_changes_needed(&computed.plan) {
            "changes_needed".to_string()
        } else {
            "up_to_date".to_string()
        };
        let mut payload = Map::new();
        payload.insert("repo_root".into(), json!(repo_root.to_string_lossy()));
        payload.insert("status".into(), json!(status));
        payload.insert("source".into(), json!(source_kind_for_report(engine)));
        payload.insert("bee_version".into(), version_value(&computed.bee_version));
        payload.insert("plan".into(), Value::Array(computed.plan.clone()));
        payload.insert(
            "skills".into(),
            json!({
                "source_root": computed.skill_sync.source_root.to_string_lossy(),
                "targets": computed.skill_sync.targets.iter().map(|t| t.to_json()).collect::<Vec<_>>(),
            }),
        );
        payload.insert("notices".into(), build_notices(&hooks_transition_notices));
        if computed.worktree_migration.applicable {
            payload.insert(
                "worktree_migration".into(),
                json!({
                    "blocked": migration_blocked,
                    "pending": computed.worktree_migration.records.iter()
                        .filter(|r| r.status != migration::RecordStatus::Conflict).count(),
                    "stranded": migration::stranded_json(&computed.worktree_migration.conflicts),
                }),
            );
        }
        if migration_blocked {
            payload.insert(
                "reason".into(),
                json!(migration::build_migration_conflict_reason(
                    &computed.worktree_migration.conflicts
                )),
            );
        } else if let Some(b) = &computed.skill_sync.blocked {
            // Reporting is not failing: plan mode exits 0 with the status.
            payload.insert("reason".into(), json!(b.reason));
            payload.insert("versions".into(), b.versions.clone());
        }
        return (ExitCode::SUCCESS, Value::Object(payload));
    }

    match apply::apply_plan(engine, &repo_root, &opts) {
        ApplyOutcome::Blocked(blocked) => {
            // Refused apply: zero mutations happened; exit nonzero (D3).
            let mut payload = Map::new();
            payload.insert("repo_root".into(), json!(repo_root.to_string_lossy()));
            payload.insert("status".into(), json!(blocked.status));
            payload.insert("bee_version".into(), version_value(&blocked.bee_version));
            payload.insert("reason".into(), json!(blocked.reason));
            // `versions`/`skills` are `undefined` for the migration and
            // hook-collision refusals — JSON.stringify drops the keys.
            if let Some(v) = &blocked.versions {
                payload.insert("versions".into(), v.clone());
            }
            if let Some(s) = &blocked.skills {
                payload.insert("skills".into(), s.clone());
            }
            if let Some(h) = &blocked.host_items {
                payload.insert("host_items".into(), Value::Array(h.clone()));
            }
            if let Some(st) = &blocked.stranded {
                payload.insert(
                    "worktree_migration".into(),
                    json!({ "blocked": true, "stranded": st }),
                );
            }
            (ExitCode::from(1), Value::Object(payload))
        }
        ApplyOutcome::Ok(result) => {
            let ApplyOk {
                applied,
                onboarding,
                bee_version,
                forced_downgrade,
                forced_versions,
                skills,
            } = *result;
            let recheck = compute_plan(engine, &repo_root, &opts);
            // Review P1-7: blocked-first precedence — a recheck can NEVER
            // read "up_to_date" while ANY target is still blocked.
            let recheck_blocked = recheck.skill_sync.blocked.clone();
            let mut payload = Map::new();
            payload.insert("repo_root".into(), json!(repo_root.to_string_lossy()));
            payload.insert("status".into(), json!("applied"));
            payload.insert("bee_version".into(), version_value(&bee_version));
            payload.insert("applied".into(), Value::Array(applied));
            payload.insert(
                "recheck".into(),
                match &recheck_blocked {
                    Some(b) => json!(b.status),
                    None if core_changes_needed(&recheck.plan) => json!("changes_needed"),
                    None => json!("up_to_date"),
                },
            );
            payload.insert("recheck_plan".into(), Value::Array(recheck.plan.clone()));
            payload.insert(
                "recheck_skills".into(),
                match &recheck_blocked {
                    Some(b) => json!({
                        "blocked": true,
                        "reason": b.reason,
                        "versions": b.versions,
                        "targets": recheck.skill_sync.targets.iter()
                            .map(|t| t.to_recheck_json()).collect::<Vec<_>>(),
                    }),
                    None => Value::Null,
                },
            );
            payload.insert("skills".into(), skills);
            payload.insert("onboarding".into(), onboarding);
            payload.insert("notices".into(), build_notices(&hooks_transition_notices));
            if forced_downgrade {
                // F9: a forced apply reports the fact machine-readably.
                payload.insert("forced_downgrade".into(), json!(true));
                payload.insert("versions".into(), forced_versions.unwrap_or(Value::Null));
            }
            (ExitCode::SUCCESS, Value::Object(payload))
        }
    }
}

/// The host directories the `.bee/onboarding.json` managed-hash ledger covers,
/// for `verbs/cells.rs`'s regen obligation. Re-exported here so the obligation
/// depends on the onboard MODULE contract rather than reaching into a private
/// submodule.
pub(crate) fn ledger_covered_roots() -> Vec<&'static str> {
    plan::ledger_covered_roots()
}

pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    if args.first().and_then(|a| a.to_str()) != Some("onboard") {
        return None;
    }
    let argv: Vec<String> = args[1..].iter().map(|a| a.to_string_lossy().into_owned()).collect();
    match parse_args(&argv) {
        ParseOutcome::Delegate => None,
        ParseOutcome::Error(message) => Some(error_exit(&message)),
        ParseOutcome::Parsed(parsed) => {
            // Geometry first: without a locatable source checkout there is
            // nothing authoritative to vendor from, so decline the command
            // before producing a byte of output.
            //
            // DECLINING IS NOT THE SAME AS NOT EXISTING. Returning None here
            // used to mean "let Node answer"; since the cutover it drops the
            // argv into `emit_unsupported_shape`, which tells the caller that
            // `bee onboard --repo-root X --json` is an unsupported ARGUMENT
            // SHAPE — pointing at the flags, which are fine. The real answer
            // is that this binary cannot see a bee source checkout from where
            // it is standing. Two callers hit this constantly and neither
            // could act on the old wording: the installer, which runs a
            // downloaded binary from a temp dir while the clone sits
            // elsewhere, and every host repo, where `bee-hive` tells the agent
            // to run exactly this command when onboarding is stale.
            // B-P2-3: a caller-passed `--repo-root` is a locate CANDIDATE
            // too, resolved the same way `run_inner` resolves it for
            // everything else — so `bee onboard --repo-root <checkout>`
            // works from a cwd that shares no ancestry with that checkout.
            let repo_root_candidate: Option<PathBuf> =
                parsed.repo_root.as_deref().map(util::path_resolve);
            let engine = match Engine::locate(repo_root_candidate.as_deref()) {
                Ok(engine) => engine,
                Err(err) => return Some(no_engine_refusal(&err, parsed.json)),
            };
            Some(run(&engine, &parsed))
        }
    }
}

/// Test seam: run against an explicit plugin root instead of the discovered
/// one, so fixtures can stand in for a whole bee checkout.
#[cfg(test)]
fn run_with_root(plugin_root: &Path, argv: &[&str]) -> (ExitCode, Value) {
    let argv: Vec<String> = argv.iter().map(|s| (*s).to_string()).collect();
    let args = match parse_args(&argv) {
        ParseOutcome::Parsed(a) => *a,
        ParseOutcome::Error(m) => panic!("fixture argv must parse: {m}"),
        ParseOutcome::Delegate => panic!("fixture argv must not delegate"),
    };
    let engine = Engine::from_plugin_root(plugin_root.to_path_buf());
    run_inner(&engine, &args)
}
