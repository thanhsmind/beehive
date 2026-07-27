//! guards — port of the CORE write-check spine of `.bee/bin/lib/guards.mjs`'s
//! `checkWrite` (rust-port-9, CONTEXT.md D2/D7): direct-edit / docs-history /
//! scratch-shape first-hit denies, lane-record resolution, cross-session
//! holds, cross-worktree holds (exclusive-path deny vs advisory warning),
//! workspace ownership, and the terminal/gated/swarming phase branches.
//!
//! `.bee/bin/lib/guards.mjs` is FROZEN for the duration of the rust-port
//! feature (D1) — this module is conformance-checked against the real hook
//! (`crates/queen-bee/tests/writeguard_core.rs` drives sha256-verified copies
//! of `bee-write-guard.mjs` as the node oracle), never edited to "improve"
//! on it. Every deny reason string below is copied VERBATIM from the mjs
//! source so a deny's stderr is byte-identical across runtimes.
//!
//! Scope boundary: rust-port-9 ported the checkWrite spine; rust-port-11
//! adds the Bash-command analysis from the same mjs source
//! (`extractBashTargets`, `checkGitBashCommand`,
//! `checkBinLibImportBashCommand` — the CLI-shape scan itself lives in the
//! hook, mirroring bee-write-guard.mjs's own layout). The read side
//! (`checkRead`, read-size, privacy/scout), `apply_patch`, and
//! `AskUserQuestion` are rust-port-12.
//!
//! GIT SPAWN PARITY (validation decision 2026-07-26): rust spawns `git`
//! exactly where node does — only inside [`resolve_git_mutation_paths`]'s
//! `commit` branch, which only runs at a terminal phase, with the idle gate
//! on, for a git-mutation-shaped command. The common hot path stays
//! spawn-free (gix revisit noted for the flip slice — D5 tension named in
//! the validation report, not hidden).
//!
//! One deliberate API shape difference from the mjs source, noted so it is
//! never "fixed" back by accident: [`check_write`] takes an
//! already-resolved [`WriteTopology`] instead of re-running
//! `resolveContext(root)` itself — the queen-bee hook derives it from the
//! adapter's own `resolve_roots(store_root)` walk (the same classification
//! `resolveContext` is built on), matching `resolveWriteTopology`'s
//! "caller-supplied controlRoot override wins" contract (msn-21).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use regex_lite::Regex;
use serde_json::Value;

use crate::claims::{heartbeat_stale, read_session, DEFAULT_HEARTBEAT_STALE_SECONDS};
use crate::config::read_config_value;
use crate::holds::{find_foreign_holds, holds_store_corrupt, normalize_path, paths_overlap};
use crate::jsdate::parse_iso_ms;
use crate::lock::iso8601_millis;
use crate::reservations::{leases_root, reservations_path, LeaseRecord};
use crate::state::{resolve_pipeline, State};
use crate::tokenize::tokenize_command;
use crate::workspace::workspace_path;

/// Paths writable in gated phases even before execution approval
/// (guards.mjs `GATE_ALLOWED_PREFIXES`).
pub const GATE_ALLOWED_PREFIXES: [&str; 4] = [".bee/", "docs/", "plans/", "AGENTS.md"];

const GATED_PHASES: [&str; 3] = ["exploring", "planning", "validating"];
const TERMINAL_PHASES: [&str; 2] = ["idle", "compounding-complete"];

/// guards.mjs `DIRECT_EDIT_DENY` — CLI-owned files whose direct hand-edit is
/// denied in every phase, first-hit, before any other checkWrite logic.
const DIRECT_EDIT_DENY: [(&str, &str); 6] = [
    (
        ".bee/state.json",
        "bee.mjs state set --owner <selected pre-mutation phase>, or the dedicated state gate/worker/scribing-run verb",
    ),
    (".bee/backlog.jsonl", "bee.mjs backlog add"),
    (
        "docs/backlog.md",
        "bee.mjs backlog pbi add / bee.mjs backlog pbi status / bee.mjs backlog pbi amend to change data, or bee.mjs backlog render --write to regenerate the view",
    ),
    (
        ".bee/runtime/cross-worktree-holds.json",
        "bee.mjs reservations reserve/release (holds are mirrored into the ledger automatically)",
    ),
    (".bee/runtime/worktree-grants.json", "bee.mjs worktree register / unregister"),
    (
        ".bee/companion-session.json",
        "bee worktree new --with-companion (started/ended automatically by the companion lifecycle)",
    ),
];

/// guards.mjs `HISTORY_CODE_EXTENSIONS`.
const HISTORY_CODE_EXTENSIONS: [&str; 22] = [
    ".sh", ".bash", ".zsh", ".fish", ".ps1", ".bat", ".cmd", ".mjs", ".cjs", ".js", ".jsx", ".ts", ".tsx", ".py",
    ".rb", ".go", ".rs", ".java", ".php", ".pl", ".lua", ".r",
];

const SCRATCH_HOME_PREFIXES: [&str; 4] = [".bee/tmp/", ".bee/spikes/", ".bee/logs/", ".bee/workers/"];
const DELIVERABLE_PREFIXES: [&str; 6] = [
    "docs/",
    ".bee/cells/",
    ".claude-plugin/skills/",
    ".codex-plugin/skills/",
    ".claude/skills/",
    ".agents/skills/",
];
const DELIVERABLE_EXACT: [&str; 1] = [".bee/decisions.jsonl"];

/// guards.mjs `DEFAULT_EXCLUSIVE_PATHS` (multisession-native-14 D4).
const DEFAULT_EXCLUSIVE_PATHS: [&str; 16] = [
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
    "**/generated/**",
];

// ─── verdicts ───────────────────────────────────────────────────────────────

/// The checkWrite verdict, mirroring the mjs `{allow, kind, reason}` /
/// `{allow: true, warning}` shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Allow { warning: Option<String> },
    Deny { kind: &'static str, reason: String },
}

impl Verdict {
    fn allow() -> Self {
        Verdict::Allow { warning: None }
    }
    fn allow_warn(warning: String) -> Self {
        Verdict::Allow { warning: Some(warning) }
    }
    fn deny(kind: &'static str, reason: String) -> Self {
        Verdict::Deny { kind, reason }
    }
}

/// The topology `checkWrite` would have resolved via
/// `resolveWriteTopology(root, controlRootOverride)` — supplied by the
/// caller (see module doc comment). `workspace_id`/`worktree_id` follow
/// `resolveContext`'s contract: an ordinary checkout is `("main", None)`;
/// a REGISTERED linked worktree reports its git-verified id for both; an
/// unregistered linked worktree reports `("main", Some(id))`. Everything
/// `None` mirrors resolveContext's all-null "give up" case (which
/// `resolveWriteTopology`'s try/catch also produces for an invalid linked
/// worktree marker — fail-open, never a deny).
#[derive(Debug, Clone)]
pub struct WriteTopology {
    pub workspace_root: Option<PathBuf>,
    pub control_root: PathBuf,
    pub workspace_id: Option<String>,
    pub worktree_id: Option<String>,
}

// ─── path normalization + allowlist ────────────────────────────────────────

/// guards.mjs `normalizeRel`: backslashes to `/`, then ONE anchored strip of
/// a leading `./` run (`/^\.\/+/` — a single regex replace, deliberately not
/// a loop: `"././x"` normalizes to `"./x"`, matching JS exactly).
pub fn normalize_rel(rel_path: &str) -> String {
    let s = rel_path.replace('\\', "/");
    if let Some(rest) = s.strip_prefix("./") {
        let trimmed = rest.trim_start_matches('/');
        return trimmed.to_string();
    }
    s
}

fn under_allowed_prefix(rel_path: &str) -> bool {
    let normalized = normalize_rel(rel_path);
    GATE_ALLOWED_PREFIXES.iter().any(|prefix| {
        if let Some(dir) = prefix.strip_suffix('/') {
            normalized == dir || normalized.starts_with(prefix)
        } else {
            normalized == *prefix
        }
    })
}

