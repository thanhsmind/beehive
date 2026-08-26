# agent-model-unpin — plan

Shape: one slice, two cells. Cites CONTEXT.md D1 (no frontmatter pin,
dispatch param is the authority), D2 (files survive a herded generation),
D3 (opencode unchanged).

## Cell amu-1 — templates + render plan (code)

- `packages/bee/agents/{bee-build,bee-gather,bee-extract,bee-review}.md.tmpl`:
  delete the `model: {{TIER_MODEL}}` frontmatter line.
- `onboard/agents.rs` `compute_agent_file_plan`: for the Claude root, render
  every template named in `AGENT_ROLES_BY_NAME` unconditionally
  (byte-compare, sync on diff); drop the resolve-to-None removal arm.
  `resolve_agent_model` stays for the opencode plan, which is untouched (D3).
- Tests in `onboard/agents.rs`/`onboard/tests.rs`: a herded generation slot
  still renders `bee-build.md`; the rendered file carries no `model:` line.

## Cell amu-2 — drift validator flip (code, deps: amu-1)

- `status_full/store.rs` `validate_agent_files_drift`: Claude root — a
  present `model:` frontmatter line IS the drift ("legacy pinned file —
  re-render"); no line is correct; the `agent-file-malformed` no-model arm
  goes away for Claude. Opencode arm byte-for-byte unchanged.
- Tests in `status_full/tests.rs`: pinned Claude file flags; unpinned file
  is clean under a herded generation; opencode expectations unchanged.

## Verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee`
filtered to `onboard` and `status_full` suites, plus `bee dev regen` leaving
a clean tree with the three agent files re-rendered (no model line) while
`models.claude.generation` stays `{kind: herding, agent: agy-flash}`.

## Cost if wrong

Rendered agent files or drift warnings misreport; no runtime dispatch path
changes — `dispatch prepare` payloads are untouched.
