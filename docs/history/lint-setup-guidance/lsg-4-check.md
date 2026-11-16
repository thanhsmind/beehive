# lsg-4 check of lsg-2 (commit d8ba60b49)

- PASS — the section names `bee config set --key commands.test --value '["<lint>","<typecheck>","<tests>"]'` (skills/bee-hive/references/onboarding.md:123) and says the set-up request is the confirmation (skills/bee-hive/references/onboarding.md:116); SKILL.md loads it on a set-up ask (skills/bee-hive/SKILL.md:128).
- PASS — the section says "Never hand-edit `.bee/config.json`" (skills/bee-hive/references/onboarding.md:129) and lists `team`, `herding.agents`, `herding.agent_command` and `hooks` as the user's (skills/bee-hive/references/onboarding.md:130); no line of the section tells the agent to write the file by hand.
- PASS — release-manifest --check is clean; `references/onboarding.md` and `SKILL.md` are byte-identical in .claude, .agents, .opencode, .claude-plugin and .codex-plugin; the only differing file, `references/gates-and-delegation.md`, differs by its `bee:only` per-harness blocks, which is the renderer's job.
- PASS — the Static checks section cites lint-setup-guidance D1 and D2 and points to the "Set up the project's checks" section of `skills/bee-hive/references/onboarding.md` (docs/knowledge/areas/onboarding/host-project-artifacts.md:326).

D3 also holds: no linter means name the stack's usual one and add it as a dev dependency after the user agrees (skills/bee-hive/references/onboarding.md:122).

## Verify

`.bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check`, exit 0:

- knowledge check: 328 concept(s) in 349 file(s), 0 OKF error(s), 0 profile error(s), 600 profile warning(s) — OK
- release_manifest --check: 950 file(s) match stored manifest, 1 unhashed artifact(s) present
