---
name: bee-knowledge-upkeep
description: "Periodic audit that keeps an existing docs/knowledge/ bundle honest. Use periodically, when a bundle exists: after a release, when a concept was reported wrong, grades orphan, or loads often unused. Not for creating a bundle (bee-knowledge-setup) or recording one fact (bee-capturing)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-knowledge-upkeep
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-knowledge-upkeep`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
