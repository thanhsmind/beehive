use super::*;
use serde_json::{Map, Value};
use std::path::{Component, Path, PathBuf};

struct Scope {
    roots: Vec<PathBuf>,
    common: Option<PathBuf>,
    home: Option<PathBuf>,
    temp: Option<PathBuf>,
    allowed: Vec<PathBuf>,
}

fn expand(raw: &str, cwd: &Path, home: Option<&Path>) -> Option<PathBuf> {
    let joined = if raw == "~" {
        home?.to_path_buf()
    } else if let Some(rest) = raw.strip_prefix("~/") {
        home?.join(rest)
    } else if raw.starts_with('/') {
        PathBuf::from(raw)
    } else {
        cwd.join(raw)
    };
    if !joined.is_absolute() {
        return None;
    }
    let mut norm = PathBuf::new();
    for c in joined.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                norm.pop();
            }
            other => norm.push(other.as_os_str()),
        }
    }
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let mut cur = norm.clone();
    loop {
        if let Ok(real) = std::fs::canonicalize(&cur) {
            let mut out = real;
            for part in tail.iter().rev() {
                out.push(part);
            }
            return Some(out);
        }
        match cur.file_name() {
            Some(name) => tail.push(name.to_os_string()),
            None => return Some(norm),
        }
        if !cur.pop() {
            return Some(norm);
        }
    }
}

fn git_common_dir(dir: &Path) -> Option<PathBuf> {
    let dot_git = dir.join(".git");
    if dot_git.is_dir() {
        return std::fs::canonicalize(dot_git).ok();
    }
    let text = std::fs::read_to_string(&dot_git).ok()?;
    let target = text.lines().next()?.strip_prefix("gitdir:")?.trim();
    let gitdir = if Path::new(target).is_absolute() {
        PathBuf::from(target)
    } else {
        dir.join(target)
    };
    if let Ok(rel) = std::fs::read_to_string(gitdir.join("commondir")) {
        return std::fs::canonicalize(gitdir.join(rel.trim())).ok();
    }
    let gitdir = std::fs::canonicalize(&gitdir).ok()?;
    match (gitdir.parent(), gitdir.parent().and_then(Path::parent)) {
        (Some(p), Some(common)) if p.file_name().map_or(false, |n| n == "worktrees") => {
            Some(common.to_path_buf())
        }
        _ => Some(gitdir),
    }
}

fn holds_root_marker(dir: &Path) -> bool {
    dir.join(".git").exists() || dir.join(".bee").join("onboarding.json").is_file()
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => {
            items.iter().filter_map(|v| v.as_str().map(str::to_string)).collect()
        }
        _ => Vec::new(),
    }
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

impl Scope {
    fn load(root: &Path, store_root: &Path, home: Option<PathBuf>) -> Scope {
        let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        let mut roots = vec![canon(root)];
        let store = canon(store_root);
        if !roots.contains(&store) {
            roots.push(store.clone());
        }
        let home = home.map(|h| canon(&h));
        let common = git_common_dir(&store).or_else(|| git_common_dir(&roots[0]));
        let config = read_json(&store_root.join(".bee").join("config.json"));
        let local = read_json(&store_root.join(".bee").join("config.local.json"));
        let mut raw_allowed: Vec<String> = Vec::new();
        for cfg in [config, local].into_iter().flatten() {
            raw_allowed.extend(string_list(cfg.get("guards").and_then(|g| g.get("cross_project_allow"))));
        }
        let mut settings: Vec<PathBuf> = Vec::new();
        if let Some(h) = &home {
            settings.push(h.join(".claude").join("settings.json"));
        }
        for base in &roots {
            settings.push(base.join(".claude").join("settings.json"));
            settings.push(base.join(".claude").join("settings.local.json"));
        }
        for file in settings {
            if let Some(doc) = read_json(&file) {
                raw_allowed.extend(string_list(
                    doc.get("permissions").and_then(|p| p.get("additionalDirectories")),
                ));
            }
        }
        let allowed = raw_allowed
            .iter()
            .filter(|e| e.starts_with('/') || e.starts_with('~'))
            .filter_map(|e| expand(e, Path::new("/"), home.as_deref()))
            .collect();
        let temp = std::fs::canonicalize(std::env::temp_dir()).ok();
        Scope { roots, common, home, temp, allowed }
    }

