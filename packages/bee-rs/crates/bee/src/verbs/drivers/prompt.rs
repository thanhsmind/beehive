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

pub(crate) const AGENT_BUILD: &str = include_str!("../../../../../../bee/agents/bee-build.md.tmpl");

pub(crate) const AGENT_DESIGN: &str = include_str!("../../../../../../bee/agents/bee-design.md.tmpl");

pub(crate) const DESIGN_CELL_PREAMBLE: &str = "Design cell: before you write UI, load the `bee-design` skill and every installed `bee-design-*` pack, and follow its workflow. The design check is `bee design gate`; record it as your proof. It needs Node 20 or newer; without it, it prints `design gate: not run — Node 20 or newer is missing`, the proof reason must carry `design gate not run`, and you never write that it ran. With no `bee-design` skill installed, say the design taste is absent and execute the cell as a plain cell executor.";

pub(crate) const AGENT_GATHER: &str = include_str!("../../../../../../bee/agents/bee-gather.md.tmpl");

pub(crate) const AGENT_EXTRACT: &str = include_str!("../../../../../../bee/agents/bee-extract.md.tmpl");

pub(crate) const AGENT_REVIEW: &str = include_str!("../../../../../../bee/agents/bee-review.md.tmpl");

pub(crate) fn embedded_agent_body(agent_name: &str) -> Option<&'static str> {
    let source = match agent_name {
        "bee-build" => AGENT_BUILD,
        "bee-design" => AGENT_DESIGN,
        "bee-gather" => AGENT_GATHER,
        "bee-extract" => AGENT_EXTRACT,
        "bee-review" => AGENT_REVIEW,
        _ => return None,
    };
    let (_front, body) = crate::textutil::split_frontmatter(source);
    Some(body.trim())
}

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

thread_local! {
    static LAST_PROMPT_ROOT: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn set_last_prompt_root(root: &Path) {
    LAST_PROMPT_ROOT.with(|cell| {
        *cell.borrow_mut() = Some(root.to_path_buf());
    });
}

fn supported_vars_for(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "worker-cell" => Some(&[
            "worker",
            "cell_id",
            "feature",
            "cell_json",
            "learned_context",
            "expertise",
            "advisor",
            "prior_rounds",
            "worktree_root",
            "control_root",
            "original_request",
            "native_bookkeeping",
            "herding",
            "native_worktree_check",
        ]),
        "gather" | "reviewer" | "advisor" => Some(&[
            "brief",
            "expertise",
            "purpose",
            "original_request",
            "seat",
        ]),
        _ => None,
    }
}

fn can_render_prompt(name: &str, template: &str) -> bool {
    let Some(supported) = supported_vars_for(name) else {
        return false;
    };
    let mut rest = template;
    while let Some(p) = rest.find("{{") {
        let after = &rest[p + 2..];
        let Some(close) = after.find("}}") else {
            return false;
        };
        let raw = &after[..close];
        if raw.starts_with('/') {
            if raw != "/if" {
                return false;
            }
        } else {
            let var = raw.strip_prefix("#if ").unwrap_or(raw);
            if !var.is_empty() && var.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                if !supported.contains(&var) {
                    return false;
                }
            }
        }
        rest = &after[close + 2..];
    }
    let vars_nonempty: Vec<(&str, &str)> = supported.iter().map(|k| (*k, "1")).collect();
    let vars_empty: Vec<(&str, &str)> = supported.iter().map(|k| (*k, "")).collect();
    render(template, &vars_nonempty).is_ok() && render(template, &vars_empty).is_ok()
}

