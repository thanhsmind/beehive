//! `queen-bee status [--json] [--lanes-full]` — the byte-parity port of
//! `bee.mjs`'s `buildStatus` (bee.mjs:724) + `renderStatusText`
//! (bee.mjs:927), assembled from the readers rust-port-13/14/16/20 already
//! ported into `bee_core` (CONTEXT.md D3 byte-compatibility, D5 hot path,
//! D7a parity).
//!
//! ## Why this lives in `queen-bee`, not `bee-core`
//!
//! Three of `buildStatus`'s inputs are *binary-crate* concerns by the
//! convention rust-port-16/20 established: `resolveRoots`/`controlRootFor`
//! (the git-worktree walk, `crate::adapter`), `resolveSessionId` (env +
//! live-session inference) and `activeWorkers` (sessions x claims). Each
//! bee-core reader takes an already-resolved root/session id instead of
//! re-deriving it, so the composition that resolves them belongs on this
//! side of the crate graph. Nothing here parses a store file that a
//! bee-core reader does not already own (this cell's `key_links`): the
//! three helpers below (`resolve_session_id`, `active_workers`, plus the
//! pure `datamark`/`bypass_banner`/normalizers) compose bee-core's public
//! primitives only.
//!
//! ## Byte-parity notes
//!
//! - Key ORDER is part of the contract: `JSON.stringify` emits JS
//!   insertion order, so every object here is built in the mjs source's
//!   literal order and `serde_json`'s `preserve_order` feature keeps it
//!   (see `crates/bee-core/Cargo.toml`).
//! - A JS object literal whose value is `undefined` is DROPPED by
//!   `JSON.stringify`. Where the mjs source can produce that (a decision
//!   event with no `id`, a session with no `last_heartbeat`), this port
//!   omits the key rather than writing `null`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bee_core::{backlog, capture, cells, claims, config, recovery, reservations, reviews, source_identity, state};
use serde_json::{json, Map, Value};

use crate::adapter;

/// bee.mjs:387 `STALE_HANDOFF_MS`.
const STALE_HANDOFF_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// bee.mjs:394 `POST_EXECUTION_REVIEW_PHASES`.
const POST_EXECUTION_REVIEW_PHASES: [&str; 3] = ["scribing", "compounding", "compounding-complete"];

/// state.mjs:111 `COMMAND_KEYS`.
const COMMAND_KEYS: [&str; 4] = ["setup", "start", "test", "verify"];

/// state.mjs:124 `WORKTREE_COMPANION_COMMAND_KEYS`.
const WORKTREE_COMPANION_COMMAND_KEYS: [&str; 3] =
    ["worktree_companion_start", "worktree_companion_end", "worktree_companion_mount"];

/// state.mjs `MODEL_NORMALIZE_SLOTS` = `[...CONFIGURABLE_SLOTS, 'advisor']`.
const MODEL_NORMALIZE_SLOTS: [&str; 4] = ["extraction", "generation", "review", "advisor"];

// ─── options + entry point ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub struct StatusOptions {
    /// `--lanes-full`: restore `buildLaneRows`'s full array (lpsp-2).
    pub lanes_full: bool,
}

/// Per-block wall time, in source order, for `status --profile`.
///
/// D5's escape hatch is explicit: a command that cannot reach its budget
/// gets "the measured p95 plus a profile", never a shrunk fixture. This is
/// that profile. It is written to STDERR only — stdout stays byte-identical
/// to the frozen oracle whether or not profiling is on, so no parity leg
/// can be affected by measuring.
#[derive(Debug, Default, Clone)]
pub struct Profile {
    pub blocks: Vec<(&'static str, f64)>,
    /// Wall time from `main()` entry to just after stdout is written —
    /// everything the process does except OS spawn and teardown. The gap
    /// between this and the block sum is the in-process ENVELOPE (arg
    /// parsing, root resolution, the untimed glue between blocks), and it
    /// is the figure that says whether a 5 ms budget was ever reachable by
    /// ANY implementation on this box, independent of reader cost.
    pub process_ms: Option<f64>,
}

impl Profile {
    pub fn record(&mut self, label: &'static str, started: std::time::Instant) {
        self.blocks.push((label, started.elapsed().as_secs_f64() * 1000.0));
    }

    pub fn set_process_ms(&mut self, started: std::time::Instant) {
        self.process_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
    }

    pub fn blocks_total_ms(&self) -> f64 {
        self.blocks.iter().map(|(_, ms)| ms).sum()
    }

