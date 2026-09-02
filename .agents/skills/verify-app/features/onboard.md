# Onboard a project

Onboard is bee's front door: it installs or upgrades the whole bee frame into a
target repository — the AGENTS.md block, the `.bee/` runtime store, the vendored
expertise layer, the hook wiring, and the rendered skills for each runtime. It is
idempotent, and without `--apply` it is a read-only plan that changes nothing.

## Sub-features

- `onboard-plan` reports what would change and mutates nothing.
- `onboard-apply` writes the frame and reports what it wrote.
- `onboard-idempotent` a second run over an unchanged repo needs no changes.
- `onboard-target` `--repo-root` installs into a repo other than the cwd.

## How to get to it (user POV)

- Run `bee onboard --repo-root <path> --json` from a bee source checkout to see
  the plan.
- Run `bee onboard --repo-root <path> --apply --json` to install.
- Run `bee onboard --json` with no `--repo-root` to target the current directory.

## Driving it with control-bee

Preconditions:

- The binary is built (`control-bee build`) and a run is live (`control-bee launch`).
- A **virgin** target repo: `control-bee newrepo target`. `launch`'s sandbox is
  already onboarded and cannot show the plan-to-apply transition.
- Onboard is driven with `control-bee host`, not `control-bee cli`. `cli` sets cwd
  to the sandbox, where onboard finds no engine to vendor and returns
  `blocked_no_engine`. `host` runs with cwd set to the real checkout and refuses
  any command whose `--repo-root` is not under `$VERIFY_HOME/run`.
- Read the binary path once: `control-bee bin`.

- **Plan is read-only.** Ask what would change without changing it. Run
  `VERIFY_CWD=target control-bee sh -- git status --porcelain` and keep the
  output, then
  `control-bee host -- <binary> onboard --repo-root <target> --json`, then repeat
  the `git status` call. The payload carries `"status": "changes_needed"`, a
  non-empty `plan[]`, `repo_root`, `source`, `bee_version`, `skills` and
  `notices`. The two `git status` outputs are identical.
- **The verification notice says which of five states the repo is in.** Read the
  `notices` array, not just its presence. `stale_advisor_notices` picks ONE
  verification line in strict priority order
  (`onboard/notices.rs:196-233`, constants at `onboard/templates.rs:286-328`):
  1. a legacy `commands.verify` key wins outright — the retirement warning, in
     its with-test or no-test form;
  2. else the skill file `.bee/verify/verify-app/SKILL.md` EXISTS — the upkeep
     pointer, whether or not a test command is declared;
  3. else a test command is declared — the tested-repo offer, which opens by
     saying the tests check the code but nothing drives the product;
  4. else — the no-test offer, which opens "This project has no command that
     proves it works".
  A top-level `advisor` key adds its own stale-key warning independently.
  Drive the states by creating or removing that one skill file and by editing
  `commands.test`; the branch reads the SOURCE path only, never a rendered
  skill home.
- **Apply installs the frame.** Run
  `control-bee host -- <binary> onboard --repo-root <target> --apply --json`. The
  payload carries `"status": "applied"`, `"bee_version"` equal to the version in
  `.claude-plugin/plugin.json`, a non-empty `applied[]`, `"recheck": "up_to_date"`
  and `"recheck_plan": []`. There is **no** `plan` key on an apply payload.
- **Confirm what landed.** Run `VERIFY_CWD=target control-bee sh -- ls -a .` and
  `VERIFY_CWD=target control-bee sh -- ls .bee`. The tree gains `AGENTS.md`,
  `CLAUDE.md`, `.gitignore`, `.bee/`, `.claude/`, `.agents/`, `.opencode/`,
  `.pi/` and `docs/`. `.bee/` holds `onboarding.json`, `config.json`,
  `state.json`, `reservations.json`, `decisions.jsonl`, `backlog.jsonl`,
  `config-sample.json`, `cells/`, `logs/`, `expertise/` and `bin/`.
- **Confirm the store agrees.** Run
  `VERIFY_CWD=target control-bee cli -- status --json`. It reports
  `onboarding.installed: true`, `onboarding.bee_version` equal to the manifest
  version, and `onboarding.drift: false`.
- **Re-running changes nothing.** Run
  `control-bee host -- <binary> onboard --repo-root <target> --json` again. The
  payload reports `"status": "up_to_date"` with `"plan": []`.
- **Aiming onboard at the real checkout is refused by the harness.** Run
  `control-bee host -- <binary> onboard --repo-root . --json`. `control-bee`
  refuses before running anything, because `--repo-root` is not under
  `$VERIFY_HOME/run`.
- **Proof.** Run `VERIFY_CWD=target control-bee snapshot onboard`. The snapshot's
  `onboarding.json` names the bee version, and `git-status.txt` shows the tree
  bee produced.

## Gotchas

- **Engine location is not what it looks like.** The marker onboard hunts for is
  `packages/bee/AGENTS.block.md` (`onboard/source.rs:128`), not a Cargo manifest.
  It tries candidates in order: `BEE_JS_ENTRY`, then the `--repo-root` path
  itself, then a walk up from cwd that **stops at the first `.git` it crosses**.
  A sandbox is a git repo, so a cwd inside one ends the walk immediately and
  onboard returns `status: "blocked_no_engine"`. That is why `control-bee host`
  exists.
- A `blocked_*` status means zero mutations happened. The set is
  `blocked_no_engine`, `blocked_no_source`, `blocked_downgrade`,
  `blocked_render`, `blocked_hooks_merge`, `blocked_worktree_migration_conflict`,
  `blocked_codex_hook_write` (`onboard/apply.rs:215-225`) and a bare `blocked`. `versions` is present for the version and downgrade
  blocks only, not for migration or hook-merge conflicts. Do not retry with
  `--force-downgrade` unless that is the behavior under test.
- The plan/apply split is the only dry-run here. Prove it by diffing
  `git status --porcelain` before and after, not by trusting the missing
  `--apply`.
- Hook wiring is **not** written by default. It lands only when `--repo-hooks` is
  passed or a previous run already recorded it (resolved at
  `onboard/mod.rs:361-365`, applied at `onboard/plan.rs:908`). When it
  does land, those hooks fire for an agent session opened inside that repo, never
  for this harness, so they cannot be verified from here.
- Installing over an existing store preserves state. A test that expects a
  pristine `state.json` must launch a new sandbox, not re-onboard an old one.
