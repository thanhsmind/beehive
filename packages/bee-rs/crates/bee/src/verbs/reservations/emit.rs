// the emission plumbing, routing and `reservations list`
//
// Split out of the single 3k-line verbs/reservations.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, resolve_store_root_worktree, Roots, RootsWt, StoreRoots};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ─── emission plumbing (mirrors status_brief.rs / bee.mjs emit/emitError) ──
// pub(crate): verbs/decisions.rs shares this exact emit/fail/timing shape.

pub(crate) struct Ctx {
    pub(crate) root: PathBuf,
    pub(crate) cmd: &'static str,
    pub(crate) use_json: bool,
    pub(crate) t0: Instant,
    pub(crate) drift_changed: bool,
    pub(crate) drift_hint: &'static str,
}

pub(crate) enum Pre {
    Go(Ctx),
    Emitted(ExitCode),
}

pub(crate) fn prelude(cmd: &'static str, use_json: bool, t0: Instant) -> Option<Pre> {
    let cwd = std::env::current_dir().ok()?;
    let root = match resolve_store_root(&cwd) {
        Roots::Ordinary(r) => r,
        Roots::Unsupported(why) => {
            return Some(Pre::Emitted(emit_unsupported_root(&cwd, cmd, use_json, t0, &why)))
        }
        Roots::None => return Some(Pre::Emitted(emit_no_root_error(&cwd, cmd, use_json, t0))),
    };
    let drift = check_manifest_drift(&root);
    Some(Pre::Go(Ctx {
        root,
        cmd,
        use_json,
        t0,
        drift_changed: drift.manifest_changed,
        drift_hint: drift.hint,
    }))
}

/// The WORKTREE-NATIVE prelude, used ONLY by this module's four verbs.
///
/// Deliberately separate from `prelude` above: verbs/decisions.rs,
/// verbs/cells.rs, verbs/state_group.rs and verbs/drivers.rs share that one,
/// and none of them carries the granted-worktree half (control-root re-rooting
/// and hold topology). At cutover the shared door stopped delegating and
/// started REFUSING that one shape by name; only the reservations verbs, which
/// do carry it, opt in here.
pub(crate) enum PreWt {
    Go(Ctx, StoreRoots),
    Emitted(ExitCode),
}

pub(crate) fn prelude_worktree(cmd: &'static str, use_json: bool, t0: Instant) -> Option<PreWt> {
    let cwd = std::env::current_dir().ok()?;
    let roots = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r,
        RootsWt::Unsupported(why) => {
            return Some(PreWt::Emitted(emit_unsupported_root(&cwd, cmd, use_json, t0, &why)))
        }
        RootsWt::None => return Some(PreWt::Emitted(emit_no_root_error(&cwd, cmd, use_json, t0))),
    };
    let drift = check_manifest_drift(&roots.root);
    Some(PreWt::Go(
        Ctx {
            root: roots.root.clone(),
            cmd,
            use_json,
            t0,
            drift_changed: drift.manifest_changed,
            drift_hint: drift.hint,
        },
        roots,
    ))
}

impl Ctx {
    /// bee.mjs emit(): drift line (stderr) + result (stdout) + timing.
    pub(crate) fn emit(&self, result: &Value, text: &str, exit_code: u8) -> ExitCode {
        if self.drift_changed {
            eprintln!("manifest_changed: true — {}", self.drift_hint);
        }
        if self.use_json {
            println!("{}", jsjson::stringify_pretty(result));
        } else {
            println!("{text}");
        }
        record_timing(&self.root, self.cmd, self.t0, exit_code == 0);
        ExitCode::from(exit_code)
    }

    /// bee.mjs emitError(): no drift line, {"error"} or stderr, exit 1.
    pub(crate) fn fail(&self, message: &str) -> ExitCode {
        if self.use_json {
            println!("{}", jsjson::stringify(&json!({ "error": message })));
        } else {
            eprintln!("{message}");
        }
        record_timing(&self.root, self.cmd, self.t0, false);
        ExitCode::FAILURE
    }
}

/// A handler outcome: an emitted payload or a thrown-Error message.
pub(crate) enum Out {
    Emit(Value, String, u8),
    Thrown(String),
}