    pub fn render(&self) -> String {
        let total = self.blocks_total_ms();
        let mut sorted: Vec<&(&'static str, f64)> = self.blocks.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut out = String::new();
        match self.process_ms {
            Some(process) => out.push_str(&format!(
                "queen-bee status profile — {process:.3} ms in-process (main entry → stdout written); {total:.3} ms in measured blocks; {:.3} ms untimed envelope (arg parse, root resolution, inter-block glue). OS spawn + teardown are on top of this and are NOT included.\n",
                process - total
            )),
            None => out.push_str(&format!("queen-bee status profile — {total:.3} ms of measured block time\n")),
        }
        for (label, ms) in sorted {
            out.push_str(&format!("  {ms:>9.3} ms  {:>5.1}%  {label}\n", if total > 0.0 { ms / total * 100.0 } else { 0.0 }));
        }
        out
    }
}

/// Time one block into `profile` (a no-op cost when profiling is off is
/// not worth branching for — `Instant::now` is a vDSO read).
macro_rules! timed {
    ($profile:expr, $label:expr, $body:expr) => {{
        let __start = std::time::Instant::now();
        let __value = $body;
        $profile.record($label, __start);
        __value
    }};
}

/// Everything `buildStatus` needs that is resolved OUTSIDE the store: the
/// mjs source reads these from `process.cwd()` / `Date.now()` / the
/// environment. Passing them in keeps the composition testable and keeps
/// the "harness never touches the live store" discipline checkable.
#[derive(Debug, Clone)]
pub struct StatusContext {
    /// `main()`'s already-resolved `storeRoot` (state.mjs `findRepoRoot`).
    pub root: PathBuf,
    /// `controlRootFor(root)`.
    pub control_root: PathBuf,
    /// `resolveRoots(process.cwd())`, for the ungranted-worktree notice.
    pub cwd_roots: state::WorktreeRootsView,
    /// `resolveSessionId()` — env-only chain, matching the mjs call sites
    /// that pass no `root` (buildLaneSummary passes `{root: ctrlRoot}`;
    /// that variant is resolved in [`resolve_session_id`]).
    pub session_id: Option<String>,
    /// `Date.now()`.
    pub now_ms: i64,
    /// `os.homedir()`.
    pub home_dir: Option<PathBuf>,
    /// `claudeProjectsRoot()`.
    pub projects_root: PathBuf,
}

impl StatusContext {
    /// Resolve every out-of-store input from the live process, exactly as
    /// `bee.mjs` main()/buildStatus do.
    pub fn from_process(root: &Path) -> Self {
        let control_root = adapter::control_root_for(root);
        let cwd = std::env::current_dir().unwrap_or_else(|_| root.to_path_buf());
        let roots = adapter::resolve_roots(&cwd);
        let cwd_roots = state::WorktreeRootsView {
            worktree_resolution: roots.worktree_resolution.to_string(),
            store_root: roots.store_root.clone(),
            main_root: roots.main_root.clone(),
        };
        StatusContext {
            root: root.to_path_buf(),
            control_root: control_root.clone(),
            cwd_roots,
            session_id: resolve_session_id(Some(&control_root)),
            now_ms: now_ms(),
            home_dir: home_dir(),
            projects_root: recovery::claude_projects_root(),
        }
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

// ─── helpers the mjs layer keeps outside bee-core's reader set ───────────

/// Port of `claims.mjs:139` `resolveSessionId({flag, root})`, flag-less:
/// `BEE_SESSION_ID` -> `CLAUDE_CODE_SESSION_ID` -> (when `root` is given)
/// the single non-stale session record on disk, else `None`.
pub fn resolve_session_id(root: Option<&Path>) -> Option<String> {
    for key in ["BEE_SESSION_ID", "CLAUDE_CODE_SESSION_ID"] {
        if let Some(v) = std::env::var_os(key) {
            let s = v.to_string_lossy().trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    let root = root?;
    let now = now_ms();
    let fresh: Vec<claims::Session> = claims::list_session_records(root)
        .into_iter()
        .filter(|s| !claims::heartbeat_stale(Some(s), now, claims::DEFAULT_HEARTBEAT_STALE_SECONDS))
        .collect();
    if fresh.len() == 1 {
        return Some(fresh[0].id.clone());
    }
    None
}

/// Port of `claims.mjs:367` `activeWorkers(root)` — the derived
/// live-sessions x active-claims view `buildStatus` puts in `workers`.
/// One row per live session: `{session_id, lane, cell, last_heartbeat}`,
/// with `last_heartbeat` OMITTED when the record carries none (JS
/// `undefined` drops out of `JSON.stringify`).
pub fn active_workers(root: &Path, now_ms: i64) -> Vec<Value> {
    let live: Vec<claims::Session> = claims::list_session_records(root)
        .into_iter()
        .filter(|s| !claims::heartbeat_stale(Some(s), now_ms, claims::DEFAULT_HEARTBEAT_STALE_SECONDS))
        .collect();
    if live.is_empty() {
        return Vec::new();
    }

    // First active claim seen for a session wins (one row per worker).
    let mut claim_cell_by_session: HashMap<String, Value> = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(claims::claims_dir(root)) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(stem) = name.strip_suffix(".json") else { continue };
            let Some(claim) = claims::read_claim(root, stem) else { continue };
            let Some(session) = claim.session.clone().filter(|s| !s.is_empty()) else { continue };
            if !claims::is_claim_active(&claim, now_ms) {
                continue;
            }
            claim_cell_by_session
                .entry(session)
                .or_insert_with(|| claim.extra.get("cell").cloned().unwrap_or(Value::Null));
        }
    }

    live.iter()
        .map(|session| {
            let mut row = Map::new();
            row.insert("session_id".to_string(), Value::String(session.id.clone()));
            row.insert(
                "lane".to_string(),
                match session.lane.as_deref().filter(|l| !l.is_empty()) {
                    Some(l) => Value::String(l.to_string()),
                    None => Value::Null,
                },
            );
            row.insert(
                "cell".to_string(),
                claim_cell_by_session.get(&session.id).cloned().unwrap_or(Value::Null),
            );
            // `session.last_heartbeat` is `undefined` on a record without
            // the key -> the object literal drops it entirely.
            if let Some(hb) = &session.last_heartbeat {
                row.insert("last_heartbeat".to_string(), Value::String(hb.clone()));
            }
            Value::Object(row)
        })
        .collect()
}

/// Port of `decisions.mjs:1047` `datamark(text)` — neutralize resurfaced
/// text so it can never act as instructions.
pub fn datamark(text: &str) -> String {
    // 1. strip runs of 3+ backticks (`/```+/g` — three or more).
    let mut stage = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '`' {
            let mut run = 0usize;
            while i + run < chars.len() && chars[i + run] == '`' {
                run += 1;
            }
            if run >= 3 {
                i += run;
                continue;
            }
            for _ in 0..run {
                stage.push('`');
            }
            i += run;
            continue;
        }
        stage.push(chars[i]);
        i += 1;
    }

    // 2. strip role-ish tags: /<\/?\s*(?:system|assistant|user|developer|tool)\b[^>]*>/gi
    let stage = strip_role_tags(&stage);

    // 3. drop control chars: [ --]
    let cleaned: String = stage
        .chars()
        .filter(|c| {
            let n = *c as u32;
            !(n <= 0x08 || n == 0x0B || n == 0x0C || (0x0E..=0x1F).contains(&n) || n == 0x7F)
        })
        .collect();

    // 4. JS `String.prototype.trim` strips whitespace AND line terminators.
    format!("«{}»", cleaned.trim_matches(is_js_trim_char))
}

/// JS `trim()` removes WhiteSpace + LineTerminator code points.
fn is_js_trim_char(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000D}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}'
        | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}' | '\u{FEFF}')
}

const ROLE_TAGS: [&str; 5] = ["system", "assistant", "user", "developer", "tool"];

