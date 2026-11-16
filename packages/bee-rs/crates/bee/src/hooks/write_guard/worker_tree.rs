use super::*;
use crate::hooks::adapter::{read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::state::hook_enabled;
use serde_json::{Map, Value};

const UNRESOLVED_REMEDY: &str =
    "FIX: inspect/restore the reservation store (.bee/reservations.json), then retry. \
A genuinely path-scoped `git commit -- <your paths>` is allowed too.";

pub(crate) fn run_herded(argv: &[String], stdin: &str) -> Outcome {
    let argv = argv.to_vec();
    let stdin = stdin.to_string();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = read_hook_context(HOOK_NAME, &argv, &stdin);
        emit_for(&ctx)
    })) {
        Ok(emit) => flush(emit, None),
        Err(_) => Outcome::Delegate,
    }
}

pub(crate) fn emit_for(ctx: &HookContext) -> Emit {
    let mut emit = Emit::default();
    let denial = team_guard_denial(ctx).or_else(|| whole_tree_denial(ctx));
    clear_corrupt_json_warnings();
    if let Some(reason) = denial {
        emit.stderr.push_str(&reason);
        emit.code = 2;
    }
    emit
}

fn team_guard_denial(ctx: &HookContext) -> Option<String> {
    if ctx.worktree_resolution == "linked-invalid" {
        return None;
    }
    let root_pb = ctx.root.clone()?;
    let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| root_pb.clone());
    if !store_root_pb.join(".bee").join("onboarding.json").is_file() {
        return None;
    }
    if !hook_enabled(&store_root_pb, HOOK_NAME) {
        return None;
    }
    let tool_name = match first_truthy(&ctx.payload, &["tool_name", "toolName"]) {
        Some(Value::String(s)) => s.clone(),
        _ => return None,
    };
    if !matches!(
        tool_name.as_str(),
        "Edit" | "Write" | "MultiEdit" | "apply_patch" | "ApplyPatch" | "Bash" | "exec"
    ) {
        return None;
    }
    let tool_input: Map<String, Value> = match ctx.payload.get("tool_input") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    let root = root_pb.to_string_lossy().into_owned();
    let cwd = ctx.cwd.to_string_lossy().into_owned();
    if let Some(reason) = global_config_write_denial(&tool_name, &tool_input, &root, &cwd) {
        return Some(reason);
    }
    let raw_paths: Vec<String> = match tool_name.as_str() {
        "apply_patch" | "ApplyPatch" => apply_patch_text(&tool_input)
            .map(|patch| extract_apply_patch_targets(&patch))
            .unwrap_or_default(),
        "Bash" | "exec" => match first_truthy(&tool_input, &["command", "cmd"]) {
            Some(Value::String(command)) => extract_bash_targets(command).paths,
            _ => Vec::new(),
        },
        _ => match tool_input.get("file_path") {
            Some(Value::String(path)) => vec![path.clone()],
            _ => Vec::new(),
        },
    };
    raw_paths.into_iter().find_map(|raw| {
        let rel = canonical_rel_path(&root, &cwd, Some(&Value::String(raw)))
            .ok()
            .flatten()?;
        project_config_write_denial(&tool_name, &tool_input, &root_pb, &rel)
    })
}

fn unresolved(classified: &TreeVerbClass, reason: &str) -> String {
    concurrent_tree_refusal(
        &classified.verb,
        classified.why,
        &format!(
            "the live-worker count could not be resolved ({reason}), which is treated as more than one worker"
        ),
        UNRESOLVED_REMEDY,
    )
}

fn whole_tree_denial(ctx: &HookContext) -> Option<String> {
    if ctx.worktree_resolution == "linked-invalid" {
        return None;
    }
    let root_pb = ctx.root.clone()?;
    let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| root_pb.clone());
    if !store_root_pb.join(".bee").join("onboarding.json").is_file() {
        return None;
    }
    if !hook_enabled(&store_root_pb, HOOK_NAME) {
        return None;
    }
    let tool_name = match first_truthy(&ctx.payload, &["tool_name", "toolName"]) {
        Some(Value::String(s)) => s.clone(),
        _ => return None,
    };
    if !matches!(tool_name.as_str(), "Bash" | "exec") {
        return None;
    }
    let tool_input: Map<String, Value> = match ctx.payload.get("tool_input") {
        Some(Value::Object(m)) => m.clone(),
        _ => return None,
    };
    let command = match first_truthy(&tool_input, &["command", "cmd"]) {
        Some(Value::String(s)) => s.clone(),
        _ => return None,
    };
    if js_trim(&command).is_empty() {
        return None;
    }
    let deep = tokenize_deep(&fence_heredocs(&command));
    if deep.truncated {
        return None;
    }
    let verbs: Vec<TreeVerbClass> = find_git_invocations(&deep.tokens)
        .into_iter()
        .filter_map(|invocation| {
            classify_concurrent_tree_verb(invocation.subcommand.as_deref(), &invocation.rest)
        })
        .collect();
    let first = verbs.first()?;
    let store_root = store_root_pb.to_string_lossy().into_owned();
    let control_root_s = ctx.control_root.as_ref().map(|p| p.to_string_lossy().into_owned());
    let topo = match resolve_write_topology(&store_root, control_root_s.as_deref()) {
        Ok(topo) => topo,
        Err(_) => return Some(unresolved(first, "the write topology is unreadable")),
    };
    let count = match resolve_live_worker_count(&store_root, &topo.control_root, &topo.ctx) {
        Ok(count) => count,
        Err(_) => return Some(unresolved(first, "the worker records are unreadable")),
    };
    for classified in &verbs {
        match &count {
            WorkerCount::Unresolved(reason) => return Some(unresolved(classified, reason)),
            WorkerCount::Resolved(n) if *n > 1 => {
                return Some(concurrent_tree_refusal(
                    &classified.verb,
                    classified.why,
                    &format!("{n} workers are live in this checkout"),
                    CONCURRENT_TREE_TEMP_INDEX_REMEDY,
                ));
            }
            _ => {}
        }
    }
    None
}