    fn is_this_project(&self, dir: &Path) -> bool {
        if self.roots.iter().any(|r| dir.starts_with(r)) {
            return true;
        }
        match (&self.common, git_common_dir(dir)) {
            (Some(ours), Some(theirs)) => *ours == theirs,
            _ => false,
        }
    }

    fn is_boundary(&self, dir: &Path) -> bool {
        dir.parent().is_none()
            || self.home.as_deref() == Some(dir)
            || self.temp.as_deref() == Some(dir)
    }

    fn other_root(&self, raw: &str, cwd: &Path) -> Option<PathBuf> {
        let path = expand(raw, cwd, self.home.as_deref())?;
        if self.roots.iter().any(|r| path.starts_with(r)) {
            return None;
        }
        if self.allowed.iter().any(|a| path.starts_with(a)) {
            return None;
        }
        let mut start = path.clone();
        while !start.exists() && start.pop() {}
        for dir in start.ancestors() {
            if self.is_boundary(dir) || !holds_root_marker(dir) {
                continue;
            }
            return if self.is_this_project(dir) { None } else { Some(dir.to_path_buf()) };
        }
        if path.is_dir() && !self.is_boundary(&path) {
            for entry in std::fs::read_dir(&path).ok()?.flatten() {
                let child = entry.path();
                if holds_root_marker(&child) && !self.is_this_project(&child) {
                    return Some(child);
                }
            }
        }
        None
    }
}

fn glob_prefix(pattern: &str) -> &str {
    let cut = pattern.find(|c| matches!(c, '*' | '?' | '[' | '{')).unwrap_or(pattern.len());
    &pattern[..cut]
}

fn token_path(token: &str) -> Option<&str> {
    let body = match token.strip_prefix("--") {
        Some(flag) => flag.split_once('=')?.1,
        None => token,
    };
    (body.starts_with('/') || body.starts_with('~')).then_some(body)
}

fn refusal(raw: &str, other: &Path) -> String {
    format!(
        "bee cross-project guard: \"{raw}\" belongs to another project ({}). \
Do not read or work it from this session. \
FIX: call waggledance_ask_state for that project, then waggledance_dispatch with leader: true so its own leader does the work. \
If the user asked you to research that repo from here, ask the user to add the path to guards.cross_project_allow in .bee/config.json.",
        other.display()
    )
}

pub(crate) fn cross_project_denial(
    root: &Path,
    store_root: &Path,
    home: Option<PathBuf>,
    harness: &HarnessRoots,
    tool_name: &str,
    tool_input: &Map<String, Value>,
    cwd: &str,
) -> Option<String> {
    let mut candidates: Vec<String> = Vec::new();
    match tool_name {
        "Read" | "Glob" | "Grep" => {
            let key = if tool_name == "Read" { "file_path" } else { "path" };
            if let Some(Value::String(s)) = tool_input.get(key).filter(|v| truthy(v)) {
                candidates.push(s.clone());
            }
            if tool_name == "Glob" {
                if let Some(Value::String(p)) = tool_input.get("pattern") {
                    let prefix = glob_prefix(p);
                    if prefix.starts_with('/') || prefix.starts_with('~') {
                        candidates.push(prefix.to_string());
                    }
                }
            }
        }
        "Bash" | "exec" => {
            let command = match first_truthy(tool_input, &["command", "cmd"]) {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            candidates.push(cwd.to_string());
            for token in tokenize_deep(&fence_heredocs(&command)).tokens {
                if let Some(p) = token_path(&token) {
                    candidates.push(p.to_string());
                }
            }
        }
        _ => return None,
    }
    if candidates.is_empty() {
        return None;
    }
    let scope = Scope::load(root, store_root, home);
    let base = Path::new(cwd);
    let (root_s, cwd_s) = (root.to_string_lossy(), cwd);
    for raw in candidates {
        let Some(other) = scope.other_root(&raw, base) else { continue };
        let harness_owned = harness_allowlisted_target(harness, &root_s, cwd_s, &Value::String(raw.clone()))
            .unwrap_or(false);
        if !harness_owned {
            return Some(refusal(&raw, &other));
        }
    }
    None
}
