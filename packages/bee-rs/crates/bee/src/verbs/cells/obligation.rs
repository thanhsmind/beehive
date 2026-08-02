// the derived regen obligation and the config slice behind it
//
// Split out of the single 9.4k-line verbs/cells.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::lock;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root, Roots};
use crate::state as bstate;
use crate::verbs::reservations as rsv;
use crate::verbs::reservations::{Err2, FlagV, Out, R2};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use serde_json::{json, Map, Number, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── derived regen obligation (lib/cells.mjs D1/D2) ────────────────────────

pub(crate) const REGEN_ACK_FIELD: &str = "regen_obligation_ack";

pub(crate) struct RegenGuardDef {
    /// Where the covered roots come from — named in the refusal so a reader
    /// can go and check the derivation rather than trust the message.
    pub(crate) authority: &'static str,
    pub(crate) covers: &'static str,
    pub(crate) required: &'static str,
    pub(crate) command: &'static str,
    pub(crate) regen: &'static str,
    pub(crate) derive: fn() -> (Vec<String>, Vec<String>),
}

// R6 CUTOVER — WHERE THE COVERED ROOTS COME FROM NOW.
//
// Both guards used to READ A .mjs FILE AT RUNTIME and parse its source for the
// paths it operated on (`path.join(REPO_ROOT, …)` literals in
// scripts/release_manifest.mjs; `checkGroup(managed.X, "<relDir>")` calls in
// scripts/ledger_parity.mjs). That was not obfuscation for its own sake — it
// was decision D2: the obligation's scope must be DERIVED from the thing it
// guards, never pasted next to it, or the two drift and the guard quietly
// covers the wrong set.
//
// Both scripts are deleted. Parsing is therefore replaced with the strongest
// available form of the same property: the guards read the SAME CONSTANT the
// authority itself uses.
//
//   * `devtools::release_manifest::INVENTORY_ROOTS` is the list
//     `build_current_records` enumerates, pinned to it in BOTH directions by
//     `every_inventory_root_covers_what_the_builder_enumerates` and
//     `every_inventory_root_is_actually_enumerated`.
//   * `onboard::plan::LEDGER_COVERED_ROOTS` is the directory set the managed
//     ledger fingerprints, pinned to `build_managed_versions` by
//     `ledger_groups_cover_every_managed_file_group`.
//
// This is stronger than the parse it replaces: a source edit that changed the
// covered set used to be caught only if the PARSER still recognised the new
// shape (and `derive_regen_guards` threw when it did not), whereas a shared
// constant cannot be out of date without a test going red.
//
// The old failure mode is gone with it. `derive_regen_guards` used to
// `continue` — silently deactivating the guard — when the script was missing.
// Deleting the two `.mjs` files would have hit exactly that arm and switched
// BOTH obligations off with no output at all. There is no missing-file arm any
// more: the authorities are compiled in, and an empty root list is still a
// loud refusal.
pub(crate) const REGEN_GUARDS: [RegenGuardDef; 2] = [
    RegenGuardDef {
        authority: "devtools::release_manifest::INVENTORY_ROOTS",
        covers: "the release manifest hashes",
        required: "bee dev release-manifest --check",
        command: "bee dev release-manifest --check",
        regen: "bee dev render-skill-trees, then bee onboard --repo-root . --apply, then bee dev release-manifest --write (in that order)",
        derive: derive_manifest_scope,
    },
    RegenGuardDef {
        authority: "onboard::plan::LEDGER_COVERED_ROOTS",
        covers: "the .bee/onboarding.json managed-hash ledger covers",
        required: "bee onboard --repo-root . --json",
        command: "bee onboard --repo-root . --json",
        regen: "bee onboard --repo-root . --apply",
        derive: derive_ledger_scope,
    },
];

/// The release-manifest scope: every inventory root EXCEPT the manifest file
/// itself, which becomes the required file instead (a cell that edits a covered
/// root must also list the regenerated manifest in `files`). Same split the
/// `.mjs` parse produced from MANIFEST_PATH.
fn derive_manifest_scope() -> (Vec<String>, Vec<String>) {
    let manifest = crate::devtools::release_manifest_rel().to_string();
    let mut roots: Vec<String> = crate::devtools::release_manifest_roots()
        .iter()
        .map(|r| (*r).to_string())
        .filter(|r| *r != manifest)
        .collect();
    js_default_str_sort(&mut roots);
    (roots, vec![manifest])
}

/// The ledger scope: the host directories the managed-hash groups cover. No
/// required file — re-running onboarding rewrites `.bee/onboarding.json`
/// itself, so there is nothing for the cell to list by hand.
pub(crate) fn derive_ledger_scope() -> (Vec<String>, Vec<String>) {
    let mut roots: Vec<String> =
        crate::onboard::ledger_covered_roots().into_iter().map(str::to_string).collect();
    js_default_str_sort(&mut roots);
    (roots, Vec::new())
}

pub(crate) struct ActiveGuard {
    pub(crate) def: &'static RegenGuardDef,
    pub(crate) roots: Vec<String>,
    pub(crate) required_files: Vec<String>,
}

/// deriveRegenGuards: absent script -> inactive; present-but-blind -> throw.
pub(crate) fn derive_regen_guards() -> MR<Vec<ActiveGuard>> {
    let mut active = Vec::new();
    for guard in REGEN_GUARDS.iter() {
        let (roots, required_files) = (guard.derive)();
        // There is no "guard not installed" arm any more (see the note above
        // REGEN_GUARDS): the authorities are compiled into this binary, so the
        // only way to get an empty scope is a real defect — and a blind guard
        // refuses rather than passing everything.
        if roots.is_empty() {
            return Err(Fail::Thrown(format!(
                "regen obligation: could not derive any covered root from {} — the guard would be blind, so the write is refused rather than passed silently. FIX: that authority returned an empty root set; restore it there (never paste a literal root list in — see D2).",
                guard.authority
            )));
        }
        active.push(ActiveGuard { def: guard, roots, required_files });
    }
    Ok(active)
}

/// lib/cells.mjs normalizeCellPath.
pub(crate) fn normalize_cell_path(value: &str) -> String {
    let mut s = js_trim(value).replace('\\', "/");
    if let Some(rest) = s.strip_prefix("./") {
        s = rest.to_string(); // /^\.\//, one occurrence
    }
    while s.ends_with('/') {
        s.pop();
    }
    s
}

pub(crate) fn path_under_root(file: &str, root_path: &str) -> bool {
    file == root_path || file.starts_with(&format!("{root_path}/"))
}

/// lib/cells.mjs regenObligationRefusal — None when nothing is owed.
pub(crate) fn regen_obligation_refusal(cell: &Map<String, Value>, verb: &str) -> MR<Option<String>> {
    if let Some(ack) = cell.get(REGEN_ACK_FIELD) {
        if matches!(ack, Value::String(s) if !js_trim(s).is_empty()) {
            return Ok(None); // D1 escape hatch
        }
    }
    let files: Vec<String> = match cell.get("files") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|f| match f {
                Value::String(s) if !js_trim(s).is_empty() => Some(normalize_cell_path(s)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    };
    if files.is_empty() {
        return Ok(None);
    }
    let verify = match cell.get("verify") {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    let id = match cell.get("id") {
        Some(Value::String(s)) if !s.is_empty() => s.clone(),
        _ => "(unknown id)".to_string(),
    };
    for guard in derive_regen_guards()? {
        let mut hit: Option<(String, String)> = None;
        for file in &files {
            if let Some(matched) = guard.roots.iter().find(|r| path_under_root(file, r)) {
                hit = Some((file.clone(), matched.clone()));
                break;
            }
        }
        let Some((hit_path, hit_root)) = hit else { continue };
        let mut missing = Vec::new();
        if !verify.contains(guard.def.required) {
            missing.push(format!("verify does not contain \"{}\"", guard.def.required));
        }
        for required_file in &guard.required_files {
            if !files.contains(required_file) {
                missing.push(format!("files does not list \"{required_file}\""));
            }
        }
        if missing.is_empty() {
            continue;
        }
        let mut fixes = Vec::new();
        if !verify.contains(guard.def.required) {
            fixes.push(format!("add `{}` to this cell's verify", guard.def.command));
        }
        for required_file in &guard.required_files {
            if !files.contains(required_file) {
                fixes.push(format!("add \"{required_file}\" to its files"));
            }
        }
        return Ok(Some(format!(
            "{verb}: REGEN_OBLIGATION — cell \"{id}\" touches \"{hit_path}\", which falls under \"{hit_root}\", a root {} (derived at runtime from {}, never a list kept here). Missing: {}. FIX: {}, and run the regen inside THIS cell — {}. To skip deliberately, set \"{REGEN_ACK_FIELD}\" on the cell to a one-line reason; it is recorded in the cell, so skipping is a named act rather than an oversight. For parallel waves, the recognized value \"wave-barrier\" defers the regen to the orchestrator, which owes the full regen chain once at wave close, in the wave-close commit, before the wave is declared clean (parallel-default D2). The write is refused; nothing was written.",
            guard.def.covers,
            guard.def.authority,
            missing.join("; "),
            fixes.join(", "),
            guard.def.regen,
        )));
    }
    Ok(None)
}

pub(crate) fn assert_regen_obligation(cell: &Map<String, Value>, verb: &str) -> MR<()> {
    match regen_obligation_refusal(cell, verb)? {
        Some(refusal) => Err(Fail::Thrown(refusal)),
        None => Ok(()),
    }
}

// ─── config slice (readConfig -> commands.test; `verify` retired) ─────────

pub(crate) const NO_TEST_SENTINEL: &str = "none";

pub(crate) struct CommandsSlice {
    /// normalizeCommands' `test`: Some(list) for a declared string/array.
    pub(crate) test: Option<Vec<String>>,
}

pub(crate) fn read_commands_slice(root: &Path) -> MR<CommandsSlice> {
    let config = bstate::read_config_raw(root).map_err(|_| Fail::Delegate)?;
    let raw = config.get("commands");
    let mut out = CommandsSlice { test: None };
    let Some(Value::Object(raw)) = raw else { return Ok(out) };
    match raw.get("test") {
        Some(Value::String(s)) if !js_trim(s).is_empty() => {
            out.test = Some(vec![js_trim(s).to_string()]);
        }
        Some(Value::Array(items)) => {
            let list: Vec<String> = items
                .iter()
                .filter_map(|c| match c {
                    Value::String(s) if !js_trim(s).is_empty() => Some(js_trim(s).to_string()),
                    _ => None,
                })
                .collect();
            if !list.is_empty() {
                out.test = Some(list);
            }
        }
        _ => {}
    }
    Ok(out)
}

/// isNoTestRepo over the normalized commands slice. `commands.verify` was
/// retired, so `commands.test: "none"` is the one way to declare a no-test repo.
pub(crate) fn is_no_test_repo(commands: &CommandsSlice) -> bool {
    matches!(&commands.test, Some(list) if list.len() == 1 && list[0] == NO_TEST_SENTINEL)
}
