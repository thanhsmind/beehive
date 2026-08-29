// exclusive-path globs and git classification
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

// ─── exclusive-path glob matcher and default deny-list ─────────────────────
//
// A narrow glob grammar purpose-built for exclusive-path gating: literal
// characters, `*` (any run within one path segment), `**` (any run of
// characters), and `**/` (zero or more whole path segments).

pub(crate) const DEFAULT_EXCLUSIVE_PATHS: [&str; 15] = [
    "**/migrations/**",
    "package-lock.json",
    "**/package-lock.json",
    "yarn.lock",
    "**/yarn.lock",
    "pnpm-lock.yaml",
    "**/pnpm-lock.yaml",
    "Cargo.lock",
    "**/Cargo.lock",
    "composer.lock",
    "**/composer.lock",
    "Gemfile.lock",
    "**/Gemfile.lock",
    "docs/history/codex-harness-hardening/release-manifest.json",
    ".bee/onboarding.json",
];

#[derive(Clone)]
pub(crate) enum GlobTok {
    Lit(char),
    Star,          // '*' matches a run of non-'/' characters within one path segment
    AnyAll,        // '**' (no trailing slash) matches any run of characters
    AnyDirsPrefix, // '**/' matches zero or more whole path segments
}

/// Tokenizes a glob pattern into the bee exclusive-path grammar above.
/// Normalizes backslashes to `/` and strips a leading `./`.
pub(crate) fn glob_tokens(glob: &str) -> Vec<GlobTok> {
    let normalized = {
        let s = glob.replace('\\', "/");
        if s.starts_with("./") {
            s[1..].trim_start_matches('/').to_string()
        } else {
            s
        }
    };
    let chars: Vec<char> = normalized.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            if chars.get(i + 2) == Some(&'/') {
                out.push(GlobTok::AnyDirsPrefix);
                i += 3;
            } else {
                out.push(GlobTok::AnyAll);
                i += 2;
            }
            continue;
        }
        if chars[i] == '*' {
            out.push(GlobTok::Star);
            i += 1;
            continue;
        }
        out.push(GlobTok::Lit(chars[i]));
        i += 1;
    }
    out
}

/// Recursive backtracking matcher over the bee exclusive-path grammar.
/// Tries the shortest match first, growing only on failure — the grammar has
/// no pathological cases (patterns are short, fixed config strings) so this
/// stays cheap without memoization.
pub(crate) fn glob_match(toks: &[GlobTok], input: &[char]) -> bool {
    match toks.first() {
        None => input.is_empty(),
        Some(GlobTok::Lit(c)) => {
            input.first() == Some(c) && glob_match(&toks[1..], &input[1..])
        }
        Some(GlobTok::Star) => {
            // Consume 0..n characters, stopping at a path separator.
            let mut n = 0usize;
            loop {
                if glob_match(&toks[1..], &input[n..]) {
                    return true;
                }
                if n >= input.len() || input[n] == '/' {
                    return false;
                }
                n += 1;
            }
        }
        Some(GlobTok::AnyAll) => {
            // Consume 0..n characters, no boundary restriction.
            let mut n = 0usize;
            loop {
                if glob_match(&toks[1..], &input[n..]) {
                    return true;
                }
                if n >= input.len() {
                    return false;
                }
                n += 1;
            }
        }
        Some(GlobTok::AnyDirsPrefix) => {
            // Zero whole segments, or any run of characters ending at a '/'.
            if glob_match(&toks[1..], input) {
                return true;
            }
            for (idx, &c) in input.iter().enumerate() {
                if c == '/' && glob_match(&toks[1..], &input[idx + 1..]) {
                    return true;
                }
            }
            false
        }
    }
}

/// True when `normalized` matches an exclusive-path glob: the built-in
/// defaults, EXTENDED (never replaced) by `config.guards.exclusive_paths`.
pub(crate) fn is_exclusive_path(root: &Path, normalized: &str) -> R<bool> {
    let config = read_config(root)?;
    let mut globs: Vec<String> = DEFAULT_EXCLUSIVE_PATHS.iter().map(|s| s.to_string()).collect();
    if let Some(Value::Object(g)) = config.get("guards") {
        if let Some(Value::Array(extra)) = g.get("exclusive_paths") {
            for e in extra {
                if let Value::String(s) = e {
                    if !js_trim(s).is_empty() {
                        globs.push(s.clone());
                    }
                }
            }
        }
    }
    let input: Vec<char> = normalized.chars().collect();
    Ok(globs.iter().any(|g| glob_match(&glob_tokens(g), &input)))
}

