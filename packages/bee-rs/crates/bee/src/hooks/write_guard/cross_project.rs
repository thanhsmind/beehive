use super::*;
use serde_json::{Map, Value};
use std::cell::OnceCell;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const REGISTRY_TIMEOUT: Duration = Duration::from_secs(2);
const SURFACE_WALK_LIMIT: usize = 50000;
const SHELL_HEADS: [&str; 6] = ["cat", "head", "tail", "wc", "ls", "jq"];

struct Scope {
    roots: Vec<PathBuf>,
    common: Option<PathBuf>,
    home: Option<PathBuf>,
    temp: Option<PathBuf>,
    allowed: Vec<PathBuf>,
    supervises: bool,
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
        let mut supervises = false;
        for cfg in [config, local].into_iter().flatten() {
            let guards = cfg.get("guards");
            raw_allowed.extend(string_list(guards.and_then(|g| g.get("cross_project_allow"))));
            if let Some(value) = guards.and_then(|g| g.get("supervises")) {
                supervises = value.as_str() == Some("waggledance");
            }
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
        Scope { roots, common, home, temp, allowed, supervises }
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

fn token_body(token: &str) -> Option<&str> {
    match token.strip_prefix("--") {
        Some(flag) => flag.split_once('=').map(|(_, v)| v),
        None => Some(token),
    }
}

fn shell_candidates(tokens: &[String], cwd: &Path, home: Option<&Path>) -> Vec<String> {
    let mut base = cwd.to_path_buf();
    let mut cd_next = false;
    let mut out = Vec::new();
    for token in tokens {
        if token == "cd" {
            cd_next = true;
            continue;
        }
        if cd_next && token.starts_with('-') {
            continue;
        }
        if let Some(body) = token_body(token) {
            if body.starts_with('/') || body.starts_with('~') {
                out.push(body.to_string());
            } else if Path::new(body).components().any(|c| c == Component::ParentDir) {
                out.push(base.join(body).to_string_lossy().into_owned());
            }
        }
        if cd_next {
            cd_next = false;
            if let Some(next) = expand(token, &base, home) {
                base = next;
            }
        }
    }
    out
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

pub(crate) type RegistryProvider<'a> = &'a dyn Fn(&[PathBuf]) -> Option<Vec<PathBuf>>;

enum Why {
    Rule(String),
    Unreadable,
    Unregistered(PathBuf),
}

impl Why {
    fn sentence(&self, other: &Path) -> String {
        match self {
            Why::Unreadable => {
                "The waggledance registry was not readable, so no project is treated as managed.".to_string()
            }
            Why::Unregistered(dir) => format!("{} is not in the waggledance registry.", dir.display()),
            Why::Rule(rule) => format!(
                "guards.supervises grants only {0}/.bee/** and {0}/docs/knowledge/** of a managed project, read-only; refused by the {1} rule.",
                other.display(),
                rule
            ),
        }
    }
}

fn rule(text: &str) -> Why {
    Why::Rule(text.to_string())
}

fn waggledance_binary(own: &[PathBuf]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join("waggledance");
        let Ok(real) = std::fs::canonicalize(&candidate) else { continue };
        if real.is_file() && !own.iter().any(|r| real.starts_with(r)) {
            return Some(candidate);
        }
    }
    None
}

fn parse_registry(bytes: &[u8]) -> Option<Vec<PathBuf>> {
    let doc: Value = serde_json::from_slice(bytes).ok()?;
    let projects = doc.get("projects")?.as_array()?;
    Some(
        projects
            .iter()
            .filter_map(|p| p.get("root_path").and_then(Value::as_str))
            .filter(|p| p.starts_with('/'))
            .map(PathBuf::from)
            .collect(),
    )
}

pub(crate) fn waggledance_registry_roots(own: &[PathBuf]) -> Option<Vec<PathBuf>> {
    let binary = waggledance_binary(own)?;
    let mut child = Command::new(binary)
        .args(["list", "--json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let read = stdout.read_to_end(&mut buf).map(|_| buf);
        let _ = tx.send(read);
    });
    let deadline = Instant::now() + REGISTRY_TIMEOUT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    if !status.success() {
        return None;
    }
    let left = deadline.saturating_duration_since(Instant::now()).max(Duration::from_millis(200));
    let bytes = rx.recv_timeout(left).ok()?.ok()?;
    parse_registry(&bytes)
}

fn is_managed(other: &Path, roots: &[PathBuf]) -> bool {
    let theirs = git_common_dir(other);
    roots.iter().any(|r| {
        r == other || matches!((&theirs, git_common_dir(r)), (Some(a), Some(b)) if *a == b)
    })
}

fn named_secret(path: &Path) -> bool {
    path.file_name().map_or(false, |n| n == "config.local.json")
        || is_secret_path(&path.to_string_lossy())
}

fn holds_secret_below(dir: &Path) -> bool {
    let mut stack = vec![dir.to_path_buf()];
    let mut seen = 0usize;
    while let Some(next) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&next) else { continue };
        for entry in entries.flatten() {
            seen += 1;
            if seen > SURFACE_WALK_LIMIT {
                return true;
            }
            let path = entry.path();
            if named_secret(&path) {
                return true;
            }
            if entry.file_type().map_or(false, |t| t.is_dir()) {
                stack.push(path);
            }
        }
    }
    false
}

