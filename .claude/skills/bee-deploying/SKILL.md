---
name: bee-deploying
description: "Learn, record and replay how one host project deploys, stopping before any live step. Use when the user says deploy, release or ship for their project, or asks to record how it deploys. Not for the bee repo's own release, which scripts/release.sh owns."
metadata:
  skillstation_alias: bee-deploying
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-deploying --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-deploying` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
