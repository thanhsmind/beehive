# harness-audit-hardening — plan

## Where this came from

Three parallel read-only audits of the packaged harness (install inventory,
guidance consistency, hook/settings layer), run 2026-08-07 on the user's
request: "review the packaged bee harness for holes and conflicts, improve."
The user's first ask — per-command help instead of one full dump — turned out
already shipped (`bee <cmd> --help`, group help, `--names`), but the audit
found the per-command path itself is holed: text help prints only `required:`
flags, so `bee dispatch prepare --help` never shows `--claim`, `--purpose`,
`--session-id` even though skills instruct them.

## Findings taken into this feature

| id | Finding | Anchor |
|---|---|---|
| H1 | Plugin-migration cleanup recognizes only pre-cutover `.mjs` hook commands; post-R6 rendered commands never match, so `bee dev plugin-distribution --apply` removes nothing and a migrated host double-fires every hook | `devtools/plugin_distribution.rs:473-485` vs the updated onboarding recognizer `onboard/hooks_wiring.rs:131-141` |
| H2 | A host `.claude/settings.json` that fails to parse is treated as absent and the whole file is rewritten as `{"hooks": …}` — host `permissions`/`env`/`model` lost; same for `.codex/hooks.json`; narrower: non-object `hooks` key or non-array event value silently dropped | `onboard/util.rs:55-61`, `onboard/hooks_wiring.rs:158-183` |
| M3+M5 | Managed gitignore block ignores neither the 9.3MB vendored binary (`.bee/bin/bee`, `.bee/bin/bee.exe`) nor the `.bak` files apply writes; bee's own repo tracks `.claude/settings.json.bak` | `onboard/templates.rs:24-53`, `apply.rs:363-377` |
| A1 | `bee <cmd> --help` text output hides optional flags — only `required:` line rendered; the registry holds full `parameters` | help renderer in router/catalog |
| A2 | `bee hook` bare panics (`range start index 2 out of range`); `bee hook --help` reports unknown hook | `hooks/mod.rs:78`, `:98-101` |
| D | ~25 doc drift/conflict items: INSTALL.md stale counts (15→9 skills, 6→7 hook entries, retired node launcher, "git-ignored" claim), README wrong write-guard allowlist (`plans/` missing, `.spikes/` invented), stale `GATE BYPASS ON`, hook count 6 vs 9, duplicate bypass table, spike-path spellings, AGENTS block pointing Claude readers at a codex-only section, tick-rule substance delegated to AGENTS.md which lacks it, wrong-file cross references in bee-hive references, tier list missing `bee-build`, help-surface spelling divergence, docs/specs vs docs/knowledge | audit 2 findings 1–19, audit 1 §3, audit 3 L4 |

## Deferred to backlog (not this feature)

M1 Windows Codex worktree fallback; M2 doctor codex byte-equality vs merge;
M4 hand-duplicated renderer structure guard; M6 unconditional
`~/.codex/config.toml` statusline write; L1 apply-loop write errors discarded;
L3 `git-common-dir/..` submodule edge; preamble command-surface slimming
(reverses locked V2 decision in `cli-surface-in-context/plan.md` — user call,
not an agent call).

## Shape — 6 cells, one slice

- **hah-1** — H1: teach `recognized_bee_command` the post-R6 command spelling
  (share or mirror the onboarding recognizer), update fixtures that pin the
  old `.mjs` strings, add a fixture with the current rendered command.
- **hah-2** — H2: a parse-failing settings/hooks file refuses the merge with a
  named remedy instead of clobbering; non-object `hooks` / non-array event
  values likewise refuse rather than silently drop; tests for each.
- **hah-3** — M3+M5: add `.bee/bin/bee`, `.bee/bin/bee.exe`, and the three
  `.bak` names to `GITIGNORE_BLOCK_PATTERNS`; bump any pinned pattern count;
  `git rm --cached .claude/settings.json.bak` in this repo.
- **hah-4** — A1: per-command text help renders every declared flag with type,
  `*` marking required (same V2 shape as the preamble), `--json` noted once;
  `required:` line stays for compatibility or folds in — writer's judgment,
  pinned by test.
- **hah-5** — A2: `bee hook` with no argument or `--help` prints the hook
  usage (nine hook names) and exits 0/2 deliberately — no panic; test.
- **hah-6** — D: the doc drift batch across INSTALL.md, README.md,
  `packages/bee/AGENTS.block.md` (+ regen repo AGENTS.md), and
  `skills/bee-hive|bee-swarming` references. Where a pointer names the wrong
  home, fix the pointer to the actual section; where substance is claimed to
  live in AGENTS.md but doesn't, make the reference file the home and point
  AGENTS.md at it (smallest honest fix, no new duplication).

hah-1..5 are code cells (dispatched workers, worktree); hah-6 is docs but
touches the shipped AGENTS block template, so it rides the same worktree.
No ordering constraints; hah-6 independent of all.

## SMALLER PATH check

Cheaper shape would drop hah-4/hah-5 (CLI ergonomics) — but they are the
user's first ask made real; dropping them fails the acceptance line. Doc
fixes could split out — but they ship in the same package and the audit is
fresh now. Kept.

## Verification

`commands.test` (cargo test --release, 1350 green baseline) at every
`bee cells finish`. hah-3 additionally: `git ls-files .claude` no longer
lists the `.bak`. hah-6: markdown link/anchor spot-check of each fixed
pointer.
