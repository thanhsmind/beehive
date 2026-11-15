# orient-host-playbook — CONTEXT

## Ask

A leader in a real host project ran `bee orient --class bugfix`, saw
`bee-principle-red-before-green` and `bee-principle-laziness-protocol`, yet
opened no playbook file, and its reply named no principle. The owner asked
why, then chose option A: fix the playbook path and make principle naming
checked, not only written.

## Findings that shaped the decisions

- `packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:741` and `:757`
  print `skills/bee-planning/playbooks/<class>...` as fixed text. That tree
  exists only in the bee repo. Onboarding puts skills in `.claude/skills/`,
  `.agents/skills/` or `.opencode/skills/`, or serves them from the
  skillstation hub, so a host leader got a path it could not open.
- The orient text has no `next:` line; the open-the-situation step lives only
  in `.bee/LEADER.md` prose.
- Rule `agents-name-applied-principles` is prose only. A worker cap enforces a
  principles answer; nothing checks a leader's reply. The Stop hook already
  warns on agreement without a `[verified]` label
  (`hooks/session_close/claims.rs`), which is the model for a new warning.

## Locked Decisions

- **D1** (`0d0c78b2`, contract `orient-host-playbook`) `bee orient --class`
  prints each playbook and situation path at the first place it exists, in
  order `skills/`, `.claude/skills/`, `.agents/skills/`, `.opencode/skills/`
  (under `bee-planning/playbooks/`). When none exists it names the
  `bee-planning` skill with the relative playbooks path and says to load it
  with `skill_get`. The text ends with one `next:` line: pick the situation
  whose triggers match, open that file, make the todo list from its numbered
  steps. `--json` carries the same resolved paths.
- **D2** (`dae9d3c7`, contract `principle-naming-stop-check`) On Stop, when the
  last turn ran `bee orient --class` and its output listed principles, the
  hook warns if the final reply names no `bee-principle-` slug and has no
  plain no-principle statement. When that output listed a situation path and
  the turn read no file under a `playbooks/` path, it warns that the playbook
  was never opened. Both are advisory warnings beside the
  unverified-agreement warning, never a block.
- **D3** The bee-hive Communication contract close names, for a multi-step
  routed task, a Principles line: each applied principle slug and the
  decision it changed, or a plain statement that none changed anything.
