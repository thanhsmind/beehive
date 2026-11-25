# agy-runtime — CONTEXT

Owner ask (2026-11-25, Vietnamese): "Làm beehive hỗ trợ agy giống claudecode
hay pi". agy (Google Antigravity CLI) already runs as a herding worker. This
feature makes it a full bee runtime: a leader session in agy gets the bee
rules, skills, guard belt, session identity and dispatch door, the way Claude
Code, Codex, OpenCode and Pi do.

Evidence: `docs/history/agy-runtime/discovery.md` (live probe of agy 1.5.17).

## Locked decisions

- D1. `agy` is the fifth runtime label, floor Antigravity CLI 1.5.17. It
  joins every runtime list: skill-marker labels (`<!-- bee:only agy -->`),
  the dispatch runtimes, the status runtimes and the `team.<runtime>`
  config key. agy reads `AGENTS.md` natively and reads skills from the shared
  `.agents/skills/` root. The codex render owns that root and keeps `agy`
  blocks, as it keeps `pi` blocks. No new skill tree.
- D2. The agy belt is the workspace file `.agents/hooks.json`. bee owns one
  top-level hook name, `"bee"`, with five events: `SessionStart`,
  `PreInvocation`, `PreToolUse` (matcher `""`), `PostToolUse` (matcher
  `""`) and `Stop`. Every handler runs `bee hook agy-belt`, resolving the
  project's `.bee/bin/bee` from the `hooks.json` folder first, then from the
  git common dir (the Claude wiring's fallback). agy takes the named
  exclusion from the hook catalog that OpenCode and Pi take: the belt is its
  projection, and a coverage test keeps it honest.
- D3. `bee hook agy-belt` translates. It reads agy's camelCase payload,
  builds the payload the existing bee hooks read (`hook_event_name`,
  `session_id` = `conversationId`, `cwd`, `tool_name`, `tool_input`,
  `prompt`, `stop_hook_active`, `bee_runtime: "agy"`), runs the same checks
  Claude Code runs for that event, and writes agy's answer:
  - SessionStart → session init; its text → `injectSteps` systemMessage.
  - PreInvocation with `invocationNum` 0 → prompt context, as
    UserPromptSubmit with `prompt` = `lastUserInput`; its text →
    `injectSteps`. A later invocation prints `{}` and runs no check.
  - PreToolUse → the D4 map, then the PreToolUse checks. A deny →
    `{"decision":"deny","reason":…}`. A repaired input →
    `{"decision":"ask","overwrite":{…}}` in agy argument keys. Otherwise
    `{"decision":"ask"}`, which keeps agy's own approval path. A crash or an
    unreadable payload on PreToolUse denies (fail closed).
  - PostToolUse → the PostToolUse checks; prints `{}`.
  - Stop → session close with `stop_hook_active` = `continuationActive`. A
    block → `{"decision":"continue","reason":…}`; otherwise `{}`.
  - Advisory events fail open: a crash prints the neutral answer.
- D4. Tool map (agy 1.5.17): `run_command` {CommandLine} → Bash {command};
  `write_file` {Path, Content} → Write {file_path, content}; `edit_file`
  {Path, Target, Replacement} → Edit {file_path, old_string, new_string};
  `read_file` {Path} → Read {file_path}; `invoke_subagent` → D5. A named
  list of tools that write nothing (`search_web`, `send_message`,
  `call_mcp_tool`, `read_url_content`, `list_dir`, `view_file`, the plan
  tool, browser tools) answers ask without a check. Any other tool that
  carries a path argument (`Path`, `AbsolutePath`, `TargetFile`, `File`) is
  checked as a Write on that path. Any other tool answers ask and logs a
  coverage-gap row.
- D5. Dispatch on agy is herding-only, the Pi rule. `bee dispatch prepare
  --runtime agy` reads `team.agy`; a native slot refuses with the herding
  remedy. The belt denies `invoke_subagent` with a reason that names
  `bee dispatch prepare --runtime agy`, so no worker starts outside the
  one door.
- D6. Session identity: `ANTIGRAVITY_CONVERSATION_ID` names an `agy`
  session. It is checked before the Codex and Claude variables, because an
  agy started from a Claude session inherits `CLAUDE_CODE_SESSION_ID`.
- D7. Onboarding: `bee onboard --apply` writes or merges `.agents/hooks.json`.
  It replaces only the `"bee"` key, keeps every other key byte-equal in
  value, backs up a pre-existing file before the first rewrite, refuses a
  file that is not a JSON object, and changes nothing on a second run. The
  onboarding template gives `team.agy` the same herding roles as `team.pi`.
  `bee doctor` gets an agy wiring row that names `bee onboard --apply` as the
  fix. Trusting the workspace in agy stays the owner's step.
- D8. Docs: a hook-runtime concept for the agy belt, the onboarding
  multi-runtime concept, the README runtime list and `team.agy` in the
  config reference, citing agy-runtime D1-D8.

## Out of scope

- agy plugin packaging (`plugin.json`, marketplace).
- Custom agent files in `.agents/agents/` and native `invoke_subagent`
  dispatch.
- `agy acp` and the Remote Control view.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md | suspect | named exclusion of hand-authored belts from the catalog |
| docs/knowledge/areas/onboarding/repo-local-guardrails.md | suspect | how onboarding vendors each runtime's belt |

### Code

| Path | What it decides |
|------|-----------------|
| `packages/bee-rs/crates/bee/src/devtools/hook_manifests.rs` | the catalog and the named exclusions |
| `packages/bee-rs/crates/bee/src/hooks/adapter.rs` | hook payload read and output encoding |
| `packages/bee-rs/crates/bee/src/hooks/mod.rs` | `bee hook <name>` dispatch |
| `packages/bee-rs/crates/bee/src/session_identity.rs` | runtime from session env |
| `packages/bee-rs/crates/bee/src/onboard/templates.rs` | runtime lists and `team.pi` template |
| `packages/bee-rs/crates/bee/src/onboard/apply.rs` | belt vendoring steps |
