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
