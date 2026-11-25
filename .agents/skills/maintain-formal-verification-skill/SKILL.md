---
name: maintain-formal-verification-skill
description: "Bring a project's formal verification back in line with the code after the code changes: find the commits that touched verified code, update the model, harness, or properties to describe the code as it is now, rerun, and say whether each failure is drift in the verification or a bug in the code. Use for /maintain-formal-verification-skill, \"is the model still true to the code\", or after merges and refactors that touched verified code without rechecking it."
disable-model-invocation: true
metadata:
  skillstation_alias: maintain-formal-verification-skill
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show maintain-formal-verification-skill --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `maintain-formal-verification-skill` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
