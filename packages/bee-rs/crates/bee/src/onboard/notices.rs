// onboard::notices — the four advisory notice sources, plus the
// lib/commands_detect.mjs port they lean on.
//
// Provenance: onboard_bee.mjs commandsNotices (l. 2594), staleAdvisorNotices
// (l. 2645), trackedGitignorePaths (l. 2665), trackedPathsNotices (l. 2678)
// and repoHooksTransitionNotices (l. 3512) — plus lib/commands_detect.mjs
// detectCommands, which this script imports.

use super::plan::verify_source_root;
use super::templates::{
    COMMAND_KEYS, DEPLOY_DRIVES_NO_PRODUCT_NOTICE, DESIGN_PACKS_OFFER, GITIGNORE_BLOCK_PATTERNS, NO_TEST_VERIFICATION_OFFER,
    RETIRED_VERIFY_KEY_NO_TEST_WARNING, RETIRED_VERIFY_KEY_WARNING, STALE_ADVISOR_KEY_WARNING,
    STATIC_CHECK_FIRST_OFFER, STATIC_CHECK_HOW_TO_OFFER, STATIC_CHECK_MISSING_OFFER,
    TESTED_REPO_VERIFICATION_OFFER, VERIFICATION_UPKEEP_POINTER, VERIFY_APP_SKILL_NAME,
};
use super::util::{exists, read_dir_sorted, read_json_if_exists, read_text_if_exists, split_lines};
use crate::verbs::protocol::{gather_facts, protocol_state, render_protocol, ProtocolState, PROTOCOL_FILE};
use serde_json::{json, Value};
use std::path::Path;

// ── lib/commands_detect.mjs ────────────────────────────────────────────────

pub struct Candidate {
    pub key: &'static str,
    pub value: String,
    pub source: String,
}

fn read_manifest_json(file: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()
}

/// scriptNames: keys of a `scripts` object whose value is a non-empty string
/// or a non-empty array (composer allows script arrays).
fn script_names(manifest: Option<Value>) -> Vec<String> {
    let Some(Value::Object(scripts)) =
        manifest.as_ref().filter(|m| m.is_object()).and_then(|m| m.get("scripts")).cloned()
    else {
        return Vec::new();
    };
    scripts
        .iter()
        .filter(|(_, v)| match v {
            Value::String(s) => !s.trim().is_empty(),
            Value::Array(a) => !a.is_empty(),
            _ => false,
        })
        .map(|(k, _)| k.clone())
        .collect()
}

/// makefileTargets: names declared at column 0 (never recipe bodies) —
/// `/^([A-Za-z0-9._-]+)\s*:(?!=)/`, skipping `.`-prefixed targets.
fn makefile_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in split_lines(text) {
        if line.starts_with('\t') || line.starts_with(' ') {
            continue;
        }
        let name_len = line
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'))
            .unwrap_or(line.len());
        if name_len == 0 {
            continue;
        }
        let (name, rest) = line.split_at(name_len);
        let rest = rest.trim_start_matches(char::is_whitespace);
        if !rest.starts_with(':') || rest.starts_with(":=") {
            continue;
        }
        if !name.starts_with('.') {
            targets.push(name.to_string());
        }
    }
    targets
}

fn first_csproj(root: &Path) -> Option<String> {
    let mut matches: Vec<String> = read_dir_sorted(root)
        .into_iter()
        .filter(|e| e.is_file && e.name.to_lowercase().ends_with(".csproj"))
        .map(|e| e.name)
        .collect();
    matches.sort();
    matches.into_iter().next()
}

