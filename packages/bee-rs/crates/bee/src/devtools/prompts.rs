// bee dev render-prompt — the prompt template loader/renderer.
//
// PROVENANCE (two sources, deliberately named together):
//   * packages/bee/lib/prompt-renderer.mjs — `loadPrompt` / `render`, the
//     original and the authority for every byte below.
//   * packages/bee-rs/crates/bee/src/verbs/drivers.rs — the FIRST Rust port
//     of the same two functions (its `normalize_template` / `load_prompt` /
//     `render`), landed with `bee dispatch prepare`.
//
// The drivers.rs port is module-private and drivers.rs is not a file this
// port may edit, so the grammar is RE-DERIVED here from the same .mjs rather
// than shared. Two ports of one contract need a pin, so this file carries
// three:
//
//   1. `prompt_renderer_mjs_matches_disk` — lib/prompt-renderer.mjs is
//      embedded at build time and byte-compared against the checkout. Edit
//      the .mjs and BOTH Rust ports are forced back to the diff (the same
//      embed-and-byte-compare technique hooks/write_guard.rs uses for the
//      vendored lib closure).
//   2. `c4_embedded_prompts_match_disk` — contract C4 restated: the four
//      templates compiled in here are the checked-out prompts/*.md bytes,
//      via the same include_str! paths drivers.rs uses.
//   3. `grammar_corpus_pins_both_ports` — the shared semantic corpus. Every
//      case and every refusal STRING below is what drivers.rs's `render`
//      produces for the same input; the two ports agree iff this corpus
//      passes in both, and it is written to be checkable by reading either
//      file against the .mjs.
//
// CLI shape (a new surface — the .mjs is a library, never a script, so there
// is no Node stdout framing to match):
//
//   bee dev render-prompt <name> [--var NAME=VALUE]...
//
// stdout is EXACTLY the rendered bytes, no added trailing newline, so a
// byte-diff against a `node -e "…loadPrompt/render…"` harness is a plain
// `cmp`.

use super::bee_source_root;
use std::process::ExitCode;

// ─── embedded templates (contract C4) ──────────────────────────────────────
// Same four include_str! paths verbs/drivers.rs uses, so both ports hash and
// render identical bytes.

const PROMPT_WORKER_CELL: &str = include_str!("../../../../../bee/prompts/worker-cell.md");
const PROMPT_GATHER: &str = include_str!("../../../../../bee/prompts/gather.md");
const PROMPT_REVIEWER: &str = include_str!("../../../../../bee/prompts/reviewer.md");
const PROMPT_ADVISOR: &str = include_str!("../../../../../bee/prompts/advisor.md");

/// The .mjs this port re-derives, embedded so a change to it turns this
/// module's test red instead of silently forking the grammar.
#[cfg(test)]
const PROMPT_RENDERER_MJS: &str = include_str!("../../../../../bee/lib/prompt-renderer.mjs");

const PROMPT_NAMES: [&str; 4] = ["advisor", "gather", "reviewer", "worker-cell"];

fn embedded_prompt(name: &str) -> Option<&'static str> {
    match name {
        "worker-cell" => Some(PROMPT_WORKER_CELL),
        "gather" => Some(PROMPT_GATHER),
        "reviewer" => Some(PROMPT_REVIEWER),
        "advisor" => Some(PROMPT_ADVISOR),
        _ => None,
    }
}

// ─── loadPrompt ────────────────────────────────────────────────────────────

/// provenance: prompt-renderer.mjs loadPrompt — CRLF normalized to LF and
/// ONE trailing newline stripped, so a checked-out template renders
/// byte-identically to the historical join('\n') string builders it replaced.
fn normalize_template(raw: &str) -> String {
    let lf = raw.replace("\r\n", "\n");
    match lf.strip_suffix('\n') {
        Some(s) => s.to_string(),
        None => lf,
    }
}

/// provenance: prompt-renderer.mjs PROMPTS_DIR — resolved RELATIVE TO THE
/// MODULE, i.e. `packages/bee/prompts/` in a canonical checkout. This dev
/// tool only ever runs in the source checkout, so that is the only arm; the
/// vendored `.bee/bin/prompts/` arm belongs to drivers.rs (which serves host
/// repos) and is deliberately absent here.
fn prompts_dir(root: &std::path::Path) -> std::path::PathBuf {
    root.join("packages").join("bee").join("prompts")
}

