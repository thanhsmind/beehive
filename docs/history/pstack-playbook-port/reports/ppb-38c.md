# ppb-38c — Check the situation routing doctrine

Checker for ppb-38 (commits `ecf77d86c`, `8bd04dcdf`). Read-only except this file.

## Verdict

Pass, with three consider-level findings. Every surface ppb-38 changed says the same thing about situations. All eleven new trigger-index skills exist in `skills/`, and `.bee/LEADER.md` matches `packages/bee/LEADER.md` byte for byte. No principle is named in any changed surface (decision a6e3d0bf).

## Surfaces and agreement

| Surface | What it says about situations | Agrees |
|---|---|---|
| `packages/bee/LEADER.md` § Route each task | `bee orient --class <c>` prints situation lines → pick the one whose trigger phrases match → open its file → its numbered steps seed the todo list; `skip:` keeps skipped steps; a class with no situation lines opens its one playbook file; session situations come from `playbooks/session/index.md` and bind no class; bee-figure-it-out when nothing fits or the work is large and cross-cutting | yes, it is the reference reading |
| `packages/bee/LEADER.md` § Trigger index | 11 new lines, each a situation pointing to one skill, with no steps restated | yes |
| `.bee/LEADER.md` | identical to the package copy (`cmp` exit 0) | yes |
| `skills/bee-planning/SKILL.md` | plan.md cites `playbooks/<class>/<situation>.md`, and the class index only for a class with no situations, by name and anchor, never copying steps | yes, same rule as planning-reference |
| `skills/bee-planning/references/planning-reference.md` § Playbooks | one index per class at `playbooks/<class>.md`; situations at `playbooks/<class>/<situation>.md`; the plan cites the situation file, and the index only when the class has no situations | yes |
| `skills/bee-hive/references/routing-and-contracts.md` | the matched situation seeds the todo list: orient prints the lines, the leader opens the file, its steps open the list, `skip:` keeps a skipped step, and the situation file is the one home of the steps | yes, matches LEADER word for word in meaning |
| Rendered trees (`.agents`, `.claude-plugin`, `.claude`, `.codex-plugin`, `.opencode`) | situation lines in the three skill files match `skills/` in all 15 tree/file pairs | yes |

The behavior matches the doctrine: the worktree build (`packages/bee-rs/target/release/bee orient --class research`) prints one `situation: skills/bee-planning/playbooks/research/<name>.md — <triggers>` line for each situation in the class index. That is what LEADER tells the leader to read.

The session list in LEADER (pick up, pause, run until done, orchestrate, babysit, ship) maps to `session-pickup`, `pause-safely`, `autonomous-run`, `orchestrate`, `babysit` and `shipping` in `playbooks/session/index.md`. All six files exist. The index has four more (`worktree-cleanup`, `autopilot-full`, `autopilot-stack`, `opening-a-pr`). LEADER's list is examples, not a closed set.

## Trigger index skills exist

`bee-architect`, `bee-arena`, `bee-interrogate`, `bee-fanout`, `bee-tdd`, `bee-show-me-your-work`, `bee-reflect`, `bee-recall`, `bee-blast-radius`, `bee-comment-review`, `bee-figure-it-out`: each has `skills/<name>/SKILL.md`. The older lines (`bee-unslop`, `bee-herdr`, `bee-grooming`) still resolve too.

## Restatement check (findings)

1. **consider — the todo-seeding mechanics live in two places.** `routing-and-contracts.md` (the paragraph after the lane table) restates LEADER § Route each task: orient lines → open the file → steps seed the list → `skip:` → the file is the one home. The two agree today, but the same fact has two homes. This predates ppb-38: the old playbook wording was duplicated the same way, and ppb-38's action told it to update that paragraph rather than collapse it. Suggested fix: cut the paragraph down to a pointer to LEADER § Route each task.
2. **consider — SKILL.md repeats the citation rule before pointing to its home.** `bee-planning/SKILL.md` gives the full rule (situation file; the index only for a class with none) and then points to planning-reference § Playbooks. It is short and agrees, but the pointer alone would be enough.
3. **consider — one stale phrase in LEADER that ppb-38 did not touch.** The pointer list near the end of `packages/bee/LEADER.md` still says "the class-bound playbook each route class cites". Under the router, a plan cites a situation file. The pointer target (planning-reference "Playbooks") is still correct, so only the wording has drifted.

## Dormant branch

Each of the nine classes has a `playbooks/<class>/` directory with at least one situation. That makes "a class with no situation lines has one playbook file" (LEADER) and "the class index only for a class with no situations" (planning) fallbacks that no current class reaches. They are consistent, just unexercised.

## Found outside scope

`.bee/bin/bee` in this worktree is a symlink to the main checkout's binary. That binary prints only the `playbook:` line, with no `situation:` lines. A leader running the store binary would not see the lines the new doctrine tells them to read until the main checkout's binary is rebuilt from this branch. This is the vendored-binary pattern (`docs/knowledge/patterns/20260821-a-vendored-binary-is-a-second-place-the-feature-must-land.md`).

## Proof

- `cmp packages/bee/LEADER.md .bee/LEADER.md` — exit 0, no output.
- `packages/bee-rs/target/release/bee orient --class bugfix|research` — prints situation lines.
- `test -f skills/<name>/SKILL.md` for all 14 trigger-index skills — all ok.
