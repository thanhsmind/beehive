---
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: Config role-key guard

## Summary

The model guard refuses an agent that hand-picks a model in a dispatch call.
An agent that instead rewrites the role-to-model table in `.bee/config.json`,
and then dispatches through the door, passes every check — the same decision,
one layer up, with no door on it.

That is not a theory. In the `memorypad` repo a Pi session hit two reviewer
failures and edited `team.pi.review.agent` from `pi-agy-flash-3.7` to
`pi-opencode-free` on its own, then ran `bee dispatch prepare` against the
table it had just rewritten.

This plan closes that layer. The write guard grows one arm over the two config
files. It refuses a write that would do either of two things: change **the
table the dispatcher actually resolves against**, or **switch off the guard
that enforces it**. Every other key — `uat_stop`, `commands.*`, `worktree_*`,
`gate_bypass`, and a `description`-only edit inside the team table itself —
stays as agent-writable as it is today, because the agent is told to write
those.

Mode: `standard` — 2 risk flags: `public-contracts` (the guard's refusal
surface is consumed by the Claude, Codex and Pi belts), `covered-contract-change`
(an existing test asserts `.bee/config.json` stays hand-writable and must stay
green).

Why not `high-risk`: the `audit-security` hard-gate flag was considered and not
counted. This arm governs agent self-governance — which model a role runs — not
auth, credentials, or user data, and a wrong arm fails LOUD (a visible refusal
the user can clear by editing the file outside the agent), never silent.

## Requirements

- D1 (`521b2e62`): an edit that changes the role-to-agent/model mapping keys in
  `.bee/config.json` is refused and names the user as its owner.
- D2 (`521b2e62`): every other config key stays freely agent-writable.

## Load-bearing claims

<!-- bee:not-a-deferral: the rows below are evidence about what the code does today — a claims table, not a promise of future work -->

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Neither config file is in the CLI-owned deny list, so any agent edit passes today | read | `hooks/write_guard/guards.rs:79-86` | `".bee/state.json" => Some(…)`, `".bee/backlog.jsonl" => …`, `".bee/runtime/cross-worktree-holds.json" => …`, `".bee/runtime/worktree-grants.json" => …`, `".bee/companion-session.json" => …`, `".bee/onboarding.json" => …`, `_ => None,` — and two prefix arms above for `.bee/cells/` and `.bee/lanes/` |
| 2 | A test asserts a `Write` of `{}` to `.bee/config.json` exits 0, deliberately | read | `hooks/write_guard/tests.rs:264-275` | `fn config_json_and_decisions_jsonl_stay_hand_writable() {` … `assert_eq!(e.code, 0, "{}", e.stderr);` |
| 3 | `check_write` takes a path and no content, so the decision cannot live inside it | read | `hooks/write_guard/checks.rs:379-388` | `pub(crate) fn check_write(` / `    root: &str,` / `    state: &Map<String, Value>,` / `    rel_path: &str,` |
| 4 | `tool_input` — holding `content`, `old_string`, `new_string`, `edits` — is parsed before any check runs | read | `hooks/write_guard/main.rs:92` | `let tool_input: Map<String, Value> = match payload.get("tool_input") {` |
| 5 | **`hooks.write-guard: false` in config switches this whole guard off, arm included** | read | `hooks/write_guard/main.rs:88`; `state.rs:194-200` | `if !hook_enabled(&store_root_pb, HOOK_NAME) {` / `        return Ok(emit);` |
| 6 | **`.bee/config.local.json` overlays `.bee/config.json`, so guarding one file alone is a one-line bypass** | read | `state.rs:176-184` | `let tracked = read_obj(root.join(".bee").join("config.json")).unwrap_or_default();` / `let overlay = read_obj(root.join(".bee").join("config.local.json"));` |
| 7 | The legacy `models` key folds into `team` on every config read | read | `state.rs:187`; `verbs/drivers/models.rs:293-295` | `crate::verbs::drivers::fold_team_key(&mut merged);` ; `} else if let Some(models_val) = map.remove("models") {` / `        map.insert("team".to_string(), models_val);` |
| 8 | **`normalize_models` IS the view the dispatcher resolves against, and it strips `description` by design** | read | `verbs/drivers/models.rs:258`, `:312` | `pub(crate) fn normalize_models(raw: Option<&Value>) -> Map<String, Value> {` ; `    Ok(normalize_models(config.get("team")))` |
| 9 | A normalized slot keeps `kind`/`command`, so a `kind: cli` slot's model — which rides inside `command` — is inside the compared value | read | `verbs/drivers/models.rs:140-147` | `if matches!(obj.get("kind"), Some(Value::String(k)) if k == "cli") {` … `out.insert("command".into(), Value::String(js_trim(cmd).to_string()));` |
| 10 | The write surface is `Edit`/`Write`/`MultiEdit`, plus `apply_patch` and Bash — an arm wired to two of them leaves three open | read | `hooks/write_guard/main.rs:64-68` | `let is_write_tool = matches!(tool_name.as_str(), "Edit" \| "Write" \| "MultiEdit");` |
| 11 | **Pi's own edit tool arrives as `MultiEdit` with an `edits[]` array — so a Write/Edit-only arm never fires on the runtime that had the incident** | read | `hooks/write_guard/tests.rs:1518-1523` | `"tool_name": "MultiEdit",` / `"tool_input": {` / `"file_path": "src/app.js",` / `"edits": [{"old_string": "a", "new_string": "b"}]` |
| 12 | Bash reaches the guard as a command string with no content, so a `sed -i` on config cannot be key-scoped | read | `hooks/write_guard/main.rs:184` | `let command = match first_truthy(&tool_input, &["command", "cmd"]) {` |
| 13 | There is no `bee team set` verb, so the refusal's FIX names the human, not a command | ran | `.bee/bin/bee team --help` | `bee team show` — `Read-only: the whole team role table` |
| 14 | **The model also lives one layer up, in `herding.agents.<name>` argv — guarding `team` alone leaves that door open** | ran | `.bee/config.json` `herding.agents` | `"pi-opencode-free": ["pi","-a","--model","opencode/x-preview-f-free:high"]` ; `"pi-gpt-5.6-luna": ["pi","-a","--model","openai-codex/gpt-5.6-luna:high"]` |
| 15 | A `null` role slot normalizes to `Some(Null)`, not to absent — a value-neutral re-serialize that drops null keys would trip a naive compare | read | `verbs/drivers/models.rs:134` | `        Value::Null => return Some(Value::Null),` |
| 16 | **bee's own refusal texts instruct the agent to ADD a team slot, so an arm that refuses adds contradicts bee** | read | `verbs/drivers/prepare.rs:212`, `:1976` | `set team.pi.{slot} in .bee/config.json to {"kind":"herding","agent":…}` ; `add "{declared}": "<model>" to team.{runtime} in .bee/config.json` |
| 17 | `bee onboard --apply` writes `config.json` only when it is absent, so it never collides with this arm | read | `onboard/apply.rs:369-373` | `create_runtime_file` — creation only, never a rewrite of an existing table |

