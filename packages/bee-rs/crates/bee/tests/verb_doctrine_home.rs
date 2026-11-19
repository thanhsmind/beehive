use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

const HOMES: &[&str] = &[
    "skills",
    "AGENTS.md",
    "CLAUDE.md",
    "packages/bee/AGENTS.block.md",
    "packages/bee/LEADER.md",
    "packages/bee/playbooks",
    "packages/bee/prompts",
    "docs/knowledge",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .unwrap()
        .to_path_buf()
}

fn commands() -> BTreeSet<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_bee"))
        .args(["--help", "--all", "--json"])
        .output()
        .unwrap();
    let index: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    index["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let invoke = c["invoke"].as_str().unwrap();
            invoke.strip_prefix("bee ").unwrap_or(invoke).to_string()
        })
        .collect()
}

fn read_tree(path: &Path, into: &mut String) {
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for entry in entries {
            read_tree(&entry, into);
        }
    } else if let Ok(text) = std::fs::read_to_string(path) {
        into.push_str(&text);
        into.push('\n');
    }
}

fn allowlist() -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/verb_doctrine_home_allowlist.txt");
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| match l.split_once(" :: ") {
            Some((cmd, reason)) => (cmd.trim().to_string(), reason.trim().to_string()),
            None => (l.trim().trim_end_matches("::").trim().to_string(), String::new()),
        })
        .collect()
}

#[test]
fn every_shipped_verb_has_a_doctrine_home_or_an_allowlist_reason() {
    let root = repo_root();
    let mut corpus = String::new();
    for home in HOMES {
        read_tree(&root.join(home), &mut corpus);
    }
    let commands = commands();
    let allowed = allowlist();
    let mut problems = Vec::new();

    for (cmd, reason) in &allowed {
        if reason.is_empty() {
            problems.push(format!("allowlist row `{cmd}` has an empty reason"));
        }
        if !commands.contains(cmd) {
            problems.push(format!("allowlist row `{cmd}` names a command that no longer exists"));
        }
    }
    let listed: BTreeSet<&str> = allowed.iter().map(|(c, _)| c.as_str()).collect();
    for cmd in &commands {
        if !corpus.contains(cmd.as_str()) && !listed.contains(cmd.as_str()) {
            problems.push(format!(
                "`bee {cmd}` is named in no doctrine home and is not in the allowlist"
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}