/// detectCommands(root): at most one candidate per key. Priority when
/// sources conflict: package.json, Makefile, composer.json, then conventions.
pub fn detect_commands(root: &Path) -> Vec<Candidate> {
    let mut by_key: Vec<(&'static str, Candidate)> = Vec::new();
    let push = |c: Candidate, by_key: &mut Vec<(&'static str, Candidate)>| {
        if !by_key.iter().any(|(k, _)| *k == c.key) {
            by_key.push((c.key, c));
        }
    };

    let pkg = script_names(read_manifest_json(&root.join("package.json")));
    for key in COMMAND_KEYS {
        if pkg.iter().any(|n| n == key) {
            let value =
                if *key == "test" { "npm test".to_string() } else { format!("npm run {key}") };
            push(Candidate { key, value, source: "package.json".into() }, &mut by_key);
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("Makefile")) {
        let targets = makefile_targets(&text);
        for key in COMMAND_KEYS {
            if targets.iter().any(|n| n == key) {
                push(
                    Candidate { key, value: format!("make {key}"), source: "Makefile".into() },
                    &mut by_key,
                );
            }
        }
    }
    let composer = script_names(read_manifest_json(&root.join("composer.json")));
    for key in COMMAND_KEYS {
        if composer.iter().any(|n| n == key) {
            push(
                Candidate { key, value: format!("composer {key}"), source: "composer.json".into() },
                &mut by_key,
            );
        }
    }

    // CONVENTION_SOURCES, in order.
    let conventions: Vec<(&'static str, &'static str, Option<String>)> = vec![
        ("test", "pytest", exists(&root.join("pyproject.toml")).then(|| "pyproject.toml".into())),
        ("test", "dotnet test", first_csproj(root)),
        ("test", "go test ./...", exists(&root.join("go.mod")).then(|| "go.mod".into())),
    ];
    for (key, value, marker) in conventions {
        if by_key.iter().any(|(k, _)| *k == key) {
            continue;
        }
        if let Some(marker) = marker {
            push(Candidate { key, value: value.into(), source: marker }, &mut by_key);
        }
    }

    // Emitted in COMMAND_KEYS order, not discovery order.
    COMMAND_KEYS
        .iter()
        .filter_map(|key| {
            by_key.iter().position(|(k, _)| k == key).map(|i| {
                let (_, c) = &by_key[i];
                Candidate { key: c.key, value: c.value.clone(), source: c.source.clone() }
            })
        })
        .collect()
}

pub struct StaticCheck {
    pub command: String,
    pub source: String,
    pub marker: String,
}

fn package_runner(root: &Path, name: &str) -> String {
    if exists(&root.join("pnpm-lock.yaml")) {
        format!("pnpm run {name}")
    } else if exists(&root.join("yarn.lock")) {
        format!("yarn {name}")
    } else if exists(&root.join("bun.lockb")) || exists(&root.join("bun.lock")) {
        format!("bun run {name}")
    } else {
        format!("npm run {name}")
    }
}

fn static_check_for(root: &Path, names: &[&str], conventions: &[(&str, &str, &str)]) -> Option<StaticCheck> {
    let pkg = script_names(read_manifest_json(&root.join("package.json")));
    if let Some(name) = names.iter().find(|n| pkg.iter().any(|p| p == *n)) {
        return Some(StaticCheck {
            command: package_runner(root, name),
            source: "package.json".into(),
            marker: (*name).into(),
        });
    }
    let targets = std::fs::read_to_string(root.join("Makefile"))
        .map(|t| makefile_targets(&t))
        .unwrap_or_default();
    if let Some(name) = names.iter().find(|n| targets.iter().any(|t| t == *n)) {
        return Some(StaticCheck {
            command: format!("make {name}"),
            source: "Makefile".into(),
            marker: (*name).into(),
        });
    }
    if names.contains(&"lint") {
        let composer = script_names(read_manifest_json(&root.join("composer.json")));
        if composer.iter().any(|n| n == "lint") {
            return Some(StaticCheck {
                command: "composer lint".into(),
                source: "composer.json".into(),
                marker: "lint".into(),
            });
        }
    }
    for (file, table, command_marker) in conventions {
        let (command, marker) = command_marker.split_once('|').unwrap_or((command_marker, ""));
        let text = std::fs::read_to_string(root.join(file)).ok();
        let hit = match text {
            Some(t) => table.is_empty() || t.contains(table),
            None => false,
        };
        if hit {
            return Some(StaticCheck {
                command: command.into(),
                source: (*file).into(),
                marker: marker.into(),
            });
        }
    }
    None
}

pub fn detect_static_checks(root: &Path) -> Vec<StaticCheck> {
    [
        static_check_for(
            root,
            &["lint"],
            &[
                ("Cargo.toml", "", "cargo clippy -- -D warnings|clippy"),
                ("pyproject.toml", "[tool.ruff", "ruff check .|ruff"),
                ("go.mod", "", "go vet ./...|vet"),
            ],
        ),
        static_check_for(
            root,
            &["typecheck", "type-check"],
            &[("pyproject.toml", "[tool.mypy", "mypy .|mypy")],
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn test_entries(commands: Option<&serde_json::Map<String, Value>>) -> Vec<String> {
    match commands.and_then(|c| c.get("test")) {
        Some(Value::String(s)) if !s.trim().is_empty() => vec![s.clone()],
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn static_check_list(checks: &[&StaticCheck]) -> String {
    checks.iter().map(|c| format!("{} — {}", c.command, c.source)).collect::<Vec<_>>().join("; ")
}

// ── notices ────────────────────────────────────────────────────────────────

/// host-packaging-gaps D2: name the two keys the one-time `add_pi_team` item
/// adds to an existing `.bee/config.json` (planned or applied — `items` is
/// whichever list this run reports).
pub fn pi_team_notices(items: &[Value]) -> Vec<String> {
    items
        .iter()
        .filter(|i| i["action"] == "add_pi_team")
        .map(|i| {
            let table = i["table"].as_str().unwrap_or("team");
            format!(
                "Pi role table: .bee/config.json gains {table}.pi (every role a herding slot on \
agent \"pi\") and herding.agents.pi = [\"pi\"] (your own Pi default model), each only if absent. \
Onboarding adds them once; delete them to opt out and they stay deleted."
            )
        })
        .collect()
}

pub fn claude_settings_notices(
    items: &[Value],
    kept: &super::hooks_wiring::ClaudeSettingsKept,
    full: bool,
) -> Vec<String> {
    if !full || !items.iter().any(|i| i["action"] == "merge_claude_settings") {
        return Vec::new();
    }
    let mut line = String::from(
        "Claude settings baseline: permission prompts are turned off for this project \
(permissions.defaultMode bypassPermissions) and the baseline tools are denied. The baseline \
is written even when bee's guard hooks are not wired, so on a --no-hooks install bee's guards \
are not the safety net behind it.",
    );
    if !kept.kept_keys.is_empty() {
        line.push_str(&format!(
            " Left at this project's own value: {}.",
            kept.kept_keys.join(", ")
        ));
    }
    if !kept.allowed_then_denied.is_empty() {
        line.push_str(&format!(
            " Your allow list still carries {}, which the baseline also denies.",
            kept.allowed_then_denied.join(", ")
        ));
    }
    line.push_str(
        " The write is add-only: no value this project already set is overwritten. Opt out with \
`bee onboard --no-claude-settings`, or for good with \
`.bee/config.json` \"claude_settings\": false.",
    );
    vec![line]
}

/// commandsNotices (l. 2594): propose-only (decision D3) — this script never
/// writes detected values to .bee/config.json.
pub fn commands_notices(repo_root: &Path, first_onboard: bool) -> Vec<String> {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let raw = config.as_ref().and_then(|c| c.get("commands")).filter(|c| c.is_object()).cloned();
    let commands_map = raw.as_ref().and_then(Value::as_object);
    let entries = test_entries(commands_map);
    let recorded = !entries.is_empty()
        || ["setup", "start"].iter().any(|key| {
            raw.as_ref()
                .and_then(|r| r.get(*key))
                .and_then(Value::as_str)
                .is_some_and(|v| !v.trim().is_empty())
        });
    let checks = detect_static_checks(repo_root);
    if !entries.is_empty() {
        if entries.iter().any(|e| e.trim() == "none") {
            return Vec::new();
        }
        let missing: Vec<&StaticCheck> = checks
            .iter()
            .filter(|c| !entries.iter().any(|e| e.to_lowercase().contains(&c.marker.to_lowercase())))
            .collect();
        if missing.is_empty() {
            return Vec::new();
        }
        return vec![format!(
            "{} {}",
            STATIC_CHECK_MISSING_OFFER.replace("{checks}", &static_check_list(&missing)),
            STATIC_CHECK_HOW_TO_OFFER
        )];
    }
    if recorded {
        return Vec::new();
    }
    let static_sentence = if checks.is_empty() {
        String::new()
    } else {
        let all: Vec<&StaticCheck> = checks.iter().collect();
        format!(
            " {} {}",
            STATIC_CHECK_FIRST_OFFER.replace("{checks}", &static_check_list(&all)),
            STATIC_CHECK_HOW_TO_OFFER
        )
    };
    let candidates = detect_commands(repo_root);
    if !candidates.is_empty() {
        let proposals: Vec<String> = candidates
            .iter()
            .map(|c| format!("{}: {} — {}", c.key, c.value, c.source))
            .collect();
        return vec![format!(
            "No standard commands recorded. Detected candidates: {}. Present them to the user as one pre-filled confirmation question (skippable) and write only confirmed values to .bee/config.json `commands` — never write unconfirmed values (D3). They power the session CI status gate.{}",
            proposals.join("; "),
            static_sentence
        )];
    }
    let mut notices = vec![
        format!("No standard commands recorded. Ask the user for the host project's setup/start/test commands and write them to .bee/config.json `commands` (skippable — never invent values). They power the session CI status gate.{static_sentence}"),
    ];
    if first_onboard {
        notices.push("Greenfield init lane (docs/09 item 6): this is the first onboard and no build was detected. Offer the init lane before any feature work — the first planning slice is one init cell whose must_haves are exactly: setup succeeds from scratch, one passing test exists, standard commands are recorded in .bee/config.json, and the repo has a clean first commit.".to_string());
    }
    notices
}

/// staleAdvisorNotices (l. 2645): warn, never error.
///
/// Also carries the `commands.verify` retirement notice (2.1.0). Same shape,
/// same never-error contract — and it is not cosmetic: a host that recorded
/// ONLY `commands.verify` used to have a merge gate and now has none, and a
/// host that declared itself no-test with `verify: "none"` no longer does.
/// Both failures are silent without this line, which is exactly the class of
/// break a warning exists for.
pub fn stale_advisor_notices(repo_root: &Path) -> Vec<String> {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let obj = config.as_ref().and_then(Value::as_object);
    let mut notices = Vec::new();
    if obj.is_some_and(|o| o.contains_key("advisor")) {
        notices.push(STALE_ADVISOR_KEY_WARNING.to_string());
    }
    let commands = obj.and_then(|o| o.get("commands")).and_then(Value::as_object);
    let has_test = declares_test(commands);
    // ARM ORDER, deliberate and asserted: the retired `commands.verify` key
    // is read FIRST and keeps every case it answered before. A host carrying
    // it lost its test gate and must delete the key; that instruction outranks
    // anything about verification, and stacking a second notice beside it
    // would only compete for the reader's one next action. Exactly one arm
    // fires, so no repo ever draws two of these notices.
    if commands.is_some_and(|c| c.contains_key("verify")) {
        notices.push(if has_test {
            RETIRED_VERIFY_KEY_WARNING.to_string()
        } else {
            RETIRED_VERIFY_KEY_NO_TEST_WARNING.to_string()
        });
    } else if exists(&verify_app_skill_file(repo_root)) {
        // Has the skill already: point at its upkeep, never re-offer it
        // (verification-in-the-flow D3).
        notices.push(VERIFICATION_UPKEEP_POINTER.to_string());
    } else {
        // Has no verification skill: offer to build one. The wording splits
        // on the test command, but the ARM never does - a test command asks
        // whether the code stayed correct, which is a different question from
        // whether the product works, so a tested repo needs this offer too.
        notices.push(if has_test {
            TESTED_REPO_VERIFICATION_OFFER.to_string()
        } else {
            NO_TEST_VERIFICATION_OFFER.to_string()
        });
    }
    if deploy_drives_no_product(repo_root) {
        notices.push(DEPLOY_DRIVES_NO_PRODUCT_NOTICE.to_string());
    }
    if design_offer_due(repo_root) {
        notices.push(DESIGN_PACKS_OFFER.to_string());
    }
    notices
}

fn deploy_drives_no_product(repo_root: &Path) -> bool {
    let bodies: Vec<String> = ["docs/DEPLOY.md", "scripts/deploy.sh"]
        .iter()
        .filter_map(|rel| std::fs::read(repo_root.join(rel)).ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .collect();
    bodies.iter().any(|body| !body.contains(".bee/verify/verify-app/"))
}

pub fn workspace_protocol_notice(repo_root: &Path) -> (Value, Option<String>) {
    let report = protocol_state(repo_root);
    let unedited = report.state == ProtocolState::Valid
        && std::fs::read(repo_root.join(PROTOCOL_FILE))
            .map(|bytes| bytes == render_protocol(&gather_facts(repo_root)).into_bytes())
            .unwrap_or(false);
    let action_needed = report.state != ProtocolState::Valid || unedited;
    let what = match report.state {
        ProtocolState::Missing => "missing".to_string(),
        ProtocolState::Unreadable => "unreadable".to_string(),
        ProtocolState::Invalid => format!("invalid ({})", report.issues.join("; ")),
        ProtocolState::Valid => "still bee's unedited render".to_string(),
    };
    let notice = action_needed.then(|| {
        format!(
            "workspace protocol: {PROTOCOL_FILE} is {what} — finish it now with the Workspace protocol procedure in bee-hive references/onboarding.md"
        )
    });
    let value = json!({
        "state": report.state.as_str(),
        "unedited": unedited,
        "issues": report.issues,
        "action_needed": action_needed,
    });
    (value, notice)
}

pub fn design_offer_due(repo_root: &Path) -> bool {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let picked = config
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|o| o.get("design"))
        .and_then(|d| d.get("packs"))
        .is_some();
    !picked && has_ui_file(repo_root)
}

const UI_FILE_EXTENSIONS: &[&str] = &["html", "css", "scss", "jsx", "tsx", "vue", "svelte"];
const UI_SCAN_SKIPPED_DIRS: &[&str] = &["node_modules", "target", "dist"];
const UI_SCAN_MAX_DEPTH: usize = 6;
const UI_SCAN_MAX_ENTRIES: usize = 10_000;

fn has_ui_file(repo_root: &Path) -> bool {
    let mut queue = std::collections::VecDeque::from([(repo_root.to_path_buf(), 0usize)]);
    let mut seen = 0usize;
    while let Some((dir, depth)) = queue.pop_front() {
        for entry in read_dir_sorted(&dir) {
            seen += 1;
            if seen > UI_SCAN_MAX_ENTRIES {
                return false;
            }
            if entry.is_dir {
                let skipped = entry.name.starts_with('.') || UI_SCAN_SKIPPED_DIRS.contains(&entry.name.as_str());
                if depth < UI_SCAN_MAX_DEPTH && !skipped {
                    queue.push_back((dir.join(&entry.name), depth + 1));
                }
            } else if entry.is_file
                && Path::new(&entry.name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| UI_FILE_EXTENSIONS.contains(&e))
            {
                return true;
            }
        }
    }
    false
}

/// The ONE path that answers "does this repo have a verification skill": its
/// SOURCE file (verification-in-the-flow D8), never a rendered runtime copy.
/// A fresh clone has the source before it has ever run `--apply`, so reading a
/// rendered home would branch on clone state instead of on what the repo has.
fn verify_app_skill_file(repo_root: &Path) -> std::path::PathBuf {
    verify_source_root(repo_root).join(VERIFY_APP_SKILL_NAME).join("SKILL.md")
}

/// The ONE definition of "this repo declares a test command": a non-empty
/// string, or an array (the multi-command list shape). The retired-verify
/// warnings and the verification offer read it through this function so the
/// two can never disagree about what "declared" means.
fn declares_test(commands: Option<&serde_json::Map<String, Value>>) -> bool {
    commands
        .and_then(|c| c.get("test"))
        .is_some_and(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()) || v.is_array())
}

/// trackedGitignorePaths (l. 2665): `git ls-files -z -- <patterns>` with an
/// argv array (never a shell string). Degrades to silence on any failure.
fn tracked_gitignore_paths(repo_root: &Path) -> Vec<String> {
    use std::process::{Command, Stdio};
    let out = Command::new("git")
        .arg("ls-files")
        .arg("-z")
        .arg("--")
        .args(GITIGNORE_BLOCK_PATTERNS)
        .current_dir(repo_root)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else { return Vec::new() };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// trackedPathsNotices (l. 2678).
pub fn tracked_paths_notices(repo_root: &Path) -> Vec<String> {
    let tracked = tracked_gitignore_paths(repo_root);
    if tracked.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "{} managed path(s) are still git-tracked; the ignore block cannot silence them — run: git rm -r --cached {}",
        tracked.len(),
        tracked.join(" ")
    )]
}

/// hasRepoHooksRecorded (l. 3460): the sticky --repo-hooks opt-in.
pub fn has_repo_hooks_recorded(repo_root: &Path) -> bool {
    let text = read_text_if_exists(&repo_root.join(".bee").join("onboarding.json"));
    let Ok(parsed) = serde_json::from_str::<Value>(&text) else { return false };
    parsed
        .get("managed")
        .and_then(|m| m.get("repo_hooks"))
        .and_then(Value::as_object)
        .is_some_and(|o| !o.is_empty())
}

/// repoHooksTransitionNotices (l. 3512).
pub fn repo_hooks_transition_notices(
    repo_root: &Path,
    plugin_source: bool,
    codex_hybrid: bool,
) -> Vec<String> {
    if !plugin_source || !has_repo_hooks_recorded(repo_root) {
        return Vec::new();
    }
    if codex_hybrid {
        return vec!["This repo previously opted into --repo-hooks (full repo-local Claude + Codex hook wiring). Onboarding as --plugin-source retires the repo-local Claude entries in .claude/settings.json (Claude's own plugin hooks take over) and keeps Codex mechanically enforced through the codex-hybrid .codex/hooks.json projection instead — no action needed.".to_string()];
    }
    vec!["This repo previously opted into --repo-hooks (full repo-local Claude + Codex hook wiring). Onboarding as --plugin-source with --runtime claude retires ALL repo-local hook entries, including Codex's — pass --runtime codex or --runtime both to keep Codex mechanically enforced via the codex-hybrid path, or use --distribution repo-copy to keep the full repo-local install as-is.".to_string()]
}

/// composeAgentsHeader (l. 2213): mechanically provable parts only.
pub fn compose_agents_header(repo_root: &Path) -> String {
    let mut lines: Vec<String> = vec![
        format!("# {}", super::util::basename(repo_root)),
        String::new(),
        "<!-- [unknown] one-line project description - replace me -->".to_string(),
    ];
    let pointers: Vec<&str> = super::templates::HEADER_POINTER_CANDIDATES
        .iter()
        .copied()
        .filter(|rel| exists(&super::util::join_rel(repo_root, rel)))
        .collect();
    if !pointers.is_empty() {
        lines.push(String::new());
        for rel in pointers {
            lines.push(format!("- {rel}"));
        }
    }
    lines.push(String::new());
    format!("{}\n", lines.join("\n"))
}

/// hasProseOutsideBlock (l. 2200): the mechanical stand-in for "does this
/// answer what this project is?". Whitespace-only and comment-only lines
/// never count as prose; an UNCLOSED comment stays in place and counts.
pub fn has_prose_outside_block(text: &str) -> bool {
    let start = text.find(super::templates::MARKER_START);
    let end = text.find(super::templates::MARKER_END);
    let outside = match (start, end) {
        (Some(s), Some(e)) if e >= s => {
            format!("{}{}", &text[..s], &text[e + super::templates::MARKER_END.len()..])
        }
        _ => text.to_string(),
    };
    // `outside.replace(/<!--[\s\S]*?-->/g, "")` — non-greedy, multi-line.
    let mut stripped = String::new();
    let mut rest = outside.as_str();
    loop {
        match rest.find("<!--") {
            Some(open) => match rest[open..].find("-->") {
                Some(close_rel) => {
                    stripped.push_str(&rest[..open]);
                    rest = &rest[open + close_rel + 3..];
                }
                None => {
                    stripped.push_str(rest);
                    break;
                }
            },
            None => {
                stripped.push_str(rest);
                break;
            }
        }
    }
    stripped.split('\n').any(|line| !line.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn static_check_repo(files: &[(&str, &str)], config: Option<Value>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, body) in files {
            std::fs::write(dir.path().join(name), body).unwrap();
        }
        if let Some(config) = config {
            std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
            std::fs::write(dir.path().join(".bee").join("config.json"), config.to_string())
                .unwrap();
        }
        dir
    }

    const STATIC_CHECK_LINT_PKG: &str = r#"{"scripts":{"lint":"eslint .","typecheck":"tsc"}}"#;
    const STATIC_CHECK_OLD_FIRST: &str = "No standard commands recorded. Ask the user for the host project's setup/start/test commands and write them to .bee/config.json `commands` (skippable — never invent values). They power the session CI status gate.";

    #[test]
    fn static_check_one_candidate_per_kind_and_manifest_beats_convention() {
        let dir = static_check_repo(
            &[
                ("package.json", STATIC_CHECK_LINT_PKG),
                ("Cargo.toml", "[package]\n"),
                ("pyproject.toml", "[tool.ruff]\n[tool.mypy]\n"),
            ],
            None,
        );
        let got = detect_static_checks(dir.path());
        let pairs: Vec<(&str, &str, &str)> =
            got.iter().map(|c| (c.command.as_str(), c.source.as_str(), c.marker.as_str())).collect();
        assert_eq!(
            pairs,
            vec![("npm run lint", "package.json", "lint"), ("npm run typecheck", "package.json", "typecheck")]
        );
    }

    #[test]
    fn static_check_conventions_and_makefile_and_composer() {
        let dir = static_check_repo(&[("Cargo.toml", "[package]\n"), ("Makefile", "typecheck:\n\ttsc\n")], None);
        let got = detect_static_checks(dir.path());
        let pairs: Vec<(&str, &str, &str)> =
            got.iter().map(|c| (c.command.as_str(), c.source.as_str(), c.marker.as_str())).collect();
        assert_eq!(
            pairs,
            vec![
                ("cargo clippy -- -D warnings", "Cargo.toml", "clippy"),
                ("make typecheck", "Makefile", "typecheck")
            ]
        );
        let dir = static_check_repo(&[("composer.json", r#"{"scripts":{"lint":"phpcs"}}"#)], None);
        assert_eq!(detect_static_checks(dir.path())[0].command, "composer lint");
        let dir = static_check_repo(&[("pyproject.toml", "[tool.ruff.lint]\n[tool.mypy]\n"), ("go.mod", "module x\n")], None);
        let cmds: Vec<String> = detect_static_checks(dir.path()).into_iter().map(|c| c.command).collect();
        assert_eq!(cmds, vec!["ruff check .", "mypy ."]);
        let dir = static_check_repo(&[("go.mod", "module x\n")], None);
        assert_eq!(detect_static_checks(dir.path())[0].command, "go vet ./...");
    }

    #[test]
    fn static_check_lockfile_picks_the_runner() {
        for (lock, want) in [
            ("pnpm-lock.yaml", "pnpm run lint"),
            ("yarn.lock", "yarn lint"),
            ("bun.lockb", "bun run lint"),
            ("bun.lock", "bun run lint"),
        ] {
            let dir = static_check_repo(&[("package.json", STATIC_CHECK_LINT_PKG), (lock, "")], None);
            assert_eq!(detect_static_checks(dir.path())[0].command, want);
        }
        let dir = static_check_repo(&[("package.json", r#"{"scripts":{"type-check":"tsc"}}"#)], None);
        assert_eq!(detect_static_checks(dir.path())[0].command, "npm run type-check");
    }

    #[test]
    fn static_check_malformed_package_json_yields_nothing() {
        let dir = static_check_repo(&[("package.json", "{not json")], None);
        assert!(detect_static_checks(dir.path()).is_empty());
        let dir = static_check_repo(&[("package.json", "[1,2]")], None);
        assert!(detect_static_checks(dir.path()).is_empty());
    }

    #[test]
    fn static_check_case_a_adds_a_sentence_to_both_arms() {
        let dir = static_check_repo(&[("Cargo.toml", "[package]\n")], None);
        let n = commands_notices(dir.path(), true);
        assert_eq!(n.len(), 2);
        assert!(n[0].starts_with(STATIC_CHECK_OLD_FIRST));
        assert!(n[0].contains("cargo clippy -- -D warnings — Cargo.toml"));
        assert!(n[0].contains("first members of the `commands.test` list, before the tests"));
        assert!(n[0].contains("green or how many findings"));
        assert!(n[0].contains("earlier answer about that check"));
        assert!(n[0].contains("as a decision"));
        assert!(n[0].contains("Never write an unconfirmed value"));
        assert!(n[1].starts_with("Greenfield init lane"));

        let dir = static_check_repo(
            &[("package.json", r#"{"scripts":{"lint":"x","start":"node ."}}"#)],
            None,
        );
        let n = commands_notices(dir.path(), false);
        assert_eq!(n.len(), 1);
        assert!(n[0].contains("Detected candidates: start: npm run start — package.json."));
        assert!(n[0].contains("npm run lint — package.json"));
    }

    #[test]
    fn static_check_case_b_fires_for_a_string_and_a_list_lacking_the_marker() {
        for test in [json!("cargo test"), json!(["cargo test", "cargo fmt --check"])] {
            let dir = static_check_repo(
                &[("Cargo.toml", "[package]\n")],
                Some(json!({"commands": {"test": test}})),
            );
            let n = commands_notices(dir.path(), false);
            assert_eq!(n.len(), 1, "for {test}");
            assert!(n[0].contains("cargo clippy -- -D warnings — Cargo.toml"));
            assert!(n[0].contains("before the tests in the `commands.test` list"));
            assert!(n[0].contains("green or how many findings"));
            assert!(n[0].contains("as a decision"));
        }
    }

    #[test]
    fn static_check_case_b_is_silent_when_an_entry_has_the_marker() {
        let dir = static_check_repo(
            &[("Cargo.toml", "[package]\n")],
            Some(json!({"commands": {"test": ["cargo clippy --all-targets -- -D warnings", "cargo test"]}})),
        );
        assert!(commands_notices(dir.path(), false).is_empty());
        let dir = static_check_repo(
            &[("package.json", STATIC_CHECK_LINT_PKG)],
            Some(json!({"commands": {"test": ["pnpm lint", "pnpm typecheck", "pnpm test"]}})),
        );
        assert!(commands_notices(dir.path(), false).is_empty());
        let dir = static_check_repo(
            &[("package.json", STATIC_CHECK_LINT_PKG)],
            Some(json!({"commands": {"test": ["pnpm lint", "pnpm test"]}})),
        );
        let n = commands_notices(dir.path(), false);
        assert_eq!(n.len(), 1);
        assert!(n[0].contains("npm run typecheck"));
        assert!(!n[0].contains("npm run lint"));
    }

    #[test]
    fn static_check_list_valued_test_with_nothing_detected_returns_nothing() {
        let dir = static_check_repo(&[], Some(json!({"commands": {"test": ["a", "b"]}})));
        assert!(commands_notices(dir.path(), true).is_empty());
    }

    #[test]
    fn static_check_none_sentinel_and_setup_only_stay_silent() {
        let dir = static_check_repo(
            &[("Cargo.toml", "[package]\n")],
            Some(json!({"commands": {"test": "none"}})),
        );
        assert!(commands_notices(dir.path(), false).is_empty());
        let dir = static_check_repo(
            &[("Cargo.toml", "[package]\n")],
            Some(json!({"commands": {"setup": "make setup"}})),
        );
        assert!(commands_notices(dir.path(), false).is_empty());
    }

    #[test]
    fn static_check_without_a_candidate_keeps_the_old_strings() {
        let dir = static_check_repo(&[], None);
        let n = commands_notices(dir.path(), false);
        assert_eq!(n, vec![STATIC_CHECK_OLD_FIRST.to_string()]);
        let dir = static_check_repo(&[("package.json", r#"{"scripts":{"test":"vitest"}}"#)], None);
        assert_eq!(
            commands_notices(dir.path(), false),
            vec!["No standard commands recorded. Detected candidates: test: npm test — package.json. Present them to the user as one pre-filled confirmation question (skippable) and write only confirmed values to .bee/config.json `commands` — never write unconfirmed values (D3). They power the session CI status gate.".to_string()]
        );
    }

    #[test]
    fn detect_commands_prefers_package_json_then_conventions() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            json!({"scripts":{"test":"vitest","start":"node ."}}).to_string(),
        )
        .unwrap();
        std::fs::write(dir.path().join("go.mod"), "module x\n").unwrap();
        let got = detect_commands(dir.path());
        let pairs: Vec<(&str, String)> = got.iter().map(|c| (c.key, c.value.clone())).collect();
        assert_eq!(
            pairs,
            vec![("start", "npm run start".to_string()), ("test", "npm test".to_string())]
        );
    }

    #[test]
    fn makefile_targets_ignore_recipes_and_assignments() {
        let text = "test:\n\techo hi\nVAR := 1\n.PHONY: test\nverify: test\n";
        assert_eq!(makefile_targets(text), vec!["test", "verify"]);
    }

    #[test]
    fn conventions_fire_only_when_nothing_explicit_matched() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("pyproject.toml"), "[project]\n").unwrap();
        let got = detect_commands(dir.path());
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].value, "pytest");
        assert_eq!(got[0].source, "pyproject.toml");
    }

    #[test]
    fn commands_notice_is_silent_once_recorded() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(
            dir.path().join(".bee").join("config.json"),
            json!({"commands":{"test":"cargo test"}}).to_string(),
        )
        .unwrap();
        assert!(commands_notices(dir.path(), false).is_empty());
    }

    #[test]
    fn first_onboard_without_a_build_adds_the_init_lane_notice() {
        let dir = tempfile::tempdir().unwrap();
        let n = commands_notices(dir.path(), true);
        assert_eq!(n.len(), 2);
        assert!(n[1].starts_with("Greenfield init lane"));
        assert_eq!(commands_notices(dir.path(), false).len(), 1);
    }

    #[test]
    fn stale_advisor_key_is_reported() {
        // Every repo now also draws a verification notice, so this case is
        // about the advisor key appearing and disappearing, never about the
        // list being empty.
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        let has_advisor_warning = |dir: &Path| {
            stale_advisor_notices(dir).contains(&STALE_ADVISOR_KEY_WARNING.to_string())
        };
        std::fs::write(
            dir.path().join(".bee").join("config.json"),
            json!({"advisor": {"mode": "x"}, "commands": {"test": "cargo test"}}).to_string(),
        )
        .unwrap();
        assert!(has_advisor_warning(dir.path()));
        std::fs::write(
            dir.path().join(".bee").join("config.json"),
            json!({"commands": {"test": "cargo test"}}).to_string(),
        )
        .unwrap();
        assert!(!has_advisor_warning(dir.path()));
    }

    /// Whether the fixture repo carries the verification skill SOURCE. Named
    /// constants because the truth table has two independent axes and a bare
    /// `true` at the call site says nothing about which one it moves.
    const WITH_SKILL: bool = true;
    const NO_SKILL: bool = false;

    fn stale_notices_for(config: Value, verify_skill: bool) -> Vec<String> {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(dir.path().join(".bee").join("config.json"), config.to_string()).unwrap();
        if verify_skill {
            let skill = verify_app_skill_file(dir.path());
            std::fs::create_dir_all(skill.parent().unwrap()).unwrap();
            std::fs::write(skill, "drive the app\n").unwrap();
        }
        stale_advisor_notices(dir.path())
    }

    fn deploy_verify_repo(files: &[(&str, &str)], config: Value) -> Vec<String> {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(dir.path().join(".bee").join("config.json"), config.to_string()).unwrap();
        for (rel, body) in files {
            let file = dir.path().join(rel);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, body).unwrap();
        }
        stale_advisor_notices(dir.path())
    }

    fn deploy_verify_count(notices: &[String]) -> usize {
        notices.iter().filter(|n| n.as_str() == DEPLOY_DRIVES_NO_PRODUCT_NOTICE).count()
    }

    #[test]
    fn deploy_verify_record_without_the_path_fires_once() {
        let notices = deploy_verify_repo(&[("docs/DEPLOY.md", "# Deploy\n")], json!({}));
        assert_eq!(deploy_verify_count(&notices), 1);
        assert_eq!(notices.len(), 2, "the verification offer stays beside it");
    }

    #[test]
    fn deploy_verify_script_naming_the_path_alone_is_silent() {
        let notices = deploy_verify_repo(
            &[("scripts/deploy.sh", "bash .bee/verify/verify-app/control-x drive --full\n")],
            json!({}),
        );
        assert_eq!(deploy_verify_count(&notices), 0);
    }

    #[test]
    fn deploy_verify_record_naming_the_path_beside_a_script_without_it_fires() {
        let notices = deploy_verify_repo(
            &[
                ("docs/DEPLOY.md", "run .bee/verify/verify-app/control-x\n"),
                ("scripts/deploy.sh", "echo deploy\n"),
            ],
            json!({}),
        );
        assert_eq!(deploy_verify_count(&notices), 1);
    }

    #[test]
    fn deploy_verify_both_files_naming_the_path_are_silent() {
        let named = "bash .bee/verify/verify-app/control-x drive --full\n";
        let notices = deploy_verify_repo(
            &[("docs/DEPLOY.md", named), ("scripts/deploy.sh", named)],
            json!({}),
        );
        assert_eq!(deploy_verify_count(&notices), 0);
    }

    #[test]
    fn deploy_verify_no_deploy_file_is_silent() {
        let notices = deploy_verify_repo(&[], json!({}));
        assert_eq!(deploy_verify_count(&notices), 0);
    }

    #[test]
    fn deploy_verify_release_script_alone_is_silent() {
        let notices = deploy_verify_repo(&[("scripts/release.sh", "echo release\n")], json!({}));
        assert_eq!(deploy_verify_count(&notices), 0);
    }

    #[test]
    fn deploy_verify_fires_beside_the_retired_verify_key() {
        let notices = deploy_verify_repo(
            &[("scripts/deploy.sh", "echo deploy\n")],
            json!({"commands": {"verify": "make check"}}),
        );
        assert_eq!(deploy_verify_count(&notices), 1);
        assert_eq!(notices[0], RETIRED_VERIFY_KEY_NO_TEST_WARNING);
    }

    #[test]
    fn deploy_verify_notice_sits_between_the_verification_arm_and_the_design_offer() {
        let notices = deploy_verify_repo(
            &[("docs/DEPLOY.md", "# Deploy\n"), ("web/index.html", "<p>hi</p>\n")],
            json!({}),
        );
        assert_eq!(
            notices,
            vec![
                NO_TEST_VERIFICATION_OFFER.to_string(),
                DEPLOY_DRIVES_NO_PRODUCT_NOTICE.to_string(),
                DESIGN_PACKS_OFFER.to_string(),
            ]
        );
    }

    #[test]
    fn deploy_verify_notice_names_the_skills_and_stays_plain() {
        for needed in ["bee-deploying", "bee-verifying", "first live step"] {
            assert!(DEPLOY_DRIVES_NO_PRODUCT_NOTICE.contains(needed), "{needed}");
        }
        for banned in ["config.json", "gate", "cap ", "proof"] {
            assert!(!DEPLOY_DRIVES_NO_PRODUCT_NOTICE.contains(banned), "{banned}");
        }
    }

    #[test]
    fn a_repo_with_no_test_command_is_offered_the_verification_skill() {
        let offer = vec![NO_TEST_VERIFICATION_OFFER.to_string()];
        // The target population: onboarded after 2.1.0, so no legacy key.
        assert_eq!(stale_notices_for(json!({"commands": {"start": "make run"}}), NO_SKILL), offer);
        // No commands block at all is the same state — nothing is declared.
        assert_eq!(stale_notices_for(json!({}), NO_SKILL), offer);
        // Neither is a blank one.
        assert_eq!(stale_notices_for(json!({"commands": {"test": "   "}}), NO_SKILL), offer);
    }

    #[test]
    fn a_legacy_verify_key_without_a_test_gets_the_retirement_warning_alone() {
        assert_eq!(
            stale_notices_for(json!({"commands": {"verify": "make check"}}), NO_SKILL),
            vec![RETIRED_VERIFY_KEY_NO_TEST_WARNING.to_string()],
            "the retired-key host already has its instruction; two notices would contradict",
        );
    }

    #[test]
    fn a_legacy_verify_key_beside_a_test_gets_the_plain_retirement_warning() {
        assert_eq!(
            stale_notices_for(
                json!({"commands": {"verify": "make check", "test": "cargo test"}}),
                NO_SKILL
            ),
            vec![RETIRED_VERIFY_KEY_WARNING.to_string()],
        );
    }

    #[test]
    fn a_legacy_verify_key_outranks_the_upkeep_pointer() {
        // The stated arm order, asserted: the retired key is read FIRST, so a
        // host that has both a legacy key and the skill hears about the key
        // it must delete and nothing else. Exactly one notice, never two.
        for shape in [
            json!({"commands": {"verify": "make check"}}),
            json!({"commands": {"verify": "make check", "test": "cargo test"}}),
        ] {
            let notices = stale_notices_for(shape.clone(), WITH_SKILL);
            assert_eq!(notices.len(), 1, "one notice only, for {shape}");
            assert!(
                notices[0].starts_with("commands.verify was retired"),
                "the retirement warning wins, for {shape}",
            );
        }
    }

    #[test]
    fn a_declared_test_command_now_draws_the_undriven_product_offer() {
        // It used to draw nothing at all: the offer was gated on the repo
        // declaring NO test command, so most repos — bee's own included —
        // never heard of verification. A test command is not a drive.
        for declared in [json!("cargo test"), json!(["cargo test", "cargo clippy"])] {
            assert_eq!(
                stale_notices_for(json!({"commands": {"test": declared}}), NO_SKILL),
                vec![TESTED_REPO_VERIFICATION_OFFER.to_string()],
                "declared as {declared}",
            );
        }
        let first = |s: &str| s.split(". ").next().unwrap().to_string();
        assert_ne!(
            first(TESTED_REPO_VERIFICATION_OFFER),
            first(NO_TEST_VERIFICATION_OFFER),
            "a tested repo must not be told it has no command that proves it works",
        );
    }

    #[test]
    fn the_verification_skill_beside_no_test_command_draws_the_upkeep_pointer() {
        assert_eq!(
            stale_notices_for(json!({"commands": {"start": "make run"}}), WITH_SKILL),
            vec![VERIFICATION_UPKEEP_POINTER.to_string()],
            "the skill exists, so there is nothing left to offer",
        );
    }

    #[test]
    fn the_verification_skill_beside_a_test_command_draws_the_upkeep_pointer() {
        assert_eq!(
            stale_notices_for(json!({"commands": {"test": "cargo test"}}), WITH_SKILL),
            vec![VERIFICATION_UPKEEP_POINTER.to_string()],
            "the test command never decided this arm; the skill's existence does",
        );
    }

    #[test]
    fn the_verification_offer_speaks_to_the_user_without_internal_terms() {
        // All three are printed verbatim, so whatever any of them says can
        // reach a person.
        for text in [
            NO_TEST_VERIFICATION_OFFER,
            TESTED_REPO_VERIFICATION_OFFER,
            VERIFICATION_UPKEEP_POINTER,
        ] {
            for banned in [".bee", "commands.", "config.json", "gate", "cap ", "proof"] {
                assert!(!text.contains(banned), "no notice may say {banned}: {text}");
            }
        }
        for offer in [NO_TEST_VERIFICATION_OFFER, TESTED_REPO_VERIFICATION_OFFER] {
            assert!(offer.contains("bee-verifying"), "it names the skill: {offer}");
            assert!(
                offer.contains("user's to accept"),
                "the offer is the user's to accept, never the agent's to assume: {offer}",
            );
        }
        assert!(
            VERIFICATION_UPKEEP_POINTER.contains("bee-verify-upkeep"),
            "the pointer names the skill that does the upkeep",
        );
        assert!(
            !VERIFICATION_UPKEEP_POINTER.contains("Offer"),
            "the upkeep notice is a pointer, not a second offer",
        );
    }

    #[test]
    fn the_design_offer_speaks_plainly_and_names_every_pack() {
        for banned in [".bee", "config.json", "design.packs", "gate", "cap ", "proof", "cell"] {
            assert!(!DESIGN_PACKS_OFFER.contains(banned), "no offer may say {banned}");
        }
        for pack in ["web", "charts", "motion", "apple", "figma", "none"] {
            assert!(DESIGN_PACKS_OFFER.contains(pack), "the offer names {pack}");
        }
        assert!(DESIGN_PACKS_OFFER.contains("replaces every edit made inside its design folders"));
        assert!(DESIGN_PACKS_OFFER.contains("never pick for them"));
    }

    #[test]
    fn the_ui_scan_skips_hidden_and_build_trees_and_stops_at_its_depth_cap() {
        let dir = tempfile::tempdir().unwrap();
        let put = |rel: &[&str]| {
            let mut p = dir.path().to_path_buf();
            for seg in rel {
                p = p.join(seg);
            }
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "x").unwrap();
        };
        put(&[".venv", "lib", "page.html"]);
        put(&["node_modules", "x", "a.css"]);
        put(&["dist", "a.jsx"]);
        put(&["target", "a.vue"]);
        put(&["src", "main.rs"]);
        let too_deep: Vec<String> = (0..=UI_SCAN_MAX_DEPTH).map(|i| format!("d{i}")).collect();
        let mut path: Vec<&str> = too_deep.iter().map(String::as_str).collect();
        path.push("deep.svelte");
        put(&path);
        assert!(!has_ui_file(dir.path()));

        let deepest: Vec<&str> = too_deep[..UI_SCAN_MAX_DEPTH].iter().map(String::as_str).chain(["ok.scss"]).collect();
        put(&deepest);
        assert!(has_ui_file(dir.path()));
    }

    #[test]
    fn prose_detection_ignores_comments_and_whitespace() {
        assert!(!has_prose_outside_block("<!-- BEE:START -->\nx\n<!-- BEE:END -->\n"));
        assert!(!has_prose_outside_block("   \n<!-- a\nmulti\nline -->\n"));
        assert!(has_prose_outside_block("# my project\n<!-- BEE:START -->\nx\n<!-- BEE:END -->\n"));
        // An unclosed comment is conservative: it counts as prose.
        assert!(has_prose_outside_block("<!-- unclosed\n"));
    }

    #[test]
    fn agents_header_names_only_files_that_exist() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("myrepo");
        std::fs::create_dir_all(&repo).unwrap();
        assert_eq!(
            compose_agents_header(&repo),
            "# myrepo\n\n<!-- [unknown] one-line project description - replace me -->\n\n"
        );
        std::fs::write(repo.join("README.md"), "x").unwrap();
        assert_eq!(
            compose_agents_header(&repo),
            "# myrepo\n\n<!-- [unknown] one-line project description - replace me -->\n\n- README.md\n\n"
        );
    }

    #[test]
    fn repo_hooks_record_detection() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!has_repo_hooks_recorded(dir.path()));
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(
            dir.path().join(".bee").join("onboarding.json"),
            json!({"managed":{"repo_hooks":{}}}).to_string(),
        )
        .unwrap();
        assert!(!has_repo_hooks_recorded(dir.path()), "an empty record is not an opt-in");
        std::fs::write(
            dir.path().join(".bee").join("onboarding.json"),
            json!({"managed":{"repo_hooks":{"adapter.mjs":"h"}}}).to_string(),
        )
        .unwrap();
        assert!(has_repo_hooks_recorded(dir.path()));
    }
}