fn strip_role_tags(text: &str) -> String {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == '<' {
            if let Some(end) = match_role_tag(&bytes, i) {
                i = end;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

/// `<` `/`? `\s*` (tag) `\b` `[^>]*` `>` — returns the index just past `>`.
fn match_role_tag(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start + 1;
    if chars.get(i) == Some(&'/') {
        i += 1;
    }
    while chars.get(i).is_some_and(|c| c.is_whitespace()) {
        i += 1;
    }
    let tag = ROLE_TAGS.iter().find(|tag| {
        let t: Vec<char> = tag.chars().collect();
        if chars.len() < i + t.len() {
            return false;
        }
        chars[i..i + t.len()].iter().zip(t.iter()).all(|(a, b)| a.eq_ignore_ascii_case(b))
    })?;
    let after = i + tag.chars().count();
    // `\b`: the next char must not be a word char [A-Za-z0-9_].
    if chars.get(after).is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_') {
        return None;
    }
    // `[^>]*>`
    let mut j = after;
    while j < chars.len() && chars[j] != '>' {
        j += 1;
    }
    if j < chars.len() && chars[j] == '>' {
        Some(j + 1)
    } else {
        None
    }
}

/// Port of `state.mjs:1932` `bypassBanner(level)`.
pub fn bypass_banner(level: &str) -> &'static str {
    match level {
        "total" => "⚡⚡⚡ GATE BYPASS: TOTAL AUTOPILOT — ZERO STOPS. Every gate (any lane, high-risk/hard-gate included), secret-file reads, and review P1 findings auto-proceed; NO human checkpoint remains. Turn off: bee-bypass-gate off",
        "full" => "⚡⚡ GATE BYPASS: FULL AUTOPILOT — ALL Gates 1-3 auto-approved including high-risk/hard-gate work; only secret-file reads and a review P1 finding still stop for the human. Turn off: bee-bypass-gate off",
        "normal" => "⚡ GATE BYPASS: NORMAL — Gates 1-3 auto-approved for tiny/small/standard work only; high-risk/hard-gate, secret reads, and Gate 4 UAT still stop. Turn off: bee-bypass-gate off",
        _ => "",
    }
}

// ─── config normalizers (state.mjs readConfig's own parse path) ──────────

/// Port of `state.mjs:126` `normalizeCommands(raw)` — key ORDER follows
/// the iteration order of `[...COMMAND_KEYS, ...WORKTREE_COMPANION_...]`,
/// not the source file's order.
fn normalize_commands(raw: Option<&Value>) -> Value {
    let mut out = Map::new();
    let Some(Value::Object(src)) = raw else { return Value::Object(out) };
    for key in COMMAND_KEYS.iter().chain(WORKTREE_COMPANION_COMMAND_KEYS.iter()) {
        if let Some(Value::String(s)) = src.get(*key) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                out.insert((*key).to_string(), Value::String(trimmed.to_string()));
            }
        }
    }
    Value::Object(out)
}

fn effort_ok(effort: &str) -> bool {
    matches!(effort, "low" | "medium" | "high" | "xhigh" | "max")
}

/// Port of `state.mjs:221` `normalizeTierValue(value)`. `None` is JS
/// `undefined` (the slot keeps its default); `Some(Value::Null)` is an
/// explicit `null` override.
fn normalize_tier_value(value: Option<&Value>) -> Option<Value> {
    match value {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(Value::String(s.trim().to_string())),
        Some(Value::Null) => Some(Value::Null),
        Some(Value::Object(map)) => normalize_tier_object(map),
        _ => None,
    }
}

