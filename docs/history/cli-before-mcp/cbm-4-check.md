# cbm-4 check: the CLI-before-MCP doctrine rule

Checked commit 8073461ca against CONTEXT.md D1 to D3.

- PASS — the `agents-cli-before-mcp` block matches D1 to D3 and contradicts no other rule: one home, CLI first, MCP fallback, waggledance on its own rule (D1) at `packages/bee/LEADER.md:317-340`; the three mcp-gateway verbs, the non-read-only yes and the `outcome_unknown` stop (D2) at `packages/bee/LEADER.md:321-324,333-337`; `skill show --json`, the `active` condition, the `skill_get` cases and `skill_resolve`/`skill_feedback` on MCP (D3) at `packages/bee/LEADER.md:325-331,333-335`. The only other MCP rule, `agents-other-project-to-its-leader` (`packages/bee/LEADER.md:293-316`), uses the same fallback conditions and is named, not overridden. The block adds "the call fails with a tool error" to D2's fallback list; D3's "the CLI fails" and the waggledance rule cover the same case, so this is not a contradiction.
- PASS — the index row and the concept cite D-numbers only: `router-triage-and-the-agents-md-duplication-boundary.md:250-253` cites `cli-before-mcp D1, D2, D3`; `skillstation-hub-skills-and-aliases.md:48,105,108` cite `cli-before-mcp D3, D4` and `D4`. No store id (bf107c86, 27015a53, 17412392, cede1997, 069eed40, 73715451) appears in `docs/knowledge` or `packages/bee/LEADER.md`.
- PASS — every rendered copy matches its source: the rule block in `.bee/LEADER.md` is byte-identical to `packages/bee/LEADER.md`, and `bee dev release-manifest --check` is clean.

Verify (`.bee/bin/bee knowledge check && .bee/bin/bee dev release-manifest --check`), both exit 0:

- knowledge check: 328 concepts in 349 files, 0 OKF errors, 0 profile errors, 600 profile warnings — OK
- release-manifest --check: 950 files match the stored manifest, 1 unhashed artifact present
