use super::Outcome;
use super::write_guard::{
    command_basename, outward_segment_head, codex_read_only_exec, gh_read_only_form,
    gh_sub_and_args, fence_heredocs, is_separator, tokenize, tokenize_deep, find_git_invocations,
};
use serde_json::{Map, Value};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

const READ_ONLY_ALLOWED: &str = "Bash(cat:*),Bash(head:*),Bash(tail:*),Bash(wc:*),Bash(ls:*),Bash(rg:*),Bash(fd:*),Bash(jq:*),Bash(git log:*),Bash(git show:*),Bash(git diff:*),Bash(git status:*),Bash(git blame:*),Bash(.bee/bin/bee status:*),Bash(.bee/bin/bee orient:*),Bash(.bee/bin/bee herding status:*),Bash(.bee/bin/bee herding notify:*),Bash(mv:*)";

const WRITE_TOOLS: [&str; 4] = ["Edit", "Write", "MultiEdit", "NotebookEdit"];

const MV_SAFE_FLAGS: [&str; 8] = ["-f", "-n", "-v", "-i", "--force", "--no-clobber", "--verbose", "--"];

pub fn run(_argv: &[String], stdin: &str) -> Outcome {
    let herding_worker = std::env::var("BEE_HERDING_WORKER").unwrap_or_default();
    let paseo_agent_id = std::env::var("PASEO_AGENT_ID").unwrap_or_default();
    let supervisor_allowed = std::env::var("BEE_SUPERVISOR_ALLOWED").unwrap_or_default();
    let read_only_job = read_only_job();

    let mode_a = !herding_worker.is_empty() && !paseo_agent_id.is_empty();
    let mode_b = !supervisor_allowed.is_empty();

    if !mode_a && !mode_b && read_only_job.is_none() {
        return Outcome::Done(ExitCode::SUCCESS);
    }

    let Ok(Value::Object(payload)) = serde_json::from_str::<Value>(stdin) else {
        return Outcome::Done(ExitCode::SUCCESS);
    };

    let tool_name = payload
        .get("tool_name")
        .or_else(|| payload.get("toolName"))
        .and_then(Value::as_str)
        .unwrap_or("");

    if let Some(job) = &read_only_job {
        let cwd = payload
            .get("cwd")
            .and_then(Value::as_str)
            .filter(|c| !c.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_default();
        let mailbox = job_mailbox(&cwd, job);
        let input = payload.get("tool_input").and_then(Value::as_object);
        if let Some(reason) = evaluate_read_only(tool_name, input, &cwd, mailbox.as_deref()) {
            eprintln!("{reason}");
            return Outcome::Done(ExitCode::from(2));
        }
    }

    if tool_name != "Bash" {
        return Outcome::Done(ExitCode::SUCCESS);
    }

    let command = payload
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|m| m.get("command").or_else(|| m.get("cmd")))
        .and_then(Value::as_str)
        .unwrap_or("");

    if command.is_empty() {
        return Outcome::Done(ExitCode::SUCCESS);
    }

    if mode_a {
        if let Some(reason) = evaluate_mode_a(command) {
            eprintln!("{reason}");
            return Outcome::Done(ExitCode::from(2));
        }
    }

    if mode_b {
        if let Some(reason) = evaluate_mode_b(command, &supervisor_allowed) {
            eprintln!("{reason}");
            return Outcome::Done(ExitCode::from(2));
        }
    }

    Outcome::Done(ExitCode::SUCCESS)
}

