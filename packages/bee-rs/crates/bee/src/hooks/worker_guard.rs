use super::Outcome;
use super::write_guard::{
    command_basename, outward_segment_head, codex_read_only_exec, gh_read_only_form,
    gh_sub_and_args, fence_heredocs, is_separator, tokenize_deep, find_git_invocations,
};
use serde_json::Value;
use std::process::ExitCode;

pub fn run(_argv: &[String], stdin: &str) -> Outcome {
    let herding_worker = std::env::var("BEE_HERDING_WORKER").unwrap_or_default();
    let paseo_agent_id = std::env::var("PASEO_AGENT_ID").unwrap_or_default();
    let supervisor_allowed = std::env::var("BEE_SUPERVISOR_ALLOWED").unwrap_or_default();

    let mode_a = !herding_worker.is_empty() && !paseo_agent_id.is_empty();
    let mode_b = !supervisor_allowed.is_empty();

    if !mode_a && !mode_b {
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
    if command.contains("$(") || command.contains('`') || command.contains('>') {
        return Some(mode_b_denial(command));
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
        return Some(mode_b_denial(command));
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
            return Some(mode_b_denial(trimmed_segment));
        }
    }

    None
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