// ─── git classification ─────────────────────────────────────────────────

pub(crate) fn git_global_flag_takes_value(t: &str) -> bool {
    matches!(t, "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace")
}

pub(crate) struct GitInvocation {
    pub(crate) subcommand: Option<String>,
    pub(crate) rest: Vec<String>,
}

/// Every git invocation in the token list, in order — not just the first.
/// `git status && git stash` must classify BOTH invocations; classifying
/// only the first let a leading allowed command shadow a denied one after
/// it (the p1-guard-compound-bypass finding).
pub(crate) fn find_git_invocations(tokens: &[String]) -> Vec<GitInvocation> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        if is_separator(&tokens[i]) {
            i += 1;
            continue;
        }
        let cmd = tokens[i].replace('\\', "/");
        let cmd = cmd.rsplit('/').next().unwrap_or("");
        if cmd != "git" {
            i += 1;
            continue;
        }
        let mut end = i + 1;
        while end < tokens.len() && !is_separator(&tokens[end]) {
            end += 1;
        }
        let invocation: Vec<String> = tokens[i + 1..end].to_vec();
        let mut subcommand: Option<String> = None;
        let mut sub_idx: Option<usize> = None;
        let mut j = 0usize;
        while j < invocation.len() {
            let t = &invocation[j];
            if git_global_flag_takes_value(t) {
                j += 2;
                continue;
            }
            if t.starts_with('-') {
                j += 1;
                continue;
            }
            subcommand = Some(t.clone());
            sub_idx = Some(j);
            break;
        }
        out.push(match (subcommand, sub_idx) {
            (Some(s), Some(idx)) => GitInvocation {
                subcommand: Some(s),
                rest: invocation[idx + 1..].to_vec(),
            },
            _ => GitInvocation { subcommand: None, rest: Vec::new() },
        });
        i = end;
    }
    out
}

pub(crate) fn run_git_capture(cwd: &str, args: &[&str]) -> Option<Vec<String>> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None; // execFileSync throws on non-zero
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    Some(
        text.split(['\n'])
            .map(|l| js_trim(l.trim_end_matches('\r')).to_string())
            .filter(|l| !l.is_empty())
            .collect(),
    )
}

pub(crate) const GIT_BROAD_PATHSPECS: [&str; 4] = [".", ":", ":/", "./"];

pub(crate) fn extract_explicit_pathspecs(rest: &[String]) -> Vec<String> {
    match rest.iter().position(|t| t == "--") {
        None => rest.iter().filter(|t| !t.starts_with('-')).cloned().collect(),
        Some(idx) => rest[idx + 1..].to_vec(),
    }
}

pub(crate) fn resolve_git_mutation_paths(cwd: &str, subcommand: &str, rest: &[String]) -> Option<Vec<String>> {
    let broad = |p: &String| GIT_BROAD_PATHSPECS.contains(&p.as_str()) || p.contains('*');
    if subcommand == "commit" {
        let dash = rest.iter().position(|t| t == "--");
        let explicit: Vec<String> = match dash {
            None => Vec::new(),
            Some(idx) => rest[idx + 1..].to_vec(),
        };
        let pre: Vec<String> = match dash {
            None => rest.to_vec(),
            Some(idx) => rest[..idx].to_vec(),
        };
        let is_all = has_git_short_flag(&pre, 'a') || pre.iter().any(|t| t == "--all");
        let staged = run_git_capture(cwd, &["diff", "--cached", "--name-only"])?;
        if !explicit.is_empty() {
            if explicit.iter().any(broad) {
                return None;
            }
            return Some(explicit);
        }
        if !is_all {
            return Some(staged);
        }
        let unstaged = run_git_capture(cwd, &["diff", "--name-only"])?;
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for p in staged.into_iter().chain(unstaged.into_iter()) {
            if seen.insert(p.clone()) {
                out.push(p);
            }
        }
        return Some(out);
    }
    let pathspecs = extract_explicit_pathspecs(rest);
    if pathspecs.is_empty() {
        return None;
    }
    if pathspecs.iter().any(broad) {
        return None;
    }
    Some(pathspecs)
}

