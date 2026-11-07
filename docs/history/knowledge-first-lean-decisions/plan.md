# knowledge-first-lean-decisions — Plan

**Lane:** standard · **Class:** feature · **Date:** 2026-11-07

## Problem

Two owner-approved fixes, in this order (owner request 2026-11-07).

1. Leaders read code before they read knowledge. In 30 days `bee knowledge
   search` ran 9 times and `bee knowledge context` 3 times, against 697 doc
   reads and about 3976 reads of `packages/` code in 14 days of transcripts.
   One leader spent about 15 tool calls in `dispatch.jsonl` and `prepare.rs`
   and gave a wrong answer; `bee knowledge search` had the answer in 31 ms
   (mailbox reflect, fix-at check, run 70bdca3d). The rule exists only as one
   prose line in `packages/bee/LEADER.md` § Start a session.
2. The decision store fills with bookkeeping. Of 552 decisions in 30 days,
   about 20% are gate auto-approval audit lines and about 22% are
   close/merge/release/scribing/promote/handoff bookkeeping. Writes outnumber
   reads about 13 to 1 (`decisions log` 516, `decisions search` 39).

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The knowledge-before-code rule is one prose line, with no check | read | packages/bee/LEADER.md:109-110 | `When the feature has knowledge under docs/knowledge/, read its context before planning or executing.` |
| 2 | The write-guard already sees Read, Glob, Grep and Bash on PreToolUse | read | packages/bee/hooks/claude-hooks.json:46 | `"matcher": "Edit|Write|MultiEdit|Bash|Read|Glob|Grep|AskUserQuestion|apply_patch|exec"` |
| 3 | The write-guard emits a non-blocking advisory with additionalContext and no permissionDecision | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:917-934 | `{"hookSpecificOutput":{"hookEventName":"PreToolUse","additionalContext":...},"systemMessage":...}` |
| 4 | A herded worker never reaches the full write-guard path | read | packages/bee-rs/crates/bee/src/hooks/mod.rs:225-236 | `write-guard` goes to `write_guard::run_herded` under `BEE_HERDING_WORKER` |
| 5 | Code never writes a decision for a gate auto-approval; the agent does, because prose tells it to | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs:165 | `it records the recommended choice, logs a one-line audit decision, and continues` |
| 6 | The gate stamp already records actor, reason and bypass level | read | packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:1053-1067 | `gates.<name> = {actor, at, reason, bypass_level}` |
| 7 | The close doors read deferral decisions by tag and feature token | read | packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:668-691 | `filter_decision_events` with tag `judge-deferral` / `uat-deferral` |
| 8 | Other deferral readers outside close | read | packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs:471-477 | tag `leader-check-deferral`; also `cells/dissent.rs:595-601` (`dissent-deferral`) and `state_group/set_gate.rs:171,202` (`capture-deferral`) |
| 9 | Bookkeeping share of the store | ran | .bee/decisions.jsonl | 538 decide rows since 2026-10-08: 73 bypass-audit, 49 release, 24 merge, 18 close, 7 promote, 7 handoff, 6 scribing, 7 deferral by keyword |

## Decisions

- D1 — The knowledge-first rule has one home: a rule block
  `agents-knowledge-before-code` in `packages/bee/LEADER.md` § Start a
  session. The hook message names the rule id and the exact command; it never
  restates the rule.
- D2 — The check is a nudge, never a block: a PreToolUse advisory, at most
  once per turn, leader sessions only.
- D3 — Gate auto-approval audit has one home: the gate stamp plus one row that
  `bee gate --actor auto` itself appends to the audit log. The agent logs no
  decision for it.
- D4 — Bookkeeping goes to a new tracked audit log `.bee/audit.jsonl`
  (`merge=union`), written by a new verb `bee audit log`. No existing log fits:
  `.bee/logs/` is git-ignored, so its rows would never reach main through a
  worktree merge, and the close doors run on main.
- D5 — No history rewrite. Old bookkeeping rows stay in `decisions.jsonl`.
  The doors read the audit log AND the legacy decision rows. `decisions
  active` / `search`, status and the preamble hide rows tagged with an audit
  kind at read time unless the caller asks for that tag.
- D6 — `bee decisions log` refuses a `--tags` value that is an audit kind
  (`gate-bypass` and the ten `*-deferral` tags) and names `bee audit log` as
  the remedy. Words like `release` or `merge` stay legal decision tags: many
  real rules carry them. For close/merge/release/scribing/promote/handoff
  bookkeeping with no tag signal, the doctrine names the audit log as the
  home (named limit: prose, not a check).

## Contract — cell kbc-1 (Fix 1, knowledge before code)

- Rule block `<!-- rule: agents-knowledge-before-code -->` in
  `packages/bee/LEADER.md` § Start a session replaces the one prose line, and
  the duplicate `## Start a session` heading goes. `.bee/LEADER.md` is
  refreshed the way the repo renders it. The rule index in
  `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`
  (§ AGENTS.md rule homes) lists the new id.