<!-- /bee:not-a-deferral -->

All paths above are under `packages/bee-rs/crates/bee/src/`.

## Discovery

The plan-step hat wave ran three seats. Five findings changed the shape; the
first is the one that would have made the whole feature theatre.

**Claim 5 — the fatal one.** `hooks.write-guard: false` is an ordinary config
key. An agent that wants a different model does not need to beat the arm; it
switches the guard off first, then writes whatever it likes. An arm that lives
inside an agent-disableable hook, and leaves the disable key writable, guards
nothing against the exact adversary it was built for. The arm now covers the
`hooks` subtree.

**Claim 2** is the Chesterton's fence. The first draft made `.bee/config.json`
a whole CLI-owned store. That test preserves config as hand-writable ON PURPOSE.
A whole-file deny breaks a flow the agent is told to use.

**Claim 6** killed the second draft. It guarded `.bee/config.json` only, and
`.bee/config.local.json` overlays it on every read — the refused agent writes
the same table one file over and wins.

**Claims 8 and 9** replaced a hand-rolled projection with the real one. The
draft was going to walk `team.<runtime>.<role>` and read `agent`/`model` out of
each slot. That misses a `kind: cli` slot, whose model rides inside `command`.
`normalize_models` is what the dispatcher resolves against, it keeps
`kind`/`command`, and it strips `description`. Comparing normalized views
answers the real question — *would this write change which model a role runs?*
— instead of guessing which keys matter. One fact, one home.

**Claims 11 and 14** are the `hat-user-impact` seat's two. Pi's edit tool
arrives as `MultiEdit`, so the draft's Write/Edit-only call site would not have
fired on the runtime that had the incident — a guard that misses the reported
case while the user believes it holds is worse than no guard. And `team` names
an agent while `herding.agents.<name>` holds that agent's argv, where `--model`
actually is. The projection had to grow to the table the dispatcher resolves
against end to end, not just its first hop.

**Claim 16** set the add/change boundary, below.

## Shape

One projection, one arm, one call site, tests.

**The projection.** `config_governed_view(text) -> Option<Value>` in the write
guard. Parse the text as a JSON object; return `None` if it does not parse.
Otherwise build a view from three parts:

1. **The role table.** `fold_team_key`, then `team` through `normalize_models`,
   then drop slots whose normalized value is `Null` so a null slot and an absent
   slot project alike (claim 15). When the object carries neither `team` nor
   `models`, project it the way the dispatcher does in that shape rather than as
   an empty table.
