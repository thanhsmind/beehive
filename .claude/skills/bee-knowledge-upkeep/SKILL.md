---
name: bee-knowledge-upkeep
description: "Periodic audit that keeps an existing docs/knowledge/ bundle honest. Use periodically, when a bundle exists: after a release, when a concept was reported wrong, grades orphan, or loads often unused. Not for creating a bundle (bee-knowledge-setup) or recording one fact (bee-capturing)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-knowledge-upkeep
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-knowledge-upkeep --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-knowledge-upkeep` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
