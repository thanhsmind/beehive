# agyrt-6 — agy runtime docs checked against the shipped code

Checker for agyrt-4 (commit 68760a611). The implementer's notes were not read.
Code paths are relative to `packages/bee-rs/crates/bee/src/` unless they start
with `.agents/` or `docs/`. Verdicts: OK, WRONG, IMPRECISE, EXTERNAL (a fact
about agy itself, not about bee code; the source is `docs/history/agy-runtime/discovery.md`).

## Wrong claims

- WRONG — `docs/config-reference.md:308-326`: "The onboarding template gives `team.agy` the same herding roles as `team.pi`" followed by an example whose slots point at `claude-opus`, `claude-fable` and `agy-flash`. The template gives every `team.agy` slot `{ "kind": "herding", "agent": "agy", "description": … }`, derived from `team.claude`'s role names (`onboard/templates.rs:294-305`, test `onboard/templates.rs:677-709`), and the template's herding roster names `agy`, not `agy-flash` (`onboard/templates.rs:275`). The example also drops the template's `plan`, `design`, `lane-1..3` slots (`onboard/templates.rs:203-216`).
- WRONG — `docs/knowledge/areas/hook-runtime/the-agy-belt.md:64`: "On a crash or unreadable payload, they [SessionStart, PreInvocation, PostToolUse, Stop] print the neutral `{}` answer". An unreadable payload on any event prints `{"decision":"deny","reason":"agy-belt: unreadable hook payload: …"}`, because `handle_payload` parses before it knows the event (`hooks/agy_belt.rs:437-446`). A child hook crash on those events does fail open (`hooks/agy_belt.rs:450-453`, `:368-372`, `:521`). The plan asked for the neutral answer on any non-PreToolUse failure (`docs/history/agy-runtime/plan.md:161`), so either the doc line or the code is wrong; the checker does not pick.
- IMPRECISE — `the-agy-belt.md:42`: "falls back to `.bee/bin/bee` under the git common directory". The command runs `$g/../.bee/bin/bee`, the parent of the git common dir (`.agents/hooks.json:6`, built by `onboard/agy_belt.rs:17-22` via `hooks_wiring::main_checkout_arm`).

## the-agy-belt.md

