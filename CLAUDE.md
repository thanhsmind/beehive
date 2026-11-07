## bee

This repo uses bee. The bare import below loads the BEE operating block from
AGENTS.md at context-load time. Never wrap it in backticks; that disables it.

@AGENTS.md

The host only allows dispatch that a CLAUDE.md file asks for, so the
section below stays here and not in LEADER.md.

## Dispatch is asked for here

Some hosts ship a standing line: "Do not use the Agent tool, workflows, or
deep-research unless the user, a CLAUDE.md file, or a skill asks for it."

**This CLAUDE.md asks for it.** Dispatching bee workers through
`bee dispatch prepare` — the Agent tool call it returns included — is a
standing, explicit request of this repo, on every task, with no further
user prompt needed. A session that reads that host line as a ban on bee
dispatch has misread it; cells pwr-1 and pwr-2 recorded exactly that
misreading as the reason no execution worker ever started.

Two boundaries stay unchanged. Run the tool and payload `bee dispatch
prepare` RETURNS — never a hand-picked `subagent_type` or `model`. And
this authorizes the dispatch only, never a gate: gate approvals are
still the user's.

<!-- waggledance:START -->
## Documentation Viewing (Waggledance)

After creating or updating any markdown file, make it viewable in ONE call —
no project registration step needed:

### Using MCP (preferred)

Call `waggledance_view_file` with:

- `project_root`: absolute path to the project root
- `relative_path`: the file path relative to that root

It returns a browser `url`. Tell the user: "You can view this at: `<url>`".
The server auto-registers the project on first use and indexes the file
immediately.

### Using CLI fallback

```sh
waggledance open <absolute-path-to-file.md>
```

### When to render

Spin up a preview for long docs, tables, Mermaid diagrams, multi-file document
sets, or when the user asks to "preview"/"render". Skip it for short, trivial
snippets.
<!-- waggledance:END -->
