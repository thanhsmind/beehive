# agyrt-6 — agy runtime docs checked against the shipped code (recheck after agyrt-7)

Checker for agyrt-4 (commit 68760a611) and its fix agyrt-7 (commit b830a9b7d).
The implementer's notes were not read. Code paths are relative to
`packages/bee-rs/crates/bee/src/` unless they start with `.agents/` or `docs/`.
Verdicts: OK, WRONG, EXTERNAL (a fact about agy itself, not about bee code; the
source is `docs/history/agy-runtime/discovery.md`).

## Wrong claims

- WRONG — `docs/knowledge/areas/hook-runtime/the-agy-belt.md:94`: "onboarding replaces only the `"bee"` key and keeps every other key's value byte-equal". `merge_into` parses the file and writes the whole object again through `jsjson::stringify_pretty` (`onboard/agy_belt.rs:53-62`, `jsjson.rs:125-130`), so an owner value is kept equal as JSON and in key order (`serde_json` `preserve_order`, `packages/bee-rs/Cargo.toml:17`), not byte for byte: the code's own test input `"extra": [1, 2, 3]` comes back as a multi-line array, and the test asserts value equality only (`onboard/agy_belt.rs:106-123`). A fix reads "keeps every other key's value equal as JSON (the file is re-printed, so formatting may change)". `repo-local-guardrails.md:148-149` says "value unchanged", which holds as JSON equality.

## Rechecked agyrt-7 corrections

- OK — `docs/config-reference.md:308-326`: `team.agy` derived from `team.claude` role for role, every slot `{ "kind": "herding", "agent": "agy", "description": … }`, `herding.agents` `"agy": ["agy"]`: `onboard/templates.rs:294-305`, `:275`, test `:677-678`, `:707-710`. The example descriptions for `code`, `read`, `plan`, `design`, `lane-1` match `onboard/templates.rs:203`, `:204`, `:208`, `:207`, `:214`; roles `plan`, `design`, `lane-1..3`, `hat-*` exist at `onboard/templates.rs:207-221`.
- OK — `the-agy-belt.md:64`: child-hook crash or spawn error fails open on SessionStart, PreInvocation, PostToolUse, Stop (`hooks/agy_belt.rs:450-453`, `:461-464`, `:521`, `:370-373`); an unreadable payload denies on every event because `handle_payload` parses first (`hooks/agy_belt.rs:437-446`).
- OK — `the-agy-belt.md:42`: fallback `$g/../.bee/bin/bee`, `$g` the absolute git common dir: `.agents/hooks.json:6`, built at `onboard/agy_belt.rs:17-22`.

## the-agy-belt.md