pub(crate) fn evaluate_mode_a(command: &str) -> Option<String> {
    let fenced = fence_heredocs(command);
    let deep = tokenize_deep(&fenced);

    if deep.truncated {
        let words: Vec<String> = deep
            .tokens
            .iter()
            .flat_map(|t| t.split_whitespace().map(str::to_string))
            .collect();
        if find_git_invocations(&words)
            .iter()
            .any(|i| i.subcommand.as_deref() == Some("push"))
        {
            return Some(mode_a_denial("`git push`"));
        }
        for word in &words {
            let base = command_basename(word);
            if base == "gh" {
                return Some(mode_a_denial("`gh`"));
            }
            if matches!(base.as_str(), "claude" | "codex" | "pi" | "opencode") {
                return Some(mode_a_denial(&format!("`{base}`")));
            }
            if base == "paseo" {
                return Some(mode_a_denial("`paseo`"));
            }
            if base == "bee" {
                return Some(mode_a_denial("`bee`"));
            }
        }
    }

    for segment in deep.tokens.split(|t| is_separator(t)) {
        if segment.is_empty() {
            continue;
        }

        let invocations = find_git_invocations(segment);
        if invocations
            .iter()
            .any(|i| i.subcommand.as_deref() == Some("push"))
        {
            return Some(mode_a_denial("`git push`"));
        }

        let Some((head_index, head)) = outward_segment_head(segment) else {
            continue;
        };
        let rest = &segment[head_index + 1..];

        if head == "gh" {
            let (sub, args) = gh_sub_and_args(rest);
            if !gh_read_only_form(sub, args) {
                let verb = args.iter().find(|t| !t.starts_with('-'));
                let blocked = match verb {
                    Some(v) => format!("`gh {sub} {v}`"),
                    None => format!("`gh {sub}`"),
                };
                return Some(mode_a_denial(&blocked));
            }
            continue;
        }

        if matches!(head.as_str(), "claude" | "codex" | "pi" | "opencode") {
            if !codex_read_only_exec(&head, rest) {
                return Some(mode_a_denial(&format!("`{head}`")));
            }
            continue;
        }

        if head == "paseo" {
            return Some(mode_a_denial("`paseo`"));
        }

        let raw_head = &segment[head_index];
        if head == "bee" || raw_head == "bee" || raw_head.ends_with("/bee") {
            let non_flag_args: Vec<&str> = rest
                .iter()
                .map(String::as_str)
                .filter(|a| !a.starts_with('-'))
                .collect();
            if let Some(&first_verb) = non_flag_args.first() {
                if first_verb == "dispatch" {
                    return Some(mode_a_denial("`bee dispatch`"));
                }
                if first_verb == "gate" {
                    return Some(mode_a_denial("`bee gate`"));
                }
                if non_flag_args.get(0..2) == Some(&["herding", "run"]) {
                    return Some(mode_a_denial("`bee herding run`"));
                }
                if non_flag_args.get(0..2) == Some(&["worktree", "merge"]) {
                    return Some(mode_a_denial("`bee worktree merge`"));
                }
            }
        }
    }

    None
}

pub(crate) fn evaluate_mode_b(command: &str, supervisor_allowed: &str) -> Option<String> {
    mode_b_blocked(command, supervisor_allowed).map(|blocked| mode_b_denial(&blocked))
}

fn mode_b_blocked(command: &str, supervisor_allowed: &str) -> Option<String> {
    if command.contains("$(") || command.contains('`') || command.contains('>') {
        return Some(command.to_string());
    }

    let mut allowed_prefixes = Vec::new();
    for entry in supervisor_allowed.split(',') {
        let entry = entry.trim();
        if let Some(rest) = entry.strip_prefix("Bash(") {
            if let Some(inner) = rest.strip_suffix(')') {
                let prefix = if let Some(p) = inner.strip_suffix(":*") {
                    p
                } else if let Some(p) = inner.strip_suffix('*') {
                    p
                } else {
                    inner
                };
                allowed_prefixes.push(prefix.trim().to_string());
            }
        }
    }

    let fenced = fence_heredocs(command);
    let deep = tokenize_deep(&fenced);
    if deep.truncated {
        return Some(command.to_string());
    }

    for segment in deep.tokens.split(|t| is_separator(t)) {
        if segment.is_empty() {
            continue;
        }

        let mut norm_tokens = segment.to_vec();
        let first = &norm_tokens[0];
        if first == "bee"
            || first == "./.bee/bin/bee"
            || (first.starts_with('/') && first.ends_with("/.bee/bin/bee"))
        {
            norm_tokens[0] = ".bee/bin/bee".to_string();
        }

        let norm_segment = norm_tokens.join(" ");
        let trimmed_segment = norm_segment.trim();

        let allowed = allowed_prefixes.iter().any(|prefix| {
            trimmed_segment == prefix || trimmed_segment.starts_with(&format!("{prefix} "))
        });

        if !allowed {
            return Some(trimmed_segment.to_string());
        }
    }

    None
}

fn read_only_job() -> Option<String> {
    let flag = std::env::var("BEE_HERDING_READ_ONLY").unwrap_or_default();
    let job = std::env::var("BEE_HERDING_JOB_ID").unwrap_or_default();
    let job = job.trim();
    (!flag.trim().is_empty() && !job.is_empty()).then(|| job.to_string())
}

