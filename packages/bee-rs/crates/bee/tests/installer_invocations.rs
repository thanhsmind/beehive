// Every bee command an installer invokes must exist.
//
// WHY. 2.1.0 replaced a `node -e` version read with `"$BEE_BIN" --version`.
// `bee --version` is not a command — it exits 1 — and install.sh runs under
// `set -euo pipefail`, so that one command substitution killed the installer
// outright, with no message, immediately after a multi-minute build. Every
// 2.1.x install died there, on every platform, and three releases shipped that
// way. CI never noticed because CI runs the test suite and never the installer.
//
// The registry knows every command bee answers. The installers are the only two
// scripts that drive it from outside. Comparing the two is cheap, and it is the
// exact question the failure asked.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(4).unwrap().to_path_buf()
}

fn read(rel: &str) -> String {
    let p = repo_root().join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

/// Every `invoke` string the registry declares, e.g. "bee cells finish".
fn declared_invocations() -> BTreeSet<String> {
    let raw = include_str!("../src/generated/registry_payload.json");
    let doc: serde_json::Value = serde_json::from_str(raw).expect("registry parses");
    doc["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|c| c["invoke"].as_str())
        .map(|s| s.trim_start_matches("bee ").to_string())
        .collect()
}

/// The variables both installers hold a bee binary in.
/// Paired with how that shell CALLS a variable. PowerShell needs the `&` call
/// operator; a bare `$beeBin` at the start of a line is an assignment, not an
/// invocation, and treating it as one is what made the first version of this
/// guard report `bee = Join-Path`.
const BINARY_VARS: [(&str, bool); 4] = [
    ("\"$BEE_BIN\"", false),
    ("\"$HOST_BEE\"", false),
    ("$beeBin", true),
    ("$hostBee", true),
];

/// Is the binary in COMMAND position here, rather than being assigned or tested?
///
/// The first version of this asked only whether the variable appeared on the
/// line, and reported `bee ]; then` from `if [ -z "$BEE_BIN" ]; then` — a guard
/// whose own output tells you it is not reading what it claims to read.
fn is_command_position(prefix: &str, needs_call_operator: bool) -> bool {
    let p = prefix.trim_end();
    if needs_call_operator {
        return p.ends_with('&');
    }
    p.is_empty()
        || p.ends_with("&&")
        || p.ends_with("||")
        || p.ends_with(';')
        || p.ends_with('|')
        || p.ends_with('(')
        || p.ends_with("$(")
}

/// Pull `<verb...>` out of every `<binary-var> <verb...>` call site, stopping at
/// the first token that is a flag, a variable, or shell punctuation.
fn invoked_verbs(script: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (i, line) in script.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with('#') {
            continue;
        }
        for (var, needs_call_operator) in BINARY_VARS {
            let Some(pos) = t.find(var) else { continue };
            if !is_command_position(&t[..pos], needs_call_operator) {
                continue;
            }
            let rest = &t[pos + var.len()..];
            let mut tokens = rest.split_whitespace().peekable();
            // A call whose FIRST token is a flag has no verb at all — and that
            // is exactly the shape that broke: `"$BEE_BIN" --version`. Skipping
            // it as "no verb found" is how the first version of this guard
            // passed on the very defect it was written for.
            if let Some(first) = tokens.peek() {
                if first.starts_with('-') {
                    found.push((i + 1, (*first).to_string()));
                    continue;
                }
            }
            let mut verb: Vec<&str> = Vec::new();
            for tok in rest.split_whitespace() {
                let stop = tok.starts_with('-')
                    || tok.starts_with('$')
                    || tok.starts_with('"')
                    || tok.starts_with('@')
                    || tok.starts_with('|')
                    || tok.starts_with(')')
                    || tok.starts_with('>');
                if stop {
                    break;
                }
                verb.push(tok);
            }
            if !verb.is_empty() {
                found.push((i + 1, verb.join(" ")));
            }
        }
    }
    found
}

#[test]
fn every_bee_command_the_installers_call_is_a_real_command() {
    let declared = declared_invocations();
    let mut offenders: Vec<String> = Vec::new();

    for rel in ["scripts/install.sh", "scripts/install.ps1"] {
        let script = read(rel);
        for (line, verb) in invoked_verbs(&script) {
            // A declared command may take trailing words the registry does not
            // spell out, so accept any declared invocation that PREFIXES this
            // call — `cells finish` covers `cells finish --id x`.
            // `bee` itself answers exactly one bare flag. Everything else that
            // looks like `bee --something` is an assumption about a surface
            // that does not exist.
            let known = if verb.starts_with('-') {
                verb == "--help"
            } else {
                declared.iter().any(|d| verb == *d || verb.starts_with(&format!("{d} ")))
            };
            if !known {
                offenders.push(format!("{rel}:{line} — `bee {verb}`"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "the installers call bee commands that do not exist. Under `set -euo pipefail` a call \
         like this kills install.sh with no message at all — which is how `bee --version` \
         silently broke every 2.1.x install:\n  {}\n\nFIX: use a declared command \
         (`bee --help --all --json` lists them), or read the value without the binary.",
        offenders.join("\n  ")
    );
}

/// The version the installer reports must come from the manifest that defines
/// it, not from a second source that can disagree — and never from a command
/// whose existence is an assumption.
#[test]
fn the_installer_reads_its_version_from_the_plugin_manifest() {
    let text = read("scripts/install.sh");
    let line = text
        .lines()
        .find(|l| !l.trim_start().starts_with('#') && l.contains("BEE_VERSION=\"$("))
        .expect("install.sh must compute BEE_VERSION");
    assert!(
        line.contains("plugin.json") || text.contains("\"$BEE_SRC/.claude-plugin/plugin.json\""),
        "BEE_VERSION must be read out of .claude-plugin/plugin.json — the file that defines it \
         and the one `version.rs` treats as the single source of truth. Got:\n  {}",
        line.trim()
    );
}
