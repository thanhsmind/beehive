// the shared dispatch frame, the bundle root, and the constants
//
// Split out of the single 4.4k-line verbs/knowledge.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::jsjson;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_any as resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV, Flags};
use serde_json::{json, Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── shared dispatch frame (pub(crate): intent_group / reviews / tmp_group) ─

pub(crate) struct GCtx {
    pub(crate) root: PathBuf,
    pub(crate) cmd: &'static str,
    pub(crate) json: bool,
    pub(crate) t0: Instant,
    pub(crate) drift_changed: bool,
    pub(crate) drift_hint: &'static str,
}

pub(crate) enum GPre {
    Go(GCtx),
    Emitted(ExitCode),
}

/// bee.mjs main()'s root + manifest-drift preamble. `pre_json` is the
/// pre-parse rest scan (the no-root error fires before parseFlags in Node);
/// `json` is the authoritative post-parse flag. Ok wrapped in Option:
/// None => delegate (linked worktree, corrupt drift cache).
pub(crate) fn g_prelude(cmd: &'static str, json: bool, pre_json: bool, t0: Instant) -> Option<GPre> {
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(GPre::Emitted(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why)))
        }
        Roots::None => return Some(GPre::Emitted(emit_no_root_error(&cwd, cmd, pre_json, t0))),
    };
    let drift = check_manifest_drift(&root);
    Some(GPre::Go(GCtx {
        root,
        cmd,
        json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    }))
}

impl GCtx {
    /// bee.mjs emit(): drift line (stderr) + result/text (stdout) + timing.
    pub(crate) fn emit(&self, result: &Value, text: &str, exit_code: u8) -> ExitCode {
        if self.drift_changed {
            eprintln!("manifest_changed: true — {}", self.drift_hint);
        }
        if self.json {
            println!("{}", jsjson::stringify_pretty(result));
        } else {
            println!("{text}");
        }
        crate::verbs::record_timing(&self.root, self.cmd, self.t0, exit_code == 0);
        ExitCode::from(exit_code)
    }

    /// bee.mjs emitError(): no drift line, {"error"} on stdout or msg on
    /// stderr, exit 1.
    pub(crate) fn fail(&self, message: &str) -> ExitCode {
        if self.json {
            println!("{}", jsjson::stringify(&json!({ "error": message })));
        } else {
            eprintln!("{message}");
        }
        crate::verbs::record_timing(&self.root, self.cmd, self.t0, false);
        ExitCode::FAILURE
    }
}

/// jsonRequested — bee.mjs main()'s pre-parse rest scan.
pub(crate) fn pre_json_scan(toks: &[&str]) -> bool {
    toks.iter().any(|t| *t == "--json" || t.starts_with("--json="))
}

/// A registry type:"boolean" flag through validate() + the handler's
/// `flags.x === true` test: bare flag => true; "true"/"false" string values
/// pass validate but are NOT `=== true`; anything else fails validate
/// (delegate => None).
pub(crate) fn js_bool_flag(flags: &Flags, name: &str) -> Option<bool> {
    match flags.get(name) {
        None => Some(false),
        Some(FlagV::Present) => Some(true),
        Some(FlagV::S(s)) if s == "true" || s == "false" => Some(false),
        Some(FlagV::S(_)) => None,
    }
}

/// Template-literal coercion for a possibly-absent field (undefined =>
/// "undefined"), shared by the group files.
pub(crate) fn js_str_or_undefined(v: Option<&Value>) -> String {
    match v {
        None => "undefined".to_string(),
        Some(v) => jsjson::js_to_string(v),
    }
}

// ─── bundle root (bundleDir + the delegating slice of resolveProductRoot) ──

/// docs/knowledge under the product root. None => delegate: a configured
/// non-empty product_root (divorce topology, GitHub #14 — Node's warn/path
/// semantics live there) or a corrupt config file (V8 warning).
pub(crate) fn bundle_dir(root: &Path) -> Option<PathBuf> {
    let config = read_config_raw(root);
    match config.get("product_root") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if s.is_empty() => {}
        Some(_) => return None,
    }
    Some(root.join("docs").join("knowledge"))
}

// ─── constants (lib/knowledge.mjs) ─────────────────────────────────────────

pub(crate) const OKF_VERSION: &str = "0.1";

pub(crate) const CONCEPT_TYPES: [&str; 9] = [
    "bee.area",
    "bee.feature",
    "bee.work-item",
    "bee.plan",
    "bee.delivery",
    "bee.decision",
    "bee.pattern",
    "bee.runbook",
    "bee.evidence",
];

pub(crate) const ROOT_KEY_ORDER: [&str; 6] = ["type", "title", "description", "tags", "timestamp", "resource"];

pub(crate) const BEE_KEY_ORDER: [&str; 17] = [
    "id",
    "lifecycle",
    "areas",
    "required_context",
    "decisions",
    "sources",
    "lane",
    "polarity",
    "critical",
    "authoritative_for",
    "review_status",
    "supersedes",
    "superseded_by",
    "owns.code",
    "owns.skills",
    "owns.tests",
    "applied_at",
];

pub(crate) const PROFILE_REQUIRED: [&[&str]; 4] = [&["title"], &["description"], &["bee", "id"], &["bee", "lifecycle"]];

pub(crate) fn key_re_ok(key: &str) -> bool {
    // /^[A-Za-z_][A-Za-z0-9_.-]*$/
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

pub(crate) fn is_reserved_basename(base: &str) -> bool {
    base == "index.md" || base == "log.md"
}

/// JS `\s` (same set String.prototype.trim strips) — via reservations.
pub(crate) fn js_is_space(c: char) -> bool {
    crate::verbs::reservations::js_is_ws(c)
}

pub(crate) fn js_quote_str(s: &str) -> String {
    jsjson::stringify(&Value::String(s.to_string()))
}