pub(crate) fn load_prompt(name: &str) -> Option<String> {
    if let Some(root) = LAST_PROMPT_ROOT.with(|cell| cell.borrow().clone()) {
        let candidates = [
            root.join("packages").join("bee").join("prompts").join(format!("{name}.md")),
            root.join(".bee").join("bin").join("prompts").join(format!("{name}.md")),
        ];
        for file in candidates {
            if let Ok(bytes) = std::fs::read(&file) {
                let disk = String::from_utf8_lossy(&bytes);
                return Some(normalize_template(&disk));
            }
        }
    }
    if let Some(source_root) = crate::devtools::bee_source_root() {
        let candidates = [
            source_root.join("packages").join("bee").join("prompts").join(format!("{name}.md")),
            source_root.join(".bee").join("bin").join("prompts").join(format!("{name}.md")),
        ];
        for file in candidates {
            if let Ok(bytes) = std::fs::read(&file) {
                let disk = String::from_utf8_lossy(&bytes);
                return Some(normalize_template(&disk));
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let roots_to_check = match resolve_store_root(&cwd) {
            Roots::Ordinary(r) => vec![r],
            Roots::Unsupported(crate::roots::Unsupported::GrantedWorktree { main_root }) => {
                vec![cwd.clone(), main_root]
            }
            _ => vec![cwd],
        };
        for r in roots_to_check {
            let candidates = [
                r.join("packages").join("bee").join("prompts").join(format!("{name}.md")),
                r.join(".bee").join("bin").join("prompts").join(format!("{name}.md")),
            ];
            for file in candidates {
                if let Ok(bytes) = std::fs::read(&file) {
                    let disk = String::from_utf8_lossy(&bytes);
                    return Some(normalize_template(&disk));
                }
            }
        }
    }
    embedded_prompt(name).map(normalize_template)
}

#[derive(Debug)]
pub(crate) enum PromptSkew {
    NoEmbedded,
    Disk(std::path::PathBuf),
}

pub(crate) fn prompt_skew(root: &Path, name: &str) -> Option<PromptSkew> {
    set_last_prompt_root(root);
    let embedded = embedded_prompt(name)?;
    let source_file = root.join("packages").join("bee").join("prompts").join(format!("{name}.md"));
    let vendored_file = root.join(".bee").join("bin").join("prompts").join(format!("{name}.md"));

    let source_bytes = std::fs::read(&source_file).ok();
    let vendored_bytes = std::fs::read(&vendored_file).ok();

    match (source_bytes, vendored_bytes) {
        (None, None) => None,
        (Some(src), Some(vnd)) => {
            let src_text = normalize_template(&String::from_utf8_lossy(&src));
            let vnd_text = normalize_template(&String::from_utf8_lossy(&vnd));
            if src_text != vnd_text {
                return Some(PromptSkew::Disk(source_file));
            }
            if !can_render_prompt(name, &src_text) {
                return Some(PromptSkew::Disk(source_file));
            }
            None
        }
        (Some(src), None) => {
            let src_text = normalize_template(&String::from_utf8_lossy(&src));
            if src_text == normalize_template(embedded) {
                None
            } else {
                Some(PromptSkew::Disk(source_file))
            }
        }
        (None, Some(vnd)) => {
            let vnd_text = normalize_template(&String::from_utf8_lossy(&vnd));
            if vnd_text == normalize_template(embedded) {
                None
            } else {
                Some(PromptSkew::Disk(vendored_file))
            }
        }
    }
}

/// The refusal a skew earns. Skew used to `return None` — "delegate to the
/// Node renderer" — and the dispatcher turned that into its generic
/// "unsupported argument shape" message, which blames the caller's flags for
/// a stale binary. The Node runtime was deleted at R6, so there is nothing to
/// delegate TO: name the file, the cause, and the remedy.
pub(crate) fn prompt_skew_message(cmd: &str, name: &str, skew: &PromptSkew) -> String {
    match skew {
        PromptSkew::NoEmbedded => format!(
            "bee {cmd}: no prompt template named `{name}` is compiled into this binary \
             (bee {}), so no worker prompt can be rendered. FIX: rebuild from a source tree \
             that carries packages/bee/prompts/{name}.md.",
            crate::version::BEE_VERSION
        ),
        PromptSkew::Disk(path) => format!(
            "bee {cmd}: prompt skew — {} does not match the `{name}` template compiled into \
             this binary (bee {}), so a worker would be handed a prompt that is not the one on \
             disk. Nothing is dispatched. FIX: re-vendor the binary from this source tree \
             (`cargo build --release --manifest-path packages/bee-rs/Cargo.toml -p bee`, then \
             copy the built `bee` over `.bee/bin/bee`), or restore that file to the bytes this \
             binary was built from.",
            path.display(),
            crate::version::BEE_VERSION
        ),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_agent_body_resolves_known_agents() {
        let body = embedded_agent_body("bee-gather").expect("bee-gather body should exist");
        assert!(body.starts_with("You are a bee gather worker"));
        assert!(!body.contains("---"));

        let build = embedded_agent_body("bee-build").expect("bee-build body should exist");
        assert!(build.starts_with("You are a bee execution worker"));
        assert!(!build.contains("---"));

        let extract = embedded_agent_body("bee-extract").expect("bee-extract body should exist");
        assert!(extract.starts_with("You are a bee extract worker"));
        assert!(!extract.contains("---"));

        let review = embedded_agent_body("bee-review").expect("bee-review body should exist");
        assert!(review.starts_with("You are a bee review worker"));
        assert!(!review.contains("---"));
    }

    #[test]
    fn embedded_agent_body_returns_none_for_unknown_agents() {
        assert_eq!(embedded_agent_body("general-purpose"), None);
        assert_eq!(embedded_agent_body("nonsense"), None);
    }

    #[test]
    fn prompt_skew_allows_matching_on_disk_prompts_even_if_different_from_embedded() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src_dir = root.join("packages").join("bee").join("prompts");
        let bin_dir = root.join(".bee").join("bin").join("prompts");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&bin_dir).unwrap();
        let custom_prompt = "worker cell custom: {{worker}} {{cell_id}} {{feature}} {{cell_json}}\n";
        std::fs::write(src_dir.join("worker-cell.md"), custom_prompt).unwrap();
        std::fs::write(bin_dir.join("worker-cell.md"), custom_prompt).unwrap();
        assert!(prompt_skew(root, "worker-cell").is_none());
        let loaded = load_prompt("worker-cell").unwrap();
        assert!(loaded.contains("worker cell custom:"));
        let rendered = render(&loaded, &[("worker", "w1"), ("cell_id", "c1"), ("feature", "f1"), ("cell_json", "{}")]).unwrap();
        assert_eq!(rendered, "worker cell custom: w1 c1 f1 {}");
    }

    #[test]
    fn prompt_skew_refuses_when_source_and_vendored_differ() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src_dir = root.join("packages").join("bee").join("prompts");
        let bin_dir = root.join(".bee").join("bin").join("prompts");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(src_dir.join("worker-cell.md"), "version A\n").unwrap();
        std::fs::write(bin_dir.join("worker-cell.md"), "version B\n").unwrap();
        let skew = prompt_skew(root, "worker-cell");
        assert!(skew.is_some());
    }

    #[test]
    fn prompt_skew_refuses_when_template_cannot_be_rendered() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src_dir = root.join("packages").join("bee").join("prompts");
        let bin_dir = root.join(".bee").join("bin").join("prompts");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&bin_dir).unwrap();
        let broken = "worker cell: {{unsupported_future_var}}\n";
        std::fs::write(src_dir.join("worker-cell.md"), broken).unwrap();
        std::fs::write(bin_dir.join("worker-cell.md"), broken).unwrap();
        let skew = prompt_skew(root, "worker-cell");
        assert!(skew.is_some());
    }

    #[test]
    fn prompt_skew_refuses_nested_if_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let src_dir = root.join("packages").join("bee").join("prompts");
        let bin_dir = root.join(".bee").join("bin").join("prompts");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&bin_dir).unwrap();
        let nested = "text\n{{#if worker}}\n{{#if feature}}\ninner\n{{/if}}\n{{/if}}\n";
        std::fs::write(src_dir.join("worker-cell.md"), nested).unwrap();
        std::fs::write(bin_dir.join("worker-cell.md"), nested).unwrap();
        let skew = prompt_skew(root, "worker-cell");
        assert!(skew.is_some());
    }
}
