---
name: bee-knowledge-setup
description: "Build a project's docs/knowledge/ bundle from the tree, once. Use when the repo has no docs/knowledge/ bundle and the user asks to set up, create or bootstrap the knowledge base (kho tri thức). Not for auditing an existing bundle (bee-knowledge-upkeep) or recording one fact (bee-capturing)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-knowledge-setup
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-knowledge-setup`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
