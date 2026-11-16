---
name: bee-writing-skills
description: "Build and pressure-test bee skills with the TDD-for-skills discipline. Use when creating a new bee skill, editing an existing one, or verifying a skill holds up under pressure. Do NOT use for project-specific AGENTS.md conventions or one-off instructions."
metadata:
  skillstation_alias: bee-writing-skills
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-writing-skills --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-writing-skills` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
