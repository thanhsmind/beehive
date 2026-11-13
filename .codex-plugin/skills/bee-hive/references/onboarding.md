# Onboarding Reference

Load ONLY when onboarding is actually in question: `.bee/onboarding.json` is
missing or stale, the user asks to install or upgrade bee, or an `onboard`
run returned a `blocked_*` status. A session that is already `up_to_date`
never needs this file — `SKILL.md`'s Onboarding section carries the three
steps a running session executes.

## Onboarding Protocol

This file is the full status contract behind the three steps in `SKILL.md`'s
Onboarding section (the router keeps the steps, this file keeps the detail). Run the bee
binary from the bee source root (the checkout or installed plugin package) — `onboard` is a
subcommand of the same binary every other verb uses:

```bash
.bee/bin/bee onboard --repo-root <repo-root> --json
```

Then inspect the result:

- `status: "up_to_date"` → continue.
- `status: "changes_needed"` → summarize the plan to the user, ask for approval, and only then re-run with `--apply`. Never apply silently. Never replace an existing compact prompt or AGENTS.md content outside the BEE markers without explicit consent. Every `--apply` also syncs the bee skill set into the host repo's two managed roots (`<repo>/.claude/skills/bee-*` for Claude Code, `<repo>/.agents/skills/bee-*` for Codex) in the same run — one command keeps vendored helpers and installed skills at the same version. The trees are committed to the host repo, never gitignored. `--global-skills` additionally syncs the legacy global `~/.claude/skills/bee-*` root; without the flag the global root is never read, written, or deleted. `--skillstation` also publishes the `bee-deploying` skill into the user's skillstation hub through the `skillstation` CLI (commit, then activate); without the CLI the onboard still succeeds and reports one skipped entry with target `skillstation`. The payload's `skills.targets` carries one entry per target root: `{kind: "repo-claude" | "repo-agents" | "global", target_root, mode, blocked, versions, items}`. When the repo being onboarded contains the running script's own skill tree (bee's own repo), the per-project targets sync through the ordinary skill-sync path (mode `sync`/`fresh`/`noop`) like every other managed target; only the exact source-equals-target root is a `noop`. Each managed root is rendered per runtime (Claude vs Codex) and stamped with a render provenance marker, so a rendered projection is never accepted back as an onboarding source. Global sync there is unchanged.
- `status: "blocked_downgrade"` → the source tree is older than the repo's vendored helpers or a target's installed skills (or a version could not be read — reported as `unknown`, refused the same way). The three-version preflight runs per target; ANY blocked target blocks the whole run (blocked-first), zero mutations happen anywhere, and the top-level `reason`/`versions` surface the blocked target(s). Surface the reported `versions` to the user; only pass `--force-downgrade` on explicit user instruction, and only when every blocked target resolved all three versions numeric — an `unknown` version is never forceable.
- `status: "blocked_no_source"` → no authoritative skill source resolved for this run (identity check failed, or source/target/repo roots overlap). Fail-closed, zero mutations, never forceable with `--force-downgrade` — surface it to the user and resolve the source location before retrying. `versions` is still reported on every blocked return (identity/overlap included), with `unknown` for each of the three (resolution was never attempted) — never `null`.
- **Forced-apply transparency:** whenever a blocked result is forceable, both the plain `--json` dry-run and a refused `--apply` (no `--force-downgrade` yet) carry every target's computed `items` inside `skills.targets` — the full per-target list of `sync_skill`/`remove_skill`/`blocked_*` items a `--force-downgrade` would apply. Show this list to the user BEFORE they authorize the force — it is exactly which skills get overwritten or DELETED, per target; a forced apply then executes precisely that reviewed set.
- Every skill-stage item (`sync_skill`, `remove_skill`, `blocked_symlink`, `blocked_alias`) carries `target` (the target kind above) and `scope: "installed" | "source"`: `installed` means `path` is relative to that target's `target_root`, `source` means `path` is relative to the running script's own skill tree. Legacy plan items (AGENTS.md, `.bee/` runtime files, vendored helpers, etc.) carry no `scope` or `target` at all — they are always repo-relative. Never resolve a skill-stage `path` against `repo_root`.
- A `blocked_symlink` item inside `plan` means one skill directory is a symlink and was skipped (not synced, not deleted) — surface it to the user; it does not block the rest of the apply.
- **Recheck honesty:** after `--apply`, the response's `recheck` field applies blocked-first precedence aggregated across ALL targets — if the skill-sync stage is still blocked post-apply on ANY target (e.g. a residual per-skill symlink/alias block left one skill's version marker un-synced after a forced downgrade), `recheck` reports that blocked status and can never read `"up_to_date"`, even when the rest of the plan is empty. `recheck_skills` carries `{blocked, reason, versions, targets}` whenever this fires.
- `--repo-hooks` only when the user asks for repo-local hook wiring.
- `--claude-md` only when plugin hooks are unavailable and the user wants the CLAUDE.md `@AGENTS.md` import fallback.