struct Grant<'a> {
    scope: &'a Scope,
    store_root: &'a Path,
    harness: &'a HarnessRoots,
    root_s: &'a str,
    cwd: &'a str,
    provider: RegistryProvider<'a>,
    managed: OnceCell<Option<Vec<PathBuf>>>,
}

impl Grant<'_> {
    fn managed_roots(&self) -> Option<&Vec<PathBuf>> {
        self.managed
            .get_or_init(|| {
                let roots = (self.provider)(&self.scope.roots)?;
                let canon: Vec<PathBuf> = roots
                    .iter()
                    .map(|r| std::fs::canonicalize(r).unwrap_or_else(|_| r.clone()))
                    .collect();
                if canon.is_empty() { None } else { Some(canon) }
            })
            .as_ref()
    }

    fn judge(&self, raw: &str, root_value: bool, walk: bool) -> Result<(), (Why, Option<PathBuf>)> {
        let cwd = Path::new(self.cwd);
        let Some(other) = self.scope.other_root(raw, cwd) else { return Ok(()) };
        let owned = harness_allowlisted_target(self.harness, self.root_s, self.cwd, &Value::String(raw.to_string()))
            .unwrap_or(false);
        if owned {
            return Ok(());
        }
        let fail = |why: Why| Err((why, Some(other.clone())));
        let Some(roots) = self.managed_roots() else { return fail(Why::Unreadable) };
        if !is_managed(&other, roots) {
            return fail(Why::Unregistered(other.clone()));
        }
        let Some(path) = expand(raw, cwd, self.scope.home.as_deref()) else {
            return fail(rule("unresolvable path"));
        };
        if Path::new(raw).components().any(|c| c == Component::ParentDir) {
            return fail(rule("parent segment"));
        }
        if root_value && (path == other || path == other.join(".bee")) {
            return Ok(());
        }
        let in_surface = [".bee", "docs/knowledge"].iter().any(|s| path.starts_with(other.join(s)));
        if !in_surface {
            return fail(rule("outside the readable surface"));
        }
        if named_secret(&path) {
            return fail(rule("secret-shaped file"));
        }
        if walk && path.is_dir() && holds_secret_below(&path) {
            return fail(rule("directory holding a secret-shaped file"));
        }
        Ok(())
    }

    fn judge_tool(
        &self,
        tool_name: &str,
        tool_input: &Map<String, Value>,
        candidates: &[String],
    ) -> Result<(), (Why, Option<PathBuf>)> {
        if tool_name == "Glob" {
            if let Some(Value::String(p)) = tool_input.get("pattern") {
                if p.contains("..") {
                    return Err((rule("glob parent segment"), None));
                }
            }
        }
        for raw in candidates {
            self.judge(raw, false, tool_name == "Grep")?;
        }
        Ok(())
    }

    fn is_bee_head(&self, token: &str) -> bool {
        if token == "bee" {
            return true;
        }
        let own = self.store_root.join(".bee").join("bin").join("bee");
        let cwd = Path::new(self.cwd);
        let home = self.scope.home.as_deref();
        match (expand(token, cwd, home), expand(&own.to_string_lossy(), cwd, home)) {
            (Some(a), Some(b)) => token.contains('/') && a == b,
            _ => false,
        }
    }

    fn judge_shell(&self, command: &str, tokens: &[String], truncated: bool) -> Result<(), (Why, Option<PathBuf>)> {
        let cwd = Path::new(self.cwd);
        let home = self.scope.home.as_deref();
        let bail = |text: &str| Err((rule(text), None));
        if command.contains('\n') || command.contains('\r') {
            return bail("newline");
        }
        if truncated {
            return bail("undecidable wrapper");
        }
        match expand(self.cwd, cwd, home) {
            Some(dir) if self.scope.is_this_project(&dir) => {}
            _ => return bail("hook cwd outside this project"),
        }
        let mut folded: Vec<&str> = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let tail = (tokens.get(i + 1).map(String::as_str), tokens.get(i + 2).map(String::as_str));
            if tokens[i] == "2>" && tail == (Some("&"), Some("1")) {
                folded.push("2>&1");
                i += 3;
            } else {
                folded.push(&tokens[i]);
                i += 1;
            }
        }
        for segment in folded.split(|t| is_separator(t)) {
            let Some(head) = segment.first() else { continue };
            let bee = self.is_bee_head(head);
            let shape_ok = if bee {
                matches!(
                    (segment.get(1), segment.get(2)),
                    (Some(&"knowledge"), Some(&"report")) | (Some(&"report"), Some(&"outcomes"))
                )
            } else {
                SHELL_HEADS.contains(head)
            };
            if !shape_ok {
                return bail("command head");
            }
            for (idx, token) in segment.iter().enumerate().skip(1) {
                if token.contains('$') || token.contains('`') {
                    return bail("shell expansion");
                }
                if matches!(*token, "cd" | "pushd") {
                    return bail("directory change");
                }
                if (token.contains('<') || token.contains('>')) && !matches!(*token, "2>/dev/null" | "2>&1") {
                    return bail("redirect");
                }
                let body = token_body(token).unwrap_or("");
                if body.starts_with('~') && body != "~" && !body.starts_with("~/") {
                    return bail("user home expansion");
                }
                if token.starts_with('-')
                    && !token.starts_with("--")
                    && (token.contains('/') || token.contains('~'))
                {
                    return bail("attached path");
                }
                if token.contains('{') && (token.contains('/') || token.contains('~') || token.contains("..")) {
                    return bail("brace expansion");
                }
                let owned = [token.to_string()];
                let Some(raw) = shell_candidates(&owned, cwd, home).into_iter().next() else { continue };
                if raw.contains(['*', '?', '[', '{', '}']) {
                    return bail("glob or brace");
                }
                let root_value = bee && (token.starts_with("--root=") || segment[idx - 1] == "--root");
                self.judge(&raw, root_value, false)?;
            }
        }
        Ok(())
    }
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
    cross_project_denial_with_registry(
        root,
        store_root,
        home,
        harness,
        tool_name,
        tool_input,
        cwd,
        &waggledance_registry_roots,
    )
}