fn trimmed_str(map: &Map<String, Value>, key: &str) -> Option<String> {
    match map.get(key) {
        Some(Value::String(s)) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

fn normalize_tier_object(map: &Map<String, Value>) -> Option<Value> {
    let kind = map.get("kind").and_then(Value::as_str);

    if kind == Some("cli") {
        if let Some(command) = trimmed_str(map, "command") {
            let mut out = Map::new();
            out.insert("kind".to_string(), Value::String("cli".to_string()));
            out.insert("command".to_string(), Value::String(command));
            return Some(Value::Object(out));
        }
    }

    if kind == Some("native") {
        if let Some(model) = trimmed_str(map, "model") {
            let mut out = Map::new();
            out.insert("kind".to_string(), Value::String("native".to_string()));
            out.insert("model".to_string(), Value::String(model));
            if let Some(effort) = trimmed_str(map, "effort").filter(|e| effort_ok(e)) {
                out.insert("effort".to_string(), Value::String(effort));
            }
            if trimmed_str(map, "fork_turns").as_deref() == Some("none") {
                out.insert("fork_turns".to_string(), Value::String("none".to_string()));
            }
            if let Some(agent_type) = trimmed_str(map, "agent_type") {
                out.insert("agent_type".to_string(), Value::String(agent_type));
            }
            return Some(Value::Object(out));
        }
    }

    // Explicit-fallback composite.
    if let Some(Value::Object(primary)) = map.get("primary") {
        let primary_native = primary.get("kind").and_then(Value::as_str) == Some("native")
            && trimmed_str(primary, "model").is_some();
        if primary_native {
            let mut out = Map::new();
            out.insert("primary".to_string(), normalize_tier_object(primary)?);
            if map.get("fallback_policy").and_then(Value::as_str) == Some("explicit-only") {
                out.insert("fallback_policy".to_string(), Value::String("explicit-only".to_string()));
                if let Some(Value::Object(fb)) = map.get("fallback") {
                    if fb.get("kind").and_then(Value::as_str) == Some("cli") {
                        if let Some(command) = trimmed_str(fb, "command") {
                            let mut fbo = Map::new();
                            fbo.insert("kind".to_string(), Value::String("cli".to_string()));
                            fbo.insert("command".to_string(), Value::String(command));
                            out.insert("fallback".to_string(), Value::Object(fbo));
                        }
                    }
                }
            }
            return Some(Value::Object(out));
        }
    }

    if kind.is_none() && !map.contains_key("kind") {
        if let Some(model) = trimmed_str(map, "model") {
            let mut out = Map::new();
            out.insert("model".to_string(), Value::String(model));
            if let Some(effort) = trimmed_str(map, "effort").filter(|e| effort_ok(e)) {
                out.insert("effort".to_string(), Value::String(effort));
            }
            return Some(Value::Object(out));
        }
    }

    None
}

/// Port of `state.mjs:334` `normalizeModels(raw)`. The default map's key
/// order is the emitted order: overriding an existing slot replaces its
/// value in place (JS object assignment), it never reorders.
fn normalize_models(raw: Option<&Value>) -> Value {
    let mut claude = Map::new();
    claude.insert("extraction".to_string(), json!("haiku"));
    claude.insert("generation".to_string(), json!("sonnet"));
    claude.insert("review".to_string(), json!("opus"));
    let mut codex = Map::new();
    codex.insert("extraction".to_string(), Value::Null);
    codex.insert("generation".to_string(), Value::Null);
    codex.insert("review".to_string(), Value::Null);

    let mut out = Map::new();
    out.insert("claude".to_string(), Value::Object(claude));
    out.insert("codex".to_string(), Value::Object(codex));

    if let Some(Value::Object(src)) = raw {
        for rt in ["claude", "codex"] {
            let Some(Value::Object(rt_src)) = src.get(rt) else { continue };
            for slot in MODEL_NORMALIZE_SLOTS {
                if let Some(normalized) = normalize_tier_value(rt_src.get(slot)) {
                    if let Some(Value::Object(target)) = out.get_mut(rt) {
                        target.insert(slot.to_string(), normalized);
                    }
                }
            }
        }
    }
    Value::Object(out)
}

// ─── buildStatus ─────────────────────────────────────────────────────────

/// Port of `bee.mjs:724` `buildStatus(root, {lanesFull})`.
pub fn build_status(ctx: &StatusContext, opts: StatusOptions) -> Value {
    let mut profile = Profile::default();
    build_status_profiled(ctx, opts, &mut profile)
}

/// [`build_status`] with per-block timings collected into `profile`. The
/// payload is byte-identical either way — measuring never changes output.
pub fn build_status_profiled(ctx: &StatusContext, opts: StatusOptions, profile: &mut Profile) -> Value {
    let root = ctx.root.as_path();
    // rust-port-23 (decision e119fc8b): ONE load point per store for this
    // whole invocation. `build_status` used to cost 4 decisions-journal
    // parses, 6 cells-directory scans and 2 transcript-root scans; every
    // reader below that used to perform its own read now takes this memo.
    // It is lazy (see `SharedReads`' doc comment), so nothing here is read
    // before something actually asks for it, and it is per-invocation:
    // constructed here, dropped at the end, never a process-global or
    // on-disk cache.
    let shared = bee_core::shared_reads::SharedReads::new(root);
    let state_rec = timed!(profile, "read_state", state::read_state(root));
    let onboarding_raw = timed!(profile, "read_onboarding", state::read_onboarding(root));
    let handoff = timed!(profile, "read_handoff", state::read_handoff(root));

    let cells_list = timed!(profile, "list_cells (counts)", shared.cells());
    let mut open = 0i64;
    let mut claimed = 0i64;
    let mut capped = 0i64;
    let mut blocked = 0i64;
    for cell in cells_list {
        match cell.status.as_deref().unwrap_or("") {
            "open" => open += 1,
            "claimed" => claimed += 1,
            "capped" => capped += 1,
            "blocked" => blocked += 1,
            _ => {}
        }
    }
    let archived = timed!(profile, "archived_totals", cells::archived_totals(root));

    // msn-18b: reservations are control-plane; bee-core's reader takes the
    // already-resolved root (rust-port-20's caller-supplies-control-root).
    let active_reservations = timed!(
        profile,
        "list_reservations (active)",
        reservations::list_reservations(&ctx.control_root, true, ctx.now_ms)
    );
    let expired_unreleased = timed!(
        profile,
        "expired_unreleased_reservations",
        reservations::expired_unreleased_reservations(&ctx.control_root, ctx.now_ms)
    );

    let raw_config = timed!(profile, "read_config_value", config::read_config_value(root));
    let commands = normalize_commands(raw_config.get("commands"));
    let models = normalize_models(raw_config.get("models"));

    let backlog_counts = timed!(profile, "read_backlog_counts", backlog::read_backlog_counts(root));

    let mut staleness: Vec<String> = Vec::new();
    if commands.as_object().map(Map::is_empty).unwrap_or(true) {
        staleness.push("No standard commands recorded — capture the host project's setup/start/test/verify into .bee/config.json `commands` so sessions can run the CI status gate.".to_string());
    }
    if let Some(v) = onboarding_raw.get("bee_version").and_then(Value::as_str) {
        if !v.is_empty() && v != state::BEE_VERSION {
            staleness.push(format!(
                "Onboarding installed bee {v} but plugin is {} — re-run onboarding.",
                state::BEE_VERSION
            ));
        }
    }
    if js_truthy(&handoff) {
        if let Some(written_at) = handoff.get("written_at").and_then(Value::as_str) {
            if let Some(written_ms) = bee_core::jsdate::parse_iso_ms(written_at) {
                let age = ctx.now_ms - written_ms;
                if age > STALE_HANDOFF_MS {
                    staleness.push(format!("HANDOFF.json is older than 7 days (written {written_at})."));
                }
            }
        }
    }
    if !expired_unreleased.is_empty() {
        staleness.push(format!(
            "{} reservation(s) expired but never released — run bee_reservations.mjs sweep.",
            expired_unreleased.len()
        ));
    }
    if timed!(profile, "has_stale_advisor_key", state::has_stale_advisor_key(root)) {
        staleness.push(state::STALE_ADVISOR_KEY_WARNING.to_string());
    }
    let raw_for_validation = state::read_raw_config_for_validation(root);
    let agent_drift = timed!(
        profile,
        "validate_agent_files_drift",
        state::validate_agent_files_drift(root, &raw_for_validation)
    );
    for problem in state::validate_models_config(&raw_for_validation) {
        let scope = match (&problem.runtime, &problem.slot) {
            (Some(rt), Some(slot)) => format!(" models.{rt}.{slot}:"),
            _ => String::new(),
        };
        staleness.push(format!("config validate [{}]{scope} {}", problem.code, problem.message));
    }
    for problem in agent_drift {
        staleness.push(format!(
            "config validate [{}] {} ({}): {}",
            problem.code, problem.agent, problem.slot, problem.message
        ));
    }
    if !state::is_known_phase(&state_rec.phase) {
        staleness.push(format!(
            "Unknown phase \"{}\" — not in the enum ({}; terminal alias: compounding-complete). Set state.phase to a valid value (idle at feature close); invented phases break machine-checkable handoffs (decision 0004).",
            state_rec.phase,
            state::PHASES.join(", ")
        ));
    }

    let review = timed!(profile, "build_review_block (gix)", reviews::build_review_block(root));
    let recovery_block = timed!(
        profile,
        "build_recovery_block (transcripts)",
        recovery::build_recovery_block(&shared, &ctx.control_root, &ctx.projects_root, root, ctx.now_ms, None)
    );

    let execution_approved = state_rec.approved_gates.execution;
    let ready = timed!(profile, "ready_cells", cells::ready_cells_from(&shared, state_rec.feature.as_deref()));
    let review_unreviewed = review.get("candidates").and_then(|c| c.get("unreviewed")).and_then(Value::as_i64).unwrap_or(0);

    let recommended: String = if !js_truthy(&onboarding_raw) {
        "Onboarding missing — run bee-hive onboarding.".to_string()
    } else if js_truthy(&handoff) {
        "HANDOFF present — present it to the user and WAIT. Never auto-resume.".to_string()
    } else if state_rec.phase == "swarming" && !execution_approved {
        "NOT ready to swarm: gate \"execution\" is not approved.".to_string()
    } else if execution_approved && !ready.is_empty() {
        format!(
            "{} ready cell(s): {} — orchestrator assigns them.",
            ready.len(),
            ready.iter().map(|c| c.id.clone()).collect::<Vec<_>>().join(", ")
        )
    } else if POST_EXECUTION_REVIEW_PHASES.contains(&state_rec.phase.as_str()) && review_unreviewed > 0 {
        format!("{review_unreviewed} review candidate(s) awaiting: full review is user-invoked only, never dispatched automatically.")
    } else if !state_rec.next_action.is_empty() {
        state_rec.next_action.clone()
    } else {
        "Invoke bee-hive.".to_string()
    };

    let runtime_drift = timed!(profile, "compute_runtime_drift (sha256)", state::compute_runtime_drift(root, &onboarding_raw));
    let repo_hive = state::find_repo_hive(root);
    let source_id = match &repo_hive {
        Some(hive) => source_identity::classify_source(Some(hive.as_path()), ctx.home_dir.as_deref()),
        None => json!({"kind": "unknown", "root": Value::Null}),
    };
    let worktree_notice = state::ungranted_worktree_notice(&ctx.cwd_roots);
    let contention = timed!(profile, "build_contention_summary", recovery::build_contention_summary(root));

    let bypass = timed!(profile, "bypass_level", state::bypass_level(root));

    // ── the payload, in bee.mjs's literal key order ──
    let mut out = Map::new();

    let mut onboarding = Map::new();
    onboarding.insert("installed".to_string(), Value::Bool(js_truthy(&onboarding_raw)));
    onboarding.insert(
        "bee_version".to_string(),
        onboarding_raw.get("bee_version").cloned().unwrap_or(Value::Null),
    );
    onboarding.insert("plugin_version".to_string(), json!(state::BEE_VERSION));
    onboarding.insert("drift".to_string(), runtime_drift.get("drift").cloned().unwrap_or(Value::Bool(false)));
    let drift_detail = runtime_drift.get("detail").and_then(Value::as_array).cloned().unwrap_or_default();
    if !drift_detail.is_empty() {
        onboarding.insert("drift_detail".to_string(), Value::Array(drift_detail));
    }
    out.insert("onboarding".to_string(), Value::Object(onboarding));

    let mut source = Map::new();
    source.insert("kind".to_string(), source_id.get("kind").cloned().unwrap_or(Value::Null));
    source.insert("root".to_string(), source_id.get("root").cloned().unwrap_or(Value::Null));
    out.insert("source".to_string(), Value::Object(source));

    out.insert("phase".to_string(), json!(state_rec.phase));
    out.insert("mode".to_string(), opt_string(&state_rec.mode));
    out.insert("feature".to_string(), opt_string(&state_rec.feature));
    out.insert(
        "gates".to_string(),
        serde_json::to_value(&state_rec.approved_gates).unwrap_or(Value::Null),
    );
    out.insert("gate_bypass".to_string(), Value::Bool(bypass != "off"));
    out.insert("gate_bypass_level".to_string(), json!(bypass));
    out.insert("models".to_string(), models);
    // rust-port-23: `ceiling_scarcity_warning` used to recompute `tier_mix`
    // from its own fresh cells scan, over the same `state.feature` this
    // line already resolved — it now consumes the mix rendered here.
    let mix = timed!(profile, "tier_mix", cells::tier_mix_from(&shared, state_rec.feature.as_deref()));
    let ceiling_scarcity =
        timed!(profile, "ceiling_scarcity_warning", cells::ceiling_scarcity_warning_from(&mix).unwrap_or(Value::Null));
    // Key ORDER is part of the contract (`JSON.stringify` emits insertion
    // order): tier_mix before ceiling_scarcity, exactly as bee.mjs writes
    // them — the values are computed above in dependency order, inserted
    // here in source order.
    out.insert("tier_mix".to_string(), mix);
    out.insert("ceiling_scarcity".to_string(), ceiling_scarcity);
    out.insert("handoff".to_string(), handoff.clone());

    let mut cells_block = Map::new();
    cells_block.insert("open".to_string(), json!(open));
    cells_block.insert("claimed".to_string(), json!(claimed));
    cells_block.insert("capped".to_string(), json!(capped));
    cells_block.insert("blocked".to_string(), json!(blocked));
    cells_block.insert("archived".to_string(), archived.clone());
    out.insert("cells".to_string(), Value::Object(cells_block));

    let lanes = timed!(
        profile,
        "lanes",
        if opts.lanes_full {
            Value::Array(state::build_lane_rows(root))
        } else {
            state::build_lane_summary(root, &ctx.control_root, ctx.session_id.as_deref())
        }
    );
    out.insert("lanes".to_string(), lanes);

    out.insert("review".to_string(), review.clone());
    out.insert("recovery".to_string(), recovery_block);

    let mut scribing =
        timed!(profile, "scribing_debt", cells::scribing_debt_from(&shared, state_rec.feature.as_deref(), None));
    if let Value::Object(ref mut map) = scribing {
        map.insert(
            "orphaned".to_string(),
            timed!(profile, "global_scribing_debt", cells::global_scribing_debt_from(&shared)),
        );
    }
    out.insert("scribing_debt".to_string(), scribing);

    let queue = timed!(profile, "capture_queue", capture::capture_queue(root));
    let mut capture_block = Map::new();
    capture_block.insert("count".to_string(), queue.get("count").cloned().unwrap_or(json!(0)));
    capture_block.insert(
        "ids".to_string(),
        Value::Array(
            queue
                .get("stubs")
                .and_then(Value::as_array)
                .map(|stubs| stubs.iter().map(|s| s.get("id").cloned().unwrap_or(Value::Null)).collect())
                .unwrap_or_default(),
        ),
    );
    out.insert("capture_queue".to_string(), Value::Object(capture_block));

    out.insert(
        "pbi".to_string(),
        match &backlog_counts {
            Some(b) => {
                let mut pbi = Map::new();
                pbi.insert("proposed".to_string(), b.get("proposed").cloned().unwrap_or(Value::Null));
                pbi.insert("in_flight".to_string(), b.get("inFlight").cloned().unwrap_or(Value::Null));
                pbi.insert("done".to_string(), b.get("done").cloned().unwrap_or(Value::Null));
                Value::Object(pbi)
            }
            None => Value::Null,
        },
    );

    out.insert("commands".to_string(), commands.clone());
    out.insert(
        "active_reservations".to_string(),
        serde_json::to_value(&active_reservations).unwrap_or(Value::Array(Vec::new())),
    );
    out.insert(
        "workers".to_string(),
        Value::Array(timed!(profile, "active_workers", active_workers(&ctx.control_root, ctx.now_ms))),
    );
    out.insert(
        "critical_patterns_present".to_string(),
        Value::Bool(root.join("docs").join("history").join("learnings").join("critical-patterns.md").exists()),
    );

    // rust-port-23: `activeDecisions(root, {recent: 3})` is the full active
    // list truncated to its first 3 entries — the shared list, taken from
    // the front, is that same value without a second journal parse.
    let recent_events = timed!(profile, "active_decisions", shared.decisions());
    let recent: Vec<Value> = recent_events
        .iter()
        .take(3)
        .map(|event| {
            let mut row = Map::new();
            // `{id: event.id}` with `event.id === undefined` is dropped by
            // JSON.stringify; an explicit null survives.
            if let Some(id) = event.get("id") {
                row.insert("id".to_string(), id.clone());
            }
            if let Some(date) = event.get("date") {
                row.insert("date".to_string(), date.clone());
            }
            row.insert("decision".to_string(), Value::String(datamark(&js_to_string(event.get("decision")))));
            Value::Object(row)
        })
        .collect();
    out.insert("recent_decisions".to_string(), Value::Array(recent));

    out.insert(
        "staleness_warnings".to_string(),
        Value::Array(staleness.iter().map(|s| Value::String(s.clone())).collect()),
    );
    out.insert("recommended_next".to_string(), Value::String(recommended));

    if let Some(notice) = &worktree_notice {
        out.insert("worktree_notice".to_string(), Value::String(notice.clone()));
    }
    if let Some(c) = contention {
        out.insert("contention".to_string(), c);
    }

    Value::Object(out)
}

fn opt_string(v: &Option<String>) -> Value {
    match v {
        Some(s) => Value::String(s.clone()),
        None => Value::Null,
    }
}

/// JS truthiness for a parsed JSON value (`null`/`false`/`0`/`""` falsy).
fn js_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0 && !f.is_nan()).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

/// `String(value ?? '')` as `datamark` performs it: `undefined`/`null`
/// both collapse to the empty string via `??`.
fn js_to_string(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(other) => other.to_string(),
    }
}

