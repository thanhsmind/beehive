# gis-1t check: skill projection staging and shell write heads in the write guard (issues 163, 165, 168)

Checked: cell gis-1, commit 0f3596640, decisions D1 (719d1a38) and D2 (681f1ed6). The tester did not use the gis-1 worker's context.

## Reproduction

The parent commit is 0f3596640^. On it:

- `git add -u -- .agents/skills .claude/skills .opencode/skills` at idle is refused with the issue 163/168 message: "bee intake gate: no bee work is active (phase: idle) — writing \".agents/skills\" is blocked."
- `extract_bash_targets("claude rm 4e1731f9")` returns `["4e1731f9"]`, so the worktree-first guard refuses `claude rm 4e1731f9` (exit 2), the issue 165 symptom.

On the fixed code both commands are allowed (exit 0) and `claude rm 4e1731f9` yields no target. The fix: `SKILL_PROJECTION_PREFIXES` and `skill_projection_path` at `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:46`, the staging exemption at `guards.rs:911`, the intake commit exemption at `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:910`, and the segment-head rule at `guards.rs:783` and `guards.rs:988`.

## Truths

| Truth | Result | Anchor |
|---|---|---|
| git add of the three skill projection trees passes the idle intake gate and the worktree-first guard | PASS | `tests.rs:3134` (idle intake), `tests.rs:3165` (worktree-first), `tests.rs:5071` (no target); tester edge `tests.rs:5091` (`claude rm 4e1731f9 && git add .claude/skills` allowed at idle) |
| a Write or Edit into .claude/skills is still gated as before | PASS | `tests.rs:3156` (Edit and `echo x >` refused at idle), `tests.rs:3165` (Edit and `echo x >` refused by worktree-first) |
| claude rm 4e1731f9 adds no write target | PASS | `tests.rs:3195`, `tests.rs:5047`; tester edge `tests.rs:5091` (`claude rm 4e1731f9 --force`, `claude mv a b`, `git commit -m 'rm src/x'` add no target; `claude rm 4e1731f9 && rm src/z` yields only `src/z`) |
| rm, mv, cp, mkdir, touch and tee at a segment head are still write targets | PASS | `tests.rs:5047` (after `|`, `&&`, `;`, `||`, `sudo`, `env`, `FOO=1`), tester edge `tests.rs:5091` (after `then`, `do`, `command`, `nohup`, `time`, `!`, inside `sh -c '...'`, `/usr/bin/env -i FOO=1`, `sudo -u root`), and `git add .claude/skills && rm src/x` refused at idle |

All files in the table are under `packages/bee-rs/crates/bee/src/hooks/write_guard/`.

## Red on the parent commit

The tester built a detached worktree at 0f3596640^, applied the gis-1 test diff and the tester edge test, and kept the parent's guard code. `cargo test --release --no-fail-fast --bin bee -- hooks::write_guard` gave `355 passed; 6 failed`:

- `git_staging_skill_projection_trees_passes_the_idle_intake_gate` FAILED: the idle intake refusal for `.agents/skills`, quoted above
- `worktree_first_allows_staging_skill_projection_trees_but_not_writing_them` FAILED: `left: 2, right: 0`
- `claude_rm_with_a_short_id_is_not_a_delete_of_the_id` FAILED: `left: 2, right: 0`
- `extract_bash_targets_counts_a_write_verb_only_at_a_segment_head` FAILED: `"claude rm 4e1731f9" -> ["4e1731f9"]`
- `extract_bash_targets_git_staging_of_skill_projection_trees_adds_no_target` FAILED: `"git add -u -- .agents/skills .claude/skills .opencode/skills" -> [".agents/skills", ".claude/skills", ".opencode/skills"]`
- `tester_write_verbs_behind_shell_keywords_and_wrappers_stay_targets` FAILED: `"claude rm 4e1731f9 && rm src/z"` gave `["4e1731f9", "src/z"]`, expected `["src/z"]`

Truth 2 has its own test, `write_and_edit_into_skill_projection_trees_stay_gated_at_idle`, and it passes on the parent too. That is expected: truth 2 says the gating is unchanged, so its test guards against a regression and cannot be red on the parent. Truth 4 works the same way. Its head-position assertions held on the parent. They are red there only because the allow cases in the same functions fail first.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- hooks::write_guard`

Result: `test result: ok. 361 passed; 0 failed; 0 ignored; 0 measured; 5049 filtered out`. The run includes the six gis-1 tests and the tester test, each `... ok`: `git_staging_skill_projection_trees_passes_the_idle_intake_gate`, `write_and_edit_into_skill_projection_trees_stay_gated_at_idle`, `worktree_first_allows_staging_skill_projection_trees_but_not_writing_them`, `claude_rm_with_a_short_id_is_not_a_delete_of_the_id`, `extract_bash_targets_counts_a_write_verb_only_at_a_segment_head`, `extract_bash_targets_git_staging_of_skill_projection_trees_adds_no_target`, `tester_write_verbs_behind_shell_keywords_and_wrappers_stay_targets`.

## Found, out of scope

A write verb that another program runs as its argument no longer counts as a target. On the fixed code, `extract_bash_targets("xargs rm src/x")` and `extract_bash_targets("find . -exec rm {} +")` both return `[]`; the tester confirmed this with a temporary probe test and then removed it. D2 limits write verbs to a segment head, so this check records the behavior and does not count it as a failure.

The test module already contains `zz_probe4` at `tests.rs:6226`, a leftover probe test from commit 3c981ecae that prints instead of asserting. It predates this feature and was left in place.