pub(crate) struct TreeVerbClass {
    pub(crate) verb: String,
    pub(crate) why: &'static str,
}

pub(crate) fn classify_concurrent_tree_verb(subcommand: Option<&str>, rest: &[String]) -> Option<TreeVerbClass> {
    let sub = subcommand?;
    if sub == "add" {
        if has_git_short_flag(rest, 'N') || rest.iter().any(|t| t == "--intent-to-add") {
            return None;
        }
        return Some(TreeVerbClass {
            verb: "add".into(),
            why: "it stages content into the SHARED index, so the next sibling worker to commit sweeps your files into their commit — the exact attribution loss that happened twice in one wave.",
        });
    }
    if sub == "commit" {
        let dash = rest.iter().position(|t| t == "--");
        let pre: Vec<String> = match dash {
            None => rest.to_vec(),
            Some(idx) => rest[..idx].to_vec(),
        };
        if has_git_short_flag(&pre, 'a') || pre.iter().any(|t| t == "--all") {
            return Some(TreeVerbClass {
                verb: "commit -a".into(),
                why: "`-a`/`--all` commits every tracked modification in the checkout, including a sibling worker's in-progress edits.",
            });
        }
        if let Some(idx) = dash {
            let pathspecs: Vec<String> = rest[idx + 1..].to_vec();
            if !pathspecs.is_empty()
                && !pathspecs
                    .iter()
                    .any(|p| GIT_BROAD_PATHSPECS.contains(&p.as_str()) || p.contains('*'))
            {
                return None;
            }
        }
        return Some(TreeVerbClass {
            verb: "commit".into(),
            why: "with no explicit `-- <paths>` pathspec it commits whatever sits in the SHARED index, which may be a sibling worker's staged work.",
        });
    }
    if sub == "stash" {
        let first_word = rest.iter().find(|t| !t.starts_with('-'));
        if let Some(w) = first_word {
            if matches!(w.as_str(), "list" | "show") {
                return None;
            }
        }
        return Some(TreeVerbClass {
            verb: "stash".into(),
            why: "it sweeps every uncommitted change in the checkout out of the tree, including edits a sibling worker is still writing.",
        });
    }
    if sub == "apply" {
        if rest.iter().any(|t| matches!(t.as_str(), "--check" | "--stat" | "--summary" | "--numstat")) {
            return None;
        }
        return Some(TreeVerbClass {
            verb: "apply".into(),
            why: "it rewrites tree content wholesale, and reservations cannot protect a tree.",
        });
    }
    if matches!(sub, "reset" | "clean" | "checkout" | "restore" | "revert" | "rebase" | "cherry-pick" | "merge") {
        return Some(TreeVerbClass {
            verb: sub.to_string(),
            why: "it rewrites the working tree or index as a whole, which no file reservation can protect — reservations govern FILES, and the working tree is not a file.",
        });
    }
    None
}

/// The `count > 1` arm's remedy — byte-identical to the text this function
/// used to hard-code for both arms, now that each arm supplies its own via
/// `concurrent_tree_refusal`'s `remedy` parameter.
pub(crate) const CONCURRENT_TREE_TEMP_INDEX_REMEDY: &str =
    "FIX: inspection is always allowed — git status / git diff / git log. To land your own work, make ONE path-scoped \
commit through your OWN temp index instead of the shared one: \
GIT_INDEX_FILE=<tmp> git read-tree HEAD, then GIT_INDEX_FILE=<tmp> git update-index --add <your paths>, \
GIT_INDEX_FILE=<tmp> git write-tree, git commit-tree <tree> -p HEAD -m \"<msg>\", git update-ref HEAD <commit>. \
For a path git does not track yet, `git add -N <path>` first (intent-to-add stages no content). \
A genuinely path-scoped `git commit -- <your paths>` is allowed too. Never reset / stash / checkout / clean / \
restore / revert across the shared tree while a sibling worker holds work in it — a file reservation cannot protect a tree.";

pub(crate) fn concurrent_tree_refusal(verb: &str, why: &str, worker_clause: &str, remedy: &str) -> String {
    format!("bee concurrent-worker git guard: `git {verb}` is refused because {worker_clause}. {why} {remedy}")
}