fn under_any_prefix(normalized: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| normalized == &prefix[..prefix.len() - 1] || normalized.starts_with(prefix))
}

fn docs_history_code_deny(normalized: &str) -> Option<String> {
    if !normalized.starts_with("docs/history/") {
        return None;
    }
    let dot = normalized.rfind('.')?;
    let ext = normalized[dot..].to_lowercase();
    if HISTORY_CODE_EXTENSIONS.contains(&ext.as_str()) {
        Some(ext)
    } else {
        None
    }
}

fn regex_test(pattern: &str, text: &str) -> bool {
    Regex::new(pattern).map(|re| re.is_match(text)).unwrap_or(false)
}

/// guards.mjs `scratchShapeDeny` — a short kind string when `normalized` is
/// a scratch-shaped write landing outside every allowed home/deliverable.
fn scratch_shape_deny(normalized: &str) -> Option<String> {
    if under_any_prefix(normalized, &SCRATCH_HOME_PREFIXES) {
        return None;
    }
    if DELIVERABLE_EXACT.contains(&normalized) {
        return None;
    }
    if under_any_prefix(normalized, &DELIVERABLE_PREFIXES) {
        return None;
    }

    let basename = normalized.rsplit('/').next().unwrap_or(normalized);
    if regex_test(r"(?i)^\.[^/]*(?:debug|stress|scratch)[^/]*$", basename) {
        return Some("a dotfile named like a debug/stress/scratch script".to_string());
    }
    if regex_test(r"(?i)^(?:verdict|probe|digest)-", basename) {
        return Some("a verdict-/probe-/digest- style scratch payload".to_string());
    }
    let test_fixture_dir =
        regex_test(r"(?i)(^|/)(test|tests|__tests__|fixtures|__fixtures__|testdata|examples)(/|$)", normalized);
    if regex_test(r"(?i)\.(tmp|log|bak)$", basename) && !test_fixture_dir {
        let dot = basename.rfind('.').unwrap_or(0);
        return Some(format!("a {} scratch file", &basename[dot..]));
    }
    None
}

// ─── intake refusal (guards.mjs intakeFixLine/intakeRefusal) ────────────────

fn intake_fix_line() -> String {
    format!(
        "FIX: commit or write bookkeeping directly — {} are exempt from this gate — \
or route the request through bee-hive first (classify the mode; tiny fixes stay tiny — one cell, a 2-minute \
reality check, Gate 3, go), then execute. Last resort, repo-level opt-out: \
bee config set --key guards.idle_gate --value false (re-enable with: bee config unset --key guards.idle_gate).",
        GATE_ALLOWED_PREFIXES.join(", ")
    )
}

/// guards.mjs `intakeRefusal(phase, blockedDescription, extraSentence = '')`
/// — `extra_sentence` carries its own trailing space when non-empty, exactly
/// like the mjs call sites.
fn intake_refusal(phase: &str, blocked_description: &str, extra_sentence: &str) -> String {
    format!(
        "bee intake gate: no bee work is active (phase: {phase}) — {blocked_description} is blocked. {extra_sentence}{}",
        intake_fix_line()
    )
}

// ─── reservation-store reads (reservations.mjs translation layer) ──────────

/// The translated reservation view `leaseToReservation` produces:
/// `{agent, cell, path, ttl_seconds, reserved_at, session, kind}`.
#[derive(Debug, Clone)]
pub struct ReservationView {
    pub agent: String,
    pub cell: String,
    pub path: String,
    pub ttl_seconds: f64,
    pub reserved_at: Option<String>,
    pub session: Option<String>,
    pub kind: String,
}

const PATH_RESOURCE_PREFIX: &str = "path:";
const AGENT_WORKSPACE_PREFIX: &str = "agent:";
const SESSIONLESS_SESSION_ID: &str = "\u{0}bee-reservation-sessionless\u{0}";

/// JS template-literal rendering of a possibly-absent field: `undefined`
/// interpolates as the literal string "undefined" (leaseToReservation leaves
/// `agent`/`cell` undefined when the raw record omits them, and the deny
/// messages interpolate them directly).
fn js_string_or_undefined(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "undefined".to_string())
}

/// lease-store.mjs `isLeaseExpired` on the RAW record's own `expires_at`.
fn is_lease_record_expired(record: &LeaseRecord, now_ms: i64) -> bool {
    let expires = match &record.expires_at {
        Some(Value::String(s)) => s,
        _ => return false, // null / absent => "never expires"
    };
    match parse_iso_ms(expires) {
        Some(ms) => ms <= now_ms,
        None => false,
    }
}

/// reservations.mjs `leaseTtlSeconds`: 0 sentinel = never expires.
fn lease_ttl_seconds(record: &LeaseRecord) -> f64 {
    let expires = match &record.expires_at {
        Some(Value::String(s)) => s,
        _ => return 0.0,
    };
    let (Some(expires_ms), Some(acquired_ms)) = (
        parse_iso_ms(expires),
        record.acquired_at.as_deref().and_then(parse_iso_ms),
    ) else {
        // Date.parse(undefined/garbage) is NaN in JS; the arithmetic then
        // yields NaN and Math.max(0, Math.round(NaN)) is NaN — the deny
        // message's own Number.isFinite(ttl) check treats that as "no
        // expiry", so a non-finite marker is the faithful translation.
        return f64::NAN;
    };
    let ttl = ((expires_ms - acquired_ms) as f64 / 1000.0).round();
    ttl.max(0.0)
}

fn lease_to_reservation(record: &LeaseRecord) -> ReservationView {
    let workspace_id = record.workspace_id.clone();
    let agent = match &workspace_id {
        Some(w) if w.starts_with(AGENT_WORKSPACE_PREFIX) => Some(w[AGENT_WORKSPACE_PREFIX.len()..].to_string()),
        other => other.clone(),
    };
    let session = match &record.session_id {
        Some(s) if !s.is_empty() && s != SESSIONLESS_SESSION_ID => Some(s.clone()),
        _ => None,
    };
    ReservationView {
        agent: js_string_or_undefined(&agent),
        cell: js_string_or_undefined(&record.workflow_id),
        path: record.resource[PATH_RESOURCE_PREFIX.len()..].to_string(),
        ttl_seconds: lease_ttl_seconds(record),
        reserved_at: record.acquired_at.clone(),
        session,
        kind: record.kind.clone().unwrap_or_else(|| "lease".to_string()),
    }
}

/// reservations.mjs `listPathLeaseRecords` + `listReservations(activeOnly)`,
/// with the control root supplied by the caller (mjs re-derives it via
/// `controlRootFor(root)`; the hook's topology already carries it).
fn list_active_reservations(control_root: &Path, now_ms: i64) -> Vec<ReservationView> {
    let base = leases_root(control_root);
    let mut out = Vec::new();
    for sub in ["cells", "paths"] {
        let dir = base.join(sub);
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else { continue };
            let Ok(record) = serde_json::from_str::<LeaseRecord>(&text) else { continue };
            if !record.resource.starts_with(PATH_RESOURCE_PREFIX) {
                continue;
            }
            if is_lease_record_expired(&record, now_ms) {
                continue;
            }
            out.push(lease_to_reservation(&record));
        }
    }
    out
}

/// reservations.mjs `findConflicts` — active reservations held by OTHER
/// agents covering any of the given paths.
pub fn find_conflicts(control_root: &Path, agent: &str, paths: &[&str], now_ms: i64) -> Vec<ReservationView> {
    let requested: Vec<&str> = paths.iter().copied().filter(|p| !p.is_empty()).collect();
    if requested.is_empty() {
        return Vec::new();
    }
    list_active_reservations(control_root, now_ms)
        .into_iter()
        .filter(|r| r.agent != agent && requested.iter().any(|p| paths_overlap(&r.path, p)))
        .collect()
}