- Hook (write-guard, full path only): a code read is Read/Glob/Grep whose
  target path, or a Bash command (`rg`, `grep`, `cat`, `sed`, `head`, `tail`,
  `fd`, `find`, `less`, `bat`) whose path argument, is under `packages/` or
  `src/` or is a source file (by extension: rs, mjs, js, ts, tsx, py, go, sh,
  java, kt, swift, rb, c, h, cpp), and is not under `docs/`, `.bee/`,
  `skills/` or a `*.md` file.
- A Bash command that runs `bee knowledge search` or `bee knowledge context`
  marks the session's current turn as searched.
- On a code read in a turn with no search and no nudge yet, the hook emits the
  advisory (the main.rs:917-934 shape) naming the rule id and the exact
  command `.bee/bin/bee knowledge search --text "<what you need>"`, and marks
  the turn as nudged. The read is never denied.
- A new turn starts at UserPromptSubmit: the prompt-context hook resets the
  session's turn state. No state yet counts as a fresh turn.
- Silent for workers: `BEE_HERDING_WORKER` set, or a hook payload that carries
  a non-empty `agent_id` or `agent_type` (a native subagent).
- State lives under `.bee/cache/`, keyed by session id.
- Behavior tests written first and seen red: nudge on the first code read
  with no search; silent after a search in the same turn; silent for a worker
  (env marker and `agent_id` payload); once per turn (second read silent, a
  new UserPromptSubmit re-arms it); a docs read is silent.
- Docs: the advisory joins
  `docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md` as a new
  behavior, citing this plan's D1-D2, no raw decision ids.

## Contract — cell kbc-2 (Fix 2, the audit log and the decision door)

- New store `.bee/audit.jsonl` at the same root the decision store resolves
  to. Row: `{id, ts, kind, feature, cell?, note, source, bypass_level?}`.
  Closed kind set: `gate-bypass`, `close`, `merge`, `release`, `scribing`,
  `promote`, `handoff`, `judge-deferral`, `uat-deferral`, `capture-deferral`,
  `knowledge-freshness-deferral`, `impact-deferral`, `routing-deferral`,
  `doc-deferral`, `leader-check-deferral`, `dissent-deferral`.
- Verbs: `bee audit log --kind <k> --feature <f> --note "<text>" [--cell <id>]`
  (feature defaults to the bound lane or the workflow feature) and `bee audit
  list [--kind] [--feature] [--json]`. An unknown kind refuses and lists the
  set. A module API the doors can call: `has_entry(root, kind, feature) ->
  bool` and `newest_note(root, kind, feature) -> Option<String>`.
- `bee gate --actor auto` appends one `gate-bypass` row (feature, gate names,
  reason, bypass level) after the stamp write succeeds.
- `bee decisions log` refuses a `--tags` entry in the audit-only set
  (`gate-bypass` plus the ten `*-deferral` kinds) with a typed error that names
  `bee audit log --kind <tag>`.
- `bee decisions active` / `search`, the status recent-decisions list and the
  session preamble recent decisions hide rows tagged with an audit-only kind,
  unless `--tag` names that kind. Rows stay on disk.
- `.bee/audit.jsonl merge=union` in `.gitattributes` and the onboarding
  template; the worktree merge bookkeeping exemption lists it beside
  `decisions.jsonl`.
- Help text in `generated/registry_payload.json` via `bee dev regen`.
- Tests first, seen red: audit log/list round trip and kind refusal; decisions
  log refuses `impact-deferral` and `gate-bypass`; search hides a legacy
  `gate-bypass` row and shows it under `--tag gate-bypass`; `gate --actor auto`
  writes one audit row.

## Contract — cell kbc-3 (Fix 2, move every door reader)

- Every reader of a bookkeeping tag reads the audit log first, then the
  legacy decision rows: close.rs judge, uat, capture, knowledge-freshness,
  impact, routing and doc deferrals; `cells/leader_check.rs`
  (leader-check-deferral); `cells/dissent.rs` (dissent-deferral);
  `state_group/set_gate.rs` capture-deferral doors. `feature-local` stays a
  decision tag (it routes a real decision to its home).
- Every refusal text that tells the agent to log a `*-deferral` decision names
  `bee audit log --kind <k> --feature <f> --note "<why>"` instead.
- Tests first, seen red: for each door, an audit row satisfies it; a legacy
  decision row still satisfies it; neither leaves it refused.

## Contract — cell kbc-4 (Fix 2, doctrine and prose)

- `packages/bee/LEADER.md` § Capture what settles gains one rule line: the
  decision store keeps rules, behavior and trade-offs; gate audit,
  close/merge/release/scribing/promote/handoff bookkeeping and door deferrals
  go to `bee audit log`.