// ─── render ────────────────────────────────────────────────────────────────

/// provenance: prompt-renderer.mjs render(template, vars), both passes.
///
/// Pass 1 is JS `template.replace(/\n\{\{#if ([A-Za-z0-9_]+)\}\}([\s\S]*?)\n\{\{\/if\}\}/g, cb)`:
/// the match consumes the newline BEFORE the opening marker and the whole
/// `\n{{/if}}` line-start after it, so a dropped block leaves zero residue
/// bytes and a kept block splices `inner` (which still begins with its own
/// `\n`) in exactly. `[\s\S]*?` being lazy makes the block end at the FIRST
/// following `\n{{/if}}`. A `{{#if ` inside `inner` is a loud refusal, and so
/// is any `{{#if `/`{{/if}}` surviving the pass.
///
/// Pass 2 is `out.replace(/\{\{([A-Za-z0-9_]+)\}\}/g, cb)` over the
/// POST-block text: a substituted value is never re-scanned, and an
/// absent/null value is a loud refusal. A `{{…}}` whose name leaves
/// `[A-Za-z0-9_]+` is not a placeholder at all and passes through literally.
fn render(template: &str, vars: &[(String, String)]) -> Result<String, String> {
    let lookup = |name: &str| -> Option<&str> {
        vars.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    };

    // ── pass 1: conditional blocks ────────────────────────────────────────
    let mut out = String::with_capacity(template.len());
    let mut i = 0usize;
    while i < template.len() {
        let rest = &template[i..];
        let Some(after_open) = rest.strip_prefix("\n{{#if ") else {
            let ch = rest.chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };
        // `([A-Za-z0-9_]+)\}\}` — greedy name run that must land on `}}`.
        let matched = after_open.find("}}").and_then(|close| {
            let name = &after_open[..close];
            let valid = !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if !valid {
                return None;
            }
            let inner_start = close + 2;
            let end_rel = after_open[inner_start..].find("\n{{/if}}")?;
            Some((name, &after_open[inner_start..inner_start + end_rel], inner_start + end_rel))
        });
        let Some((name, inner, end_rel)) = matched else {
            let ch = rest.chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        };
        if inner.contains("{{#if ") {
            return Err(format!(
                "prompt-renderer: nested {{{{#if}}}} inside block \"{name}\" — nesting is not supported."
            ));
        }
        // JS truthiness of a string var: only a non-empty string is truthy;
        // an absent var is `undefined`, i.e. falsy.
        if lookup(name).map(|v| !v.is_empty()).unwrap_or(false) {
            out.push_str(inner);
        }
        i += "\n{{#if ".len() + end_rel + "\n{{/if}}".len();
    }
    if out.contains("{{#if ") || out.contains("{{/if}}") {
        return Err(
            "prompt-renderer: unmatched or malformed {{#if}}/{{/if}} marker in template.".to_string(),
        );
    }

    // ── pass 2: {{name}} substitution ─────────────────────────────────────
    let mut result = String::with_capacity(out.len());
    let mut i = 0usize;
    while i < out.len() {
        let rest = &out[i..];
        let hit = rest.strip_prefix("{{").and_then(|after| {
            let close = after.find("}}")?;
            let name = &after[..close];
            let valid = !name.is_empty()
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if valid { Some((name, close)) } else { None }
        });
        match hit {
            Some((name, close)) => {
                match lookup(name) {
                    Some(v) => result.push_str(v),
                    None => {
                        return Err(format!(
                            "prompt-renderer: no value supplied for placeholder {{{{{name}}}}}."
                        ))
                    }
                }
                i += 2 + close + 2;
            }
            None => {
                let ch = rest.chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    Ok(result)
}

// ─── CLI ───────────────────────────────────────────────────────────────────

fn usage() -> ExitCode {
    eprintln!(
        "usage: bee dev render-prompt <{}> [--var NAME=VALUE]...",
        PROMPT_NAMES.join("|")
    );
    ExitCode::FAILURE
}

pub(super) fn run(args: &[&str]) -> Option<ExitCode> {
    let Some(root) = bee_source_root() else { return None };

    let mut name: Option<&str> = None;
    let mut vars: Vec<(String, String)> = Vec::new();
    let mut i = 0usize;
    while i < args.len() {
        match args[i] {
            "--var" => {
                let Some(pair) = args.get(i + 1) else { return Some(usage()) };
                let Some((k, v)) = pair.split_once('=') else { return Some(usage()) };
                vars.push((k.to_string(), v.to_string()));
                i += 2;
            }
            other if other.starts_with("--") => return Some(usage()),
            other if name.is_none() => {
                name = Some(other);
                i += 1;
            }
            _ => return Some(usage()),
        }
    }
    let Some(name) = name else { return Some(usage()) };
    if embedded_prompt(name).is_none() {
        eprintln!("bee dev render-prompt: unknown prompt \"{name}\" (known: {})", PROMPT_NAMES.join(", "));
        return Some(ExitCode::FAILURE);
    }

    // loadPrompt reads from DISK; the embedded copy is the C4 pin, not the
    // source of truth for this tool. A checkout whose template drifted from
    // the binary renders what the checkout says — and the C4 test is what
    // makes that drift loud.
    let file = prompts_dir(&root).join(format!("{name}.md"));
    let raw = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("bee dev render-prompt: cannot read {}: {e}", file.display());
            return Some(ExitCode::FAILURE);
        }
    };
    let template = normalize_template(&String::from_utf8_lossy(&raw));

    match render(&template, &vars) {
        Ok(text) => {
            use std::io::Write;
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(text.as_bytes());
            let _ = stdout.flush();
            Some(ExitCode::SUCCESS)
        }
        Err(message) => {
            eprintln!("bee dev render-prompt: {message}");
            Some(ExitCode::FAILURE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        // CARGO_MANIFEST_DIR = packages/bee-rs/crates/bee
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("..")
    }

    // ── pin 1: the .mjs this port re-derives ──────────────────────────────
    #[test]
    fn prompt_renderer_mjs_matches_disk() {
        let disk = std::fs::read(
            repo_root().join("packages").join("bee").join("lib").join("prompt-renderer.mjs"),
        )
        .expect("lib/prompt-renderer.mjs readable");
        assert_eq!(
            String::from_utf8_lossy(&disk),
            PROMPT_RENDERER_MJS,
            "lib/prompt-renderer.mjs changed — re-derive BOTH Rust ports \
             (devtools/prompts.rs and verbs/drivers.rs) against the new grammar"
        );
    }

    // ── pin 2: contract C4 ────────────────────────────────────────────────
    #[test]
    fn c4_embedded_prompts_match_disk() {
        for name in PROMPT_NAMES {
            let disk = std::fs::read(
                repo_root()
                    .join("packages")
                    .join("bee")
                    .join("prompts")
                    .join(format!("{name}.md")),
            )
            .unwrap_or_else(|e| panic!("prompts/{name}.md readable: {e}"));
            assert_eq!(
                String::from_utf8_lossy(&disk),
                embedded_prompt(name).unwrap(),
                "C4: compiled-in prompts/{name}.md is stale — rebuild"
            );
        }
    }

    // ── pin 3: the shared grammar corpus (this port vs verbs/drivers.rs) ──
    #[test]
    fn grammar_corpus_pins_both_ports() {
        let v = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
            pairs.iter().map(|(k, x)| (k.to_string(), x.to_string())).collect()
        };

        // plain substitution
        assert_eq!(render("a {{x}} b", &v(&[("x", "Q")])).unwrap(), "a Q b");

        // a substituted value is NEVER re-scanned
        assert_eq!(
            render("{{x}}", &v(&[("x", "{{y}}"), ("y", "no")])).unwrap(),
            "{{y}}"
        );

        // kept block: the leading newline rides along with `inner`
        assert_eq!(
            render("head\n{{#if f}}\nbody\n{{/if}}\ntail", &v(&[("f", "1")])).unwrap(),
            "head\nbody\ntail"
        );
        // dropped block: zero residue bytes
        assert_eq!(
            render("head\n{{#if f}}\nbody\n{{/if}}\ntail", &v(&[("f", "")])).unwrap(),
            "head\ntail"
        );
        // an absent var is undefined -> falsy
        assert_eq!(
            render("head\n{{#if f}}\nbody\n{{/if}}\ntail", &[]).unwrap(),
            "head\ntail"
        );
        // block content is still substituted after splicing
        assert_eq!(
            render("h\n{{#if f}}\n{{x}}\n{{/if}}\nt", &v(&[("f", "y"), ("x", "V")])).unwrap(),
            "h\nV\nt"
        );

        // lazy inner: the block ends at the FIRST following `\n{{/if}}`
        assert_eq!(
            render("a\n{{#if f}}\n1\n{{/if}}\nmid\n{{#if f}}\n2\n{{/if}}\nz", &v(&[("f", "1")]))
                .unwrap(),
            "a\n1\nmid\n2\nz"
        );

        // a marker at byte 0 has no preceding \n, so it is NOT a block —
        // it survives pass 1 and trips the unmatched-marker refusal.
        assert_eq!(
            render("{{#if f}}\nx\n{{/if}}", &v(&[("f", "1")])).unwrap_err(),
            "prompt-renderer: unmatched or malformed {{#if}}/{{/if}} marker in template."
        );

        // nesting: loud refusal, exact wording
        assert_eq!(
            render("a\n{{#if o}}\n\n{{#if i}}\nx\n{{/if}}\n{{/if}}\n", &v(&[("o", "1"), ("i", "1")]))
                .unwrap_err(),
            "prompt-renderer: nested {{#if}} inside block \"o\" — nesting is not supported."
        );

        // missing placeholder: loud refusal, exact wording
        assert_eq!(
            render("a {{missing}}", &[]).unwrap_err(),
            "prompt-renderer: no value supplied for placeholder {{missing}}."
        );

        // a name outside [A-Za-z0-9_]+ is not a placeholder — literal passthrough
        assert_eq!(render("{{a-b}}", &[]).unwrap(), "{{a-b}}");
        // …and not a block opener either (the `}}`-run check fails)
        assert_eq!(
            render("x\n{{#if a-b}}\ny\n{{/if}}\nz", &[]).unwrap_err(),
            "prompt-renderer: unmatched or malformed {{#if}}/{{/if}} marker in template."
        );
    }

    #[test]
    fn load_prompt_normalization_matches_node() {
        assert_eq!(normalize_template("a\r\nb\r\n"), "a\nb");
        // exactly ONE trailing newline is stripped
        assert_eq!(normalize_template("a\n\n"), "a\n");
        assert_eq!(normalize_template("a"), "a");
    }

    /// Every real template renders with the vars dispatch-prepare supplies —
    /// the round-trip that C4 actually protects.
    #[test]
    fn real_templates_render_end_to_end() {
        for name in PROMPT_NAMES {
            let template = normalize_template(embedded_prompt(name).unwrap());
            // Collect every placeholder/block name the template mentions and
            // supply a marker value for each, so a refusal here means the
            // grammar diverged rather than the fixture being short.
            let mut vars: Vec<(String, String)> = Vec::new();
            let mut rest = template.as_str();
            while let Some(p) = rest.find("{{") {
                let after = &rest[p + 2..];
                let Some(close) = after.find("}}") else { break };
                let raw = &after[..close];
                let key = raw.strip_prefix("#if ").unwrap_or(raw);
                if !key.is_empty()
                    && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !vars.iter().any(|(k, _)| k == key)
                {
                    vars.push((key.to_string(), format!("<{key}>")));
                }
                rest = &after[close + 2..];
            }
            let rendered = render(&template, &vars)
                .unwrap_or_else(|e| panic!("prompts/{name}.md failed to render: {e}"));
            assert!(!rendered.contains("{{"), "prompts/{name}.md left a marker unrendered");
            // With every block var falsy the template must still render.
            let empty: Vec<(String, String)> =
                vars.iter().map(|(k, _)| (k.clone(), String::new())).collect();
            render(&template, &empty)
                .unwrap_or_else(|e| panic!("prompts/{name}.md failed to render (falsy): {e}"));
        }
    }
}
