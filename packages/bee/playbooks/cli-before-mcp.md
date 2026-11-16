# CLI before MCP

Read when: calling `mcp-gateway` or `skillstation`.

<!-- rule: agents-cli-before-mcp -->
- A tool that ships a CLI and MCP tools is reached through its CLI
  first and the MCP tool is the fallback, because the CLI runs the
  same handler and puts far less text in context.
  - mcp-gateway: `mcp-gateway tools <keywords>` (MCP `search_tools`),
    `mcp-gateway describe <upstream.tool>` (`get_tool_definition`),
    `mcp-gateway call [--text] <upstream.tool> '<json>'` (`call_tool`).
    A tool that is not read-only still needs the user's yes.
  - Skillstation: `skillstation skill show <id> --json` loads a hub
    skill (MCP `skill_get`) — follow its `content` field, only when
    `manifest.status` is `active`. Call `skill_get` when the content
    sends you to a relative file of the skill (the write guard refuses
    reads inside the hub folder, and `skill_get` copies the skill to a
    readable cache), when the status is not `active`, or when the hub
    is on another machine. `skill_resolve` and `skill_feedback` have no
    CLI verb and stay on MCP.
  - Fallback: take the MCP tool when the binary is not on PATH, the
    verb is unknown (exit 2), a guard or permission refuses the shell
    call, or the call fails with a tool error. After `mcp-gateway call`
    exits 1 with code `outcome_unknown`, never repeat it through
    `call_tool`: read the system state first. A subagent with no shell
    uses the MCP tool. waggledance has its own mapping in rule
    `agents-other-project-to-its-leader`.
<!-- /rule -->