If onboarding is not complete, do not continue into the rest of the bee workflow.

### Greenfield init lane

When the onboarding result carries the init-lane notice (first onboard, no detectable build), offer it before any feature work: the first planning slice is **one init cell** whose `must_haves` are exactly the initialization checklist — setup succeeds from scratch, one passing test exists, standard commands recorded in `.bee/config.json`, clean first commit. The user may decline; a declined offer is recorded as a deferred idea, never silently dropped.

## Workspace protocol

A host project may keep one root file, `WORKSPACE_PROTOCOL.md`. It names the
human decision boundaries, the merge owner and the deploy owner. The binary
writes the facts it can read. The agent adds what only reading can find. Read
the tree. Do not ask the owner to describe the project.

**When.** Run this procedure after every `bee onboard` whose result has
`workspace_protocol.action_needed: true`. That means the file is missing,
invalid, unreadable, or still bee's unedited render. Run it also when the owner
asks. `bee onboard` never writes the file.

**Steps.**

1. Run `.bee/bin/bee protocol init --json`. When a file exists, run
   `.bee/bin/bee protocol init --print` instead. It writes nothing. Compare the
   render with the file.
2. Read every path in `review`. Those are the project's own rule files:
   `LEADER.md`, `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `docs/DEPLOY.md`
   and the `.github/workflows/*` files that exist.
3. Edit `WORKSPACE_PROTOCOL.md`. Add the project's own human decision
   boundaries, protected areas and anti-patterns. Set `PROJECT_CRITICALITY` to
   `low`, `medium` or `high`.
4. Cite every line you add in `## Sources`. Name the file it came from.
5. Leave no placeholder. Add no test command. Test commands live in
   `.bee/config.json` `commands.test`.
6. Run `.bee/bin/bee protocol check` until the state is `valid`.
7. Commit the file.
8. Show the owner a table of key, value and source. Take each row from the
   `facts` list and from your own `## Sources` lines. When no source names an
   owner, keep `human` and say so in the table.
9. Run `.bee/bin/bee onboard --repo-root . --json` again. Confirm that
   `workspace_protocol.action_needed` is false.

**Where to look.**

- `PROJECT_CRITICALITY`: `high` for real users, money, personal data,
  production deploys or a public package that others install. `medium` for an
  internal service or tool with a team. `low` for a personal or throwaway repo.
- `MERGE_OWNER`: `.bee/config.json` `uat_stop`, branch protection hints in
  `.github/`, and `CONTRIBUTING.md`.
- `DEPLOY_OWNER`: `docs/DEPLOY.md`, the release scripts, the deploy and publish
  workflows in `.github/workflows`, the `Dockerfile` and the package publish
  config.
- `HUMAN_DECISION_BOUNDARIES`: things that cannot be undone or that reach an
  audience. These are deploy, release, tag, push, publish, database
  migrations, data deletion, auth and permission changes, billing, secrets,
  public API or schema changes and major dependency upgrades. Add the rules in
  the review files that say ask, never or must.
- `PROTECTED_AREAS`: generated files (`DO NOT EDIT` headers and codegen
  outputs), lockfiles, vendored trees, applied migrations, CI and release
  config, secret-shaped files, and paths the rule files forbid.
- `ANTI_PATTERNS`: the never and do-not lines of the rule files, and past
  mistakes in `docs/knowledge` or in recorded decisions.

**What good looks like.**

- Each item is one concrete action or path that an agent can recognize. Write
  "a database migration that drops a column", not "risky changes".
- A protected area names the path and the only allowed way to change it.
- Every line you add has a source in `## Sources`.
- Four to ten boundaries is normal. Fewer means the tree was not read.
- Do not copy bee's own rules that the base set already carries.
- Never invent an owner. When no source names one, keep `human`.

**Self-check before commit.**

- Every key has a source.
- No item is generic.
- No test command is in the file.
- `bee protocol check` is `valid`.
- `bee onboard` reports `action_needed` false.

The repo-root `WORKSPACE_PROTOCOL.md` of the bee repo is the reference result.
