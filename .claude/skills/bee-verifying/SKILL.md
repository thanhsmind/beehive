---
name: bee-verifying
description: "Generate a project-local verify-app skill that drives the real product like a user and prove it end to end. Use ONCE per repo with no way to prove behavior: no test command, or a request for a drive harness or a way to run the app. Not for auditing one (bee-verify-upkeep)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-verifying
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show bee-verifying --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `bee-verifying` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
