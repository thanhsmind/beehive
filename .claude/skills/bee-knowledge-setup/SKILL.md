---
name: bee-knowledge-setup
description: "Build a project's docs/knowledge/ bundle from the tree, once. Use when the repo has no docs/knowledge/ bundle and the user asks to set up, create or bootstrap the knowledge base (kho tri thức). Not for auditing an existing bundle (bee-knowledge-upkeep) or recording one fact (bee-capturing)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-knowledge-setup
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-knowledge-setup --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-knowledge-setup` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
