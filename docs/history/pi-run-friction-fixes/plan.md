---
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Pi run friction fixes

## Summary

A real Pi run on `anphabe-goglbe` (2026-09-17: one leader, six herding
reviewers, one fix worker) wasted calls and wrote wrong records. An
advisor checked each cause against bee 2.41.0. This plan fixes the ones that
are still present: four guard/session bugs and the herding brief problems.

Mode: `standard` — 1 risk flag: `public-contracts` (the herding brief text and
the write-guard refusal surface are read by every runtime belt). The brief
change also touches a covered contract: a test pins the result-form block
byte for byte, and that test is updated on purpose.

Decision: `91450355-8261-4e25-b6cd-a2f9660d45a3`.

## Load-bearing claims

<!-- bee:not-a-deferral: the rows below are evidence about what the code does today -->

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The git guard tokenizes the raw command, heredoc bodies included | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:714` | `let deep = tokenize_deep(command);` |
| 2 | A heredoc fence already exists but only target extraction calls it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:459` | `pub(crate) fn fence_heredocs(command: &str) -> String {` |
| 3 | An explicit session id wins with no record check | read | `packages/bee-rs/crates/bee/src/verbs/state_group/store.rs:388-395` | `if let Some(f) = flag {` / `return Ok(Some(js_trim(f).to_string()));` |
| 4 | A session record is over when its status is dead or closed | read | `packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs:135` | `if matches!(record.get("status"), Some(Value::String(s)) if s == "dead" \|\| s == "closed") {` |
| 5 | The Pi belt routes an unknown tool with no command to Write on its first path field, and `url` is not a path field | read | `.pi/extensions/bee-guard.ts:345-355` | `const PATH_FIELDS = [` … `"outputPath",` |
| 6 | A dispatch with no feature falls back to state.json's feature for the ORIGINAL REQUEST | read | `packages/bee-rs/crates/bee/src/verbs/intent_group.rs:280-284` | `} else if let Ok(Some(active)) = active_feature(root) {` |
| 7 | The brief says "files listed under the Expertise section" even when no section renders | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:331` | `files listed under the Expertise section are yours to read.` |
| 8 | The result schema lists options, leaning and dissent inline with required keys | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:372-374` | `\"options\": [\"<one self-contained sentence per way forward>\", \"...\"],` |
| 9 | A test pins the result-form block byte for byte | read | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:1142` | `fn render_brief_keeps_the_result_form_block_byte_identical_apart_from_the_new_ack_block() {` |
| 10 | Herded activity takes its feature from the lane or state.json, never from the job | read | `packages/bee-rs/crates/bee/src/hooks/activity.rs:838-851` | `let ReadJson::Parsed(Value::Object(state)) = read_json(&ctrl.join(".bee").join("state.json"))` |
| 11 | In the run, all six reviewer jobs recorded `expertise: []` and feature `zoom-report-in-campaign-detail` while reviewing `fb-lead-ads-auto-sync` | ran | `packages/bee-rs/crates/bee/src/herding/mailbox.rs:280-283` | `"expertise": []` / `"feature": "zoom-report-in-campaign-detail"` (host `.bee/mailbox/job-*/job.json`, `activity.json`) |

<!-- /bee:not-a-deferral -->

## Cells (current slice)

```json
[
  {
    "id": "prf-1",
    "feature": "pi-run-friction-fixes",
    "title": "Fence heredoc bodies before the git guard scans for git verbs",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs", "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs", "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Red first: add a test in write_guard/tests.rs where the default record is idle and a Bash command pipes a heredoc whose body is prose containing the words 'git range' into another command (shape: `{ cat <<'PROMPT'\\nreview the git range af..d5\\nPROMPT\\n} | echo hi`). Today the intake gate refuses with 'running `git range` is blocked'; watch it fail for that reason. Fix: in check_git_bash_command (checks.rs:714) tokenize `fence_heredocs(command)` instead of the raw command. Also add a test that a REAL git verb outside a heredoc (e.g. `git push` after a heredoc) is still judged as before.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee write_guard",
    "must_haves": {"truths": ["prose inside a heredoc body never reaches the git verb scan", "a git verb outside a heredoc is still judged"], "artifacts": [], "key_links": [], "prohibitions": ["Do not weaken the depth-bound fail-open branch"]},
    "behavior_change": true
  },
  {
    "id": "prf-2",
    "feature": "pi-run-friction-fixes",
    "title": "Refuse a borrowed closed session id",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/state_group/store.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/state_group/store.rs", "packages/bee-rs/crates/bee/src/verbs/state_group/sessions.rs", "packages/bee-rs/crates/bee/src/session_identity.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "A Pi leader set BEE_SESSION_ID / --session-id to a CLOSED Claude session from another day and bee accepted it for claims and waiting-on marks. In resolve_session_id (store.rs:388) refuse an id from the explicit flag or from BEE_SESSION_ID when its session record exists with status closed or dead AND the id is not the caller's own harness identity (crate::session_identity locate_caller_from / CLAUDE_CODE_SESSION_ID / PI_SESSION_ID / Codex id). Keep: no record at all still resolves (new ids are legal); the caller's own released session still resolves (release then re-engage is a normal flow). The refusal text names the session, says it is closed and belongs to another session, and gives a FIX: use your own session id (name the env var) or start/bind your own session. Check the Ex error type used by the callers so the refusal surfaces as a typed error, not a panic. Red first: tests in store.rs's test module (add `#[cfg(test)] mod` if none) for closed-foreign refused, closed-own allowed, missing-record allowed.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee state_group",
    "must_haves": {"truths": ["a closed session id that is not the caller's own is refused with a FIX line", "an id with no record still resolves", "the caller's own closed session still resolves"], "artifacts": [], "key_links": [], "prohibitions": ["Do not add a new CLI flag"]},
    "behavior_change": true
  },
  {
    "id": "prf-3",
    "regen_obligation_ack": "wave-barrier",
    "feature": "pi-run-friction-fixes",
    "title": "Route a url-only Pi tool as a non-write",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": [".pi/extensions/bee-guard.ts"],
    "read_first": [".pi/extensions/bee-guard.ts", "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs", "packages/bee-rs/crates/bee/src/doctor.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "The Pi tool fetch_content({url, mode}) was denied as a write with the containment message, because the default arm routes an unknown tool with no command to Write with file_path \"\". Keep the fail-safe (never a silent allow for a write-capable shape). Change only: when an unknown tool has no command and no PATH_FIELDS value, but has a string `url` (or `urls`) field, route it as a read-only web fetch shape bee already lets through (check main.rs: which tool names are not write tools — e.g. tool_name \"WebFetch\" with tool_input {url}) instead of Write with an empty path. An unknown tool with neither command, path nor url keeps today's behavior. Find how the belt is tested (rg for a bee-guard test harness, e.g. a node/bun test or a Rust test that runs the extension; doctor.rs include_str keeps the source canonical). If a harness exists add a case; if none exists, prove with a short node/bun script run that imports or evaluates the routing function and state that proof in the cap.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor",
    "must_haves": {"truths": ["a tool call with only url/mode args is not routed to Write with an empty path"], "artifacts": [], "key_links": [], "prohibitions": ["Do not allow an unknown tool that carries a path field or a command without a bee verdict"]},
    "behavior_change": true
  },
  {
    "id": "prf-4",
    "feature": "pi-run-friction-fixes",
    "title": "Stop rendering another feature's request into an unbound dispatch",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/intent_group.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/intent_group.rs", "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "dispatch_original_request (intent_group.rs:269) falls back to state.json's active feature when no feature was resolved. In the Pi run the default record still named a stale feature while the real work lived in lane records, so six reviewer prompts carried the wrong ORIGINAL REQUEST. Change the fallback: use active_feature(root) only when no live lane record exists (a live lane = a `.bee/lanes/*.json` or workflow record whose phase is not idle/compounding-complete and status not closed; reuse an existing lane-listing helper, rg for one in verbs/state_group). With live lanes and no resolved feature, return None (no ORIGINAL REQUEST block) — silence beats a wrong anchor. Single-session repos with no lanes keep today's behavior. Red first: a test with a stale state.json feature anchor plus one live lane and feature None → None; and no lanes → the state.json anchor as before.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee intent",
    "must_haves": {"truths": ["with live lanes and no resolved feature, no ORIGINAL REQUEST is rendered", "with no lanes the state.json fallback is unchanged"], "artifacts": [], "key_links": [], "prohibitions": ["Do not read .bee/intent/default.json"]},
    "behavior_change": true
  },
  {
    "id": "prf-5",
    "feature": "pi-run-friction-fixes",
    "title": "Make the herding brief say only what applies",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/herding/mailbox.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/herding/mailbox.rs", "packages/bee-rs/crates/bee/src/herding/run.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Three brief fixes in render_brief (mailbox.rs:257), each red first in the mailbox test module. (1) The clause 'and files listed under the Expertise section are yours to read' (line 331) renders only when spec.expertise is non-empty. (2) The result schema block: keep status, summary, files_changed, proof, report_path as the required object; move options, leaning and dissent into a separate block headed so a worker sees they are OPTIONAL — options+leaning only when status is blocked with a choice, dissent only when disagreeing — and say to omit the keys entirely (not empty strings or empty arrays) otherwise. Keep the parser unchanged, but make an empty-string dissent claim parse as no dissent if it does not already (parse_dissent). (3) When the job's worktree root .bee/config.json (read via the existing config reader, e.g. crate::state config helper) has a non-empty commands.test, render a '# Proof command' section with that exact command, before '# Files you may touch'; absent → no section. Update render_brief_keeps_the_result_form_block_byte_identical_apart_from_the_new_ack_block and the zero-expertise byte-identical test deliberately, with a one-line comment naming this cell. If BriefSpec needs a new field for the proof command, add it and fill it in run.rs where BriefSpec is built — run.rs is read-only for this cell unless that one construction site needs the field; if so, touch only that site.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding",
    "must_haves": {"truths": ["no Expertise clause without an Expertise section", "optional result keys are presented apart from required ones", "commands.test reaches the brief when set"], "artifacts": [], "key_links": [], "prohibitions": ["Do not change the ack, report or result file protocol or paths"]},
    "behavior_change": true
  },
  {
    "id": "prf-6",
    "regen_obligation_ack": "wave-barrier",
    "feature": "pi-run-friction-fixes",
    "title": "Give reviewer dispatches the review method",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "packages/bee/prompts/reviewer.md"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs", "packages/bee/prompts/reviewer.md", "skills/bee-reviewing/references/reviewing-reference.md"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Six reviewer jobs ran with expertise [] and no lens card, so each invented its own method. In prepare.rs, for kind reviewer with no --expertise and no --brief-file: when <root>/.bee/expertise/review.md exists, default the expertise to one entry `.bee/expertise/review.md :: finding quality and severity calibration :: Where to look` (use parse_expertise's line format so every transport renders it the same). An explicit --expertise wins unchanged. In packages/bee/prompts/reviewer.md add a short lens line: the dispatcher names one review lens (Purpose) in the purpose/prompt; review through that lens only, and follow review.md for what a finding is and how to set severity. Red first: a prepare test that a reviewer dispatch in a repo with .bee/expertise/review.md carries that expertise entry, and one without the file carries none. If the prompts are rendered/vendored by regen (check .bee/bin/prompts), note it in the cap; the orchestrator runs regen.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee prepare",
    "must_haves": {"truths": ["a reviewer dispatch carries .bee/expertise/review.md when it exists and no expertise was passed"], "artifacts": [], "key_links": [], "prohibitions": ["Do not add a new CLI flag", "Do not change cell, gather or advisor kinds"]},
    "behavior_change": true
  },
  {
    "id": "prf-7",
    "feature": "pi-run-friction-fixes",
    "title": "Attribute herded activity to the job's feature",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/hooks/activity.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/hooks/activity.rs", "packages/bee-rs/crates/bee/src/herding/mailbox.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Herded reviewer activity recorded feature zoom-report-in-campaign-detail (state.json) while the job reviewed another feature. In activity.rs, when the session is herded (the `herded` job id is known), resolve the feature from the job first: read the job's job.json in the mailbox (job cwd); if that cwd is a feature worktree, take its feature (the granted worktree record in .bee/runtime/worktree-grants.json, or the `<repo>--wt--<slug>` directory name as fallback). Only then fall back to resolve_feature's lane/state.json order. Also: when a herded session's current round result file (result-<round>.json) already exists at write time, set work status to done rather than leaving it open (read the existing work-status field names in this file). Red first with the existing activity test helpers.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee activity",
    "must_haves": {"truths": ["a herded job running in a feature worktree records that feature", "a non-herded session's feature resolution is unchanged"], "artifacts": [], "key_links": [], "prohibitions": ["Do not change the activity state machine transitions"]},
    "behavior_change": true
  },
  {
    "id": "prf-8",
    "feature": "pi-run-friction-fixes",
    "title": "Parse a bare --no-mistakes on cells finish as a boolean",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["91450355-8261-4e25-b6cd-a2f9660d45a3"],
    "files": ["packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs"],
    "read_first": ["packages/bee-rs/crates/bee/src/verbs/reservations/flags.rs", "packages/bee-rs/crates/bee/src/verbs/cells/util.rs"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Found while capping this wave: the cells finish verb with a bare no-mistakes flag is refused with 'unsupported argument shape' because no-mistakes is not in FLAG_ALONE_BOOLEANS, so the bare flag never parses as Present. Red first: a parse_flags test that `--no-mistakes` at the end and before another flag parses as FlagV::Present; then add \"no-mistakes\" to FLAG_ALONE_BOOLEANS.",
    "verify": "PATH=\"$HOME/.cargo/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee reservations::flags",
    "must_haves": {"truths": ["a bare --no-mistakes parses as a boolean"], "artifacts": [], "key_links": [], "prohibitions": []},
    "behavior_change": true
  }
]
```