pub(crate) fn session_workspace_id(control_root: &str, session_id: &Value) -> R<String> {
    let sid = match session_id {
        Value::String(s) => s.clone(),
        _ => return Ok("main".to_string()), // requireId throw → readSession null → 'main'
    };
    let session = read_session(control_root, &sid)?;
    Ok(match session.and_then(|s| s.get("workspace_id").cloned()) {
        Some(Value::String(w)) if !js_trim(&w).is_empty() => w,
        _ => "main".to_string(),
    })
}

pub(crate) enum WorkerCount {
    Resolved(usize),
    Unresolved(&'static str),
}

/// gc-2 (wgg-2): an acting-holder string no hold row can ever carry — a
/// holder is a git-verified worktree id or the literal `"main"`, and neither
/// can hold a NUL. Handed to `find_foreign_holds`, whose `holder !== acting`
/// filter then discards nothing, so the "foreign" reader returns EVERY active
/// hold. That is exactly the trick `bee reservations list` plays for its
/// `cross_worktree:` section (verbs/reservations/leases.rs
/// LIST_ALL_HOLDS_SENTINEL); this is the write-guard's own spelling of it, so
/// the guard reuses the mirrored-holds reader instead of growing a second one.
const ALL_HOLDS_SENTINEL: &str = "\u{0}bee-write-guard-all-holds\u{0}";

/// The request path that overlaps every hold that carries a path at all:
/// `paths_overlap` strips a trailing `*` down to an empty prefix, and an empty
/// prefix overlaps any non-empty path. A hold with no path coerces to `""`,
/// which `paths_overlap` rejects on both sides — not a hold anyone can hold.
const ALL_HOLDS_PATH: &str = "*";

/// gc-2 (wgg-2): how many workers actually share a GRANTED worktree's index.
///
/// The worktree's own `.bee/reservations.json` is the wrong place to ask: a
/// wave's reservations are written by the orchestrator from the control root,
/// so the worktree's store reads empty while three siblings edit its tree
/// (docs/history/wave-guard-gaps/CONTEXT.md, "Gap 2"). The mirrored-holds
/// ledger at the control root is the one record that spans both checkouts, and
/// it already stamps each row with the work stream that owns it — `holder` is
/// the granted worktree's id for a cell whose feature owns that worktree
/// (verbs/reservations/reserve.rs, hha-1).
///
/// A worker is its cell: bee hands exactly one cell to one worker, so distinct
/// non-empty `cell` values among this worktree's active holds ARE the sibling
/// count. A hold with no cell names no worker and is skipped rather than
/// guessed at — counting it could push a lone worker to two, and blocking a
/// solo worker is a worse defect than the blindness being fixed.
fn worktree_hold_cohort_size(control_root: &str, own_workspace: &str) -> R<usize> {
    let holds = find_foreign_holds(control_root, ALL_HOLDS_SENTINEL, &[ALL_HOLDS_PATH.to_string()])?;
    let mut cells: std::collections::HashSet<String> = std::collections::HashSet::new();
    for hold in &holds {
        match hold.get("holder") {
            Some(Value::String(h)) if h == own_workspace => {}
            _ => continue,
        }
        let cell = match hold.get("cell") {
            Some(Value::String(c)) => js_trim(c).to_string(),
            _ => String::new(),
        };
        if cell.is_empty() {
            continue;
        }
        cells.insert(cell);
    }
    Ok(cells.len())
}

pub(crate) fn resolve_live_worker_count(root: &str, control_root: &str, ctx: &JsCtx) -> R<WorkerCount> {
    let own_workspace = ctx.workspace_id.clone().unwrap_or_else(|| "main".to_string());
    if reservation_store_corrupt(root) {
        return Ok(WorkerCount::Unresolved("the reservation store is present but unparseable"));
    }
    let reservations = list_active_reservations(root)?;
    let mut worker_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut sessions_with_agents: std::collections::HashSet<String> = std::collections::HashSet::new();
    for resv in &reservations {
        let agent = match &resv.agent {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        if agent.is_empty() {
            continue;
        }
        let session = match &resv.session {
            Some(Value::String(s)) => js_trim(s).to_string(),
            _ => String::new(),
        };
        // sameWorkspace: unattributed counts; else compare stamped ids.
        if !session.is_empty()
            && session_workspace_id(control_root, &Value::String(session.clone()))? != own_workspace
        {
            continue;
        }
        worker_keys.insert(format!("{}::agent:{}", session, agent));
        if !session.is_empty() {
            sessions_with_agents.insert(session);
        }
    }
    let workers = active_worker_session_ids(control_root, None)?;
    for sid in workers {
        let sid_t = js_trim(&sid).to_string();
        if sid_t.is_empty() || sessions_with_agents.contains(&sid_t) {
            continue;
        }
        if session_workspace_id(control_root, &Value::String(sid_t.clone()))? != own_workspace {
            continue;
        }
        worker_keys.insert(format!("{}::session", sid_t));
    }
    // gc-2 (wgg-2): inside a GRANTED worktree, the two halves above read the
    // wrong checkout and both resolve to zero — `count > 1` was unreachable in
    // the one place parallel workers actually run. The mirrored-holds cohort
    // is the third, cross-checkout half.
    //
    // `workspace_id` is the worktree id ONLY for a granted worktree; every
    // ordinary checkout (and every ungranted one) stamps `"main"`, so this
    // whole block is skipped there and the main-checkout verdict is unchanged,
    // byte for byte.
    //
    // `max`, never a sum: the cohort is an INDEPENDENT view of the same
    // workers, not extra ones. A solo worker that reserved its own path is
    // seen once by the reservation half and once by the cohort; adding them
    // would deny the one session that must never be denied.
    if own_workspace != "main" {
        // Same fail-safe shape the reservation store already takes: an
        // unreadable record is "more than one worker", never a silent zero.
        if holds_store_corrupt(control_root) {
            return Ok(WorkerCount::Unresolved(
                "the cross-worktree holds ledger .bee/runtime/cross-worktree-holds.json is present but unparseable",
            ));
        }
        let cohort = worktree_hold_cohort_size(control_root, &own_workspace)?;
        return Ok(WorkerCount::Resolved(worker_keys.len().max(cohort)));
    }
    Ok(WorkerCount::Resolved(worker_keys.len()))
}

/// The opt-out used to be spelled `bee config set --key guards.idle_gate`, but
/// `bee config` is not a built verb, so the busiest guard in the harness —
/// the first refusal most people ever see — would have ended a paragraph of
/// good advice by naming a command that answers "not built into this
/// binary". It names the file instead.
pub(crate) fn intake_fix_line() -> String {
    format!(
        "FIX: commit or write bookkeeping directly — {} are exempt from this gate — \
or route the request through bee-hive first (classify the mode; tiny fixes stay tiny — one cell, a 2-minute \
reality check, Gate 2, go), then execute. Last resort, repo-level opt-out: \
set guards.idle_gate to false in .bee/config.json (plain JSON; delete the key to re-enable).",
        GATE_ALLOWED_PREFIXES_INTAKE.join(", ")
    )
}

/// sfg-1 / slp-followup-gaps D2. The intake FIX line tells the caller to
/// route the request through the workflow. For a session that is bound to no
/// lane, that is the wrong remedy: the work IS routed — this session just is
/// not attached to it, so the gate read the control-root default record
/// instead of the lane the session is actually working under. Named only when
/// the default record answered AND this session carries no lane binding; a
/// lane-bound session, a claim-derived one, and a sessionless call never see
/// this line.
pub(crate) fn session_bind_remedy_line() -> String {
    " ALSO: this session is bound to no lane, so the gate judged it against the default record \
(.bee/state.json), not the lane it is working under — bind it to that lane \
(`bee state session bind --lane <feature>`), or claim its cell, and retry."
        .to_string()
}

pub(crate) fn intake_refusal(
    phase: &Value,
    blocked: &str,
    extra: &str,
    session_bind_remedy: bool,
) -> String {
    format!(
        "bee intake gate: no bee work is active (phase: {}) — {} is blocked. {}{}{}",
        js_disp(phase),
        blocked,
        extra,
        intake_fix_line(),
        if session_bind_remedy { session_bind_remedy_line() } else { String::new() }
    )
}

pub(crate) fn is_terminal_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if s == "idle" || s == "compounding-complete")
}

pub(crate) fn is_gated_phase(phase: &Value) -> bool {
    matches!(phase, Value::String(s) if s == "exploring" || s == "planning")
}