/// reservations.mjs `findSessionConflicts` — active reservations owned by a
/// DIFFERENT session; session-less rows never conflict.
pub fn find_session_conflicts(
    control_root: &Path,
    session_id: &str,
    paths: &[&str],
    now_ms: i64,
) -> Vec<ReservationView> {
    let requested: Vec<&str> = paths.iter().copied().filter(|p| !p.is_empty()).collect();
    if requested.is_empty() {
        return Vec::new();
    }
    let acting = session_id.trim();
    list_active_reservations(control_root, now_ms)
        .into_iter()
        .filter(|r| {
            r.session.as_deref().map(|s| !s.trim().is_empty() && s != acting).unwrap_or(false)
                && requested.iter().any(|p| paths_overlap(&r.path, p))
        })
        .collect()
}

/// reservations.mjs `isHardConflict` — hard unless kind 'intent' AND the
/// stored path is not the exact same resource as the target.
pub fn is_hard_conflict(reservation: &ReservationView, target_path: &str) -> bool {
    !(reservation.kind == "intent" && normalize_path(&reservation.path) != normalize_path(target_path))
}

/// guards.mjs `reservationStoreCorrupt`: missing store = open; a present but
/// unreadable/unparseable store fails closed.
pub fn reservation_store_corrupt(root: &Path) -> bool {
    let file = reservations_path(root);
    if !file.exists() {
        return false;
    }
    match fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<Value>(&text).is_err(),
        Err(_) => true, // readFileSync throw => caught => corrupt
    }
}

// ─── expiry display strings ─────────────────────────────────────────────────

/// guards.mjs `holdExpiry`.
fn hold_expiry(reservation: &ReservationView) -> String {
    let reserved_ms = reservation.reserved_at.as_deref().and_then(parse_iso_ms);
    let ttl = reservation.ttl_seconds;
    let (Some(reserved_ms), true) = (reserved_ms, ttl.is_finite() && ttl > 0.0) else {
        return "no expiry".to_string();
    };
    format!("expires {}", iso8601_millis(reserved_ms + (ttl * 1000.0) as i64))
}

/// guards.mjs `foreignHoldExpiry` — same convention rebased on the ledger
/// hold's `mirrored_at`/`ttl_seconds`.
fn foreign_hold_expiry(mirrored_at: Option<&str>, ttl_seconds: Option<&Value>) -> String {
    let mirrored_ms = mirrored_at.and_then(parse_iso_ms);
    let ttl = match ttl_seconds {
        Some(Value::Number(n)) => n.as_f64(),
        _ => None,
    };
    match (mirrored_ms, ttl) {
        (Some(ms), Some(t)) if t.is_finite() && t > 0.0 => {
            format!("expires {}", iso8601_millis(ms + (t * 1000.0) as i64))
        }
        _ => "no expiry".to_string(),
    }
}

// ─── exclusive-resource globs (multisession-native-14 D4) ───────────────────

/// guards.mjs `globToRegExp` — the same translation, verbatim: `**/` an
/// optional run of whole segments, `**` any remainder, `*` any run of
/// non-slash characters, everything else literal.
fn glob_to_regex(glob: &str) -> Option<Regex> {
    let normalized = {
        let s = glob.replace('\\', "/");
        if let Some(rest) = s.strip_prefix("./") {
            rest.trim_start_matches('/').to_string()
        } else {
            s
        }
    };
    let chars: Vec<char> = normalized.chars().collect();
    let mut pattern = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '*' && chars.get(i + 1) == Some(&'*') {
            let mut j = i + 2;
            if chars.get(j) == Some(&'/') {
                pattern.push_str("(?:.*/)?");
                j += 1;
            } else {
                pattern.push_str(".*");
            }
            i = j;
            continue;
        }
        if c == '*' {
            pattern.push_str("[^/]*");
            i += 1;
            continue;
        }
        if ".+^${}()|[]\\".contains(c) {
            pattern.push('\\');
            pattern.push(c);
            i += 1;
            continue;
        }
        pattern.push(c);
        i += 1;
    }
    Regex::new(&format!("^{pattern}$")).ok()
}

/// guards.mjs `isExclusivePath` — defaults EXTENDED by
/// `.bee/config.json`'s `guards.exclusive_paths` (never replaced).
fn is_exclusive_path(config: &Value, normalized_path: &str) -> bool {
    let extra: Vec<String> = config
        .get("guards")
        .and_then(|g| g.get("exclusive_paths"))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter(|s| !s.trim().is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    DEFAULT_EXCLUSIVE_PATHS
        .iter()
        .map(|s| s.to_string())
        .chain(extra)
        .any(|glob| glob_to_regex(&glob).map(|re| re.is_match(normalized_path)).unwrap_or(false))
}

// ─── hold topology + workspace ownership ────────────────────────────────────

struct HoldTopology<'a> {
    main_root: &'a Path,
    holder: String,
}

/// guards.mjs `resolveHoldTopology` — `None` means "skip the foreign-hold
/// consultation entirely" (fail-open, never a deny).
fn resolve_hold_topology<'a>(topology: &'a WriteTopology) -> Option<HoldTopology<'a>> {
    topology.workspace_root.as_ref()?;
    let Some(worktree_id) = &topology.worktree_id else {
        return Some(HoldTopology { main_root: &topology.control_root, holder: "main".to_string() });
    };
    match &topology.workspace_id {
        Some(ws) if ws == worktree_id => {
            Some(HoldTopology { main_root: &topology.control_root, holder: ws.clone() })
        }
        _ => None,
    }
}