pub(crate) fn cross_project_denial_with_registry(
    root: &Path,
    store_root: &Path,
    home: Option<PathBuf>,
    harness: &HarnessRoots,
    tool_name: &str,
    tool_input: &Map<String, Value>,
    cwd: &str,
    provider: RegistryProvider,
) -> Option<String> {
    let mut candidates: Vec<String> = Vec::new();
    let mut shell: Option<(String, DeepTokens)> = None;
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
            let deep = tokenize_deep(&fence_heredocs(&command));
            candidates.extend(shell_candidates(&deep.tokens, Path::new(cwd), home.as_deref()));
            shell = Some((command, deep));
        }
        _ => return None,
    }
    if candidates.is_empty() {
        return None;
    }
    let scope = Scope::load(root, store_root, home);
    let base = Path::new(cwd);
    let (root_s, cwd_s) = (root.to_string_lossy(), cwd);
    let mut first: Option<(String, PathBuf)> = None;
    for raw in &candidates {
        let Some(other) = scope.other_root(raw, base) else { continue };
        let harness_owned = harness_allowlisted_target(harness, &root_s, cwd_s, &Value::String(raw.clone()))
            .unwrap_or(false);
        if !harness_owned {
            first = Some((raw.clone(), other));
            break;
        }
    }
    let (raw, other) = first?;
    let text = refusal(&raw, &other);
    if !scope.supervises {
        return Some(text);
    }
    let grant = Grant {
        scope: &scope,
        store_root,
        harness,
        root_s: &root_s,
        cwd: cwd_s,
        provider,
        managed: OnceCell::new(),
    };
    let verdict = match &shell {
        Some((command, deep)) => grant.judge_shell(command, &deep.tokens, deep.truncated),
        None => grant.judge_tool(tool_name, tool_input, &candidates),
    };
    match verdict {
        Ok(()) => None,
        Err((why, at)) => Some(format!("{text} {}", why.sentence(at.as_deref().unwrap_or(&other)))),
    }
}