// ─── renderStatusText ────────────────────────────────────────────────────

/// Port of `bee.mjs:894` `formatSlot(value)`.
fn format_slot(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(v) => {
            if v.get("kind").and_then(Value::as_str) == Some("cli") {
                let command = js_to_string_loose(v.get("command"));
                let first = command.split_whitespace().next().unwrap_or("").to_string();
                return format!("cli({first})");
            }
            match v.get("model") {
                Some(model) if js_truthy(model) => {
                    let model_s = js_to_string_loose(Some(model));
                    match v.get("effort") {
                        Some(effort) if js_truthy(effort) => {
                            format!("{model_s}@{}", js_to_string_loose(Some(effort)))
                        }
                        _ => model_s,
                    }
                }
                _ => "null".to_string(),
            }
        }
    }
}

/// JS `String(x)` template coercion for the shapes `formatSlot` meets.
fn js_to_string_loose(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

/// Port of `bee.mjs:905` `formatLaneRow(l)`.
fn format_lane_row(lane: &Value) -> String {
    let gates = state::GATE_NAMES
        .iter()
        .map(|g| {
            let approved = lane.get("approved_gates").and_then(|ag| ag.get(*g)).map(js_truthy).unwrap_or(false);
            format!("{g}={}", if approved { "approved" } else { "pending" })
        })
        .collect::<Vec<_>>()
        .join(" ");
    let bound: Vec<String> = lane
        .get("bound_sessions")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|v| js_to_string_loose(Some(v))).collect())
        .unwrap_or_default();
    let bound_str = if bound.is_empty() { String::new() } else { format!(" sessions={}", bound.join(",")) };
    format!(
        "{} [{}] {gates}{bound_str}",
        js_to_string_loose(lane.get("feature")),
        js_to_string_loose(lane.get("phase"))
    )
}

