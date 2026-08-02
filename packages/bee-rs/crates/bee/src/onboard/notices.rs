// onboard::notices — the four advisory notice sources, plus the
// lib/commands_detect.mjs port they lean on.
//
// Provenance: onboard_bee.mjs commandsNotices (l. 2594), staleAdvisorNotices
// (l. 2645), trackedGitignorePaths (l. 2665), trackedPathsNotices (l. 2678)
// and repoHooksTransitionNotices (l. 3512) — plus lib/commands_detect.mjs
// detectCommands, which this script imports.

use super::templates::{
    COMMAND_KEYS, GITIGNORE_BLOCK_PATTERNS, RETIRED_VERIFY_KEY_NO_TEST_WARNING,
    RETIRED_VERIFY_KEY_WARNING, STALE_ADVISOR_KEY_WARNING,
};
use super::util::{exists, read_dir_sorted, read_json_if_exists, read_text_if_exists, split_lines};
use serde_json::Value;
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

// ── notices ────────────────────────────────────────────────────────────────

/// commandsNotices (l. 2594): propose-only (decision D3) — this script never
/// writes detected values to .bee/config.json.
pub fn commands_notices(repo_root: &Path, first_onboard: bool) -> Vec<String> {
    let config = read_json_if_exists(&repo_root.join(".bee").join("config.json"));
    let raw = config.as_ref().and_then(|c| c.get("commands")).filter(|c| c.is_object()).cloned();
    let recorded = COMMAND_KEYS.iter().any(|key| {
        raw.as_ref()
            .and_then(|r| r.get(*key))
            .and_then(Value::as_str)
            .is_some_and(|v| !v.trim().is_empty())
    });
    if recorded {
        return Vec::new();
    }
    let candidates = detect_commands(repo_root);
    if !candidates.is_empty() {
        let proposals: Vec<String> = candidates
            .iter()
            .map(|c| format!("{}: {} — {}", c.key, c.value, c.source))
            .collect();
        return vec![format!(
            "No standard commands recorded. Detected candidates: {}. Present them to the user as one pre-filled confirmation question (skippable) and write only confirmed values to .bee/config.json `commands` — never write unconfirmed values (D3). They power the session CI status gate.",
            proposals.join("; ")
        )];
    }
    let mut notices = vec![
        "No standard commands recorded. Ask the user for the host project's setup/start/test commands and write them to .bee/config.json `commands` (skippable — never invent values). They power the session CI status gate.".to_string(),
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
    if let Some(commands) = commands {
        if commands.contains_key("verify") {
            let has_test = commands
                .get("test")
                .is_some_and(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()) || v.is_array());
            notices.push(if has_test {
                RETIRED_VERIFY_KEY_WARNING.to_string()
            } else {
                RETIRED_VERIFY_KEY_NO_TEST_WARNING.to_string()
            });
        }
    }
    notices
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
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".bee")).unwrap();
        std::fs::write(
            dir.path().join(".bee").join("config.json"),
            json!({"advisor": {"mode": "x"}}).to_string(),
        )
        .unwrap();
        assert_eq!(stale_advisor_notices(dir.path()), vec![STALE_ADVISOR_KEY_WARNING.to_string()]);
        std::fs::write(dir.path().join(".bee").join("config.json"), json!({}).to_string()).unwrap();
        assert!(stale_advisor_notices(dir.path()).is_empty());
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