pub(crate) type R2<T> = Result<T, Err2>;

#[derive(Debug)]
pub(crate) enum Err2 {
    Ex,
    Msg(String),
}

impl From<Exotic> for Err2 {
    fn from(_: Exotic) -> Self {
        Err2::Ex
    }
}

// ─── routing ───────────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "reservations" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let toks: Vec<&str> = args[2..]
        .iter()
        .map(|a| a.to_str())
        .collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None; // Node renders command-scoped help
    }
    let (flags, use_json) = parse_flags(&toks)?;
    match verb {
        "list" => run_list(flags, use_json, t0),
        "reserve" => run_reserve(flags, use_json, t0),
        "release" => run_release(flags, use_json, t0),
        "sweep" => run_sweep(flags, use_json, t0),
        _ => None,
    }
}

pub(crate) fn finish(ctx: &Ctx, out: R2<Out>) -> Option<ExitCode> {
    match out {
        Ok(Out::Emit(result, text, code)) => Some(ctx.emit(&result, &text, code)),
        Ok(Out::Thrown(msg)) => Some(ctx.fail(&msg)),
        Err(Err2::Msg(msg)) => Some(ctx.fail(&msg)),
        Err(Err2::Ex) => None,
    }
}

// ─── reservations list ─────────────────────────────────────────────────────

pub(crate) fn run_list(flags: Flags, use_json: bool, t0: Instant) -> Option<ExitCode> {
    if !keys_known(&flags, &["active-only"]) {
        return None;
    }
    // validate(): a boolean-typed flag given as =value must be true/false.
    match flags.get("active-only") {
        None | Some(FlagV::Present) => {}
        Some(FlagV::S(s)) if s == "true" || s == "false" => {}
        Some(FlagV::S(_)) => return None,
    }
    let active_only = matches!(flags.get("active-only"), Some(FlagV::Present));

    let (ctx, roots) = match prelude_worktree("reservations list", use_json, t0)? {
        PreWt::Go(c, r) => (c, r),
        PreWt::Emitted(code) => return Some(code),
    };
    // resolveMainRoot(root): the shared ledger always lives in MAIN's store.
    let ledger_root = roots.main_root();
    let root_s = ctx.root.to_str()?.to_string();
    let out = (|| -> R2<Out> {
        let reservations = list_reservations(&root_s, active_only, now_ms())?;
        let store = read_holds_store(&ledger_root)?;
        let cross: Vec<Value> = find_foreign_holds(&store, LIST_ALL_HOLDS_SENTINEL, "*", now_ms())?
            .into_iter()
            .cloned()
            .collect();
        let mut cross_lines = Vec::new();
        for h in &cross {
            let cell = match jget(h, "cell") {
                Some(v) if truthy(v) => js_disp(v),
                _ => "unknown".to_string(),
            };
            cross_lines.push(format!(
                "{} | cell {} | {} | mirrored {} | {}",
                js_disp_opt(jget(h, "holder")),
                cell,
                js_disp_opt(jget(h, "path")),
                js_disp_opt(jget(h, "mirrored_at")),
                hold_foreign_expiry(h)?
            ));
        }

        let mut lines: Vec<String> = Vec::new();
        if reservations.is_empty() {
            lines.push("No reservations.".to_string());
        } else {
            lines.push(
                reservations
                    .iter()
                    .map(|r| {
                        // released_at is null by construction (msn-16 shim) —
                        // the ternary's released branch is unreachable.
                        format!(
                            "{} | cell {} | {} | reserved {} | active/expired by TTL",
                            js_disp_opt(r.agent.as_ref()),
                            js_disp_opt(r.cell.as_ref()),
                            r.path,
                            js_disp_opt(r.reserved_at.as_ref()),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        if !cross.is_empty() {
            lines.push("cross_worktree:".to_string());
            lines.extend(cross_lines);
        }
        let result = json!({
            "reservations": reservations.iter().map(resv_to_value).collect::<Vec<_>>(),
            "cross_worktree": cross,
        });
        Ok(Out::Emit(result, lines.join("\n"), 0))
    })();
    finish(&ctx, out)
}