- OK — :22 fifth runtime, no `Runtime::Agy`, no generated projection: `devtools/hook_manifests.rs:54-65`; label in `onboard/templates.rs:453`.
- EXTERNAL — :28-30 agy reads `AGENTS.md`/`GEMINI.md`, `.agents/skills`, `.agents/hooks.json` merge order: agy behaviour, discovery.md.
- OK — :34-40 `"bee"` key, five events, PreToolUse/PostToolUse grouped with `matcher: ""`, others flat: `.agents/hooks.json:2-47`, `onboard/agy_belt.rs:32-46`.
- EXTERNAL — :42 `sh -c` with cwd = `.agents/`: agy behaviour; the command's `../.bee/bin/bee` and `git -C ..` assume it (`.agents/hooks.json:6`).
- OK — :42 neutral fallback `{"decision":"ask"}` on PreToolUse, `{}` elsewhere: `onboard/agy_belt.rs:34-44`, `.agents/hooks.json:6,13,23,35,44`.
- OK — :46 camelCase payload, translated fields `hook_event_name`, `session_id`=`conversationId`, `cwd`, `transcript_path`, `bee_runtime: "agy"`: `hooks/agy_belt.rs:36-49`, `:160-170`; exit 0 and one JSON object: `hooks/agy_belt.rs:534-538`.
- OK — :50 SessionStart → catalog SessionStart rows (session-init), `injectSteps` systemMessage or `{}`: `hooks/agy_belt.rs:448-455`, `:251-284`, `devtools/hook_manifests.rs:130`.
- OK — :51 PreInvocation n=0 → UserPromptSubmit, `prompt`=`lastUserInput`: `hooks/agy_belt.rs:456-466`, `:172-174`, `devtools/hook_manifests.rs:136`.
- OK — :52 PreInvocation n>0 → `{}`, no check: `hooks/agy_belt.rs:457-459`.
- OK — :53 PreToolUse answers deny/ask+overwrite/ask: `hooks/agy_belt.rs:302-366`.
- OK — :54 PostToolUse runs checks, prints `{}`: `hooks/agy_belt.rs:504-524`.
- OK — :55 Stop, `stop_hook_active`=`continuationActive`, block → continue: `hooks/agy_belt.rs:181-183`, `:368-390`, `:525-529`; session-close on Stop: `devtools/hook_manifests.rs:225-232`.
- EXTERNAL — :58 `{}` on PreToolUse denies every tool on agy 1.5.17: discovery.md; code never prints `{}` on PreToolUse: `hooks/agy_belt.rs:467-503`.
- OK — :61 repair key map command→CommandLine, file_path→Path, content→Content, old_string→Target, new_string→Replacement: `hooks/agy_belt.rs:286-300`, `:353-362`.
- OK — :62 first denial wins; spawn error / unreadable payload / non-zero hook exit deny on PreToolUse: `hooks/agy_belt.rs:304-329`, `:438-446`, `:468-479`.
- OK — :63 Stop continue on block, else `{}`: `hooks/agy_belt.rs:368-390`.
- WRONG — :64 see above.
- OK — :72-75 run_command/write_file/edit_file/read_file maps: `hooks/agy_belt.rs:88-122`.
- EXTERNAL — :76 `invoke_subagent` args `Prompt`, `Role`, `TypeName`, `Model`: code ignores the args (`hooks/agy_belt.rs:123`); names come from discovery.md. Denial: `hooks/agy_belt.rs:493-496`.
- OK — :77 no-write list incl. `list_directory`, `browser_*`, `capture_browser*`, `read_browser*`; "plan tool" is `update_steps`: `hooks/agy_belt.rs:11-22`, `:124-128`.
- OK — :78 other tool with `Path`/`AbsolutePath`/`TargetFile`/`File` → Write `{file_path, content: ""}`: `hooks/agy_belt.rs:24`, `:129-140`.
- OK — :79 unknown → ask + coverage-gap row: `hooks/agy_belt.rs:141-143`, `:426-435`, `:498-501`.
- OK — :83 herding-only, `team.agy`, deny reason names `bee dispatch prepare --runtime agy`: `verbs/drivers/prepare.rs:150`, `:203-210`, `:2798`; `hooks/agy_belt.rs:26`.
- OK — :87 `ANTIGRAVITY_CONVERSATION_ID` checked before Pi, Codex, Claude: `session_identity.rs:103`, `:112`, `:121`, `:143`.
- OK — :93-94 fresh write / replace only `"bee"`: `onboard/agy_belt.rs:48-63`, test `:106-`.
- OK — :95 backup `.agents/hooks.json.bee-backup` before first rewrite: `onboard/apply.rs:835-845`.
- OK — :96 non-object / unparseable refused, nothing written: `onboard/agy_belt.rs:80-84`, `onboard/apply.rs:483-492` (`blocked_hooks_merge`).
- OK — :97 second run no change: `onboard/agy_belt.rs:85-86`, `:65-73`.
- OK — :98 `team.agy` herding roles like `team.pi`; doctor agy row names `bee onboard --apply`: `onboard/templates.rs:282-305`, `doctor.rs:506-530`.
- EXTERNAL — :99 workspace trust is the owner's step: CONTEXT D7.

## catalog-projections-and-activation.md

- OK — :22-31 five runtimes, three hand-authored belts, agy needs a translator: `devtools/hook_manifests.rs:54-65`, `hooks/agy_belt.rs:160-187`.
- OK — :67-72 no `Runtime::Agy`, belt = `.agents/hooks.json` + `bee hook agy-belt`: `devtools/hook_manifests.rs:58-65`, `hooks/mod.rs:81`, `:260`.
- OK — :80 OpenCode, Pi and agy carry their own belt: `devtools/hook_manifests.rs:54-65`.

## repo-local-guardrails.md

- OK — :146-154 merge under `"bee"`, backup, refusal, idempotent, `team.agy`, doctor row, trust: same anchors as the-agy-belt.md:93-99.

## config-reference.md

- OK — :172-174 five `team` runtime keys: `onboard/templates.rs:453`, `:674`.
- OK — :306 legal at `dispatch prepare` and `dispatch wave`: `verbs/drivers/prepare.rs:143-150`, runtime enum `["codex","claude","pi","agy"]` twice in `generated/registry_payload.json`.
- OK — :306 belt under `"bee"`, runs `bee hook agy-belt`, denies `invoke_subagent`: `.agents/hooks.json:2-6`, `hooks/agy_belt.rs:26`.
- OK — :308 non-herding slot refuses (`agy_requires_herding`): `verbs/drivers/prepare.rs:203-210`, `:266`, `:2798`.
- WRONG — :308-326 template example, see above.

## README.md

- OK — :7 Claude Code, Codex, OpenCode, Pi and agy: `onboard/templates.rs:453`.
- OK — :710 hook automation on all five runtimes: `.agents/hooks.json`, `hooks/agy_belt.rs:534-538`.

## Bundle checks

- `.bee/bin/bee knowledge check`: exit 0, "331 concept(s) in 352 file(s), 0 OKF error(s), 0 profile error(s), 610 profile warning(s) — OK".
- `.bee/bin/bee knowledge index --check`: exit 0, "20 expected index file(s), 0 stale — OK".