fn job_mailbox(cwd: &Path, job: &str) -> Option<PathBuf> {
    if job == "." || job == ".." || job.contains(['/', '\\']) {
        return None;
    }
    let root = match crate::roots::resolve_roots_core(cwd) {
        crate::roots::Resolution::Ordinary { store_root, .. } => store_root,
        crate::roots::Resolution::LinkedValid { main_root, .. } => main_root,
        _ => cwd.to_path_buf(),
    };
    Some(root.join(".bee").join("mailbox").join(job))
}

fn settle(path: &Path) -> PathBuf {
    let mut lexical = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                lexical.pop();
            }
            other => lexical.push(other),
        }
    }
    let mut existing = lexical;
    let mut tail = Vec::new();
    while !existing.exists() {
        let Some(name) = existing.file_name().map(|n| n.to_os_string()) else { break };
        tail.push(name);
        existing.pop();
    }
    let mut out = existing.canonicalize().unwrap_or(existing);
    for name in tail.iter().rev() {
        out.push(name);
    }
    out
}

fn in_mailbox(target: &str, cwd: &Path, mailbox: Option<&Path>) -> bool {
    let Some(mailbox) = mailbox else { return false };
    if target.is_empty() || target.contains(['$', '~', '*', '?', '[']) {
        return false;
    }
    let raw = Path::new(target);
    let absolute = if raw.is_absolute() { raw.to_path_buf() } else { cwd.join(raw) };
    let target = settle(&absolute);
    let zone = settle(mailbox);
    target != zone && target.starts_with(&zone)
}

pub(crate) fn evaluate_read_only(
    tool_name: &str,
    input: Option<&Map<String, Value>>,
    cwd: &Path,
    mailbox: Option<&Path>,
) -> Option<String> {
    let field = |keys: &[&str]| {
        input
            .and_then(|m| keys.iter().find_map(|k| m.get(*k).and_then(Value::as_str)))
            .unwrap_or("")
            .to_string()
    };
    if WRITE_TOOLS.contains(&tool_name) {
        let target = field(&["file_path", "notebook_path", "path"]);
        if in_mailbox(&target, cwd, mailbox) {
            return None;
        }
        return Some(read_only_denial(&format!("`{tool_name}` on `{target}`"), mailbox));
    }
    if tool_name == "Bash" {
        return read_only_bash_blocked(&field(&["command", "cmd"]), cwd, mailbox)
            .map(|blocked| read_only_denial(&blocked, mailbox));
    }
    None
}

fn read_only_bash_blocked(command: &str, cwd: &Path, mailbox: Option<&Path>) -> Option<String> {
    if ["$(", "`", "<(", ">("].iter().any(|s| command.contains(s)) {
        return Some("command substitution".to_string());
    }
    let stripped = match strip_redirects(&fence_heredocs(command), cwd, mailbox) {
        Ok(s) => s,
        Err(target) => return Some(format!("a redirect to {target} outside the job mailbox")),
    };
    let deep = tokenize_deep(&stripped);
    if deep.truncated {
        return Some("a command nested too deep to read".to_string());
    }
    if deep.tokens != tokenize(&stripped) {
        return Some("`sh -c` or `eval`".to_string());
    }
    for segment in deep.tokens.split(|t| is_separator(t)) {
        if let Some(blocked) = read_only_segment_blocked(segment, cwd, mailbox) {
            return Some(blocked);
        }
    }
    mode_b_blocked(&stripped, READ_ONLY_ALLOWED).map(|blocked| format!("`{blocked}`"))
}

