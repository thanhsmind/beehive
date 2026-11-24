---
name: bee-how
description: "Answer how a part of this repo actually runs, as a path of steps a reader can open. Use when the user types /bee-how or asks how a command, hook, flow or subsystem runs, which layer owns it, or for a walkthrough before a change. Not for why (bee-why), teaching (bee-teach), or changing code."
metadata:
  skillstation_alias: bee-how
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-how --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-how` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