/// Port of `bee.mjs:915` `formatLaneSummaryLine(summary)`.
fn format_lane_summary_line(summary: &Value) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(active) = summary.get("active").filter(|a| js_truthy(a)) {
        parts.push(format!("active: {}", format_lane_row(active)));
    }
    let ids: Vec<String> = summary
        .get("ids")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|v| js_to_string_loose(Some(v))).collect())
        .unwrap_or_default();
    if !ids.is_empty() {
        let counts_str = summary
            .get("counts")
            .and_then(Value::as_object)
            .map(|m| m.iter().map(|(phase, n)| format!("{phase}={}", js_to_string_loose(Some(n)))).collect::<Vec<_>>().join(" "))
            .unwrap_or_default();
        parts.push(format!("{} other lane(s) [{counts_str}] (ids: {})", ids.len(), ids.join(", ")));
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("Lanes: {}", parts.join(" | ")))
    }
}

/// Port of `bee.mjs:927` `renderStatusText(status)`.
pub fn render_status_text(status: &Value) -> String {
    let mut lines: Vec<String> = Vec::new();

    if let Some(notice) = status.get("worktree_notice").filter(|v| js_truthy(v)) {
        lines.push(js_to_string_loose(Some(notice)));
    }

    lines.push(format!("bee status (plugin v{})", state::BEE_VERSION));

    let onboarding = status.get("onboarding").cloned().unwrap_or(Value::Null);
    let installed = onboarding.get("installed").map(js_truthy).unwrap_or(false);
    let drift = onboarding.get("drift").map(js_truthy).unwrap_or(false);
    let drift_suffix = if drift {
        match onboarding.get("drift_detail").and_then(Value::as_array) {
            Some(detail) => format!(" [drift: {} file(s)]", detail.len()),
            None => " [drift]".to_string(),
        }
    } else {
        String::new()
    };
    let onboarding_body = if installed {
        format!("installed (bee {})", js_to_string_loose(onboarding.get("bee_version")))
    } else {
        "MISSING".to_string()
    };
    lines.push(format!("Onboarding: {onboarding_body}{drift_suffix}"));

    lines.push(format!(
        "Phase: {} | Mode: {} | Feature: {}",
        js_to_string_loose(status.get("phase")),
        nullish_or(status.get("mode"), "none"),
        nullish_or(status.get("feature"), "none")
    ));

    let gates = status.get("gates").cloned().unwrap_or(Value::Null);
    lines.push(format!(
        "Gates: {}",
        state::GATE_NAMES
            .iter()
            .map(|g| format!("{g}={}", if gates.get(*g).map(js_truthy).unwrap_or(false) { "approved" } else { "pending" }))
            .collect::<Vec<_>>()
            .join(" ")
    ));

    if let Some(level) = status.get("gate_bypass_level").and_then(Value::as_str) {
        if !level.is_empty() && level != "off" {
            lines.push(bypass_banner(level).to_string());
        }
    }

    lines.push(format!(
        "Handoff: {}",
        if status.get("handoff").map(js_truthy).unwrap_or(false) {
            "PRESENT — surface it and WAIT"
        } else {
            "none"
        }
    ));

    let cells_block = status.get("cells").cloned().unwrap_or(Value::Null);
    let capped = cells_block.get("capped").and_then(Value::as_i64).unwrap_or(0);
    let archived_capped = cells_block.get("archived").and_then(|a| a.get("capped")).and_then(Value::as_i64).unwrap_or(0);
    lines.push(format!(
        "Cells: open={} claimed={} capped={} blocked={} archived={} (total capped={})",
        js_to_string_loose(cells_block.get("open")),
        js_to_string_loose(cells_block.get("claimed")),
        js_to_string_loose(cells_block.get("capped")),
        js_to_string_loose(cells_block.get("blocked")),
        js_to_string_loose(cells_block.get("archived").and_then(|a| a.get("total"))),
        capped + archived_capped
    ));

    match status.get("lanes") {
        Some(Value::Array(rows)) => {
            if !rows.is_empty() {
                lines.push(format!(
                    "Lanes: {}",
                    rows.iter().map(format_lane_row).collect::<Vec<_>>().join(" | ")
                ));
            }
        }
        Some(summary) => {
            if let Some(line) = format_lane_summary_line(summary) {
                lines.push(line);
            }
        }
        None => {}
    }

    let phase = status.get("phase").and_then(Value::as_str).unwrap_or("");
    let unreviewed = status
        .get("review")
        .and_then(|r| r.get("candidates"))
        .and_then(|c| c.get("unreviewed"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    if POST_EXECUTION_REVIEW_PHASES.contains(&phase) && unreviewed > 0 {
        lines.push(format!(
            "Completed and verified; independent review not requested; {unreviewed} candidate(s) awaiting review."
        ));
    }

    if let Some(debt) = status.get("scribing_debt").filter(|v| js_truthy(v)) {
        let count = debt.get("count").and_then(Value::as_i64).unwrap_or(0);
        if count > 0 {
            let cells_str: Vec<String> = debt
                .get("cells")
                .and_then(Value::as_array)
                .map(|a| a.iter().map(|v| js_to_string_loose(Some(v))).collect())
                .unwrap_or_default();
            lines.push(format!(
                "Scribing debt: {count} behavior_change cell(s) uncaptured ({}) — run bee-scribing capture (decision 0011)",
                cells_str.join(", ")
            ));
        }
    }

    if let Some(queue) = status.get("capture_queue").filter(|v| js_truthy(v)) {
        let count = queue.get("count").and_then(Value::as_i64).unwrap_or(0);
        if count > 0 {
            lines.push(format!(
                "Capture queue: {count} stub(s) pending flush — run bee-scribing flush at wrap-up, before compact/clear, or now if idle (decision 0017)"
            ));
        }
    }

    if let Some(pbi) = status.get("pbi").filter(|v| js_truthy(v)) {
        lines.push(format!(
            "PBI: {} done / {} in-flight / {} proposed",
            js_to_string_loose(pbi.get("done")),
            js_to_string_loose(pbi.get("in_flight")),
            js_to_string_loose(pbi.get("proposed"))
        ));
    }

    let commands = status.get("commands").cloned().unwrap_or(Value::Null);
    let recorded: Vec<String> = COMMAND_KEYS
        .iter()
        .filter(|key| commands.get(**key).map(js_truthy).unwrap_or(false))
        .map(|key| format!("{key}={}", js_to_string_loose(commands.get(*key))))
        .collect();
    lines.push(format!(
        "Standard commands: {}",
        if recorded.is_empty() { "none recorded".to_string() } else { recorded.join(" | ") }
    ));

    lines.push(format!(
        "Active reservations: {}",
        status.get("active_reservations").and_then(Value::as_array).map(Vec::len).unwrap_or(0)
    ));
    lines.push(format!(
        "Active workers: {}",
        status.get("workers").and_then(Value::as_array).map(Vec::len).unwrap_or(0)
    ));

    if let Some(contention) = status.get("contention").filter(|v| js_truthy(v)) {
        let top: Vec<String> = contention
            .get("top_locks")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|l| {
                        format!(
                            "{}×{}",
                            js_to_string_loose(l.get("lock_name")),
                            js_to_string_loose(l.get("busy_count"))
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let worst_lock = contention.get("worst_wait_lock");
        let worst_suffix = match worst_lock {
            Some(v) if js_truthy(v) => format!(" on \"{}\"", js_to_string_loose(Some(v))),
            _ => String::new(),
        };
        lines.push(format!(
            "Contention: {} LOCK_BUSY event(s) recently (top: {}); worst wait {}ms{worst_suffix}",
            js_to_string_loose(contention.get("busy_count")),
            top.join(", "),
            js_to_string_loose(contention.get("worst_wait_ms"))
        ));
    }

    lines.push(format!(
        "Critical patterns file: {}",
        if status.get("critical_patterns_present").map(js_truthy).unwrap_or(false) { "present" } else { "absent" }
    ));

    if let Some(models) = status.get("models").filter(|v| js_truthy(v)) {
        let claude = models.get("claude");
        lines.push(format!(
            "Models (claude): generation={} extraction={} review={} · ceiling = the session model (keep it scarce; decisions 0012/0015/0021)",
            format_slot(claude.and_then(|c| c.get("generation"))),
            format_slot(claude.and_then(|c| c.get("extraction"))),
            format_slot(claude.and_then(|c| c.get("review")))
        ));
    }

    if let Some(mix) = status.get("tier_mix").filter(|v| js_truthy(v)) {
        if mix.get("tiered").and_then(Value::as_i64).unwrap_or(0) > 0 {
            let counts = mix.get("counts");
            let share = mix.get("ceilingShare").and_then(Value::as_f64).unwrap_or(0.0);
            lines.push(format!(
                "Tier mix: extraction={} generation={} ceiling={} untiered={} (ceiling {}%)",
                js_to_string_loose(counts.and_then(|c| c.get("extraction"))),
                js_to_string_loose(counts.and_then(|c| c.get("generation"))),
                js_to_string_loose(counts.and_then(|c| c.get("ceiling"))),
                js_to_string_loose(counts.and_then(|c| c.get("untiered"))),
                js_math_round(share * 100.0)
            ));
        }
    }

    if let Some(scarcity) = status.get("ceiling_scarcity").filter(|v| js_truthy(v)) {
        lines.push(format!(
            "⚠ Ceiling scarcity: {}/{} tiered cells on ceiling ({}%) — re-tier routine cells (decision 0012)",
            js_to_string_loose(scarcity.get("ceiling")),
            js_to_string_loose(scarcity.get("tiered")),
            js_to_string_loose(scarcity.get("pct"))
        ));
    }

    let high_risk = status.get("review").and_then(|r| r.get("high_risk_unreviewed")).and_then(Value::as_i64).unwrap_or(0);
    if high_risk > 0 {
        lines.push(format!(
            "⚠ High-risk unreviewed: {high_risk} high-risk candidate(s) have not passed independent review — bee will not auto-dispatch reviewers; request review before merge/release."
        ));
    }

    if let Some(recent) = status.get("recent_decisions").and_then(Value::as_array) {
        if !recent.is_empty() {
            lines.push("Recent decisions:".to_string());
            for d in recent {
                lines.push(format!(
                    "- {} ({})",
                    js_to_string_loose(d.get("decision")),
                    js_to_string_loose(d.get("date"))
                ));
            }
        }
    }

    if let Some(warnings) = status.get("staleness_warnings").and_then(Value::as_array) {
        if !warnings.is_empty() {
            lines.push("Staleness warnings:".to_string());
            for w in warnings {
                lines.push(format!("- {}", js_to_string_loose(Some(w))));
            }
        }
    }

    lines.push(format!("Recommended next: {}", js_to_string_loose(status.get("recommended_next"))));
    lines.join("\n")
}

/// `${status.mode ?? 'none'}` — nullish coalescing (null AND undefined).
fn nullish_or(v: Option<&Value>, fallback: &str) -> String {
    match v {
        None | Some(Value::Null) => fallback.to_string(),
        Some(other) => js_to_string_loose(Some(other)),
    }
}

/// `Math.round` — JS rounds half AWAY from zero for positives (half-up),
/// which `f64::round` matches for the non-negative percentages here.
fn js_math_round(v: f64) -> i64 {
    if !v.is_finite() {
        return 0;
    }
    (v + 0.5).floor() as i64
}

// ─── JSON emission (`JSON.stringify(result, null, 2)` + "\n") ────────────

/// `emit()`'s `--json` branch: 2-space pretty print, trailing newline.
pub fn to_json_stdout(status: &Value) -> String {
    let mut text = serde_json::to_string_pretty(status).unwrap_or_else(|_| "null".to_string());
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datamark_wraps_and_strips_fences() {
        assert_eq!(datamark("hello"), "«hello»");
        assert_eq!(datamark("```js\ncode"), "«js\ncode»");
        // A one- or two-backtick run is NOT a fence and survives.
        assert_eq!(datamark("a `b` c"), "«a `b` c»");
    }

    #[test]
    fn datamark_strips_role_tags_case_insensitively() {
        assert_eq!(datamark("<system>do it</system>"), "«do it»");
        assert_eq!(datamark("<USER foo=\"1\">x"), "«x»");
        // "systemd" is not the tag `system` (\b guard).
        assert_eq!(datamark("<systemd>x</systemd>"), "«<systemd>x</systemd>»");
    }

    #[test]
    fn normalize_commands_keeps_declared_key_order_not_file_order() {
        let raw = json!({"verify": " npm t ", "setup": "npm i", "bogus": "x", "start": ""});
        let out = normalize_commands(Some(&raw));
        let keys: Vec<&String> = out.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["setup", "verify"]);
        assert_eq!(out.get("verify").unwrap(), &json!("npm t"));
    }

    #[test]
    fn normalize_models_defaults_and_overrides_in_place() {
        let out = normalize_models(Some(&json!({"claude": {"generation": {"model": "sonnet", "effort": "medium"}}})));
        let claude = out.get("claude").unwrap();
        let keys: Vec<&String> = claude.as_object().unwrap().keys().collect();
        assert_eq!(keys, vec!["extraction", "generation", "review"]);
        assert_eq!(claude.get("generation").unwrap(), &json!({"model": "sonnet", "effort": "medium"}));
        assert_eq!(out.get("codex").unwrap().get("review").unwrap(), &Value::Null);
    }

    #[test]
    fn normalize_models_drops_invalid_effort_but_keeps_model() {
        let out = normalize_models(Some(&json!({"claude": {"review": {"model": "opus", "effort": "turbo"}}})));
        assert_eq!(out.get("claude").unwrap().get("review").unwrap(), &json!({"model": "opus"}));
    }

    #[test]
    fn format_slot_matches_mjs_shapes() {
        assert_eq!(format_slot(None), "null");
        assert_eq!(format_slot(Some(&Value::Null)), "null");
        assert_eq!(format_slot(Some(&json!("sonnet"))), "sonnet");
        assert_eq!(format_slot(Some(&json!({"kind": "cli", "command": "codex exec -m x"}))), "cli(codex)");
        assert_eq!(format_slot(Some(&json!({"model": "opus"}))), "opus");
        assert_eq!(format_slot(Some(&json!({"model": "opus", "effort": "high"}))), "opus@high");
    }

    #[test]
    fn js_math_round_rounds_half_up() {
        assert_eq!(js_math_round(29.4), 29);
        assert_eq!(js_math_round(29.5), 30);
        assert_eq!(js_math_round(0.0), 0);
    }
}
