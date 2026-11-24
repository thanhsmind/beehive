---
name: bee-verify-upkeep
description: "Periodic audit of an existing project-local verify-app skill and its feature map. Use when a repo already has a drive harness and it may have drifted: after a release, when drive recipes stop matching the app, or on request to audit it. Not for creating one (bee-verifying)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-verify-upkeep
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-verify-upkeep --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-verify-upkeep` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