/// state.mjs's (unexported) `resolveWritePolicyMode`, duplicated in
/// guards.mjs and kept byte-identical here.
fn resolve_write_policy_mode(config: &Value) -> &'static str {
    let configured = config
        .get("guards")
        .and_then(|g| g.get("write_policy"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    match configured {
        "observe" => "observe",
        "shared-disjoint" => "shared-disjoint",
        _ => "isolated",
    }
}

/// guards.mjs `sessionWorkspaceId` — a session's stamped `workspace_id`,
/// defaulting to `'main'` for legacy/missing/unreadable records.
fn session_workspace_id(control_root: &Path, session_id: &str) -> String {
    read_session(control_root, session_id)
        .and_then(|s| {
            s.extra
                .get("workspace_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "main".to_string())
}

enum OwnershipCheck {
    NotBlocked,
    Corrupt,
    Blocked { owner: String },
}

/// guards.mjs `checkWorkspaceOwnership` — read-only; missing record is open,
/// a present-but-unreadable record fails closed, a live foreign owner blocks.
/// The missing-vs-corrupt split mirrors workspace-store.mjs `readWorkspace`'s
/// `WORKSPACE_MISSING` vs `WORKSPACE_CORRUPT` refusals (ENOENT is missing;
/// any other read error, unparseable JSON, a non-object payload, or an `id`
/// field that does not match the requested workspace is corrupt).
fn check_workspace_ownership(control_root: &Path, workspace_id: &str, session_id: &str) -> OwnershipCheck {
    let file = workspace_path(control_root, workspace_id);
    let text = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return OwnershipCheck::NotBlocked,
        Err(_) => return OwnershipCheck::Corrupt,
    };
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return OwnershipCheck::Corrupt,
    };
    if !parsed.is_object() {
        return OwnershipCheck::Corrupt;
    }
    if parsed.get("id").and_then(Value::as_str) != Some(workspace_id) {
        return OwnershipCheck::Corrupt;
    }
    let owner = match parsed.get("write_owner_session").and_then(Value::as_str) {
        Some(o) if !o.is_empty() => o.to_string(),
        _ => return OwnershipCheck::NotBlocked, // no owner (or defaulted null)
    };
    if owner == session_id {
        return OwnershipCheck::NotBlocked;
    }
    let owner_session = read_session(control_root, &owner);
    let live = match &owner_session {
        Some(s) => !heartbeat_stale(Some(s), now_ms(), DEFAULT_HEARTBEAT_STALE_SECONDS),
        None => false,
    };
    if !live {
        return OwnershipCheck::NotBlocked;
    }
    OwnershipCheck::Blocked { owner }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ─── checkWrite ─────────────────────────────────────────────────────────────

/// Port of guards.mjs `checkWrite(root, state, relPath, agentName,
/// {sessionId, controlRoot})` — see the mjs source's long doc comment for
/// the full decision contract; the branch ORDER below is copied exactly
/// (first hit wins).
pub fn check_write(
    root: &Path,
    state: &State,
    rel_path: &str,
    agent_name: Option<&str>,
    session_id: Option<&str>,
    topology: &WriteTopology,
) -> Verdict {
    let normalized = normalize_rel(rel_path);

    if let Some((_, verb)) = DIRECT_EDIT_DENY.iter().find(|(p, _)| *p == normalized) {
        return Verdict::deny(
            "direct-edit",
            format!(
                "bee direct-edit guard: \"{normalized}\" is CLI-owned — direct edits are blocked in every phase. \
Hand-edited state files reintroduce schema drift (the exact class the CLI validates away). \
FIX: use {verb} instead of editing this file directly."
            ),
        );
    }

    if let Some(ext) = docs_history_code_deny(&normalized) {
        return Verdict::deny(
            "docs-history-code",
            format!(
                "bee docs-history guard: \"{normalized}\" writes a \"{ext}\" code file into docs/history/, which is \
the tech-agnostic KNOWLEDGE layer (.md only — CONTEXT.md, plan.md, reports, walkthrough). Code never lives there. \
FIX: put a persistent verify/helper script in the project's own scripts (committed with the product) and point \
the cell's verify command at it; put a disposable proof in .bee/spikes/<feature>/. Never docs/history."
            ),
        );
    }

    if let Some(scratch_kind) = scratch_shape_deny(&normalized) {
        return Verdict::deny(
            "scratch-shape",
            format!(
                "bee scratch-shape guard: \"{normalized}\" looks like {scratch_kind} landing in a tracked directory. \
Every ephemeral file bee writes for its own working purposes belongs in .bee/tmp/<feature-or-session>/ \
(feasibility code in .bee/spikes/<feature>/), never a tracked path (docs/specs/doctrine-layer.md). \
FIX: write it to .bee/tmp/ instead (or .bee/spikes/ for a feasibility proof), and let `bee tmp sweep` clear it later."
            ),
        );
    }

    let control_root = topology.control_root.as_path();

    // resolveWriteRecord: a bound sessionId reads through resolvePipeline's
    // lane record; an absent one uses the caller's own `state`.
    let session_trimmed = session_id.map(str::trim).filter(|s| !s.is_empty());
    let (record, record_source): (State, &'static str) = match session_trimmed {
        Some(sid) => match resolve_pipeline(control_root, Some(sid)) {
            Ok(resolved) => (resolved.record, resolved.source),
            Err(reason) => {
                return Verdict::deny("lane", format!("bee lane guard: {reason}"));
            }
        },
        None => (state.clone(), "default"),
    };

    // Cross-session hold deny (fsh-7, D3) — unconditional on phase.
    if let Some(acting) = session_trimmed {
        if reservation_store_corrupt(root) {
            return Verdict::deny(
                "holds-unreadable",
                "bee hold guard: the reservation store (.bee/reservations.json) is present but \
unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as empty. \
FIX: inspect/restore the reservation store, then retry."
                    .to_string(),
            );
        }
        let hold_conflicts = find_session_conflicts(control_root, acting, &[&normalized], now_ms());
        if !hold_conflicts.is_empty() {
            let acting_workspace_id = topology.workspace_id.clone().unwrap_or_else(|| "main".to_string());
            let same_workspace: Vec<&ReservationView> = hold_conflicts
                .iter()
                .filter(|holder| {
                    let holder_session = holder.session.as_deref().unwrap_or("");
                    session_workspace_id(control_root, holder_session) == acting_workspace_id
                })
                .collect();
            if let Some(holder) = same_workspace.first() {
                let session_display = js_string_or_undefined(&holder.session);
                return Verdict::deny(
                    "hold",
                    format!(
                        "bee cross-session hold: \"{normalized}\" is held by session \"{session_display}\" \
(agent {agent}, cell {cell}), {expiry}. \
Wait for the hold to expire or coordinate with that session — a cross-session hold is a hard block (D3).",
                        agent = holder.agent,
                        cell = holder.cell,
                        expiry = hold_expiry(holder),
                    ),
                );
            }
        }
    }

    // Cross-WORKTREE foreign-hold consultation (xwh-4 / multisession-native-14
    // D4) — unconditional on phase and on sessionId.
    if let Some(hold_topology) = resolve_hold_topology(topology) {
        if holds_store_corrupt(hold_topology.main_root) {
            return Verdict::deny(
                "worktree-holds-unreadable",
                "bee cross-worktree hold guard: the shared holds ledger (.bee/runtime/cross-worktree-holds.json \
in the main checkout) is present but unreadable/corrupt — failing closed rather than silently \
treating it as empty. FIX: inspect/restore the ledger in the main checkout, then retry."
                    .to_string(),
            );
        }
        let foreign = find_foreign_holds(hold_topology.main_root, &hold_topology.holder, &[&normalized]);
        if let Some(hold) = foreign.first() {
            let feature = hold.feature.clone().filter(|f| !f.is_empty()).unwrap_or_else(|| "unknown".to_string());
            let cell_suffix = match &hold.cell {
                Some(c) if !c.is_empty() => format!(", cell {c}"),
                _ => String::new(),
            };
            let expiry = foreign_hold_expiry(hold.mirrored_at.as_deref(), hold.ttl_seconds.as_ref());
            let config = read_config_value(root);
            if is_exclusive_path(&config, &normalized) {
                return Verdict::deny(
                    "worktree-hold",
                    format!(
                        "bee cross-worktree hold: \"{normalized}\" is held by checkout \"{holder}\" \
(feature {feature}{cell_suffix}), {expiry}. \
Wait for the hold to expire or coordinate with that checkout — a cross-worktree hold is a hard block.",
                        holder = hold.holder,
                    ),
                );
            }
            return Verdict::allow_warn(format!(
                "bee cross-worktree hold: \"{normalized}\" is also held by checkout \"{holder}\" \
(feature {feature}{cell_suffix}), {expiry} — \
advisory only (different workspace, not an exclusive resource). \
Coordinate with that checkout if possible; otherwise \"bee worktree merge\" will surface any real conflict \
between the two checkouts at merge time.",
                holder = hold.holder,
            ));
        }
    }

    let phase = if record.phase.is_empty() { "idle".to_string() } else { record.phase.clone() };

    // Workspace-ownership deny (msn-21, deny class (c)).
    if let Some(sid) = session_trimmed {
        if record_source == "default" && phase != "swarming" {
            let config = read_config_value(root);
            if resolve_write_policy_mode(&config) == "isolated" {
                let workspace_id = topology.workspace_id.clone().unwrap_or_else(|| "main".to_string());
                match check_workspace_ownership(control_root, &workspace_id, sid) {
                    OwnershipCheck::Corrupt => {
                        return Verdict::deny(
                            "workspace-unreadable",
                            format!(
                                "bee workspace-ownership guard: the workspace record for \"{workspace_id}\" \
(.bee/runtime/workspaces/{workspace_id}.json) is present but \
unreadable/corrupt — failing closed for a session-aware write rather than silently treating it as \
unowned. FIX: inspect/restore the workspace record, then retry."
                            ),
                        );
                    }
                    OwnershipCheck::Blocked { owner } => {
                        return Verdict::deny(
                            "workspace-ownership",
                            format!(
                                "bee write-policy: workspace \"{workspace_id}\" is write-owned by session \"{owner}\" \
— a second write-capable session defaults to isolation, never a shared write into the same checkout. \
FIX: coordinate with that session, wait for its heartbeat to go stale, or start your own feature with \
`bee.mjs state start-feature --isolate` (or set guards.auto_isolate to true in .bee/config.json) to work \
in a fresh worktree instead."
                            ),
                        );
                    }
                    OwnershipCheck::NotBlocked => {}
                }
            }
        }
    }

    if TERMINAL_PHASES.contains(&phase.as_str()) {
        let config = read_config_value(root);
        let idle_gate_on =
            !matches!(config.get("guards").and_then(|g| g.get("idle_gate")), Some(Value::Bool(false)));
        if idle_gate_on && !under_allowed_prefix(&normalized) {
            return Verdict::deny("intake", intake_refusal(&phase, &format!("writing \"{normalized}\""), ""));
        }
        return Verdict::allow();
    }

    if GATED_PHASES.contains(&phase.as_str()) {
        let execution_approved = record.approved_gates.execution;
        if !execution_approved && !under_allowed_prefix(&normalized) {
            return Verdict::deny(
                "gate",
                format!(
                    "bee gate: phase is \"{phase}\" and gate \"execution\" is not approved — \
writing \"{normalized}\" is blocked. Allowed now: {allowed}. \
Get execution approval (bee-hive) before touching source files.",
                    allowed = GATE_ALLOWED_PREFIXES.join(", "),
                ),
            );
        }
        return Verdict::allow();
    }

    if phase == "swarming" {
        let env_agent = env::var("BEE_AGENT_NAME").ok().filter(|v| !v.is_empty());
        let agent = agent_name.map(str::to_string).or(env_agent);
        if let Some(agent) = agent {
            let conflicts = find_conflicts(control_root, &agent, &[&normalized], now_ms());
            if !conflicts.is_empty() {
                let hard_conflicts: Vec<&ReservationView> =
                    conflicts.iter().filter(|c| is_hard_conflict(c, &normalized)).collect();
                if !hard_conflicts.is_empty() {
                    let held = hard_conflicts
                        .iter()
                        .map(|c| format!("{} holds \"{}\" (cell {})", c.agent, c.path, c.cell))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Verdict::deny(
                        "reservation",
                        format!(
                            "bee reservation conflict: \"{normalized}\" is reserved by another agent — {held}. \
Reserve the path first or return [BLOCKED] to the orchestrator."
                        ),
                    );
                }
                let warned = conflicts
                    .iter()
                    .map(|c| format!("{}'s declared intent \"{}\" (cell {}) covers \"{normalized}\"", c.agent, c.path, c.cell))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Verdict::allow_warn(format!(
                    "bee reservation intent: {warned} — advisory only (kind: intent), not a hard block."
                ));
            }
        }
        return Verdict::allow();
    }

    Verdict::allow()
}


// ─── bash write-target extraction (guards.mjs extractBashTargets) ──────────

const WRITE_COMMANDS: [&str; 6] = ["rm", "mv", "cp", "mkdir", "touch", "tee"];
const BROAD_TARGETS: [&str; 7] = [".", "..", "/", "~", "*", "./*", "/*"];

fn is_separator(token: &str) -> bool {
    matches!(token, "&&" | "||" | ";" | "|" | "&")
}

fn is_flag(token: &str) -> bool {
    token.starts_with('-')
}

fn token_basename(token: &str) -> String {
    token.replace('\\', "/").rsplit('/').next().unwrap_or("").to_string()
}

fn is_broad(target: &str) -> bool {
    let normalized = normalize_rel(target);
    BROAD_TARGETS.contains(&target)
        || BROAD_TARGETS.contains(&normalized.as_str())
        || normalized.ends_with("/*")
        || normalized.ends_with("/.")
        || normalized == "*"
}

/// guards.mjs `hasGitShortFlag`: a `/^-[a-zA-Z]+$/` token whose letter run
/// contains `letter` (case-sensitive — `-A` and `-a` are distinct).
fn has_git_short_flag(tokens: &[String], letter: char) -> bool {
    tokens.iter().any(|t| {
        let mut chars = t.chars();
        chars.next() == Some('-') && {
            let rest = &t[1..];
            !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphabetic()) && rest.contains(letter)
        }
    })
}

/// The mjs redirect regex `/^\d?>{1,2}(.*)$/`: an optional single digit, one
/// or two `>`, then the (possibly empty) inline target. `None` when the
/// token is not redirect-shaped at all.
fn parse_redirect(token: &str) -> Option<&str> {
    let bytes = token.as_bytes();
    let mut idx = 0;
    if bytes.first().map(|b| b.is_ascii_digit()).unwrap_or(false) {
        idx = 1;
    }
    let mut gt = 0;
    while gt < 2 && bytes.get(idx + gt) == Some(&b'>') {
        gt += 1;
    }
    if gt == 0 {
        return None;
    }
    Some(&token[idx + gt..])
}

/// The `{ paths, broadWrite }` result of [`extract_bash_targets`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BashTargets {
    pub paths: Vec<String>,
    pub broad_write: bool,
}

/// Port of guards.mjs `extractBashTargets(command)` — file targets a bash
/// command may write to (khuym patterns: `sed -i`, `tee`, `rm`, `mv`, `cp`,
/// `mkdir`, `touch`, `git add|mv|rm`, redirection `>`), plus the
/// blanket-staging broad-write flags (`git add -A/-u/--all/--update`,
/// `git commit -a/--all`) counted as broad writes (bsg-1).
pub fn extract_bash_targets(command: &str) -> BashTargets {
    let tokens = tokenize_command(command);
    let mut paths: Vec<String> = Vec::new();
    let mut broad_write = false;

    fn add_target(paths: &mut Vec<String>, broad_write: &mut bool, target: &str) {
        if target.is_empty() || target == "/dev/null" || target == "NUL" {
            return;
        }
        if is_broad(target) {
            *broad_write = true;
        }
        paths.push(target.to_string());
    }

    let mut i = 0usize;
    while i < tokens.len() {
        let token = &tokens[i];

        // Redirection: "> file", ">> file", ">file", "2> file".
        // NOT a file write: fd-duplication like `2>&1`, `1>&2`, `>&2` — the
        // target starts with `&` (a file descriptor, not a filename).
        if let Some(inline) = parse_redirect(token) {
            if !inline.is_empty() {
                if !inline.starts_with('&') {
                    add_target(&mut paths, &mut broad_write, inline);
                }
            } else if let Some(next) = tokens.get(i + 1) {
                if !is_separator(next) && !next.starts_with('&') {
                    add_target(&mut paths, &mut broad_write, next);
                    i += 1;
                }
            }
            i += 1;
            continue;
        }

        if is_separator(token) {
            i += 1;
            continue;
        }

        let cmd = token_basename(token);

        if cmd == "git" && matches!(tokens.get(i + 1).map(String::as_str), Some("add" | "mv" | "rm")) {
            // D8: `git add` only STAGES already-written content — staging a
            // CLI-owned file (DIRECT_EDIT_DENY) is not a direct-edit target.
            // `git mv`/`git rm` genuinely change what's on disk.
            let git_verb = tokens[i + 1].clone();
            let mut end = i + 2;
            while end < tokens.len() && !is_separator(&tokens[end]) {
                end += 1;
            }
            let segment = &tokens[i + 2..end];
            // `-A`/`--all`/`-u`/`--update` stage every changed path — the
            // reservation guard must see this as a broad write.
            if git_verb == "add"
                && (segment.iter().any(|t| t == "--all")
                    || segment.iter().any(|t| t == "--update")
                    || has_git_short_flag(segment, 'A')
                    || has_git_short_flag(segment, 'u'))
            {
                broad_write = true;
            }
            for t in segment {
                if !is_flag(t) {
                    let cli_owned_stage_only = git_verb == "add"
                        && DIRECT_EDIT_DENY.iter().any(|(p, _)| *p == normalize_rel(t));
                    if !cli_owned_stage_only {
                        add_target(&mut paths, &mut broad_write, t);
                    }
                }
            }
            i = end; // mjs: i = end - 1, then the for-loop i++
            continue;
        }

        if cmd == "git" && tokens.get(i + 1).map(String::as_str) == Some("commit") {
            // `-a`/`--all`/`-am` folds tracked-but-unstaged changes into the
            // commit — blanket staging the guard must see, same as `git add -A`.
            let mut end = i + 2;
            while end < tokens.len() && !is_separator(&tokens[end]) {
                end += 1;
            }
            let segment = &tokens[i + 2..end];
            if segment.iter().any(|t| t == "--all") || has_git_short_flag(segment, 'a') {
                broad_write = true;
            }
            i = end;
            continue;
        }

        if cmd == "sed" {
            let mut in_place = false;
            let mut last = i;
            let mut args: Vec<&String> = Vec::new();
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                if tokens[j].starts_with("-i") {
                    in_place = true;
                } else if !is_flag(&tokens[j]) {
                    args.push(&tokens[j]);
                }
                last = j;
                j += 1;
            }
            if in_place {
                // First non-flag arg is the script; the rest are files.
                for file in args.iter().skip(1) {
                    add_target(&mut paths, &mut broad_write, file);
                }
            }
            i = last + 1;
            continue;
        }

        if WRITE_COMMANDS.contains(&cmd.as_str()) {
            let mut saw_any = false;
            let mut last = i;
            let mut j = i + 1;
            while j < tokens.len() && !is_separator(&tokens[j]) {
                if !is_flag(&tokens[j]) {
                    add_target(&mut paths, &mut broad_write, &tokens[j]);
                    saw_any = true;
                }
                last = j;
                j += 1;
            }
            if cmd == "rm" && !saw_any {
                broad_write = true;
            }
            i = last + 1;
            continue;
        }

        i += 1;
    }

    BashTargets { paths, broad_write }
}