2. **The agent argv.** `herding.agents` and `herding.agent_command`, verbatim
   (claim 14).
3. **The guard's own switch.** The `hooks` subtree, verbatim (claim 5).

**The arm.** `config_governed_change_deny(rel, old_text, new_text)` fires only
when `rel` is `.bee/config.json` or `.bee/config.local.json`, and refuses when
the two views differ — with one carve-out:

- **An ADD is allowed; a CHANGE and a REMOVAL are refused.** A role slot absent
  before and present after passes, because bee's own refusal texts instruct that
  exact write (claim 16). A slot that existed and now holds a different value,
  or is now gone, refuses. The same rule applies inside `herding.agents`.
- The `hooks` part takes no carve-out. Any change to it refuses, in either
  direction — a guard's own off-switch is not an ordinary key.

Old text absent (a first write, no file on disk) allows: there is no table to
change. Old text unparseable and new text parseable allows, so a corrupt config
can still be repaired.

**The call site**, in `main.rs`'s `write_capable` branch, once per resolved
config path:

| Surface | New content | Behavior |
|---|---|---|
| `Write` | `content` | project and compare |
| `Edit` | on-disk file with `old_string` → `new_string`, honoring `replace_all` | project and compare |
| `MultiEdit` | on-disk file with each `edits[]` pair applied in order | project and compare |
| `apply_patch` | patch text, not a reconstructable file | **refuse, naming Edit/Write as the way in** |
| Bash | command string only (claim 12) | **refuse, naming Edit/Write as the way in** |

The last two rows close the surface without a hole. An in-place shell rewrite or
a patch envelope aimed at a config file is refused with a message pointing at
`Edit`/`Write` — the surfaces where the check can actually run. That costs the
agent nothing (it has both tools) and does not weaken D2, because the redirected
surface honors D2 in full.

**Fail closed.** A config write the arm cannot reconstruct — an `Edit` carrying
no `old_string`, unreadable on-disk content — refuses. This guard's stated
posture is already fail-closed on a detection error; a config write that cannot
be checked is that same case.

**The refusal names the change.** The arm reports the FIRST differing address
and both values — `team.pi.review.agent: "pi-gpt-5.6-luna" → "pi-opencode-free"`
— never a bare "config changed". A refusal the agent can turn into one line the
user acts on is protection; a wall is friction. There is no `bee team set`
(claim 13), so the FIX names the human as the owner and the file as the place.

**Smaller path check.** A path-only deny is cheaper and fails D2 (claim 2). A
doctrine-only AGENTS.md line is cheaper and fails D1 — the Pi incident is the
evidence that unenforced doctrine does not hold. Guarding at dispatch time
instead fails D1's letter: the edit lands and only a later dispatch complains.
Whole-subtree equality is one comparison cheaper and fails D2 on the
`description` and `null` cases. Comparing normalized views is the cheapest shape
that honors both. PASS.

## Cells

One cell. The `hat-alternatives` seat was right that a three-cell split was one
cell pretending to be three: the red-first test cannot go green until the wiring
exists, and a red proof refuses a cap. Red-first runs INSIDE the cell.

- `crkg-1` — the projection, the arm, the call site across all five surfaces,
  and both test sets.
  - Refusals: a `team.pi.review.agent` change via `Write`, `Edit` and
    `MultiEdit`; the same change routed through `.bee/config.local.json`; the
    legacy `models` spelling; a `herding.agents.<name>` argv rewrite; a
    `hooks.write-guard: false` flip; an `apply_patch` and a `sed -i` at a config
    path; a slot removal.
  - Preserved: `uat_stop`, `commands.test`, `gate_bypass`, a re-indent, a
    `description`-only edit, a dropped `null` slot, a NEW role slot (claim 16),
    a first write with no file on disk, a repair of a corrupt config — and
    `config_json_and_decisions_jsonl_stay_hand_writable` untouched and green.
  - Files: `hooks/write_guard/guards.rs`, `hooks/write_guard/main.rs`,
    `hooks/write_guard/tests.rs`.

## Verify

`cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml`
— the repo's declared `commands.test`, matching what CI runs
(`.github/workflows/ci.yml:102`). Scope reason: the guard's own suite, the hook
contract suites and the three runtime belt contract suites all live in that one
manifest.

## Recorded gaps

Named, not smuggled:

- **Adds are allowed** (claim 16) because bee's own refusal texts instruct the
  agent to make them. An agent can still pick the model for a role that had no
  slot. Closing this needs those FIX strings rewritten to address the user
  first; filed as its own backlog item, not folded in here.
- **The arm compares this checkout's config.** In a linked worktree
  `.bee/config.json` is a per-worktree copy while `dispatch prepare` resolves
  through `resolve_store_root`. Whether a worktree-copy edit even reaches
  dispatch is a separate question, filed with the item above.