fn strip_redirects(command: &str, cwd: &Path, mailbox: Option<&Path>) -> Result<String, String> {
    let chars: Vec<char> = command.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(command.len());
    let mut i = 0usize;
    while i < n {
        let ch = chars[i];
        if ch == '"' || ch == '\'' {
            let end = chars[i + 1..].iter().position(|&c| c == ch).map_or(n, |p| p + i + 2);
            out.extend(&chars[i..end]);
            i = end;
            continue;
        }
        if ch == '\\' && i + 1 < n {
            out.push(ch);
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if ch == '\n' || ch == '\r' {
            out.push(';');
            i += 1;
            continue;
        }
        if ch != '>' {
            out.push(ch);
            i += 1;
            continue;
        }
        i += 1;
        if i < n && (chars[i] == '>' || chars[i] == '|') {
            i += 1;
        }
        while i < n && (chars[i] == ' ' || chars[i] == '\t') {
            i += 1;
        }
        let duplicate = i < n && chars[i] == '&';
        if duplicate {
            i += 1;
        }
        let mut target = String::new();
        while i < n && !chars[i].is_whitespace() && !matches!(chars[i], ';' | '&' | '|' | '<' | '>' | '(' | ')') {
            let c = chars[i];
            if c == '"' || c == '\'' {
                let close = chars[i + 1..].iter().position(|&q| q == c).map_or(n, |p| p + i + 1);
                target.extend(&chars[i + 1..close]);
                i = (close + 1).min(n);
                continue;
            }
            target.push(c);
            i += 1;
        }
        let fd_copy = duplicate && (target == "-" || (!target.is_empty() && target.chars().all(|c| c.is_ascii_digit())));
        if !fd_copy && !in_mailbox(&target, cwd, mailbox) {
            return Err(format!("`{target}`"));
        }
        out.push(' ');
    }
    Ok(out)
}

fn read_only_segment_blocked(segment: &[String], cwd: &Path, mailbox: Option<&Path>) -> Option<String> {
    let (head, args) = segment.split_first()?;
    match command_basename(head).as_str() {
        "rg" if args.iter().any(|a| a == "--pre" || a.starts_with("--pre=")) => Some("`rg --pre`".to_string()),
        "fd" if args.iter().any(|a| fd_exec_flag(a)) => Some("`fd --exec`".to_string()),
        "git" if args.iter().any(|a| a == "--output" || a.starts_with("--output=")) => {
            Some("`git --output`".to_string())
        }
        "mv" => {
            let unsafe_flag = args.iter().any(|a| a.starts_with('-') && !MV_SAFE_FLAGS.contains(&a.as_str()));
            let paths: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
            let outside = paths.len() < 2 || paths.iter().any(|p| !in_mailbox(p, cwd, mailbox));
            (unsafe_flag || outside).then(|| "`mv` outside the job mailbox".to_string())
        }
        _ => None,
    }
}

fn fd_exec_flag(arg: &str) -> bool {
    matches!(arg, "--exec" | "--exec-batch")
        || arg.starts_with("--exec=")
        || arg.starts_with("--exec-batch=")
        || (arg.starts_with('-') && !arg.starts_with("--") && arg.contains(['x', 'X']))
}

fn read_only_denial(blocked: &str, mailbox: Option<&Path>) -> String {
    let zone = mailbox.map(|m| format!(" ({})", m.display())).unwrap_or_default();
    format!(
        "bee worker-guard denied this call: {blocked} is not allowed — this herded run is read-only. You may still write your ack, report and result files in your job mailbox{zone}. FIX: put a blocker in your result file, or send it with `bee herding notify`."
    )
}

fn mode_a_denial(blocked: &str) -> String {
    format!(
        "bee worker-guard denied this shell command: {blocked} is not allowed for a Paseo worker. FIX: stop and report blocked; the leader does this step."
    )
}

fn mode_b_denial(blocked: &str) -> String {
    format!(
        "bee worker-guard denied this shell command: `{blocked}` is outside the supervisor allowlist. FIX: run only commands permitted by the supervisor allowlist."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_a_denies_table() {
        assert!(evaluate_mode_a("git push").is_some());
        assert!(evaluate_mode_a("gh pr create").is_some());
        assert!(evaluate_mode_a("pi -p x").is_some());
        assert!(evaluate_mode_a("claude").is_some());
        assert!(evaluate_mode_a("paseo ls").is_some());
        assert!(evaluate_mode_a("bee dispatch prepare").is_some());
        assert!(evaluate_mode_a(".bee/bin/bee herding run").is_some());
        assert!(evaluate_mode_a("bee gate").is_some());
        assert!(evaluate_mode_a("cargo test && git push").is_some());
    }

    #[test]
    fn mode_a_allows_table() {
        assert!(evaluate_mode_a("gh pr view").is_none());
        assert!(evaluate_mode_a("cargo test").is_none());
        assert!(evaluate_mode_a("git commit").is_none());
        assert!(evaluate_mode_a("git status").is_none());
        assert!(evaluate_mode_a("bee cells list").is_none());
    }

    #[test]
    fn mode_b_table() {
        let allowed = "Bash(.bee/bin/bee status:*),Read";
        assert!(evaluate_mode_b(".bee/bin/bee status --json", allowed).is_none());
        assert!(evaluate_mode_b("rm -rf x", allowed).is_some());
        assert!(evaluate_mode_b(".bee/bin/bee status; rm x", allowed).is_some());
        assert!(evaluate_mode_b(".bee/bin/bee status > f", allowed).is_some());
    }

    fn nested_wrapper(depth: usize) -> String {
        let mut command = "rg x".to_string();
        for _ in 0..depth {
            let escaped: String = command
                .chars()
                .flat_map(|c| if matches!(c, ' ' | '\\' | '\'' | '"' | ';' | '&' | '|') { vec!['\\', c] } else { vec![c] })
                .collect();
            command = format!("sh -c {escaped}");
        }
        command
    }

    #[test]
    fn read_only_table() {
        let tmp = tempfile::tempdir().unwrap();
        let cwd = tmp.path();
        let mailbox = cwd.join(".bee").join("mailbox").join("job-1");
        let mb = mailbox.display().to_string();
        let bash = |command: &str| {
            let input = serde_json::json!({ "command": command });
            evaluate_read_only("Bash", input.as_object(), cwd, Some(&mailbox))
        };
        let write = |tool: &str, path: &str| {
            let input = serde_json::json!({ "file_path": path, "content": "x" });
            evaluate_read_only(tool, input.as_object(), cwd, Some(&mailbox))
        };

        for allowed in [
            "rg -n foo src".to_string(),
            "git log --oneline -5".to_string(),
            "git status 2>&1".to_string(),
            ".bee/bin/bee herding notify --kind progress --text hi".to_string(),
            format!("mv {mb}/result-1.json.tmp {mb}/result-1.json"),
            format!("cat src/a.rs > {mb}/report.md"),
            format!("cat > {mb}/ack-1.json <<'EOF'\n{{\"ok\": true}}\nEOF"),
        ] {
            assert!(bash(&allowed).is_none(), "expected allow: {allowed} -> {:?}", bash(&allowed));
        }
        assert!(write("Write", &format!("{mb}/result-1.json")).is_none());
        assert!(write("Write", ".bee/mailbox/job-1/result-1.json").is_none());
        assert!(write("MultiEdit", &format!("{mb}/report.md")).is_none());
        assert!(evaluate_read_only("Read", None, cwd, Some(&mailbox)).is_none());

        for denied in [
            "cat $(echo src/a.rs)".to_string(),
            "cat `echo src/a.rs`".to_string(),
            nested_wrapper(6),
            "cat a.rs > src/b.rs".to_string(),
            "cat a.rs >> /tmp/out.txt".to_string(),
            format!("cat a.rs > {mb}/../../x"),
            "rg foo | sed s/a/b/".to_string(),
            "rg --pre cat foo".to_string(),
            "rg --pre=cat foo".to_string(),
            "fd -x rm".to_string(),
            "fd -X rm".to_string(),
            "fd --exec rm".to_string(),
            "fd --exec-batch rm".to_string(),
            "git log --output=/tmp/x".to_string(),
            "git diff --output /tmp/x".to_string(),
            "sed -i s/a/b/ src/a.rs".to_string(),
            "find . -delete".to_string(),
            "rg -l foo | xargs rm".to_string(),
            "cat a.rs | tee src/b.rs".to_string(),
            "env rm a.rs".to_string(),
            "sh -c 'cat a.rs'".to_string(),
            "eval cat a.rs".to_string(),
            "rg foo\nrm a.rs".to_string(),
            "rm a.rs".to_string(),
            format!("mv {mb}/a src/a.rs"),
            format!("mv --target-directory=src {mb}/a"),
        ] {
            let reason = bash(&denied).unwrap_or_else(|| panic!("expected deny: {denied}"));
            assert!(reason.contains("read-only") && reason.contains("bee herding notify"), "{reason}");
        }
        assert!(bash(&nested_wrapper(6)).unwrap().contains("nested too deep"));
        assert!(bash("sh -c 'cat a.rs'").unwrap().contains("`sh -c` or `eval`"));
        assert!(write("Write", "src/a.rs").is_some());
        assert!(write("Edit", &format!("{mb}/../job-2/result-1.json")).is_some());
        assert!(write("Write", "").is_some());
        assert!(evaluate_read_only("Write", serde_json::json!({ "file_path": format!("{mb}/r") }).as_object(), cwd, None).is_some());
    }

    #[test]
    fn read_only_env_gates_non_bash_tools() {
        let _guard = crate::hooks::herding_env_lock();
        let keys = ["BEE_HERDING_WORKER", "PASEO_AGENT_ID", "BEE_SUPERVISOR_ALLOWED", "BEE_HERDING_READ_ONLY", "BEE_HERDING_JOB_ID"];
        let prior: Vec<_> = keys.iter().map(|k| std::env::var_os(k)).collect();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        std::fs::write(tmp.path().join(".bee").join("onboarding.json"), "{}").unwrap();
        let code = |path: &str| {
            let payload = serde_json::json!({
                "tool_name": "Write",
                "cwd": tmp.path().display().to_string(),
                "tool_input": { "file_path": path, "content": "x" }
            });
            match run(&[], &payload.to_string()) {
                Outcome::Done(code) => format!("{code:?}"),
                Outcome::Delegate => "delegate".to_string(),
            }
        };

        for k in keys {
            unsafe { std::env::remove_var(k) };
        }
        unsafe { std::env::set_var("BEE_HERDING_JOB_ID", "job-1") };
        let free = code("src/a.rs");
        unsafe { std::env::set_var("BEE_HERDING_READ_ONLY", "1") };
        let outside = code("src/a.rs");
        let inside = code(".bee/mailbox/job-1/result-1.json");

        for (k, v) in keys.iter().zip(prior) {
            match v {
                Some(v) => unsafe { std::env::set_var(k, v) },
                None => unsafe { std::env::remove_var(k) },
            }
        }
        let ok = format!("{:?}", ExitCode::SUCCESS);
        assert_eq!(free, ok);
        assert_eq!(outside, format!("{:?}", ExitCode::from(2)));
        assert_eq!(inside, ok);
    }

    #[test]
    fn pane_only_marker_allows_git_push() {
        let _guard = crate::hooks::herding_env_lock();
        let prior_worker = std::env::var_os("BEE_HERDING_WORKER");
        let prior_paseo = std::env::var_os("PASEO_AGENT_ID");
        let prior_sup = std::env::var_os("BEE_SUPERVISOR_ALLOWED");

        unsafe {
            std::env::set_var("BEE_HERDING_WORKER", "1");
            std::env::remove_var("PASEO_AGENT_ID");
            std::env::remove_var("BEE_SUPERVISOR_ALLOWED");
        }

        let payload = serde_json::json!({
            "tool_name": "Bash",
            "tool_input": {
                "command": "git push"
            }
        });
        let outcome = run(&[], &payload.to_string());
        match outcome {
            Outcome::Done(code) => assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::SUCCESS)),
            Outcome::Delegate => panic!("expected ExitCode::SUCCESS"),
        }

        match prior_worker {
            Some(v) => unsafe { std::env::set_var("BEE_HERDING_WORKER", v) },
            None => unsafe { std::env::remove_var("BEE_HERDING_WORKER") },
        }
        match prior_paseo {
            Some(v) => unsafe { std::env::set_var("PASEO_AGENT_ID", v) },
            None => unsafe { std::env::remove_var("PASEO_AGENT_ID") },
        }
        match prior_sup {
            Some(v) => unsafe { std::env::set_var("BEE_SUPERVISOR_ALLOWED", v) },
            None => unsafe { std::env::remove_var("BEE_SUPERVISOR_ALLOWED") },
        }
    }

    #[test]
    fn mode_a_run_denies_git_push_for_paseo_worker() {
        let _guard = crate::hooks::herding_env_lock();
        let prior_worker = std::env::var_os("BEE_HERDING_WORKER");
        let prior_paseo = std::env::var_os("PASEO_AGENT_ID");
        let prior_sup = std::env::var_os("BEE_SUPERVISOR_ALLOWED");

        unsafe {
            std::env::set_var("BEE_HERDING_WORKER", "1");
            std::env::set_var("PASEO_AGENT_ID", "agent-1");
            std::env::remove_var("BEE_SUPERVISOR_ALLOWED");
        }

        let payload = serde_json::json!({
            "tool_name": "Bash",
            "tool_input": {
                "command": "git push"
            }
        });
        let outcome = run(&[], &payload.to_string());
        match outcome {
            Outcome::Done(code) => assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(2))),
            Outcome::Delegate => panic!("expected ExitCode 2"),
        }

        match prior_worker {
            Some(v) => unsafe { std::env::set_var("BEE_HERDING_WORKER", v) },
            None => unsafe { std::env::remove_var("BEE_HERDING_WORKER") },
        }
        match prior_paseo {
            Some(v) => unsafe { std::env::set_var("PASEO_AGENT_ID", v) },
            None => unsafe { std::env::remove_var("PASEO_AGENT_ID") },
        }
        match prior_sup {
            Some(v) => unsafe { std::env::set_var("BEE_SUPERVISOR_ALLOWED", v) },
            None => unsafe { std::env::remove_var("BEE_SUPERVISOR_ALLOWED") },
        }
    }
}