// ─── git write-exemption classification (D1/D3/D4, ige-2 / P46 / GH #1) ────

/// Read-only git subcommands, deliberately enumerated — never inferred.
const GIT_READONLY_SUBCOMMANDS: [&str; 12] = [
    "status", "log", "diff", "show", "rev-parse", "ls-files", "check-ignore", "merge-base", "rev-list",
    "describe", "blame", "cat-file",
];

/// Read-only ONLY with a specific flag (a bare `git branch <name>` /
/// `git tag <name>` MUTATES; `git remote` needs -v/--verbose).
fn git_readonly_flag_gated(subcommand: &str) -> Option<&'static [&'static str]> {
    match subcommand {
        "branch" => Some(&["--list"]),
        "tag" => Some(&["--list"]),
        "remote" => Some(&["-v", "--verbose"]),
        _ => None,
    }
}

/// Mutating git subcommands this exemption logic recognizes at all (D1).
/// `push` is deliberately NOT a member — it never gets the bookkeeping-path
/// exemption and is classified separately.
const GIT_MUTATING_SUBCOMMANDS: [&str; 15] = [
    "commit", "add", "rm", "mv", "checkout", "restore", "tag", "merge", "reset", "stash", "clean", "apply",
    "cherry-pick", "revert", "rebase",
];

