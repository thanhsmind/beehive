// the prompt renderer (contract C4)
//
// Split out of the single 4.9k-line verbs/drivers.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::fsutil::{ensure_dir, read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use crate::roots::{resolve_store_root, Roots};
use crate::state::read_config_raw;
use crate::verbs::reservations::{
    finish, js_is_ws, parse_flags, prelude, pseudo_uuid_v4, truthy, FlagV, Flags, Out, Pre, R2,
};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use crate::verbs::reservations::{
    release_reservations_for_agent, reserve_path_atomic, Err2, ReserveOutcome,
};
use serde_json::{Map, Number, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::Instant;

// ═══ prompt renderer (lib/prompt-renderer.mjs — contract C4) ═══════════════
//
// The templates are embedded at BUILD time from the canonical checkout, the
// same way crate::registry embeds the command-manifest payload: both runtimes
// then hash/render identical bytes. loadPrompt's normalization (CRLF -> LF,
// ONE trailing newline stripped) is applied here at load, exactly as Node
// applies it after readFileSync.

pub(crate) const PROMPT_WORKER_CELL: &str = include_str!("../../../../../../bee/prompts/worker-cell.md");

pub(crate) const PROMPT_GATHER: &str = include_str!("../../../../../../bee/prompts/gather.md");

pub(crate) const PROMPT_REVIEWER: &str = include_str!("../../../../../../bee/prompts/reviewer.md");

pub(crate) const PROMPT_ADVISOR: &str = include_str!("../../../../../../bee/prompts/advisor.md");

pub(crate) fn embedded_prompt(name: &str) -> Option<&'static str> {
    match name {
        "worker-cell" => Some(PROMPT_WORKER_CELL),
        "gather" => Some(PROMPT_GATHER),
        "reviewer" => Some(PROMPT_REVIEWER),
        "advisor" => Some(PROMPT_ADVISOR),
        _ => None,
    }
}

/// provenance: prompt-renderer.mjs loadPrompt — CRLF normalized to LF and ONE
/// trailing newline stripped.
pub(crate) fn normalize_template(raw: &str) -> String {
    let lf = raw.replace("\r\n", "\n");
    match lf.strip_suffix('\n') {
        Some(s) => s.to_string(),
        None => lf,
    }
}

pub(crate) fn load_prompt(name: &str) -> Option<String> {
    embedded_prompt(name).map(normalize_template)
}

/// Runtime skew guard (R2 write-guard discipline). The Node renderer resolves
/// prompts RELATIVE TO ITS OWN MODULE — `packages/bee/prompts/` for a
/// canonical checkout, `.bee/bin/prompts/` for a vendored engine. Whenever a
/// repo ships either directory, its bytes must equal the compiled-in bytes or
/// this port would render a stale template: byte-compare and delegate on any
/// mismatch. A repo shipping neither (a pure-binary install) trusts the
/// embedded copy, which is the only copy that exists there.
pub(crate) fn prompts_match_disk(root: &Path, name: &str) -> bool {
    let embedded = match embedded_prompt(name) {
        Some(t) => t,
        None => return false,
    };
    let candidates = [
        root.join("packages").join("bee").join("prompts").join(format!("{name}.md")),
        root.join(".bee").join("bin").join("prompts").join(format!("{name}.md")),
    ];
    for file in candidates {
        let Ok(bytes) = std::fs::read(&file) else { continue };
        let disk = String::from_utf8_lossy(&bytes);
        if normalize_template(&disk) != normalize_template(embedded) {
            return false;
        }
    }
    true
}

/// provenance: prompt-renderer.mjs render(template, vars) — the whole minimal
/// grammar, byte-faithful:
///   * `\n{{#if NAME}}<inner>\n{{/if}}` blocks are consumed WITH the newline
///     that precedes the opening marker; a truthy var splices `<inner>` in
///     verbatim (substitution still runs over it afterwards), a falsy var
///     leaves zero residue bytes. Non-greedy `[\s\S]*?` == first following
///     `\n{{/if}}`.
///   * a `{{#if ` inside an inner block is a loud refusal (nesting), as is any
///     surviving `{{#if `/`{{/if}}` after the block pass.
///   * `{{NAME}}` placeholders substitute String(vars[NAME]); an
///     undefined/null value is a loud refusal.
/// Names are `[A-Za-z0-9_]+` in both markers.
pub(crate) fn render(template: &str, vars: &[(&str, &str)]) -> Result<String, String> {
    let lookup = |name: &str| -> Option<&str> {
        vars.iter().find(|(k, _)| *k == name).map(|(_, v)| *v)
    };

    // ── pass 1: conditional blocks ────────────────────────────────────────
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        // Match /\n\{\{#if ([A-Za-z0-9_]+)\}\}/ at i.
        let Some(rest) = template.get(i..) else {
            out.push_str(&template[i..]);
            break;
        };
        if !rest.starts_with("\n{{#if ") {
            let ch = template[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let name_start = i + "\n{{#if ".len();
        let name_end = match template[name_start..].find("}}") {
            Some(p) => name_start + p,
            None => {
                let ch = template[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
                continue;
            }
        };
        let name = &template[name_start..name_end];
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            let ch = template[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let inner_start = name_end + 2;
        let Some(close_rel) = template[inner_start..].find("\n{{/if}}") else {
            let ch = template[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };
        let inner = &template[inner_start..inner_start + close_rel];
        if inner.contains("{{#if ") {
            return Err(format!(
                "prompt-renderer: nested {{{{#if}}}} inside block \"{name}\" — nesting is not supported."
            ));
        }
        // JS truthiness of the substituted var: only a non-empty string here
        // (dispatch-prepare passes joined line lists), undefined -> falsy.
        if lookup(name).map(|v| !v.is_empty()).unwrap_or(false) {
            out.push_str(inner);
        }
        i = inner_start + close_rel + "\n{{/if}}".len();
    }
    if out.contains("{{#if ") || out.contains("{{/if}}") {
        return Err("prompt-renderer: unmatched or malformed {{#if}}/{{/if}} marker in template.".to_string());
    }

    // ── pass 2: {{name}} substitution (a substituted value is never
    //    re-scanned — the scan walks the PRE-substitution text) ────────────
    let mut result = String::with_capacity(out.len());
    let mut i = 0usize;
    while i < out.len() {
        let rest = &out[i..];
        if let Some(after) = rest.strip_prefix("{{") {
            if let Some(close) = after.find("}}") {
                let name = &after[..close];
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    match lookup(name) {
                        Some(v) => result.push_str(v),
                        None => {
                            return Err(format!(
                                "prompt-renderer: no value supplied for placeholder {{{{{name}}}}}."
                            ))
                        }
                    }
                    i += 2 + close + 2;
                    continue;
                }
            }
        }
        let ch = rest.chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }
    Ok(result)
}
