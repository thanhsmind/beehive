# agent-model-unpin — locked context

Feature: remove the `model:` frontmatter pin from Claude agent files; the
dispatch door's `model` param is the one model authority.

## Locked decisions

- **D1 (user, 2026-08-26):** The Claude agent templates must not pin a model
  in frontmatter. Model selection is unified through `bee dispatch prepare`,
  whose payload already carries the `model` param — and that param overrides
  agent-file frontmatter in the Claude harness. "nên xem lại template đó
  không nên fix model vậy mà nên chuyển qua dùng thống nhất như cách hiện
  tại đang làm."
- **D2 (user, 2026-08-26):** `models.claude.generation` stays routed to the
  agy-flash herd — the earlier change meant "change the model generation
  uses", not "rename or retire the role". The agent files must survive that
  routing: a herded slot removing `bee-build.md` broke native dispatch for
  the still-native `code`/`test` roles.
- **D3 (scope):** Claude side only. On opencode the rendered agent file IS
  the model enforcement (no dispatch model param — known open gap), so the
  opencode render path and its pin stay unchanged.

## Consequences

- `compute_agent_file_plan` (Claude) renders every known agent template
  unconditionally; a slot's shape (herding/cli/null) no longer removes the
  file.
- `validate_agent_files_drift` flips for Claude: a file that still carries a
  `model:` line is the drift (legacy pin — re-render); a file without one is
  correct. Opencode keeps the existing expected-model comparison.
- Templates `packages/bee/agents/*.md.tmpl` drop the `model: {{TIER_MODEL}}`
  line.