/// Subset whose changed paths can be resolved from real git state (D4); the
/// rest always fail closed, never inferred safe.
const GIT_PATH_RESOLVABLE_SUBCOMMANDS: [&str; 6] = ["commit", "add", "rm", "mv", "checkout", "restore"];
const GIT_BROAD_PATHSPECS: [&str; 4] = [".", ":", ":/", "./"];

fn git_global_flag_takes_value(token: &str) -> bool {
    token == "-C" || token == "-c" || token == "--git-dir" || token == "--work-tree" || token == "--namespace"
}

/// guards.mjs `findGitInvocation` — the FIRST top-level `git <subcommand>`
/// invocation (skipping git's own global flags): `Some((subcommand, rest))`
/// with `subcommand: None` for a bare `git`, or `None` when `command`
/// contains no git invocation at all.
fn find_git_invocation(tokens: &[String]) -> Option<(Option<String>, Vec<String>)> {
    for i in 0..tokens.len() {
        if is_separator(&tokens[i]) {
            continue;
        }
        if token_basename(&tokens[i]) != "git" {
            continue;
        }
        let mut end = i + 1;
        while end < tokens.len() && !is_separator(&tokens[end]) {
            end += 1;
        }
        let invocation = &tokens[i + 1..end];
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
            return Some((Some(t.clone()), invocation[j + 1..].to_vec()));
        }
        return Some((None, Vec::new()));
    }
    None
}

/// guards.mjs `runGitCapture` — execFileSync('git', args, {cwd}) semantics:
/// any spawn failure or non-zero exit reads as `None`; stdout otherwise
/// splits on `\r?\n`, trimmed, empties dropped.
fn run_git_capture(cwd: &str, args: &[&str]) -> Option<Vec<String>> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Some(
        text.split('\n')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
    )
}

/// guards.mjs `extractExplicitPathspecs` — everything after a literal `--`,
/// or (when no `--` is present) every non-flag token.
fn extract_explicit_pathspecs(rest_tokens: &[String]) -> Vec<String> {
    match rest_tokens.iter().position(|t| t == "--") {
        None => rest_tokens.iter().filter(|t| !t.starts_with('-')).cloned().collect(),
        Some(idx) => rest_tokens[idx + 1..].to_vec(),
    }
}

fn is_broad_or_glob_pathspec(p: &str) -> bool {
    GIT_BROAD_PATHSPECS.contains(&p) || p.contains('*')
}

/// Port of guards.mjs `resolveGitMutationPaths(cwd, subcommand, restTokens)`
/// — the repo-relative paths a mutating git subcommand would actually
/// change, from REAL git state at check time (D4). `None` when the set
/// cannot be proved (the caller fails closed on `None`).
///
/// GIT SPAWN PARITY: the ONLY `git` spawns in this crate live in the
/// `commit` branch here, exactly where the node source spawns.
fn resolve_git_mutation_paths(cwd: &str, subcommand: &str, rest_tokens: &[String]) -> Option<Vec<String>> {
    if subcommand == "commit" {
        let dash_dash_idx = rest_tokens.iter().position(|t| t == "--");
        let explicit_pathspecs: Vec<String> = match dash_dash_idx {
            None => Vec::new(),
            Some(idx) => rest_tokens[idx + 1..].to_vec(),
        };
        let pre_dash_dash: &[String] = match dash_dash_idx {
            None => rest_tokens,
            Some(idx) => &rest_tokens[..idx],
        };
        let is_all = has_git_short_flag(pre_dash_dash, 'a') || pre_dash_dash.iter().any(|t| t == "--all");

        let staged = run_git_capture(cwd, &["diff", "--cached", "--name-only"])?;

        if !explicit_pathspecs.is_empty() {
            if explicit_pathspecs.iter().any(|p| is_broad_or_glob_pathspec(p)) {
                return None;
            }
            return Some(explicit_pathspecs);
        }
        if !is_all {
            return Some(staged);
        }
        let unstaged = run_git_capture(cwd, &["diff", "--name-only"])?;
        // new Set([...staged, ...unstaged]) — insertion order preserved.
        let mut merged = staged;
        for p in unstaged {
            if !merged.contains(&p) {
                merged.push(p);
            }
        }
        return Some(merged);
    }

    // add / rm / mv / checkout / restore: resolve to literal pathspec args.
    let pathspecs = extract_explicit_pathspecs(rest_tokens);
    if pathspecs.is_empty() {
        return None; // bare/flags-only invocation: unprovable
    }
    if pathspecs.iter().any(|p| is_broad_or_glob_pathspec(p)) {
        return None; // broad/glob: unprovable
    }
    Some(pathspecs)
}

