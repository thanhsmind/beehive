// the main orchestration
//
// Split out of the single 5.9k-line hooks/write_guard.rs. Code unchanged; only module placement and item visibility moved.
#![allow(unused_imports)]

use super::*;
use crate::hooks::adapter::{append_hook_log, now_iso, read_hook_context, HookContext};
use crate::hooks::Outcome;
use crate::jsjson;
use crate::state::hook_enabled;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ─── main orchestration ─────────────────────────────────────────────────────

pub fn run(argv: &[String], stdin: &str) -> Outcome {
    let argv = argv.to_vec();
    let stdin = stdin.to_string();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let ctx = read_hook_context(HOOK_NAME, &argv, &stdin);
        run_native(&ctx).map(|emit| (emit, ctx.source))
    })) {
        Ok(Ok((emit, source))) => flush(emit, source),
        Ok(Err(Nd)) => Outcome::Delegate,
        Err(_) => Outcome::Delegate, // a native panic is never a verdict
    }
}

pub(crate) fn first_truthy<'a>(map: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(v) = map.get(*key) {
            if truthy(v) {
                return Some(v);
            }
        }
    }
    None
}

pub(crate) fn run_native(ctx: &HookContext) -> R<Emit> {
    run_native_with_roots(ctx, &HarnessRoots::detect())
}