- Remove "logs a one-line audit decision" from the preamble
  (`hooks/session_preamble/render.rs`), the session-close nudge
  (`hooks/session_close/nudges.rs`), `skills/bee-hive/references/go-mode.md`,
  `gates-and-delegation.md`, `routing-and-contracts.md`, `scout-and-ticks.md`;
  replace every `decisions log --tags <x>-deferral` instruction in skills
  (`bee-shaping/references/shaping-reference.md`,
  `bee-capturing/references/promotion.md`,
  `bee-principle-split-never-shrink/SKILL.md` and any other hit) with `bee
  audit log`.
- `docs/knowledge/areas/decision-memory/audit-log.md` holds the audit-log
  split (cite this plan's D3-D6).
- Proof: the declared suite (doctrine parity tests included).

## Cells — current slice (preview)

| id | title | issue |
|---|---|---|
| kbc-1 | Nudge a leader to search knowledge before its first code read in a turn | Fix 1 |
| kbc-2 | Add the audit log, write gate auto-approvals to it, keep audit tags out of decisions | Fix 2 |
| kbc-3 | Read door deferrals from the audit log and the legacy decision rows | Fix 2 |
| kbc-4 | Point doctrine and skills at the audit log for bookkeeping | Fix 2 |

Order: kbc-1, then kbc-2, then kbc-3 and kbc-4. kbc-3 needs the kbc-2 module
API; kbc-4 names the kbc-2 verb. One worktree, so cells run one at a time.

```json
[{"id":"kbc-1","feature":"knowledge-first-lean-decisions","lane":"standard","role":"code","title":"Nudge a leader to search knowledge before its first code read in a turn","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs","packages/bee-rs/crates/bee/src/hooks/write_guard/knowledge_first.rs","packages/bee-rs/crates/bee/src/hooks/write_guard/mod.rs","packages/bee-rs/crates/bee/src/hooks/prompt_context.rs","packages/bee-rs/crates/bee/tests/knowledge_first_nudge.rs","packages/bee/LEADER.md",".bee/LEADER.md","docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md","docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md"],"read_first":["docs/history/knowledge-first-lean-decisions/plan.md","docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md"],"action":"Follow the Contract for kbc-1 in docs/history/knowledge-first-lean-decisions/plan.md. A leader code read (Read/Glob/Grep or a read-shaped Bash command on packages/, src/ or a source file, never docs/, .bee/, skills/ or *.md) in a turn with no bee knowledge search/context yet gets one PreToolUse advisory (the main.rs:917-934 shape, no permissionDecision) naming rule agents-knowledge-before-code and the exact command .bee/bin/bee knowledge search --text \"<what you need>\". Never deny. Once per turn; UserPromptSubmit (prompt-context) resets the session turn state under .bee/cache/. Silent for BEE_HERDING_WORKER and for payloads with agent_id or agent_type. The rule text lives only in packages/bee/LEADER.md as a rule block; refresh .bee/LEADER.md and the rule index. Write the behavior tests first and watch them fail.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml","must_haves":{"truths":["A leader code read with no knowledge search in the turn gets one advisory naming the exact search command","A read after bee knowledge search or context in the same turn is silent","A worker (BEE_HERDING_WORKER or agent_id payload) never gets the nudge","The nudge fires at most once per turn and re-arms at the next UserPromptSubmit","The rule text has one home in packages/bee/LEADER.md and the hook only points to it"]},"affects_skills":[],"affects_specs":["docs/knowledge/areas/hook-runtime/advisories-and-turn-control.md"],"decisions":["8b95790d"],"deps":[]},
{"id":"kbc-2","feature":"knowledge-first-lean-decisions","lane":"standard","role":"code","title":"Add the audit log, write gate auto-approvals to it, keep audit tags out of decisions","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/audit.rs","packages/bee-rs/crates/bee/src/lib.rs","packages/bee-rs/crates/bee/src/verbs/mod.rs","packages/bee-rs/crates/bee/src/verbs/audit.rs","packages/bee-rs/crates/bee/src/verbs/decisions/verbs_read.rs","packages/bee-rs/crates/bee/src/verbs/decisions/read.rs","packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs","packages/bee-rs/crates/bee/src/verbs/status_full/build.rs","packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs","packages/bee-rs/crates/bee/src/verbs/worktree/merge.rs","packages/bee-rs/crates/bee/src/onboard/templates.rs","packages/bee-rs/crates/bee/src/generated/registry_payload.json",".gitattributes","packages/bee-rs/crates/bee/tests/audit_log.rs"],"read_first":["docs/history/knowledge-first-lean-decisions/plan.md"],"action":"Follow the Contract for kbc-2 in docs/history/knowledge-first-lean-decisions/plan.md. Add .bee/audit.jsonl (same root resolution as the decision store, merge=union) with the closed kind set, verbs bee audit log and bee audit list, and the module API has_entry / newest_note for the doors. bee gate --actor auto appends one gate-bypass row after the stamp write. bee decisions log refuses audit-only tags (gate-bypass and the ten *-deferral kinds) naming bee audit log. decisions active/search, status recent decisions and the preamble recent decisions hide audit-only tagged rows unless --tag names that kind; never rewrite decisions.jsonl. List audit.jsonl beside decisions.jsonl in the worktree merge bookkeeping exemption and the gitattributes template. Regenerate help with bee dev regen. Tests first, seen red.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml","must_haves":{"truths":["bee audit log appends a row to .bee/audit.jsonl and bee audit list returns it; an unknown kind refuses","bee gate --actor auto writes one gate-bypass audit row and no decision","bee decisions log refuses gate-bypass and *-deferral tags and names bee audit log","decisions search hides legacy audit-tagged rows unless --tag asks for them, and decisions.jsonl is never rewritten"]},"affects_skills":[],"affects_specs":[],"decisions":["79f208dc"],"deps":["kbc-1"]},
{"id":"kbc-3","feature":"knowledge-first-lean-decisions","lane":"standard","role":"code","title":"Read door deferrals from the audit log and the legacy decision rows","behavior_change":true,"files":["packages/bee-rs/crates/bee/src/verbs/drivers/close.rs","packages/bee-rs/crates/bee/src/verbs/cells/leader_check.rs","packages/bee-rs/crates/bee/src/verbs/cells/dissent.rs","packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs","packages/bee-rs/crates/bee/tests/audit_doors.rs"],"read_first":["docs/history/knowledge-first-lean-decisions/plan.md"],"action":"Follow the Contract for kbc-3 in docs/history/knowledge-first-lean-decisions/plan.md. Every door that reads a *-deferral decision (close.rs judge, uat, capture, knowledge-freshness, impact, routing, doc; leader_check.rs; dissent.rs; set_gate.rs capture-deferral) reads the audit log first through the kbc-2 module API, then the legacy decision rows. feature-local stays a decision tag. Every refusal text that told the agent to log a *-deferral decision names bee audit log --kind <k> --feature <f> --note instead. Tests first, seen red: an audit row satisfies each door, a legacy decision row still does.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml","must_haves":{"truths":["An audit row of each deferral kind satisfies its door","A legacy deferral decision row still satisfies its door","No door refusal text tells the agent to log a deferral decision"]},"affects_skills":[],"affects_specs":[],"decisions":["79f208dc"],"deps":["kbc-2"]},
{"id":"kbc-4","feature":"knowledge-first-lean-decisions","lane":"standard","role":"code","title":"Point doctrine and skills at the audit log for bookkeeping","behavior_change":true,"files":["packages/bee/LEADER.md",".bee/LEADER.md","packages/bee-rs/crates/bee/src/hooks/session_preamble/render.rs","packages/bee-rs/crates/bee/src/hooks/session_close/nudges.rs","skills/bee-hive/references/go-mode.md","skills/bee-hive/references/gates-and-delegation.md","skills/bee-hive/references/routing-and-contracts.md","skills/bee-hive/references/scout-and-ticks.md","skills/bee-shaping/references/shaping-reference.md","skills/bee-capturing/references/promotion.md","skills/bee-principle-split-never-shrink/SKILL.md","docs/knowledge/areas/decision-memory/audit-log.md"],"read_first":["docs/history/knowledge-first-lean-decisions/plan.md"],"action":"Follow the Contract for kbc-4 in docs/history/knowledge-first-lean-decisions/plan.md. One rule line in packages/bee/LEADER.md § Capture what settles: decisions keep rules, behavior and trade-offs; gate audit, close/merge/release/scribing/promote/handoff bookkeeping and door deferrals go to bee audit log. Remove every instruction to log a gate auto-approval audit decision (preamble render.rs, session_close nudges.rs, skills/bee-hive references) since bee gate --actor auto now writes the audit row. Replace every decisions log --tags <x>-deferral instruction in skills with bee audit log --kind <x>-deferral. Add docs/knowledge/areas/decision-memory/audit-log.md citing plan D3-D6, no raw decision ids. Refresh .bee/LEADER.md and any rendered skill copies with bee dev regen.","verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml","must_haves":{"truths":["No doctrine, skill, preamble or nudge text tells the agent to log a gate auto-approval or a deferral as a decision","packages/bee/LEADER.md names bee audit log as the bookkeeping home"]},"affects_skills":["bee-hive","bee-shaping","bee-capturing","bee-principle-split-never-shrink"],"affects_specs":["docs/knowledge/areas/decision-memory/audit-log.md"],"decisions":["79f208dc"],"deps":["kbc-2"]}]
```