/// Port of guards.mjs `checkGitBashCommand(root, state, command, {cwd,
/// sessionId, controlRoot})` — git-command awareness for the intake gate
/// (D1/D3/D4, ige-2 / P46 / GH #1), scoped ONLY to the terminal-phase intake
/// gate. `None` = not a git command, phase isn't terminal, or the idle gate
/// is disabled: caller's existing logic decides. Allow verdicts (read-only
/// git, bookkeeping-only mutation) carry no warning; the hook only ever
/// reacts to a deny, matching the mjs `gitVerdict.allow === false` check.
pub fn check_git_bash_command(
    root: &Path,
    state: &State,
    command: &str,
    cwd: &str,
    session_id: Option<&str>,
    control_root: &Path,
) -> Option<Verdict> {
    let session_trimmed = session_id.map(str::trim).filter(|s| !s.is_empty());
    let record: State = match session_trimmed {
        Some(sid) => match resolve_pipeline(control_root, Some(sid)) {
            Ok(resolved) => resolved.record,
            Err(reason) => return Some(Verdict::deny("lane", format!("bee lane guard: {reason}"))),
        },
        None => state.clone(),
    };
    let phase = if record.phase.is_empty() { "idle".to_string() } else { record.phase.clone() };
    if !TERMINAL_PHASES.contains(&phase.as_str()) {
        return None;
    }

    let config = read_config_value(root);
    let idle_gate_on = !matches!(config.get("guards").and_then(|g| g.get("idle_gate")), Some(Value::Bool(false)));
    if !idle_gate_on {
        return None;
    }

    let tokens = tokenize_command(command);
    let (subcommand, rest) = find_git_invocation(&tokens)?;

    if let Some(sub) = subcommand.as_deref() {
        if GIT_READONLY_SUBCOMMANDS.contains(&sub) {
            return Some(Verdict::allow()); // kind git-read-only
        }
        if let Some(flags) = git_readonly_flag_gated(sub) {
            if rest.iter().any(|t| flags.contains(&t.as_str())) {
                return Some(Verdict::allow()); // kind git-read-only
            }
        }

        if sub == "push" {
            return Some(Verdict::deny(
                "git-push",
                intake_refusal(
                    &phase,
                    "`git push`",
                    "git push is outward-facing and is never exempted from this gate, regardless of what it would push. ",
                ),
            ));
        }

        if GIT_MUTATING_SUBCOMMANDS.contains(&sub) {
            let resolved_paths = if GIT_PATH_RESOLVABLE_SUBCOMMANDS.contains(&sub) {
                resolve_git_mutation_paths(cwd, sub, &rest)
            } else {
                None
            };
            let Some(resolved_paths) = resolved_paths else {
                return Some(Verdict::deny(
                    "intake",
                    intake_refusal(
                        &phase,
                        &format!("running `git {sub}` (its changed paths could not be proved bookkeeping-only)"),
                        "",
                    ),
                ));
            };
            if let Some(offending) = resolved_paths
                .iter()
                .map(|p| normalize_rel(p))
                .find(|p| !under_allowed_prefix(p))
            {
                return Some(Verdict::deny(
                    "intake",
                    intake_refusal(&phase, &format!("running `git {sub}` — it would change \"{offending}\""), ""),
                ));
            }
            return Some(Verdict::allow()); // kind git-bookkeeping
        }
    }

    Some(Verdict::deny(
        "git-unrecognized",
        intake_refusal(
            &phase,
            &format!(
                "running `git {}`",
                subcommand.unwrap_or_else(|| command.trim().to_string())
            ),
            "This git subcommand is not recognized as read-only or as a modeled bookkeeping-eligible mutation, so it is refused rather than assumed safe. ",
        ),
    ))
}

// ─── internals-reach guard (state-query-surface, cell sqs-a, D 3fbe2f79) ────

const NODE_INVOCATION_BASENAMES: [&str; 2] = ["node", "nodejs"];
const INLINE_EVAL_FLAGS: [&str; 3] = ["-e", "--eval", "-p"];

/// guards.mjs `inlineEvalScriptsInSegment` — the token(s) right after a bare
/// `-e`/`--eval`/`-p` flag, plus the attached `--eval=<script>` form.
fn inline_eval_scripts_in_segment(segment: &[String]) -> Vec<String> {
    let mut scripts = Vec::new();
    for i in 0..segment.len() {
        let token = &segment[i];
        if INLINE_EVAL_FLAGS.contains(&token.as_str()) {
            if let Some(next) = segment.get(i + 1) {
                scripts.push(next.clone());
            }
            continue;
        }
        if let Some(attached) = token.strip_prefix("--eval=") {
            scripts.push(attached.to_string());
        }
    }
    scripts
}

/// guards.mjs `libImportSpecifierIn` — first `import(...)`/`require(...)`/
/// `import ... from '...'` specifier resolving into a `bin/lib/` or
/// `packages/bee/lib/` module, else `None`.
fn lib_import_specifier_in(script: &str) -> Option<String> {
    if script.is_empty() {
        return None;
    }
    let import_re = Regex::new(
        r#"(?:\brequire\s*\(\s*|\bimport\s*\(\s*|\bimport\b[^'"()]*\bfrom\s*)['"`]([^'"`]+)['"`]"#,
    )
    .ok()?;
    let lib_re = Regex::new(r"(^|/)(?:bin/lib|packages/bee/lib)(/|$)").ok()?;
    for caps in import_re.captures_iter(script) {
        if let Some(specifier) = caps.get(1).map(|m| m.as_str()) {
            if !specifier.is_empty() && lib_re.is_match(specifier) {
                return Some(specifier.to_string());
            }
        }
    }
    None
}

/// Port of guards.mjs `checkBinLibImportBashCommand(command)` — denies ONLY
/// the inline-eval reach (`node -e`/`--eval`/`-p` whose script text
/// imports/requires a `bin/lib/` or `packages/bee/lib/` module), never a
/// file-based `node <path>.mjs` run. `None` = allow (caller unaffected).
pub fn check_bin_lib_import_bash_command(command: &str) -> Option<Verdict> {
    if command.trim().is_empty() {
        return None;
    }
    let tokens = tokenize_command(command);

    let mut i = 0usize;
    while i < tokens.len() {
        if is_separator(&tokens[i]) {
            i += 1;
            continue;
        }
        let mut end = i;
        while end < tokens.len() && !is_separator(&tokens[end]) {
            end += 1;
        }
        let segment = &tokens[i..end];
        i = end;

        let cmd = token_basename(segment.first().map(String::as_str).unwrap_or(""));
        if !NODE_INVOCATION_BASENAMES.contains(&cmd.as_str()) {
            continue;
        }

        for script in inline_eval_scripts_in_segment(segment) {
            if let Some(specifier) = lib_import_specifier_in(&script) {
                return Some(Verdict::deny(
                    "internals-reach",
                    format!(
                        "bee internals-reach guard: this inline eval imports \"{specifier}\" — a bin/lib/ or \
packages/bee/lib/ internal module, reached via `node -e`/`--eval`/`-p` rather than the CLI. \
Internals carry no compatibility promise and this bypasses the CLI's own validation. \
FIX: use the paved read instead — `bee status --json` for current state, or \
`bee <group> --help --json` for a command group's full schema. \
(File-based `node <path>.mjs` runs that import lib modules, e.g. tests, are unaffected.)"
                    ),
                ));
            }
        }
    }

    None
}

// ─── read guard (guards.mjs checkRead: privacy + scout) — rust-port-12 ─────

/// guards.mjs `SECRET_PATTERNS` — file-path patterns that must never be read
/// without asking the human. Case-insensitive, matching the mjs `/i` flag.
const SECRET_PATTERNS: [&str; 7] = [
    r"(?i)(^|[\\/])\.env(\.[A-Za-z0-9._-]+)?$",
    r"(?i)\.pem$",
    r"(?i)\.key$",
    r"(?i)(^|[\\/])id_rsa[^\\/]*$",
    r"(?i)\.p12$",
    r"(?i)(^|[\\/])credentials[^\\/]*$",
    r"(?i)(^|[\\/])secrets\.[^\\/]+$",
];