/// gh-1 (E1): the harness allowlist roots ride in as a parameter so tests can
/// inject their own bases without env mutation; production always passes
/// HarnessRoots::detect() via run_native above.
pub(crate) fn run_native_with_roots(ctx: &HookContext, harness_roots: &HarnessRoots) -> R<Emit> {
    clear_corrupt_json_warnings(); // one queue per evaluation (tests reuse the thread)
    let mut emit = Emit::default();
    let Some(root_pb) = ctx.root.clone() else {
        return Ok(emit);
    };
    let root = root_pb.to_string_lossy().into_owned();
    np_check_modelable(&root)?;
    let payload = &ctx.payload;

    // toolName = payload.tool_name || payload.toolName || ""
    let tool_name: String = match first_truthy(payload, &["tool_name", "toolName"]) {
        Some(Value::String(s)) => s.clone(),
        Some(_) => "\u{0}non-string-tool\u{0}".to_string(), // truthy non-string never matches a tool
        None => String::new(),
    };
    let is_write_tool = matches!(tool_name.as_str(), "Edit" | "Write" | "MultiEdit");
    let is_apply = matches!(tool_name.as_str(), "apply_patch" | "ApplyPatch");
    let is_read_tool = matches!(tool_name.as_str(), "Read" | "Glob" | "Grep");
    let write_capable = is_write_tool || tool_name == "Bash" || is_apply;

    if write_capable && ctx.worktree_resolution == "linked-invalid" {
        emit.stderr.push_str(
            "bee worktree guard denied this write: WORKTREE_LINK_INVALID — linked worktree metadata could not be validated. \
FIX: repair or recreate the Git worktree before retrying; no worktree-local .bee store is trusted.",
        );
        emit.code = 2;
        return Ok(emit);
    }

    let store_root_pb = ctx.store_root.clone().unwrap_or_else(|| root_pb.clone());
    let store_root = store_root_pb.to_string_lossy().into_owned();
    np_check_modelable(&store_root)?;
    // "Is bee installed at this store root?" — the onboarding marker, which
    // every install writes and nothing else does.
    if !store_root_pb.join(".bee").join("onboarding.json").is_file() {
        return Ok(emit);
    }

    if !hook_enabled(&store_root_pb, HOOK_NAME) {
        return Ok(emit);
    }

    let tool_input: Map<String, Value> = match payload.get("tool_input") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    let cwd = ctx.cwd.to_string_lossy().into_owned();

    let mut denial: Option<String> = None;
    let mut fixed_ask: Option<(Value, Vec<String>)> = None;
    let mut reservation_warnings: Vec<String> = Vec::new();
    let control_root_s: Option<String> =
        ctx.control_root.as_ref().map(|p| p.to_string_lossy().into_owned());

    if tool_name == "AskUserQuestion" {
        match check_ask_user_question(&tool_input)? {
            AskResult::Allow => {}
            AskResult::Deny(reason) => denial = Some(reason),
            AskResult::Fixed { fixed, notes } => fixed_ask = Some((fixed, notes)),
        }
    } else if is_read_tool {
        let raw = first_truthy(&tool_input, &["file_path", "path"]);
        if let Some(rel) = lexical_rel_path(&root, &cwd, raw)? {
            match check_read(&rel) {
                ReadVerdict::Deny { reason, marker } => {
                    let mut parts = vec![reason];
                    if let Some(m) = marker {
                        parts.push(m);
                    }
                    denial = Some(parts.join("\n"));
                }
                ReadVerdict::Allow => {
                    if tool_name == "Read"
                        && !tool_input.contains_key("offset")
                        && !tool_input.contains_key("limit")
                    {
                        let config = read_config(&store_root_pb)?;
                        let threshold = resolve_max_read_lines(&config);
                        let abs = Path::new(&root).join(rel.replace('/', &SEP.to_string()));
                        if let Some(reason) = check_read_size_denial(&abs, &rel, threshold) {
                            denial = Some(reason);
                        }
                    }
                }
            }
        }
    } else if write_capable {
        let state = read_state(&store_root_pb)?;
        let agent_name = infer_agent_name(payload, &tool_input);
        let session_id = str_trim_nonempty(payload.get("session_id"));
        let mut rel_paths: Vec<String> = Vec::new();
        let mut shared_candidates: Vec<(String, String)> = Vec::new(); // (rel, abs)

        if is_apply {
            match apply_patch_text(&tool_input) {
                None => {
                    push_gap(
                        &mut emit,
                        &root_pb,
                        "applypatch-unparsed",
                        "apply_patch intercepted but no canonical patch envelope found in tool_input".to_string(),
                    );
                }
                Some(patch_text) => {
                    let targets = extract_apply_patch_targets(&patch_text);
                    for t in &targets {
                        if let Some(r) =
                            canonical_rel_path(&root, &cwd, Some(&Value::String(t.clone())))?
                        {
                            rel_paths.push(r);
                        }
                    }
                    if targets.is_empty() || rel_paths.len() < targets.len() {
                        let detail = if targets.is_empty() {
                            "apply_patch intercepted but no Add/Update/Delete/Move/\"Move to\" target line could be parsed from the patch body".to_string()
                        } else {
                            format!(
                                "apply_patch intercepted but {} of {} target(s) could not be proved inside the repo",
                                targets.len() - rel_paths.len(),
                                targets.len()
                            )
                        };
                        push_gap(&mut emit, &root_pb, "applypatch-unparsed", detail);
                        denial = Some(
                            "bee apply_patch guard: this patch's target set could not be fully proved inside the repo — \
denying rather than risking an unchecked write. \
FIX: use canonical \"*** Add File:\", \"*** Update File:\", \"*** Delete File:\", and \"*** Move to:\" \
lines naming plain in-repo relative paths (no path traversal, no unresolvable escapes), then resubmit."
                                .to_string(),
                        );
                    }
                }
            }
        } else if tool_name == "Bash" {
            let command = match tool_input.get("command") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if !command.is_empty() {
                let targets = extract_bash_targets(&command);
                // D4: which unresolvable-shell-syntax refusal wording a cand
                // gets — shell syntax, brace expansion, or cd opacity are
                // mutually exclusive per-candidate classifications, checked
                // in that order.
                #[derive(Clone, Copy)]
                enum Unresolved {
                    ShellSyntax,
                    BraceExpansion,
                    CdOpaque,
                }
                struct Cand {
                    raw: String,
                    canonical: Option<String>,
                    rel: Option<String>,
                    delegate: bool,
                    unresolved: Option<Unresolved>,
                }
                let mut cands: Vec<Cand> = Vec::new();
                for (idx, p) in targets.paths.iter().enumerate() {
                    // D4/D5: shell-syntax classification lives on the Bash
                    // surface only. A token still carrying unexpanded `$` or
                    // a backquote is never handed to canonical_rel_path (it
                    // is not a literal path) and never reaches
                    // companion_mount_rel — with .bee/companion-session.json
                    // present that call would raise Err(Nd) and delegate
                    // (fail-open), silently swapping the D2 denial for an
                    // allow. Marking it unresolved here keeps it inside the
                    // ordinary wall-failed-candidate flow below, which always
                    // ends in a denial for this cand.
                    if has_unexpanded_shell_syntax(p) {
                        cands.push(Cand {
                            raw: p.clone(),
                            canonical: None,
                            rel: None,
                            delegate: false,
                            unresolved: Some(Unresolved::ShellSyntax),
                        });
                        continue;
                    }
                    // D4: a brace group (`{a,b}`, `{1..9}`) is expanded by
                    // the shell into several literal targets before this
                    // write-verb ever runs — same reasoning and same
                    // unresolved-candidate path as shell syntax above. Plain
                    // glob characters (`*`, `?`, `[`) are deliberately NOT
                    // checked here — globs stay exactly as today.
                    if has_brace_expansion(p) {
                        cands.push(Cand {
                            raw: p.clone(),
                            canonical: None,
                            rel: None,
                            delegate: false,
                            unresolved: Some(Unresolved::BraceExpansion),
                        });
                        continue;
                    }
                    // D4: a target extracted after a `cd` earlier in this
                    // compound command no longer resolves against the
                    // directory the guard tracked — `extract_bash_targets`
                    // marks this in `targets.cd_opaque`, parallel to `paths`.
                    if targets.cd_opaque.get(idx).copied().unwrap_or(false) {
                        cands.push(Cand {
                            raw: p.clone(),
                            canonical: None,
                            rel: None,
                            delegate: false,
                            unresolved: Some(Unresolved::CdOpaque),
                        });
                        continue;
                    }
                    let canonical =
                        canonical_rel_path(&root, &cwd, Some(&Value::String(p.clone())))?;
                    // D2: an Err(Nd) from THIS candidate must never abort the
                    // loop and skip its siblings — that would let one
                    // undecidable target swallow a denial already earned by
                    // another. Capture it on the candidate instead; the
                    // escalation to Outcome::Delegate happens once, below,
                    // only after every candidate has had its chance to deny.
                    let mut delegate = false;
                    let rel = match &canonical {
                        Some(c) => Some(c.clone()),
                        None => match companion_mount_rel(&root) {
                            Ok(rel) => rel, // marker absent → still native
                            Err(Nd) => {
                                delegate = true; // marker present → undecidable
                                None
                            }
                        },
                    };
                    if rel.is_none() && !delegate {
                        // gmr-3: memory-root consult only for wall-failed
                        // targets; a declared root delegates.
                        if memory_root_hit(&store_root_pb).is_err() {
                            delegate = true;
                        }
                    }
                    cands.push(Cand { raw: p.clone(), canonical, rel, delegate, unresolved: None });
                }
                rel_paths = cands.iter().filter_map(|c| c.rel.clone()).collect();
                for c in &cands {
                    if let Some(canonical) = &c.canonical {
                        shared_candidates
                            .push((canonical.clone(), lexical_abs_target(&root, &cwd, &c.raw)?));
                    }
                }
                // memory-root hits are always 0 natively (a declared root
                // delegates above), so the count equation reduces to:
                if rel_paths.len() != targets.paths.len() {
                    // gh-1 (E1): a wall-failed target whose RESOLVED location
                    // sits inside a harness-owned surface is exempt from the
                    // containment deny — and ONLY that deny; an exempt target
                    // never enters rel_paths, so every check below keeps its
                    // existing order and reach over in-repo targets.
                    //
                    // D2: a delegate-flagged candidate is excluded from this
                    // native-denial search entirely — native code could not
                    // decide it, so it must never manufacture a denial for
                    // it. It only ever reaches the escalation below.
                    let mut first_failing: Option<&Cand> = None;
                    for c in cands.iter().filter(|c| c.rel.is_none() && !c.delegate) {
                        // D4: a shell-syntax, brace-expansion, or cd-opaque
                        // cand never reaches the harness allowlist check: it
                        // was never resolved as a path, so there is no
                        // resolved location to test against a harness root.
                        if c.unresolved.is_some()
                            || !harness_allowlisted_target(
                                harness_roots,
                                &root,
                                &cwd,
                                &Value::String(c.raw.clone()),
                            )?
                        {
                            first_failing = Some(c);
                            break;
                        }
                    }
                    if let Some(c) = first_failing {
                        // D1/D2/D4: a target that was never a literal path —
                        // still-unexpanded shell syntax, a brace group, or a
                        // target stranded after a `cd` — names its own
                        // resolution failure and quotes the raw token,
                        // instead of the containment-denial wording.
                        denial = Some(match c.unresolved {
                            Some(Unresolved::ShellSyntax) => unresolvable_bash_target_message(&c.raw),
                            Some(Unresolved::BraceExpansion) => {
                                brace_expansion_bash_target_message(&c.raw)
                            }
                            Some(Unresolved::CdOpaque) => cd_opaque_bash_target_message(&c.raw),
                            None => {
                                let enriched = describe_cross_worktree_target(
                                    &root,
                                    &cwd,
                                    &Value::String(c.raw.clone()),
                                )?;
                                enriched.unwrap_or_else(|| {
                                    GENERIC_BASH_CONTAINMENT_MESSAGE.to_string()
                                })
                            }
                        });
                    }
                }
                if denial.is_none() && cands.iter().any(|c| c.delegate) {
                    // D2: only once no sibling candidate earned a native
                    // denial does an undecidable candidate get to hand the
                    // whole request to the companion — the same escape point
                    // as before this restructure (ahead of shared-checkout,
                    // check_write, worktree-first, and CLI-shape checks).
                    return Err(Nd);
                }
                if denial.is_none() && rel_paths.is_empty() && targets.broad_write {
                    rel_paths = vec!["**".to_string()];
                }
            }
        } else {
            // Edit / Write / MultiEdit: toolInput.file_path || ""
            let raw_v = match tool_input.get("file_path") {
                Some(v) if truthy(v) => v.clone(),
                _ => Value::String(String::new()),
            };
            let canonical = canonical_rel_path(&root, &cwd, Some(&raw_v))?;
            // D2: mirror of the Bash-branch restructure above — a
            // companion/memory-root Nd on this single target must not
            // escape before it is clear no OTHER check here already has a
            // verdict for it. With a single target there is no sibling to
            // swallow, so this only changes shape, not observed behavior:
            // the delegate escalation below fires at the same point the
            // naked `?` used to.
            let mut delegate = false;
            let rel = match &canonical {
                Some(c) => Some(c.clone()),
                None => match companion_mount_rel(&root) {
                    Ok(rel) => rel,
                    Err(Nd) => {
                        delegate = true;
                        None
                    }
                },
            };
            let mem_hit = if rel.is_none() && !delegate {
                match memory_root_hit(&store_root_pb) {
                    Ok(hit) => hit,
                    Err(Nd) => {
                        delegate = true;
                        false
                    }
                }
            } else {
                false
            };
            if let Some(rel) = rel {
                rel_paths = vec![rel];
                if let Some(canonical) = canonical {
                    if let Value::String(raw_s) = &raw_v {
                        shared_candidates
                            .push((canonical, lexical_abs_target(&root, &cwd, raw_s)?));
                    }
                }
            } else if delegate {
                return Err(Nd);
            } else if mem_hit {
                // gmr-3 D6: pre-approved short-circuit (unreachable natively —
                // a declared memory root delegates; kept for shape parity).
            } else if harness_allowlisted_target(harness_roots, &root, &cwd, &raw_v)? {
                // gh-1 (E1): the resolved target sits inside a harness-owned
                // surface (<home>/.claude/projects/ or <system-temp>/claude/)
                // — the containment deny, and only it, does not fire. The
                // target stays out of rel_paths, so reservation/hold,
                // gate-boundary, direct-edit, and secret checks keep their
                // existing order and reach.
            } else {
                let enriched = describe_cross_worktree_target(&root, &cwd, &raw_v)?;
                denial = Some(enriched.unwrap_or_else(|| GENERIC_CONTAINMENT_MESSAGE.to_string()));
            }
        }

        // wcg-2: shared nested-checkout refusal, BEFORE checkWrite.
        let mut shared_denied = false;
        if denial.is_none() {
            for (rel, abs) in &shared_candidates {
                // sfg-5: three answers, not two. The unreadable-session arm is
                // the fail-open sfg-4 left here — this loop runs for EVERY
                // Edit/Write with a resolvable target, so one truncated
                // `.bee/sessions/<id>.json` used to switch the whole guard off
                // on a real write. It denies now, in bee's own wording; see
                // `unreadable_session_refusal` for why that departs from Node.
                match is_shared_nested_checkout_target(
                    &root,
                    abs,
                    session_id.as_deref(),
                    control_root_s.as_deref(),
                )? {
                    SharedNested::No => {}
                    SharedNested::Yes => {
                        denial = Some(shared_nested_checkout_refusal(rel));
                        shared_denied = true;
                        break;
                    }
                    SharedNested::UnreadableSession(file) => {
                        denial = Some(unreadable_session_refusal(rel, &file));
                        shared_denied = true;
                        break;
                    }
                }
            }
        }

        // D3: guarded by `denial.is_none()` too — not only `!shared_denied`.
        // rel_paths on the Bash surface holds every candidate that DID
        // resolve, even when a sibling candidate already earned a denial
        // above (unresolvable shell syntax, or a resolved-but-not-contained
        // target). Without this guard, check_write's own Deny for one of
        // those resolved siblings would overwrite the first, already-decided
        // denial — the LAST reason in loop order would win instead of the
        // FIRST decisive one. Same principle as the shared-checkout arm just
        // above: once a denial exists, nothing after it may replace it.
        if !shared_denied && denial.is_none() {
            for rel in &rel_paths {
                match check_write(
                    &store_root,
                    &state,
                    rel,
                    agent_name.as_deref(),
                    session_id.as_deref(),
                    control_root_s.as_deref(),
                    &mut emit,
                )? {
                    WV::Deny(reason) => {
                        denial = Some(reason);
                        break;
                    }
                    WV::AllowWarn(warning) => reservation_warnings.push(warning),
                    WV::Allow => {}
                }
            }
        }

        if denial.is_none() && !rel_paths.is_empty() {
            // The worktree-first guard must judge the same ACTING record
            // every other write check already resolved (resolve_write_record:
            // the lane record for a lane-bound session, the default state
            // otherwise) — never the raw default state.json. Resolved once
            // here, not once per rel_path.
            //
            // A RecordResolution::Fail is never swallowed into an allow: it
            // already denied every rel_path above inside check_write's own
            // resolution, so denial.is_none() is already false by the time a
            // Fail would reach here — this arm has nothing to add on that
            // path and simply does not run.
            //
            // `&store_root`, not `&root`: check_write above resolves its own
            // topology against `&store_root` (its own `root` parameter) —
            // this call must feed resolve_write_topology the SAME value so
            // the "same acting record" claim rests on code, not on the two
            // never happening to diverge under `ordinary` today.
            let topo = resolve_write_topology(&store_root, control_root_s.as_deref())?;
            if let RecordResolution::Ok { record, .. } =
                resolve_write_record(&topo.control_root, &state, session_id.as_deref(), &mut emit)?
            {
                if let Some(reason) = check_worktree_first(
                    ctx.worktree_resolution,
                    &root,
                    &store_root_pb,
                    &record,
                    &rel_paths,
                    session_id.as_deref(),
                )? {
                    denial = Some(reason);
                }
            }
        }

        if denial.is_none() && tool_name == "Bash" {
            let command = match tool_input.get("command") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if !command.is_empty() {
                if let Some(WV::Deny(reason)) = check_git_bash_command(
                    &store_root,
                    &state,
                    &command,
                    &cwd,
                    session_id.as_deref(),
                    control_root_s.as_deref(),
                    &mut emit,
                )? {
                    denial = Some(reason);
                }
            }
        }

        if denial.is_none() && tool_name == "Bash" {
            let command = match tool_input.get("command") {
                Some(Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            if !js_trim(&command).is_empty() && has_node_inline_eval(&command) {
                return Err(Nd); // internals-reach guard: regex scan not ported
            }
        }
    }

    // Check (d) — CLI-shape validation, NATIVE (hooks/cli_shape.rs), resolved
    // against the embedded REGISTRY_PAYLOAD. check_cli_shape returns None
    // unless a bee-CLI-shaped token resolves, and it can only ASSIGN a denial
    // when none exists yet, so short-circuiting on `denial.is_none()` is the
    // intended precedence.
    if tool_name == "Bash" {
        let command = match tool_input.get("command") {
            Some(Value::String(s)) => s.clone(),
            _ => String::new(),
        };
        if !command.is_empty() && denial.is_none() {
            if let Some(reason) = crate::hooks::cli_shape::check_cli_shape(&command) {
                denial = Some(reason);
            }
        }
    }

    if let Some((fixed, notes)) = fixed_ask {
        let notes_joined = notes.join("; ");
        let mut hso = Map::new();
        hso.insert("hookEventName".into(), Value::String("PreToolUse".into()));
        // "ask", never "allow": the question tool's permission prompt IS the
        // question the human answers. An "allow" verdict pre-approves that
        // prompt away, the tool returns no selection, and the asker silently
        // falls back to its own default — the repair swallowed the question it
        // was meant to save. "ask" forces the prompt even where the permission
        // mode would skip it, and carries updatedInput with it.
        hso.insert("permissionDecision".into(), Value::String("ask".into()));
        hso.insert("permissionDecisionReason".into(), Value::String(notes_joined.clone()));
        hso.insert("updatedInput".into(), fixed);
        hso.insert(
            "additionalContext".into(),
            Value::String(format!("bee AskUserQuestion guard auto-fixed: {notes_joined}")),
        );
        let mut output = Map::new();
        output.insert("hookSpecificOutput".into(), Value::Object(hso));
        output.insert(
            "systemMessage".into(),
            Value::String(format!("bee AskUserQuestion guard: {notes_joined}")),
        );
        emit.stdout.push_str(&jsjson::stringify(&Value::Object(output)));
        emit.code = 0;
        return Ok(emit);
    }

    if denial.is_none() && !reservation_warnings.is_empty() {
        let joined = reservation_warnings.join("\n");
        let mut hso = Map::new();
        hso.insert("hookEventName".into(), Value::String("PreToolUse".into()));
        // No permissionDecision: an advisory warning must not double as a
        // pre-approval. "allow" here would wave the write past the permission
        // prompt the host would otherwise show — a soft reservation notice
        // buying the write more permission than it had. The warning rides
        // additionalContext (to the agent) and systemMessage (to the human);
        // the write itself takes the ordinary permission flow.
        hso.insert("additionalContext".into(), Value::String(joined.clone()));
        let mut output = Map::new();
        output.insert("hookSpecificOutput".into(), Value::Object(hso));
        output.insert("systemMessage".into(), Value::String(joined));
        emit.stdout.push_str(&jsjson::stringify(&Value::Object(output)));
        emit.code = 0;
        return Ok(emit);
    }

    if let Some(reason) = denial {
        emit.stderr.push_str(&reason);
        emit.code = 2;
        return Ok(emit);
    }
    Ok(emit)
}