- OK — :22 fifth runtime, no catalog `Runtime::Agy`, no generated projection: `devtools/hook_manifests.rs:54-65` (the catalog enum is `Claude`, `Codex`). `doctor.rs:63`, `:71`, `:506` has its own `Runtime::Agy`; the sentence is scoped to the hook catalog, so it holds.
- EXTERNAL — :28-30 agy reads `AGENTS.md`/`GEMINI.md`, `.agents/skills`, merges `.agents/hooks.json` names in order: discovery.md.
- OK — :34-40 `"bee"` key, five events, PreToolUse/PostToolUse grouped with `matcher: ""`, the rest flat: `onboard/agy_belt.rs:32-46`, `.agents/hooks.json:2-47`.
- EXTERNAL — :42 `sh -c`, cwd = `.agents/`: agy behaviour; the command's `../.bee/bin/bee` and `git -C ..` assume it (`.agents/hooks.json:6`).
- OK — :42 missing-binary neutral answer `{"decision":"ask"}` on PreToolUse, `{}` elsewhere: `onboard/agy_belt.rs:34-44`, `.agents/hooks.json:6`, `:13`, `:23`, `:35`, `:44`.
- OK — :46 camelCase payload; translated `hook_event_name`, `session_id`=`conversationId`, `cwd`, `transcript_path`, `bee_runtime: "agy"`: `hooks/agy_belt.rs:36-49`, `:165-170`; one JSON object, exit 0: `hooks/agy_belt.rs:534-538`.
- OK — :50 SessionStart → catalog SessionStart (`bee-session-init`), `injectSteps` `{"systemMessage":{"systemMessage":…}}` or `{}`: `hooks/agy_belt.rs:448-455`, `:251-284`, `devtools/hook_manifests.rs:125-131`.
- OK — :51 PreInvocation n=0 → UserPromptSubmit, `prompt`=`lastUserInput`, prompt-context and activity: `hooks/agy_belt.rs:456-466`, `:172-174`, `devtools/hook_manifests.rs:133-139`.
- OK — :52 PreInvocation n>0 → `{}`, no check: `hooks/agy_belt.rs:457-459`.
- OK — :53 PreToolUse → deny / ask+overwrite / ask: `hooks/agy_belt.rs:302-366`, `:480-492`.
- OK — :54 PostToolUse runs the checks, prints `{}`: `hooks/agy_belt.rs:504-524`.
- OK — :55 Stop, `stop_hook_active`=`continuationActive`, session-close and state-sync, block → continue: `hooks/agy_belt.rs:181-183`, `:525-529`, `:368-390`, `devtools/hook_manifests.rs:224-235`.
- EXTERNAL — :59 `{}` on PreToolUse denies every tool on agy 1.5.17: discovery.md; the code never prints `{}` on PreToolUse: `hooks/agy_belt.rs:467-503`.
- OK — :61 repair map command→CommandLine, file_path→Path, content→Content, old_string→Target, new_string→Replacement, first repair kept: `hooks/agy_belt.rs:286-300`, `:353-362`.
- OK — :62 first denial wins; spawn error, non-zero exit, unreadable payload deny on PreToolUse: `hooks/agy_belt.rs:304-329`, `:437-446`.
- OK — :63 Stop block → `{"decision":"continue","reason":…}`, else `{}`: `hooks/agy_belt.rs:378-389`.
- OK — :72-75 run_command, write_file, edit_file, read_file maps: `hooks/agy_belt.rs:88-122`.
- EXTERNAL — :76 `invoke_subagent` args `Prompt`, `Role`, `TypeName`, `Model`: discovery.md; the code ignores the args (`hooks/agy_belt.rs:123`) and denies (`:493-496`).
- OK — :77 no-write list with `list_directory`, plan tool = `update_steps`, prefixes `browser_`, `capture_browser`, `read_browser`: `hooks/agy_belt.rs:11-22`, `:124-128`, `:497`.
- OK — :78 other tool with `Path`/`AbsolutePath`/`TargetFile`/`File` → Write `{file_path, content: ""}`: `hooks/agy_belt.rs:24`, `:129-140`.
- OK — :79 unknown → ask plus coverage-gap row `unknown-agy-tool`: `hooks/agy_belt.rs:141-143`, `:426-435`, `:498-501`.
- OK — :83 herding-only, `team.agy`, native/model/cli/budget refuse: `verbs/drivers/prepare.rs:150`, `:203-210`, `:224-235`, `:2798-2799`; deny reason names `bee dispatch prepare --runtime agy`: `hooks/agy_belt.rs:26`.
- OK — :87 `ANTIGRAVITY_CONVERSATION_ID` checked before Pi, Codex, Claude: `session_identity.rs:103`, `:112`, `:121`, `:143`.
- OK — :93 fresh file gets `{"bee": …}` only: `onboard/agy_belt.rs:51`, `:77`, test `:98-104`.
- WRONG — :94 see above.
- OK — :95 backup `.agents/hooks.json.bee-backup` before the first rewrite: `onboard/apply.rs:835-840`.
- OK — :96 non-object or unparseable refused with a typed error, nothing written: `onboard/agy_belt.rs:80-84`, `onboard/apply.rs:483-495` (`blocked_hooks_merge`).
- OK — :97 second run no change: `onboard/agy_belt.rs:85-86`, test `:144-149`.
- OK — :98 `team.agy` herding roles like `team.pi`; doctor agy row names `bee onboard --apply`: `onboard/templates.rs:282-305`, `doctor.rs:506-530`.
- EXTERNAL — :99 workspace trust is the owner's step: CONTEXT D7.

## catalog-projections-and-activation.md

- OK — :67-72 fifth runtime, catalog exclusion, belt = `.agents/hooks.json` + `bee hook agy-belt`: `devtools/hook_manifests.rs:54-65`, `hooks/mod.rs:81`, `:260`.
- OK — :80 OpenCode, Pi and agy carry their own belt: `devtools/hook_manifests.rs:45-58`.

## repo-local-guardrails.md

- OK — :146-154 merge under `"bee"`, owner values unchanged as JSON, backup, refusal, idempotent, `team.agy`, doctor row, trust: same anchors as the-agy-belt.md:93-99.

## config-reference.md

- OK — :172-174 five `team` runtime keys `claude`, `codex`, `opencode`, `pi`, `agy`: `verbs/drivers/models.rs:75`.
- OK — :306 legal at `dispatch prepare` and `dispatch wave`: `verbs/drivers/prepare.rs:143-150`, runtime enum `["codex","claude","pi","agy"]` twice in `generated/registry_payload.json`.
- OK — :306 belt under `"bee"`, runs `bee hook agy-belt`, denies `invoke_subagent`: `.agents/hooks.json:2-6`, `hooks/agy_belt.rs:26`, `:493-496`.
- OK — :308 non-herding slot refuses (`agy_requires_herding`): `verbs/drivers/prepare.rs:203-210`, `:2798-2799`.
- OK — :308-326 template example: see the rechecked corrections above.

## README.md

- OK — :7 Claude Code, Codex, OpenCode, Pi and agy: `onboard/templates.rs:453`.
- OK — :710 hook automation on all five runtimes: `.agents/hooks.json`, `hooks/mod.rs:260`.

## Bundle checks

- `.bee/bin/bee knowledge check`: exit 0, "331 concept(s) in 352 file(s), 0 OKF error(s), 0 profile error(s), 610 profile warning(s) — OK".
- `.bee/bin/bee knowledge index --check`: exit 0, "20 expected index file(s), 0 stale — OK".