/// guards.mjs `SCOUT_DIRS` — directories agents should never scout through.
pub const SCOUT_DIRS: [&str; 8] =
    ["node_modules/", "dist/", "build/", ".git/objects", "vendor/", "coverage/", ".next/", "__pycache__/"];

/// `checkRead`'s verdict shape: `{allow, kind, reason, marker}` — `marker`
/// carries the `@@BEE_PRIVACY@@...@@END@@` envelope on a privacy deny only,
/// and rides along inside the hook's own denial reason (joined with a
/// newline), never surfaced separately — mirroring the mjs hook's own
/// `parts.push(verdict.marker)` handling.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadVerdict {
    pub allow: bool,
    pub kind: Option<&'static str>,
    pub reason: Option<String>,
    pub marker: Option<String>,
}

impl ReadVerdict {
    fn allow() -> Self {
        ReadVerdict { allow: true, kind: None, reason: None, marker: None }
    }
}

/// Port of guards.mjs `checkRead(relPath)` — privacy (secret-shaped paths
/// emit the `@@BEE_PRIVACY@@{"file":...,"question":...}@@END@@` marker byte-
/// identically — key order is alphabetical either way: "file" < "question")
/// and scout-dir denies. First-hit-wins, same as the mjs source.
pub fn check_read(rel_path: &str) -> ReadVerdict {
    let normalized = normalize_rel(rel_path);

    if SECRET_PATTERNS.iter().any(|pattern| regex_test(pattern, &normalized)) {
        let question =
            format!("\"{normalized}\" looks like a secret/credential file. Ask the user before reading it.");
        let marker_body = serde_json::json!({ "file": normalized, "question": question }).to_string();
        return ReadVerdict {
            allow: false,
            kind: Some("privacy"),
            reason: Some(format!("bee privacy guard: {question}")),
            marker: Some(format!("@@BEE_PRIVACY@@{marker_body}@@END@@")),
        };
    }

    if let Some(scout_hit) =
        SCOUT_DIRS.iter().find(|dir| normalized.starts_with(*dir) || normalized.contains(&format!("/{dir}")))
    {
        return ReadVerdict {
            allow: false,
            kind: Some("scout"),
            reason: Some(format!(
                "bee scout guard: \"{normalized}\" is inside \"{scout_hit}\" — generated/vendored content. \
Read the source or lockfile instead."
            )),
            marker: None,
        };
    }

    ReadVerdict::allow()
}

// ─── AskUserQuestion schema guard (guards.mjs checkAskUserQuestion) ────────
// rust-port-12.

/// `checkAskUserQuestion`'s verdict: a clean allow, a schema deny (`kind`
/// is `"ask-schema"` throughout, matching the mjs source), or an
/// ask-guard-autofix repair — every violation found was fixable (over-long
/// headers only), so the call proceeds with the rewritten `fixed` input plus
/// human-readable `notes` describing each rewrite (ask-guard-autofix D1/D2).
#[derive(Debug, Clone, PartialEq)]
pub enum AskVerdict {
    Allow,
    Deny { kind: &'static str, reason: String },
    Fixed { fixed: Value, notes: Vec<String> },
}

/// JS `string.length` is a UTF-16 code-unit count, not a codepoint or byte
/// count — matched here so header-length checks agree with the mjs oracle
/// on any non-ASCII header.
fn js_utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// JS `str.slice(0, units)` — a UTF-16-unit prefix.
fn js_utf16_slice_first(s: &str, units: usize) -> String {
    let prefix: Vec<u16> = s.encode_utf16().take(units).collect();
    String::from_utf16_lossy(&prefix)
}

struct HeaderFix {
    index: usize,
    old_header: String,
    new_header: String,
}

/// Port of guards.mjs `checkAskUserQuestion(toolInput)` — turns the
/// harness's opaque "Invalid tool parameters" rejection of an
/// AskUserQuestion call into a clear, self-documenting deny that names the
/// exact schema violation (or, for an over-long header ALONE, an
/// ask-guard-autofix repair instead of a deny). Fail-open on any shape not
/// confidently invalid — `toolInput.questions` absent/non-array allows.
pub fn check_ask_user_question(tool_input: &Value) -> AskVerdict {
    let Some(questions) = tool_input.get("questions").and_then(Value::as_array) else {
        return AskVerdict::Allow;
    };
    let count = questions.len();
    if count < 1 || count > 4 {
        return AskVerdict::Deny {
            kind: "ask-schema",
            reason: format!(
                "bee AskUserQuestion guard: {count} question(s) — the tool takes 1–4 per call. Split into separate calls."
            ),
        };
    }

    let mut header_fixes: Vec<HeaderFix> = Vec::new();
    for (i, q) in questions.iter().enumerate() {
        let Some(q_obj) = q.as_object() else { continue };
        let where_str = if count > 1 { format!(" (question {})", i + 1) } else { String::new() };

        if let Some(header) = q_obj.get("header").and_then(Value::as_str) {
            if js_utf16_len(header) > 12 {
                let truncated = js_utf16_slice_first(header, 11);
                let new_header = format!("{}\u{2026}", truncated.trim_end());
                header_fixes.push(HeaderFix { index: i, old_header: header.to_string(), new_header });
            }
        }

        if let Some(options) = q_obj.get("options").and_then(Value::as_array) {
            let opt_count = options.len();
            if opt_count < 2 || opt_count > 4 {
                return AskVerdict::Deny {
                    kind: "ask-schema",
                    reason: format!(
                        "bee AskUserQuestion guard: {opt_count} option(s){where_str} — each question needs 2–4 options (an \"Other\" free-text choice is added automatically). Fold overflow into a follow-up question."
                    ),
                };
            }
            for (j, o) in options.iter().enumerate() {
                let Some(o_obj) = o.as_object() else { continue };
                let label = o_obj.get("label").and_then(Value::as_str).unwrap_or("");
                if label.trim().is_empty() {
                    return AskVerdict::Deny {
                        kind: "ask-schema",
                        reason: format!(
                            "bee AskUserQuestion guard: option {}{where_str} is missing a non-empty \"label\". Every option needs a label and a description.",
                            j + 1
                        ),
                    };
                }
                let description = o_obj.get("description").and_then(Value::as_str).unwrap_or("");
                if description.trim().is_empty() {
                    return AskVerdict::Deny {
                        kind: "ask-schema",
                        reason: format!(
                            "bee AskUserQuestion guard: option \"{label}\"{where_str} is missing a non-empty \"description\". Every option needs a label and a description."
                        ),
                    };
                }
            }
        }
    }

    if header_fixes.is_empty() {
        return AskVerdict::Allow;
    }

    // Every violation found was fixable — deep-clone toolInput, rewrite each
    // over-long header, and report the rewrite. The original `tool_input` is
    // never mutated (mirrors the mjs `JSON.parse(JSON.stringify(toolInput))`
    // clone).
    let mut fixed = tool_input.clone();
    let mut notes = Vec::with_capacity(header_fixes.len());
    for fix in &header_fixes {
        if let Some(header_slot) =
            fixed.get_mut("questions").and_then(|v| v.get_mut(fix.index)).and_then(|q| q.get_mut("header"))
        {
            *header_slot = Value::String(fix.new_header.clone());
        }
        notes.push(format!(
            "header \"{}\" ({} chars) → \"{}\"",
            fix.old_header,
            js_utf16_len(&fix.old_header),
            fix.new_header
        ));
    }
    AskVerdict::Fixed { fixed, notes }
}

// Conformance proof for this module lives in
// crates/queen-bee/tests/writeguard_core.rs (rust-port-9's core spine),
// crates/queen-bee/tests/writeguard_bash.rs (rust-port-11's Bash path), and
// crates/queen-bee/tests/writeguard_read.rs (rust-port-12's read side,
// apply_patch, and AskUserQuestion), driving the sha256-verified node oracle
// per the rust-port-7 rig discipline.
